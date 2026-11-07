# Phase 6 guarded import verification · November 7, 2026

This branch continues the inactive database authority at `a7618bf5fbfb0bee41fde7c9d0bd0b497285819b`.
The public PR #1 → #2 → #3 stack remains on its green baseline.

- Worker permits keep the token alive through queued or detached blocking
  work. The coordinator cannot start a second worker or recover the token
  for completion while a permit exists.
- The guarded processor adapters take the pruning session, point and
  metadata guards before the authority mutex and retain them through
  readiness publication or the authorized database commit.
- Completion checks committed point and retention history, UTXO position,
  explicit stability flags, missing anticone bodies, and proof descriptor
  consistency. A fresh read of the pruning UTXO rows must reproduce the
  committed pruning header's MuHash. Legacy cache defaults do not qualify.
- Focused regressions cover missing, staged and inconsistent records,
  commitment mismatch, foreign owners and batches, overlapping workers,
  cancellation, and guard retention.

The adapters are test-only. The verifier still needs production-grade
pruning proof authentication, applicable SMT and shielded commitments,
and an authorized verified restart path. Protocol and direct import
callers must use the same token from the first write, avoid nested
pruning-session reads, and drain database handles before staging cleanup.

## Validation

[Run 42402654584](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42402654584)
passed all eight jobs on `23fe1993616231da6099d6e8bd6fdec88bb8aa35`. Nextest ran 1,545 tests with
zero failures and 25 skipped, including all 27 added database and import
regressions. The no-asm suite passed 32 tests. Workspace doctests,
formatting, Clippy, Linux release, and WASM also passed.

The preceding database-authority run passed its 20 focused regressions.
Its first workspace attempt recorded 1,537 passes and one gRPC port
collision in the daemon compute-budget test. The daemon test passed in
this combined run. A retry of the earlier exact commit is underway.
