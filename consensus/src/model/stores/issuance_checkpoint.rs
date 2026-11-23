//! Uncached storage for a verified issuance checkpoint in the authority batch.

use super::selected_chain::{DbSelectedChainStore, SelectedChainStoreReader};
use crate::processes::coinbase::CoinbaseSettlement;
use bincode::Options;
use kaspa_consensus_core::issuance_checkpoint::{DeveloperPayoutRelease, IssuanceCheckpoint, IssuanceClaim};
use kaspa_database::{
    prelude::{CachePolicy, DB, DeferredDbBatch, StoreError, StoreResult},
    registry::DatabaseStorePrefixes,
};
use kaspa_hashes::Hash;
use parking_lot::RwLockReadGuard;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Distinct from the readiness and promotion-marker keys in this namespace.
const CHECKPOINT_KEY: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, b'I', 1];

const LOCAL_REPLAY_ANCHOR_KEY: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, b'I', 2];

/// Trusted context obtained independently of persisted or imported settlement data.
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct IssuanceCheckpointAnchor {
    pub network_id: Hash,
    pub pruning_point: Hash,
    pub selected_index: u64,
    pub supply_cap: u64,
    pub expected_commitment: Hash,
}

/// Complete local consensus replay through the checkpoint's pruning point.
/// The path, claims and releases must come from local verified consensus.
/// Imported checkpoint contents cannot serve as replay evidence.
pub struct LocalIssuanceReplay<'a> {
    pub network_genesis: Hash,
    pub supply_cap: u64,
    pub selected_path: &'a [Hash],
    pub claims: &'a [IssuanceClaim],
    pub releases: &'a [DeveloperPayoutRelease],
}

/// One replaceable summary. Per-source acceptance remains owned by consensus.
pub struct DbIssuanceCheckpoint {
    db: Arc<DB>,
}

impl DbIssuanceCheckpoint {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Compute an expected digest without reading the checkpoint to import.
    /// The caller holds its authoritative selected-chain guard across this
    /// derivation and the batch commit. A pruned prefix fails closed.
    pub fn derive_anchor_from_local_replay(
        &self,
        selected: &RwLockReadGuard<'_, DbSelectedChainStore>,
        replay: LocalIssuanceReplay<'_>,
    ) -> StoreResult<IssuanceCheckpointAnchor> {
        if !selected.is_for_database(&self.db) {
            return Err(inconsistent("selected-chain guard belongs to another database"));
        }
        let Some((&pruning_point, prefix)) = replay.selected_path.split_last() else {
            return Err(inconsistent("issuance replay has no selected path"));
        };
        if replay.selected_path.first() != Some(&replay.network_genesis) {
            return Err(inconsistent("issuance replay does not start at configured genesis"));
        }
        let selected_index = u64::try_from(prefix.len()).map_err(|_| inconsistent("issuance replay selected index overflow"))?;
        let committed = DbSelectedChainStore::new(self.db.clone(), CachePolicy::Empty);
        if committed.get_tip()?.0 < selected_index {
            return Err(inconsistent("issuance replay extends past the committed selected tip"));
        }
        for (index, &block) in (0u64..).zip(replay.selected_path) {
            if committed.get_by_index(index)? != block || committed.get_by_hash(block)? != index {
                return Err(inconsistent("issuance replay path disagrees with committed selected ancestry"));
            }
        }
        let expected = IssuanceCheckpoint::derive_from_settlements(
            replay.network_genesis,
            pruning_point,
            replay.supply_cap,
            replay.selected_path,
            replay.claims,
            replay.releases,
        )
        .map_err(|error| inconsistent(format!("issuance replay verification failed: {error:?}")))?;
        Ok(IssuanceCheckpointAnchor {
            network_id: replay.network_genesis,
            pruning_point,
            selected_index,
            supply_cap: replay.supply_cap,
            expected_commitment: expected.commitment(),
        })
    }

    /// Retain the locally derived anchor before its selected prefix is pruned.
    /// The authoritative guard must remain held until the shared batch commits.
    /// Only this replay-backed path writes the retained anchor key.
    pub fn stage_retained_replay_checkpoint<'guard>(
        &self,
        batch: &mut DeferredDbBatch<'guard>,
        selected: &'guard RwLockReadGuard<'_, DbSelectedChainStore>,
        replay: LocalIssuanceReplay<'_>,
        checkpoint: &IssuanceCheckpoint,
    ) -> StoreResult<()> {
        let result = (|| {
            let anchor = self.derive_anchor_from_local_replay(selected, replay)?;
            self.stage(batch, selected, anchor, checkpoint)?;
            let bytes = codec().serialize(&anchor).map_err(|error| inconsistent(error.to_string()))?;
            batch.reserve_unique_key(LOCAL_REPLAY_ANCHOR_KEY)?;
            batch.put_raw(&self.db, LOCAL_REPLAY_ANCHOR_KEY, bytes)
        })();
        if result.is_err() {
            batch.reject();
        }
        result
    }

    /// Recover a locally authenticated pair after the replay prefix is gone.
    /// The caller supplies network, cap and point from its trusted view.
    pub fn load_retained_replay_checkpoint(
        &self,
        selected: &RwLockReadGuard<'_, DbSelectedChainStore>,
        network_genesis: Hash,
        pruning_point: Hash,
        supply_cap: u64,
    ) -> StoreResult<IssuanceCheckpoint> {
        // Read both records at one sequence number even if another reader
        // holds the same selected-chain lock while committing an authority batch.
        let snapshot = self.db.snapshot();
        let anchor_bytes =
            snapshot.get(LOCAL_REPLAY_ANCHOR_KEY)?.ok_or_else(|| inconsistent("missing locally verified issuance anchor"))?;
        let anchor: IssuanceCheckpointAnchor = codec().deserialize(&anchor_bytes).map_err(|error| inconsistent(error.to_string()))?;
        if anchor.network_id != network_genesis || anchor.pruning_point != pruning_point || anchor.supply_cap != supply_cap {
            return Err(inconsistent("locally verified issuance anchor has a different network, point or cap"));
        }
        let checkpoint_bytes = snapshot.get(CHECKPOINT_KEY)?.ok_or_else(|| inconsistent("missing retained issuance checkpoint"))?;
        let checkpoint = codec().deserialize(&checkpoint_bytes).map_err(|error| inconsistent(error.to_string()))?;
        self.verify(selected, anchor, &checkpoint)?;
        Ok(checkpoint)
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

    /// Stage a reconciled successor of the currently persisted checkpoint.
    /// The caller serializes authority writes and supplies an independently
    /// authenticated commitment for each point in the same selected view.
    pub fn stage_reconciled_transition<'guard>(
        &self,
        batch: &mut DeferredDbBatch<'guard>,
        selected: &'guard RwLockReadGuard<'_, DbSelectedChainStore>,
        previous_anchor: IssuanceCheckpointAnchor,
        next_anchor: IssuanceCheckpointAnchor,
        checkpoint: &IssuanceCheckpoint,
        settlement: &CoinbaseSettlement,
    ) -> StoreResult<()> {
        let result = (|| {
            if previous_anchor.selected_index.checked_add(1) != Some(next_anchor.selected_index)
                || previous_anchor.supply_cap != next_anchor.supply_cap
            {
                return Err(inconsistent("settlement checkpoint anchors are not consecutive under the same cap"));
            }
            let previous =
                self.load(selected, previous_anchor)?.ok_or_else(|| inconsistent("missing persisted settlement predecessor"))?;
            if !settlement.matches_checkpoint_transition(&previous, checkpoint) {
                return Err(inconsistent("checkpoint changes disagree with reconciled coinbase settlement"));
            }
            self.stage(batch, selected, next_anchor, checkpoint)
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
    use kaspa_consensus_core::ChainPath;
    use kaspa_database::prelude::{BatchDbWriter, ConnBuilder};
    use parking_lot::RwLock;
    use rocksdb::WriteBatch;

    const CAP: u64 = 2_900_000_000_000_000_000;
    const BATCH_PROBE: &[u8] = &[DatabaseStorePrefixes::KascadeBranchAuthorityData as u8, 0xf2];

    fn fixture(network: Hash, point: Hash) -> (IssuanceCheckpointAnchor, IssuanceCheckpoint) {
        let claim =
            IssuanceClaim { reward_source: Hash::from_u64_word(9), settlement_block: point, selected_index: 1, minted: 100, paid: 85 };
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

    #[test]
    fn issuance_checkpoint_replay_anchor_rejects_substitution_and_reorg() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let [genesis, parent, tip, sibling, source] = [1, 2, 3, 4, 5].map(Hash::from_u64_word);
        let path = [genesis, parent, tip];
        let claim = IssuanceClaim { reward_source: source, settlement_block: parent, selected_index: 1, minted: CAP, paid: CAP - 20 };
        let release = DeveloperPayoutRelease { settlement_block: tip, selected_index: 2, amount: 20 };
        let claims = [claim];
        let releases = [release];
        let replay = || LocalIssuanceReplay {
            network_genesis: genesis,
            supply_cap: CAP,
            selected_path: &path,
            claims: &claims,
            releases: &releases,
        };
        let checkpoint = IssuanceCheckpoint::derive_from_settlements(genesis, tip, CAP, &path, &claims, &releases).unwrap();

        let db = connect();
        let mut chain = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
        let mut init = WriteBatch::default();
        chain.init_with_pruning_point(&mut init, genesis).unwrap();
        db.write(init).unwrap();
        let mut add = WriteBatch::default();
        chain.apply_changes(&mut add, &ChainPath { removed: vec![], added: vec![parent, tip] }).unwrap();
        db.write(add).unwrap();
        let selected = RwLock::new(chain);
        let store = DbIssuanceCheckpoint::new(db.clone());
        {
            let guard = selected.read();
            let anchor = store.derive_anchor_from_local_replay(&guard, replay()).unwrap();
            let altered = IssuanceClaim { reward_source: sibling, ..claim };
            let substituted = IssuanceCheckpoint::derive_from_settlements(genesis, tip, CAP, &path, &[altered], &releases).unwrap();
            assert_eq!((substituted.minted, substituted.paid, substituted.deferred), (CAP, CAP, 0));
            assert_ne!(substituted.commitment(), anchor.expected_commitment);

            let mut rejected = DeferredDbBatch::new(db.clone());
            rejected.put_raw(&db, BATCH_PROBE, b"must not publish").unwrap();
            assert!(store.stage(&mut rejected, &guard, anchor, &substituted).is_err());
            assert!(rejected.commit().is_err());
            assert!(db.get(BATCH_PROBE).unwrap().is_none());
            assert!(store.load(&guard, anchor).unwrap().is_none());

            let mut abandoned = DeferredDbBatch::new(db.clone());
            store.stage(&mut abandoned, &guard, anchor, &checkpoint).unwrap();
            drop(abandoned);
            assert!(store.load(&guard, anchor).unwrap().is_none());

            let mut batch = DeferredDbBatch::new(db.clone());
            store.stage(&mut batch, &guard, anchor, &checkpoint).unwrap();
            batch.commit().unwrap();
        }
        drop(store);
        drop(selected);
        drop(db);

        let db = connect();
        let selected = RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
        let store = DbIssuanceCheckpoint::new(db.clone());
        {
            let guard = selected.read();
            let anchor = store.derive_anchor_from_local_replay(&guard, replay()).unwrap();
            assert_eq!(store.load(&guard, anchor).unwrap(), Some(checkpoint));
        }
        {
            let mut chain = selected.write();
            let mut reorg = WriteBatch::default();
            chain.apply_changes(&mut reorg, &ChainPath { removed: vec![tip], added: vec![sibling] }).unwrap();
            db.write(reorg).unwrap();
        }
        let guard = selected.read();
        assert!(store.derive_anchor_from_local_replay(&guard, replay()).is_err());
        let replacement_path = [genesis, parent, sibling];
        let replacement_releases = [DeveloperPayoutRelease { settlement_block: sibling, ..release }];
        let replacement_replay = LocalIssuanceReplay { selected_path: &replacement_path, releases: &replacement_releases, ..replay() };
        let anchor = store.derive_anchor_from_local_replay(&guard, replacement_replay).unwrap();
        assert!(store.load(&guard, anchor).is_err());
        let replacement =
            IssuanceCheckpoint::derive_from_settlements(genesis, sibling, CAP, &replacement_path, &claims, &replacement_releases)
                .unwrap();
        let mut batch = DeferredDbBatch::new(db);
        store.stage(&mut batch, &guard, anchor, &replacement).unwrap();
        batch.commit().unwrap();
        assert_eq!(store.load(&guard, anchor).unwrap(), Some(replacement));
    }

    #[test]
    fn retained_issuance_anchor_recovers_after_pruning_and_rejects_stale_selection() {
        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let [genesis, parent, point, sibling, source] = [1, 2, 3, 4, 5].map(Hash::from_u64_word);
        let path = [genesis, parent, point];
        let claims =
            [IssuanceClaim { reward_source: source, settlement_block: parent, selected_index: 1, minted: CAP, paid: CAP - 20 }];
        let releases = [DeveloperPayoutRelease { settlement_block: point, selected_index: 2, amount: 20 }];
        let replay = || LocalIssuanceReplay {
            network_genesis: genesis,
            supply_cap: CAP,
            selected_path: &path,
            claims: &claims,
            releases: &releases,
        };
        let checkpoint = IssuanceCheckpoint::derive_from_settlements(genesis, point, CAP, &path, &claims, &releases).unwrap();

        {
            let db = connect();
            let mut chain = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
            let mut init = WriteBatch::default();
            chain.init_with_pruning_point(&mut init, genesis).unwrap();
            db.write(init).unwrap();
            let mut add = WriteBatch::default();
            chain.apply_changes(&mut add, &ChainPath { removed: vec![], added: vec![parent, point] }).unwrap();
            db.write(add).unwrap();

            let selected = RwLock::new(chain);
            let store = DbIssuanceCheckpoint::new(db.clone());
            {
                let guard = selected.read();
                let mut missing = DeferredDbBatch::new(db.clone());
                let ordinary_anchor = store.derive_anchor_from_local_replay(&guard, replay()).unwrap();
                store.stage(&mut missing, &guard, ordinary_anchor, &checkpoint).unwrap();
                missing.commit().unwrap();
                assert!(store.load_retained_replay_checkpoint(&guard, genesis, point, CAP).is_err());

                let mut rejected = DeferredDbBatch::new(db.clone());
                let forged = IssuanceCheckpoint { ordered_claims_root: sibling, ..checkpoint };
                assert!(store.stage_retained_replay_checkpoint(&mut rejected, &guard, replay(), &forged).is_err());
                assert!(rejected.commit().is_err());
                assert!(db.get(LOCAL_REPLAY_ANCHOR_KEY).unwrap().is_none());

                let mut abandoned = DeferredDbBatch::new(db.clone());
                store.stage_retained_replay_checkpoint(&mut abandoned, &guard, replay(), &checkpoint).unwrap();
                drop(abandoned);
                assert!(db.get(LOCAL_REPLAY_ANCHOR_KEY).unwrap().is_none());

                let mut batch = DeferredDbBatch::new(db.clone());
                store.stage_retained_replay_checkpoint(&mut batch, &guard, replay(), &checkpoint).unwrap();
                batch.commit().unwrap();
            }
            let mut chain = selected.write();
            let mut prune = WriteBatch::default();
            chain.prune_below_point(BatchDbWriter::new(&mut prune), parent).unwrap();
            db.write(prune).unwrap();
        }

        let db = connect();
        let selected = RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
        let store = DbIssuanceCheckpoint::new(db.clone());
        {
            let guard = selected.read();
            assert!(store.derive_anchor_from_local_replay(&guard, replay()).is_err());
            assert_eq!(store.load_retained_replay_checkpoint(&guard, genesis, point, CAP).unwrap(), checkpoint);
            assert!(store.load_retained_replay_checkpoint(&guard, sibling, point, CAP).is_err());
            assert!(store.load_retained_replay_checkpoint(&guard, genesis, point, CAP - 1).is_err());
        }
        {
            let mut chain = selected.write();
            let mut reorg = WriteBatch::default();
            chain.apply_changes(&mut reorg, &ChainPath { removed: vec![point], added: vec![sibling] }).unwrap();
            db.write(reorg).unwrap();
        }
        let guard = selected.read();
        assert!(store.load_retained_replay_checkpoint(&guard, genesis, point, CAP).is_err());
        assert!(store.load_retained_replay_checkpoint(&guard, genesis, sibling, CAP).is_err());
    }
}
