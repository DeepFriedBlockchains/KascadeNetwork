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

The exact green source can be used for the production adapter checkpoint.
