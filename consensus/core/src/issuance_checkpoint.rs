//! Candidate issuance checkpoint commitment for a future consensus-authenticated restore.
//! The expected digest must come from verified consensus evidence independent of the checkpoint.

use kaspa_hashes::{Hash, HasherBase, TransactionHash};
use kaspa_merkle::calc_merkle_root;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const DOMAIN: &[u8] = b"KASCADE/issuance-checkpoint/v1";
const CLAIM_DOMAIN: &[u8] = b"KASCADE/issuance-claim/v1";
const RELEASE_DOMAIN: &[u8] = b"KASCADE/issuance-developer-release/v1";

/// Fixed-size summary of one selected pruning point's issuance and settlement.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct IssuanceCheckpoint {
    pub network_id: Hash,
    pub pruning_point: Hash,
    pub ordered_claims_root: Hash,
    pub claim_count: u64,
    pub minted: u64,
    pub paid: u64,
    pub deferred: u64,
}

/// Size of the proposed versioned issuance field. This codec is inactive:
/// no transaction parser, block builder or validation rule uses it.
pub const ISSUANCE_COMMITMENT_FIELD_SIZE: usize = 34;
const ISSUANCE_COMMITMENT_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssuanceCommitmentField {
    digest: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssuanceFieldError {
    WrongLength,
    UnsupportedVersion,
}

impl IssuanceCommitmentField {
    /// Construct from a locally verified checkpoint before placing it in a
    /// proposal. The caller owns verification of the underlying settlement.
    pub fn from_checkpoint(checkpoint: &IssuanceCheckpoint) -> Self {
        Self { digest: checkpoint.commitment() }
    }

    pub fn encode(self) -> [u8; ISSUANCE_COMMITMENT_FIELD_SIZE] {
        let mut field = [0; ISSUANCE_COMMITMENT_FIELD_SIZE];
        field[..2].copy_from_slice(&ISSUANCE_COMMITMENT_VERSION.to_le_bytes());
        field[2..].copy_from_slice(&self.digest.as_bytes());
        field
    }

    /// Decode only the exact field, without scanning or advancing a payload.
    /// Decoding establishes syntax. The caller authenticates the bytes and
    /// establishes the selected parent through independently verified evidence.
    pub fn decode(bytes: &[u8]) -> Result<Self, IssuanceFieldError> {
        let field: &[u8; ISSUANCE_COMMITMENT_FIELD_SIZE] = bytes.try_into().map_err(|_| IssuanceFieldError::WrongLength)?;
        if u16::from_le_bytes([field[0], field[1]]) != ISSUANCE_COMMITMENT_VERSION {
            return Err(IssuanceFieldError::UnsupportedVersion);
        }
        let mut digest = [0; 32];
        digest.copy_from_slice(&field[2..]);
        Ok(Self { digest: Hash::from_bytes(digest) })
    }

    /// Compare a checkpoint against an independently authenticated field. The
    /// expected parent must be the selected parent and the intended pruning
    /// point. The caller validates ancestry and the historical settlement.
    pub fn verify_checkpoint(
        self,
        network: Hash,
        selected_parent: Hash,
        supply_cap: u64,
        checkpoint: &IssuanceCheckpoint,
    ) -> Result<(), VerificationError> {
        checkpoint.verify(network, selected_parent, supply_cap, self.digest)
    }

    /// Compare the field to a checkpoint derived entirely from a bounded,
    /// locally verified settlement replay. The caller supplies the selected
    /// path and consensus-validated reward and release records independently
    /// of the block carrying this field. No import checkpoint enters here.
    pub fn verify_local_settlement(
        self,
        network: Hash,
        selected_parent: Hash,
        supply_cap: u64,
        selected_path: &[Hash],
        claims: &[IssuanceClaim],
        releases: &[DeveloperPayoutRelease],
    ) -> Result<IssuanceCheckpoint, VerificationError> {
        if selected_path.first() != Some(&network) {
            return Err(VerificationError::WrongContext);
        }
        let local =
            IssuanceCheckpoint::derive_from_settlements(network, selected_parent, supply_cap, selected_path, claims, releases)?;
        self.verify_checkpoint(network, selected_parent, supply_cap, &local)?;
        Ok(local)
    }
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

/// Payment of already accounted developer issuance at a selected block.
/// A release is unique by selected block and contributes no new subsidy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeveloperPayoutRelease {
    pub settlement_block: Hash,
    pub selected_index: u64,
    pub amount: u64,
}

impl DeveloperPayoutRelease {
    fn hash(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(RELEASE_DOMAIN);
        hasher.update(self.settlement_block.as_bytes());
        hasher.update(self.selected_index.to_le_bytes());
        hasher.update(self.amount.to_le_bytes());
        hasher.finalize()
    }
}

#[derive(Clone, Copy)]
enum SettlementEvent<'a> {
    Claim(&'a IssuanceClaim),
    Release(&'a DeveloperPayoutRelease),
}

impl SettlementEvent<'_> {
    fn key(self) -> (u64, u8, Hash) {
        match self {
            Self::Claim(claim) => (claim.selected_index, 0, claim.reward_source),
            Self::Release(release) => (release.selected_index, 1, Hash::default()),
        }
    }

    fn hash(self) -> Hash {
        match self {
            Self::Claim(claim) => claim.hash(),
            Self::Release(release) => release.hash(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationError {
    WrongContext,
    AboveSupplyCap,
    UnbalancedSettlement,
    CommitmentMismatch,
    DuplicateRewardSource,
    DuplicateDeveloperRelease,
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
        Self::derive_events(network_id, pruning_point, supply_cap, selected_path, claims, &[])
    }

    /// Derive a checkpoint from source claims and developer releases. Claims
    /// accrue before the release at the same selected index. Release amounts
    /// are drawn from deferred liability and included in the same Merkle root.
    /// Callers must obtain both the path and expected digest independently.
    pub fn derive_from_settlements(
        network_id: Hash,
        pruning_point: Hash,
        supply_cap: u64,
        selected_path: &[Hash],
        claims: &[IssuanceClaim],
        releases: &[DeveloperPayoutRelease],
    ) -> Result<Self, VerificationError> {
        if claims.iter().any(|claim| claim.paid > claim.minted) {
            return Err(VerificationError::UnbalancedSettlement);
        }
        Self::derive_events(network_id, pruning_point, supply_cap, selected_path, claims, releases)
    }

    fn derive_events(
        network_id: Hash,
        pruning_point: Hash,
        supply_cap: u64,
        selected_path: &[Hash],
        claims: &[IssuanceClaim],
        releases: &[DeveloperPayoutRelease],
    ) -> Result<Self, VerificationError> {
        if selected_path.last() != Some(&pruning_point) {
            return Err(VerificationError::WrongContext);
        }
        let mut ordered =
            claims.iter().map(SettlementEvent::Claim).chain(releases.iter().map(SettlementEvent::Release)).collect::<Vec<_>>();
        ordered.sort_unstable_by_key(|event| event.key());
        let mut seen = HashSet::with_capacity(claims.len());
        let mut last_release = None;
        let mut minted = 0u64;
        let mut paid = 0u64;
        for (position, event) in ordered.iter().copied().enumerate() {
            let (selected_index, _, _) = event.key();
            let index = usize::try_from(selected_index).map_err(|_| VerificationError::OffSelectedPath)?;
            let block = match event {
                SettlementEvent::Claim(claim) => claim.settlement_block,
                SettlementEvent::Release(release) => release.settlement_block,
            };
            if selected_path.get(index) != Some(&block) {
                return Err(VerificationError::OffSelectedPath);
            }
            match event {
                SettlementEvent::Claim(claim) => {
                    if !seen.insert(claim.reward_source) {
                        return Err(VerificationError::DuplicateRewardSource);
                    }
                    minted = minted.checked_add(claim.minted).ok_or(VerificationError::AboveSupplyCap)?;
                    if minted > supply_cap {
                        return Err(VerificationError::AboveSupplyCap);
                    }
                    paid = paid.checked_add(claim.paid).ok_or(VerificationError::UnbalancedSettlement)?;
                }
                SettlementEvent::Release(release) => {
                    if last_release == Some(selected_index) {
                        return Err(VerificationError::DuplicateDeveloperRelease);
                    }
                    last_release = Some(selected_index);
                    if release.amount == 0 || minted.checked_sub(paid).is_none_or(|balance| release.amount > balance) {
                        return Err(VerificationError::UnbalancedSettlement);
                    }
                    paid = paid.checked_add(release.amount).ok_or(VerificationError::UnbalancedSettlement)?;
                }
            }
            if paid > minted && ordered.get(position + 1).is_none_or(|next| next.key().0 != selected_index) {
                return Err(VerificationError::UnbalancedSettlement);
            }
        }
        let deferred = minted.checked_sub(paid).ok_or(VerificationError::UnbalancedSettlement)?;
        let claim_count = u64::try_from(claims.len()).map_err(|_| VerificationError::UnbalancedSettlement)?;
        let ordered_claims_root = calc_merkle_root(ordered.into_iter().map(SettlementEvent::hash));
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
    fn inactive_issuance_field_has_canonical_encoding() {
        let saved = checkpoint();
        let field = IssuanceCommitmentField::from_checkpoint(&saved);
        let mut wire = field.encode();
        assert_eq!(&wire[..2], &[1, 0]);
        assert_eq!(&wire[2..], &saved.commitment().as_bytes());
        assert_eq!(IssuanceCommitmentField::decode(&wire), Ok(field));
        assert_eq!(IssuanceCommitmentField::decode(&wire[..33]), Err(IssuanceFieldError::WrongLength));
        assert_eq!(IssuanceCommitmentField::decode(&[0; 35]), Err(IssuanceFieldError::WrongLength));
        wire[1] = 1;
        assert_eq!(IssuanceCommitmentField::decode(&wire), Err(IssuanceFieldError::UnsupportedVersion));
    }

    #[test]
    fn inactive_issuance_field_rejects_context_and_equal_total_substitution() {
        let saved = checkpoint();
        let wire = IssuanceCommitmentField::from_checkpoint(&saved).encode();
        let field = IssuanceCommitmentField::decode(&wire).unwrap();
        assert_eq!(field.verify_checkpoint(saved.network_id, saved.pruning_point, CAP, &saved), Ok(()));
        assert_eq!(
            field.verify_checkpoint(Hash::from_u64_word(7), saved.pruning_point, CAP, &saved),
            Err(VerificationError::WrongContext)
        );
        assert_eq!(
            field.verify_checkpoint(saved.network_id, Hash::from_u64_word(7), CAP, &saved),
            Err(VerificationError::WrongContext)
        );
        let substituted = IssuanceCheckpoint { ordered_claims_root: Hash::from_u64_word(7), ..saved };
        assert_eq!(
            field.verify_checkpoint(saved.network_id, saved.pruning_point, CAP, &substituted),
            Err(VerificationError::CommitmentMismatch)
        );
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
        assert_eq!(IssuanceCheckpoint::derive_from_claims(genesis, stale, CAP, &path, &[a, b]), Err(VerificationError::WrongContext));
        assert_eq!(derive(&[IssuanceClaim { minted: CAP, ..a }, b]), Err(VerificationError::AboveSupplyCap));
        assert_eq!(derive(&[IssuanceClaim { paid: 101, ..a }]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(derive(&[IssuanceClaim { paid: 101, ..a }, b]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(
            IssuanceCheckpoint::derive_from_claims(genesis, point, u64::MAX, &path, &[IssuanceClaim { minted: u64::MAX, ..a }, b]),
            Err(VerificationError::AboveSupplyCap)
        );
        assert_eq!(derive(&[IssuanceClaim { paid: u64::MAX, ..a }, b]), Err(VerificationError::UnbalancedSettlement));
    }

    #[test]
    fn commits_developer_releases_without_minting_or_replaying_stale_liabilities() {
        let [genesis, parent, point, sibling, source] = [1, 2, 3, 4, 5].map(Hash::from_u64_word);
        let path = [genesis, parent, point];
        let claim = IssuanceClaim { reward_source: source, settlement_block: parent, selected_index: 1, minted: CAP, paid: CAP - 40 };
        let early = DeveloperPayoutRelease { settlement_block: parent, selected_index: 1, amount: 10 };
        let late = DeveloperPayoutRelease { settlement_block: point, selected_index: 2, amount: 30 };
        let derive = |releases: &[DeveloperPayoutRelease]| {
            IssuanceCheckpoint::derive_from_settlements(genesis, point, CAP, &path, &[claim], releases)
        };
        let checkpoint = derive(&[early, late]).unwrap();
        assert_eq!(derive(&[late, early]), Ok(checkpoint));
        assert_eq!((checkpoint.claim_count, checkpoint.minted, checkpoint.paid, checkpoint.deferred), (1, CAP, CAP, 0));
        assert_eq!(derive(&[early, early]), Err(VerificationError::DuplicateDeveloperRelease));
        assert_eq!(derive(&[early, DeveloperPayoutRelease { amount: 31, ..late }]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(derive(&[DeveloperPayoutRelease { amount: 0, ..early }]), Err(VerificationError::UnbalancedSettlement));
        assert_eq!(derive(&[DeveloperPayoutRelease { settlement_block: sibling, ..late }]), Err(VerificationError::OffSelectedPath));
        let first_claim = IssuanceClaim { minted: CAP - 10, ..claim };
        let embedded_release =
            IssuanceClaim { reward_source: sibling, settlement_block: point, selected_index: 2, minted: 10, paid: 11 };
        assert_eq!(
            IssuanceCheckpoint::derive_from_settlements(genesis, point, CAP, &path, &[first_claim, embedded_release], &[]),
            Err(VerificationError::UnbalancedSettlement)
        );

        let reordered_payment =
            derive(&[DeveloperPayoutRelease { amount: 20, ..early }, DeveloperPayoutRelease { amount: 20, ..late }]).unwrap();
        assert_eq!((reordered_payment.minted, reordered_payment.paid, reordered_payment.deferred), (CAP, CAP, 0));
        assert_eq!(reordered_payment.verify(genesis, point, CAP, checkpoint.commitment()), Err(VerificationError::CommitmentMismatch));

        let sibling_path = [genesis, parent, sibling];
        let restore = |releases: &[DeveloperPayoutRelease]| {
            IssuanceCheckpoint::derive_from_settlements(genesis, sibling, CAP, &sibling_path, &[claim], releases)
        };
        assert_eq!(restore(&[early, late]), Err(VerificationError::OffSelectedPath));
        let replacement = restore(&[early, DeveloperPayoutRelease { settlement_block: sibling, ..late }]).unwrap();
        assert_eq!(replacement.verify(genesis, sibling, CAP, checkpoint.commitment()), Err(VerificationError::CommitmentMismatch));
    }
}
