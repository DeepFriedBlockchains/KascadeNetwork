# Phase 6 relay and testnet evidence checkpoint

November 22, 2026

The archive backfill (#17), three-node partition/rejoin and restart (#19), testnet profile audit (#20), and native PoW relay (#21) each have eight successful jobs on the implementation SHAs in [CURRENT_HANDOFF.md](../../CURRENT_HANDOFF.md). The associated documentation commits preserve the exact tested sources.

| Draft | Branch | Implementation commit | Actions |
| --- | --- | --- | --- |
| [#22 AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `test/phase6-auxpow-relay-20261122` | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439), 7/8. Dedicated native/AuxPoW relay step passed. Native Test Suite is running the workspace build. |
| [#23 5-BPS loopback](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/23) | `docs/phase6-five-bps-evidence-20261122` | `d042358d92a8ab0ae99b549d18dad1c5254533a2` | [45086577123](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086577123), 6/8. Linux release and the native Test Suite remain in progress. |
| [#24 bundled miner](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/24) | `test/phase6-miner-auxpow-interop-20261122` | `436a2ba92f619212576dcae41ca3d73b0a908edc` | [45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062), 3/8. The separately focused bundled miner step passed. The full native suite is running. |
| [#25 combined acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) | `test/phase6-acceptance-convergence-20261122` | `25cfeed643d40b35f906332e4b024dce23632ae0` | Staged #17, #19, and the focused-passing #22 source. Combined CI deferred until #22 finishes. |

PR #22's duplicate push run `45085111236` was canceled. Use the PR run above. PR #23 emits exact Simnet configuration and 50 per-block timing rows. The native job must publish both evidence files. Once it passes, report the achieved submit pace, per-peer propagation percentiles, and the final selected tip on both peers. The 10-second, transparent simulated-PoW sample is one baseline. A sustained run and shielded or real-PoW load require separate evidence.

PR #24 tests the bundled miner’s synthetic parent work and RPC witness round-trip. It also tests cancellation and exposes the node's rejection details. The [miner report](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/436a2ba92f619212576dcae41ca3d73b0a908edc/docs/development/PHASE6_MINER_AUXPOW_2026-11-22.md) is on that branch and defines the live parent and pool evidence still needed.

## Next integration sequence

1. Review native job `137401920040` on #22. Capture the dedicated relay test results and full matrix. If it fails, diagnose the failed step and rerun focused tests before the matrix on the corrected commit.
2. Retrieve timing evidence from #23 job `137406503498` and inspect the two bundled miner tests on #24. Investigate native failures before changing any acceptance assertions.
3. Once #22 finishes, record its result in the #25 convergence report and use a normal push to start combined CI. The staged workflow already selects the archive, partition, and PoW relay cases. Rerun archive recovery, three-node partition/rejoin/restart, native and AuxPoW relay, and the full eight-job matrix on the exact combined source. Record separate evidence for the sustained 5-BPS gate and real parent interoperability.
4. Resolve KASCADE-specific AuxPoW commitment domain and live parent confirmation assumptions before requesting any activation. Keep the Phase 6 preview default-off and limited to Simnet, with shielded consensus activation disabled. Quantum migration research stays in [draft #18](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/18) on the post-launch track.
