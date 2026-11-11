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

## Initial push checkpoint

Initial source `ccb68579d2cdb4b4770bcf679e8776f45418dddf` opened
[draft PR #9](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/9).
PR run 43066415215 queued behind cancellation of push run 43066407288.
The follow-up holds the UTXO done frame until disconnect, preventing a
buffered completion frame from turning an interrupted test into a
legitimate complete import. Reservation retention after disconnect is
asserted while the native SMT writer is still outstanding; completed
atomic writers may correctly finish cancellation immediately.

## Formatting correction

Source `1c1042147e4b68b17460a46a85a55d7b961f71d0` reached
PR run 43067275773. Check (all tests with preview features), Check
WASM32 and Check no_std passed. Lints stopped on 13 Rustfmt hunks
in the new integration helpers and pause controller; strict Clippy did
not run. The exact formatter diff from the GitHub job was applied.
Native and remaining build jobs were still pending at correction time.
No native pass is claimed for this candidate yet.

## Active-window fixture strengthening

Formatting source `13a1e3de896eed9deb59e535f2431209be18b663` passed
Check, strict Clippy, Check WASM32, Check no_std and Test WASM32 in
run 43067753790. Native validation was still building.

The transparent fixture now refreshes its ten signed user lanes every
50 mined blocks, including the extra pruning-advance loop. SMT export
filters lanes to the pruning point's finality window, so a one-off early
transaction set could expire before that point advances. The fixture
now asserts at least ten active lanes at the actual pruning point,
ensuring the partial-write test cannot become an empty import.

## Current validation checkpoint

[draft PR #9](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/9)
is at implementation `110d1625763c9fecd2cf2a226e5e7fb25add99bf` on
`feature/phase6-payload-crash-recovery-20261111`.
Authoritative [PR run 43068789717](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43068789717)
is pending. Same-SHA push run 43068779387 was superseded.
Formatting, strict Clippy, Check, Check WASM32 and Check no_std passed.
Native focused and complete workspace tests have not finished. Expected
focused additions are the six regressions above; require the exact
observed counts from the completed job logs before reporting a pass.

Next, retrieve this exact source/run, fix actual native failures and
complete all eight jobs. Do not restart fixture/provenance recovery,
descriptor work, or the completed PR #8 regressions.

## Focused native result

Run 43068789717 completed the focused authority integration step successfully.
All six new payload/cancellation/SIGKILL-recovery cases are included in that
filter. The regular workspace suite is now running. Seven other jobs passed.
Exact test counts will be copied from the completed native job log. Full
eight-job validation remains pending; keep source `110d1625763c9fecd2cf2a226e5e7fb25add99bf` unchanged while it runs.
