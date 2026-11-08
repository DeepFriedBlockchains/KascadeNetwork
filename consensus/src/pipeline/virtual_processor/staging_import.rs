//! Inactive staging import coordination for a single processor-owned session.

use super::{
    VirtualStateProcessor, branch_readiness::ReadinessPoint, import_operation::ImportWorkerSession,
    import_proof_readiness::ValidatedImportProof, persistent_branch_authority::PersistentBranchLease,
};
use kaspa_consensus_core::block::Block;
use kaspa_database::{
    prelude::{DeferredDbBatch, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
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
                Headers, HeadersCompact, CompressedHeaders, Ghostdag, GhostdagCompact, HeadersSelectedTip, PastPruningPoints,
                PruningPoint, RetentionCheckpoint, RetentionPeriodRoot, Reachability, ReachabilityReindexRoot,
                ReachabilityRelations, ReachabilityTreeChildren, ReachabilityFutureCoveringSet, RelationsParents,
                RelationsChildren, PruningProofDescriptor, BodyMissingAnticone, Statuses, BlockTransactions,
            ],
            Self::Smt => &[SmtBranchVersions, SmtLaneVersions, SmtScoreIndex, SmtSeqCommitMeta, SmtSyncFlag],
            Self::Shielded => &[
                ShieldedNullifiers, ShieldedTreeFrontier, ShieldedAnchors, ShieldedSupply, ShieldedNullifierMuHash,
                ShieldedSyncFlag, ShieldedBurns, ShieldedAnchorProducers, ShieldedDevAccrued, ShieldedAnchorSourceScore,
            ],
            Self::Utxo => &[PruningUtxoset, PruningUtxosetPosition, PruningUtxosetSyncFlag],
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
                let prefixes: Vec<_> = component.write_prefixes().iter().copied().map(u8::from).collect();
                processor.apply_authorized_staging_batch(operation, &prefixes, build)?;
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
        Ok(VerifiedStaging { processor: self.processor.clone(), lease })
    }
}

impl VerifiedStaging {
    /// Publish the inactive promotion record and replacement readiness
    /// revision together. The processor rechecks the exact completion lease.
    pub(super) fn promote(self) -> StoreResult<()> {
        self.processor.commit_staging_promotion(self.lease)
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
        assert!(db.get(DatabaseStorePrefixes::BodyMissingAnticone).unwrap().is_none());
        assert!(db.get(DatabaseStorePrefixes::ShieldedSyncFlag).unwrap().is_none());
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        assert!(DbBranchReadiness::new(db).closed_revision().unwrap().is_some());
        assert!(StagingImport::begin(processor.clone()).is_ok());
    }
}
