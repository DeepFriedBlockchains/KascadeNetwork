# Phase 6 native CI timing and reopened selected ancestry

November 23, 2026

## Latest checkpoint — November 23, 2026

| Draft | Branch | Exact commit | Actions evidence |
| --- | --- | --- | --- |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), **8/8 passed**. Focused Simnet genesis and issuance conservation passed; 39 authority, 1,580 regular native (25 skipped), and 32 no-asm passed. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), 7/8 passed. Both focused payout regressions passed. Native 137560443988 running regular workspace tests. |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), 7/8 passed. Focused checkpoint persistence passed. Native 137563153013 advanced past the devnet-prealloc build into authority regressions. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), 7/8 passed. Focused five-minute sample and [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666/artifacts/14801585581) passed. Native 137559872891 passed regular workspace tests and advanced into no-asm checks. |
| [#42](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320), 7/8 passed. Three-process CLI check and [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320/artifacts/14800768777) passed with simulated PoW. Native 137568406534 advanced into authority regressions. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged behind #39. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `test/phase6-issuance-checkpoint-process-exit-20261123` | `e87401b6bca55a5684f513beb0fa893fed862d24` | Staged behind #39. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `test/phase6-cli-real-pow-recovery-20261123` | `6d03bcb4b55bf75234be6322b2878f27801843e6` | Native-PoW test and artifact checks extended. Staged behind #42 and unvalidated. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `test/phase6-issuance-current-selected-path-20261123` | `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b` | Test-only code pushed. Staged behind #41 and unvalidated. |

The remaining native jobs showed step progress. GitHub's Actions API did not return their live logs. Two completed comparable native jobs took approximately 49 and 58 minutes from the first Cargo command to the last. Timestamped logs show separate feature builds and 6m31s–8m13s in regular Nextest execution itself. No failed, queued, cancelled or stalled job was observed in these runs. 

PR #45 now reruns the child-body and selected-chain checks against a reopened RocksDB handle. It rejects the stale sibling with either tip, checks that descendant state includes only the selected sibling's fixture nullifier, and verifies the settled checkpoint commitment before testing paid/deferred corruption. The test uses fixture-seeded ancestry and illustrative commitments.

PR #43 checks the exact 12-block native-PoW count from genesis, confirms all three nodes agree on the selected tip, block count, pruning point and virtual DAA score after restart, and adds the final DAG fields to the recovery artifact. No native-PoW result is claimed before Actions exercises the updated head.

**Next:** On full green #39, run #41 and #44 with their focused checks and full matrices. On full green #42, run #43, retain the native-PoW test output and record restart/catch-up and per-node resource data. On full green #41, run #45. Authenticated pruning restoration tied to real settlement and atomic readiness, production hard-cap accounting, representative native/AuxPoW load, and approved network parameters remain gates. The proposed finite policy remains 29B KCADE maximum with a 21B primary allocation and 8B tail. Production issuance remains disabled.



## Timing evidence

Durations below use Cargo's build completion timings and Nextest's own test summaries. The first-to-last comparison uses the timestamp of the first Cargo command and completion of the final Cargo example. Setup, cache handling and runner availability are outside that span.

| Native job | First focused consensus build | Focused integration build | Devnet-prealloc build | Authority build / tests | Regular build / tests | First-to-last Cargo span |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [#23 job 137416068338](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439/job/137416068338) | 11m05s | 11m36s | 7m19s | 4m28s / 4m04s | 4m22s / 8m13s | 57m38s |
| [#25 job 137428872019](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286/job/137428872019) | 9m18s* | 9m54s | 5m56s | 3m39s / 3m49s | 3m34s / 6m31s | 48m31s |
| [#32 job 137560421199](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319/job/137560421199) | 7m57s | 7m59s | 5m04s | 3m26s / 2m47s | 3m07s / 4m15s | See job log |

\* #25 first focused consensus step wall time from the command timestamp to the next command. It includes Cargo startup and the filtered test execution.

These results explain how the native job can trail the other jobs even while progressing. Keep the running commits and coverage intact. If a job fails, inspect the first failed step and its log before choosing a retry. The follow-on implementation gap is proving that verified pruning input carries the live settlement state through the guarded atomic promotion. The selected-ancestry fixture covers the reopened stores used by that work.
