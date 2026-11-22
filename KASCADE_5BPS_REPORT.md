# KASCADE 5-BPS candidate and evidence plan

November 22, 2026. Derived from the current repository's parameter generator. This is an input for isolated testing. All runtime evidence below is pending, and the values are open to revision after measurement.

| Parameter | Unmodified `BlockrateParams::new::<5>()` baseline |
| --- | ---: |
| Target interval | 200 ms |
| GHOSTDAG K | 67 |
| Maximum parents | 16 |
| Mergeset limit | 180 |
| DAA sample rate | 20 blocks |
| Median time sample rate | 50 blocks |
| Merge depth | 18,000 blocks |
| Finality depth | 216,000 blocks |
| Pruning depth | 540,000 blocks |
| Coinbase maturity | 500 blocks |
| Shielded anchor maturity / maximum age | 3,000 / 135,000 blocks |

The local formula's retention lower bound is 300,376 blocks. Its time-derived 30-hour bound dominates at 540,000 blocks. A shielded anchor maximum of 135,000 sits below the 324,000-block pruning-minus-finality margin. These are arithmetic checks on the candidate. The K table assumes a five-second delay bound and a 0.01 Poisson tail threshold. Its assumptions still need evaluating under the expected topology. [Rate generator](consensus/core/src/config/bps.rs) · [Profile constructor](consensus/core/src/config/params.rs) · [Consensus constants](consensus/core/src/config/constants.rs)

Initial DAA and timestamp controls should be recorded alongside the profile. The inherited defaults imply 661 sampled DAA entries, 150 minimum DAA entries, 27 median-time entries and the inherited timestamp-deviation tolerance constant of 132. Record the pre-Crescendo interval and activation state as well, because subsidy arithmetic uses BPS history. When measuring runtime, record timestamp deviation in its interpreted unit and each node's actual configuration hash.

## Acceptance experiments

| Experiment | What to record and check |
| --- | --- |
| Controlled 5-BPS relay | Pace submissions against a monotonic clock at 200 ms. Start with clean isolated nodes and fixed peer topology. Measure offered and accepted blocks per wall-clock second, time from submit to peer validation, backlog slope, orphan rate and convergence. Compare against 1-BPS and 10-BPS controls with the same build and host. |
| DAA and actual work | Run separately with native PoW enabled, a recorded difficulty history and independent miner capacity. Record achieved BPS and target evolution during step changes in hashrate. Profile mining work separately from node processing. |
| Partition and retention | Exercise asymmetric delay and temporary rejoin. Compare accepted chain, UTXO and shielded commitments. Verify finality and prune advancement and stable backlog before and after restart. Short-depth CI fixtures cover behavior at boundaries. Full-depth acceptance requires its own long-run evidence. |
| Cost of block workload | Include empty, representative and upper-budget blocks. Record p50/p95/p99 relay and validation latency, CPU, memory and database growth. Cover block byte/mass/compute budgets, lane constraints and wallet confirmations. Measure shielded workload only under the existing isolated preview. |
| Pool and mixed mining | Record template cadence, stale job rate and the amount of work to keep a 5-BPS target with native and AuxPoW input. Check priority and independent difficulty using induced changes in mining mix. |

The first executable step is a paced three-node loopback baseline at 200 ms with transparent blocks and measured propagation. Save raw per-block times and settings as artifacts. Agree on latency, resource, and sustained acceptance tolerances before measuring a launch candidate. Use the existing [acceptance matrix](docs/development/PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md) for rejoin, restart, wallet and public-profile gates. Keep production activation off.
