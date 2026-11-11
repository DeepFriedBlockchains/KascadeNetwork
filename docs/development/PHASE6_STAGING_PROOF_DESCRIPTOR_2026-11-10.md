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
