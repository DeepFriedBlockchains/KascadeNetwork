//! Candidate issuance checkpoint commitment for a future consensus-authenticated restore.
//! The expected digest must come from verified consensus evidence independent of the checkpoint.

use kaspa_hashes::{Hash, HasherBase, TransactionHash};
use kaspa_merkle::calc_merkle_root;
use std::collections::HashSet;

const DOMAIN: &[u8] = b"KASCADE/issuance-checkpoint/v1";
const CLAIM_DOMAIN: &[u8] = b"KASCADE/issuance-claim/v1";

/// Fixed-size summary of one selected pruning point's issuance and settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssuanceCheckpoint {
    pub network_id: Hash,
    pub pruning_point: Hash,
    pub ordered_claims_root: Hash,
    pub claim_count: u64,
    pub minted: u64,
    pub paid: u64,
    pub deferred: u64,
}

/// One unique reward source attributed to the selected block that settles it.
/// Paid amounts exclude recycled transaction fees and can include deferred
/// issuance from an earlier settlement. All amounts are in atomic units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssuanceClaim {
    pub reward_source: Hash,
    pub settlement_block: Hash,
    pub selected_index: u64,
    pub minted: u64,
    pub paid: u64,
}

impl IssuanceClaim {
    fn hash(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(CLAIM_DOMAIN);
        hasher.update(self.reward_source.as_bytes());
        hasher.update(self.settlement_block.as_bytes());
        hasher.update(self.selected_index.to_le_bytes());
        hasher.update(self.minted.to_le_bytes());
        hasher.update(self.paid.to_le_bytes());
        hasher.finalize()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationError {
    WrongContext,
    AboveSupplyCap,
    UnbalancedSettlement,
    CommitmentMismatch,
    DuplicateRewardSource,
    OffSelectedPath,
}

impl IssuanceCheckpoint {
    /// Derive a candidate from the complete claim set and an independently
    /// verified selected path indexed from genesis through the pruning point.
    ///
    /// Sort by selected-chain index, then reward source hash. This leaves the
    /// commitment independent of arrival order. The input claims must be
    /// attributable to actual consensus settlement. A restore caller must
    /// still pass the independently anchored digest to `verify`.
    ///
    /// This whole-history derivation is for bounded validation inputs. A live
    /// pruning implementation must retain an authenticated incremental claim
    /// view with reorganization support.
    pub fn derive_from_claims(
        network_id: Hash,
        pruning_point: Hash,
        supply_cap: u64,
        selected_path: &[Hash],
        claims: &[IssuanceClaim],
    ) -> Result<Self, VerificationError> {
        if selected_path.last() != Some(&pruning_point) {
            return Err(VerificationError::WrongContext);
        }
        let mut ordered = claims.iter().collect::<Vec<_>>();
        ordered.sort_unstable_by_key(|claim| (claim.selected_index, claim.reward_source));
        let mut seen = HashSet::with_capacity(ordered.len());
        let mut minted = 0u64;
        let mut paid = 0u64;
        for (position, claim) in ordered.iter().enumerate() {
            let index = usize::try_from(claim.selected_index).map_err(|_| VerificationError::OffSelectedPath)?;
            if selected_path.get(index) != Some(&claim.settlement_block) {
                return Err(VerificationError::OffSelectedPath);
            }
            if !seen.insert(claim.reward_source) {
                return Err(VerificationError::DuplicateRewardSource);
            }
            minted = minted.checked_add(claim.minted).ok_or(VerificationError::AboveSupplyCap)?;
            if minted > supply_cap {
                return Err(VerificationError::AboveSupplyCap);
            }
            paid = paid.checked_add(claim.paid).ok_or(VerificationError::UnbalancedSettlement)?;
            if paid > minted && ordered.get(position + 1).is_none_or(|next| next.selected_index != claim.selected_index) {
                return Err(VerificationError::UnbalancedSettlement);
            }
        }
        let deferred = minted.checked_sub(paid).ok_or(VerificationError::UnbalancedSettlement)?;
        let claim_count = u64::try_from(ordered.len()).map_err(|_| VerificationError::UnbalancedSettlement)?;
        let ordered_claims_root = calc_merkle_root(ordered.into_iter().map(IssuanceClaim::hash));
        Ok(Self { network_id, pruning_point, ordered_claims_root, claim_count, minted, paid, deferred })
    }

    /// Hash the v1 fields in their declared order, with amounts in atomic units
    /// and integer fields encoded as unsigned 64-bit little-endian values.
    pub fn commitment(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(DOMAIN);
        hasher.update(self.network_id.as_bytes());
        hasher.update(self.pruning_point.as_bytes());
        hasher.update(self.ordered_claims_root.as_bytes());
        hasher.update(self.claim_count.to_le_bytes());
        hasher.update(self.minted.to_le_bytes());
        hasher.update(self.paid.to_le_bytes());
        hasher.update(self.deferred.to_le_bytes());
        hasher.finalize()
    }

    /// Verify the checkpoint against a digest taken from authenticated consensus
    /// evidence. This checks the summary and binds its claim root, while the
    /// caller must establish the selected path and verify the underlying claims.
    pub fn verify(&self, network_id: Hash, pruning_point: Hash, supply_cap: u64, expected: Hash) -> Result<(), VerificationError> {
        if self.network_id != network_id || self.pruning_point != pruning_point {
            return Err(VerificationError::WrongContext);
        }
        if self.minted > supply_cap {
            return Err(VerificationError::AboveSupplyCap);
        }
        if self.paid.checked_add(self.deferred) != Some(self.minted) {
            return Err(VerificationError::UnbalancedSettlement);
        }
        if self.commitment() != expected {
            return Err(VerificationError::CommitmentMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: u64 = 2_900_000_000_000_000_000;

    fn checkpoint() -> IssuanceCheckpoint {
        IssuanceCheckpoint {
            network_id: Hash::from_u64_word(1),
            pruning_point: Hash::from_u64_word(2),
            ordered_claims_root: Hash::from_u64_word(3),
            claim_count: 5,
            minted: 200,
            paid: 180,
            deferred: 20,
        }
    }

    #[test]
    fn authenticates_summary_with_an_external_digest() {
        let original = checkpoint();
        let expected = original.commitment();
        let verify = |candidate: IssuanceCheckpoint| candidate.verify(original.network_id, original.pruning_point, CAP, expected);
        assert_eq!(verify(original), Ok(()));
        assert_eq!(
            verify(IssuanceCheckpoint { network_id: original.pruning_point, ..original }),
            Err(VerificationError::WrongContext)
        );
        assert_eq!(
            verify(IssuanceCheckpoint { pruning_point: original.network_id, ..original }),
            Err(VerificationError::WrongContext)
        );
        for altered in [
            IssuanceCheckpoint { paid: 179, deferred: 21, ..original },
            IssuanceCheckpoint { minted: 201, paid: 181, ..original },
            IssuanceCheckpoint { claim_count: 6, ..original },
            IssuanceCheckpoint { ordered_claims_root: Hash::from_u64_word(4), ..original },
        ] {
            assert_eq!(verify(altered), Err(VerificationError::CommitmentMismatch));
        }
    }

    #[test]
    fn rejects_unbalanced_overflowing_and_over_cap_summaries() {
        let original = checkpoint();
        let verify =
            |candidate: IssuanceCheckpoint| candidate.verify(original.network_id, original.pruning_point, CAP, candidate.commitment());
        let at_cap = IssuanceCheckpoint { minted: CAP, paid: CAP - 20, ..original };
        assert_eq!(verify(at_cap), Ok(()));
        assert_eq!(verify(IssuanceCheckpoint { minted: CAP + 1, paid: CAP - 19, ..original }), Err(VerificationError::AboveSupplyCap));
        assert_eq!(verify(IssuanceCheckpoint { paid: 181, ..original }), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(verify(IssuanceCheckpoint { paid: u64::MAX, ..original }), Err(VerificationError::UnbalancedSettlement));
    }

    #[test]
    fn derives_canonical_commitment_from_unique_selected_claims() {
        let [genesis, first, point, source_a, source_b, source_c, stale] = [1, 2, 3, 4, 5, 6, 7].map(Hash::from_u64_word);
        let path = [genesis, first, point];
        let a = IssuanceClaim { reward_source: source_a, settlement_block: first, selected_index: 1, minted: 100, paid: 80 };
        let b = IssuanceClaim { reward_source: source_b, settlement_block: point, selected_index: 2, minted: 50, paid: 60 };
        let c = IssuanceClaim { reward_source: source_c, settlement_block: point, selected_index: 2, minted: 50, paid: 40 };
        let derive = |claims: &[IssuanceClaim]| IssuanceCheckpoint::derive_from_claims(genesis, point, CAP, &path, claims);
        let canonical = derive(&[a, b, c]).unwrap();
        assert_eq!(derive(&[c, a, b]), Ok(canonical));
        assert_eq!((canonical.claim_count, canonical.minted, canonical.paid, canonical.deferred), (3, 200, 180, 20));
        let expected = canonical.commitment();
        assert_eq!(canonical.verify(genesis, point, CAP, expected), Ok(()));
        let altered = derive(&[IssuanceClaim { paid: 79, ..a }, b, c]).unwrap();
        assert_eq!(altered.verify(genesis, point, CAP, expected), Err(VerificationError::CommitmentMismatch));
        assert_eq!(derive(&[a, IssuanceClaim { reward_source: source_a, ..b }]), Err(VerificationError::DuplicateRewardSource));
        assert_eq!(derive(&[a, IssuanceClaim { settlement_block: stale, ..b }]), Err(VerificationError::OffSelectedPath));
        assert_eq!(derive(&[a, IssuanceClaim { selected_index: 1, ..b }]), Err(VerificationError::OffSelectedPath));
        assert_eq!(
            IssuanceCheckpoint::derive_from_claims(genesis, stale, CAP, &path, &[a, b]),
            Err(VerificationError::WrongContext)
        );
        assert_eq!(derive(&[IssuanceClaim { minted: CAP, ..a }, b]), Err(VerificationError::AboveSupplyCap));
        assert_eq!(derive(&[IssuanceClaim { paid: 101, ..a }]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(derive(&[IssuanceClaim { paid: 101, ..a }, b]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(
            IssuanceCheckpoint::derive_from_claims(genesis, point, u64::MAX, &path, &[IssuanceClaim { minted: u64::MAX, ..a }, b]),
            Err(VerificationError::AboveSupplyCap)
        );
        assert_eq!(derive(&[IssuanceClaim { paid: u64::MAX, ..a }, b]), Err(VerificationError::UnbalancedSettlement));
    }
}
