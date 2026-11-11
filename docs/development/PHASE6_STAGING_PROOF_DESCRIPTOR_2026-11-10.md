# Phase 6 staging proof descriptor preservation

November 10, 2026

PR #7 at `ebcb3ce00043b0d3f73a2ee7508f0cc68a363a69` passed seven jobs in
[run 42909699012](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42909699012).
The focused suite passed ten regressions and failed the two-daemon
shielded preview. The default workspace suite did not run after that failure.

The production sanity check called the normal cached proof builder.
That builder optimizes an external descriptor and persists a local
descriptor, including changing its external flag. The rebuilt header
sets matched the accepted proof, but the durable descriptor no longer
matched the authenticated receipt at completion.

The sanity check now uses a reconstruction path that writes neither the
descriptor nor the normal proof cache. Ordinary proof construction keeps
its existing descriptor persistence. Exact descriptor verification and
fresh committed-header checks continue to apply at completion and
promotion.

The focused proof-worker regression checks reconstruction header sets,
exact descriptor row bytes before and after reconstruction, and the
original authenticated evidence after the rebuild. The unchanged
two-daemon test exercises the production sanity check and verified
promotion. Candidate CI is pending.

After the fix passes, continue with peer-driven stream interruption and
pre-promotion restart coverage. The preview remains default-off and
Simnet-only.

## Exact validation checkpoints

PR #7 remains at `2ff50a53e727511a738c04fec6ddcd432c389790` on
`feature/phase6-production-import-preview-20261110`. [Run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
has passed seven jobs. Native focused and workspace results are pending.
The same-SHA push run 42930270803 was superseded.

The separate test candidate is `42a65a83e6d269618af826f4cda67ebe24b1d264` on
`feature/phase6-preview-peer-interruption-20261110`.
[Run 42931593794](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42931593794)
covers the interruption and reopen regressions and full workspace
validation. The initial test candidate `f429ec4` triggered run
42931436753, which reported three formatting changes. The candidate has
a new source head and validation is pending.
