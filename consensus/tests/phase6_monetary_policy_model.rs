//! Integer reference vectors for the proposed Phase 6 monetary schedule.
//!
//! This model uses abstract, unique reward slots. It is not called by consensus and does not map DAA scores to slots.

use kaspa_consensus_core::constants::{MAX_SOMPI, SOMPI_PER_KASPA};

const PRIMARY_BUDGET: u128 = 21_000_000_000 * SOMPI_PER_KASPA as u128;
const TAIL_BUDGET: u128 = 8_000_000_000 * SOMPI_PER_KASPA as u128;
const PRIMARY_STEP_SLOTS: u64 = 16_470_000;
const FINAL_EXTENSION: u64 = 750_000;
const PRIMARY_SLOTS: u64 = 50 * PRIMARY_STEP_SLOTS + FINAL_EXTENSION;

fn candidate_primary_paid(slots: u64) -> u128 {
    let standard_slots = 50 * PRIMARY_STEP_SLOTS;
    let regular = slots.min(standard_slots);
    let complete_steps = regular / PRIMARY_STEP_SLOTS;
    let partial_step = regular % PRIMARY_STEP_SLOTS;
    let whole_sum = complete_steps * (101 - complete_steps) / 2;
    let extension = slots.saturating_sub(standard_slots).min(FINAL_EXTENSION);

    u128::from(whole_sum * PRIMARY_STEP_SLOTS + partial_step * (50 - complete_steps) + extension)
        * u128::from(SOMPI_PER_KASPA)
}

fn proposed_tail_paid(slots: u64, duration: std::num::NonZeroU64) -> u128 {
    let length = duration.get();
    TAIL_BUDGET * u128::from(slots.min(length)) / u128::from(length)
}

fn proposed_total_paid(slots: u64, duration: std::num::NonZeroU64) -> u128 {
    candidate_primary_paid(slots) + proposed_tail_paid(slots.saturating_sub(PRIMARY_SLOTS), duration)
}

#[test]
fn approved_allocations_match_amount_limit() {
    assert_eq!(PRIMARY_BUDGET + TAIL_BUDGET, u128::from(MAX_SOMPI));
    assert_eq!(u128::from(MAX_SOMPI), 2_900_000_000_000_000_000);
}

#[test]
fn candidate_primary_cascade_matches_independent_step_totals() {
    let unit = u128::from(SOMPI_PER_KASPA);
    let mut expected = 0;
    assert_eq!(candidate_primary_paid(0), 0);
    for (index, subsidy) in (1..=50u128).rev().enumerate() {
        let start = index as u64 * PRIMARY_STEP_SLOTS;
        let end = start + PRIMARY_STEP_SLOTS;
        assert_eq!(candidate_primary_paid(start), expected);
        assert_eq!(candidate_primary_paid(start + 1) - expected, subsidy * unit);
        expected += subsidy * unit * u128::from(PRIMARY_STEP_SLOTS);
        assert_eq!(candidate_primary_paid(end), expected);
        assert_eq!(candidate_primary_paid(end) - candidate_primary_paid(end - 1), subsidy * unit);
    }

    assert_eq!(candidate_primary_paid(PRIMARY_SLOTS - 1), PRIMARY_BUDGET - unit);
    assert_eq!(candidate_primary_paid(PRIMARY_SLOTS), PRIMARY_BUDGET);
    assert_eq!(candidate_primary_paid(PRIMARY_SLOTS + 1), PRIMARY_BUDGET);
}

#[test]
fn finite_tail_transitions_and_single_sompi_rounding() {
    use std::num::NonZeroU64;

    for length in [3, 3_944_700_000, 7_889_400_000, 15_778_800_000, 16_000_000_000] {
        let duration = NonZeroU64::new(length).unwrap();
        let quotient = TAIL_BUDGET / u128::from(length);
        let upper_payment = TAIL_BUDGET.div_ceil(u128::from(length));
        let end = PRIMARY_SLOTS + length;

        assert_eq!(proposed_total_paid(PRIMARY_SLOTS), PRIMARY_BUDGET);
        assert_eq!(proposed_total_paid(PRIMARY_SLOTS + 1), PRIMARY_BUDGET + quotient);
        assert_eq!(proposed_total_paid(end - 1), PRIMARY_BUDGET + TAIL_BUDGET - upper_payment);
        assert_eq!(proposed_total_paid(end), u128::from(MAX_SOMPI));
        assert_eq!(proposed_total_paid(end + 1), u128::from(MAX_SOMPI));
        assert_eq!(proposed_total_paid(u64::MAX), u128::from(MAX_SOMPI));

        for previous in [0, 1, length / 2, length - 1] {
            let delta = proposed_tail_paid(previous + 1, duration) - proposed_tail_paid(previous, duration);
            assert!((quotient..=upper_payment).contains(&delta));
        }
    }
}

#[test]
fn wide_tail_products_fit_with_maximum_slot_count() {
    let duration = std::num::NonZeroU64::new(u64::MAX).unwrap();
    assert!(proposed_tail_paid(u64::MAX - 1, duration) <= TAIL_BUDGET);
    assert_eq!(proposed_tail_paid(u64::MAX, duration), TAIL_BUDGET);
}
