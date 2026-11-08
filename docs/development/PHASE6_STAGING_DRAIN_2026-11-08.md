# Phase 6 detached worker drain

November 8, 2026

The inactive worker coordinator now owns the blocking permit across the
entire closure. The cancellation path can wait for an abandoned worker to
release its permit before deleting the staging database. The regression
drops the join handle, verifies cleanup remains pending while the worker
still holds the database, then reopens RocksDB and confirms its marker
remains CLOSED. The worker may complete an authorized write while cancellation
drains, with readiness closed throughout.

Live staging cancellation and direct/IBD import callers still need to adopt
the coordinator and hold the owner from the first mutation through final
verification. Integrating that path requires preserving the caller's existing
pruning session, moving the staging promotion boundary behind authenticated
state completion, and draining all other staging handles before deletion.

## Integration review

The existing stack is #1 at `ba4ce307`, #2 at `91da4d8` and #3 at
`f64839c`, over protected main at `62248169`. All three are open and #1
is draft. The combined stack passed all eight jobs and 1,518 native tests in
[run 42348096245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245).
The older standalone runs for #1 and #2 failed. Once approved, use merge
commits to fold #3 into #2, then #2 into #1, keeping the cumulative fixes
together. Validate the updated cumulative heads and review the final main
PR before merging. This branch contains independent Phase 6 candidates.

## Verified validation

[Run 42489994109](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489994109) passed all
eight jobs at `cf098af5e1081dd5013dae0446321f06775ef8ef`. Native nextest passed 1,553 workspace
tests with 25 skipped, and the separate no-asm suite passed 32 tests.
The detached staging worker and both abrupt WAL recovery regressions passed.
