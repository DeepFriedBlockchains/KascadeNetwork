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

## November 25 operational validation

| Gate | Exact evidence and remaining scope |
| --- | --- |
| Combined settlement, restart, partitions, reorganization and readiness | [#82 45707467797](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707467797) passed 8/8 on head `406535f2` with 43 authority and 1,614 workspace tests. |
| Multi-page pruning retry and reopen | [#91 at `936d89c`](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/91) passed the exact focused case locally (17.50 s), with all 1,302 committed UTXOs authenticated and transferred after interruption, restart and retry. Live reorganization and offline reopen compared equal. [45756192380](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45756192380) remains pending. |
| Actual management cleanup process termination | [#84 45707670756](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707670756) passed 8/8 on `ca6e7c3f`. The same RocksDB was reopened after the two asserted process exits, with consistent staging and preserved active state. Live reorganization and readiness remain separate named checks. |
| Signed transactions at a configured rate | [#83 45707623704](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707623704) passed 8/8. Eight measured simulated-PoW blocks, 16 included signed spends, three agreeing nodes, and 4.99344 submitted BPS across seven intervals in the [recorded artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707623704/artifacts/15067398989). |
| Operational launcher and health | [#85 45707905412](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707905412) passed 8/8. The replacement [#87 run 45755694915](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755694915) must still verify launcher peer topology and persistent restart with real daemon processes. |
| Sustained native operation | [Draft #90 and run 45755839747](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755839747) are in progress for this workload. The next real-process workload must record a paced mining interval, offline follower gap, same-directory reopen and exact post-restart convergence. The short native partition and simulated throughput evidence cover their own scopes. |

## November 25 cross-stack and operational evidence

| Gate | Evidence and scope |
| --- | --- |
| Cross-stack recovery, issuance, relay and readiness | [#76, 45622780578](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622780578) passed all eight jobs. The exact combined native run passed 42 authority and 1,613 workspace tests. Evidence includes real Simnet pruning transfer interruption, reorganization, nonzero guarded exit/retry, local issuance replay and durable failure readiness. |
| Settlement claim conservation | [#77, 45622845484](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622845484) passed all eight jobs and the expanded coinbase/RocksDB regression. Applies to local attributed settlement. Fresh pruned receivers still require approved independent authentication. |
| Staging directory and metadata cleanup | [#79, 45623057634](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45623057634) passed all eight jobs. The legacy partial database state opens and cleans up with the active directory preserved. Live retry/reorganization also passed with the fix. Multi-page retry and actual interruption inside management cleanup are follow-up coverage. |
| Native competing branches | [#80, 45623236895](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45623236895) passed all eight jobs. Three loopback daemons mined competing native branches, rolled back and resumed mining and relay. Representative sustained transaction and resource workload remains incomplete. |
| Operator evidence | [#78, 45622905196](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622905196) passed all eight jobs. New source, daemon and evidence digest tooling also passed local offline fixtures. |

## November 25 exact-source validation

| Gate | Classification | Evidence and remaining work |
| --- | --- | --- |
| Native DAA restart and relay | Verified in native Simnet | [#71 run 45473741398](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473741398), 8/8 at `9d91cf6`. Native competing-branch partition and representative sustained workload remain open. |
| Inactive issuance replay | Verified in local replay | [#72 run 45473757936](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473757936), 8/8 at `396599f`. Fresh-receiver authentication still requires the approved carrier and bootstrap rules. |
| Interrupted pruning import and bounded wire pages | Verified in guarded Simnet | [#73 run 45498114481](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45498114481), 8/8 at `d5cf25c`, has the same source tree as combined `84589bb` and covers #74. The immediate recovery task is atomic abandoned-staging bookkeeping; multi-page retry coverage follows. |
| Durable final evidence rejection | Verified for the RocksDB authority fixture | [#75 run 45473893184](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473893184), 8/8 at `a583b3d`. |
| Production issuance and public testnet | Blocked on owner decisions | The final network profile and issuance authentication, migration and bootstrap trust require approval. |

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

## November 23 extended recovery checkpoint

- Native-PoW three-node and peer-recovery acceptance remain validated on their recorded #43 and #48 heads.
- Local replay verification passed the focused #54 step. Retained-anchor pruning passed the focused #55 step. Full matrices are pending.
- Nonzero atomic pruning publication is in [draft #57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) on the guarded #56 route. Verify both named tests in [45238570956](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45238570956), then require a process-exit test and a consensus-driven branch replacement. The selected-index replacement in #57 verifies stale checkpoint rejection.
- A fresh pruned receiver needs a consensus-validated issuance digest independent of the checkpoint. The [carrier and recovery design](PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md) records the verification boundary and owner decisions. A complete recovery test must begin without a local history prefix and reject missing, substituted or stale evidence.
- Production issuance stays disabled. Before configuring a private testnet, confirm full validation of the guarded publication chain and obtain owner approval for the isolated network profile, issuance activation and bootstrap/migration policy.

## Live recovery validation gates

| Gate | Evidence |
| --- | --- |
| Native-PoW networking | Three-node and peer recovery passed on the recorded #43 and #48 heads. |
| Actual settlement | #52's focused RocksDB settlement step passed. Both release/coinbase tests passed on #53, which is 8/8. |
| Local issuance authentication | #54 replay and #55 selected-index pruning passed their named steps. |
| Atomic pruning publication | #56's zero-issuance guarded test passed. #57's nonzero fixture had a held database handle on reopen and requires successful replacement CI. |
| Process exit | #58 tests subprocess exit before and after the actual nonzero batch. Pending acceptance. Verify the two exit codes and the captured log. |
| Consensus-driven reorganization | #59 tests checkpoint replacement after a branch becomes selected through Simnet consensus. Pending acceptance. Developer accrual and guarded readiness under a live reorganization remain unproven. |
| Fresh pruned receiver | Requires an independently authenticated issuance commitment, validated settlement transitions and approved bootstrap trust. |
| Production activation | Disabled. The owner must approve the network profile, commitment layout, genesis/migration accounting, activation and trust policy. |

See the [fresh-receiver design and decision record](PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md). Complete local validation and verify the nonzero recovery log before proceeding to the first approved private deployment configuration.


## Extended validation checkpoint

| Gate | Evidence and next check |
| --- | --- |
| Native-PoW networking | Three nodes and peer recovery passed on the recorded #43/#48 runs. |
| Actual nonzero settlement | #52 and #53 are 8/8 with named native passes. |
| Local replay and retained state | #54 is 8/8. #55 is 8/8 with named retained-anchor pruning and reopen pass. |
| Guarded atomic publication | #56 focused zero-issuance publication passed. Check both #57 named regressions on [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865). |
| Real process exit and retry | Inspect #58 exit 71 and 72 and #62 retry exit 0. Confirm persisted complete readiness and accounting in their log artifacts. |
| Live reorganization | Confirm #59 actual branch replacement and #61 developer payout/liability persistence. Still need a live pruning-point fixture with proof and UTXO/SMT evidence for guarded readiness and crash/reopen under a real reorg. |
| Fresh pruned receiver | #60 codec is inactive. After owner approval require full-validator replay/migration, authenticated child proof and fresh bootstrap validation. |
| Production activation | Disabled. Owner approval of carrier, fork/activation, bootstrap trust, migration and isolated network profile is required. |


## Live pruning and receiver gate update

- #56 is 8/8 with named guarded-readiness proof. #57's named two-test step passed after the database reopen fix; its full matrix is pending.
- #58 passed the two-exit process test and produced artifact 14853226458. Verify raw exit lines and #62's separate fresh retry in completed native logs.
- #63 exercises real selected-child authentication and eligible pruning depth, with a fixed pruning candidate and live competing descendants. This is a prerequisite for a verified receiver import.
- Before completing the live readiness gate, exercise the real exporter proof and receiver UTXO, SMT and virtual import, then confirm compatible fixed-point recovery and rejection of the stale child after the fork.
- #60 and #64 remain inactive. No network selects the codec or replay verifier.


## Exporter-to-receiver recovery acceptance

| Gate | Evidence and next check |
| --- | --- |
| Actual exporter and isolated receiver | #65 candidate uses the existing P2P import workflow. Require the named native test to pass with a non-genesis proof, selected child, nonempty UTXOs, committed SMT lanes, authenticated empty shielded root and verified virtual state. |
| Guarded publication and reopen | #65 compares the receiver's durable evidence before and after opening the same RocksDB directory, before reconnecting. Also require post-reopen block acceptance. Nonempty Orchard evidence currently has separate coverage. |
| Interrupted accounting publication | #58 and #62 artifacts confirm 71 before commit, 72 after commit before acknowledgement, and 0 for the fresh retry. These cases use seeded chain evidence. A joint real-export crash case remains open. |
| Reorganization at fixed pruning point | #59 verifies live checkpoint replacement. #63 verifies eligibility and stale ancestry for a prospective pruning point. Add a legitimate live fork above finality while keeping the committed point fixed. Test stale immediate-child rejection during a lawful staging or pre-finality transition. Retain the common checkpoint and reject incompatible descendant claims without double issuance. |
| Independent historical issuance | `finish_verified_ibd_import_with_issuance` requires selected replay already validated locally. Production P2P completion supplies no fresh-receiver issuance witness. Require owner-approved carrier, activation, migration and bootstrap trust, then prove nonzero settlement and atomic readiness together, with missing/altered evidence, restart and reorganization. |

## November 24 classified testnet gates

Verified rows apply to the recorded configurations and test scopes. Launch acceptance requires an owner-approved network profile and a combined run on a pinned source and profile.

| Gate | Classification | Evidence and remaining scope |
| --- | --- | --- |
| Deterministic genesis initialization | Verified | Inherited profiles pass `test_genesis_hashes` at `f4d1d4f` in [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319). Approval of the KASCADE-specific genesis and network identity remains blocked below. |
| Multi-node startup | Verified | Three-node CLI native-PoW and catch-up case at `0b89ddc` in [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224). |
| Peer discovery and connectivity | Incomplete | Explicit peer connection and reconnect verified at `9132237` in [45156258308](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308), with [artifact 14809571770](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308/artifacts/14809571770). Prove discovery and bootstrap in the approved topology. [#71](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/71) adds offline difficulty recovery and next-work relay after a real native-PoW retarget. |
| Block creation and propagation | Verified | Native nonce rejection and bidirectional relay at `bd4f658` in [45081005178](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45081005178). The CLI and combined suites also cover native relay. |
| Consensus agreement across nodes | Verified | Competing branches, selected-tip convergence and restart in `authority_integration_preview_three_node_partition_rejoin`, [45069680867 attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3). |
| Difficulty adjustment | Verified | At `79fe770b`, `authority_integration_native_pow_relays_across_difficulty_retarget` passed both selections. Two native-PoW daemons agreed across the sampled retarget and rejected stale difficulty. [45458883372](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883372) passed 8/8. |
| Coinbase payouts | Verified | Independent settlement, developer liability and reorganization in #53 and #61. See [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761) and [45251578576](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45251578576). |
| Total issuance and monetary schedule | Incomplete | Local replay, retained checkpoints and reorg replacement pass #55/#57/#59/#62. An approved complete emission curve, tail horizon, activation, migration and independent historical verification for fresh receivers remain open. |
| Restart and durable recovery | Verified | Same-directory native peer reopen in #48 and guarded RocksDB worker exits in #58/#62, including fresh retry. The [#62 log](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522/artifacts/14854234556) contains exit 0. A real export plus accounting crash case is still open. |
| Reorganization handling | Verified | #19 covers multi-node Simnet. #59/#61 verify actual selected-chain issuance and developer release replacement. The real exporter fixed-point case passed 8/8 at `ee2a81d9` on [#70](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/70) in [run 45459339790](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45459339790). |
| Pruning and retained state | Verified | #55 verifies replay-anchor retention through selected-index pruning. #63 verifies live pruning eligibility and selected-child authentication in [45254033624](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45254033624). The new exporter guard is in #68. |
| Pruning import/export integrity | Verified | Transparent Simnet proof and state comparison passed at `81a8feb6` in [45458788829](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458788829). The production guard and fixed-point reorganization passed together at `bbbee69` in all eight jobs in [45469685474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45469685474). |
| Interrupted live pruning import and retry | Implemented but unverified in full CI | [#73](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/73) interrupts after a staged UTXO commit, checks the active database on reopen, retries and verifies exact state through a fixed-point reorganization. It passed locally, including with #74 on `84589bb`. [Full #73 CI](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473758359) is underway. |
| Interrupted operations | Verified | #58/#62 pass 8/8 and record the guarded write and lost-ack boundaries. #69 verifies a distinct abandoned marker. [#75](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/75) focused coverage verifies readiness stays closed after failed final corroboration and RocksDB reopen. |
| Invalid blocks and malformed messages | Incomplete | Invalid native nonce rejection passes at `bd4f658`. #37 verifies queue-capacity release in [45105129333](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105129333). [#74](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/74) adds a receiver bound for pruning UTXO pages with two passing focused tests for full/empty pages, missing row fields and excess rows. Other malformed-message coverage and the target wire profile remain open. |
| Network partition and reconnection | Verified | #19 passes three-node partition/rejoin. #48 passes native-PoW disconnect, catch-up and reopen with recorded artifacts. |
| Sustained single-workload agreement | Verified | #33 accepted 1,500 simulated-PoW blocks at approximately 5 BPS without tip divergence in [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195), [artifact 14787499841](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195/artifacts/14787499841). Representative sustained native/AuxPoW, mixed traffic and resource thresholds remain incomplete. |
| Fresh pruned-receiver issuance verification | Blocked | Requires a carrier checked during full validation and owner approval of carrier placement, activation/migration and independent bootstrap trust. #64/#67 are inactive validation. |
| Network activation | Blocked | Owner approval of network identity, deterministic genesis, ports, bootstrap peers and activation policy is required. No merge, release or public deployment follows from these scoped results. |
