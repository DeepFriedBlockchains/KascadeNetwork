# Phase 6 acceptance convergence

November 22, 2026

This draft stages the independently validated archive recovery and three-node partition/rejoin branches. Its source parents are the documentation checkpoints from both drafts. The archive branch owns the consensus, daemon test and native workflow changes. The partition branch adds a disjoint integration test and library module registration. Both source and documentation histories remain reachable.

| Source | Validated implementation | CI |
| --- | --- | --- |
| [#17 shielded archive](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) | `11ab68a3be26fc9616a91d3d06ac5d99bc23679b` | [45069228114, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114/attempts/3), 8/8 |
| [#19 partition/rejoin/restart](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/19) | `978584bbd91e81b8b637bbebb6655426fe207d9d` | [45069680867, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3), 8/8 |

| [#22 native and AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439). Dedicated relay step passed. Full native job running at staging. |

The combined tree stages the exact relay source alongside #17 and #19. The workflow keeps the archive backfill check, adds the three-node case to the focused step, and runs both mining relay cases before broader CI. The separate #22 run remains the first validation target. Once it passes all eight jobs, update this report with its final result and let that normal push trigger combined CI. Record the resulting exact tested SHA. Keep the default-off, Simnet-only preview, shielded consensus activation disabled, and the PR draft and unmerged.
