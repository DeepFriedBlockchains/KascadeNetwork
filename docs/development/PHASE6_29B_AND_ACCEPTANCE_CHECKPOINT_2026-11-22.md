# Phase 6 acceptance and monetary checkpoint

November 22, 2026

## Validated runtime baselines

| Milestone | Implementation | Actions |
| --- | --- | --- |
| #13 bounded cleanup and persistent reopen | `8084f0e2edff50741e1781bd7a8b2495f7bebd01` | [43523739668](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43523739668), 8/8 |
| #14 descendant reconciliation | `e312306bce7c603a20c0e9251c7e784ab1c7e715` | [45049268981](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45049268981), 8/8 |
| #15 authenticated export snapshots | `6f3466f2e030934ffae9d926a2478fe25e1d7b60` | [45055166221](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45055166221), 8/8 |
| #16 two-node descendant IBD | See exact source in the handoff | [45063994270](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45063994270), 8/8 |

See [CURRENT_HANDOFF.md](../../CURRENT_HANDOFF.md) for the current #17 and #19–#26 sources, checks and run links.

## Monetary design

[Draft #26](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/26) isolates the approved budget documentation and reference vectors from the active Phase 6 branches. It proves the candidate arithmetic subject to unique reward slots. No consensus issuance path is enabled by the draft. A sustained cap requires a proof and production regression suite covering merge rewards, allocation conservation, reorgs, pruning and authenticated snapshot recovery. Confirm the approved curve and reward clock before adding the runtime calculation. Keep the existing miner, KRN, Development Treasury and fallback allocation policy.

## Public source review

- The current public daemon defaults to loopback gRPC and disables unsafe RPC and optional wRPC listeners unless configured. The core service denies privileged operations such as peer administration and shutdown when unsafe mode is disabled. These checks establish the defaults and service gate. Network exposure, per-client limits and any required upstream authentication remain acceptance checks. [Daemon](../../kaspad/src/daemon.rs) · [Arguments](../../kaspad/src/args.rs) · [RPC service](../../rpc/service/src/service.rs)
- The P2P router uses bounded queues and explicit drop or disconnect overflow policy. Independent load tests should record resource bounds for concurrent peers, normal progress under backpressure and recovery after disconnect. Queue controls do not by themselves establish coverage over separate caches, subscriptions and worker resources. [P2P routing](../../protocol/p2p/src/core/router.rs)
- Restart, reorg and shielded coinbase correctness still need to be checked on the combined source. Keep isolated success, rejection and activation evidence distinct. Keep confidential security material out of this repository.

## Execution order

1. Finish review of #22's native workspace tests. If the matrix passes, record validation at the exact source and update #25's convergence report without a CI skip marker to start the combined matrix.
2. Review #23's corrected native job and retrieve the CSV/JSON artifacts. Record the strict evidence upload result and preserve the prior measured 50-block result as short-loopback evidence only.
3. Finish #24 and #26 validation. Ask for a decision on the monetary schedule once the options and implementation dependencies are reviewed. The Phase 6 branches remain separate.

## Validation follow-ups

Draft #26 retains its original public policy documentation lineage and now builds from the validated #21 source and workflow. The initial run failed in the inherited WASM dependency configuration on the documentation-only base. The exact replacement source and run are in the handoff. Pending tests include the focused integer reference vectors, formatting, and all eight jobs.

[Draft #27](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/27) stages a defensive correction to the integration harness's test listeners and a default-deny gRPC case. The production daemon defaults and controls were unchanged. Run the focused RPC test and full CI after capacity is available. Keep #25's combined acceptance scope independent of both new drafts.
