# KASCADE public development handoff

Updated November 7, 2026.

Public main starts at the reviewed root `62248169f0920277b3ee60cf275080c221a21786`.
The validation stack is [PR #1](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/1),
then [PR #2](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/2), then
[PR #3](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/3).

The regenerated goref fixtures are on PR #3. The
[hosted regeneration run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42341317991)
passed all five targeted release replay tests and the independent verifier.
The upstream archive is preserved. The generated blobs were verified at
`868a0db679755b93349567bce698d068a8c69de6`. See the
[dated continuation report](docs/development/PHASE6_CONTINUATION_2026-11-06.md) for exact test and DAA counts.

[Combined run 42343429452](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42343429452)
passed all five goref tests and seven jobs. Its one native failure was a
pre-Toccata header assertion using mainnet's active fork configuration.
The corrected header test and all goref cases passed in
[run 42345506966](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42345506966).
That run completed all 1,518 selected tests, with 1,466 passes and 52
failures in legacy test profiles, daemon/simulator fixtures, and address
vectors. The next combined run validates those corrections.

After the full workspace is green, continue the inactive database-bound
authority wrapper with committed readiness checks and import-owner
enforcement. Consensus activation remains gated.
