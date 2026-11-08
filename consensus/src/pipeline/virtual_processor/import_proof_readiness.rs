//! Evidence from the existing pruning proof validator for inactive import completion.

use super::branch_readiness::ReadinessPoint;
use crate::{
    consensus::Consensus,
    model::stores::{
        headers::{DbHeadersStore, HeaderStoreReader},
        pruning::{DbPruningStore, PruningProofDescriptor, PruningStoreReader},
    },
};
use kaspa_consensus_core::{
    BlockHashSet,
    api::ConsensusApi,
    hashing,
    pruning::{PruningPointProof, PruningProofMetadata},
};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use std::sync::Arc;

/// Created by validating against the defending consensus before acquiring the
/// receiver's pruning guards. An imported descriptor alone cannot mint it.
pub(super) struct ValidatedImportProof {
    descriptor: PruningProofDescriptor,
    headers: BlockHashSet,
}

impl ValidatedImportProof {
    pub(super) fn authenticate(
        defender: &Consensus,
        proof: &PruningPointProof,
        metadata: &PruningProofMetadata,
    ) -> StoreResult<Self> {
        let evidence = Self::capture_validated(proof)?;
        defender.validate_pruning_proof(proof, metadata).map_err(|err| StoreError::DataInconsistency(err.to_string()))?;
        Ok(evidence)
    }

    fn capture_validated(proof: &PruningPointProof) -> StoreResult<Self> {
        let point = proof.first().and_then(|level| level.last()).ok_or_else(|| inconsistent("empty pruning proof"))?.hash;
        if proof.iter().any(Vec::is_empty) {
            return Err(inconsistent("pruning proof has an empty level"));
        }
        let mut headers = BlockHashSet::default();
        for header in proof.iter().flatten() {
            if hashing::header::hash(header) != header.hash {
                return Err(inconsistent("pruning proof header does not match its hash"));
            }
            headers.insert(header.hash);
        }
        Ok(Self { descriptor: PruningProofDescriptor::from_proof(proof, point, true), headers })
    }

    /// Corroborate the authenticated proof against fresh receiver DB reads.
    /// The receiver must retain the pruning guards through READY publication.
    pub(super) fn verify_committed(&self, db: &Arc<DB>, point: ReadinessPoint) -> StoreResult<()> {
        let actual = DbPruningStore::new(db.clone()).pruning_proof_descriptor()?;
        if point.hash != self.descriptor.pruning_point
            || actual.pruning_point != self.descriptor.pruning_point
            || actual.external != self.descriptor.external
            || actual.tips != self.descriptor.tips
            || actual.roots != self.descriptor.roots
            || actual.counts != self.descriptor.counts
        {
            return Err(inconsistent("committed pruning proof descriptor differs from the authenticated proof"));
        }

        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        for hash in &self.headers {
            let header = headers.get_header(*hash)?;
            if header.hash != *hash || hashing::header::hash(&header) != *hash {
                return Err(inconsistent("committed proof header does not match its authenticated hash"));
            }
        }
        Ok(())
    }
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        consensus::test_consensus::TestConsensus,
        model::stores::{headers::HeaderStore, pruning::PruningStore},
        pipeline::virtual_processor::persistent_branch_authority::PersistentBranchAuthority,
    };
    use kaspa_consensus_core::{
        block::Block,
        config::{ConfigBuilder, genesis::DEVNET_GENESIS, params::DEVNET_PARAMS},
        header::Header,
    };
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};

    #[test]
    fn authenticated_completion_rejects_missing_committed_import_metadata() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &ConfigBuilder::new(DEVNET_PARAMS).build(), sender);
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let point = ReadinessPoint { hash: header.hash, index: 3 };
        processor.pruning_point_store.write().set(point.hash, point.index).unwrap();

        let proof = vec![vec![Arc::new(header.clone())]];
        let proof = ValidatedImportProof::capture_validated(&proof).unwrap();
        let child = Block::new(header, vec![]);
        let owner = PersistentBranchAuthority::new(db);
        let token = owner.begin_import().unwrap();
        assert!(processor.finish_authenticated_authority_import(&owner, token, point, &proof, &child).is_err());
        assert!(processor.authority_lease(&owner).is_err());
    }

    #[test]
    fn matching_descriptor_cannot_substitute_for_committed_proof_headers() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let mut pruning = DbPruningStore::new(db.clone());
        let mut root = Header::from(&DEVNET_GENESIS);
        root.finalize();
        let mut middle = root.clone();
        middle.nonce += 1;
        middle.finalize();
        let mut tip = middle.clone();
        tip.nonce += 1;
        tip.finalize();

        let proof = vec![vec![Arc::new(root), Arc::new(middle), Arc::new(tip)]];
        let validated = ValidatedImportProof::capture_validated(&proof).unwrap();
        let point = ReadinessPoint { hash: proof[0][2].hash, index: 3 };

        headers.insert(proof[0][0].hash, proof[0][0].clone(), 0).unwrap();
        headers.insert(point.hash, proof[0][2].clone(), 0).unwrap();
        pruning.set_pruning_proof_descriptor(validated.descriptor.clone()).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());

        headers.insert(proof[0][1].hash, proof[0][1].clone(), 0).unwrap();
        validated.verify_committed(&db, point).unwrap();
        let mut substituted = validated.descriptor.clone();
        substituted.counts[0] += 1;
        pruning.set_pruning_proof_descriptor(substituted).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());
    }
}
