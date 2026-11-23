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
        hasher.update(checkpoint_bytes(self));
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

// Fixture keys in a dedicated temporary database.
const CHECKPOINT_ROW: &[u8] = b"issuance-fixture/checkpoint/v1";
const READY_ROW: &[u8] = b"issuance-fixture/ready/v1";

fn checkpoint_bytes(checkpoint: &Checkpoint) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&checkpoint.network.as_bytes());
    bytes.extend_from_slice(&checkpoint.point.as_bytes());
    bytes.extend_from_slice(&checkpoint.balance.slots.to_le_bytes());
    bytes.extend_from_slice(&checkpoint.balance.paid.to_le_bytes());
    bytes.extend_from_slice(&checkpoint.balance.deferred.to_le_bytes());
    bytes.extend_from_slice(&u64::try_from(checkpoint.ordered_claims.len()).unwrap().to_le_bytes());
    for source in &checkpoint.ordered_claims {
        bytes.extend_from_slice(&source.as_bytes());
    }
    bytes
}

fn decode_checkpoint(bytes: &[u8]) -> Option<Checkpoint> {
    const HEADER: usize = 96;

    let network = Hash::from_bytes(bytes.get(..32)?.try_into().ok()?);
    let point = Hash::from_bytes(bytes.get(32..64)?.try_into().ok()?);
    let slots = u64::from_le_bytes(bytes.get(64..72)?.try_into().ok()?);
    let paid = u64::from_le_bytes(bytes.get(72..80)?.try_into().ok()?);
    let deferred = u64::from_le_bytes(bytes.get(80..88)?.try_into().ok()?);
    let count = u64::from_le_bytes(bytes.get(88..HEADER)?.try_into().ok()?);
    let count = usize::try_from(count).ok()?;
    if count > Checkpoint::MAX_FIXTURE_CLAIMS || bytes.len() != HEADER + count * 32 {
        return None;
    }

    let ordered_claims = bytes[HEADER..].chunks_exact(32).map(|source| Hash::from_bytes(source.try_into().unwrap())).collect();
    Some(Checkpoint { network, point, balance: Balance { slots, paid, deferred }, ordered_claims })
}

fn checkpoint_batch(checkpoint: &Checkpoint) -> rocksdb::WriteBatch {
    let mut batch = rocksdb::WriteBatch::default();
    batch.put(CHECKPOINT_ROW, checkpoint_bytes(checkpoint));
    batch.put(READY_ROW, checkpoint.commitment().as_bytes());
    batch
}

fn read_ready_checkpoint(db: &kaspa_database::prelude::DB, network: Hash, point: Hash, expected: Hash) -> Option<Checkpoint> {
    let ready = db.get(READY_ROW).unwrap()?;
    if ready.as_slice() != expected.as_bytes().as_slice() {
        return None;
    }
    let row = db.get(CHECKPOINT_ROW).unwrap()?;
    decode_checkpoint(&row)?.restore(network, point, expected)
}

#[test]
fn issuance_checkpoint_reopens_atomic_batch_against_independent_anchor() {
    use kaspa_database::prelude::ConnBuilder;

    let temp = kaspa_database::utils::get_kaspa_tempdir();
    let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
    let network = id(1);
    let first = cumulative_kazmi(1);
    let second = cumulative_kazmi(2) - first;
    let parent = Checkpoint::genesis(network).advance(id(3), &[id(2)], first - 5, 0, 5).unwrap();
    let child = parent.advance(id(5), &[id(4)], second + 5 + 10, 10, 0).unwrap();
    let parent_anchor = parent.commitment();
    let child_anchor = child.commitment();
    {
        let db = connect();
        db.write(checkpoint_batch(&parent)).unwrap();
        // Simulate interruption before the new checkpoint batch is submitted.
        drop(checkpoint_batch(&child));
    }
    {
        let db = connect();
        assert_eq!(read_ready_checkpoint(&db, network, parent.point, parent_anchor), Some(parent.clone()));
        assert!(read_ready_checkpoint(&db, network, child.point, child_anchor).is_none());
        db.write(checkpoint_batch(&child)).unwrap();
    }
    {
        let db = connect();
        assert!(read_ready_checkpoint(&db, network, parent.point, parent_anchor).is_none());
        assert_eq!(read_ready_checkpoint(&db, network, child.point, child_anchor), Some(child.clone()));

        let mut altered = child.clone();
        altered.balance.paid -= 1;
        altered.balance.deferred += 1;
        assert!(altered.balance.valid(), "this alteration conserves total issuance but changes settlement");
        db.put(CHECKPOINT_ROW, checkpoint_bytes(&altered)).unwrap();
        assert!(read_ready_checkpoint(&db, network, child.point, child_anchor).is_none());

        db.write(checkpoint_batch(&child)).unwrap();
        db.delete(READY_ROW).unwrap();
    }
    let db = connect();
    assert!(read_ready_checkpoint(&db, network, child.point, child_anchor).is_none());
}

#[test]
fn issuance_checkpoint_decoder_rejects_short_oversized_and_trailing_records() {
    let checkpoint = Checkpoint::genesis(id(1));
    let valid = checkpoint_bytes(&checkpoint);
    assert_eq!(decode_checkpoint(&valid), Some(checkpoint));
    for len in 0..valid.len() {
        assert!(decode_checkpoint(&valid[..len]).is_none());
    }
    let mut altered = valid.clone();
    altered[88..96].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(decode_checkpoint(&altered).is_none());
    let mut extended = valid;
    extended.push(0);
    assert!(decode_checkpoint(&extended).is_none());
}
