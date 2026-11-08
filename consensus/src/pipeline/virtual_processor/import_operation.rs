//! Exclusive worker ownership for a persistent pruning import session.
//!
//! Acquire the operation before spawning blocking work and move it into the
//! closure. Dropping the coordinator or join handle cannot release the token
//! while the blocking task is queued or running.

use super::persistent_branch_authority::{PersistentBranchAuthority, PersistentImportToken};
use kaspa_database::prelude::{DeferredDbBatch, StoreResult};
use std::sync::Arc;
use tokio::{sync::Notify, task::JoinHandle};

/// The single coordinator that can issue operations and recover the token.
pub(super) struct ImportWorkerSession {
    token: Arc<PersistentImportToken>,
    released: Arc<Notify>,
}

/// A non-cloneable permit that pins the session through actual worker exit.
pub(super) struct ImportOperation {
    token: Arc<PersistentImportToken>,
    // Fields drop in order. Release the token before waking staging cleanup.
    _exit: WorkerExit,
}

struct WorkerExit(Arc<Notify>);

impl Drop for WorkerExit {
    fn drop(&mut self) {
        self.0.notify_one();
    }
}

impl ImportWorkerSession {
    pub(super) fn new(token: PersistentImportToken) -> Self {
        Self { token: Arc::new(token), released: Arc::new(Notify::new()) }
    }

    /// Only one worker can hold a permit at a time. The coordinator is the
    /// only source of new permits, so no separate counter or lock is needed.
    pub(super) fn try_start(&mut self) -> Option<ImportOperation> {
        Arc::get_mut(&mut self.token)?;
        Some(ImportOperation { token: self.token.clone(), _exit: WorkerExit(self.released.clone()) })
    }

    /// Own the permit for the full blocking call, including closure capture
    /// destruction. The work can use database handles but cannot return one.
    pub(super) fn spawn_blocking(
        &mut self,
        work: impl FnOnce(&mut ImportOperation) -> StoreResult<()> + Send + 'static,
    ) -> Option<JoinHandle<StoreResult<()>>> {
        let mut operation = self.try_start()?;
        Some(tokio::task::spawn_blocking(move || work(&mut operation)))
    }

    /// Cancel without publishing readiness. Staging cleanup can drop its
    /// database owner after this returns and then delete the database.
    pub(super) async fn cancel_and_wait(self) {
        while Arc::strong_count(&self.token) > 1 {
            self.released.notified().await;
        }
    }

    /// Recover the token for owner verification after the worker exits.
    /// Keep the coordinator on failure so completion can be retried.
    pub(super) fn try_finish(self) -> Result<PersistentImportToken, Self> {
        Arc::try_unwrap(self.token).map_err(|token| Self { token, released: self.released })
    }
}

impl ImportOperation {
    /// Each batch is checked by the creating authority against its exact
    /// owner, session, database, and committed closed revision. Borrow the
    /// permit exclusively so the worker cannot use it for parallel writes.
    pub(super) fn apply(&mut self, owner: &PersistentBranchAuthority, batch: DeferredDbBatch<'_>) -> StoreResult<()> {
        owner.apply_import(&self.token, batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::virtual_processor::branch_readiness::DbBranchReadiness;
    use kaspa_database::{
        create_temp_db,
        prelude::{ConnBuilder, DB},
    };
    use std::sync::mpsc;
    use tokio::sync::oneshot;

    const DATA_KEY: &[u8] = b"import-operation-test";

    fn data_batch(db: &Arc<DB>) -> DeferredDbBatch<'static> {
        let mut batch = DeferredDbBatch::new(db.clone());
        batch.put_raw(db, DATA_KEY, b"imported").unwrap();
        batch
    }

    #[tokio::test]
    async fn cancelled_session_stays_pinned_until_detached_worker_exits() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let owner = Arc::new(PersistentBranchAuthority::new(db.clone()));
        let mut session = ImportWorkerSession::new(owner.begin_import().unwrap());
        let mut operation = session.try_start().unwrap();
        let worker_owner = owner.clone();
        let worker_db = db.clone();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (exited_tx, exited_rx) = oneshot::channel();
        let worker = tokio::task::spawn_blocking(move || {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            operation.apply(&worker_owner, data_batch(&worker_db)).unwrap();
            drop(operation);
            drop(worker_owner);
            drop(worker_db);
            exited_tx.send(()).unwrap();
        });

        started_rx.await.unwrap();
        drop(worker);
        drop(session);
        assert!(owner.begin_import().is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());

        release_tx.send(()).unwrap();
        exited_rx.await.unwrap();
        assert_eq!(db.get(DATA_KEY).unwrap().unwrap().as_slice(), b"imported");
        assert!(DbBranchReadiness::new(db).closed_revision().unwrap().is_some());
        assert!(owner.begin_import().is_ok());
    }

    #[tokio::test]
    async fn staging_cleanup_drains_detached_worker_and_releases_database() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(32).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        let old_db = Arc::downgrade(&db);
        let owner = Arc::new(PersistentBranchAuthority::new(db.clone()));
        let mut session = ImportWorkerSession::new(owner.begin_import().unwrap());
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = session
            .spawn_blocking(move |operation| {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                operation.apply(&owner, data_batch(&db))
            })
            .unwrap();

        started_rx.await.unwrap();
        drop(worker);
        let mut drain = Box::pin(session.cancel_and_wait());
        tokio::select! {
            biased;
            () = &mut drain => panic!("staging cleanup finished before the detached worker exited"),
            () = std::future::ready(()) => {}
        }
        assert!(old_db.upgrade().is_some());

        release_tx.send(()).unwrap();
        drain.await;
        assert!(old_db.upgrade().is_none());
        let reopened = connect();
        assert_eq!(reopened.get(DATA_KEY).unwrap().unwrap().as_slice(), b"imported");
        assert!(DbBranchReadiness::new(reopened).closed_revision().unwrap().is_some());
    }

    #[tokio::test]
    async fn completion_and_second_worker_wait_for_first_permit_to_drop() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let owner = PersistentBranchAuthority::new(db.clone());

        let mut session = ImportWorkerSession::new(owner.begin_import().unwrap());
        let operation = session.try_start().unwrap();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = tokio::task::spawn_blocking(move || {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            drop(operation);
        });
        started_rx.await.unwrap();
        assert!(session.try_start().is_none());
        let Err(mut session) = session.try_finish() else {
            panic!("completion recovered the token while the worker held it");
        };

        release_tx.send(()).unwrap();
        worker.await.unwrap();
        drop(session.try_start().expect("the next worker can now start"));
        let Ok(token) = session.try_finish() else {
            panic!("completion could not recover the token after worker exit");
        };
        owner.apply_import(&token, data_batch(&db)).unwrap();
        assert_eq!(db.get(DATA_KEY).unwrap().unwrap().as_slice(), b"imported");
    }

    #[test]
    fn permit_rejects_foreign_owner_and_batch_before_writing() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let (_other, other_db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let owner = PersistentBranchAuthority::new(db.clone());
        let wrong_owner = PersistentBranchAuthority::new(db.clone());
        let mut session = ImportWorkerSession::new(owner.begin_import().unwrap());
        let mut operation = session.try_start().unwrap();

        assert!(operation.apply(&wrong_owner, data_batch(&db)).is_err());
        assert!(operation.apply(&owner, data_batch(&other_db)).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
        assert!(other_db.get(DATA_KEY).unwrap().is_none());

        operation.apply(&owner, data_batch(&db)).unwrap();
        assert_eq!(db.get(DATA_KEY).unwrap().unwrap().as_slice(), b"imported");
    }
}
