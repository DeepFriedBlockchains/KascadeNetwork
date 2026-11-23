# KASCADE Reward Node economics under the 35-year candidate

November 22, 2026

The 35-year candidate budgets 21B KCADE over 1,577,880,000 primary rewarded units and 8B over 3,944,700,000 tail units. Both periods assume five unique rewarded units per second and 365.25 days per nominal year. The model uses the continuous reference curve for projections. Its primary reward starts at 23.617993763784 KCADE, reaches 3 KCADE at year 10, then declines to 1.056075240196 KCADE just before subsidy ends. The annualized token figures describe the subsidy and the 30,000 KCADE bond at each point on the curve. Costs, price, actual eligibility, and the consensus definition of a uniquely rewarded unit affect outcomes.

## Payout and eligibility rules

The public specification gives units 1 through 1,500 a 90% miner and 10% Development split, even if a KRN qualifies sooner. After unit 1,500, the same split continues until the first reward with a verified eligible KRN. That event enters Standard Mode permanently. In Standard Mode the target is 90% miner, 8% to one uniformly selected eligible KRN and 2% Development. If the eligible set is empty, the 8% goes to the KASCADE Community Fund (KCF). Returning KRNs resume lottery rewards upon eligibility. Transaction fees initially go entirely to miners, including after subsidy ends. [Reward distribution and KCF](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/KASCADE-SPEC.md#5-reward-distribution)

An eligible KRN has one ticket and a unique 30,000 KCADE bond. The maximum active set of 300 bonds locks 9,000,000 KCADE without transferring ownership. Registration and 72 continuous hours of qualifying service precede eligibility. The spec targets checks about every 30 minutes, one hour of grace before reward suspension, and deactivation after 24 continuous hours offline. Return requires requalification for 72 hours. Downtime does not slash collateral. A voluntary exit follows the 14-day minimum commitment and seven-day unbond. A node missing eligibility has no lottery ticket for that unit under the eventual accepted eligibility snapshot. [Eligibility and deterministic lottery](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/KASCADE-SPEC.md#8-kascade-reward-nodes-krn)

The spec treats all payouts as subdivisions of the same subsidy. For a subsidy of `S` Kazmi, in Standard Mode calculate `dev = floor(2S/100)`, `krn_or_kcf = floor(8S/100)`, and `miner = S - dev - krn_or_kcf`. Bootstrap must conserve `S` as well. A reference implementation can assign `floor(10S/100)` to Development and the remainder to the miner, following the Standard Mode rounding convention. The current code reviewed at `51b747ae11b11b636612245b756135a564dde4e4` uses the inherited configurable developer levy, with a mainnet test expecting 50 permille. These files do not establish an active KRN registry, lottery or KCF payout system. Allocation behavior still requires implementation and acceptance tests. [Specification](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/KASCADE-SPEC.md#5-reward-distribution) · [Coinbase source](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/consensus/src/processes/coinbase.rs) · [Denomination](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/consensus/core/src/constants.rs)

## Expected payout by participation

The projections assume a constant, fully eligible set and Standard Mode throughout each 30-day run-rate window. They hold the subsidy at the milestone rate so different node counts can be compared directly. A forward forecast would accumulate the actual per-unit integer payouts over the window. Each entry is expected **KCADE per day / KCADE per 30 days per eligible KRN** at five unique rewarded units per second. At and after the terminal cutoff, all modeled subsidy earnings are zero for every participation level.

| Active KRNs | Year 5 | Year 10 | Year 15 | Year 20 | Year 25 | Year 30 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 459,958.9 / 13,798,768 | 103,680.0 / 3,110,400 | 90,243.6 / 2,707,308 | 76,807.2 / 2,304,216 | 63,370.8 / 1,901,123 | 49,934.4 / 1,498,031 |
| 5 | 91,991.8 / 2,759,754 | 20,736.0 / 622,080 | 18,048.7 / 541,462 | 15,361.4 / 460,843 | 12,674.2 / 380,225 | 9,986.9 / 299,606 |
| 10 | 45,995.9 / 1,379,877 | 10,368.0 / 311,040 | 9,024.4 / 270,731 | 7,680.7 / 230,422 | 6,337.1 / 190,112 | 4,993.4 / 149,803 |
| 25 | 18,398.4 / 551,951 | 4,147.2 / 124,416 | 3,609.7 / 108,292 | 3,072.3 / 92,169 | 2,534.8 / 76,045 | 1,997.4 / 59,921 |
| 50 | 9,199.2 / 275,975 | 2,073.6 / 62,208 | 1,804.9 / 54,146 | 1,536.1 / 46,084 | 1,267.4 / 38,022 | 998.7 / 29,961 |
| 100 | 4,599.6 / 137,988 | 1,036.8 / 31,104 | 902.4 / 27,073 | 768.1 / 23,042 | 633.7 / 19,011 | 499.3 / 14,980 |
| 300 | 1,533.2 / 45,996 | 345.6 / 10,368 | 300.8 / 9,024 | 256.0 / 7,681 | 211.2 / 6,337 | 166.4 / 4,993 |

Network-wide gross subsidy flows before integer rounding and fee income follow the same curve. The 8% column is paid to one KRN per unit when eligible. It goes to KCF when Standard Mode has begun and the eligible set is empty.

| Nominal year | Subsidy per paid unit | Network subsidy/day | Miner share/day | KRN/KCF share/day | Developer share/day |
| --- | ---: | ---: | ---: | ---: | ---: |
| 5 | 13.308997 | 5,749,487 | 5,174,538 | 459,959 | 114,990 |
| 10 | 3.000000 | 1,296,000 | 1,166,400 | 103,680 | 25,920 |
| 15 | 2.611215 | 1,128,045 | 1,015,240 | 90,244 | 22,561 |
| 20 | 2.222430 | 960,090 | 864,081 | 76,807 | 19,202 |
| 25 | 1.833645 | 792,135 | 712,921 | 63,371 | 15,843 |
| 30 | 1.444860 | 624,180 | 561,762 | 49,934 | 12,484 |
| 35 | 0.000000 | 0 | 0 | 0 | 0 |

## Lottery variance

For `n` continuously eligible nodes, the modeled win probability per rewarded unit is `p = 1/n`. For `u` unique rewarded units with constant KRN payment `k`, independent uniform selection gives expected wins `up`, win-count variance `up(1-p)`, expected gross payout `kup`, and payout variance `k²up(1-p)`. When unit subsidies vary, the corresponding terms are `p × Σ kᵢ` and `p(1-p) × Σ kᵢ²` for a fixed independent eligible set. The implementation target is an unbiased deterministic winner. Consecutive winners may be correlated if entropy or eligibility depends on related DAG history. The estimates below isolate lottery variance under independence.

| Active KRNs | Wins/day | Wins/30 days | SD wins/day | SD wins/30 days | 30-day CV |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 432,000 | 12,960,000 | 0.0 | 0.0 | 0.000% |
| 5 | 86,400 | 2,592,000 | 262.9 | 1,440.0 | 0.056% |
| 10 | 43,200 | 1,296,000 | 197.2 | 1,080.0 | 0.083% |
| 25 | 17,280 | 518,400 | 128.8 | 705.5 | 0.136% |
| 50 | 8,640 | 259,200 | 92.0 | 504.0 | 0.194% |
| 100 | 4,320 | 129,600 | 65.4 | 358.2 | 0.276% |
| 300 | 1,440 | 43,200 | 37.9 | 207.5 | 0.480% |

Multiply each win-count SD by the per-win payment of approximately `0.08 × subsidy` for payout SD in KCADE. At 300 eligible nodes and 5 unique units/s, year 30 gives 4,993.44 KCADE per 30 days, with a modeled SD of 23.98 KCADE. Reward decline, downtime, changes in eligibility and changes in unit throughput can move realized payouts more than this lottery SD. At 300 participants an operator eligible for 95% of the window expects about 95% of the full-time gross if subsidy and eligible-set size stay steady. Qualification and requalification delays can make eligibility time much lower than uptime.

## Operating costs and break-even

These are scenario inputs in USD. The low-cost case uses $15 per month for incremental power and network share on existing equipment, the middle case uses $40 for a small dedicated deployment, and the higher-cost case uses $100 for higher availability or hosting. Actual costs must include hardware amortization, power, bandwidth, administration and any paid redundancy. The examples exclude tax, payout-conversion fees, and collateral opportunity cost unless stated. Break-even token price equals monthly cost divided by expected 30-day token earnings. These thresholds are model outputs.

At $40 per month, gross operating-cost break-even by participation is as follows, in USD/KCADE.

| Active KRNs | Year 5 | Year 10 | Year 15 | Year 20 | Year 25 | Year 30 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | $0.000003 | $0.000013 | $0.000015 | $0.000017 | $0.000021 | $0.000027 |
| 5 | $0.000014 | $0.000064 | $0.000074 | $0.000087 | $0.000105 | $0.000134 |
| 10 | $0.000029 | $0.000129 | $0.000148 | $0.000174 | $0.000210 | $0.000267 |
| 25 | $0.000072 | $0.000322 | $0.000369 | $0.000434 | $0.000526 | $0.000668 |
| 50 | $0.000145 | $0.000643 | $0.000739 | $0.000868 | $0.001052 | $0.001335 |
| 100 | $0.000290 | $0.001286 | $0.001477 | $0.001736 | $0.002104 | $0.002670 |
| 300 | $0.000870 | $0.003858 | $0.004432 | $0.005208 | $0.006312 | $0.008011 |

Full-participation example with 300 active nodes, at 5 unique units/s. Annualized gross divided by the bond uses `365.25 × expected daily KCADE / 30,000`, held at each checkpoint rate. It measures token earnings relative to locked tokens, before costs and downtime. It is a run-rate comparison and does not compound or forecast actual yearly payouts.

| Year | KCADE per unit | KCADE per 30 days | 30-day payout SD | Annualized gross / bond | $15/mo | $40/mo | $100/mo |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 13.308997 | 45,995.9 | 220.9 | 1,866.7% | $0.000326 | $0.000870 | $0.002174 |
| 10 | 3.000000 | 10,368.0 | 49.8 | 420.8% | $0.001447 | $0.003858 | $0.009645 |
| 15 | 2.611215 | 9,024.4 | 43.3 | 366.2% | $0.001662 | $0.004432 | $0.011081 |
| 20 | 2.222430 | 7,680.7 | 36.9 | 311.7% | $0.001953 | $0.005208 | $0.013020 |
| 25 | 1.833645 | 6,337.1 | 30.4 | 257.2% | $0.002367 | $0.006312 | $0.015780 |
| 30 | 1.444860 | 4,993.4 | 24.0 | 202.7% | $0.003004 | $0.008011 | $0.020026 |

For a target annual collateral opportunity rate `r`, the 30-day bond cost in USD is `30,000 × token_price × r × 30/365.25`. The monthly operating-cost threshold becomes `cost / (expected_30_day_KCADE − 30,000 × r × 30/365.25)` when the denominator is positive. At year 30 and 300 participants, a hypothetical 5% annual opportunity rate raises the $40 threshold from about $0.008011 to $0.008213 USD/KCADE. After the cutoff, subsidy income alone cannot cover a positive operating or collateral cost at any token price.

### Throughput sensitivity

At a fixed year-30 milestone **slot index**, 300-node operation yields these approximate run rates. Low or high sustained throughput changes elapsed time to each boundary while fixed reward-unit budgets preserve total issuance. The final column assumes the indicated rate persists for the entire schedule. If consensus targets 35 elapsed years even with variable accepted throughput, deriving a different clock requires separate approval and validation.

| Unique rewarded units/s | KCADE/day | KCADE/30 days | 30-day payout SD | $40/mo break-even | Years to exhaust same slots |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2.5 | 83.2 | 2,496.7 | 17.0 | $0.016021 | 70.00 |
| 5 | 166.4 | 4,993.4 | 24.0 | $0.008011 | 35.00 |
| 6 | 199.7 | 5,992.1 | 26.3 | $0.006675 | 29.17 |

The companion [scenario CSV](PHASE6_KRN_ECONOMICS_35_YEAR_2026-11-22.csv) records expected wins, daily and 30-day earnings, both payout variances and SDs, collateral-denominated gross return and all three cost thresholds across all seven participation levels, milestone years and throughput scenarios. The price cells are empty for zero subsidy.

## Terminal cutoff and remaining decisions

The last marginal rewarded unit has a positive subsidy under the candidate curve. Then subsidy becomes zero. At five units/s, the reference curve immediately before the cutoff pays about 456,225 KCADE in total network subsidy per day, or about 410,602 KCADE to miners and 36,498 KCADE to KRNs or KCF. At full participation this represents about 3,650 KCADE per KRN per 30 days just before the boundary. A declining schedule therefore still has a discontinuity at expiry. Its financial significance depends on the market and network operating costs at that time.

Under the documented fee rule, miners retain fees after expiry and KRNs, the Development Treasury and KCF lose subsidy. A transition planning review needs to quantify sustained transaction fee revenue relative to hashpower and merge-mining economics, and estimate minimum KRN coverage needed for the actual service. Potential KRN incentives include separately funded service contracts or a fee share approved under an explicit conservation rule. Neither changes the issuance cap. Review contingency governance and failure modes well before the cutoff, with any consensus-visible change treated as a separately approved upgrade.

Before the cutoff, generous token-denominated rewards can coexist with unprofitable operation at a low enough price. The cap also creates a concentration risk: if one operator controls `h` independently eligible identities, the equal-ticket model gives that operator `h/n` of the 8% allocation in expectation. Multiple physical or virtual identities under one operator therefore need concentration and independence review. At most 2.32B KCADE can ever be assigned to the 8% subsidy slice across the entire issuance schedule. Bootstrap, integer rounding and any KCF fallbacks lower the amount actually paid to KRNs.

Before KRN activation, define and test the unique reward index, canonical eligible-set ordering, mode transition and reorg rollback, service verification and grace boundaries, unbiased entropy and replay behavior, rounding and the zero-subsidy terminal event. Deterministic tests should cover exact units 1,500 and 1,501, delayed first activation, eligible set recovery after fallback, offline and requalification boundaries, 300 members, duplicate collateral and no payout after expiry. Compare simulated lottery outcomes with reference variance and run concentrated-ownership and downtime scenarios under actual eligibility and payout rules.
