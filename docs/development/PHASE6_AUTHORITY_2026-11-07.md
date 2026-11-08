# Phase 6 database-bound authority · November 7, 2026

## Baseline

Public PRs #1, #2 and #3 remain stacked and open. PR #3 at `f64839c5c8ec8bff501cf5f69698723a0f7d9f37`
passed all eight jobs in [run 42348096245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245).
Native nextest ran 1,518 tests with zero failures and 25 skipped. The job
also passed workspace doctests and no-asm tests. Goref regeneration is complete.

## Inactive milestone

The authority wrapper is compiled only for tests. A lease requires the
committed marker and matching current and historical pruning metadata.
The exact database handle, owner, generation, marker revision and pruning
point are checked again before committing. The write and renewed READY
revision are persisted together in one synchronous WAL batch after a durable
CLOSED transition. Deferred cache invalidation runs after persistence.

Starting an import persists CLOSED before the first write. The same owner
and token authorize each batch, and the final operation in each batch
preserves CLOSED. Failed batches invalidate the session. Reopening requires
an explicit verifier and a matching committed pruning point and history
before and after verification. A legacy marker alone cannot authorize a lease.

The new regressions cover foreign batches and owners, stale lease and token
reuse, same-point revisions, missing or inconsistent pruning metadata,
abandoned and failed batches, and interrupted import recovery on reopen.
Existing abrupt-exit tests cover WAL recovery. New Rust results are pending
on this feature branch.

## Next implementation

Bind the wrapper to the processor's pruning session and metadata guards.
Verify committed UTXO position, proof and anticone state, and the applicable
SMT and shielded roots before enabling any readiness transition on a live
import path. Thread the same owner token through proof, pruning-point, UTXO,
SMT, shielded and cleanup callers. Detached blocking workers must hold their
operation permit through exit. Staging deletion must release old database
handles and avoid reacquiring an already-held pruning session guard.

Consensus activation and integration into main require explicit approval.
