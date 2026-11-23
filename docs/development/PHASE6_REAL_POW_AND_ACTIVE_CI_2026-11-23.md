# Phase 6 real-PoW staging and CI checkpoint

November 23, 2026

| Draft | Exact source | Actions status |
| --- | --- | --- |
| [#39 Atomic checkpoint](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) 6/8. Native 137563153013 before the focused model. |
| [#41 Selected-child fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged. Trigger after #39 passes. |
| [#32 Genesis conservation](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 7/8. Focused test passed. |
| [#34 Red rewards and deferred payout](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 7/8. Both the fee/subsidy builder and real Simnet eligible-red steps passed. |
| [#40 CPU/RSS](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 7/8. Five-minute native measurement running, evidence upload pending. |
| [#42 CLI runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320) 3/8. Native 137568406534 is executing the dedicated CLI step, evidence upload pending. |
| [#43 Real native PoW CLI](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `e2b86501f47313daa34c3592d458ae8793482ae7` | Staged on `test/phase6-cli-real-pow-recovery-20261123` behind #42. No CI yet. |

#43 adds a separate easy native PoW case with three daemon processes, validates rejection of an above-target nonce, mines and relays twelve blocks, and requires a follower to catch up by exactly six blocks after reopening. It records the final DAG tip, selected-parent agreement checks, per-node CPU/RSS, disk I/O and storage size, and P2P byte counters. This fixture uses fresh Simnet directories and explicit test-only parameters; production consensus is unchanged. [Exact scope](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/e2b86501f47313daa34c3592d458ae8793482ae7/docs/development/PHASE6_CLI_REAL_POW_RECOVERY_2026-11-23.md)

Keep #39 and #42 stable under test. When #39 is green, trigger #41's focused selected-child check before the broad matrix. When #42 is green, collect the CLI artifact, preserve its tested source, and trigger #43. Fetch #40's five-minute artifact when it exists. Public or private activation still requires authenticated pruning restore tied to settlement, representative mining/load evidence and approved network parameters. Preserve the inactive 29B KCADE schedule and the 21B/8B allocations.
