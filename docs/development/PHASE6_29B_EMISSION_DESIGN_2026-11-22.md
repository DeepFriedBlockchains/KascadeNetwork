# KASCADE 29B issuance design for review

November 22, 2026

## Approved budgets and decisions still open

| Allocation | KCADE | KASMI |
| --- | ---: | ---: |
| Primary | 21,000,000,000 | 2,100,000,000,000,000,000 |
| Finite tail | 8,000,000,000 | 800,000,000,000,000,000 |
| Hard maximum | 29,000,000,000 | 2,900,000,000,000,000,000 |

The public policy branch's draft cascade uses 50 steps descending from 50 to 1 KCADE over 16,470,000 slots each, plus 750,000 slots at 1 KCADE. The arithmetic is exact: 16,470,000 × 1,275 + 750,000 = 21 billion. It takes about 5.22 nominal years at five unique rewarded slots per second. Actual elapsed time depends on the consensus reward clock. The primary shape, start, interval, and phase boundary require approval.

The same branch shows a flat 0.5 KCADE tail over 16 billion slots, about 101.4 nominal years. An alternative family spreads the same 8B budget across an approved number of slots using cumulative division. Examples assume five unique slots per second and 365.25 days per year.

| Tail example | Slots | Per-slot payment range in KASMI |
| --- | ---: | ---: |
| 25 nominal years | 3,944,700,000 | 202,803,762–202,803,763 |
| 50 nominal years | 7,889,400,000 | 101,401,881–101,401,882 |
| 100 nominal years | 15,778,800,000 | 50,700,940–50,700,941 |
| Existing flat example | 16,000,000,000 | 50,000,000 exactly |

For approved budget B and positive slot count N, define cumulative paid at completed slot i as floor(B × min(i, N) / N). The amount for slot i is the difference between adjacent cumulative values. Each delta is a floor or ceiling amount, all deltas telescope to B, and payment stays zero afterward. `u128` safely represents `B × u64::MAX` at the 8B budget. This is a reference model, with no subsidy rule wired to a node.

### Owner decisions

- Approve or revise the primary cascade shape and expected duration.
- Approve the tail shape and duration. The examples above have no selected default.
- Confirm how genesis and any existing issued amounts map to the primary budget, and which event and network activation start reward-slot accounting.
- Approve any consensus-visible change required to score and persist unique rewarded slots. Calendar-year projections depend on this rule.

## Consensus inventory

Reviewed the staged Phase 6 source `25cfeed643d40b35f906332e4b024dce23632ae0` as well as the public policy doc branch. The existing consensus `MAX_SOMPI` in [constants](../../consensus/core/src/constants.rs) already equals 29B × 10⁸ and is used by [transaction-isolation checks](../../consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs) and [UTXO-context checks](../../consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs). Those per-transaction bounds do not enforce cumulative issuance.

| Path | Integration review |
| --- | --- |
| `consensus/src/processes/coinbase.rs` | Inherited scaled monthly schedule, pre-deflationary base and perpetual tail must yield to the approved active profile. Existing Crescendo emission test intentionally excludes the tail. Recheck expected coinbase distribution, shielded balance and developer-fee accrual and payout conservation separately. |
| `consensus/core/src/config/params.rs` and `consensus/core/src/config/bps.rs` | Profile-specific initial amount, rate history, DAA activation and overrides. Ensure the public profile cannot select a schedule implicitly through BPS and old params. Review genesis templates and any allocation separately. |
| `consensus/src/pipeline/body_processor/body_validation_in_context.rs` | Coinbase payload subsidy is checked against the current DAA calculation. Keep construction and validation consistent and update the existing explicit wrong-subsidy vectors once activated. |
| `consensus/src/pipeline/virtual_processor/processor.rs` and `consensus/core/src/coinbase.rs` | Templates and accepted rewards use mergeset reward data and selected-parent state. Audit red and blue ownership, accepted reward order, fee recycling and the developer-accrual checkpoint under a reorg, pruning and restart. |
| Public RPC, wallet and explorer | Supply estimates and remaining issuance need the same semantics after activation. Transaction-size validation can retain the approved per-transaction bound. |
| KRN, miner, Development Treasury and KCF | Keep the existing distribution policy. Prove allocation sums to the exact subsidy and balances carry through reorgs. Do not activate inherited fee changes as a side effect of adjusting issuance. |

## Enforcement gate

A block header's DAA score identifies rate progress. The runtime pays individual accepted mergeset rewards, and concurrent blocks can share rate progress. A per-DAA subsidy table alone is insufficient evidence that each budgeted slot is paid once. Choose and prove unique reward ownership or an authenticated issuance accumulator. If state tracks paid slots and remaining balances, it must be derived from the same ordered accepted history as the rewards, survive snapshot import and pruned reopen, and roll back together with the credited rewards on a reorg. Preserve existing accumulated entitlement without reminting it. Test genesis and premine accounting, repeated or skipped DAA scores, competing DAG tips, blue/red mergesets, duplicate rewards, phase straddles, final zero issuance, near-integer overflow, fee recycling and existing reward splits.

The historical 23B recommendation is superseded by the approved 29B policy. The cumulative-cap finding is open. The tests in this draft validate arithmetic only. Runtime cap validation requires production-path tests and the full Phase 6 matrix after a schedule is approved.
