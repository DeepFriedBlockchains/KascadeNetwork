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

## Restart finding

Run 42932087535 passed seven jobs and 14 of 15 focused regressions.
The UTXO interruption drained successfully, but reopening the active
database incorrectly advertised staging support. The factory skips
genesis processing for an existing active database, so that configuration
flag does not identify a staging receiver.

The correction gives freshly allocated staging receivers an explicit
in-memory role. The controller revokes it when making the receiver
active, and reopened active databases start without it. The peer test
checks the durable verified promotion record after success and requires
the active receiver role to be closed. The original restart assertion
remains in place.
