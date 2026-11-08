//! Partial corroboration of the committed metadata for a pruning import.
//!
//! The caller must hold the pruning session, pruning point, and pruning
//! metadata guards while verifying and through any readiness publication.
//! The UTXO check corroborates rows against the committed pruning header.
//! Proof authentication and other state commitments remain separate checks.

use super::branch_readiness::ReadinessPoint;
use crate::model::stores::{
    headers::{DbHeadersStore, HeaderStoreReader},
    past_pruning_points::{DbPastPruningPointsStore, PastPruningPointsStoreReader},
    pruning::{DbPruningStore, PruningProofDescriptor, PruningStoreReader},
    utxo_set::DbUtxoSetStore,
};
use kaspa_consensus_core::{hashing, muhash::MuHashExtensions};
use kaspa_database::{
    prelude::{CachePolicy, CachedDbItem, DB, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
use kaspa_hashes::Hash;
use kaspa_muhash::MuHash;
use std::sync::Arc;

/// Evidence of metadata consistency only. No authority API accepts this as
/// verification of the import's UTXO, SMT, shielded, or pruning proof data.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct CommittedImportMetadata {
    pub(super) point: ReadinessPoint,
    retention: ReadinessPoint,
    descriptor: Arc<PruningProofDescriptor>,
}

impl CommittedImportMetadata {
    pub(super) fn verify(db: &Arc<DB>, point: ReadinessPoint, retention: ReadinessPoint) -> StoreResult<Self> {
        // Fresh readers never see values eagerly cached by a staging writer.
        let pruning = DbPruningStore::new(db.clone());
        let history = DbPastPruningPointsStore::new(db.clone(), CachePolicy::Empty);
        if pruning.pruning_point_and_index()? != (point.hash, point.index) || history.get(point.index)? != point.hash {
            return Err(inconsistent("committed pruning point does not match the import"));
        }
        if retention.index > point.index
            || pruning.retention_checkpoint()? != retention.hash
            || pruning.retention_period_root()? != retention.hash
            || history.get(retention.index)? != retention.hash
        {
            return Err(inconsistent("committed retention checkpoint does not match pruning history"));
        }

        let position = CachedDbItem::<Hash>::new(db.clone(), DatabaseStorePrefixes::PruningUtxosetPosition.into());
        if position.read()? != point.hash {
            return Err(inconsistent("committed pruning UTXO position does not match the import"));
        }

        for prefix in [
            DatabaseStorePrefixes::PruningUtxosetSyncFlag,
            DatabaseStorePrefixes::SmtSyncFlag,
            DatabaseStorePrefixes::ShieldedSyncFlag,
        ] {
            if !CachedDbItem::<bool>::new(db.clone(), prefix.into()).read()? {
                return Err(inconsistent("committed pruning import stability flag is false"));
            }
        }
        // This row tracks missing bodies, rather than the size of the anticone.
        let missing_bodies = CachedDbItem::<Vec<Hash>>::new(db.clone(), DatabaseStorePrefixes::BodyMissingAnticone.into());
        if !missing_bodies.read()?.is_empty() {
            return Err(inconsistent("pruning import still has missing anticone bodies"));
        }

        let descriptor = pruning.pruning_proof_descriptor()?;
        if descriptor.pruning_point != point.hash
            || descriptor.tips.first() != Some(&point.hash)
            || descriptor.roots.len() != descriptor.tips.len()
            || descriptor.counts.len() != descriptor.tips.len()
            || descriptor.counts.contains(&0)
        {
            return Err(inconsistent("committed pruning proof descriptor does not match the import"));
        }

        Ok(Self { point, retention, descriptor })
    }

    pub(super) fn verify_current(&self, db: &Arc<DB>) -> StoreResult<()> {
        if Self::verify(db, self.point, self.retention)? != *self {
            return Err(inconsistent("committed pruning import evidence changed after verification"));
        }
        Ok(())
    }
}

/// Corroborate the actual committed UTXO rows against the pruning header.
/// The caller must authenticate the header and hold the same import guards.
pub(super) fn verify_pruning_utxos(db: &Arc<DB>, point: ReadinessPoint) -> StoreResult<()> {
    let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
    let header = headers.get_header(point.hash)?;
    if hashing::header::hash(&header) != point.hash {
        return Err(inconsistent("committed pruning header does not hash to the import point"));
    }

    let utxos = DbUtxoSetStore::new(db.clone(), CachePolicy::Empty, DatabaseStorePrefixes::PruningUtxoset.into());
    let mut multiset = MuHash::new();
    for row in utxos.iterator() {
        let (outpoint, entry) = row.map_err(|error| StoreError::DataInconsistency(error.to_string()))?;
        multiset.combine(&MuHash::from_utxo(&outpoint, &entry));
    }
    if multiset.finalize() != header.utxo_commitment {
        return Err(inconsistent("committed pruning UTXO rows do not match the header commitment"));
    }
    Ok(())
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
pub(super) mod tests {
    use super::{
        super::{import_operation::ImportWorkerSession, persistent_branch_authority::PersistentBranchAuthority},
        *,
    };
    use crate::{
        config::ConfigBuilder,
        consensus::test_consensus::TestConsensus,
        model::stores::{
            headers::HeaderStore, past_pruning_points::PastPruningPointsStore, pruning::PruningStore, pruning_meta::PruningMetaStores,
        },
        params::DEVNET_PARAMS,
    };
    use kaspa_consensus_core::{
        config::{genesis::DEVNET_GENESIS, params::ForkActivation},
        header::Header,
        tx::{ScriptPublicKey, TransactionOutpoint, UtxoEntry},
    };
    use kaspa_database::{
        create_temp_db,
        prelude::{ConnBuilder, DeferredDbBatch},
    };
    use rocksdb::WriteBatch;

    fn point() -> ReadinessPoint {
        ReadinessPoint { hash: Hash::from_bytes([9; 32]), index: 3 }
    }

    fn retention() -> ReadinessPoint {
        ReadinessPoint { hash: Hash::from_bytes([7; 32]), index: 1 }
    }

    pub(in crate::pipeline::virtual_processor) fn install_point(db: &Arc<DB>, point: ReadinessPoint, retention: ReadinessPoint) {
        let mut pruning = DbPruningStore::new(db.clone());
        let history = DbPastPruningPointsStore::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        pruning.set_batch(&mut batch, point.hash, point.index).unwrap();
        pruning.set_retention_checkpoint(&mut batch, retention.hash).unwrap();
        pruning.set_retention_period_root(&mut batch, retention.hash).unwrap();
        db.write(batch).unwrap();
        history.insert(point.index, point.hash).unwrap();
        history.insert(retention.index, retention.hash).unwrap();
        let descriptor = PruningProofDescriptor::new(point.hash, vec![point.hash], vec![retention.hash], vec![2]);
        pruning.set_pruning_proof_descriptor(descriptor).unwrap();
    }

    pub(in crate::pipeline::virtual_processor) fn stage_explicit_evidence(
        metadata: &mut PruningMetaStores,
        batch: &mut WriteBatch,
        point: ReadinessPoint,
    ) {
        metadata.set_utxoset_position(batch, point.hash).unwrap();
        metadata.set_pruning_utxoset_stable_flag(batch, true).unwrap();
        metadata.set_pruning_smt_stable_flag(batch, true).unwrap();
        metadata.set_pruning_shielded_stable_flag(batch, true).unwrap();
        metadata.set_body_missing_anticone(batch, vec![]).unwrap();
    }

    #[test]
    fn rejects_legacy_defaults_and_staged_evidence() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        install_point(&db, point(), retention());
        let mut metadata = PruningMetaStores::new(db.clone(), CachePolicy::Empty);
        let mut position = WriteBatch::default();
        metadata.set_utxoset_position(&mut position, point().hash).unwrap();
        db.write(position).unwrap();

        assert!(metadata.pruning_utxoset_stable_flag());
        assert!(CommittedImportMetadata::verify(&db, point(), retention()).is_err());

        let mut staged = WriteBatch::default();
        stage_explicit_evidence(&mut metadata, &mut staged, point());
        assert!(CommittedImportMetadata::verify(&db, point(), retention()).is_err());
        db.write(staged).unwrap();
        assert_eq!(
            CommittedImportMetadata::verify(&db, point(), retention()).unwrap(),
            CommittedImportMetadata {
                point: point(),
                retention: retention(),
                descriptor: Arc::new(PruningProofDescriptor::new(point().hash, vec![point().hash], vec![retention().hash], vec![2])),
            }
        );
    }

    #[test]
    fn rejects_incomplete_anticone_and_unfinished_retention() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        install_point(&db, point(), retention());
        let mut metadata = PruningMetaStores::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        stage_explicit_evidence(&mut metadata, &mut batch, point());
        metadata.set_body_missing_anticone(&mut batch, vec![retention().hash]).unwrap();
        db.write(batch).unwrap();
        assert!(CommittedImportMetadata::verify(&db, point(), retention()).is_err());

        let mut batch = WriteBatch::default();
        metadata.set_body_missing_anticone(&mut batch, vec![]).unwrap();
        DbPruningStore::new(db.clone()).set_retention_checkpoint(&mut batch, point().hash).unwrap();
        db.write(batch).unwrap();
        assert!(CommittedImportMetadata::verify(&db, point(), retention()).is_err());
    }

    #[test]
    fn rejects_stale_proof_descriptor() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        install_point(&db, point(), retention());
        let mut metadata = PruningMetaStores::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        stage_explicit_evidence(&mut metadata, &mut batch, point());
        db.write(batch).unwrap();

        let stale = PruningProofDescriptor::new(retention().hash, vec![retention().hash], vec![retention().hash], vec![1]);
        DbPruningStore::new(db.clone()).set_pruning_proof_descriptor(stale).unwrap();
        assert!(CommittedImportMetadata::verify(&db, point(), retention()).is_err());
    }

    #[test]
    fn guarded_completion_checks_actual_utxo_rows_and_pins_metadata() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let config =
            ConfigBuilder::new(DEVNET_PARAMS).edit_consensus_params(|p| p.toccata_activation = ForkActivation::never()).build();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let mut utxos = DbUtxoSetStore::new(db.clone(), CachePolicy::Empty, DatabaseStorePrefixes::PruningUtxoset.into());
        let outpoint = TransactionOutpoint::new(Hash::from_bytes([8; 32]), 0);
        let entry = UtxoEntry::new(10, ScriptPublicKey::from_vec(0, vec![0x51]), 1, false, None);
        let mut header = Header::from(&DEVNET_GENESIS);
        header.utxo_commitment = MuHash::from_utxo(&outpoint, &entry).finalize();
        header.finalize();
        let point = ReadinessPoint { hash: header.hash, index: 3 };
        headers.insert(point.hash, Arc::new(header), 0).unwrap();
        install_point(&db, point, retention());
        processor.pruning_point_store.write().set(point.hash, point.index).unwrap();
        let mut metadata = PruningMetaStores::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        stage_explicit_evidence(&mut metadata, &mut batch, point);
        db.write(batch).unwrap();

        let owner = PersistentBranchAuthority::new(db.clone());
        let proof = super::super::import_proof_readiness::ValidatedImportProof::authenticate(
            &consensus,
            &Vec::new(),
            &kaspa_consensus_core::pruning::PruningProofMetadata::new(Default::default()),
        );
        assert!(proof.is_err());
        let token = owner.begin_import().unwrap();
        assert!(processor.finish_authority_import(&owner, token, retention(), |_, _| panic!("missing UTXOs accepted")).is_err());
        assert!(owner.lease(point).is_err());

        let token = owner.begin_import().unwrap();
        let mut session = ImportWorkerSession::new(token);
        let mut operation = session.try_start().unwrap();
        let unrelated = PersistentBranchAuthority::new(db.clone());
        let mut rejected_accumulator = MuHash::new();
        assert!(
            processor
                .append_authorized_pruning_utxos(&unrelated, &mut operation, &[(outpoint, entry.clone())], &mut rejected_accumulator)
                .is_err()
        );
        assert_eq!(rejected_accumulator.finalize(), MuHash::new().finalize());
        assert!(verify_pruning_utxos(&db, point).is_err());

        let mut accumulated = MuHash::new();
        let duplicate = [(outpoint, entry.clone()), (outpoint, entry.clone())];
        assert!(processor.append_authorized_pruning_utxos(&owner, &mut operation, &duplicate, &mut accumulated).is_err());
        assert_eq!(accumulated.finalize(), MuHash::new().finalize());
        assert!(verify_pruning_utxos(&db, point).is_err());

        let chunk = [(outpoint, entry.clone())];
        processor.append_authorized_pruning_utxos(&owner, &mut operation, &chunk, &mut accumulated).unwrap();
        assert!(processor.append_authorized_pruning_utxos(&owner, &mut operation, &chunk, &mut accumulated).is_err());
        assert_eq!(accumulated.finalize(), MuHash::from_utxo(&outpoint, &entry).finalize());
        drop(operation);
        let Ok(token) = session.try_finish() else {
            panic!("completed UTXO worker retained its operation");
        };
        processor
            .finish_authority_import(&owner, token, retention(), |_, actual| {
                assert_eq!(actual, point);
                assert!(processor.pruning_point_store.try_write().is_none());
                assert!(processor.pruning_meta_stores.try_write().is_none());
                Ok(())
            })
            .unwrap();
        let lease = processor.authority_lease(&owner).unwrap();
        let mut write = DeferredDbBatch::new(db.clone());
        let data_key = [DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, 1];
        write.put_raw(&db, data_key, b"committed").unwrap();
        processor.authority_commit(&owner, lease, write).unwrap();
        assert!(db.get(data_key).unwrap().is_some());

        utxos.clear().unwrap();
        let token = owner.begin_import().unwrap();
        assert!(processor.finish_authority_import(&owner, token, retention(), |_, _| panic!("removed UTXOs accepted")).is_err());
        assert!(owner.lease(point).is_err());
    }
}
