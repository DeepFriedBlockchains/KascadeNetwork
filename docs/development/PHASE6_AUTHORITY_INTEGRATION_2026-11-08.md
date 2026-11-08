# Phase 6 authority integration

November 8, 2026

The integration branch descends from public `f436a1938def7370bf54d99d4f40606dff3f87ce`, with PR #3
at `f64839c` as the merge base. The latest source in that checkpoint,
`1e3cf48`, passed 1,557 workspace tests and all eight jobs in
[run 42493738676](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493738676).
The existing stack and its branches are preserved.

## Integrated regression coverage

- A real processor-owned pruning UTXO worker retains exclusive import
  ownership while detached. Staging drain waits for it and RocksDB can
  reopen only after its handle is released. A persisted UTXO row from the
  interrupted import fails verification. A stale lease cannot write even
  after fresh corroboration, while a fresh lease can.
- A worker whose authentic owner observes a changed or unreadable durable
  CLOSED revision is invalidated while retaining the reservation. Restoring
  the original marker cannot revive the token. Foreign owner checks happen
  before the marker read and cannot invalidate a valid session.

The processor integration fixture checks committed metadata and the UTXO
commitment. Independent shielded, Orchard, proof and SMT checks retain their
existing regressions. Production activation remains gated.

## Validation

Draft [PR #4](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/4) is stacked on PR #3 at `f64839c`.
The exact implementation head is `c44805170fb44dcd51e645e50341072c2a38e1e3` on
`feature/phase6-authority-integration-20261108`.
[PR run 42500808732](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500808732) is the authoritative combined run
and is pending at this handoff. The superseded
[push run 42500792929](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500792929) was cancelled by the PR run.
The focused and workspace test counts are pending. This handoff is kept
on a separate documentation branch so PR checks stay attached to the
implementation head.

The combined workflow runs the two focused authority regressions before
the full native workspace suite, followed by doctests and no-asm tests.
The other jobs cover strict Clippy, formatting, Linux release and the
platform checks. The previously verified candidates remain the green baseline.

## Next executable task

After combined CI is green, add an inactive staging import coordinator
that carries the processor-owned session through proof, SMT, shielded and
UTXO mutations and requires authenticated completion before promotion.
Cover cancellation and verification failure through the same boundary.
