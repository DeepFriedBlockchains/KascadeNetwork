//! Inactive owner layer for durable readiness. Live processor migration is pending.
//!
//! A caller must sample the pruning point under its authoritative metadata guard
//! and retain that guard through lease issuance, completion, or commit. Lock
//! order is pruning/session guard, metadata guard, then this owner's mutex.
//! This module cannot independently corroborate imported consensus state.
//! The verifier reads committed state and must not reenter the authority.

use super::{
    branch_readiness::{DbBranchReadiness, ReadinessPoint, ReadinessRevision},
    import_readiness::{CommittedImportMetadata, verify_pruning_utxos},
};
use kaspa_database::{
    prelude::{DB, DeferredDbBatch, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, Weak},
};

pub(super) struct PersistentBranchAuthority {
    db: Arc<DB>,
    readiness: DbBranchReadiness,
    inner: Arc<AuthorityInner>,
    owner: Arc<()>,
}

struct AuthorityInner {
    // Reserve the Arc allocation identity without keeping RocksDB open.
    _database_identity: Weak<DB>,
    state: Mutex<AuthorityState>,
}

type AuthorityRegistry = Mutex<HashMap<usize, Weak<AuthorityInner>>>;
static AUTHORITIES: OnceLock<AuthorityRegistry> = OnceLock::new();

#[derive(Default)]
struct AuthorityState {
    generation: Arc<()>,
    import: Weak<()>,
    import_failed: bool,
    // A newly opened authority must corroborate committed state before issuing leases.
    verified: Option<VerifiedReadiness>,
}

struct VerifiedReadiness {
    revision: ReadinessRevision,
    metadata: CommittedImportMetadata,
}

/// Opaque, non-cloneable, and usable only with its exact creating owner.
pub(super) struct PersistentImportToken {
    inner: Arc<AuthorityInner>,
    closed_revision: ReadinessRevision,
    owner: Arc<()>,
    session: Arc<()>,
}

pub(super) struct PersistentBranchLease {
    inner: Arc<AuthorityInner>,
    revision: ReadinessRevision,
    owner: Arc<()>,
    generation: Arc<()>,
    point: ReadinessPoint,
}

fn rejected() -> StoreError {
    StoreError::DataInconsistency("persistent branch authority is not ready for this owner/session/point".into())
}

impl PersistentBranchAuthority {
    pub(super) fn new(db: Arc<DB>) -> Self {
        let mut registry = AUTHORITIES.get_or_init(|| Mutex::new(HashMap::new())).lock();
        registry.retain(|_, authority| authority.strong_count() != 0);
        let key = Arc::as_ptr(&db) as usize;
        let inner = registry.get(&key).and_then(Weak::upgrade).unwrap_or_else(|| {
            let inner =
                Arc::new(AuthorityInner { _database_identity: Arc::downgrade(&db), state: Mutex::new(AuthorityState::default()) });
            registry.insert(key, Arc::downgrade(&inner));
            inner
        });
        Self { readiness: DbBranchReadiness::new(db.clone()), db, inner, owner: Arc::new(()) }
    }

    /// Enter before ANY import mutation. Failed marker persistence blocks this
    /// owner and returns no token, so callers must not start mutation on error.
    /// This is also the explicit entry for verified legacy/restart recovery.
    pub(super) fn begin_import(&self) -> StoreResult<PersistentImportToken> {
        let mut state = self.inner.state.lock();
        if state.import.upgrade().is_some() {
            return Err(rejected());
        }
        state.verified = None;
        state.generation = Arc::new(());
        let closed_revision = self.readiness.close_revision_sync()?;
        state.import_failed = false;
        let session = Arc::new(());
        state.import = Arc::downgrade(&session);
        Ok(PersistentImportToken { inner: self.inner.clone(), closed_revision, owner: self.owner.clone(), session })
    }

    pub(super) fn owns_database(&self, db: &Arc<DB>) -> bool {
        Arc::ptr_eq(&self.db, db)
    }

    fn check_import(&self, state: &AuthorityState, token: &PersistentImportToken) -> StoreResult<()> {
        let active = state.import.upgrade().ok_or_else(rejected)?;
        if state.import_failed
            || !Arc::ptr_eq(&self.inner, &token.inner)
            || !Arc::ptr_eq(&self.owner, &token.owner)
            || !Arc::ptr_eq(&active, &token.session)
            || self.readiness.closed_revision()? != Some(token.closed_revision)
        {
            return Err(rejected());
        }
        Ok(())
    }

    fn check_ready(&self, state: &mut AuthorityState, point: ReadinessPoint) -> StoreResult<ReadinessRevision> {
        let verified = state.verified.as_ref().ok_or_else(rejected)?;
        if state.import.upgrade().is_some() || verified.metadata.point != point {
            return Err(rejected());
        }

        let revision = verified.revision;
        let result = self.readiness.ready_revision(point).and_then(|committed| {
            if committed != Some(revision) {
                return Err(rejected());
            }
            verified.metadata.verify_current(&self.db)
        });
        if let Err(error) = result {
            state.verified = None;
            state.generation = Arc::new(());
            self.readiness.close_revision_sync()?;
            return Err(error);
        }
        Ok(revision)
    }

    /// Apply one authenticated import batch while the durable marker stays closed.
    pub(super) fn apply_import(&self, token: &PersistentImportToken, mut batch: DeferredDbBatch<'_>) -> StoreResult<()> {
        let mut state = self.inner.state.lock();
        self.check_import(&state, token)?;
        batch.ensure_database(&self.db)?;
        self.readiness.stage_closed(&mut batch, token.closed_revision)?;
        if let Err(error) = batch.commit() {
            // An invalidated worker still owns the session until its token is dropped.
            state.import_failed = true;
            return Err(error);
        }
        Ok(())
    }

    /// Verify committed import metadata and UTXO contents before reopening.
    /// The processor must provide proof and remaining root verification under
    /// its pruning session, point and metadata guards.
    pub(super) fn complete_verified_import(
        &self,
        token: PersistentImportToken,
        point: ReadinessPoint,
        retention: ReadinessPoint,
        verify_remaining: impl FnOnce(&Arc<DB>, ReadinessPoint) -> StoreResult<()>,
    ) -> StoreResult<()> {
        self.complete_import(token, point, retention, |db, point| {
            verify_pruning_utxos(db, point)?;
            verify_remaining(db, point)
        })
    }

    /// Only the owner may complete a session, after independent authoritative
    /// corroboration under the exact metadata guards retained through this call.
    /// A verifier failure, dropped token, or failed sync write leaves it closed.
    /// A nested subcall cannot obtain a second token or complete this token.
    fn complete_import(
        &self,
        token: PersistentImportToken,
        point: ReadinessPoint,
        retention: ReadinessPoint,
        verify: impl FnOnce(&Arc<DB>, ReadinessPoint) -> StoreResult<()>,
    ) -> StoreResult<()> {
        let mut state = self.inner.state.lock();
        self.check_import(&state, &token)?;
        let metadata = CommittedImportMetadata::verify(&self.db, point, retention)?;
        verify(&self.db, point)?;
        self.check_import(&state, &token)?;
        metadata.verify_current(&self.db)?;
        let revision = self.readiness.complete_sync(point)?;
        state.verified = Some(VerifiedReadiness { revision, metadata });
        state.generation = Arc::new(());
        state.import = Weak::new();
        Ok(())
    }

    pub(super) fn lease(&self, point: ReadinessPoint) -> StoreResult<PersistentBranchLease> {
        let mut state = self.inner.state.lock();
        let revision = self.check_ready(&mut state, point)?;
        Ok(PersistentBranchLease {
            inner: self.inner.clone(),
            revision,
            owner: self.owner.clone(),
            generation: state.generation.clone(),
            point,
        })
    }

    /// Commit the authorized write and its new READY revision in one sync batch.
    pub(super) fn commit(
        &self,
        point: ReadinessPoint,
        lease: PersistentBranchLease,
        mut batch: DeferredDbBatch<'_>,
    ) -> StoreResult<()> {
        let mut state = self.inner.state.lock();
        if !Arc::ptr_eq(&self.inner, &lease.inner)
            || !Arc::ptr_eq(&self.owner, &lease.owner)
            || !Arc::ptr_eq(&state.generation, &lease.generation)
            || lease.point != point
        {
            return Err(rejected());
        }
        batch.ensure_database(&self.db)?;
        batch.ensure_key_prefix(DatabaseStorePrefixes::KascadeBranchAuthorityData as u8)?;
        if self.check_ready(&mut state, point)? != lease.revision {
            return Err(rejected());
        }
        let mut verified = state.verified.take().ok_or_else(rejected)?;
        state.generation = Arc::new(());
        self.readiness.close_revision_sync()?;
        verified.revision = self.readiness.stage_ready(&mut batch, point)?;
        batch.commit()?;
        state.verified = Some(verified);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::import_readiness::tests::{install_point as install_import_point, stage_explicit_evidence};
    use crate::model::stores::{
        past_pruning_points::{DbPastPruningPointsStore, PastPruningPointsStore},
        pruning::{DbPruningStore, PruningStoreReader},
        pruning_meta::PruningMetaStores,
    };
    use kaspa_database::{create_temp_db, prelude::{CachePolicy, ConnBuilder}};
    use rocksdb::WriteBatch;

    const DATA_KEY: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, 0];

    fn point() -> ReadinessPoint {
        ReadinessPoint { hash: kaspa_hashes::Hash::from_bytes([7; 32]), index: 11 }
    }

    fn retention() -> ReadinessPoint {
        ReadinessPoint { hash: kaspa_hashes::Hash::from_bytes([6; 32]), index: 10 }
    }

    fn install_point(db: &Arc<DB>) {
        install_import_point(db, point(), retention());
        let mut metadata = PruningMetaStores::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        stage_explicit_evidence(&mut metadata, &mut batch, point());
        db.write(batch).unwrap();
    }

    fn complete(owner: &PersistentBranchAuthority) {
        let token = owner.begin_import().unwrap();
        owner.complete_import(token, point(), retention(), |_, _| Ok(())).unwrap();
    }

    fn data_batch(db: &Arc<DB>) -> DeferredDbBatch<'static> {
        let mut batch = DeferredDbBatch::new(db.clone());
        batch.put_raw(db, DATA_KEY, b"committed").unwrap();
        batch
    }

    #[test]
    fn commits_data_and_readiness_and_revokes_sibling_leases() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();
        let sibling = owner.lease(point()).unwrap();
        let batch = data_batch(&db);
        assert!(db.get(DATA_KEY).unwrap().is_none());
        owner.commit(point(), lease, batch).unwrap();
        assert_eq!(db.get(DATA_KEY).unwrap().unwrap().as_slice(), b"committed");
        assert!(owner.commit(point(), sibling, data_batch(&db)).is_err());
        assert!(PersistentBranchAuthority::new(db).lease(point()).is_ok());
    }

    #[test]
    fn rejects_foreign_database_owner_and_point_before_write() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let (_other, foreign) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        let other_owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();
        assert!(owner.commit(point(), lease, data_batch(&foreign)).is_err());
        let lease = owner.lease(point()).unwrap();
        assert!(other_owner.commit(point(), lease, data_batch(&db)).is_err());
        let lease = owner.lease(point()).unwrap();
        assert!(owner.commit(ReadinessPoint { index: 12, ..point() }, lease, data_batch(&db)).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
        assert!(foreign.get(DATA_KEY).unwrap().is_none());
        assert!(owner.lease(point()).is_ok());
    }

    #[test]
    fn lease_batch_cannot_change_its_own_committed_evidence() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();

        let position = DatabaseStorePrefixes::PruningUtxosetPosition;
        let before = db.get(position).unwrap().unwrap();
        let mut batch = data_batch(&db);
        batch.put_raw(&db, position, b"invalid").unwrap();
        assert!(owner.commit(point(), lease, batch).is_err());
        assert_eq!(db.get(position).unwrap().unwrap(), before);
        assert!(db.get(DATA_KEY).unwrap().is_none());

        let lease = owner.lease(point()).unwrap();
        owner.commit(point(), lease, data_batch(&db)).unwrap();
        assert!(db.get(DATA_KEY).unwrap().is_some());
    }

    #[test]
    fn metadata_drift_closes_readiness_and_restoring_rows_cannot_revive_a_lease() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();
        let mut pruning = DbPruningStore::new(db.clone());
        let before = pruning.pruning_proof_descriptor().unwrap();
        let mut changed = (*before).clone();
        changed.counts[0] += 1;
        pruning.set_pruning_proof_descriptor(changed).unwrap();

        assert!(owner.commit(point(), lease, data_batch(&db)).is_err());
        assert!(!DbBranchReadiness::new(db.clone()).is_ready(point()).unwrap());
        assert!(db.get(DATA_KEY).unwrap().is_none());

        pruning.set_pruning_proof_descriptor((*before).clone()).unwrap();
        assert!(owner.lease(point()).is_err());
        complete(&owner);
        assert!(owner.lease(point()).is_ok());
    }

    #[test]
    fn rejects_same_point_aba_and_pruning_history_mismatch() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let old = owner.lease(point()).unwrap();
        let readiness = DbBranchReadiness::new(db.clone());
        readiness.close_sync().unwrap();
        readiness.complete_sync(point()).unwrap();
        assert!(owner.commit(point(), old, data_batch(&db)).is_err());
        assert!(owner.lease(point()).is_err());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();
        DbPastPruningPointsStore::new(db.clone(), CachePolicy::Empty).set(point().index, 42.into()).unwrap();
        assert!(owner.lease(point()).is_err());
        assert!(owner.commit(point(), lease, data_batch(&db)).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
    }

    #[test]
    fn import_writes_require_exact_token_and_keep_marker_closed() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let (_other, foreign) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        let other_owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let old = owner.lease(point()).unwrap();
        let token = owner.begin_import().unwrap();
        assert!(other_owner.begin_import().is_err());
        assert!(other_owner.apply_import(&token, data_batch(&db)).is_err());
        assert!(owner.apply_import(&token, data_batch(&foreign)).is_err());
        assert!(owner.commit(point(), old, data_batch(&db)).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
        assert!(foreign.get(DATA_KEY).unwrap().is_none());

        let mut batch = data_batch(&db);
        let readiness = DbBranchReadiness::new(db.clone());
        readiness.stage_ready(&mut batch, point()).unwrap();
        owner.apply_import(&token, batch).unwrap();
        assert!(!readiness.is_ready(point()).unwrap());
        assert!(owner.lease(point()).is_err());
        owner
            .complete_import(token, point(), retention(), |verified_db, actual| {
                assert!(Arc::ptr_eq(&db, verified_db));
                assert_eq!(actual, point());
                assert_eq!(verified_db.get(DATA_KEY)?.unwrap().as_slice(), b"committed");
                Ok(())
            })
            .unwrap();
        assert!(owner.lease(point()).is_ok());
    }

    #[test]
    fn dropped_or_failed_import_cannot_reopen_without_verification() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let token = owner.begin_import().unwrap();
        owner.apply_import(&token, data_batch(&db)).unwrap();
        drop(token);
        drop(owner);
        let restarted = PersistentBranchAuthority::new(db);
        assert!(restarted.lease(point()).is_err());
        let token = restarted.begin_import().unwrap();
        assert!(restarted.complete_import(token, point(), retention(), |_, _| Err(rejected())).is_err());
        assert!(restarted.lease(point()).is_err());
        complete(&restarted);
    }

    #[test]
    fn failed_batch_leaves_closed_marker_and_no_data() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let lease = owner.lease(point()).unwrap();
        let mut batch = data_batch(&db);
        batch.reject();
        assert!(owner.commit(point(), lease, batch).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
        assert!(PersistentBranchAuthority::new(db).lease(point()).is_err());
    }

    #[test]
    fn legacy_and_missing_committed_history_require_explicit_recovery() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        let owner = PersistentBranchAuthority::new(db.clone());
        let mut legacy = [0; 42];
        legacy[0] = 1;
        legacy[1] = 1;
        legacy[2..34].copy_from_slice(&point().hash.as_bytes());
        legacy[34..].copy_from_slice(&point().index.to_le_bytes());
        db.put(DatabaseStorePrefixes::KascadeBranchReadiness.as_ref(), legacy).unwrap();
        assert!(owner.lease(point()).is_err());
        let token = owner.begin_import().unwrap();
        assert!(owner.complete_import(token, point(), retention(), |_, _| panic!("missing history accepted")).is_err());
        assert!(owner.lease(point()).is_err());
        install_point(&db);
        complete(&owner);
    }

    #[test]
    fn failed_import_batch_invalidates_token_and_persists_closure() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(32).with_mem_budget(16_usize * 1024 * 1024));
        install_point(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        let token = owner.begin_import().unwrap();
        let mut batch = data_batch(&db);
        batch.reject();
        assert!(owner.apply_import(&token, batch).is_err());
        let other_owner = PersistentBranchAuthority::new(db.clone());
        assert!(other_owner.begin_import().is_err());
        assert!(owner.apply_import(&token, data_batch(&db)).is_err());
        assert!(owner.complete_import(token, point(), retention(), |_, _| panic!("invalid token accepted")).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
        assert!(owner.lease(point()).is_err());
        assert!(other_owner.begin_import().is_ok());
    }

    #[test]
    fn durable_ready_marker_requires_verification_after_database_reopen() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(32).with_db_path(temp.path().to_owned()).build().unwrap();
        {
            let db = connect();
            install_point(&db);
            complete(&PersistentBranchAuthority::new(db));
        }
        let db = connect();
        let readiness = DbBranchReadiness::new(db.clone());
        assert!(readiness.is_ready(point()).unwrap());

        let owner = PersistentBranchAuthority::new(db);
        assert!(owner.lease(point()).is_err());
        let token = owner.begin_import().unwrap();
        assert!(owner.complete_import(token, point(), retention(), |_, _| Err(rejected())).is_err());
        assert!(!readiness.is_ready(point()).unwrap());
        assert!(owner.lease(point()).is_err());

        complete(&owner);
        assert!(owner.lease(point()).is_ok());
    }

    #[test]
    fn interrupted_import_stays_closed_on_database_reopen() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(32).with_db_path(temp.path().to_owned()).build().unwrap();
        {
            let db = connect();
            install_point(&db);
            let owner = PersistentBranchAuthority::new(db.clone());
            complete(&owner);
            let token = owner.begin_import().unwrap();
            owner.apply_import(&token, data_batch(&db)).unwrap();
        }
        let db = connect();
        let owner = PersistentBranchAuthority::new(db.clone());
        assert!(owner.lease(point()).is_err());
        assert_eq!(db.get(DATA_KEY).unwrap().unwrap().as_slice(), b"committed");
        let token = owner.begin_import().unwrap();
        owner
            .complete_import(token, point(), retention(), |verified_db, _| {
                if verified_db.get(DATA_KEY)?.as_deref() != Some(b"committed".as_slice()) {
                    return Err(rejected());
                }
                Ok(())
            })
            .unwrap();
        assert!(owner.lease(point()).is_ok());
    }

    #[test]
    fn abandoned_capabilities_do_not_pin_staging_database_handles() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(32).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        install_point(&db);
        let old_db = Arc::downgrade(&db);
        let owner = PersistentBranchAuthority::new(db.clone());
        complete(&owner);
        let old_lease = owner.lease(point()).unwrap();
        let old_token = owner.begin_import().unwrap();
        drop(owner);
        drop(db);
        assert!(old_db.upgrade().is_none(), "opaque capabilities must not keep RocksDB open");

        let db = connect();
        let restarted = PersistentBranchAuthority::new(db.clone());
        assert!(restarted.lease(point()).is_err());
        let recovery = restarted.begin_import().unwrap();
        assert!(restarted.apply_import(&old_token, data_batch(&db)).is_err());
        restarted.complete_import(recovery, point(), retention(), |_, _| Ok(())).unwrap();
        assert!(restarted.commit(point(), old_lease, data_batch(&db)).is_err());
        assert!(db.get(DATA_KEY).unwrap().is_none());
    }
}
