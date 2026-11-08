# Phase 6 committed evidence and lease write scope

November 8, 2026

This branch continues from staging drain source `cf098af5e1081dd5013dae0446321f06775ef8ef`.
Authenticated import at `a6dd7fe` passed all eight jobs, 1,552 workspace tests
with 25 skipped, and 32 no-asm tests in
[run 42489992639](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489992639).

- Retain the exact committed pruning proof descriptor, point and retention
  evidence verified on this database open. Both lease issuance and commit
  recheck the revision and fresh committed evidence under the processor's
  pruning guards. Observed drift revokes the generation and persists CLOSED.
- Check physical RocksDB point operations before the lease can stage its
  replacement READY marker. Only the inactive branch-data prefix is allowed.
  Unknown batch operations and writes outside that prefix are rejected before
  payload persistence. Authorized data and the new marker remain one sync
  write.
- Regressions check that a batch cannot publish data while changing its own
  readiness evidence, and that restoring changed proof metadata cannot revive
  a stale lease. The same regression moves the pruning point and confirms
  that restoring the old point leaves its old lease revoked.

Hosted validation is pending. The isolated authority and import adapters
remain test-only. Real branch-domain writes and live import caller migration
need a reviewed integration with a verified restart path for populated databases.
