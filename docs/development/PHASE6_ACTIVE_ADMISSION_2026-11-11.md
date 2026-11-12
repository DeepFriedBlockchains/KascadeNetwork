# Phase 6 active-receiver admission regression candidate

November 11, 2026

## Validated baseline

This candidate is stacked on PR #9 at documentation head
`9599133be7737b8006cab373d9a99d8fe9bafff8`.
Its implementation `110d1625763c9fecd2cf2a226e5e7fb25add99bf`
passed all eight jobs in run 43068789717: 21 focused regressions,
1,568 workspace tests (25 skips) and 32 no-asm tests.
The real payload and SIGKILL/reopen milestones are complete.

## Additional implementation

The production preview harness obtains a real proof through the
descriptor-preserving reconstruction API and validates it against the
ordinary active receiver before IBD. Actual
`begin_verified_staging_import` admission must return `Unsupported`;
the test does not rely only on the support diagnostic.

A second genuinely validated receipt is retained across the complete
import and promotion. Admission on the activated receiver must again
return `Unsupported`, and the durable verified-promotion record and
closed staging role must remain intact. The same helper is exercised
by the persistent SIGKILL/reopen cases, so ordinary reopened active
databases receive the same explicit negative admission test.

Receipts are created only by production proof validation. No test
constructor, forged token, reduced proof check or production authority
change is introduced. Proof validation runs in blocking workers before
the import pause is armed.

## Validation checkpoint

Branch: `feature/phase6-active-admission-regression-20261111`.
Source and Actions run IDs will be recorded after push.
Validation is pending. Existing positive, cancellation and restart
assertions remain. Require focused regressions and all eight jobs
before treating this candidate as validated.

## Next executable task

Retrieve the latest candidate source and Actions run, fix actual
failures and complete focused/full validation. Then extend the real
nonempty fixture to reject a tampered nullifier stream before
publication and require a complete fresh retry. The existing shielded
importer still buffers the full declared set; bounded-memory ingestion
is another production-readiness item requiring preservation of the
count cap, duplicate detection, authenticated root and owner/session
checks.

PRs #1–#9 remain intact. The preview remains default-off and Simnet-only.
No merge, release, private-history import or consensus activation is
authorized. Development and validation use GitHub exclusively.

## Exact handoff checkpoint

Current implementation: `509e54b5af6a686b24086eea30fe97e8dc02af23`.
Branch: `feature/phase6-active-admission-regression-20261111`.
Draft PR: [#10](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/10).
Authoritative [run 43076086692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43076086692)
is in progress. Push run 43076078613 was superseded.
Formatting passed; Test WASM32, Check WASM32 and Check no_std passed.
Check, strict Clippy, native Test Suite and Linux/WASM SDK release
builds were pending when this checkpoint was committed. No focused or
full native pass is claimed for this candidate. Native job ID is
130718919553. This Markdown-only continuation changes no implementation
and intentionally leaves that exact source run in progress.

PR #9 is complete at implementation
`110d1625763c9fecd2cf2a226e5e7fb25add99bf`, documentation head
`9599133be7737b8006cab373d9a99d8fe9bafff8`, run 43068789717,
all eight jobs passed. Counts are 21 focused, 1,568 regular workspace
with 25 existing skips, and 32 no-asm tests.

Next executable task: fetch PR #10's latest head and run 43076086692.
Fix actual failures in the retained-receipt admission regression, then
require all eight jobs. Preserve `Unsupported`, the real proof
validation, closed-role, promotion-record, cancellation and restart
assertions. Do not repeat the completed PR #9 payload or crash work.
No owner decision blocks continued testing. Merge, release and
consensus activation remain unauthorized.

## Final session checkpoint

Implementation `509e54b5af6a686b24086eea30fe97e8dc02af23` remains
under [run 43076086692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43076086692).
Formatting, strict Clippy, Check, Test WASM32, Check WASM32 and
Check no_std and WASM SDK release passed. Six jobs have completed
successfully; native Test Suite and Linux release remain pending. The native job is 130718919553. No new native
pass is claimed for PR #10. Markdown-only handoff commits do not change
the validating implementation.

The next session should verify all eight jobs of that exact run.
If native tests fail, inspect the retained real proof validation and
actual admission rejection in `run_payload_ibd_at`; preserve
`Unsupported`, durable-promotion and closed-role assertions.
PR #9's real payload, cancellation and SIGKILL cases are complete and
passed; do not reimplement them. After PR #10 turns green, continue
tampered nonempty nullifier stream rejection and authenticated retry.
No owner decision is needed for continued tests.

## Validation completed November 12, 2026

Run 43076086692 completed all eight jobs at implementation
`509e54b5af6a686b24086eea30fe97e8dc02af23`.
Observed counts: 21 focused passed (1,583 excluded), 1,568 regular
workspace passed (25 existing skips), and 32 no-asm passed.
Formatting, strict Clippy and all remaining checks passed.
The former pending checkpoint is historical. Continue with tampered
nonempty shielded stream rejection; preserve PR #10's validated head.
