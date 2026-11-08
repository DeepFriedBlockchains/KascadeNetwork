# KASCADE public development handoff

Updated November 8, 2026.

The [validation stack](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/3) remains PR #1 → #2 → #3.
The stack tip `f64839c5c8ec8bff501cf5f69698723a0f7d9f37` passed all eight jobs in
[run 42348096245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245),
including 1,518 native tests with zero failures and 25 skipped.
Goref regeneration and validation are complete.

The Phase 6 feature branch implements the inactive database-bound authority
wrapper and synchronous deferred batch. The wrapper authenticates import
writes and closes readiness durably across failure or interruption. It
requires committed pruning metadata and exact owner and database identity.
See the [dated report](docs/development/PHASE6_AUTHORITY_2026-11-07.md) for scope and test coverage.

Next: validate this branch, then bind the processor's pruning guards and
committed import verification to the wrapper. Live import callers, detached
worker permits and staging cleanup remain open. Consensus activation stays gated.

The guarded import continuation adds worker permits, committed metadata
corroboration, and UTXO commitment verification under processor pruning
guards. See [the continuation report](docs/development/PHASE6_IMPORT_VERIFICATION_2026-11-07.md). Full proof, SMT and
shielded verification, and live import caller wiring remain pending.

The import commitment branch adds guarded SMT corroboration and an isolated
shielded-state regression. See [the commitment report](docs/development/PHASE6_IMPORT_COMMITMENTS_2026-11-07.md).
CI and live integration are pending.
The commitment branch also verifies that abandoned capabilities do not
keep a staging database open. Worker lifetime and database handle lifetime
are separate: the operation must finish before its active owner is dropped.

The guarded import milestone passed all eight jobs and 1,545 native tests
in [run 42402654584](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42402654584).
The next inactive adapter stages pruning UTXOs through the worker permit
and updates the accumulator only after commit. See [the append report](docs/development/PHASE6_UTXO_IMPORT_2026-11-07.md).
Live wiring, proof authentication and full branch validation remain pending.

The [restart and session report](docs/development/PHASE6_AUTHORITY_RESTART_2026-11-08.md)
records the current candidate. Lease issuance requires a revision verified on
this database open. Failed import writes invalidate the token while retaining
the session until the worker releases it. The shielded import fixture now
checks the actual Orchard action nullifiers. Hosted validation is pending.

The next [proof and shielded evidence candidate](docs/development/PHASE6_AUTHENTICATED_IMPORT_2026-11-08.md)
uses the existing pruning proof validator and rechecks committed evidence in
the isolated receiver. Guarded UTXO appends also reject duplicates before
writing. The candidate requires hosted validation. Live IBD and direct import
migration still need end-to-end staging ownership and cleanup.

The [staging drain report](docs/development/PHASE6_STAGING_DRAIN_2026-11-08.md) adds
an inactive cleanup seam that waits for a detached import worker to release
its permit and captured database owner. It also records the integration
sequence for the existing PR stack. Hosted validation is pending.

The authenticated import source `a6dd7fe` passed all eight jobs in
[run 42489992639](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489992639).
Native nextest passed 1,552 tests with 25 skipped, and the no-asm suite passed
32 tests. This includes both abrupt-reopen regressions and the Orchard and
isolated shielded verification cases.

The [lease write scope report](docs/development/PHASE6_AUTHORITY_WRITE_SCOPE_2026-11-08.md) records the next inactive milestone.
Lease issuance and commit re-read the exact committed import evidence.
The scoped RocksDB batch keeps a lease from modifying the rows that
authorize it. Hosted validation of this milestone is pending.

The [pruning session retention report](docs/development/PHASE6_PRUNING_SESSION_RETENTION_2026-11-08.md) records a worker
ownership fix used by the consensus session API. Cancelling an async wait
keeps the exact pruning guard in the detached callback until it exits.
The focused cancellation regression and full workspace checks are pending.
