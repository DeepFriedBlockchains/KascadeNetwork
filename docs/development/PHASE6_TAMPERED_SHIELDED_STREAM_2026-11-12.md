# Phase 6 tampered shielded stream candidate

November 12, 2026

## Baseline

Stacked on PR #10 at `a0885ad19eac58edec15283394e9ad78cacc98d2`.
Implementation `509e54b5af6a686b24086eea30fe97e8dc02af23` passed all
eight jobs in run 43076086692, with 21 focused tests, 1,568 workspace
tests (25 skips) and 32 no-asm tests.

## Implementation

The real Orchard-spend fixture now supports a tampered-wire mode.
An explicitly armed test-only control flips one byte of the first
nonempty nullifier frame. Counts, authenticated metadata, proof and
source database remain unchanged. A pre-send rendezvous makes the
initial temporary peer connection deterministic.

The receiver observation is recorded only after the actual shielded
worker has joined with an error. Both new production-path regressions
require the committed accumulator mismatch, complete staging cleanup,
genesis/no promotion/no active admission, then a fresh valid retry.
One case also reopens the same receiver directory after rejection.
The existing fixture requires the exact legitimate nullifier set,
durable verified promotion, post-activation admission rejection and
post-IBD block acceptance. Corruption is one-shot and disarmed before
retry.

The controls compile only with test-staging-interruption. Ordinary
sender payloads and production authentication are unchanged.
No verifier or safety assertion is relaxed.

## Validation checkpoint

Branch: `feature/phase6-tampered-shielded-stream-20261112`.
Implementation and run IDs will be recorded after push. Validation is
pending. Focused additions:
- `authority_integration_preview_rejects_tampered_nullifier_payload`
- `authority_integration_preview_reopens_after_tampered_nullifier_payload`

Require focused regressions, strict Clippy, formatting and all eight
Actions jobs before treating this candidate as validated.

## Next milestone

After independent tampered-stream validation, implement bounded-memory
shielded ingestion. The current importer buffers the full nullifier
set and the seed builds one large RocksDB batch. Preserve authenticated
root/count verification, duplicate rejection, worker reservations,
session ownership, atomic readiness publication, cancellation and
restart safety. Do not change consensus or monetary rules.

GitHub and GitHub Actions are the exclusive development environment.
The preview stays default-off and Simnet-only. PRs #1–#10 remain intact;
no merge, release or activation is authorized.

## Initial validation checkpoint

Initial source `c309386d2f18568b5ee4e91156ff6b9dde378d8e` opened
draft PR #11 and PR run 43308475970 (push 43308466345 superseded).
Lints stopped on five Rustfmt layout hunks; strict Clippy did not run.
The exact CI formatter output was applied without changing assertions.
The replacement source/run will be recorded in the PR description.
Native validation is pending.
