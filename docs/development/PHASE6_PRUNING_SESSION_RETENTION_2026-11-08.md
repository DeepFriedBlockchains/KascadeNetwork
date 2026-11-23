# Phase 6 pruning session retention

November 8, 2026

This milestone follows the committed evidence and write-scope candidate at
`67090eccbc48a8a4d5f01d2a4434b615d845f242`.

The consensus proxy moved only its consensus handle into the blocking
callback under Rust 2024 capture rules. If the async wait was cancelled,
the pruning guard could be dropped while the blocking callback still
used the database. The closure now retains the complete owned session
through callback return or unwind.

The regression starts a guarded blocking callback, aborts its awaiting
task, and verifies a writer on the exact pruning lock stays pending.
It then releases the callback and verifies the same writer acquires the
lock. Production block validation, payout and monetary rules remain
inactive with respect to the Phase 6 authority changes.

Live import ownership through all state
writes and staging promotion remains the next implementation boundary.

## Verified validation

[Run 42493736508](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493736508) passed all eight
jobs at `efd3c5c4fb4417f224d8d2a0ed5761b3f749b7ed`. Native nextest passed 1,557 workspace tests
with 25 skipped. The separate no-asm suite passed 32 tests.
The detached blocking callback regression confirmed the pruning writer stayed blocked until callback exit.
