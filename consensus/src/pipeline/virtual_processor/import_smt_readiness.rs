//! Read-only corroboration of pruning-point SMT import data.
//!
//! The caller supplies the network configuration and a proof-authenticated
//! pruning point. It must hold the same pruning session, pruning-point and
//! metadata guards used for import completion, through readiness publication.

use super::branch_readiness::ReadinessPoint;
use crate::model::stores::{
    headers::{DbHeadersStore, HeaderStoreReader},
    smt_metadata::DbSmtMetadataStore,
};
use kaspa_consensus_core::{BlockHashSet, config::params::ForkActivation, hashing, header::Header};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use kaspa_hashes::{Hash, SeqCommitActiveCollapsedNode, SeqCommitActiveNode, ZERO_HASH};
use kaspa_seq_commit::verify::{SmtMetadata, verify_smt_metadata};
use kaspa_smt::{SmtHasher, hash_node, store::Node};
use kaspa_smt_store::processor::{SmtReadBounds, SmtStores};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CommittedPruningSmt {
    PreToccata,
    Active { lanes_root: Hash, lane_count: u64 },
}

pub(super) fn verify_pruning_smt(
    db: &Arc<DB>,
    point: ReadinessPoint,
    genesis: Hash,
    toccata_activation: ForkActivation,
    finality_depth: u64,
) -> StoreResult<CommittedPruningSmt> {
    let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
    let header = read_header(&headers, point.hash)?;
    if !toccata_activation.is_active(header.daa_score) {
        return Ok(CommittedPruningSmt::PreToccata);
    }
    if point.hash == genesis {
        return Err(inconsistent("active genesis has no imported SMT metadata"));
    }

    let parent = read_parent(&headers, &header)?;
    let bounds = SmtReadBounds::for_pov(header.blue_score, finality_depth);
    let mut canonical = BlockHashSet::default();
    canonical.insert(point.hash);
    let mut ancestor = parent.clone();

    // Each authenticated post-Toccata chain header commits its selected
    // parent as the first direct parent. Stop at the first pre-fork header
    // or below the active window, as the IBD shortcut derivation does.
    while toccata_activation.is_active(ancestor.daa_score) && ancestor.blue_score >= bounds.min_blue_score {
        canonical.insert(ancestor.hash);
        if ancestor.hash == genesis {
            break;
        }
        ancestor = read_parent(&headers, &ancestor)?;
    }
    let shortcut = if header.blue_score <= finality_depth { read_header(&headers, genesis)? } else { ancestor };
    let inactivity_shortcut =
        if toccata_activation.is_active(shortcut.daa_score) { shortcut.accepted_id_merkle_root } else { ZERO_HASH };

    let metadata = DbSmtMetadataStore::new(db.clone(), CachePolicy::Empty).get(point.hash)?;
    if metadata.inactivity_shortcut_block() != shortcut.hash {
        return Err(inconsistent("committed SMT shortcut does not match authenticated ancestry"));
    }

    // Imported versions use ZERO_HASH. Other versions must come from the
    // authenticated chain window. Both scans read persisted rows directly.
    let is_canonical = |hash| hash == ZERO_HASH || canonical.contains(&hash);
    let stores = SmtStores::new(db.clone(), 1, 1);
    let (lanes_root, lane_count) =
        stores.recompute_lanes_root_from_leaf_stream(bounds, metadata.active_lanes_count(), is_canonical)?;
    if lane_count != metadata.active_lanes_count() {
        return Err(inconsistent("committed SMT lane count does not match metadata"));
    }
    let branch_root = stores
        .branch_version
        .get_at_canonical(0, ZERO_HASH, bounds.target_blue_score, bounds.min_blue_score, is_canonical)?
        .and_then(|entry| *entry.data())
        .map_or_else(SeqCommitActiveNode::empty_root, |node| match node {
            Node::Internal(root) => root,
            Node::Collapsed(leaf) => hash_node::<SeqCommitActiveCollapsedNode>(leaf.lane_key, leaf.leaf_hash),
        });
    if lanes_root != branch_root {
        return Err(inconsistent("committed SMT branch root does not match canonical lane rows"));
    }
    let payload_and_ctx_digest = metadata.payload_and_ctx_digest();
    verify_smt_metadata(
        &SmtMetadata {
            lanes_root: &lanes_root,
            payload_and_ctx_digest: &payload_and_ctx_digest,
            parent_seq_commit: &parent.accepted_id_merkle_root,
        },
        inactivity_shortcut,
        header.accepted_id_merkle_root,
        parent.accepted_id_merkle_root,
    )
    .map_err(|error| StoreError::DataInconsistency(error.to_string()))?;

    Ok(CommittedPruningSmt::Active { lanes_root, lane_count })
}

fn read_header(headers: &DbHeadersStore, hash: Hash) -> StoreResult<Arc<Header>> {
    let header = headers.get_header(hash)?;
    if header.hash != hash || hashing::header::hash(&header) != hash {
        return Err(inconsistent("SMT header does not hash to its committed key"));
    }
    Ok(header)
}

fn read_parent(headers: &DbHeadersStore, child: &Header) -> StoreResult<Arc<Header>> {
    let hash =
        child.direct_parents().first().copied().ok_or_else(|| inconsistent("active SMT header is missing its selected parent"))?;
    let parent = read_header(headers, hash)?;
    if parent.blue_score >= child.blue_score {
        return Err(inconsistent("SMT parent blue score does not precede its child"));
    }
    Ok(parent)
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::{headers::HeaderStore, smt_metadata::SmtBlockMetadata};
    use kaspa_consensus_core::header::CompressedParents;
    use kaspa_database::{
        create_temp_db,
        prelude::{ConnBuilder, DirectDbWriter},
    };
    use kaspa_seq_commit::{
        hashing::{activity_root_hash, seq_commit, seq_state_root, smt_leaf_hash},
        types::{SeqCommitInput, SeqState, SmtLeafInput},
    };
    use kaspa_smt::store::CollapsedLeaf;
    use rocksdb::WriteBatch;

    fn header(parent: Option<Hash>, score: u64, seq_commit: Hash) -> Arc<Header> {
        let mut parents = CompressedParents::default();
        if let Some(parent) = parent {
            parents.push(vec![parent]);
        }
        Arc::new(Header::new_finalized(0, parents, ZERO_HASH, seq_commit, ZERO_HASH, 0, 0, 0, score, 0.into(), score, ZERO_HASH))
    }

    #[test]
    fn committed_lanes_and_branch_root_must_match_authenticated_seqcommit() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let genesis = header(None, 0, ZERO_HASH);
        let anchor = header(Some(genesis.hash), 2, Hash::from_bytes([2; 32]));
        let parent = header(Some(anchor.hash), 3, Hash::from_bytes([3; 32]));
        let lane_key = Hash::from_bytes([10; 32]);
        let lane_tip = Hash::from_bytes([11; 32]);
        let payload_and_ctx_digest = Hash::from_bytes([12; 32]);
        let leaf_hash = smt_leaf_hash(&SmtLeafInput { lane_tip: &lane_tip, blue_score: 4 });
        let lanes_root = hash_node::<SeqCommitActiveCollapsedNode>(lane_key, leaf_hash);
        let activity_root = activity_root_hash(&anchor.accepted_id_merkle_root, &lanes_root);
        let state_root = seq_state_root(&SeqState { activity_root: &activity_root, payload_and_ctx_digest: &payload_and_ctx_digest });
        let seqcommit = seq_commit(&SeqCommitInput { parent_seq_commit: &parent.accepted_id_merkle_root, state_root: &state_root });
        let pruning = header(Some(parent.hash), 4, seqcommit);
        let point = ReadinessPoint { hash: pruning.hash, index: 1 };
        for header in [&genesis, &anchor, &parent, &pruning] {
            headers.insert(header.hash, Arc::clone(header), 0).unwrap();
        }

        let verify = || verify_pruning_smt(&db, point, genesis.hash, ForkActivation::new(2), 1);
        let stores = SmtStores::new(db.clone(), 1, 1);
        stores.lane_version.put(DirectDbWriter::new(&db), lane_key, 4, ZERO_HASH, &lane_tip).unwrap();
        // This lane lies inside the score window but belongs to a side fork.
        stores
            .lane_version
            .put(DirectDbWriter::new(&db), Hash::from_bytes([14; 32]), 4, Hash::from_bytes([15; 32]), &lane_tip)
            .unwrap();
        stores
            .branch_version
            .put(DirectDbWriter::new(&db), 0, ZERO_HASH, 4, ZERO_HASH, Some(Node::Collapsed(CollapsedLeaf { lane_key, leaf_hash })))
            .unwrap();

        let metadata = DbSmtMetadataStore::new(db.clone(), CachePolicy::Count(1));
        let mut staged = WriteBatch::default();
        metadata.insert_batch(&mut staged, point.hash, SmtBlockMetadata::new(payload_and_ctx_digest, genesis.hash, 1)).unwrap();
        assert!(verify().is_err());
        db.write(staged).unwrap();
        assert!(verify().is_err()); // The legacy genesis row cannot replace the active anchor.

        let mut batch = WriteBatch::default();
        metadata.insert_batch(&mut batch, point.hash, SmtBlockMetadata::new(payload_and_ctx_digest, anchor.hash, 1)).unwrap();
        db.write(batch).unwrap();
        assert_eq!(verify().unwrap(), CommittedPruningSmt::Active { lanes_root, lane_count: 1 });

        // Alter both persisted representations consistently. Only the
        // authenticated header commitment detects this coordinated change.
        let changed_tip = Hash::from_bytes([13; 32]);
        let changed_leaf = smt_leaf_hash(&SmtLeafInput { lane_tip: &changed_tip, blue_score: 4 });
        stores.lane_version.put(DirectDbWriter::new(&db), lane_key, 4, ZERO_HASH, &changed_tip).unwrap();
        assert!(verify().is_err());
        stores
            .branch_version
            .put(
                DirectDbWriter::new(&db),
                0,
                ZERO_HASH,
                4,
                ZERO_HASH,
                Some(Node::Collapsed(CollapsedLeaf { lane_key, leaf_hash: changed_leaf })),
            )
            .unwrap();
        assert!(verify().is_err());
    }

    #[test]
    fn pre_toccata_header_does_not_require_imported_smt_rows() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let legacy = header(None, 0, Hash::from_bytes([8; 32]));
        headers.insert(legacy.hash, legacy.clone(), 0).unwrap();
        let point = ReadinessPoint { hash: legacy.hash, index: 0 };
        assert_eq!(verify_pruning_smt(&db, point, legacy.hash, ForkActivation::new(2), 1).unwrap(), CommittedPruningSmt::PreToccata);
        assert!(verify_pruning_smt(&db, point, legacy.hash, ForkActivation::always(), 1).is_err());
    }
}
