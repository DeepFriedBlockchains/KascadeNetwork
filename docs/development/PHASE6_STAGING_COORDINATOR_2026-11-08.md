# Phase 6 inactive staging coordinator and restart

November 8, 2026

PR #4 at `c44805170fb44dcd51e645e50341072c2a38e1e3` passed all eight jobs in
[run 42500808732](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500808732),
including both focused regressions, 1,559 workspace tests with 25 skipped,
and 32 no-asm tests.

## Coordinator

- An ordered coordinator owns one processor reservation through proof,
  SMT, shielded and UTXO writes. Blocking workers retain the permit and
  pruning session during batch construction and commit. Physical write
  scopes reject cross-component mutations, including readiness and
  promotion keys. Successful component batches preserve CLOSED readiness.
- Only authenticated durable completion can issue the promotion lease,
  and the processor rechecks it before publishing the inactive promotion
  record. An intervening session or commit invalidates a stale lease.
- Regressions exercise out-of-scope partial writes, detachment and drain,
  competing ownership, incomplete stages and stale promotion leases.
  The inactive adapter accepts internally constructed batches, and the
  successful test uses a controlled proof and pre-Toccata fixture.

## Restart with descendants

Restart reads committed virtual state independently of caches, checks the
selected-chain index and selected-parent links, authenticates the shielded
state at the pruning point and applied tip, and subtracts descendant
nullifiers from the verified tip set. Missing or repeated members fail
closed. Nonblocking virtual and chain lock acquisition avoids reversing
the live lock order. The fixture verifies reopen, corrupt descendant
diff rejection, and recovery from a restored committed view.

A restart requires a validated pruning proof and a child with an
authenticated commitment to the applied tip. At the header tip without
such a child, readiness stays closed. The production IBD/direct write
adapters and staging promotion switch still require integration.

## Validation and next checkpoint

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

Verify all eight jobs before integrating the production adapters.
