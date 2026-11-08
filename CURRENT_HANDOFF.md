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
and is pending at this handoff. The superseded
[push run 42500792929](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500792929) was cancelled by the PR run.
The focused and workspace test counts are pending. This handoff is kept
on a separate documentation branch so PR checks stay attached to the
implementation head.

## Next executable task

After combined CI is green, add an inactive staging import coordinator
that carries the processor-owned session through proof, SMT, shielded and
UTXO mutations and requires authenticated completion before promotion.
Cover cancellation and verification failure through the same boundary.

## Remaining

Production IBD and direct import callers still need to thread the same
owner from the first mutation through authenticated state completion,
then drain every staging handle before deletion or promotion. Verified
restart on a database with descendants needs a stable pruning-point
nullifier view and authenticated proof evidence. Lease writes currently
use a dedicated inactive namespace. Main integration and activation
remain gated.
