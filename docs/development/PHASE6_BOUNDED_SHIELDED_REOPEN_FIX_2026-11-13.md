# Phase 6 bounded shielded persistent reopen correction

November 13, 2026

## Verified starting point

PUBLIC DeepFriedBlockchains/KascadeNetwork, draft PR #12.
Branch: feature/phase6-bounded-shielded-ingestion-20261113.
Remote handoff head: 434d71867d746b7664e93465824483a602cb21f1.
Implementation acba224032ab7613ac805bcb33160fc696436f2c passed seven
of eight jobs in run 43511658558. Native job 132183262907 failed the
persistent reopen regression. Two focused tests passed, one failed,
and 28 of the 31 focused cases were not run due to fail-fast. The full
workspace suite did not execute. PR #12 is not validated.

No newer open PR or PR #12 head was present on resume. PRs #1–#11
and their validated heads remain unchanged.

## Root cause and correction

The panic at header_processor/processor.rs:373 was HashAlreadyExists
for the genesis GHOSTDAG hash during commit_header. The test had closed
a database with committed genesis and partial nullifier membership, then
called TestConsensus::with_db using the original fresh-database config.
That constructor honors process_genesis; it is not a production factory
that detects an existing active consensus entry.

Production Factory::new_active_consensus explicitly sets process_genesis
false for ConsensusEntryType::Existing and true only for a new entry.
The fixture now reopens through config.to_builder().skip_adding_genesis(),
matching that existing-database contract. Production header insertion,
authentication and recovery behavior are unchanged. This is a test-fixture
initialization error, not evidence of a defect in the production recovery
path. The constructor's duplicate-genesis rejection remains intact.

All prior assertions remain: real physical close/reopen, persisted partial
membership, false readiness, inactive staging role, refusal to reuse dirty
rows, explicit cleanup, complete fresh retry and stable two-row completion.
A new exact-row assertion also requires the persisted unready marker to
exist before closing and remain byte-for-byte identical after reopening.

## Validation

The native Test Suite now runs this exact regression first, with a
backtrace, before workspace build, the 31-test authority/bounded selection
and the complete workspace suite. All eight existing CI jobs remain.
No tests or safety assertions are skipped or relaxed. Source SHA and new
Actions IDs will be recorded after push. Focused and full validation are
pending; PR #12 must not be marked validated until all eight jobs pass.

## Next checkpoint

Finish the focused regression and full run. Fix actual failures. Only
after PR #12 passes, create a new stacked draft PR for bounded failed-import
cleanup under the same authenticated owner permit, preserving unready
durability, worker reservations and the no-validated-descendants condition.
Descendant recovery and export snapshots follow as separate milestones.

All source edits and validation use GitHub integration and Actions only.
The preview remains default-off and Simnet-only. No merge, release,
force-push or consensus activation is authorized.

## Pushed correction checkpoint

Correction source: `f1808f1231697474de66bf3f99bad0c2f01ead0d`.
Remote PR #12 head was verified after push. Authoritative replacement
PR run: [43515652940](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43515652940).
Duplicate push run 43515643677 is being cancelled by workflow concurrency;
the PR run is pending while that cancellation finishes. No replacement
focused or full validation result is claimed. Native job order is the
single persistent-reopen regression, then workspace build, 31 focused
cases and full workspace validation. Resume this run rather than rerunning
the prior known-bad source. PR #12 stays draft and unvalidated.

## Isolated validation branch for stalled Actions cancellation

The push run 43515643677 has seven cancelled jobs but Lints job
132196802751 still reports in_progress with no steps. PR run 43515652940
remains pending with no jobs. The GitHub connector has no cancellation
action; no local or browser access is used as a workaround.

Branch `validation/phase6-bounded-reopen-20261113` carries the exact
same corrected Rust source and eight-job workflow as f1808f1, plus this
documentation checkpoint. Its push uses a different existing concurrency
group, avoiding the stuck group without modifying the workflow or its
checks. PR #12 is fast-forwarded to the same commit so that the exact
latest head is tested. No history rewrite or additional PR is involved.
The new validation push run is authoritative once its exact SHA and all
eight results are verified. Source and workflow equivalence will be
checked through GitHub's commit comparison. No pass is claimed yet.

## Active authoritative replacement run

Validation commit: `fa229e6524a78bb3bcd032d08e861f62a926637e`.
Both the PR #12 feature branch and `validation/phase6-bounded-reopen-20261113`
were verified at that commit. All eight jobs have started in push run
[43516463729](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43516463729).
GitHub's comparison f1808f1...fa229e6 lists only CURRENT_HANDOFF.md and
this correction report: all Rust source, tests and workflow are identical.
The original PR group's newer run 43516476491 remained pending; feature
push 43516466968 was cancelled. Use 43516463729 for actual validation,
not those duplicates. No replacement test result is claimed yet.
Any later handoff head is documentation-only unless explicitly recorded.

## Focused reopen regression passed

Authoritative run 43516463729 at validation commit
`fa229e6524a78bb3bcd032d08e861f62a926637e`, native job
132199538931, completed the dedicated persistent-reopen step successfully.
This was independently confirmed through the job-steps endpoint. All
retained restart, dirty-row, retry and readiness assertions, plus the
exact persisted-marker check, executed in that step. This confirms the
fixture initialization correction. The job has continued to workspace
build; the broader 31-case selection and full workspace suite are still
pending. PR #12 remains unvalidated until all eight jobs finish
successfully. Bounded cleanup remains gated on that result.

## PR #12 verified eight-job result

Validation commit `fa229e6524a78bb3bcd032d08e861f62a926637e` passed
all eight jobs in [run 43516463729](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43516463729).
The completed native job 132199538931 confirms the dedicated persistent
reopen regression passed first, all 31 authority/bounded cases passed,
1,576 workspace tests passed (25 existing skips), and all 32 no-asm tests
passed. Workspace and no-asm doc tests, cargo doc, the kip-10 example,
formatting, strict Clippy and platform/build checks also passed.

This independently validates the fixture-only correction at
`f1808f1231697474de66bf3f99bad0c2f01ead0d`: populated-database reopen
skips duplicate genesis initialization exactly as production Factory does.
No assertion, authentication check, header insertion rule or restart
guarantee was weakened. The exact persisted false marker assertion passed.

GitHub comparison from the validating commit through handoff head
`3ef81662656decfb6ba89595c0c82cdc4c38bf29` contains only the two
handoff documents. This milestone commit also changes documentation only.
PR #12 remains draft, unmerged and inactive. Its source is validated.

Next: a new stacked draft PR for bounded failed-import cleanup.
Keep descendant recovery and export snapshots as later milestones.
