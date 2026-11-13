//! Shielded evidence for an isolated receiver or a guarded restart.
//!
//! Hold the pruning session, point, and metadata guards through this check and
//! readiness publication. The caller must authenticate the selected child on
//! the header chain independently of the imported body. A restart must also
//! pin the committed virtual and selected-chain view through publication.

use super::branch_readiness::ReadinessPoint;
use crate::{
    model::stores::{
        ghostdag::{DbGhostdagStore, GhostdagStoreReader},
        headers::{DbHeadersStore, HeaderStoreReader},
        selected_chain::{DbSelectedChainStore, SelectedChainStoreReader},
        shielded::{DbNullifierDiffStore, NullifierDiffStoreReader},
    },
    processes::shielded::{PruningPointShieldedMetadata, ShieldedStateManager},
};
use kaspa_consensus_core::{block::Block, hashing, merkle::calc_hash_merkle_root, zkas_state_binding::extract_state_root};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use kaspa_hashes::Hash;
use std::{collections::HashSet, sync::Arc};

pub(super) fn verify_pruning_shielded(db: &Arc<DB>, point: ReadinessPoint, child: &Block, trusted_child: Hash) -> StoreResult<()> {
    verify_pruning_shielded_at_tip(db, point, child, trusted_child, None)
}

pub(super) fn verify_pruning_shielded_at_tip(
    db: &Arc<DB>,
    point: ReadinessPoint,
    child: &Block,
    trusted_child: Hash,
    selected_tip: Option<(Hash, &Block)>,
) -> StoreResult<()> {
    let expected = authenticated_commitment(db, point.hash, child, trusted_child)?;
    let store = ShieldedStateManager::new(db.clone(), CachePolicy::Empty);
    // Isolated import completion must not materialize the global set merely to
    // issue or consume a lease. Descendant projection retains its stricter path.
    if selected_tip.is_none() || selected_tip.is_some_and(|(tip, _)| tip == point.hash) {
        if let Some((_, tip_child)) = selected_tip {
            if tip_child.hash() != trusted_child {
                return Err(inconsistent("restart supplied different children for the same state"));
            }
            let tip_commitment = authenticated_commitment(db, point.hash, tip_child, tip_child.hash())?;
            if tip_commitment != expected {
                return Err(inconsistent("restart commitment changed"));
            }
        }
        if let Some(metadata) = store.export_pruning_point_shielded(point.hash)? {
            ShieldedStateManager::verify_import_binding(&metadata, expected).map_err(StoreError::DataInconsistency)?;
            store.verify_import_rows(&metadata)?;
        } else if expected != PruningPointShieldedMetadata::empty_state_root()
            || store.state_root_at(point.hash)? != expected
            || store.nullifiers().iter_all().next().transpose()?.is_some()
        {
            return Err(inconsistent("committed shielded state does not match the authenticated empty commitment"));
        }
        return Ok(());
    }
    let mut nullifiers = store.nullifier_set_snapshot()?;
    if let Some((tip, tip_child)) = selected_tip {
        if tip == point.hash && tip_child.hash() != trusted_child {
            return Err(inconsistent("restart supplied different children for the same state"));
        }
        let tip_commitment = authenticated_commitment(db, tip, tip_child, tip_child.hash())?;
        verify_commitment(&store, tip, tip_commitment, &nullifiers)?;
        nullifiers = pruning_nullifiers(db, point.hash, trusted_child, tip, nullifiers)?;
    }
    verify_commitment(&store, point.hash, expected, &nullifiers)
}

fn authenticated_commitment(db: &Arc<DB>, point: Hash, child: &Block, trusted_child: Hash) -> StoreResult<[u8; 32]> {
    let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
    let header = headers.get_header(trusted_child)?;
    if trusted_child == point
        || header.hash != trusted_child
        || hashing::header::hash(&header) != trusted_child
        || child.hash() != trusted_child
        || hashing::header::hash(&child.header) != trusted_child
    {
        return Err(inconsistent("shielded commitment child does not match the authenticated header"));
    }

    let ghostdag = DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty);
    if ghostdag.get_selected_parent(trusted_child)? != point {
        return Err(inconsistent("shielded commitment child does not select the pruning point"));
    }
    let coinbase = child
        .transactions
        .first()
        .filter(|tx| tx.is_coinbase())
        .ok_or_else(|| inconsistent("shielded commitment child has no first coinbase"))?;
    if calc_hash_merkle_root(child.transactions.iter()) != header.hash_merkle_root {
        return Err(inconsistent("shielded commitment child body does not match the authenticated header"));
    }
    extract_state_root(&coinbase.payload).ok_or_else(|| inconsistent("missing authenticated shielded commitment"))
}

fn verify_commitment(store: &ShieldedStateManager, point: Hash, expected: [u8; 32], nullifiers: &[[u8; 32]]) -> StoreResult<()> {
    if let Some(metadata) = store.export_pruning_point_shielded(point)? {
        ShieldedStateManager::verify_import_binding(&metadata, expected).map_err(StoreError::DataInconsistency)?;
        ShieldedStateManager::verify_pruning_point_shielded(&metadata, nullifiers.iter()).map_err(StoreError::DataInconsistency)?;
    } else if expected != PruningPointShieldedMetadata::empty_state_root()
        || store.state_root_at(point)? != expected
        || !nullifiers.is_empty()
    {
        return Err(inconsistent("committed shielded state does not match the authenticated empty commitment"));
    }
    Ok(())
}

fn pruning_nullifiers(db: &Arc<DB>, point: Hash, child: Hash, tip: Hash, nullifiers: Vec<[u8; 32]>) -> StoreResult<Vec<[u8; 32]>> {
    let chain = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
    let point_index = chain.get_by_hash(point)?;
    let (tip_index, committed_tip) = chain.get_tip()?;
    if committed_tip != tip || point_index > tip_index || chain.get_by_index(point_index)? != point {
        return Err(inconsistent("committed selected chain does not match virtual restart state"));
    }

    let mut nullifiers: HashSet<_> = nullifiers.into_iter().collect();
    let diffs = DbNullifierDiffStore::new(db.clone(), CachePolicy::Empty);
    let ghostdag = DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty);
    let mut previous = point;
    for index in point_index..tip_index {
        let block = chain.get_by_index(index + 1)?;
        if chain.get_by_hash(block)? != index + 1
            || ghostdag.get_selected_parent(block)? != previous
            || (index == point_index && block != child)
        {
            return Err(inconsistent("restart descendants are not the authenticated selected chain"));
        }
        for nullifier in diffs.get(block)? {
            if !nullifiers.remove(&nullifier) {
                return Err(inconsistent("descendant nullifier is missing or repeated in committed membership"));
            }
        }
        previous = block;
    }
    Ok(nullifiers.into_iter().collect())
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::{
        ghostdag::{GhostdagData, GhostdagStore},
        headers::HeaderStore,
        shielded::NullifierSetStore,
    };
    use kaspa_consensus_core::{
        config::genesis::DEVNET_GENESIS, header::Header, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction,
        zkas_state_binding::SHIELDED_COMMITMENT_OFFSET,
    };
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};
    use rocksdb::WriteBatch;

    #[test]
    fn isolated_shielded_import_requires_committed_child_body_and_membership_evidence() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let point = ReadinessPoint { hash: Hash::from_bytes([7; 32]), index: 3 };
        let mut payload = vec![0; SHIELDED_COMMITMENT_OFFSET];
        payload.extend_from_slice(&PruningPointShieldedMetadata::empty_state_root());
        payload.extend_from_slice(&[0; 3]);
        let coinbase = Transaction::new(0, vec![], vec![], 0, SUBNETWORK_ID_COINBASE, 0, payload);
        let mut header = Header::from(&DEVNET_GENESIS);
        header.hash_merkle_root = calc_hash_merkle_root(std::iter::once(&coinbase));
        header.finalize();
        let trusted_child = header.hash;
        let child = Block::new(header.clone(), vec![coinbase.clone()]);

        assert!(verify_pruning_shielded(&db, point, &child, trusted_child).is_err());
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        headers.insert(trusted_child, Arc::new(header.clone()), 0).unwrap();
        assert!(verify_pruning_shielded(&db, point, &child, trusted_child).is_err());
        let ghostdag = DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty);
        ghostdag.insert(trusted_child, Arc::new(GhostdagData::new_with_selected_parent(point.hash, 5))).unwrap();
        verify_pruning_shielded(&db, point, &child, trusted_child).unwrap();
        let wrong_point = ReadinessPoint { hash: Hash::from_bytes([8; 32]), ..point };
        assert!(verify_pruning_shielded(&db, wrong_point, &child, trusted_child).is_err());

        let mut forged_coinbase = coinbase;
        forged_coinbase.payload[SHIELDED_COMMITMENT_OFFSET] ^= 1;
        forged_coinbase.finalize();
        let forged = Block::new(header, vec![forged_coinbase]);
        assert_eq!(forged.hash(), trusted_child);
        assert!(verify_pruning_shielded(&db, point, &forged, trusted_child).is_err());

        let store = ShieldedStateManager::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        store.nullifiers().insert_batch(&mut batch, [9; 32]).unwrap();
        db.write(batch).unwrap();
        assert!(verify_pruning_shielded(&db, point, &child, trusted_child).is_err());
    }
}
