//! Evidence from the existing pruning proof validator for inactive import completion.

use super::branch_readiness::ReadinessPoint;
#[cfg(test)]
use crate::consensus::Consensus;
use crate::model::stores::{
    headers::{DbHeadersStore, HeaderStoreReader},
    pruning::{DbPruningStore, PruningProofDescriptor, PruningStoreReader},
};
use kaspa_consensus_core::{BlockHashSet, api::ValidatedStagingPruningProof, hashing, pruning::PruningPointProof};
#[cfg(test)]
use kaspa_consensus_core::{api::ConsensusApi, pruning::PruningProofMetadata};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use std::sync::Arc;

/// Created by validating against the defending consensus before acquiring the
/// receiver's pruning guards. An imported descriptor alone cannot mint it.
#[derive(Clone)]
pub(super) struct ValidatedImportProof {
    descriptor: PruningProofDescriptor,
    headers: Arc<BlockHashSet>,
}

impl ValidatedImportProof {
    #[cfg(test)]
    pub(super) fn authenticate(defender: &Consensus, proof: &PruningPointProof, metadata: &PruningProofMetadata) -> StoreResult<Self> {
        let evidence = Self::capture_validated(proof)?;
        defender.validate_pruning_proof(proof, metadata).map_err(|err| StoreError::DataInconsistency(err.to_string()))?;
        Ok(evidence)
    }

    pub(super) fn from_receipt(receipt: &ValidatedStagingPruningProof) -> StoreResult<Self> {
        Self::capture_validated(receipt.proof())
    }

    pub(super) fn pruning_point(&self) -> kaspa_hashes::Hash {
        self.descriptor.pruning_point
    }

    pub(super) fn capture_validated(proof: &PruningPointProof) -> StoreResult<Self> {
        let point = proof.first().and_then(|level| level.last()).ok_or_else(|| inconsistent("empty pruning proof"))?.hash;
        if proof.iter().any(Vec::is_empty) {
            return Err(inconsistent("pruning proof has an empty level"));
        }
        let mut headers = BlockHashSet::default();
        for header in proof.iter().flatten() {
            if hashing::header::hash(header) != header.hash {
                return Err(inconsistent("pruning proof header does not match its hash"));
            }
            headers.insert(header.hash);
        }
        Ok(Self { descriptor: PruningProofDescriptor::from_proof(proof, point, true), headers: Arc::new(headers) })
    }

    /// Corroborate the authenticated proof against fresh receiver DB reads.
    /// The receiver must retain the pruning guards through READY publication.
    pub(super) fn verify_committed(&self, db: &Arc<DB>, point: ReadinessPoint) -> StoreResult<()> {
        let actual = DbPruningStore::new(db.clone()).pruning_proof_descriptor()?;
        if point.hash != self.descriptor.pruning_point
            || actual.pruning_point != self.descriptor.pruning_point
            || actual.external != self.descriptor.external
            || actual.tips != self.descriptor.tips
            || actual.roots != self.descriptor.roots
            || actual.counts != self.descriptor.counts
        {
            return Err(inconsistent("committed pruning proof descriptor differs from the authenticated proof"));
        }

        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        for hash in self.headers.iter() {
            let header = headers.get_header(*hash)?;
            if header.hash != *hash || hashing::header::hash(&header) != *hash {
                return Err(inconsistent("committed proof header does not match its authenticated hash"));
            }
        }
        Ok(())
    }
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        consensus::test_consensus::TestConsensus,
        model::stores::{headers::HeaderStore, pruning::PruningStore},
    };
    use kaspa_consensus_core::{
        block::Block,
        config::{ConfigBuilder, genesis::DEVNET_GENESIS, params::DEVNET_PARAMS},
        header::Header,
    };
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};
    use kaspa_hashes::Hash;

    #[test]
    fn authenticated_completion_rejects_missing_committed_import_metadata() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &ConfigBuilder::new(DEVNET_PARAMS).build(), sender);
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let point = ReadinessPoint { hash: header.hash, index: 3 };
        processor.pruning_point_store.write().set(point.hash, point.index).unwrap();

        let proof = vec![vec![Arc::new(header.clone())]];
        let proof = ValidatedImportProof::capture_validated(&proof).unwrap();
        let child = Block::new(header, vec![]);
        let token = processor.begin_authority_import().unwrap();
        assert!(processor.finish_authenticated_authority_import(token, point, &proof, &child).is_err());
        assert!(processor.authority_lease().is_err());
    }

    #[test]
    fn matching_descriptor_cannot_substitute_for_committed_proof_headers() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let mut pruning = DbPruningStore::new(db.clone());
        let mut root = Header::from(&DEVNET_GENESIS);
        root.finalize();
        let mut middle = root.clone();
        middle.nonce += 1;
        middle.finalize();
        let mut tip = middle.clone();
        tip.nonce += 1;
        tip.finalize();

        let proof = vec![vec![Arc::new(root), Arc::new(middle), Arc::new(tip)]];
        let validated = ValidatedImportProof::capture_validated(&proof).unwrap();
        let point = ReadinessPoint { hash: proof[0][2].hash, index: 3 };

        headers.insert(proof[0][0].hash, proof[0][0].clone(), 0).unwrap();
        headers.insert(point.hash, proof[0][2].clone(), 0).unwrap();
        pruning.set_pruning_proof_descriptor(validated.descriptor.clone()).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());

        headers.insert(proof[0][1].hash, proof[0][1].clone(), 0).unwrap();
        validated.verify_committed(&db, point).unwrap();
        let mut substituted = validated.descriptor.clone();
        substituted.counts[0] += 1;
        pruning.set_pruning_proof_descriptor(substituted).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());
    }

    fn install_shielded_child(db: &Arc<DB>, parent: Hash, nonce: u64) -> Block {
        use crate::{
            model::stores::{
                ghostdag::{DbGhostdagStore, GhostdagData, GhostdagStore},
                reachability::{DbReachabilityStore, ReachabilityStore},
            },
            processes::{reachability::interval::Interval, shielded::PruningPointShieldedMetadata},
        };
        use kaspa_consensus_core::{
            merkle::calc_hash_merkle_root, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction,
            zkas_state_binding::SHIELDED_COMMITMENT_OFFSET,
        };

        let mut payload = vec![0; SHIELDED_COMMITMENT_OFFSET];
        payload.extend_from_slice(&PruningPointShieldedMetadata::empty_state_root());
        payload.extend_from_slice(&[0; 3]);
        let coinbase = Transaction::new(0, vec![], vec![], 0, SUBNETWORK_ID_COINBASE, 0, payload);
        let mut header = Header::from(&DEVNET_GENESIS);
        header.nonce = nonce;
        header.hash_merkle_root = calc_hash_merkle_root(std::iter::once(&coinbase));
        header.finalize();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, Arc::new(header.clone()), 0)
            .unwrap();
        DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, Arc::new(GhostdagData::new_with_selected_parent(parent, 5)))
            .unwrap();
        DbReachabilityStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, parent, Interval::new(1, 10), nonce)
            .unwrap();
        Block::new(header, vec![coinbase])
    }

    #[test]
    fn authority_integration_reopen_before_promotion_rejects_incomplete_evidence() {
        use super::super::{
            branch_readiness::DbBranchReadiness,
            import_readiness::tests::{install_point, stage_explicit_evidence},
            staging_import::PROMOTION_KEY,
        };
        use crate::model::stores::{
            ghostdag::GhostdagData,
            headers_selected_tip::{DbHeadersSelectedTipStore, HeadersSelectedTipStore},
            selected_chain::{DbSelectedChainStore, SelectedChainStore},
            statuses::{DbStatusesStore, StatusesStore},
            virtual_state::{VirtualState, VirtualStateStore},
        };
        use crate::processes::ghostdag::ordering::SortableBlock;
        use kaspa_consensus_core::{blockstatus::BlockStatus::StatusUTXOValid, config::params::ForkActivation};
        use rocksdb::WriteBatch;

        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        let config = ConfigBuilder::new(DEVNET_PARAMS)
            .edit_consensus_params(|params| params.toccata_activation = ForkActivation::never())
            .skip_adding_genesis()
            .build();
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender.clone());
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let point = ReadinessPoint { hash: header.hash, index: 0 };
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(header.clone())]]).unwrap();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty).insert(point.hash, Arc::new(header), 0).unwrap();
        install_point(&db, point, point);
        DbPruningStore::new(db.clone()).set_pruning_proof_descriptor(proof.descriptor.clone()).unwrap();
        let child = install_shielded_child(&db, point.hash, 201);
        DbHeadersSelectedTipStore::new(db.clone()).set(SortableBlock::new(child.hash(), child.header.blue_work)).unwrap();
        DbStatusesStore::new(db.clone(), CachePolicy::Empty).set(point.hash, StatusUTXOValid).unwrap();
        let mut batch = WriteBatch::default();
        DbSelectedChainStore::new(db.clone(), CachePolicy::Empty).init_with_pruning_point(&mut batch, point.hash).unwrap();
        stage_explicit_evidence(&mut processor.pruning_meta_stores.write(), &mut batch, point);
        db.write(batch).unwrap();
        processor
            .virtual_stores
            .write()
            .state
            .set(Arc::new(VirtualState {
                parents: vec![point.hash],
                ghostdag_data: GhostdagData::new_with_selected_parent(point.hash, 5),
                ..Default::default()
            }))
            .unwrap();

        let token = processor.begin_authority_import().unwrap();
        let unspent = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        drop(unspent);
        let mut batch = WriteBatch::default();
        processor.pruning_meta_stores.write().set_pruning_smt_stable_flag(&mut batch, false).unwrap();
        db.write(batch).unwrap();
        drop(consensus);
        drop(db);

        let db = connect();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        assert!(processor.authority_lease().is_err());
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        assert!(processor.recover_authenticated_staging(point, &proof, &child, &child).is_err());
        assert!(!DbBranchReadiness::new(db.clone()).is_ready(point).unwrap());

        let mut batch = WriteBatch::default();
        processor.pruning_meta_stores.write().set_pruning_smt_stable_flag(&mut batch, true).unwrap();
        db.write(batch).unwrap();
        assert!(processor.authority_lease().is_err());
        let recovered = processor.recover_authenticated_staging(point, &proof, &child, &child).unwrap();
        processor.commit_staging_promotion_checked(recovered, &proof, &child, true).unwrap();
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
    }

    #[test]
    fn authority_integration_promotes_and_recovers_a_committed_descendant_chain() {
        use super::super::{
            branch_readiness::DbBranchReadiness,
            import_readiness::tests::install_point,
            staging_import::{PROMOTION_KEY, StagingComponent, StagingImport},
        };
        use crate::{
            model::stores::{
                ghostdag::GhostdagData,
                headers_selected_tip::{DbHeadersSelectedTipStore, HeadersSelectedTipStore},
                selected_chain::{DbSelectedChainStore, SelectedChainStore},
                shielded::DbNullifierDiffStore,
                statuses::{DbStatusesStore, StatusesStore},
                virtual_state::{VirtualState, VirtualStateStore},
            },
            processes::ghostdag::ordering::SortableBlock,
        };
        use kaspa_consensus_core::{
            api::{StagingPromotion, StagingPromotionError},
            blockstatus::BlockStatus::StatusUTXOValid,
            config::params::ForkActivation,
            muhash::MuHashExtensions,
            tx::{ScriptPublicKey, TransactionOutpoint, UtxoEntry},
        };
        use kaspa_database::{prelude::DeferredDbBatch, registry::DatabaseStorePrefixes as Prefix};
        use kaspa_muhash::MuHash;
        use rocksdb::WriteBatch;

        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        let config = ConfigBuilder::new(DEVNET_PARAMS)
            .edit_consensus_params(|params| params.toccata_activation = ForkActivation::never())
            .skip_adding_genesis()
            .build();
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender.clone());
        let processor = consensus.virtual_processor();
        let mut point_header = Header::from(&DEVNET_GENESIS);
        point_header.nonce = 100;
        let outpoint = TransactionOutpoint::new(12.into(), 0);
        let entry = UtxoEntry::new(10, ScriptPublicKey::from_vec(0, vec![0x51]), 1, false, None);
        point_header.utxo_commitment = MuHash::from_utxo(&outpoint, &entry).finalize();
        point_header.finalize();
        let point = ReadinessPoint { hash: point_header.hash, index: 3 };
        let retention = ReadinessPoint { hash: 7.into(), index: 1 };
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(point_header.clone())]]).unwrap();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty).insert(point.hash, Arc::new(point_header), 0).unwrap();
        install_point(&db, point, retention);
        let child = install_shielded_child(&db, point.hash, 101);
        let mut header_tip = DbHeadersSelectedTipStore::new(db.clone());
        header_tip.set(SortableBlock::new(child.hash(), child.header.blue_work)).unwrap();

        let mut staging = StagingImport::begin_fixture(processor.clone(), proof.clone()).unwrap();
        assert!(staging.finish(retention, &child).is_err());
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err());
        let (_foreign_temp, foreign_db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let foreign = TestConsensus::with_db(foreign_db, &config, sender.clone());
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let components = [
            (
                StagingComponent::Proof,
                vec![
                    (Prefix::PruningProofDescriptor, bincode::serialize(&proof.descriptor).unwrap()),
                    (Prefix::BodyMissingAnticone, bincode::serialize(&Vec::<Hash>::new()).unwrap()),
                ],
            ),
            (StagingComponent::Smt, vec![(Prefix::SmtSyncFlag, bincode::serialize(&true).unwrap())]),
            (StagingComponent::Shielded, vec![(Prefix::ShieldedSyncFlag, bincode::serialize(&true).unwrap())]),
        ];
        for (component, rows) in components {
            let worker_db = db.clone();
            runtime.block_on(async {
                staging
                    .submit(component, true, move || {
                        let mut batch = DeferredDbBatch::new(worker_db.clone());
                        for (prefix, data) in rows {
                            batch.put_raw(&worker_db, prefix, data)?;
                        }
                        Ok(batch)
                    })
                    .unwrap()
                    .await
                    .unwrap()
                    .unwrap();
            });
            assert!(processor.authority_lease().is_err());
        }
        let chunk = vec![(outpoint, entry.clone())];
        assert!(foreign.stage_phase6_pruning_utxo_chunk(&mut staging, chunk.clone()).is_err());
        assert!(foreign.complete_phase6_pruning_utxos(&mut staging, point.hash).is_err());
        assert!(staging.submit(StagingComponent::Utxo, true, || panic!("generic UTXO batch started")).is_err());
        runtime.block_on(async { consensus.stage_phase6_pruning_utxo_chunk(&mut staging, chunk).unwrap().await }).unwrap().unwrap();
        assert!(staging.finish(retention, &child).is_err());
        runtime.block_on(async { consensus.complete_phase6_pruning_utxos(&mut staging, point.hash).unwrap().await }).unwrap().unwrap();
        let verified = staging.finish(retention, &child).unwrap();
        let sibling = processor.authority_lease().unwrap();

        assert!(matches!(
            consensus.authorize_staging_promotion(StagingPromotion::new(())),
            Err(StagingPromotionError::InvalidReceipt)
        ));
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        consensus.authorize_staging_promotion(verified.into_promotion()).unwrap();
        assert!(processor.commit_staging_promotion(sibling, &proof, &child).is_err());
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
        drop(staging);

        let token = processor.begin_authority_import().unwrap();
        let stale = processor.finish_authenticated_authority_import(token, retention, &proof, &child).unwrap();
        let mut metadata = processor.pruning_meta_stores.write();
        metadata.utxo_set.clear().unwrap();
        drop(metadata);
        assert!(processor.commit_staging_promotion(stale, &proof, &child).is_err());
        assert!(DbBranchReadiness::new(db.clone()).closed_revision().unwrap().is_some());
        let mut metadata = processor.pruning_meta_stores.write();
        metadata.utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry.clone()))]).unwrap();
        drop(metadata);
        assert!(processor.authority_lease().is_err());
        let token = processor.begin_authority_import().unwrap();
        let reverified = processor.finish_authenticated_authority_import(token, retention, &proof, &child).unwrap();
        processor.commit_staging_promotion(reverified, &proof, &child).unwrap();

        // The IBD receipt also requires the finalized virtual UTXO set.
        let write_retention = |hash| {
            let mut pruning = DbPruningStore::new(db.clone());
            let mut batch = WriteBatch::default();
            pruning.set_retention_checkpoint(&mut batch, hash).unwrap();
            pruning.set_retention_period_root(&mut batch, hash).unwrap();
            db.write(batch).unwrap();
        };
        write_retention(point.hash);
        DbStatusesStore::new(db.clone(), CachePolicy::Empty).set(point.hash, StatusUTXOValid).unwrap();
        let token = processor.begin_authority_import().unwrap();
        assert!(processor.finish_verified_ibd_import(token, &proof, &child).is_err());
        assert!(processor.authority_lease().is_err());

        let state = VirtualState {
            parents: vec![point.hash],
            ghostdag_data: GhostdagData::new_with_selected_parent(point.hash, 5),
            multiset: MuHash::from_utxo(&outpoint, &entry),
            ..Default::default()
        };
        let mut virtual_write = processor.virtual_stores.write();
        virtual_write.state.set(Arc::new(state)).unwrap();
        virtual_write.utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry.clone()))]).unwrap();
        drop(virtual_write);
        let token = processor.begin_authority_import().unwrap();
        let receipt = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        processor.virtual_stores.write().utxo_set.clear().unwrap();
        assert!(processor.commit_staging_promotion_checked(receipt, &proof, &child, true).is_err());
        assert!(processor.authority_lease().is_err());

        processor.virtual_stores.write().utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry))]).unwrap();
        let token = processor.begin_authority_import().unwrap();
        let receipt = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        processor.commit_staging_promotion_checked(receipt, &proof, &child, true).unwrap();

        write_retention(retention.hash);
        let tip_child = install_shielded_child(&db, child.hash(), 102);
        header_tip.set(SortableBlock::new(tip_child.hash(), tip_child.header.blue_work)).unwrap();
        let mut chain = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        chain.init_with_pruning_point(&mut batch, point.hash).unwrap();
        chain.write_entry(&mut batch, 1, child.hash()).unwrap();
        chain.set_highest_index(&mut batch, 1).unwrap();
        db.write(batch).unwrap();
        let state = VirtualState { ghostdag_data: GhostdagData::new_with_selected_parent(child.hash(), 5), ..Default::default() };
        processor.virtual_stores.write().state.set(Arc::new(state)).unwrap();
        drop(chain);
        drop(header_tip);
        drop(consensus);
        drop(db);

        let db = connect();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let readiness = DbBranchReadiness::new(db.clone());
        assert!(readiness.is_ready(point).unwrap());
        assert!(processor.authority_lease().is_err());
        let diffs = DbNullifierDiffStore::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        diffs.set_batch(&mut batch, child.hash(), vec![[9; 32]]).unwrap();
        db.write(batch).unwrap();
        assert!(processor.recover_authenticated_staging(retention, &proof, &child, &tip_child).is_err());
        assert!(!readiness.is_ready(point).unwrap());

        let mut batch = WriteBatch::default();
        diffs.delete_batch(&mut batch, child.hash()).unwrap();
        db.write(batch).unwrap();
        assert!(processor.recover_authenticated_staging(retention, &proof, &child, &tip_child).is_ok());
        assert!(processor.authority_lease().is_ok());
    }
}
