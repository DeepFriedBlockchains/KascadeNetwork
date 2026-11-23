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

This branch tests the bundled synthetic engine. Live parent and pool interoperability, sustained 5-BPS results, and activation approval remain separate gates. CI is pending.
