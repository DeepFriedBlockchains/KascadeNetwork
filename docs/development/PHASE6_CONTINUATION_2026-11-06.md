# Phase 6 development continuation · November 6, 2026

## Public validation stack

PR #1 starts the public validation workflow at `ba4ce3072bd7785e1b3061647f2dacbba044d8e6`.
PR #2 applies the CI repairs at `91da4d8f728fe147529420dbaaa5e628765fd8f3`
on PR #1. PR #3 adds the regenerated fixtures on PR #2. Its
pre-regeneration checkpoint was `ed4abfe3a105b45145e7449b91149b52544f6262`.

## Regeneration and release replay

[Regeneration run 42341317991](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42341317991)
at `4493d54f29652b6804d8a1bbea2b7e1bd09e9afb` generated and verified all three DAGs. It ran the
generation test (1 passed), independently checked the full set of headers
and parent links, and replayed the DAGs in release mode (5 passed, 0 failed,
the explicit generation test ignored during replay). The output job
verified all three SHA-256 digests and stored the fixtures at
`868a0db679755b93349567bce698d068a8c69de6`. The upstream archives are byte-preserved.

| Fixture | Blocks after genesis | Spends | Merge blocks | Distinct DAA bits |
| --- | ---: | ---: | ---: | ---: |
| Small transaction DAG | 265 | 224 | 6 | 1 |
| No-transaction DAG | 5,000 | 0 | 125 | 121 |
| Custom pruning depth DAG | 5,000 | 4,790 | 125 | 1,861 |

The canonical genesis payload is 63 bytes and every regenerated non-genesis
payload is 93 bytes. The custom pruning case spans 5,000 blue-score units
at a depth of 450 and its replay asserts a non-genesis pruning point.
The small DAG retains its historical file path. Its length exercises
transaction and merge validation while the longer DAGs cover DAA variation.

The earlier [PR #3 run 42225379635](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42225379635)
passed seven of eight jobs. Its native suite recorded 1,129 passes, 3
failures, 24 skipped and 386 unrun. The
[companion run 42341318436](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42341318436)
on the generator source passed the seven compilation, lint, Linux and
WASM checks. The native job still used the upstream-format active fixtures
and recorded 1,129 passes, four legacy coinbase failures, and 25 skipped.
The final PR #3 commit pairs the validated source with the regenerated blobs
and requires fresh combined workspace validation.

## Next executable task

Verify the exact PR #3 head's complete workspace, lint, release and WASM
results. Review the stacked bases before requesting integration review.
After CI stabilization, compile and test the isolated database-readiness and
owner-token wrapper. Follow with processor-bound atomic database writes and
verified restart recovery. Consensus activation remains gated.

## November 7 workspace follow-up

[Combined run 42343429452](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42343429452)
at `cc4e800e8fae4ddf0997ba68961ca5ffbee202b5` passed all five goref tests and seven of eight workflow jobs.
The native run reported 1,134 passed, one failed, 25 skipped and 383 unrun.
The failing header-isolation test expected the pre-Toccata block version
while using the active mainnet fork configuration. Pinning its test-only
fork configuration preserves the original version-rejection assertion;
the separate activation test exercises the version transition. The workspace
job now reports all test results even if a test fails. The corrected head
requires a fresh combined run before the stack can be called green.
