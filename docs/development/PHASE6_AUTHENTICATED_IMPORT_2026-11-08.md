# Phase 6 authenticated import evidence

November 8, 2026

This branch continues the restart candidate at `433c10f7d65cac9d4ee6ec861c4e777fb6554107`.
The PR #1 → #2 → #3 stack remains open. The combined stack tip passed all eight
jobs in [run 42348096245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245).

## Inactive candidate

- The existing pruning proof validator authenticates the challenger against
  the defending consensus before the receiver's guards are acquired. The
  receipt records the exact point, level descriptors and header hashes.
  Guarded completion corroborates those values from committed receiver rows.
- The receiver requires a committed selected-chain child whose GHOSTDAG
  selected parent is the pruning point. It hashes the complete body and
  checks the coinbase commitment against the committed shielded state and
  actual nullifier membership. The check assumes an isolated receiver with
  header and virtual processing quiesced and no applied descendants.
- Guarded pruning UTXO appends reject duplicate outpoints within a chunk or
  against earlier committed chunks before writing or changing the accumulator.

Focused regressions cover missing committed proof headers and metadata,
a mismatching descriptor, a substituted child body, missing or stray
nullifier rows, and duplicate UTXO chunks. Hosted CI is pending.

## Remaining

Live IBD and direct callers still need one owner from the first proof or
pruning write through SMT, shielded and UTXO completion. Cleanup needs to
drain detached workers and database owners before deleting staging. Verified
restart for a receiver that has applied descendants needs a stable
pruning-point view of the nullifier set and a freshly authenticated proof
or a durable proof receipt. All adapters in this candidate are test-only.
