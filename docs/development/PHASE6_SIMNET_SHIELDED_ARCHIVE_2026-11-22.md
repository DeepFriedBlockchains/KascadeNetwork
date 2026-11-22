# Phase 6 Simnet shielded archive backfill

November 22, 2026

## Observed behavior

The [PR #16 two-node run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45060976100) logged a valid archive index of 200 rejected at DAA score 199. The first DAA window excludes genesis. The selected-chain index still counts the transition from genesis, so DAA is an invalid upper bound in this case.

## Scope and invariant

In the default-off Simnet Phase 6 preview, a claimed archive index that would shift the local chain index is checked against the anchor's committed GHOSTDAG blue score. Each selected-chain step adds its selected parent to the blue count, so blue score bounds the number of selected-chain steps from genesis. An unavailable local score or a claim beyond it returns an error before any archive write. The default build retains its existing DAA policy and test assertions. The archive remains subject to replay against the authenticated pruning-point frontier before it can advertise complete history.

## Verification

Draft [PR #17](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) is stacked on PR #16 at `c9f378177c0ddd25f3152b55c8d34b593c3cfcc4`. Candidate source is `11ab68a3be26fc9616a91d3d06ac5d99bc23679b`. [Actions run 45069228114](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114) tests this exact source.

The focused two-node scenario mines a shielded Simnet chain and requires its archive index to exceed the anchor's DAA score while remaining within its blue score. The receiving node must import and replay the archive, advertise complete history from DAA 0, and agree on the anchor index. A claim above local blue score must leave the visible completeness and anchor index intact. The existing catch-up and staging-promotion assertions remain active.

## Next testnet acceptance work

Use the validated Simnet base to add restart or reorg coverage that ties archive completeness, selected-chain ancestry, and persisted promotion authority to the same node. Keep shielded consensus activation disabled and the Phase 6 feature default-off and Simnet-only.

Validation is pending. The branch head remains stable while Actions runs.
