# Phase 6 three-node Simnet partition and rejoin

November 22, 2026. Parent PR #16 passed all eight jobs at `c9f378177c0ddd25f3152b55c8d34b593c3cfcc4` in [run 45063994270](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45063994270). This candidate stacks on the documentation-only parent `049ffaf17d3fdc74c93a36b399edb0b4b66cb0b3`.

Three independent daemons use the existing shielded Simnet parameters. The first mines beyond pruning depth. Both receivers perform authenticated IBD. One receiver disconnects while it and the remaining two nodes mine competing branches. After rejoin, the test proves the isolated selected tip was rolled back and the majority gap tip was UTXO-validated. A block relayed from the rejoined node gives all peers a common checkpoint. The test compares the selected tip, pruning point, finality point, shielded commitment, UTXO commitment, and complete virtual UTXO rows for the fixture. It restarts one receiver with its existing data directory, verifies preserved state and promotion, then requires live relay on all three nodes.

The existing CI selection runs `authority_integration_preview_three_node_partition_rejoin` under the Phase 6 preview. The candidate has no production changes. Validation is pending on the exact implementation commit. After a green run, record all eight jobs, native test results and the validated SHA. Keep the stack draft and unmerged.

## Verified three-node Simnet acceptance

All eight jobs passed on implementation `978584bbd91e81b8b637bbebb6655426fe207d9d` in [run 45069680867, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3). Native job 137387238295 ran the three-node test successfully in 26.487 seconds. All 36 Phase 6 cases, 1,579 workspace tests (25 skips), and 32 no-asm tests passed. The case verifies authenticated sync, rollback of the isolated branch, convergence on the same persisted state, and block relay after reopening the receiver.

The [resumed-work handoff](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/90d5672c106a6158f366e080df8f789bded4e961/CURRENT_HANDOFF.md) links the independent profile checks, 5-BPS harness and native PoW follow-on. The PR remains draft and unmerged.
