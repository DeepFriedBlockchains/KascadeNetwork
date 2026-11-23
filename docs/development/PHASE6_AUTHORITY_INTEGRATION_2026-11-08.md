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

Combined Actions will run the focused authority integration tests before
the full workspace suite, strict Clippy, formatting, and the platform
checks. Exact run IDs and counts will be recorded after the branch is
pushed. The previously verified candidates remain the green baseline.

## Next executable task

After combined CI is green, add an inactive staging import coordinator
that carries the processor-owned session through proof, SMT, shielded and
UTXO mutations and requires authenticated completion before promotion.
Cover cancellation and verification failure through the same boundary.
