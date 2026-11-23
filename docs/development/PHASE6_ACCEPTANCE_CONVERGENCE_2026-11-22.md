# Phase 6 acceptance convergence

November 22, 2026

This draft stages the independently validated archive recovery and three-node partition/rejoin branches. Its source parents are the documentation checkpoints from both drafts. The archive branch owns the consensus, daemon test and native workflow changes. The partition branch adds a disjoint integration test and library module registration. Both source and documentation histories remain reachable.

| Source | Validated implementation | CI |
| --- | --- | --- |
| [#17 shielded archive](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) | `11ab68a3be26fc9616a91d3d06ac5d99bc23679b` | [45069228114, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114/attempts/3), 8/8 |
| [#19 partition/rejoin/restart](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/19) | `978584bbd91e81b8b637bbebb6655426fe207d9d` | [45069680867, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3), 8/8 |
| [#22 native and AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439). 8/8. The dedicated native and AuxPoW relay regressions both passed. |

The combined tree stages the exact relay source alongside #17 and #19. The workflow keeps the archive backfill check, adds the three-node case to the focused step, and runs both mining relay cases before broader CI. The combined implementation is staged at `25cfeed643d40b35f906332e4b024dce23632ae0`. Documentation commit `c542833e385f1f222538f42bc72a9843a1cd2ca4` started the full combined CI matrix after verification of the #22 baseline. The Phase 6 preview remains default-off and Simnet-only. Shielded consensus activation is disabled. The PR remains draft and unmerged.

First-party results for #22's [native job 137401920040](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439/job/137401920040) include 2/2 focused relay tests, 37/37 Phase 6 tests, 1,579/1,579 workspace tests, and 32/32 no-asm tests. The workspace suite skipped 25 tests. Separate Check, strict lint, Linux release, WASM32 check/test/SDK, and no-std jobs all passed.

## Combined Simnet validation

All eight jobs passed on exact CI head `c542833e385f1f222538f42bc72a9843a1cd2ca4` in [run 45093748286](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286). The [native job 137428872019](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286/job/137428872019) passed persistent reopen, all three bounded cleanup regressions, authenticated export/descendant recovery, the three focused archive/partition checks, both focused PoW relay checks, 39 targeted Phase 6 regressions, 1,579 workspace tests with 25 skipped, and 32 no-asm tests. Strict lint, Check, Linux release, WASM32 check/test/SDK, and no-std passed.

This is the accepted combined Simnet baseline. Sustained throughput, real external mining interoperability, and launch-profile activation evidence remain open.
