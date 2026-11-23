# Phase 6 acceptance convergence

November 22, 2026

This draft stages the independently validated archive recovery and three-node partition/rejoin branches. Its source parents are the documentation checkpoints from both drafts. The archive branch owns the consensus, daemon test and native workflow changes. The partition branch adds a disjoint integration test and library module registration. Both source and documentation histories remain reachable.

| Source | Validated implementation | CI |
| --- | --- | --- |
| [#17 shielded archive](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) | `11ab68a3be26fc9616a91d3d06ac5d99bc23679b` | [45069228114, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114/attempts/3), 8/8 |
| [#19 partition/rejoin/restart](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/19) | `978584bbd91e81b8b637bbebb6655426fe207d9d` | [45069680867, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3), 8/8 |

The combined tree is an unvalidated staging checkpoint. Once draft #22 finishes, attach its exact validated relay source. Preserve the archive recovery regression in the native workflow while adding the relay checks. Run the focused archive, three-node partition and both mining relay cases, then all eight Actions jobs. Confirm the combined source uses default-off, Simnet-only Phase 6 preview and keeps shielded consensus activation disabled. Keep this candidate draft and unmerged.
