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

## Exact continuation checkpoint

Draft [PR #10](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/10)
has source `509e54b5af6a686b24086eea30fe97e8dc02af23` on
`feature/phase6-active-admission-regression-20261111`.
Authoritative [PR run 43076086692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43076086692)
is pending, waiting for cancellation of superseded push run 43076078613.
No native result or full validation is claimed for PR #10 yet.
PR #9 remains green at implementation
`110d1625763c9fecd2cf2a226e5e7fb25add99bf`, run 43068789717,
and documentation head `9599133be7737b8006cab373d9a99d8fe9bafff8`.

Next, retrieve PR #10's exact source/run, correct actual formatting,
compilation, Clippy or native failures, then require all eight jobs.
Do not repeat the completed PR #9 fixture, interruption or crash work.
