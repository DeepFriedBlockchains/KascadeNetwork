# Phase 6 typed pruning UTXO adapter

November 8, 2026

The branch starts at green PR #5 source `4eb422549fa822b804968f9c2911018b518286f0`.

[PR run 42506751029](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506751029) passed all eight jobs.
All four focused regressions and 1,561 native workspace tests passed
(25 skipped). The no-asm suite passed 32 tests. Formatting and strict
Clippy passed.

The first adapter slice adds an inactive method on the consensus owner
for typed pruning UTXO chunks. It rejects a different processor before
submitting work, then uses the existing coordinator reservation and
processor-owned deferred write. The generic UTXO batch scope only
allows position and stability metadata, so UTXO rows use the typed path.
The promotion and descendant-restart fixture now requires a real UTXO
row with an authenticated commitment and rejects a foreign processor.

This candidate still compiles only in tests. Production IBD forwarding,
the remaining typed import stages, and the production promotion switch
remain open. [Draft PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6) is stacked on PR #5.
The source checkpoint is `2afec236ac42e03bdeaf21d06c0191a70bba99f7` on `feature/phase6-typed-pruning-utxo-adapter-20261108`.
[PR run 42509613211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42509613211) is pending and covers the focused tests,
workspace suite and strict Clippy. The same-SHA [push run 42509600116](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42509600116)
was superseded. Candidate test counts are pending.
