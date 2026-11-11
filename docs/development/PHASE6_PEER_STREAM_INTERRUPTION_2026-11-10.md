# Phase 6 peer stream interruption candidate

November 10, 2026

This test candidate builds on descriptor correction `2ff50a53e727511a738c04fec6ddcd432c389790`.
The correction's [run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
passed all eight jobs. The separate branch preserves PR #7's tested source.

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

Parent and candidate validation are complete. These
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

## Exact continuation checkpoint

[Draft PR #8](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/8)
is stacked on PR #7. Source `bc886b6aaf9df3f2deceac75691f1a1bb932f9fa`
on `feature/phase6-preview-peer-interruption-20261110` includes the role
correction. [Run 42935509499](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42935509499)
passed all eight jobs. Push run 42935435587 was superseded.

Parent PR #7 at `2ff50a53e727511a738c04fec6ddcd432c389790` passed all
eight jobs, 11 focused tests, 1,567 workspace tests (25 skipped), and
32 no-asm tests in run 42930280069.

The shielded fixture has no spent nullifiers, so its interruption is
before metadata. The transparent UTXO fixture is empty, so its pause is
before the done frame. The same-directory restart is after cancellation
drains the workers. Abrupt termination before promotion and nonempty
shielded/UTXO stream interruption remain separate coverage.

All 15 focused regressions and the complete eight-job suite passed.
Keep the preview default-off and Simnet-only.

## Focused regressions

- `authority_integration_preview_cancels_interrupted_smt_peer`
- `authority_integration_preview_cancels_interrupted_shielded_peer`
- `authority_integration_preview_restarts_after_interrupted_utxo_peer`
- `authority_integration_reopen_before_promotion_rejects_incomplete_evidence`

The first, second and fourth cases passed in run 42932087535. The third
failed the retained assertion that a reopened active database must not
advertise staging import support. Source `bc886b6` corrects that role
classification. The existing successful production preview test now
checks the durable promotion record and revocation of the staging role.

## Completed validation

Exact implementation `bc886b6aaf9df3f2deceac75691f1a1bb932f9fa`
passed [all eight jobs in run 42935509499](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42935509499).
The focused suite passed 15 tests. The regular workspace suite passed
1,568 tests with 25 skipped. The no-asm suite passed 32 tests.
Formatting, strict Clippy, workspace doc tests, no-asm doc tests,
cargo doc, kip10 example, Linux release, WASM SDK, WASM tests,
WASM checking and no-std checking all passed.

The previously failing
`authority_integration_preview_restarts_after_interrupted_utxo_peer`
passed, as did the SMT and shielded peer cancellation/retry tests,
the persistent-evidence reopen test and the original successful
two-daemon verified promotion test. No assertions were removed or relaxed.

This follow-up documentation commit changes only Markdown. It preserves
the exact tested implementation. The source commits remain public and
the PR #1–#8 stack remains open.

## Next executable checkpoint

Continue from PR #8's latest branch head, with tested implementation
`bc886b6aaf9df3f2deceac75691f1a1bb932f9fa`.
Extend `run_shielded_ibd` in
`testing/integration/src/daemon_integration_tests.rs` with valid spent
nullifiers and a separate nonempty UTXO fixture. Interrupt after
payload writes begin, using the existing test-only transfer controls.
Add abrupt termination before promotion against the same data directory.
Use a new feature branch stacked on PR #8. No implementation CI remains
pending, and no owner decision blocks this next test milestone.
Merge and activation remain subject to separate owner approval.
