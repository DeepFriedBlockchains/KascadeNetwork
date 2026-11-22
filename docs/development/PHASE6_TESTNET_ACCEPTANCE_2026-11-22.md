# KASCADE testnet acceptance

The [protocol draft](../../KASCADE-SPEC.md#21-required-testnet-coverage) defines coverage. Record the exact commit, configuration digest, network identity, Actions run, and artifacts for each result. Keep the Phase 6 preview default-off and Simnet-only, with shielded consensus activation disabled.

| Area | Current evidence | Acceptance check |
| --- | --- | --- |
| Multi-node sync, partition, and rejoin | PRs #14 and #15 validated descendant reconciliation and authenticated export. PR #16's two-node recovery test passed both alone and in the Phase 6 suite on [run 45063994270](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45063994270). | Run three isolated nodes. Form competing branches during a partition, reconnect, then compare selected tip, pruning point, UTXO commitment, and shielded commitment before and after reopening. Retain topology and per-node checkpoints as artifacts. |
| Native PoW and AuxPoW | Simnet daemon fixtures exercise templates and block submission. | In an isolated PoW-enabled profile, verify valid native and AuxPoW blocks converge across nodes. Cover parent/domain rejection, difficulty, and native progress without AuxPoW. |
| Sustained 5 BPS | The shielded IBD fixture runs a bounded workload with a 100 ms target. | Exercise a 200 ms candidate with representative topology and transaction mix. Establish pass thresholds for observed BPS, propagation and verification latency, memory and CPU, state growth, orphan/reorg rates, and finality progress before running the acceptance benchmark. |
| Transaction correctness | Existing IBD tests cover shielded commitments, nullifier transfer, and retry. | Exercise T→T, T→Z, Z→Z, and Z→T on isolated Simnet. Check value conservation, expected rejection of invalid transactions, and matching post-reorg commitments on a fresh receiver. |
| Emission and rewards | The draft specifies a 23B KCADE integer hard cap and reward modes. | Cross-check cumulative issuance and every transition with independent vectors, including rounding, the zero-subsidy boundary, reward-mode persistence, and KCF fallback. |
| KRN and KCF | The draft specifies eligibility and lottery behavior. | Verify qualification, grace, suspension, unbonding, the 300-KRN cap, deterministic payouts, and zero-eligible fallback. |
| Network isolation | README documents local Simnet. Candidate parameters still include inherited and placeholder values. | Verify network ID and magic, genesis, ports, data directories, address prefixes, bootstrap peers, and cross-network rejection. Keep RPC on loopback for CI. |
| Wallet, RPC, explorer and faucet | CI checks the WASM SDK and wallet components. Wallet history coverage is open. | Fund isolated test wallets, spend, restore and rescan against pruned and history-serving nodes. Compare wallet and explorer results against node commitments and issuance. |

## Next executable check

Use the existing [daemon fixture](../../testing/integration/src/daemon_integration_tests.rs) to add the three-node competing-branch/rejoin case under `phase6-staging-preview`. Compare the committed pruning and shielded views after reconnect and restart. Run the focused case and all eight [CI jobs](../../.github/workflows/ci.yaml). Keep public activation gated on separate maintainer authorization.

## Reproducible acceptance evidence

Record these fields in the development report for each completed check.

- Exact source SHA and configuration SHA-256, build features, runner or host profile, and peer topology.
- For each node and checkpoint, the selected tip and its blue score, pruning point, finality point, and the UTXO and shielded state commitments for the same selected block.
- For restart cases, explicit evidence of reopening the same database, checking the durable state before peer reconnect, and accepting fresh blocks after reconnect.
- Focused regression and full GitHub Actions run identifiers with pass counts and expected skips. Distinguish failure, skipped coverage, and checks still in progress.

A change to the implementation or consensus configuration requires rechecking the affected rows. A test tied to Simnet's simulated PoW does not establish real PoW progress, production network isolation, or throughput under public traffic.
