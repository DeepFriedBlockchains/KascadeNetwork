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

## Initial Actions checkpoint

Draft PR #12: https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/12.
Initial implementation `d13ba931018f08ca71ddb78c242a42432d4d3431` was
pushed and its remote head verified. PR run 43509749012 superseded push
43509739189. Lints reported 22 Rustfmt layout hunks; the exact formatter
output has been applied without altering checks or assertions. Full
validation is pending for the replacement source. No pass is claimed.

## Replacement validation checkpoint

Current implementation: `454ec2fb37bed61562f1e2a598d522d299558c78`.
Remote PR #12 head was verified after push. Authoritative PR run:
[43510035150](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43510035150).
Push run 43510024178 was superseded. Formatting passed; strict Clippy,
Check, native focused tests and full validation are still running.
This documentation-only continuation preserves the exact source above.
Resume that run before changing code or repeating completed work.

## Compiler correction

Run 43510035150 passed formatting, no-std, WASM checking and WASM tests,
but Check reported E0308 in the new stale-authority test: its last
begin_fixture call passed a borrowed Arc. The correction clones the
processor handle; it changes no safety assertion. Row-count assertions
also explicitly propagate iterator errors without collecting a full set.
The replacement full run must pass before this candidate is validated.
Follow-up memory work also includes bounded deletion during failed-import
cleanup; the legacy clear still constructs one full-set deletion batch.

## Current executable checkpoint

Source `acba224032ab7613ac805bcb33160fc696436f2c` is pushed on
`feature/phase6-bounded-shielded-ingestion-20261113`, draft PR #12 above PR #11.
The borrowed-Arc test correction is in ffe3ba69cfbbc30b2d39d2b70b478ece19f18a42;
acba224 matches the one CI formatter wrap exactly.

Authoritative full run: [43511658558](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43511658558).
Push 43511647206 was superseded. Run 43511360104 stopped at the single
formatter wrap before Clippy; it is superseded. No new native tests have
yet completed. Require the eight new bounded_shielded_ regressions and
all 23 existing authority_integration_ cases, then the full workspace,
strict Clippy and all eight jobs. Do not treat earlier WASM/no-std passes
as a native or combined validation pass. This documentation commit does
not alter the validating source.

After that run is green, the next implementation task is bounded
failed-import cleanup: replace the legacy full membership deletion batch
with bounded durable deletions under the same owner permit and a false
readiness marker, preserving no-validated-descendants preconditions.
Then address descendant recovery/export snapshots independently. No
other Phase 6 milestone has been declared complete by this checkpoint.

## Verified compilation and handoff

Source `acba224032ab7613ac805bcb33160fc696436f2c` in draft PR #12
has passed Check (including tests/benches and preview), formatting,
strict Clippy, WASM checking/tests and no-std checking in run 43511658558.
Those checks occupy five passing jobs. The native Test Suite, Linux
release and WASM SDK build remain in progress. No native test counts
or full eight-job pass are claimed. The native job is still compiling
before the focused selection; expect eight new bounded tests plus the
23 existing authority regressions, but record actual completed counts.

Current handoff documents the exact next step: finish that run, fix any
actual runtime failure, then independently implement bounded failed-import
cleanup under the authenticated permit. Full-set descendant recovery/
export snapshots and metadata/burn serialization remain follow-ups.
This documentation-only continuation preserves the validating source.
