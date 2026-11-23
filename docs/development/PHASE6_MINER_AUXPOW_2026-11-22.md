# Phase 6 bundled AuxPoW miner checkpoint

November 22, 2026

The bundled `zkas-miner --merged` synthesizes a Kaspa-shaped parent with one coinbase, mines it with the same kHeavyHash implementation used by consensus, and attaches its witness to the KASCADE header sent through gRPC. The focused tests require the child nonce to fail native PoW and the decoded witness to satisfy consensus verification. They also exercise a canceled template. The miner checks cancellation while finding the child nonce, and its rejection logs include the details returned by the node.

## Real parent and pool acceptance

| Area | Evidence needed |
| --- | --- |
| Parent job source | Record a supported parent-node or pool interface and demonstrate authenticated template retrieval or document the trusted operator boundary. Capture job identity, expiry, and replacement behavior. |
| Coinbase | Place the KASCADE commitment in the agreed miner-writable slot. Preserve the parent coinbase outside that slot and verify the full coinbase Merkle branch against the parent header. |
| Targets | Preserve the live parent's compact bits in its header. Check KASCADE acceptance against its child target and test the parent block and pool share thresholds independently. The synthetic fixture copies the child bits for convenience. |
| Dual submission | Exercise a KASCADE-only solution and a solution that also meets the live parent target. Capture independent acceptance responses and stale-job behavior for each endpoint. |
| Protocol | Resolve the domain identifier and treatment of parent-chain confirmation explicitly before any activation. |

This branch tests the bundled synthetic engine. Live parent and pool interoperability, sustained 5-BPS results, and activation approval remain separate gates. The native and cross-target CI matrix passed on the exact source below.

## Validated bundled-miner baseline

[Actions run 45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062) passed all eight jobs at source `436a2ba92f619212576dcae41ca3d73b0a908edc`. [Native job 137407465112](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062/job/137407465112) passed 2 focused miner regressions, 36 Phase 6 authority regressions, 1,581 workspace tests (25 skipped) and 32 no-asm tests. The focused tests cover a canceled template and a synthetic witness that survives gRPC serialization.

## Upstream Kaspa pool boundary

Pin the parent daemon and reference Stratum bridge to Rusty Kaspa v2.1.0 (`01b532e8b553523216471682649693af92f0fd16`) for the first interoperability experiment. The upstream public `GetBlockTemplate` accepts a payout address and UTF-8 `extraData` and returns a complete candidate block and a sync indicator. `SubmitBlock` reports parent-node acceptance separately. The upstream bridge supports external and in-process node modes. [Release](https://github.com/kaspanet/rusty-kaspa/releases/tag/v2.1.0) · [Pinned gRPC schema](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/rpc/grpc/core/proto/rpc.proto) · [Pinned bridge guide](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/bridge/docs/README.md)

The stock bridge captures a server-wide coinbase tag at startup and passes that same tag on template requests. Its configurable suffix is truncated to 64 allowed ASCII bytes. The current KASCADE candidate needs 68 bytes for the four-byte `ZKMM` marker and the 64-character child hash, and that hash changes with the child template. Stock suffix configuration therefore cannot carry a complete per-job KASCADE commitment. The pinned bridge source and the live parent payload limit are separate constraints. [Pinned template client](https://github.com/kaspanet/rusty-kaspa/blob/01b532e8b553523216471682649693af92f0fd16/bridge/src/kaspaapi.rs) · [KASCADE witness format](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/436a2ba92f619212576dcae41ca3d73b0a908edc/consensus/core/src/auxpow.rs)

Use direct node gRPC to establish the parent-template baseline. A separate pool adapter or bridge extension must supply the changing commitment at job creation and retain the complete parent template and child identity until their job expires. A miner-facing Stratum share response establishes that a pool target was met, while the two node-submit responses establish the parent and child outcomes independently.

## Isolated acceptance cases

Use only loopback Simnet listeners, synthetic payout addresses and disposable data directories. Pin both implementations and save the exact chain/config digests and source SHAs with every result. Use the parent network's own transaction Merkle algorithm and require at least one non-coinbase transaction in the parent fixture.

| Case | Required evidence |
| --- | --- |
| Stock configuration | Verify the pinned bridge produces its documented static tag and that the 68-byte candidate cannot fit the configured suffix. Save returned payload length and template identity. |
| Direct parent template | Request a template bound to the child header and verify the complete commitment survives in a single occurrence. Confirm the coinbase leaf and nonempty branch reconstruct the same parent header Merkle root. Record fee transactions and payload byte counts without relying on the synthetic fixture's single-leaf shortcut. |
| Independent target routing | Build fixtures with deliberately distinguishable share, child and parent thresholds. Record expected and observed acceptance for a share below network requirements, a child-only solution, and a dual solution. Keep the original parent bits and submit independently to each node only where its own target and job checks pass. |
| Job refresh and duplicate shares | Rotate the child after a known block, then independently rotate the parent. A late response from an expired pair stays associated with its original job and cannot be submitted as the new pair. Verify a duplicate share does not trigger repeated node submissions. |
| Supported pool boundary | Record the exact pool adapter version, credential boundary, job refresh event and stale-share decision. Repeat the direct-node evidence end to end through the pool adapter and retain pool and both node results. |

The direct parent fixture and pool adapter are still unimplemented. After the direct fixture passes, use its exact parent/child job pair and target cases as adapter fixtures. Activation also requires the project-specific commitment domain decision and the separate parent-chain confirmation policy decision.
