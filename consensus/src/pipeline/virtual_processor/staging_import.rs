//! Inactive staging import coordination for a single processor-owned session.

use super::{
    VirtualStateProcessor,
    branch_readiness::ReadinessPoint,
    import_operation::{ImportOperation, ImportWorkerSession},
    import_proof_readiness::ValidatedImportProof,
    persistent_branch_authority::PersistentBranchLease,
};
use crate::consensus::Consensus;
use kaspa_consensus_core::{
    api::{StagingPromotion, StagingPromotionError},
    block::Block,
    tx::{TransactionOutpoint, UtxoEntry},
};
use kaspa_database::{
    prelude::{DeferredDbBatch, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
use kaspa_hashes::Hash;
use kaspa_muhash::MuHash;
use parking_lot::Mutex;
use std::sync::Arc;
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

    fn write_prefixes(self) -> &'static [DatabaseStorePrefixes] {
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
    progress: Arc<Mutex<StagingProgress>>,
}

/// Only authenticated completion can produce the lease for promotion.
pub(super) struct VerifiedStaging {
    processor: Arc<VirtualStateProcessor>,
    lease: PersistentBranchLease,
    proof: ValidatedImportProof,
    child: Block,
}

impl StagingImport {
    pub(super) fn begin(processor: Arc<VirtualStateProcessor>) -> StoreResult<Self> {
        let session = ImportWorkerSession::new(processor.begin_authority_import()?);
        let progress = Arc::new(Mutex::new(StagingProgress::Writing(StagingComponent::Proof)));
        Ok(Self { processor, session: Some(session), progress })
    }

    /// Build the component batch on the blocking worker. Each successful
    /// batch leaves durable readiness closed, including the final chunk.
    pub(super) fn submit(
        &mut self,
        component: StagingComponent,
        last_chunk: bool,
        build: impl FnOnce() -> StoreResult<DeferredDbBatch<'static>> + Send + 'static,
    ) -> StoreResult<JoinHandle<StoreResult<()>>> {
        if component == StagingComponent::Utxo {
            return Err(inconsistent("staging UTXO requires the typed import path"));
        }
        self.submit_operation(component, last_chunk, move |processor, operation| {
            let prefixes: Vec<_> = component.write_prefixes().iter().copied().map(u8::from).collect();
            processor.apply_authorized_staging_batch(operation, &prefixes, build)
        })
    }

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

    pub(super) async fn cancel(self) {
        if let Some(session) = self.session {
            session.cancel_and_wait().await;
        }
    }

    pub(super) fn finish(
        &mut self,
        retention: ReadinessPoint,
        proof: &ValidatedImportProof,
        child: &Block,
    ) -> StoreResult<VerifiedStaging> {
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
        let lease = self.processor.finish_authenticated_authority_import(token, retention, proof, child)?;
        Ok(VerifiedStaging { processor: self.processor.clone(), lease, proof: proof.clone(), child: child.clone() })
    }
}

impl Consensus {
    pub(crate) fn authorize_phase6_staging_promotion(&self, promotion: StagingPromotion) -> Result<(), StagingPromotionError> {
        let verified = promotion.downcast::<VerifiedStaging>()?;
        if !Arc::ptr_eq(&self.virtual_processor, &verified.processor) {
            return Err(StagingPromotionError::InvalidReceipt);
        }
        self.virtual_processor
            .commit_staging_promotion(verified.lease, &verified.proof, &verified.child)
            .map_err(|error| StagingPromotionError::Rejected(error.to_string()))
    }

    /// Inactive typed ingress for direct pruning UTXO chunks. The staging
    /// session remains the only path to the database write.
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

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{consensus::test_consensus::TestConsensus, pipeline::virtual_processor::branch_readiness::DbBranchReadiness};
    use kaspa_consensus_core::config::{ConfigBuilder, params::DEVNET_PARAMS};
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};
    use std::sync::mpsc;
    use tokio::sync::oneshot;

    #[test]
    fn authority_integration_rejects_partial_staging_and_drains_cancelled_worker() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let config = ConfigBuilder::new(DEVNET_PARAMS).skip_adding_genesis().build();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let anticone_before = db.get(DatabaseStorePrefixes::BodyMissingAnticone).unwrap();
        let shielded_before = db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap();
        let mut staging = StagingImport::begin(processor.clone()).unwrap();
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
        assert!(StagingImport::begin(processor.clone()).is_err());

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
        assert!(StagingImport::begin(processor.clone()).is_ok());
    }
}
