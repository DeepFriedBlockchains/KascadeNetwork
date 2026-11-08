# KASCADE public development handoff

Updated November 8, 2026.

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
The source checkpoint is `6e5c4ae4479920cd5dee63acdac9ec816d1e1fb2` on `feature/phase6-staging-coordinator-20261108`.
[PR run 42504801081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42504801081) validates the corrected source.
The same-SHA [push run 42504797523](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42504797523) was superseded.
Focused and full workspace results are pending.

[Run 42503818037](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503818037)
reported two E0573 errors in the staging proof tests. Both `Check`
and `Lints` found the missing consensus hash type. The current source
imports `kaspa_hashes::Hash` in the test module. The implementation
of staging and descendant restart is unchanged.

## Next executable task

Verify the focused and full CI for this exact source before binding
production IBD and direct import adapters to the coordinator. The live
promotion boundary, transport-to-typed write adapters, and integration
with the production staging manager remain open. Activation and main
integration remain gated.
