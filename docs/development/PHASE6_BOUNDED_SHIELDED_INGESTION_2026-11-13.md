# Phase 6 bounded shielded ingestion

November 13, 2026

## Baseline

PUBLIC repository DeepFriedBlockchains/KascadeNetwork.
Draft PR #11 at 9612aac2ebca91f384591c3c135fa9df57dbc6d3 passed all
eight jobs in run 43309997530, independently rechecked this session.
Logs confirm 23 focused tests, 1,568 workspace tests (25 skips) and
32 no-asm tests.
PRs #1–#11 are preserved. All work uses connected GitHub tools and
GitHub Actions exclusively.

## Implementation

Branch: `feature/phase6-bounded-shielded-ingestion-20261113`, stacked on PR #11.
The consensus adapter incrementally accounts for each streamed frame.
A hard 4,096-nullifier frame limit bounds raw payload to 128 KiB; a
single-frame duplicate set and deferred RocksDB batch have the same
fixed upper entry bound. The protocol already limits frames and has a
two-frame channel. No full-set Vec or full-set membership write batch
remains in ingestion.

The preview proxy now calls a dedicated owner-bound shielded adapter.
Its noncloneable operation pins the exact processor session. Every
batch checks the session, database, revision and allowed shielded
prefixes before synchronous persistence. It retains the proxy's existing
pruning session guard and never acquires a duplicate receiver guard.
Network waits hold no point/metadata/authority mutex. Store writes
follow point then metadata then authority ordering.

An initial durable false marker precedes payload writes. Dirty membership
requires a cleared import; duplicates within or across frames are
rejected. Received count and accumulator must match authenticated
metadata. A streaming committed-row recheck validates the actual
database, including count, before final publication. Metadata and stable
readiness share one atomic deferred batch. Deferred typed stores avoid
cache publication on rejected writes. Historical shared-anchor producers
are grouped explicitly so deferred writes cannot overwrite each other.

Cancellation, stream errors and premature EOF retain partial rows with
readiness false. The owned adapter marks its session failed before work
and restores progress only on success. Existing protocol join-before-
cleanup behavior remains. Raw ingestion cannot bypass an isolated
receiver's authenticated adapter. Promotion still rechecks authenticated
proof and durable component evidence before consuming the exact lease.

## Regression coverage

- Large lazy nonempty stream: the next frame is unavailable until the
  preceding one is committed and readiness is false; four frames exceed
  the per-frame bound without a full-stream buffer.
- Partial stream: persistent database close/reopen, unready marker,
  refusal to reuse dirty rows, clear and complete retry.
- Empty/oversize frames, declared count mismatch and corrupted accumulator.
- Duplicate membership across frames.
- Committed-row corruption after receipt but before publication.
- Rejected final batch: no metadata/cache publication and false readiness.
- Stale revision between real batches and foreign processor session rejection.
- Abandoned metadata batch and correct shared historical anchor producers.
- Existing production disconnect and SIGKILL hooks now stop a real shielded
  membership worker before final metadata; existing tamper, restart,
  active-admission and successful promotion cases run unchanged.

The focused Actions selection includes both bounded_shielded_ and
authority_integration_. Full eight-job validation is mandatory; no pass
is claimed at candidate creation. Exact SHA, PR, run IDs and results
will be appended after push.

## Remaining work

Descendant restart projection and export still materialize full membership
snapshots. Their exact descendant-difference checks remain intact.
Metadata/burn receipt serialization is unchanged; this candidate bounds
nullifier ingestion and isolated promotion verification. Review remaining
wire metadata and export/recovery limits as separate validated milestones
after this candidate passes.

The preview remains default-off and Simnet-only. No consensus, emission,
monetary, merge-mining or KRN activation; no merge or release.
