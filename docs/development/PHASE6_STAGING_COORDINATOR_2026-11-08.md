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
The current source checkpoint is `2854f56fea4bf8a67a4e225b445499282e7f8bc2` on
`feature/phase6-staging-coordinator-20261108`.
[Combined PR run 42503818037](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503818037) is in progress and covers
the focused authority regressions, full workspace suite and strict
Clippy. The same-SHA [push run 42503813493](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503813493) was
superseded by the PR run. Candidate test counts are pending.

The original [PR run 42503679589](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503679589)
on `9e332bb5e22f9b6e6c429b45a25350df60379c72` reported rustfmt differences. The current source
applies those changes. The handoff is on a separate branch so PR checks
remain attached to the source commit.

Verify all eight jobs before integrating the production adapters.
