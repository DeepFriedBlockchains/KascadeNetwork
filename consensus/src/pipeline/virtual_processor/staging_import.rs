//! Inactive staging import coordination for a single processor-owned session.

#[cfg(test)]
use super::branch_readiness::ReadinessPoint;
use super::{
    VirtualStateProcessor,
    import_operation::{ImportOperation, ImportWorkerSession},
    import_proof_readiness::ValidatedImportProof,
    persistent_branch_authority::PersistentBranchLease,
};
use crate::consensus::Consensus;
use kaspa_consensus_core::{
    api::{
        ConsensusApi, StagingImportComponent, StagingImportSession, StagingPromotion, StagingPromotionError,
        ValidatedStagingPruningProof,
    },
    block::Block,
    header::Header,
    pruning::PruningPointsList,
    trusted::TrustedBlock,
    tx::{TransactionOutpoint, UtxoEntry},
};
#[cfg(test)]
use kaspa_database::prelude::DeferredDbBatch;
use kaspa_database::{
    prelude::{StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
#[cfg(test)]
use kaspa_hashes::Hash;
use kaspa_muhash::MuHash;
use parking_lot::Mutex;
use std::sync::Arc;
#[cfg(test)]
use tokio::task::JoinHandle;

pub(super) const PROMOTION_KEY: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, b'P'];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StagingComponent {
    Proof,
    Smt,
    Shielded,
    Utxo,
}

impl StagingComponent {
    fn next(self) -> StagingProgress {
        use StagingComponent::*;
        match self {
            Proof => StagingProgress::Writing(Smt),
            Smt => StagingProgress::Writing(Shielded),
            Shielded => StagingProgress::Writing(Utxo),
            Utxo => StagingProgress::Verify,
        }
    }

    pub(super) fn write_prefixes(self) -> &'static [DatabaseStorePrefixes] {
        use DatabaseStorePrefixes::*;
        match self {
            Self::Proof => &[
                Headers,
                HeadersCompact,
                CompressedHeaders,
                Ghostdag,
                GhostdagCompact,
                HeadersSelectedTip,
                PastPruningPoints,
                PruningPoint,
                RetentionCheckpoint,
                RetentionPeriodRoot,
                Reachability,
                ReachabilityReindexRoot,
                ReachabilityRelations,
                ReachabilityTreeChildren,
                ReachabilityFutureCoveringSet,
                RelationsParents,
                RelationsChildren,
                PruningProofDescriptor,
                BodyMissingAnticone,
                Statuses,
                BlockTransactions,
            ],
            Self::Smt => &[SmtBranchVersions, SmtLaneVersions, SmtScoreIndex, SmtSeqCommitMeta, SmtSyncFlag],
            Self::Shielded => &[
                ShieldedNullifiers,
                ShieldedTreeFrontier,
                ShieldedAnchors,
                ShieldedSupply,
                ShieldedNullifierMuHash,
                ShieldedSyncFlag,
                ShieldedBurns,
                ShieldedAnchorProducers,
                ShieldedDevAccrued,
                ShieldedAnchorSourceScore,
            ],
            Self::Utxo => &[],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StagingProgress {
    Writing(StagingComponent),
    Verify,
    Failed,
}

pub(super) struct StagingImport {
    processor: Arc<VirtualStateProcessor>,
    session: Option<ImportWorkerSession>,
    proof: ValidatedImportProof,
    unapplied_proof: Option<ValidatedStagingPruningProof>,
    progress: Arc<Mutex<StagingProgress>>,
}

/// Only authenticated completion can produce the lease for promotion.
pub(super) struct VerifiedStaging {
    processor: Arc<VirtualStateProcessor>,
    lease: PersistentBranchLease,
    proof: ValidatedImportProof,
    child: Block,
    verify_virtual: bool,
}

impl StagingImport {
    fn start_guarded_operation(&mut self, component: StagingComponent) -> StoreResult<ImportOperation> {
        if *self.progress.lock() != StagingProgress::Writing(component) {
            return Err(inconsistent("staging component is out of order or the session has failed"));
        }
        self.session
            .as_mut()
            .and_then(ImportWorkerSession::try_start)
            .ok_or_else(|| inconsistent("staging session has ended or a worker still holds its permit"))
    }

    fn begin(
        processor: Arc<VirtualStateProcessor>,
        proof: ValidatedImportProof,
        unapplied_proof: Option<ValidatedStagingPruningProof>,
    ) -> StoreResult<Self> {
        let session = ImportWorkerSession::new(processor.begin_authority_import()?);
        let progress = Arc::new(Mutex::new(StagingProgress::Writing(StagingComponent::Proof)));
        Ok(Self { processor, session: Some(session), proof, unapplied_proof, progress })
    }

    #[cfg(test)]
    pub(super) fn begin_fixture(processor: Arc<VirtualStateProcessor>, proof: ValidatedImportProof) -> StoreResult<Self> {
        Self::begin(processor, proof, None)
    }

    /// Build the component batch on the blocking worker. Each successful
    /// batch leaves durable readiness closed, including the final chunk.
    #[cfg(test)]
    pub(super) fn submit(
        &mut self,
        component: StagingComponent,
        last_chunk: bool,
        build: impl FnOnce() -> StoreResult<DeferredDbBatch<'static>> + Send + 'static,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if component == StagingComponent::Utxo {
            return Err(inconsistent("staging UTXO requires the typed import path"));
        }
        if self.unapplied_proof.is_some() {
            return Err(inconsistent("the authenticated staging proof has not been applied"));
        }
        self.submit_operation(component, last_chunk, move |processor, operation| {
            let prefixes: Vec<_> = component.write_prefixes().iter().copied().map(u8::from).collect();
            processor.apply_authorized_staging_batch(operation, &prefixes, build)
        })
    }

    #[cfg(test)]
    fn submit_operation(
        &mut self,
        component: StagingComponent,
        last_chunk: bool,
        apply: impl FnOnce(&VirtualStateProcessor, &mut ImportOperation) -> StoreResult<()> + Send + 'static,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if *self.progress.lock() != StagingProgress::Writing(component) {
            return Err(inconsistent("staging component is out of order or the session has failed"));
        }

        let processor = self.processor.clone();
        let progress = self.progress.clone();
        self.session
            .as_mut()
            .ok_or_else(|| inconsistent("staging session has ended"))?
            .spawn_blocking(move |operation| {
                // A build error, write error, or unwind aborts this staging attempt.
                *progress.lock() = StagingProgress::Failed;
                apply(&processor, operation)?;
                *progress.lock() = if last_chunk { component.next() } else { StagingProgress::Writing(component) };
                Ok(())
            })
            .ok_or_else(|| inconsistent("a staging worker still holds the import permit"))
    }

    #[cfg(test)]
    pub(super) async fn cancel(self) {
        if let Some(session) = self.session {
            session.cancel_and_wait().await;
        }
    }

    #[cfg(test)]
    pub(super) fn finish(&mut self, retention: ReadinessPoint, child: &Block) -> StoreResult<VerifiedStaging> {
        if *self.progress.lock() != StagingProgress::Verify {
            return Err(inconsistent("staging import is incomplete"));
        }
        let session = self.session.take().ok_or_else(|| inconsistent("staging session has ended"))?;
        let token = match session.try_finish() {
            Ok(token) => token,
            Err(session) => {
                self.session = Some(session);
                return Err(inconsistent("a staging worker still holds the import permit"));
            }
        };
        let lease = self.processor.finish_authenticated_authority_import(token, retention, &self.proof, child)?;
        Ok(VerifiedStaging {
            processor: self.processor.clone(),
            lease,
            proof: self.proof.clone(),
            child: child.clone(),
            verify_virtual: false,
        })
    }
}

impl Consensus {
    pub(crate) fn start_owned_staging_import(
        &self,
        receipt: ValidatedStagingPruningProof,
    ) -> Result<StagingImportSession, StagingPromotionError> {
        self.begin_phase6_staging_import(receipt).map(|import| StagingImportSession::new(Mutex::new(import))).map_err(rejected)
    }

    fn owned_staging_import<'a>(
        &self,
        session: &'a StagingImportSession,
    ) -> Result<parking_lot::MutexGuard<'a, StagingImport>, StagingPromotionError> {
        let import = session.downcast_ref::<Mutex<StagingImport>>()?.lock();
        if !Arc::ptr_eq(&self.virtual_processor, &import.processor) {
            return Err(StagingPromotionError::InvalidReceipt);
        }
        Ok(import)
    }

    pub(crate) fn apply_owned_staging_proof(
        &self,
        session: &StagingImportSession,
        trusted: &[TrustedBlock],
        header_only_chain_segment: &[Arc<Header>],
        pruning_points: PruningPointsList,
    ) -> Result<(), StagingPromotionError> {
        let mut import = self.owned_staging_import(session)?;
        if pruning_points.last().map(|header| header.hash) != Some(import.proof.pruning_point()) {
            return Err(rejected(inconsistent("pruning history does not match the authenticated staging proof")));
        }
        let mut operation = import.start_guarded_operation(StagingComponent::Proof).map_err(rejected)?;
        let receipt = import.unapplied_proof.take().ok_or_else(|| rejected(inconsistent("no unapplied staging proof")))?;
        *import.progress.lock() = StagingProgress::Failed;
        self.virtual_processor
            .apply_guarded_staging_proof(&mut operation, &import.proof, || {
                self.apply_staging_pruning_proof_and_history(receipt, trusted, header_only_chain_segment, pruning_points)
                    .map_err(|error| StoreError::DataInconsistency(error.to_string()))
            })
            .map_err(rejected)?;
        *import.progress.lock() = StagingProgress::Writing(StagingComponent::Proof);
        Ok(())
    }

    pub(crate) fn check_owned_staging_component(
        &self,
        session: &StagingImportSession,
        component: StagingImportComponent,
        complete: bool,
    ) -> Result<(), StagingPromotionError> {
        let mut import = self.owned_staging_import(session)?;
        let component = match component {
            StagingImportComponent::Proof => StagingComponent::Proof,
            StagingImportComponent::Smt => StagingComponent::Smt,
            StagingImportComponent::Shielded => StagingComponent::Shielded,
            StagingImportComponent::Utxo => StagingComponent::Utxo,
        };
        if import.unapplied_proof.is_some() {
            return Err(rejected(inconsistent("the authenticated staging proof has not been applied")));
        }
        let mut operation = import.start_guarded_operation(component).map_err(rejected)?;
        self.virtual_processor.check_staging_operation(&mut operation).map_err(rejected)?;
        let point = self.virtual_processor.staging_import_point().map_err(rejected)?;
        if point.hash != import.proof.pruning_point() {
            return Err(rejected(inconsistent("staging pruning point changed during import")));
        }
        if complete {
            *import.progress.lock() = component.next();
        }
        Ok(())
    }

    pub(crate) fn clear_owned_staging_shielded(
        &self,
        session: &StagingImportSession,
        point: kaspa_hashes::Hash,
    ) -> Result<(), StagingPromotionError> {
        let mut import = self.owned_staging_import(session)?;
        if import.proof.pruning_point() != point {
            return Err(rejected(inconsistent("shielded cleanup differs from the authenticated pruning point")));
        }
        let mut operation = import.start_guarded_operation(StagingComponent::Shielded).map_err(rejected)?;
        *import.progress.lock() = StagingProgress::Failed;
        self.virtual_processor
            .clear_bounded_pruning_shielded(point, |batch| {
                self.virtual_processor.commit_guarded_shielded_batch(&mut operation, batch)
            })
            .map_err(rejected)?;
        *import.progress.lock() = StagingProgress::Writing(StagingComponent::Shielded);
        Ok(())
    }

    pub(crate) fn import_owned_staging_shielded(
        &self,
        session: &StagingImportSession,
        point: kaspa_hashes::Hash,
        metadata: kaspa_consensus_core::api::ShieldedExportMetadata,
        expected_state_root: [u8; 32],
        batches: kaspa_consensus_core::api::ShieldedNullifierBatchIterator<'_>,
    ) -> Result<(), StagingPromotionError> {
        let mut import = self.owned_staging_import(session)?;
        if import.proof.pruning_point() != point {
            return Err(rejected(inconsistent("shielded import differs from the authenticated pruning point")));
        }
        let mut operation = import.start_guarded_operation(StagingComponent::Shielded).map_err(rejected)?;
        *import.progress.lock() = StagingProgress::Failed;
        self.virtual_processor
            .import_bounded_pruning_shielded(point, metadata, Some(expected_state_root), batches, |batch| {
                self.virtual_processor.commit_guarded_shielded_batch(&mut operation, batch)
            })
            .map_err(|error| rejected(inconsistent(&error)))?;
        *import.progress.lock() = StagingProgress::Writing(StagingComponent::Shielded);
        Ok(())
    }

    pub(crate) fn append_owned_staging_utxos(
        &self,
        session: &StagingImportSession,
        utxos: &[(TransactionOutpoint, UtxoEntry)],
        multiset: &mut MuHash,
    ) -> Result<(), StagingPromotionError> {
        let mut import = self.owned_staging_import(session)?;
        let mut operation = import.start_guarded_operation(StagingComponent::Utxo).map_err(rejected)?;
        self.virtual_processor.append_guarded_staging_utxos(&mut operation, utxos, multiset).map_err(rejected)
    }

    pub(crate) fn finish_owned_staging_import(
        &self,
        session: StagingImportSession,
        child: &Block,
    ) -> Result<StagingPromotion, StagingPromotionError> {
        let mut import = session.downcast::<Mutex<StagingImport>>()?.into_inner();
        if !Arc::ptr_eq(&self.virtual_processor, &import.processor) {
            return Err(StagingPromotionError::InvalidReceipt);
        }
        if *import.progress.lock() != StagingProgress::Verify {
            return Err(rejected(inconsistent("staging import is incomplete")));
        }
        let session = import.session.take().ok_or_else(|| rejected(inconsistent("staging session has ended")))?;
        let token = session.try_finish().map_err(|_| rejected(inconsistent("a staging worker still holds the import permit")))?;
        let lease = self.virtual_processor.finish_verified_ibd_import(token, &import.proof, child).map_err(rejected)?;
        Ok(VerifiedStaging { processor: import.processor, lease, proof: import.proof, child: child.clone(), verify_virtual: true }
            .into_promotion())
    }

    pub(super) fn begin_phase6_staging_import(&self, receipt: ValidatedStagingPruningProof) -> StoreResult<StagingImport> {
        let proof = ValidatedImportProof::from_receipt(&receipt)?;
        StagingImport::begin(self.virtual_processor.clone(), proof, Some(receipt))
    }

    /// Apply the bound proof and pruning history. Trusted-body processing
    /// and the remaining staging components still have to complete.
    #[cfg(test)]
    pub(super) fn stage_phase6_pruning_proof(
        self: &Arc<Self>,
        staging: &mut StagingImport,
        trusted: Vec<TrustedBlock>,
        header_only_chain_segment: Vec<Arc<Header>>,
        pruning_points: PruningPointsList,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if !Arc::ptr_eq(&self.virtual_processor, &staging.processor) {
            return Err(inconsistent("proof input does not belong to the staging processor"));
        }
        if pruning_points.last().map(|header| header.hash) != Some(staging.proof.pruning_point()) {
            return Err(inconsistent("pruning history does not match the authenticated staging proof"));
        }
        let receipt = staging.unapplied_proof.take().ok_or_else(|| inconsistent("no unapplied staging proof"))?;
        let proof = staging.proof.clone();
        let consensus = self.clone();
        let result = staging.submit_operation(StagingComponent::Proof, false, move |processor, operation| {
            processor.apply_authorized_pruning_proof(operation, &proof, || {
                consensus
                    .apply_staging_pruning_proof_and_history(receipt, &trusted, &header_only_chain_segment, pruning_points)
                    .map_err(|error| StoreError::DataInconsistency(error.to_string()))
            })
        });
        if result.is_err() {
            *staging.progress.lock() = StagingProgress::Failed;
        }
        result
    }

    pub(crate) fn authorize_phase6_staging_promotion(&self, promotion: StagingPromotion) -> Result<(), StagingPromotionError> {
        let verified = promotion.downcast::<VerifiedStaging>()?;
        if !Arc::ptr_eq(&self.virtual_processor, &verified.processor) {
            return Err(StagingPromotionError::InvalidReceipt);
        }
        self.virtual_processor
            .commit_staging_promotion_checked(verified.lease, &verified.proof, &verified.child, verified.verify_virtual)
            .map_err(|error| StagingPromotionError::Rejected(error.to_string()))
    }

    /// Inactive typed ingress for direct pruning UTXO chunks. The staging
    /// session remains the only path to the database write.
    #[cfg(test)]
    pub(super) fn stage_phase6_pruning_utxo_chunk(
        &self,
        staging: &mut StagingImport,
        utxos: Vec<(TransactionOutpoint, UtxoEntry)>,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if !Arc::ptr_eq(&self.virtual_processor, &staging.processor) {
            return Err(inconsistent("UTXO input does not belong to the staging processor"));
        }

        staging.submit_operation(StagingComponent::Utxo, false, move |processor, operation| {
            processor.append_authorized_pruning_utxos(operation, &utxos, &mut MuHash::new())
        })
    }

    #[cfg(test)]
    pub(super) fn complete_phase6_pruning_utxos(
        &self,
        staging: &mut StagingImport,
        point: Hash,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if !Arc::ptr_eq(&self.virtual_processor, &staging.processor) {
            return Err(inconsistent("UTXO completion does not belong to the staging processor"));
        }
        staging.submit_operation(StagingComponent::Utxo, true, move |processor, operation| {
            processor.complete_authorized_pruning_utxos(operation, point)
        })
    }
}

impl VerifiedStaging {
    pub(super) fn into_promotion(self) -> StagingPromotion {
        StagingPromotion::new(self)
    }
}

fn rejected(error: StoreError) -> StagingPromotionError {
    StagingPromotionError::Rejected(error.to_string())
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{consensus::test_consensus::TestConsensus, pipeline::virtual_processor::branch_readiness::DbBranchReadiness};
    use kaspa_consensus_core::{
        config::{
            ConfigBuilder,
            genesis::DEVNET_GENESIS,
            params::{DEVNET_PARAMS, ForkActivation},
        },
        pruning::PruningProofMetadata,
    };
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};
    use std::sync::mpsc;
    use tokio::sync::oneshot;

    #[test]
    fn authority_integration_pruning_proof_worker_checks_receiver_and_revision() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let config = ConfigBuilder::new(DEVNET_PARAMS)
            .edit_consensus_params(|params| params.toccata_activation = ForkActivation::never())
            .build();
        let (_defender_temp, defender_db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let defender = TestConsensus::with_db(defender_db, &config, sender.clone());
        let receiver_config = ConfigBuilder::new(config.params.clone()).skip_adding_genesis().build();
        let receiver = TestConsensus::with_db(db.clone(), &receiver_config, sender);
        let proof = defender.get_pruning_point_proof();
        let header = proof[0].last().unwrap().clone();
        let metadata = PruningProofMetadata::new(header.blue_work);
        let receipt = defender.validate_staging_pruning_proof((*proof).clone(), &metadata).unwrap();
        let mut staging = receiver.begin_phase6_staging_import(receipt).unwrap();
        let receipt = defender.validate_staging_pruning_proof((*proof).clone(), &metadata).unwrap();
        assert!(receiver.begin_phase6_staging_import(receipt).is_err());
        assert!(defender.stage_phase6_pruning_proof(&mut staging, vec![], vec![], vec![header.clone()]).is_err());
        assert!(receiver.stage_phase6_pruning_proof(&mut staging, vec![], vec![], vec![]).is_err());
        assert!(staging.submit(StagingComponent::Proof, true, || panic!("unbound proof writer started")).is_err());

        let before = db.get(DatabaseStorePrefixes::PruningProofDescriptor).unwrap();
        let readiness = DbBranchReadiness::new(db.clone());
        readiness.close_revision_sync().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let result = runtime.block_on(async {
            receiver.stage_phase6_pruning_proof(&mut staging, vec![], vec![], vec![header.clone()]).unwrap().await.unwrap()
        });
        assert!(result.is_err());
        assert_eq!(db.get(DatabaseStorePrefixes::PruningProofDescriptor).unwrap(), before);
        runtime.block_on(staging.cancel());

        let receipt = defender.validate_staging_pruning_proof((*proof).clone(), &metadata).unwrap();
        let mut staging = receiver.begin_phase6_staging_import(receipt).unwrap();
        runtime.block_on(async {
            receiver.stage_phase6_pruning_proof(&mut staging, vec![], vec![], vec![header]).unwrap().await.unwrap().unwrap();
        });
        let descriptor_before = db.get(DatabaseStorePrefixes::PruningProofDescriptor).unwrap().unwrap();
        let rebuilt = receiver.get_staging_pruning_point_proof();
        assert_eq!(rebuilt.len(), proof.len());
        for (actual, expected) in rebuilt.iter().zip(proof.iter()) {
            let actual: kaspa_consensus_core::BlockHashSet = actual.iter().map(|header| header.hash).collect();
            let expected: kaspa_consensus_core::BlockHashSet = expected.iter().map(|header| header.hash).collect();
            assert_eq!(actual, expected);
        }
        assert_eq!(db.get(DatabaseStorePrefixes::PruningProofDescriptor).unwrap().unwrap(), descriptor_before);
        let point = receiver.virtual_processor().staging_import_point().unwrap();
        staging.proof.verify_committed(&db, point).unwrap();
        assert!(readiness.closed_revision().unwrap().is_some());
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        runtime.block_on(staging.cancel());
    }

    #[test]
    fn authority_integration_rejects_partial_staging_and_drains_cancelled_worker() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let config = ConfigBuilder::new(DEVNET_PARAMS).skip_adding_genesis().build();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let anticone_before = db.get(DatabaseStorePrefixes::BodyMissingAnticone).unwrap();
        let shielded_before = db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(header)]]).unwrap();
        let mut staging = StagingImport::begin_fixture(processor.clone(), proof.clone()).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker_db = db.clone();
        runtime.block_on(async {
            let worker = staging
                .submit(StagingComponent::Proof, true, move || {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    let mut batch = DeferredDbBatch::new(worker_db.clone());
                    batch.put_raw(&worker_db, DatabaseStorePrefixes::BodyMissingAnticone, b"in-scope")?;
                    batch.put_raw(&worker_db, DatabaseStorePrefixes::ShieldedSyncFlag, b"out-of-scope")?;
                    Ok(batch)
                })
                .unwrap();
            started_rx.await.unwrap();
            drop(worker);
            assert!(staging.submit(StagingComponent::Proof, true, || panic!("overlapping worker launched")).is_err());
        });
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err());

        runtime.block_on(async {
            let mut drain = Box::pin(staging.cancel());
            tokio::select! {
                biased;
                () = &mut drain => panic!("staging cancellation completed before worker release"),
                () = std::future::ready(()) => {}
            }
            release_tx.send(()).unwrap();
            drain.await;
        });
        assert_eq!(db.get(DatabaseStorePrefixes::BodyMissingAnticone).unwrap(), anticone_before);
        assert_eq!(db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap(), shielded_before);
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        assert!(DbBranchReadiness::new(db).closed_revision().unwrap().is_some());
        assert!(StagingImport::begin_fixture(processor.clone(), proof).is_ok());
    }
    #[test]
    fn bounded_shielded_batches_reject_stale_authority_and_foreign_session() {
        use crate::model::stores::shielded::{DbNullifierSetStore, NullifierSetStoreReader};
        use crate::processes::shielded::PruningPointShieldedMetadata;
        use kaspa_consensus_core::api::ShieldedExportMetadata;
        use kaspa_database::prelude::CachePolicy;
        let config = ConfigBuilder::new(DEVNET_PARAMS).build();
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(header)]]).unwrap();
        let mut import = StagingImport::begin_fixture(processor.clone(), proof.clone()).unwrap();
        *import.progress.lock() = StagingProgress::Writing(StagingComponent::Shielded);
        let foreign = TestConsensus::new(&config);
        let session = StagingImportSession::new(Mutex::new(import));
        let metadata = PruningPointShieldedMetadata {
            frontier: Default::default(),
            supply: Default::default(),
            burns: Default::default(),
            nullifier_muhash: MuHash::new(),
            state_root: PruningPointShieldedMetadata::empty_state_root(),
            in_window_anchors: vec![],
            dev_accrued: 0,
            in_window_anchor_source_scores: vec![],
        };
        let wire = ShieldedExportMetadata { data: metadata.to_wire_bytes(), nullifier_count: 0 };
        let mut forbidden = std::iter::from_fn(|| -> Option<Vec<[u8; 32]>> { panic!("foreign session consumed input") });
        assert!(
            foreign.import_owned_staging_shielded(&session, DEVNET_GENESIS.hash, wire, metadata.state_root, &mut forbidden).is_err()
        );

        import = session.downcast::<Mutex<StagingImport>>().unwrap().into_inner();
        let mut operation = import.start_guarded_operation(StagingComponent::Shielded).unwrap();
        let readiness = DbBranchReadiness::new(db.clone());
        let mut accumulator = MuHash::new();
        accumulator.add_element(&[1; 32]);
        accumulator.add_element(&[2; 32]);
        let mut metadata = metadata;
        metadata.nullifier_muhash = accumulator;
        metadata.state_root = PruningPointShieldedMetadata::recompute_state_root(
            &metadata.frontier,
            &metadata.supply,
            &metadata.nullifier_muhash,
            &metadata.burns,
        )
        .unwrap();
        let mut frame = 0;
        let mut source = std::iter::from_fn(|| {
            frame += 1;
            match frame {
                1 => Some(vec![[1; 32]]),
                2 => {
                    readiness.close_revision_sync().unwrap();
                    Some(vec![[2; 32]])
                }
                _ => None,
            }
        });
        let error = processor.import_bounded_pruning_shielded(
            DEVNET_GENESIS.hash,
            ShieldedExportMetadata { data: metadata.to_wire_bytes(), nullifier_count: 2 },
            Some(metadata.state_root),
            &mut source,
            |batch| processor.commit_guarded_shielded_batch(&mut operation, batch),
        );
        assert!(error.is_err());
        let rows = DbNullifierSetStore::new(db.clone(), CachePolicy::Empty);
        assert!(rows.contains(&[1; 32]).unwrap());
        assert!(!rows.contains(&[2; 32]).unwrap());
        assert!(!consensus.is_pruning_shielded_stable());
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err());
        drop(operation);
        drop(import);
        assert!(StagingImport::begin_fixture(processor.clone(), proof).is_ok());
    }
    #[test]
    fn bounded_cleanup_rejects_foreign_busy_out_of_order_and_stale_permits() {
        use crate::model::stores::shielded::{DbNullifierSetStore, NullifierSetStore};
        use kaspa_consensus_core::api::SHIELDED_NULLIFIER_IMPORT_BATCH_LIMIT;
        use kaspa_database::prelude::CachePolicy;
        let config = ConfigBuilder::new(DEVNET_PARAMS).build();
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let point = DEVNET_GENESIS.hash;
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(header)]]).unwrap();
        let import = StagingImport::begin_fixture(processor.clone(), proof.clone()).unwrap();
        let session = StagingImportSession::new(Mutex::new(import));
        let rows = DbNullifierSetStore::new(db.clone(), CachePolicy::Empty);
        let count = SHIELDED_NULLIFIER_IMPORT_BATCH_LIMIT + 1;
        let mut batch = rocksdb::WriteBatch::default();
        for i in 0..count {
            let mut nf = [0; 32];
            nf[..8].copy_from_slice(&(i as u64).to_le_bytes());
            rows.insert_batch(&mut batch, nf).unwrap();
        }
        db.write(batch).unwrap();
        let marker = db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap();
        assert!(consensus.clear_owned_staging_shielded(&session, point).is_err(), "proof component cannot clear shielded data");
        {
            let import = session.downcast_ref::<Mutex<StagingImport>>().unwrap().lock();
            *import.progress.lock() = StagingProgress::Writing(StagingComponent::Shielded);
        }
        let foreign = TestConsensus::new(&config);
        assert!(foreign.clear_owned_staging_shielded(&session, point).is_err());
        assert!(consensus.clear_owned_staging_shielded(&session, Hash::from_bytes([9; 32])).is_err());
        assert_eq!(db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap(), marker);
        assert_eq!(rows.iter_all().try_fold(0usize, |n, row| row.map(|_| n + 1)).unwrap(), count);

        let mut operation = session
            .downcast_ref::<Mutex<StagingImport>>()
            .unwrap()
            .lock()
            .start_guarded_operation(StagingComponent::Shielded)
            .unwrap();
        assert!(consensus.clear_owned_staging_shielded(&session, point).is_err(), "live worker keeps cleanup excluded");
        let readiness = DbBranchReadiness::new(db.clone());
        let mut writes = 0;
        let result = processor.clear_bounded_pruning_shielded(point, |batch| {
            processor.commit_guarded_shielded_batch(&mut operation, batch)?;
            writes += 1;
            if writes == 2 {
                readiness.close_revision_sync()?;
            }
            Ok(())
        });
        assert!(result.is_err(), "the next deletion must reject the replaced revision");
        assert_eq!(writes, 2, "only false readiness and one bounded deletion may persist");
        assert_eq!(rows.iter_all().try_fold(0usize, |n, row| row.map(|_| n + 1)).unwrap(), 1);
        assert!(!consensus.is_pruning_shielded_stable());
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err());
        drop(session);
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err(), "detached worker still reserves the owner");
        drop(operation);
        let import = StagingImport::begin_fixture(processor.clone(), proof).unwrap();
        *import.progress.lock() = StagingProgress::Writing(StagingComponent::Shielded);
        let fresh_session = StagingImportSession::new(Mutex::new(import));
        consensus.clear_owned_staging_shielded(&fresh_session, point).unwrap();
        assert!(rows.iter_all().next().is_none());
        assert!(!consensus.is_pruning_shielded_stable());
    }
}
