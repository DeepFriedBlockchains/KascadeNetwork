# Phase 6 authorized pruning UTXO append · November 7, 2026

This inactive candidate is based on the import commitment code at
`1137db02a4ad3110302dde71a20497aff5b81a9e`. The earlier guarded-import milestone passed all eight
jobs and 1,545 native tests in
[run 42402654584](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42402654584).

The processor test adapter stages pruning UTXOs through the existing typed
database store while holding the pruning session, point and metadata guards.
The worker permit authenticates the exact owner, database and closed import
session before persistence. The UTXO batch preserves the CLOSED marker
and only then contributes to the in-memory multiset.

The regression attempts the append with a different owner and checks that
neither the rows nor the accumulator change. The authenticated append must
then contribute exactly the committed UTXO and enable verified completion.
The adapter and test are inactive in live consensus. GitHub Actions
validation is pending.

Remaining work includes wiring token ownership into every IBD and direct
import mutation, binding shielded evidence and proof authentication to
completion, and validating interruption and staging cleanup end to end.
