//! Inactive durable readiness storage for processor-owned branch authority.
//!
//! This is storage, not an authorization or import-verification API. The owner
//! must hold its pruning-point and metadata guards from corroboration through
//! the synchronous completion write. No live consensus path reads this marker.

use kaspa_database::prelude::{DB, DeferredDbBatch, StoreError, StoreResult};
use kaspa_database::registry::DatabaseStorePrefixes;
use kaspa_hashes::Hash;
use rand::RngCore;
use std::sync::Arc;

const VERSION: u8 = 2;
const CLOSED: u8 = 0;
const READY: u8 = 1;
const ENCODED_LEN: usize = 74;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReadinessRevision([u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReadinessPoint {
    pub(super) hash: Hash,
    pub(super) index: u64,
}

pub(super) struct DbBranchReadiness {
    db: Arc<DB>,
}

impl DbBranchReadiness {
    pub(super) fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    fn key() -> [u8; 1] {
        [DatabaseStorePrefixes::KascadeBranchReadiness as u8]
    }

    fn encode(point: Option<ReadinessPoint>) -> StoreResult<[u8; ENCODED_LEN]> {
        let mut bytes = [0; ENCODED_LEN];
        bytes[0] = VERSION;
        if let Some(point) = point {
            bytes[1] = READY;
            bytes[2..34].copy_from_slice(&point.hash.as_bytes());
            bytes[34..42].copy_from_slice(&point.index.to_le_bytes());
        }
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes[42..])
            .map_err(|error| StoreError::DataInconsistency(format!("branch readiness randomness unavailable: {error}")))?;
        Ok(bytes)
    }

    /// Reads committed bytes every time. Absence never migrates to readiness.
    /// A ready marker at a different hash OR index does not authorize this point.
    pub(super) fn is_ready(&self, point: ReadinessPoint) -> StoreResult<bool> {
        Ok(self.ready_revision(point)?.is_some())
    }

    /// Revision changes on every close or completion, including same-point
    /// transitions. A reconstructed owner cannot revive an earlier lease.
    pub(super) fn ready_revision(&self, point: ReadinessPoint) -> StoreResult<Option<ReadinessRevision>> {
        let Some(bytes) = self.db.get_pinned(Self::key())? else {
            return Ok(None);
        };
        // V1 lacked a durable revision and cannot safely authorize this owner.
        // Migration is explicit close + independently corroborated completion.
        if bytes.len() == 42 && bytes[0] == 1 {
            return Ok(None);
        }
        if bytes.len() != ENCODED_LEN || bytes[0] != VERSION {
            return Err(StoreError::DataInconsistency("invalid branch readiness marker".into()));
        }
        match bytes[1] {
            CLOSED if bytes[2..42].iter().all(|byte| *byte == 0) => Ok(None),
            READY => {
                if bytes[2..34] != point.hash.as_bytes() || bytes[34..42] != point.index.to_le_bytes() {
                    return Ok(None);
                }
                let mut revision = [0; 32];
                revision.copy_from_slice(&bytes[42..]);
                Ok(Some(ReadinessRevision(revision)))
            }
            _ => Err(StoreError::DataInconsistency("invalid branch readiness status".into())),
        }
    }

    /// Must succeed before the first import mutation. An error must leave the
    /// processor's in-memory authority closed and prevent starting the import.
    #[cfg(test)]
    pub(super) fn close_sync(&self) -> StoreResult<()> {
        self.close_revision_sync().map(|_| ())
    }

    pub(super) fn close_revision_sync(&self) -> StoreResult<ReadinessRevision> {
        let bytes = Self::encode(None)?;
        let mut revision = [0; 32];
        revision.copy_from_slice(&bytes[42..]);
        let mut batch = DeferredDbBatch::new(self.db.clone());
        batch.put_raw(&self.db, Self::key(), bytes)?;
        batch.commit()?;
        Ok(ReadinessRevision(revision))
    }

    pub(super) fn closed_revision(&self) -> StoreResult<Option<ReadinessRevision>> {
        let Some(bytes) = self.db.get_pinned(Self::key())? else {
            return Ok(None);
        };
        if bytes.len() == 42 && bytes[0] == 1 {
            return Ok(None);
        }
        if bytes.len() != ENCODED_LEN || bytes[0] != VERSION {
            return Err(StoreError::DataInconsistency("invalid branch readiness marker".into()));
        }
        match bytes[1] {
            CLOSED if bytes[2..42].iter().all(|byte| *byte == 0) => {
                let mut revision = [0; 32];
                revision.copy_from_slice(&bytes[42..]);
                Ok(Some(ReadinessRevision(revision)))
            }
            READY => Ok(None),
            _ => Err(StoreError::DataInconsistency("invalid branch readiness status".into())),
        }
    }

    /// Only stage after independent, authoritative verification of the complete
    /// state at this point. Never use default-true cached flags as that proof.
    /// A dropped or failed batch does not expose readiness to any reader.
    pub(super) fn stage_ready(&self, batch: &mut DeferredDbBatch<'_>, point: ReadinessPoint) -> StoreResult<()> {
        batch.put_raw(&self.db, Self::key(), Self::encode(Some(point))?)
    }

    pub(super) fn stage_closed(&self, batch: &mut DeferredDbBatch<'_>, revision: ReadinessRevision) -> StoreResult<()> {
        let mut bytes = [0; ENCODED_LEN];
        bytes[0] = VERSION;
        bytes[42..].copy_from_slice(&revision.0);
        batch.put_raw(&self.db, Self::key(), bytes)
    }

    /// The owning processor must exclude other sessions/mutations and retain
    /// the same metadata guards across verification and this entire call.
    /// Explicit recovery uses the same verified path; merely retrying a nested
    /// import operation must never call this and reopen an outer session.
    pub(super) fn complete_sync(&self, point: ReadinessPoint) -> StoreResult<()> {
        let mut batch = DeferredDbBatch::new(self.db.clone());
        self.stage_ready(&mut batch, point)?;
        batch.commit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};

    fn point() -> ReadinessPoint {
        ReadinessPoint { hash: Hash::from_bytes([7; 32]), index: 11 }
    }

    #[test]
    fn absent_corrupt_and_different_point_fail_closed() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let store = DbBranchReadiness::new(db.clone());
        assert!(!store.is_ready(point()).unwrap());
        for bytes in [vec![], vec![VERSION, READY], vec![2; ENCODED_LEN], vec![0; ENCODED_LEN]] {
            db.put(DbBranchReadiness::key(), bytes).unwrap();
            assert!(store.is_ready(point()).is_err());
        }
        store.complete_sync(point()).unwrap();
        assert!(store.is_ready(point()).unwrap());
        assert!(!store.is_ready(ReadinessPoint { index: 12, ..point() }).unwrap());
        assert!(!store.is_ready(ReadinessPoint { hash: Hash::from_bytes([8; 32]), ..point() }).unwrap());
        store.close_sync().unwrap();
        assert!(!store.is_ready(point()).unwrap());
    }

    #[test]
    fn dropped_completion_and_foreign_database_do_not_ready_original() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let (_other, other) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let store = DbBranchReadiness::new(db.clone());
        store.close_sync().unwrap();
        {
            let mut dropped = DeferredDbBatch::new(db.clone());
            store.stage_ready(&mut dropped, point()).unwrap();
            assert!(!store.is_ready(point()).unwrap());
        }
        assert!(!DbBranchReadiness::new(db.clone()).is_ready(point()).unwrap());
        let mut foreign = DeferredDbBatch::new(other.clone());
        assert!(store.stage_ready(&mut foreign, point()).is_err());
        assert!(foreign.commit().is_err());
        assert!(!store.is_ready(point()).unwrap());
        assert!(!DbBranchReadiness::new(other).is_ready(point()).unwrap());
    }

    #[test]
    fn readonly_failed_completion_keeps_committed_marker_closed() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let path = temp.path().to_owned();
        {
            let db = ConnBuilder::default()
                .with_files_limit(32)
                .with_mem_budget(16_usize * 1024 * 1024)
                .with_db_path(path.clone())
                .build()
                .unwrap();
            DbBranchReadiness::new(db).close_sync().unwrap();
        }
        let db = ConnBuilder::default()
            .with_files_limit(32)
            .with_mem_budget(16_usize * 1024 * 1024)
            .with_db_path(path.clone())
            .build_readonly()
            .unwrap();
        let store = DbBranchReadiness::new(db.clone());
        assert!(store.complete_sync(point()).is_err());
        assert!(store.close_sync().is_err());
        assert!(!store.is_ready(point()).unwrap());
        drop(store);
        drop(db);
        let db =
            ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024).with_db_path(path).build().unwrap();
        assert!(!DbBranchReadiness::new(db.clone()).is_ready(point()).unwrap());
        drop(db);
        drop(temp);
    }

    fn reopen_after_abrupt_exit(ready: bool, exact_test: &str) {
        const CHILD_PATH: &str = "KASCADE_READINESS_CHILD_PATH";
        if let Some(path) = std::env::var_os(CHILD_PATH) {
            let db = ConnBuilder::default()
                .with_files_limit(32)
                .with_mem_budget(16_usize * 1024 * 1024)
                .with_db_path(path.into())
                .build()
                .unwrap();
            let store = DbBranchReadiness::new(db.clone());
            store.close_sync().unwrap();
            if ready {
                store.complete_sync(point()).unwrap();
            }
            let mut unfinished = DeferredDbBatch::new(db);
            store.stage_ready(&mut unfinished, point()).unwrap();
            // Exercise WAL recovery without normal DB destruction or flushing.
            std::process::exit(17);
        }
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", exact_test, "--nocapture"])
            .env(CHILD_PATH, temp.path())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(17));
        let db = ConnBuilder::default()
            .with_files_limit(32)
            .with_mem_budget(16_usize * 1024 * 1024)
            .with_db_path(temp.path().to_owned())
            .with_create_if_missing(false)
            .build()
            .unwrap();
        let store = DbBranchReadiness::new(db.clone());
        assert_eq!(store.is_ready(point()).unwrap(), ready);
        if !ready {
            // Recovery is explicit and only a future processor's corroborated
            // owner path may invoke this operation, never an automatic default.
            store.complete_sync(point()).unwrap();
            assert!(store.is_ready(point()).unwrap());
        }
        drop(store);
        drop(db);
        drop(temp);
    }

    #[test]
    fn interrupted_import_remains_closed_after_abrupt_writer_exit() {
        reopen_after_abrupt_exit(
            false,
            "pipeline::virtual_processor::branch_readiness::tests::interrupted_import_remains_closed_after_abrupt_writer_exit",
        );
    }

    #[test]
    fn completed_marker_survives_abrupt_writer_exit() {
        reopen_after_abrupt_exit(
            true,
            "pipeline::virtual_processor::branch_readiness::tests::completed_marker_survives_abrupt_writer_exit",
        );
    }
}
