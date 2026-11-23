//! Candidate issuance checkpoint commitment for a future consensus-authenticated restore.
//! The expected digest must come from verified consensus evidence independent of the checkpoint.

use kaspa_hashes::{Hash, HasherBase, TransactionHash};

const DOMAIN: &[u8] = b"KASCADE/issuance-checkpoint/v1";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationError {
    WrongContext,
    AboveSupplyCap,
    UnbalancedSettlement,
    CommitmentMismatch,
}

impl IssuanceCheckpoint {
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
}
