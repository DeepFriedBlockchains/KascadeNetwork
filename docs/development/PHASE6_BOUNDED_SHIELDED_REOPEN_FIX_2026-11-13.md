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
