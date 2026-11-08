//! Shielded evidence for an isolated pruning import receiver.
//!
//! Hold the pruning session, point, and metadata guards through this check and
//! readiness publication. The caller must authenticate the selected child on
//! the header chain independently of the imported body. The receiver must have
//! no applied descendants because its global nullifier rows must represent the
//! pruning point itself.

use super::branch_readiness::ReadinessPoint;
use crate::{
    model::stores::{
        ghostdag::{DbGhostdagStore, GhostdagStoreReader},
        headers::{DbHeadersStore, HeaderStoreReader},
    },
    processes::shielded::{PruningPointShieldedMetadata, ShieldedStateManager},
};
use kaspa_consensus_core::{block::Block, hashing, merkle::calc_hash_merkle_root, zkas_state_binding::extract_state_root};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use kaspa_hashes::Hash;
use std::sync::Arc;

pub(super) fn verify_pruning_shielded(db: &Arc<DB>, point: ReadinessPoint, child: &Block, trusted_child: Hash) -> StoreResult<()> {
    let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
    let header = headers.get_header(trusted_child)?;
    if trusted_child == point.hash
        || header.hash != trusted_child
        || hashing::header::hash(&header) != trusted_child
        || child.hash() != trusted_child
        || hashing::header::hash(&child.header) != trusted_child
    {
        return Err(inconsistent("shielded commitment child does not match the authenticated header"));
    }

    let ghostdag = DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty);
    if ghostdag.get_selected_parent(trusted_child)? != point.hash {
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
    let expected = extract_state_root(&coinbase.payload).ok_or_else(|| inconsistent("missing authenticated shielded commitment"))?;

    let store = ShieldedStateManager::new(db.clone(), CachePolicy::Empty);
    let nullifiers = store.nullifier_set_snapshot()?;
    if let Some(metadata) = store.export_pruning_point_shielded(point.hash)? {
        ShieldedStateManager::verify_import_binding(&metadata, expected).map_err(StoreError::DataInconsistency)?;
        ShieldedStateManager::verify_pruning_point_shielded(&metadata, nullifiers.iter()).map_err(StoreError::DataInconsistency)?;
    } else if expected != PruningPointShieldedMetadata::empty_state_root()
        || store.state_root_at(point.hash)? != expected
        || !nullifiers.is_empty()
    {
        return Err(inconsistent("committed shielded state does not match the authenticated empty commitment"));
    }
    Ok(())
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
        config::genesis::DEVNET_GENESIS,
        header::Header,
        subnets::SUBNETWORK_ID_COINBASE,
        tx::Transaction,
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
