//! Test-only reference for issuance checkpoints. The retained claim set is a
//! bounded fixture oracle. Production must prove uniqueness using accepted DAG state.

use super::emission_candidate::{TOTAL_SLOTS, cumulative_kazmi};
use kaspa_hashes::{Hash, HasherBase, TransactionHash};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Balance {
    slots: u64,
    paid: u64,
    deferred: u64,
}

impl Balance {
    fn valid(&self) -> bool {
        self.slots <= TOTAL_SLOTS && self.paid.checked_add(self.deferred) == Some(cumulative_kazmi(self.slots))
    }

    fn settle(&self, eligible: u64, gross_outputs: u64, recycled_fees: u64, deferred: u64) -> Option<Self> {
        if !self.valid() {
            return None;
        }
        let slots = self.slots.checked_add(eligible)?.min(TOTAL_SLOTS);
        let paid = self.paid.checked_add(gross_outputs.checked_sub(recycled_fees)?)?;
        let next = Self { slots, paid, deferred };
        next.valid().then_some(next)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Checkpoint {
    network: Hash,
    point: Hash,
    balance: Balance,
    ordered_claims: Vec<Hash>,
}

impl Checkpoint {
    const MAX_FIXTURE_CLAIMS: usize = 64;

    fn genesis(network: Hash) -> Self {
        Self { network, point: network, balance: Balance::default(), ordered_claims: vec![] }
    }

    // The caller supplies source hashes after consensus determines eligibility
    // and order. This model rejects repeated source identities without sorting
    // the input, so changing its order changes the committed history.
    fn advance(&self, point: Hash, claims: &[Hash], gross_outputs: u64, fees: u64, deferred: u64) -> Option<Self> {
        if self.ordered_claims.len().checked_add(claims.len())? > Self::MAX_FIXTURE_CLAIMS {
            return None;
        }
        let mut seen: BTreeSet<_> = self.ordered_claims.iter().copied().collect();
        for source in claims {
            if *source == self.network || !seen.insert(*source) {
                return None;
            }
        }
        let balance = self.balance.settle(claims.len().try_into().ok()?, gross_outputs, fees, deferred)?;
        let mut ordered_claims = self.ordered_claims.clone();
        ordered_claims.extend_from_slice(claims);
        Some(Self { network: self.network, point, balance, ordered_claims })
    }

    fn commitment(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(b"KASCADE/issuance-checkpoint-fixture/v1");
        hasher.update(self.network.as_bytes());
        hasher.update(self.point.as_bytes());
        hasher.update(self.balance.slots.to_le_bytes());
        hasher.update(self.balance.paid.to_le_bytes());
        hasher.update(self.balance.deferred.to_le_bytes());
        hasher.update(u64::try_from(self.ordered_claims.len()).unwrap().to_le_bytes());
        for source in &self.ordered_claims {
            hasher.update(source.as_bytes());
        }
        hasher.finalize()
    }

    // Expected commitment must come from separately authenticated consensus
    // context. Importing a peer-provided commitment along with its data is
    // not authentication.
    fn restore(self, network: Hash, point: Hash, expected: Hash) -> Option<Self> {
        let distinct: BTreeSet<_> = self.ordered_claims.iter().copied().collect();
        (self.network == network
            && self.point == point
            && self.balance.valid()
            && self.ordered_claims.len() <= Self::MAX_FIXTURE_CLAIMS
            && distinct.len() == self.ordered_claims.len()
            && !distinct.contains(&network)
            && u64::try_from(distinct.len()).ok() == Some(self.balance.slots)
            && self.commitment() == expected)
            .then_some(self)
    }
}

fn id(word: u64) -> Hash {
    Hash::from_u64_word(word)
}

#[test]
fn issuance_checkpoint_rejects_duplicate_genesis_and_excess_payout() {
    let source = id(2);
    let first = cumulative_kazmi(1);
    let initial = Checkpoint::genesis(id(1));
    assert!(initial.advance(id(3), &[id(1)], first, 0, 0).is_none());
    assert!(initial.advance(id(3), &[source, source], cumulative_kazmi(2), 0, 0).is_none());
    assert!(initial.advance(id(3), &[source], first + 1, 0, 0).is_none());
    assert!(initial.advance(id(3), &[source], 0, 1, first).is_none());
    let accepted = initial.advance(id(3), &[source], first, 0, 0).unwrap();
    assert!(accepted.advance(id(4), &[source], cumulative_kazmi(2) - first, 0, 0).is_none());
}

#[test]
fn issuance_checkpoint_restores_branch_and_replays_deferred_settlement_once() {
    let network = id(1);
    let initial = Checkpoint::genesis(network);
    let first = cumulative_kazmi(1);
    let parent = initial.advance(id(3), &[id(2)], first + 17 - 5, 17, 5).unwrap();
    let expected = parent.commitment();
    let second = cumulative_kazmi(2) - first;
    let left = parent.advance(id(5), &[id(4)], second + 5, 0, 0).unwrap();
    let right = parent.advance(id(7), &[id(6)], second + 5, 0, 0).unwrap();
    assert_ne!(left.commitment(), right.commitment());
    assert_eq!(left.balance, right.balance);

    let mut altered = parent.clone();
    altered.balance.deferred += 1;
    altered.balance.paid -= 1;
    assert!(altered.balance.valid());
    assert!(altered.restore(network, id(3), expected).is_none());
    assert!(parent.clone().restore(id(10), id(3), expected).is_none());
    assert!(left.restore(network, id(3), expected).is_none());
    let restored = parent.restore(network, id(3), expected).unwrap();
    assert_eq!(restored.advance(id(7), &[id(6)], second + 5, 0, 0).unwrap(), right);
    assert!(right.advance(id(8), &[], 5, 0, 0).is_none());
}

#[test]
fn issuance_checkpoint_binds_order_and_caps_terminal_issuance() {
    let initial = Checkpoint::genesis(id(1));
    let paid = cumulative_kazmi(2);
    let checkpoint = initial.advance(id(5), &[id(2), id(3)], paid, 0, 0).unwrap();
    let reordered = initial.advance(id(5), &[id(3), id(2)], paid, 0, 0).unwrap();
    assert_eq!(reordered.balance, checkpoint.balance);
    assert!(reordered.restore(id(1), id(5), checkpoint.commitment()).is_none());

    let slots = TOTAL_SLOTS - 1;
    let paid = cumulative_kazmi(slots);
    let last = cumulative_kazmi(TOTAL_SLOTS) - paid;
    let near_cap = Balance { slots, paid, deferred: 0 };
    assert!(near_cap.settle(1, last + 1, 0, 0).is_none());
    let terminal = near_cap.settle(2, last, 0, 0).unwrap();
    assert_eq!(terminal.slots, TOTAL_SLOTS);
    assert_eq!(terminal.paid, 2_900_000_000_000_000_000);
    assert_eq!(terminal.settle(3, 7, 7, 0), Some(terminal.clone()));
    assert!(terminal.settle(1, 8, 7, 0).is_none());
    assert!(terminal.settle(u64::MAX, 0, 0, 0).is_none());
    assert!(terminal.settle(0, u64::MAX, 0, 0).is_none());
}
