# Phase 6 reward accounting after restart and restore

November 22, 2026

The current body verifier relates the declared subsidy to DAA score. The validated #30 fixture shows two source blocks at the same DAA score can both be credited when merged into the same selected chain. Each source hash is the distinguishing identity in that fixture. Selected-parent coinbase inclusion is a separate event from crediting the source. Monetary state must distinguish an assigned reward, accepted spendable outputs or notes, and any deferred payable balance.

## Authenticated restoration acceptance

1. Derive the credit ordinal from a reviewed deterministic sequence of eligible unique claims in the accepted chain. Verify qualifying red and blue subsidy paths, non-DAA fee-only claims, genesis exceptions and pending entitlements. Use the net new subsidy for supply accounting and account separately for fees and deferred payouts.
2. Define the canonical state and its consensus commitment before modifying the wire format. The virtual processor atomically stores acceptance records and related state today. Its existing shielded export/import and transparent UTXO import are useful controls. Neither proves an additional reward counter. A snapshot that supplies an ordinal without authenticating it against selected-chain commitments must be rejected.
3. Compare the same tip on a full-history node and a fresh node authenticated from pruning. Change only the imported ordinal while leaving balances and frontier valid and require rejection. Include interrupted import, resumed authenticated sync, branch change, restart and advancement across the cap. Check that no missing or repeated source increases paid or deferred totals. Select activation defaults only after those checks and explicit approval.

## Slot throughput and duration

These estimates use average unique credited slots per second, including all bottlenecks and downtime. They are derived from 1,577,880,000 primary slots and 3,944,700,000 tail slots over a 365.25-day year.

| Unique slots/sec | Primary years | Tail years | Total years |
| ---: | ---: | ---: | ---: |
| 5 | 10.00 | 25.00 | 35.00 |
| 4.5 | 11.11 | 27.78 | 38.89 |
| 4 | 12.50 | 31.25 | 43.75 |
| 3 | 16.67 | 41.67 | 58.33 |

PR #23 measured a short generated-block loopback sample. To support elapsed-year projections, sample unique committed reward claims over a sustained real-mining period, track rollbacks and fees independently, and record the committed ordinal delta against elapsed time. The 29B budget is a slot limit regardless of elapsed time once unique ownership and production enforcement are proven.
