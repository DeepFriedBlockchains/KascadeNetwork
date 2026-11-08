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
  at `cf098af` adds cleanup for detached import workers. See
  [the exact Actions run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489994109).
- The [lease write scope](docs/development/PHASE6_AUTHORITY_WRITE_SCOPE_2026-11-08.md)
  rechecks committed evidence on lease issuance and commit. The
  [pruning session retention change](docs/development/PHASE6_PRUNING_SESSION_RETENTION_2026-11-08.md)
  keeps a detached blocking callback's pruning guard through callback exit.
- The [processor ownership candidate](docs/development/PHASE6_PROCESSOR_AUTHORITY_OWNER_2026-11-08.md) binds the inactive
  adapter to the owner created by the processor itself. Hosted validation
  is pending on the newest candidate.

## Remaining

Production IBD and direct import callers still need to thread the same
owner from the first mutation through authenticated state completion,
then drain every staging handle before deletion or promotion. Verified
restart on a database with descendants needs a stable pruning-point
nullifier view and authenticated proof evidence. Lease writes currently
use a dedicated inactive namespace. Main integration and activation
remain gated.
