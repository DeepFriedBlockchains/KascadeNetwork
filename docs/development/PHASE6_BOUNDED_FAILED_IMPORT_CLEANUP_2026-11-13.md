# Phase 6 bounded failed-import cleanup

November 13, 2026

## Baseline and scope

Repository: DeepFriedBlockchains/KascadeNetwork.
This candidate stacks on PR #12 at documentation checkpoint
3734e6fa30d2d5e4c82b30034f42dbedcf30bbd6. Its unchanged source passed
all eight jobs in run 43516463729 at fa229e6524a78bb3bcd032d08e861f62a926637e:
one dedicated reopen regression, 31 focused cases, 1,576 workspace tests
(25 existing skips) and 32 no-asm tests.
It preserves the existing PR stack and leaves consensus inactive: the
staging preview remains default-off and Simnet-only. No merge or release.

## Implementation

Before deletion starts, a synchronous WAL-backed deferred batch persists
false shielded readiness. Membership is then read with one RocksDB
iterator and deleted in batches of at most 4,096 nullifiers. No full-set
Vec, HashSet or deletion batch is built. After verifying that committed
membership is empty, a fixed four-key batch removes the pruning-point
frontier, supply, nullifier accumulator and burn snapshots. Readiness
remains false until a complete fresh import succeeds.

Typed deferred deletion invalidates caches only after successful durable
persistence. Rejected marker or deletion batches publish no cache changes.
Partial cleanup can restart from the remaining committed rows after a
physical database reopen.

The production staging proxy routes cleanup through the same authenticated
session as ingestion. A noncloneable worker operation pins the processor
owner, database and revision until actual worker exit. Every commit checks
the shielded prefix allowlist and exact permit under point, metadata,
then authority ordering. Wrong owner, wrong point, out-of-order component,
busy worker and stale revision are rejected. An error poisons the staging
attempt; a fresh authenticated session is required after workers drain.
The proxy retains its existing pruning guard without reacquiring it.

Cleanup errors propagate back to IBD instead of proceeding to re-seed.
The existing no-validated-descendants refusal remains in place before
cleanup. The matching-root shortcut also requires stable readiness:
an interrupted deletion can leave old snapshots alongside partial
membership, so a matching old root alone cannot reopen that state.

Per-block diffs and historical anchor indexes remain intact. Descendant
recovery projections, export snapshots and metadata/burn serialization
remain separate later milestones.

## Regression coverage and validation

The existing F-15 clean-reseed regression retains every assertion while
calling the bounded implementation. Three additional regressions cover:

- Interruption after exactly one full deletion batch, physical close and
  reopen, unchanged durable unready marker, dirty-import refusal, resumed
  cleanup and a complete fresh import.
- Rejected initial marker and final snapshot deletion, with unchanged warm
  caches on rejected batches and durable readiness left false after
  deletion has started.
- Foreign, wrong-point, out-of-order, busy and stale permits, exact
  surviving membership, reservation retention after coordinator drop,
  worker release and fresh-session cleanup.

The native job runs the cleanup regressions before broader workspace
validation. The original persistent-reopen regression, all existing
authority/bounded tests and all eight CI jobs remain required.
Candidate source, PR and run IDs will be recorded after push. No cleanup
validation pass is claimed until the corresponding complete run succeeds.

## Initial cleanup validation checkpoint

Initial source `dafe86c7cde41e613a7666b9e2068c58fdda902c` is in draft
[PR #13](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/13),
stacked directly on PR #12.
[Run 43522414095](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43522414095)
reported four Rustfmt layout hunks. Their exact output is applied here,
without changing assertions or behavior. Replacement focused and complete
eight-job validation are required; no cleanup pass is claimed.
The PR #12 validation recorded above remains complete and unchanged.
