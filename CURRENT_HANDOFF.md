# KASCADE public development handoff

Updated November 13, 2026.

## Validation stack

PR [#1](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/1) →
[#2](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/2) →
[#3](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/3) remains open.
The combined tip `f64839c` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245)
with 1,518 native tests and 25 skipped. Goref regeneration is complete.

## Phase 6 continuation

- The [authenticated import candidate](docs/development/PHASE6_AUTHENTICATED_IMPORT_2026-11-08.md)
  at `a6dd7fe` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489992639),
  1,552 native tests with 25 skipped, and 32 no-asm tests. The readiness
  reopen, failed-import reservation, abrupt WAL recovery, and shielded
  proof and nullifier regressions passed.
- The [staging drain candidate](docs/development/PHASE6_STAGING_DRAIN_2026-11-08.md)
  at `cf098af` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489994109),
  with 1,553 native tests, 25 skipped, and 32 no-asm tests. The staging
  cleanup and both abrupt WAL recovery regressions passed.
- The [lease write scope](docs/development/PHASE6_AUTHORITY_WRITE_SCOPE_2026-11-08.md)
  rechecks committed evidence on lease issuance and commit. Its source
  `c4a5aab` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493733661)
  and 1,556 native tests. The [pruning session retention change](docs/development/PHASE6_PRUNING_SESSION_RETENTION_2026-11-08.md)
  at `efd3c5c` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493736508)
  and 1,557 native tests.
- The [processor ownership candidate](docs/development/PHASE6_PROCESSOR_AUTHORITY_OWNER_2026-11-08.md) binds the inactive
  adapter to the owner created by the processor itself. Its exact
  [tested source](https://github.com/DeepFriedBlockchains/KascadeNetwork/commit/1e3cf487d66da46dfb35b04c027f1126e923070b)
  passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493738676),
  1,557 native tests with 25 skipped, and 32 no-asm tests.

## Authority integration

The public integration branch builds on `f436a1938def7370bf54d99d4f40606dff3f87ce`,
the documentation-only continuation of green source `1e3cf48`.
It preserves the PR #1 → #2 → #3 stack and brings the verified authority
candidates together with focused cross-component regressions. A processor
worker is detached, drained and reopened against the same RocksDB path.
An import token that observes a durable revision replacement stays revoked
until its worker releases the reservation.

See the [dated integration report](docs/development/PHASE6_AUTHORITY_INTEGRATION_2026-11-08.md).
Draft [PR #4](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/4) is stacked on PR #3 at `f64839c`.
The exact implementation head is `c44805170fb44dcd51e645e50341072c2a38e1e3` on
`feature/phase6-authority-integration-20261108`.
[PR run 42500808732](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500808732) is the authoritative combined run
and passed all eight jobs. Both focused regressions passed, along with
1,559 workspace tests (25 skipped), 32 no-asm tests, and strict Clippy.
The superseded
[push run 42500792929](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500792929) was cancelled by the PR run.
PR #4 retains its tested implementation head.

## Inactive staging coordinator

The [staging coordinator report](docs/development/PHASE6_STAGING_COORDINATOR_2026-11-08.md) describes the next stacked
candidate. One processor-owned session covers proof, SMT, shielded and
UTXO batches. A failed or interrupted component keeps readiness closed.
Authenticated completion issues the exact lease used by the inactive
promotion gate.

Restart on a database with descendants pins committed virtual and
selected-chain state, verifies the applied tip and pruning-point
shielded commitments, and checks the post-pruning nullifier projection.
Unavailable proof, a missing authenticated tip child or an inconsistent
committed view leaves authority closed.

## Candidate validation

[Draft PR #5](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/5) is stacked on PR #4.
The source checkpoint is `4eb422549fa822b804968f9c2911018b518286f0` on `feature/phase6-staging-coordinator-20261108`.
[PR run 42506751029](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506751029) passed all eight jobs.
All four focused regressions and 1,561 native workspace tests passed
(25 skipped). The no-asm suite passed 32 tests. Formatting and strict
Clippy passed.
The same-SHA [push run 42506746483](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506746483) was superseded.

[Run 42503818037](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503818037)
reported the missing `kaspa_hashes::Hash` test import in Check and
Clippy. Its correction `6e5c4ae` passed both jobs and five other platform
and build checks, with seven of eight jobs passing overall in
[run 42504801081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42504801081).
The focused suite passed three of four regressions, including committed
descendant recovery. The staging cancellation fixture expected an absent
anticone row, but consensus initialization creates it. The corrected test
requires both scoped rows to equal their pre-operation bytes after the
rejected batch. The staging and restart implementation remains unchanged.

## Typed UTXO adapter

[Draft PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6)
is stacked on PR #5. The original typed chunk source
`2afec236ac42e03bdeaf21d06c0191a70bba99f7` passed all eight jobs in
[run 42509613211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42509613211).
Four focused regressions, 1,561 native workspace tests (25 skipped), and
32 no-asm tests passed.

## Owner-bound completion and promotion receiver

The [November 10 report](docs/development/PHASE6_PROMOTION_RECEIVER_2026-11-10.md)
records the continuation on the PR #6 branch. The typed UTXO completion
worker verifies committed UTXO rows before publishing the position and
stable flag in the same authority-checked batch. The consensus manager has
a separate verified commit entry point. It asks the exact staging receiver
to consume the completion receipt before swapping consensus. At that checkpoint the receipt issuer and receiver were inactive test
code. The later preview wires them into production IBD behind a
default-off Simnet feature. The initial source `bc393eae2d9bd03df2f684d06e53ec78e865da60` reached
[run 42894808941](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42894808941).
Formatting reported two line wraps in the regression test. The correction
`72af6c818be80c8a0365036e23c4744da8b4bf94` passed formatting in
[run 42895086028](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42895086028).
Check and strict Clippy also passed. The full workspace result was
pending when the follow-up source was prepared.

## IBD proof receipt and promotion recheck

IBD now passes the exact proof accepted by the defender to the staging
receiver in a typed receipt. This uses the existing validation and
application logic. The inactive promotion commit now independently
rechecks the committed UTXO, SMT, proof, and shielded evidence before
spending the lease. A regression removes committed UTXOs after receipt
issuance and requires the write to fail with readiness closed. The follow-up passed all eight jobs in run 42896198192, recorded below.

## Owner-bound proof application

The candidate on `feature/phase6-owned-proof-import-20261110` builds on draft
[PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6)
at `42476e2661b5e579f665407d18352a36af09c301`. The baseline
[run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
had passed six jobs at candidate preparation. The Linux release build and
workspace Test Suite were still running. The candidate uses a separate
branch to preserve that run.

The [dated report](docs/development/PHASE6_OWNED_PROOF_IMPORT_2026-11-10.md) records the receiver-owned proof worker.
The coordinator binds the defender's exact receipt before its first
database write and keeps the proof evidence through completion. Both IBD
proof branches use the same proof and history application entry point.
The coordinator-specific entry point remains inactive. The initial proof candidate `863ec4fd7293fa3c54916b229c0f9c5f9a6ba659` passed formatting and Check in
[run 42897872282](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42897872282)
before the streaming follow-up.

The streaming follow-up also drains the SMT and shielded writer on a
stream error before returning to the staging caller. A focused test
covers error propagation only after worker release. The completed combined CI results are recorded below.

The combined source is `ecf0aacafa9cfc3028eb363021c5aebf733c62e1` on
`feature/phase6-owned-proof-import-20261110`. Its exact validation is
[run 42898217206](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42898217206).
The unchanged PR #6 baseline
[run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
passed all eight jobs at `42476e2661b5e579f665407d18352a36af09c301`.
It passed six focused regressions, 1,563 native workspace tests
(25 skipped), and 32 no-asm tests. Combined run 42898217206 passed all eight jobs.

## Production IBD preview

See [the dated development report](docs/development/PHASE6_PRODUCTION_IMPORT_PREVIEW_2026-11-10.md). The continuation adds a
default-off `kaspa-consensus/phase6-staging-preview` feature for an
isolated Simnet staging receiver. It admits the exact defender-accepted proof
before writes, carries the same reservation through production IBD
workers, and verifies durable virtual UTXOs before verified promotion.
[Draft PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is stacked on the green `feature/phase6-owned-proof-import-20261110`
branch at `ecf0aacafa9cfc3028eb363021c5aebf733c62e1`. The initial head was
`ebcb3ce00043b0d3f73a2ee7508f0cc68a363a69` on `feature/phase6-production-import-preview-20261110`.
[Run 42909699012](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42909699012)
passed seven jobs and failed the two-daemon shielded preview on exact
descriptor verification. The correction and green replacement run are
recorded below. The parent passed eight focused regressions, 1,565
workspace tests (25 skipped), and 32 no-asm tests in run 42898217206.

## Descriptor preservation correction

[PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is validated at `2ff50a53e727511a738c04fec6ddcd432c389790` on
`feature/phase6-production-import-preview-20261110`.
[Run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
passed all eight jobs, including formatting, strict Clippy, 11 focused
regressions, 1,567 workspace tests (25 skipped), and 32 no-asm tests.
Workspace doc tests, no-asm doc tests, cargo doc and kip10 example checks
also passed. The same-SHA push run 42930270803 was superseded.

The production sanity rebuild had replaced the authenticated external
descriptor with an optimized local descriptor. It now reconstructs
without descriptor or proof-cache writes. Exact descriptor verification
is unchanged. The original two-daemon shielded promotion regression
passed. See [the descriptor report](docs/development/PHASE6_STAGING_PROOF_DESCRIPTOR_2026-11-10.md).

## Peer interruption and active receiver role

[Draft PR #8](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/8)
is stacked on PR #7. Its exact source is
`bc886b6aaf9df3f2deceac75691f1a1bb932f9fa` on
`feature/phase6-preview-peer-interruption-20261110`.
[PR run 42935509499](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42935509499)
passed all eight jobs. Formatting, strict Clippy, all 15 focused
regressions, 1,568 workspace tests (25 skipped), and 32 no-asm tests passed.
Workspace doc tests, no-asm doc tests, cargo doc and the kip10 example
also passed. Same-SHA push run 42935435587 was superseded.
This documentation-only continuation changes no tested implementation.

The tests pause actual peer transfers, disconnect a temporary peer,
require reservation retention while paused and worker draining before
retry, then require verified promotion. The UTXO case reopens the same
data directory before retry. A separate persistent database regression
abandons a receipt before promotion, reopens incomplete SMT evidence and
requires fresh authentication.

Predecessor `b510f585f3e746e7541a58cc5e09b69c988189d4` passed seven jobs
and 14 of 15 focused regressions in
[run 42932087535](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42932087535).
The remaining test exposed a real bug. Existing active databases skip
genesis initialization when reopened, but that flag was also used to
identify staging receivers. Fresh staging allocation now explicitly
assigns a volatile role, activation revokes it and active reopen starts
without it. The original rejection assertion remains. Successful import
tests additionally require a durable verified promotion record and a
closed active staging role.

See [the interruption report](docs/development/PHASE6_PEER_STREAM_INTERRUPTION_2026-11-10.md).
Shielded interruption occurs before metadata because this fixture has
zero spent nullifiers. Its transparent UTXO stream is empty and pauses
before the done frame. Nonempty payload interruption and abrupt process
termination before promotion remain additional coverage.

## Nonempty payload and abrupt restart validation

[Draft PR #9](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/9)
is stacked on PR #8. Its tested implementation is
`110d1625763c9fecd2cf2a226e5e7fb25add99bf` on
`feature/phase6-payload-crash-recovery-20261111`.
[Run 43068789717](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43068789717)
passed all eight jobs. Formatting, strict Clippy, 21 focused regressions,
1,568 workspace tests (25 skipped), 32 no-asm tests, doc tests,
cargo doc and the kip10 example passed. No implementation CI remains
pending for this milestone.

The fixtures mine a real Orchard spend and nonempty transparent UTXOs.
Ten signed user lanes are refreshed inside the pruning-point finality
window. Cancellation pauses after a real native SMT batch, nonempty
UTXO append or verified atomic shielded seed. The UTXO done frame is
held back until disconnect. Real SIGKILL cases reopen the same sender
and receiver directories, reject incomplete staging, then require fresh
authentication, verified promotion and successful post-IBD blocks.
All six new regressions passed with their safety assertions retained.

See [the dated report](docs/development/PHASE6_PAYLOAD_CRASH_RECOVERY_2026-11-11.md).
This documentation-only continuation changes no tested implementation.

## Active receiver admission validation

[PR #10](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/10)
retains documentation head `a0885ad19eac58edec15283394e9ad78cacc98d2`.
Implementation `509e54b5af6a686b24086eea30fe97e8dc02af23` passed
all eight jobs in [run 43076086692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43076086692).
The completed job log confirms 21 focused tests, 1,568 workspace tests
(25 existing skips), and 32 no-asm tests. Formatting, strict Clippy,
documentation, example and platform/build checks passed.
Real authenticated receipts were rejected by actual admission calls
on ordinary active receivers before import and after promotion,
including the restart scenarios. Do not repeat this completed work.

## Tampered shielded stream validation

Draft [PR #11](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/11)
is validated at `9612aac2ebca91f384591c3c135fa9df57dbc6d3` on
`feature/phase6-tampered-shielded-stream-20261112`.
[Run 43309997530](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43309997530)
passed all eight jobs, independently rechecked November 13, 2026.
Logs confirm 23 focused tests, 1,568 workspace tests (25 skips), and
32 no-asm tests.
The original tampered-stream, same-directory recovery, active-admission
and verified-promotion assertions passed. Do not reimplement these.
Initial run 43308475970 failed five formatter hunks; the validated
correction changed formatting only.

## Bounded shielded ingestion — PR #12

Draft [PR #12](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/12)
is stacked directly on PR #11 at
`9612aac2ebca91f384591c3c135fa9df57dbc6d3`.
Branch: `feature/phase6-bounded-shielded-ingestion-20261113`.
Bounded ingestion implementation: `acba224032ab7613ac805bcb33160fc696436f2c`.
Current reopen test/CI correction: `f1808f1231697474de66bf3f99bad0c2f01ead0d`.
Later checkpoint commits contain documentation only.

Spent-nullifier ingestion no longer accumulates the full set or one
full-set membership batch. Frames are capped at 4,096 entries (128 KiB
raw payload). Each deferred synchronous RocksDB batch checks the exact
processor-owned session, database, revision and allowed shielded prefixes.
The existing proxy pruning guard and worker reservation remain pinned
through the blocking call; network waits hold no store locks.

Partial rows remain unready. Declared count, received accumulator and
streamed committed database evidence must all agree. Verified metadata
and readiness publish together in one atomic deferred batch, with cache
invalidation after persistence. Failed owned imports poison their session.
The raw adapter rejects isolated receivers lacking an authenticated permit.
Isolated completion/promotion checks also stream committed membership.

Eight new bounded_shielded_ regressions cover lazy large streams,
persistent close/reopen/retry, interrupted and corrupt payloads, duplicate
membership, changed committed evidence, rejected final publication,
stale authority, foreign ownership and deferred metadata/shared anchors.
Existing production shielded disconnect and SIGKILL controls now stop
after a real membership batch, before metadata publication. The original
23 authority integration cases remain selected, including tampered retry,
restart, activation rejection and verified successful promotion.

## PR #12 reopen correction and current validation

Run [43511658558](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43511658558)
finished with seven of eight jobs passing. Native job 132183262907 failed
bounded_shielded_partial_stream_reopens_unready_and_retries_fresh with
HashAlreadyExists while trying to insert committed genesis GHOSTDAG data.
Two focused cases passed, one failed, 28 were not run; full workspace
runtime validation did not execute. PR #12 is not validated.

This is a fixture initialization error: TestConsensus::with_db re-used
process_genesis=true when reopening a populated database. Production
Factory::new_active_consensus disables genesis processing for an Existing
entry. The correction uses skip_adding_genesis only on the fixture's
second open, matching that production contract. All prior assertions
remain, with an additional byte-for-byte persisted unready-marker check.
No production header or readiness check has been changed.

The native CI job now runs that focused regression first, before its
workspace build and broader 31-case selection. Require all eight jobs.
See [the correction report](docs/development/PHASE6_BOUNDED_SHIELDED_REOPEN_FIX_2026-11-13.md) for evidence and details.
Replacement source SHA and Actions run IDs will be recorded after push.

## Exact next executable checkpoint

Verify the corrected focused persistent reopen regression, then the
31 authority/bounded cases and full eight-job run. Fix genuine failures
without weakening readiness, authentication, worker draining, restart or
promotion assertions. Do not repeat completed earlier milestones.

Only after PR #12 independently passes, create a new stacked draft PR
for bounded failed-import cleanup: replace the legacy full-set deletion
batch with bounded durable deletions under the same owner permit. Keep
readiness false through interruption/restart and preserve the
no-validated-descendants precondition. Descendant recovery/export
snapshots remain subsequent independently validated milestones.

PUBLIC repository and GitHub Actions only. PRs #1–#12 remain intact,
draft and unmerged. Preview stays default-off and Simnet-only. No merge,
release, force-push, private-history import or consensus activation.

## Pushed correction checkpoint

Correction source: `f1808f1231697474de66bf3f99bad0c2f01ead0d`.
Remote PR #12 head was verified after push. Authoritative replacement
PR run: [43515652940](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43515652940).
Duplicate push run 43515643677 is being cancelled by workflow concurrency;
the PR run is pending while that cancellation finishes. No replacement
focused or full validation result is claimed. Native job order is the
single persistent-reopen regression, then workspace build, 31 focused
cases and full workspace validation. Resume this run rather than rerunning
the prior known-bad source. PR #12 stays draft and unvalidated.

## Isolated validation branch for stalled Actions cancellation

The push run 43515643677 has seven cancelled jobs but Lints job
132196802751 still reports in_progress with no steps. PR run 43515652940
remains pending with no jobs. The GitHub connector has no cancellation
action; no local or browser access is used as a workaround.

Branch `validation/phase6-bounded-reopen-20261113` carries the exact
same corrected Rust source and eight-job workflow as f1808f1, plus this
documentation checkpoint. Its push uses a different existing concurrency
group, avoiding the stuck group without modifying the workflow or its
checks. PR #12 is fast-forwarded to the same commit so that the exact
latest head is tested. No history rewrite or additional PR is involved.
The new validation push run is authoritative once its exact SHA and all
eight results are verified. Source and workflow equivalence will be
checked through GitHub's commit comparison. No pass is claimed yet.
