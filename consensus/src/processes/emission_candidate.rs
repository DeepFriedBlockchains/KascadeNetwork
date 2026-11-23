//! Review-only 35-year issuance candidate for an ordinal of unique rewarded units.
//! No live coinbase consumer or network activation is wired to this module.

use kaspa_consensus_core::constants::SOMPI_PER_KASPA;

pub const SLOTS_PER_YEAR: u64 = 157_788_000;
pub const PRIMARY_SLOTS: u64 = 10 * SLOTS_PER_YEAR;
pub const TAIL_SLOTS: u64 = 25 * SLOTS_PER_YEAR;
pub const TOTAL_SLOTS: u64 = PRIMARY_SLOTS + TAIL_SLOTS;

pub const PRIMARY_KAZMI: u64 = 21_000_000_000 * SOMPI_PER_KASPA;
pub const TAIL_KAZMI: u64 = 8_000_000_000 * SOMPI_PER_KASPA;
const JOIN_KAZMI: u128 = 3 * SOMPI_PER_KASPA as u128;
const PRIMARY_N: u128 = PRIMARY_SLOTS as u128;
const TAIL_N: u128 = TAIL_SLOTS as u128;
const PRIMARY_EXCESS: u128 = PRIMARY_KAZMI as u128 - JOIN_KAZMI * PRIMARY_N;
const TAIL_SHORTFALL: u128 = JOIN_KAZMI * TAIL_N - TAIL_KAZMI as u128;

const PRIMARY: LinearPhase =
    LinearPhase::new(PRIMARY_N, JOIN_KAZMI * PRIMARY_N * PRIMARY_N + PRIMARY_EXCESS, 2 * PRIMARY_EXCESS, PRIMARY_KAZMI);
const TAIL: LinearPhase =
    LinearPhase::new(TAIL_N, JOIN_KAZMI * TAIL_N * TAIL_N - TAIL_SHORTFALL * (2 * TAIL_N - 1), 2 * TAIL_SHORTFALL, TAIL_KAZMI);

struct LinearPhase {
    slots: u128,
    denominator: u128,
    last_numerator: u128,
    step: u128,
    rounding_slots: u128,
}

impl LinearPhase {
    const fn new(slots: u128, last_numerator: u128, step: u128, budget: u64) -> Self {
        let denominator = slots * slots;
        let rounded_down = floor_sum(slots, denominator, step, last_numerator);
        let rounding_slots = budget as u128 - rounded_down;
        assert!(rounding_slots < slots);

        Self { slots, denominator, last_numerator, step, rounding_slots }
    }

    fn cumulative(&self, count: u64) -> u64 {
        let count = u128::from(count).min(self.slots);
        let first_numerator = self.last_numerator + (self.slots - count) * self.step;
        let rounded_down = floor_sum(count, self.denominator, self.step, first_numerator);
        (rounded_down + count.min(self.rounding_slots)) as u64
    }
}

// Sum floor((a*i + b)/m) for i in 0..n. First peel integer quotients,
// then transpose the residual lattice points. Each transpose reduces m.
const fn floor_sum(mut n: u128, mut m: u128, mut a: u128, mut b: u128) -> u128 {
    let mut total = 0;
    while n != 0 {
        total += n * (n - 1) / 2 * (a / m) + n * (b / m);
        a %= m;
        b %= m;
        let top = a * n + b;
        if top < m {
            break;
        }
        n = top / m;
        b = top % m;
        let previous_denominator = m;
        m = a;
        a = previous_denominator;
    }
    total
}

/// Scheduled total after `completed_slots` unique reward events.
/// The candidate starts from zero genesis allocation.
pub fn cumulative_kazmi(completed_slots: u64) -> u64 {
    PRIMARY.cumulative(completed_slots) + TAIL.cumulative(completed_slots.saturating_sub(PRIMARY_SLOTS))
}

/// Subsidy for a zero-based reward slot. All slots after exhaustion pay zero.
pub fn subsidy_kazmi(slot: u64) -> u64 {
    if slot >= TOTAL_SLOTS {
        return 0;
    }
    cumulative_kazmi(slot + 1) - cumulative_kazmi(slot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kaspa_consensus_core::{config::genesis::GENESIS, constants::MAX_SOMPI};

    #[test]
    fn emission_candidate_matches_exact_published_budgets() {
        assert_eq!(SOMPI_PER_KASPA, 100_000_000);
        assert_eq!(SLOTS_PER_YEAR, 5 * 86_400 * 365 + 5 * 21_600);
        assert_eq!(PRIMARY_KAZMI + TAIL_KAZMI, MAX_SOMPI);
        assert_eq!(cumulative_kazmi(0), 0);
        assert_eq!(cumulative_kazmi(PRIMARY_SLOTS), 2_100_000_000_000_000_000);
        assert_eq!(cumulative_kazmi(TOTAL_SLOTS), 2_900_000_000_000_000_000);
        assert_eq!(cumulative_kazmi(u64::MAX), MAX_SOMPI);
        assert_eq!(subsidy_kazmi(TOTAL_SLOTS), 0);
        assert_eq!(subsidy_kazmi(u64::MAX), 0);
    }

    #[test]
    fn emission_candidate_matches_boundary_vectors() {
        assert_eq!(subsidy_kazmi(0), 2_361_799_376);
        assert_eq!(subsidy_kazmi(PRIMARY_SLOTS - 1), 300_000_000);
        assert_eq!(subsidy_kazmi(PRIMARY_SLOTS), 300_000_000);
        assert_eq!(subsidy_kazmi(TOTAL_SLOTS - 1), 105_607_524);
        assert_eq!(PRIMARY.rounding_slots, 788_939_996);
        assert_eq!(TAIL.rounding_slots, 1_972_350_004);
    }

    #[test]
    fn emission_candidate_floor_sum_agrees_with_small_direct_oracle() {
        for n in 0..12 {
            for m in 1..12 {
                for a in 0..12 {
                    for b in 0..12 {
                        assert_eq!(floor_sum(n, m, a, b), (0..n).map(|i| (a * i + b) / m).sum::<u128>());
                    }
                }
            }
        }
    }

    #[test]
    fn emission_candidate_never_increases_at_sampled_slots_and_rounding_boundaries() {
        for slot in (0..TOTAL_SLOTS).step_by(1_000_003).chain([
            PRIMARY.rounding_slots as u64 - 1,
            PRIMARY_SLOTS - 1,
            PRIMARY_SLOTS + TAIL.rounding_slots as u64 - 1,
            TOTAL_SLOTS - 1,
        ]) {
            assert!(subsidy_kazmi(slot) >= subsidy_kazmi(slot + 1), "increase after slot {slot}");
        }
    }

    #[test]
    fn emission_candidate_range_accounting_survives_branch_replay() {
        // This models two branches consuming different numbers of slots
        // from one checkpoint. The ledger integration must supply that checkpoint.
        for boundary in [PRIMARY_SLOTS, TOTAL_SLOTS] {
            let parent = boundary - 2;
            let parent_total = cumulative_kazmi(parent);
            let old_branch_total = cumulative_kazmi(parent + 1);
            let replacement_end = parent + 5;
            let replacement = (parent..replacement_end).map(subsidy_kazmi).sum::<u64>();

            assert_eq!(old_branch_total - subsidy_kazmi(parent), parent_total);
            assert_eq!(parent_total + replacement, cumulative_kazmi(replacement_end));
            let remaining = (parent + 1..replacement_end).map(subsidy_kazmi).sum::<u64>();
            assert_eq!(remaining + old_branch_total, parent_total + replacement);
        }
    }

    #[test]
    fn emission_candidate_genesis_fixture_has_no_spendable_output() {
        assert!(GENESIS.build_genesis_transactions().iter().all(|tx| tx.outputs.is_empty()));
    }
}
