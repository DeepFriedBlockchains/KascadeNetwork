# KASCADE 5-BPS candidate and evidence plan

November 22, 2026. Derived from the current repository's parameter generator. This is an input for isolated testing. The profile remains open to revision after measurement.

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

The [opt-in loopback harness](testing/integration/tests/phase6_five_bps_baseline.rs) paces a transparent three-node workload and prints per-block submit and observed peer-validation times as CSV. Run it in GitHub Actions with `cargo nextest run -p kaspa-testing-integration --test phase6_five_bps_baseline --features phase6-staging-preview --run-ignored all --success-output immediate`. The harness exercises simulated PoW for the short relay sample. Sustained-rate acceptance is open.

## 5-BPS measurement candidate

The existing opt-in harness now has a branch-scoped Actions step and fails unless it emits a configuration file and all 50 samples. It separately waits for all three nodes to select the final tip. This measures short, transparent Simnet relay with simulated PoW. Sustained throughput and production mining require separate experiments.

## November 22, 2026 · First measured loopback

The native test passed in 14.822 seconds on [Actions run 45086577123](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086577123), job `137406503498`, source `d042358d92a8ab0ae99b549d18dad1c5254533a2`. All 50 submissions were accepted and both peers and the sender selected the final tip. The first-to-last submit window was 9.800693 seconds, or **4.9996 blocks per second** over 49 intervals. The final selected hash was `3933a5188c0b7bf773b698ac5fdae11ecfb80f22e52fafdbca2f40d65c81097e`.

| Peer | p50 | p95 | p99 and max |
| --- | ---: | ---: | ---: |
| First | 6.294 ms | 12.088 ms | 12.603 ms |
| Second | 6.289 ms | 11.829 ms | 12.091 ms |

These are nearest-rank percentiles of submit-to-first-observed UTXO-valid latency in a single 50-block, transparent Simnet sample. The fixture used simulated PoW and polled every 5 ms. It does not establish sustained throughput or live mining performance.

The native job failed during CSV/JSON extraction because nextest indents captured stdout. The workflow now strips leading horizontal whitespace while requiring all 50 rows and exactly one configuration record. The replacement run below verified artifact publication and the remainder of the native job. The original samples and config remain in the linked job log.

## Validated replacement run

All eight jobs passed on implementation commit `999f3e1342d82c1feebfeb47d55023807be17276`. The [native job](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439/job/137416068338) passed the 50-block loopback in 14.835 seconds, 35 targeted regressions, 1,579 workspace tests with 25 skipped, and 32 no-asm tests.

The first-to-last submit window was 9.800571 seconds across 49 intervals, or **4.9997 blocks per second**. All three nodes selected final tip `86e67be18961bd5fceb8e6e8540ef35b5e21be1acff868b09663735ace5a31bb`.

| Peer | p50 | p95 | p99 and max |
| --- | ---: | ---: | ---: |
| First | 6.641 ms | 12.410 ms | 12.589 ms |
| Second | 6.598 ms | 12.325 ms | 12.656 ms |

The workflow verified a 51-row CSV with one header and all 50 samples, and a single profile record. [Evidence artifact 14781544000](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439/artifacts/14781544000) has archive SHA-256 `0ad608775c671c450f83286c88630c64c826bcb3bdd017a456ecdd2c7f980216`. The [full CI run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439) validates this short sample and its collector. Sustained 5-BPS operation and live mining remain acceptance experiments.

## Five-minute loopback acceptance candidate

Drafted November 22, 2026. The branch-scoped native job measures 1,500 transparent blocks under the existing 200 ms, simulated-PoW, three-node Simnet fixture. It requires each submit RPC to accept and both peers to reach UTXO-valid for every block. All three nodes must select the final tip. CI checks contiguous indexes, unique hashes, increasing submit offsets, the requested profile and matching final tip. It requires 4.95–5.05 accepted blocks per second over 1,499 measured intervals. That tolerance applies to this controlled runner fixture.

The artifact includes the exact JSON profile override, CSV timings, final tip and measured span and nearest-rank latency quantiles. The native job produces raw evidence for an approximately five-minute run. Real PoW, loaded blocks, long-window finality and pruning, and a multi-host simulation remain separate testnet checks.

Run the same fixture in isolated GitHub Actions with `cargo nextest run -p kaspa-testing-integration --test phase6_five_bps_baseline --features phase6-staging-preview --run-ignored all --success-output immediate -E 'test(=phase6_five_bps_loopback_five_minutes)'`. The 50-block case has an exact-name filter as well and retains its existing raw-evidence checks.
