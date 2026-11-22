# Phase 6 descendant recovery, November 22, 2026
November 22, 2026. Branch `feature/phase6-descendant-recovery-20261122` is stacked on draft PR #13
at documentation checkpoint `f74a4421c0f48f71931ee2d5be646d2872567056`.
See [the development report](docs/development/PHASE6_DESCENDANT_RECOVERY_2026-11-22.md).

When IBD encounters an unready shielded state with validated descendants,
the Simnet preview can verify the existing state and durably restore the
shielded readiness flag without running a destructive import. It requires
the authenticated pruning child and a selected-header child that commits the
pinned virtual tip. The committed selected chain, descendant diffs, tip
membership, and pruning-point projection must all agree. The full
historical membership set is streamed, with exact duplicate detection
over selected descendant diffs. On error readiness remains closed.

The persistent reopen fixture now checks rejected evidence, changed tip,
wrong commitment child, gated readiness publication and promotion record
preservation. GitHub Actions must pass the dedicated regression and all
eight jobs before advancing to authenticated export snapshots. The preview
remains default-off and Simnet-only.

Draft [PR #14](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/14) first ran at
`e9dd9376c6bc8d8e0fbfe1c354315133a4c5a715` in [run 45048948814](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45048948814).
Rustfmt reported three layout changes, applied in this revision. The
revision also pins the latest selected header tip through the final ancestry
check and marker commit. Complete validation is pending.
