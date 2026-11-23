//! Uncached storage for a verified issuance checkpoint in the authority batch.

use super::selected_chain::{DbSelectedChainStore, SelectedChainStoreReader};
use bincode::Options;
use kaspa_consensus_core::issuance_checkpoint::IssuanceCheckpoint;
use kaspa_database::{
    prelude::{CachePolicy, DB, DeferredDbBatch, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
use kaspa_hashes::Hash;
use parking_lot::RwLockReadGuard;
use std::sync::Arc;

// Distinct from the readiness and promotion-marker keys in this namespace.
const CHECKPOINT_KEY: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, b'I', 1];

/// Trusted context obtained independently of persisted or imported settlement data.
#[derive(Clone, Copy)]
pub struct IssuanceCheckpointAnchor {
    pub network_id: Hash,
    pub pruning_point: Hash,
    pub selected_index: u64,
    pub supply_cap: u64,
    pub expected_commitment: Hash,
}

/// One replaceable summary. Per-source acceptance remains owned by consensus.
pub struct DbIssuanceCheckpoint {
    db: Arc<DB>,
}

impl DbIssuanceCheckpoint {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Stage a verified row without publishing it. The caller must hold the
    /// consensus selected-chain lock on this DB through the batch commit.
    /// An invalid row rejects the whole batch, including any staged readiness.
    pub fn stage<'guard>(
        &self,
        batch: &mut DeferredDbBatch<'guard>,
        selected: &'guard RwLockReadGuard<'_, DbSelectedChainStore>,
        anchor: IssuanceCheckpointAnchor,
        checkpoint: &IssuanceCheckpoint,
    ) -> StoreResult<()> {
        let result = (|| {
            self.verify(selected, anchor, checkpoint)?;
            let bytes = codec().serialize(checkpoint).map_err(|error| inconsistent(error.to_string()))?;
            batch.reserve_unique_key(CHECKPOINT_KEY)?;
            batch.put_raw(&self.db, CHECKPOINT_KEY, bytes)
        })();
        if result.is_err() {
            batch.reject();
        }
        result
    }

    /// Recover the committed row only if it still matches the pinned selected
    /// path and an independently supplied expected commitment.
    pub fn load(
        &self,
        selected: &RwLockReadGuard<'_, DbSelectedChainStore>,
        anchor: IssuanceCheckpointAnchor,
    ) -> StoreResult<Option<IssuanceCheckpoint>> {
        let Some(bytes) = self.db.get(CHECKPOINT_KEY)? else {
            return Ok(None);
        };
        let checkpoint = codec().deserialize(&bytes).map_err(|error| inconsistent(error.to_string()))?;
        self.verify(selected, anchor, &checkpoint)?;
        Ok(Some(checkpoint))
    }

    fn verify(
        &self,
        selected: &RwLockReadGuard<'_, DbSelectedChainStore>,
        anchor: IssuanceCheckpointAnchor,
        checkpoint: &IssuanceCheckpoint,
    ) -> StoreResult<()> {
        if !selected.is_for_database(&self.db) {
            return Err(inconsistent("selected-chain guard belongs to another database"));
        }
        let committed = DbSelectedChainStore::new(self.db.clone(), CachePolicy::Empty);
        if committed.get_tip()?.0 < anchor.selected_index
            || committed.get_by_index(anchor.selected_index)? != anchor.pruning_point
            || committed.get_by_hash(anchor.pruning_point)? != anchor.selected_index
        {
            return Err(inconsistent("issuance checkpoint is outside the committed selected chain"));
        }
        checkpoint
            .verify(anchor.network_id, anchor.pruning_point, anchor.supply_cap, anchor.expected_commitment)
            .map_err(|error| inconsistent(format!("issuance checkpoint verification failed: {error:?}")))
    }
}

fn codec() -> impl Options {
    bincode::DefaultOptions::new().with_fixint_encoding().reject_trailing_bytes().with_limit(256)
}

fn inconsistent(reason: impl Into<String>) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::selected_chain::SelectedChainStore;
    use kaspa_consensus_core::{ChainPath, issuance_checkpoint::IssuanceClaim};
    use kaspa_database::prelude::ConnBuilder;
    use parking_lot::RwLock;
    use rocksdb::WriteBatch;

    const CAP: u64 = 2_900_000_000_000_000_000;
    const BATCH_PROBE: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, 0xf2];

    fn fixture(network: Hash, point: Hash) -> (IssuanceCheckpointAnchor, IssuanceCheckpoint) {
        let claim = IssuanceClaim {
            reward_source: Hash::from_u64_word(9),
            settlement_block: point,
            selected_index: 1,
            minted: 100,
            paid: 85,
        };
        let checkpoint = IssuanceCheckpoint::derive_from_claims(network, point, CAP, &[network, point], &[claim]).unwrap();
        let anchor = IssuanceCheckpointAnchor {
            network_id: network,
            pruning_point: point,
            selected_index: 1,
            supply_cap: CAP,
            expected_commitment: checkpoint.commitment(),
        };
        (anchor, checkpoint)
    }

    #[test]
    fn issuance_checkpoint_store_reopens_and_rejects_abandoned_selected_owner() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let [genesis, original, alternate] = [1, 2, 3].map(Hash::from_u64_word);
        let (anchor, checkpoint) = fixture(genesis, original);
        let (replacement_anchor, replacement) = fixture(genesis, alternate);

        {
            let db = connect();
            let mut selected = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
            let mut init = WriteBatch::default();
            selected.init_with_pruning_point(&mut init, genesis).unwrap();
            db.write(init).unwrap();
            let mut add = WriteBatch::default();
            selected.apply_changes(&mut add, &ChainPath { removed: vec![], added: vec![original] }).unwrap();
            db.write(add).unwrap();
            let selected = RwLock::new(selected);
            let selected = selected.read();
            let store = DbIssuanceCheckpoint::new(db.clone());

            let mut pending = DeferredDbBatch::new(db.clone());
            store.stage(&mut pending, &selected, anchor, &checkpoint).unwrap();
            pending.put_raw(&db, BATCH_PROBE, b"unsubmitted").unwrap();
            drop(pending);
        }

        {
            let db = connect();
            let selected = RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
            let guard = selected.read();
            let store = DbIssuanceCheckpoint::new(db.clone());
            assert!(store.load(&guard, anchor).unwrap().is_none());
            assert!(db.get(BATCH_PROBE).unwrap().is_none());

            let mut batch = DeferredDbBatch::new(db.clone());
            batch.put_raw(&db, BATCH_PROBE, b"unauthenticated").unwrap();
            let altered = IssuanceCheckpoint { paid: 84, deferred: 16, ..checkpoint };
            assert!(store.stage(&mut batch, &guard, anchor, &altered).is_err());
            assert!(batch.commit().is_err());
            assert!(store.load(&guard, anchor).unwrap().is_none());
            assert!(db.get(BATCH_PROBE).unwrap().is_none());

            let mut batch = DeferredDbBatch::new(db.clone());
            store.stage(&mut batch, &guard, anchor, &checkpoint).unwrap();
            batch.put_raw(&db, BATCH_PROBE, b"committed").unwrap();
            batch.commit().unwrap();
        }

        {
            let db = connect();
            let selected = RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
            let store = DbIssuanceCheckpoint::new(db.clone());
            assert_eq!(store.load(&selected.read(), anchor).unwrap(), Some(checkpoint));
            assert_eq!(db.get(BATCH_PROBE).unwrap().unwrap().as_slice(), b"committed");

            {
                let mut selected = selected.write();
                let mut reorg = WriteBatch::default();
                selected.apply_changes(&mut reorg, &ChainPath { removed: vec![original], added: vec![alternate] }).unwrap();
                db.write(reorg).unwrap();
            }
            let guard = selected.read();
            assert!(store.load(&guard, anchor).is_err());
            let mut stale = DeferredDbBatch::new(db.clone());
            assert!(store.stage(&mut stale, &guard, anchor, &checkpoint).is_err());
            assert!(stale.commit().is_err());

            let mut batch = DeferredDbBatch::new(db);
            store.stage(&mut batch, &guard, replacement_anchor, &replacement).unwrap();
            batch.commit().unwrap();
        }

        let db = connect();
        let selected = RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
        let store = DbIssuanceCheckpoint::new(db);
        let guard = selected.read();
        assert_eq!(store.load(&guard, replacement_anchor).unwrap(), Some(replacement));
        assert!(store.load(&guard, anchor).is_err());
    }
}
