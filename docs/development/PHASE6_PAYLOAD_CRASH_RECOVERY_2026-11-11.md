# Phase 6 nonempty import and abrupt restart candidate

November 11, 2026

## Baseline

This candidate is stacked on draft PR #8 at
`4f23ff9f77bed788085efe03441420e522ad9595`.
Implementation `bc886b6aaf9df3f2deceac75691f1a1bb932f9fa` passed
all eight jobs in run 42935509499, including 15 focused regressions,
1,568 workspace tests (25 skipped) and 32 no-asm tests.
No descriptor, fixture regeneration or recovery work is repeated.

## Implementation

The shielded fixture mines real Orchard coinbase notes, reconstructs a
wallet from canonical accepted-chain data, proves a payment against a
mature anchor after Simnet's existing anchor fork, submits it through
production RPC admission and mines it below the pruning point.
The source and recovered receiver must export identical nonempty
spent-nullifier sets.

A separate test-only parameter file uses transparent coinbase rewards
with the same small Simnet pruning parameters. Valid signed transactions
populate ten user SMT lanes. The pruning-point UTXO query must be
nonempty before the receiver connects. Existing shielded fixtures and
their assertions remain available.

The interruption controls distinguish the existing pre-payload pause
from a post-write pause. Shielded pauses after its atomic seed write and
before stable completion. UTXO pauses after a nonempty authorized append.
SMT pauses after a real DB batch write returns and before root/metadata
completion. Only an explicitly armed test feature uses single-entry SMT
batches; ordinary batch sizing and production features are unchanged.
The core rendezvous acquires no database or authority lock. Worker
reservations and pruning guards remain owned by the actual importer.

Cancellation bans the temporary peer while the write continuation is
paused. The reservation must remain held until release; the test then
requires draining, a genesis active receiver, closed active staging
authority, a same-directory reopen and a complete verified retry.

Three Unix-only parent tests spawn an actual integration-test child
process. It runs both daemons against parent-owned persistent paths and
signals only after the selected receiver write checkpoint. The parent
sends SIGKILL and checks signal 9, then opens the same sender and receiver
directories. Before starting network services the receiver must still be
at genesis, without a worker reservation, staging role or verified
promotion marker. A fresh proof/session import must complete, promote
through the existing verified boundary and close the activated role.
No graceful shutdown, cancellation drain or RocksDB close runs in the
killed process. Test children are killed on parent error as well.

## Validation status

Branch: `feature/phase6-payload-crash-recovery-20261111`.
Source and Actions run IDs will be recorded after the first push.
The implementation is a candidate, not yet validated.

Focused regressions add:
- `authority_integration_preview_interrupts_spent_nullifier_payload`
- `authority_integration_preview_interrupts_nonempty_utxo_payload`
- `authority_integration_preview_cancels_partial_smt_write`
- `authority_integration_preview_recovers_abrupt_smt_payload`
- `authority_integration_preview_recovers_abrupt_shielded_payload`
- `authority_integration_preview_recovers_abrupt_utxo_payload`

Focused CI runs with two test workers to bound simultaneous real proof
generation and daemon processes. No test is skipped or relaxed.
Full eight-job validation follows the focused step.

## Next executable task

Read the new draft PR head and Actions run. Resolve actual compilation,
formatting, Clippy or native failures before considering this candidate
validated. Require all new focused cases and the full workspace suite.
After green validation, review remaining production adapter and
promotion-boundary gaps while retaining the default-off Simnet preview.

All development uses the GitHub connection and GitHub Actions. Local
Windows, WSL and existing checkouts are outside the authorized workflow.
No private history, local files or project references are imported.
PRs #1–#8 remain intact. No merge, release or consensus activation is
authorized.
