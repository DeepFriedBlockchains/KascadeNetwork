# Phase 6 peer stream interruption candidate

November 10, 2026

This test candidate builds on descriptor correction `2ff50a53e727511a738c04fec6ddcd432c389790`.
The correction's [run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
is still running. The separate branch preserves PR #7 validation.

The integration test can pause a real peer transfer before its next
protocol frame, then ban the temporary peer through RPC. SMT pauses
after metadata with a blocking receiver writer started. Shielded pauses
before metadata, while the receiver owns the reservation. UTXO pauses
before the explicit done frame. Each case requires the reservation to
remain held while paused, then requires worker cleanup before retry.

The UTXO case restarts the receiver against the same data directory
after cancellation and checks the unopened network state before allowing
a new connection. A persistent database regression also abandons a
verified receipt before promotion, makes SMT evidence incomplete,
reopens the database, and requires fresh authentication before promotion.

The pause controls compile only with a test feature and are armed
explicitly by the integration harness. Default node builds do not
include them. The preview remains default-off and Simnet-only.

Both parent validation and candidate validation are pending. These
tests do not authorize replacing the default promotion path.

## Exact validation checkpoints

PR #7 remains at `2ff50a53e727511a738c04fec6ddcd432c389790` on
`feature/phase6-production-import-preview-20261110`. [Run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
has passed seven jobs. Native focused and workspace results are pending.
The same-SHA push run 42930270803 was superseded.

The separate test candidate is `42a65a83e6d269618af826f4cda67ebe24b1d264` on
`feature/phase6-preview-peer-interruption-20261110`.
[Run 42931593794](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42931593794)
covers the interruption and reopen regressions and full workspace
validation. The initial test candidate `f429ec4` triggered run
42931436753, which reported three formatting changes. The candidate has
a new source head and validation is pending.
