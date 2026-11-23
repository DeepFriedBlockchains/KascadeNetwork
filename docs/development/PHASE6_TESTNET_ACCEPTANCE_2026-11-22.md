# KASCADE testnet acceptance

The [protocol draft](../../KASCADE-SPEC.md#21-required-testnet-coverage) defines coverage. Record the exact commit, configuration digest, network identity, Actions run, and artifacts for each result. Keep the Phase 6 preview default-off and Simnet-only, with shielded consensus activation disabled.

| Area | Current evidence | Acceptance check |
| --- | --- | --- |
| Multi-node sync, partition, and restart | PR #16 validated two-node descendant recovery. PR #17 validated shielded archive backfill on [45069228114](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114/attempts/3). PR #19 validated three-node partition, competing branches, rejoin and persistent DB reopening on [45069680867](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3). | Rerun both #17 and #19 on the same acceptance source. Keep topology and post-reopen commitments tied to that source and exact profile. |
| Native PoW and AuxPoW | PR #21 validated real native kHeavyHash rejection and relay on [45081005178](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45081005178). PR #22 exercises two-node AuxPoW relay with a synthetic parent on [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439). PR #24 exercises the bundled miner. | Validate #22 and carry both mining paths into combined CI. Cover the KASCADE commitment domain, the live parent target and coinbase proof, and mining tool interoperability before activation. |
| Sustained 5 BPS | PR #23 runs a 50-block transparent loopback at a 200 ms interval. The [Actions run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086577123) collects timestamps and configuration. | Collect the baseline. Define agreed acceptance tolerances and repeat at sustained 5 BPS with representative topology and transaction mix. Measure achieved BPS, latency percentiles, memory and CPU, state growth, orphan/reorg rates, and finality progress. |
| Transaction correctness | Existing IBD tests cover shielded commitments, nullifier transfer, and retry. | Exercise T→T, T→Z, Z→Z, and Z→T on isolated Simnet. Check value conservation, expected rejection of invalid transactions, and matching post-reorg commitments on a fresh receiver. |
| Emission and rewards | The approved cap is 29B KCADE, allocated as 21B primary and 8B finite tail. The schedule and runtime enforcement are pending. | Cross-check cumulative issuance and every transition with independent vectors, including rounding, the zero-subsidy boundary, reward-mode persistence, and KCF fallback. |
| KRN and KCF | The draft specifies eligibility and lottery behavior. | Verify qualification, grace, suspension, unbonding, the 300-KRN cap, deterministic payouts, and zero-eligible fallback. |
| Network isolation | README documents local Simnet. Candidate parameters still include inherited and placeholder values. | Verify network ID and magic, genesis, ports, data directories, address prefixes, bootstrap peers, and cross-network rejection. Keep RPC on loopback for CI. |
| Wallet, RPC, explorer and faucet | CI checks the WASM SDK and wallet components. Wallet history coverage is open. | Fund isolated test wallets, spend, restore and rescan against pruned and history-serving nodes. Compare wallet and explorer results against node commitments and issuance. |

## Next executable check

[Draft #25](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) combines the independently validated #17 and #19 branches. It also stages the #22 relay source after its focused step passed. Once #22 validates, trigger and validate the combined candidate on all eight [CI jobs](../../.github/workflows/ci.yaml). Keep public activation gated on separate maintainer authorization.

## Reproducible acceptance evidence

Record these fields in the development report for each completed check.

- Exact source SHA and configuration SHA-256, build features, runner or host profile, and peer topology.
- For each node and checkpoint, the selected tip and its blue score, pruning point, finality point, and the UTXO and shielded state commitments for the same selected block.
- For restart cases, explicit evidence of reopening the same database, checking the durable state before peer reconnect, and accepting fresh blocks after reconnect.
- Focused regression and full GitHub Actions run identifiers with pass counts and expected skips. Distinguish failure, skipped coverage, and checks still in progress.

A change to the implementation or consensus configuration requires rechecking the affected rows. A test tied to Simnet's simulated PoW does not establish real PoW progress, production network isolation, or throughput under public traffic.

See [the source audit](PHASE6_TESTNET_PROFILE_AUDIT_2026-11-22.md) for the inherited settings and candidate-profile checks.

## Phase 6 private testnet gates — November 23, 2026

| Area | Verified status | Gate |
| --- | --- | --- |
| Native-PoW networking | #43 and #48 passed 8/8, including real three-daemon native-PoW and live link-loss recovery. #48 observed reconnect in 105 ms, three-block catch-up in 106 ms and offline reopen in 315 ms. | Representative workload and owner-approved isolated network configuration remain separate. [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224) · [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308) · [Artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308/artifacts/14809571770) |
| Locally replay-authenticated issuance | #49–#51 passed their verifier, canonical claim and RocksDB checkpoint cases. #52/#53 focused settlement and release steps passed on active runs. #54/#55 replay and retention validation continues. | Check exact named tests and completed matrices before relying on local replay or retained provenance. |
| Atomic local pruning readiness | Candidate #56 exercises the IBD authority's first ready batch with a locally replayed issuance pair. Focused run pending. | Verify missing/altered replay, aborted import and RocksDB reopen, then extend to nonzero settlement, interruption and selected replacement through this publisher. [#56 CI](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236408364) |
| Fresh pruned receiver | Existing selected-child evidence proves header/body membership and the shielded root. The cumulative issuance digest is missing from consensus. | Require an independently verified versioned issuance commitment checked during block acceptance. Then verify the proof, checkpoint and readiness together without prior local history. |
| Owner decisions | Production issuance and public testnet remain disabled. | Approve commitment placement, activation and migration, network identity/genesis, bootstrap configuration and the 29B KCADE schedule (21B primary and 8B finite tail) before activation. |

