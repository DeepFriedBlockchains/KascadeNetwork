# Phase 6 validated matrices and follow-ups

November 23, 2026

| Draft | Branch | Exact source | Result |
| --- | --- | --- | --- |
| [#30](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `test/phase6-reward-claim-accounting-20261122` | `29707a918033aa62a0d763509f8629ee0e6562c4` | [45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081), 8/8. Focused replay and RocksDB reopen; 39 authority, 1,580 workspace. |
| [#31](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31) | `test/phase6-merge-fee-accounting-20261122` | `5228e73e9b8a3981f611a5cee512fbe24b50f5f3` | [45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993), 8/8. Focused fee-only red subsidy; 39 authority, 1,581 workspace. |
| [#33](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/33) | `test/phase6-five-bps-sustained-20261122` | `e48bf578e32eecbd40b6db2f7e596899c0c370e7` | [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195), 8/8. Five minutes, 1,500 accepted and matching final tips; 35 authority, 1,579 workspace. |
| [#37](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/37) | `test/phase6-p2p-router-resources-20261122` | `4ae10e0732f53de7752db86a7737ff1eb3db4fb8` | [45105129333](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105129333), 8/8. Two focused P2P queue and cleanup cases; 39 authority, 1,581 workspace. |
| [#38](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/38) | `test/phase6-authenticated-issuance-model-20261122` | `82dfd474f5ce1e31112bc7f7eab00db42db42d21` | [45105132180](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105132180), 8/8. Three issuance commitment, replay and cap model cases; 36 authority, 1,588 workspace. |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `8612ee121513905c830a26cbf700a15b12b3d55d` | [45135461607](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135461607), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `3b02227faec756934e1c140148f5480cd1ef2cfd` | Staged behind #39; focused selected-child body and persistence fixture enabled. |

## Sustained measurement

One isolated transparent simulated-PoW run accepted all 1,500 sequential submissions. First-to-last interval was **299.801365 seconds**. Observed rate over 1,499 intervals was **4.9999772349 BPS**. All three daemons selected tip `1f4fae8828a71efd7b6ae3ea7270853cb212e957683d06dfd6006c68792ac4d6`. Compared with the 50-block baseline, observed rate remained close to 5 BPS and p95 remained near 12.4 ms.

| Peer validation latency | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: |
| First | 6.436 ms | 12.419 ms | 12.625 ms | 12.977 ms |
| Second | 6.412 ms | 12.399 ms | 12.607 ms | 13.024 ms |

[Artifact 14787499841](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195/artifacts/14787499841) contains raw CSV, settings, tip, and aggregate JSON. This shows transparent loopback validation under pacing. CPU/RSS measurements are pending #40. Native PoW, AuxPoW, mixed transactions and network partition load remain separate acceptance conditions.

## Binding and next gates

#41 connects the real selected-child verifier to the test-only issuance checkpoint reopen fixture. It exercises the stored header and selected-parent row, child body Merkle binding and a changed paid/deferred split. The shielded field is a test stand-in. Pruning ancestry trust, a production issuance field, atomic consensus promotion and crash coverage still need proof. Preserve the approved 29B cap and 21B/8B budgets without enabling the schedule.

Review #32 and #34 focused results, then the rest of their matrices. Review #39 for real test-database atomic checkpoint and restart coverage, and trigger the already-staged #41 focused test after #39 validates. Retrieve #40's raw CPU/RSS once complete. #37 was the count-based in-process acceptance gate. Keep byte budgets, node-wide resource measurements and private security evidence in their separate review paths. Keep branches draft and the preview default-off and Simnet-only.
