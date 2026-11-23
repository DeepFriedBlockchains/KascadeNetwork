# Phase 6 CLI runbook exercise checkpoint

November 23, 2026

| Draft | Branch | Exact source | CI at checkpoint |
| --- | --- | --- | --- |
| [#39 Atomic checkpoint](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) 2/8; bounded reopen running, focused model pending. |
| [#41 Selected-child binding](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged behind #39. |
| [#32 Genesis and issuance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 6/8; focused genesis/replay pending. |
| [#34 Red-block and deferred payout](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 6/8; focused builder/live merge pending. |
| [#40 Five-minute CPU/RSS](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 6/8; measurement pending. |
| [#42 Actual CLI runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320) pending. Dedicated native step exercises real processes. |

No failures were reported on these active runs at the checkpoint. #42 now exercises the original three documented loopback commands with isolated temporary paths. The added simulated-PoW recovery pass only restarts A with its explicitly documented test mining flag. It requires advancing two blocks, C catching up from its existing path after missing the second block, matching pruning point/virtual DAA values, and clean SIGINT process shutdown. Collect `phase6-cli-runbook` logs before marking it verified. Earlier multi-node in-process and simulated-PoW evidence stays pinned.

Next actions: verify #39's focused persistence test and all eight jobs before triggering #41. Check #32 and #34's own focused assertions and record CPU/RSS from #40. Review #42's real process driver and per-node logs first on a failure. Keep the 29B total, 21B primary, 8B tail and disabled production issuance. Representative real mining and authenticated pruning restore with actual settlement remain acceptance gates.
