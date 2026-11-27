# KASCADE Phase 6 loopback operator runbook

November 23, 2026

Use a dedicated host or runner and one verified binary for all nodes. Record its exact source SHA and file hash. [The combined Simnet fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286) was validated at `c542833e385f1f222538f42bc72a9843a1cd2ca4`. The later parent of this runbook adds evidence only. This manual procedure is source-reviewed. Its equivalent automated partition/rejoin fixture passed, and this particular shell procedure has yet to be exercised.

## Local startup

Use fresh, separate data directories with no inherited process configuration for another network. These commands start an isolated loopback topology. Build or obtain the binary from your vetted checkout and set its absolute path as `KASPAD` in each terminal. Keep the preview and RPC permission defaults.

```bash
# Terminal A
"$KASPAD" --simnet --appdir=./phase6-node-a --listen=127.0.0.1:19111 \
  --rpclisten=127.0.0.1:19110 --nodnsseed --disable-upnp --outpeers=0
```

```bash
# Terminal B
"$KASPAD" --simnet --appdir=./phase6-node-b --listen=127.0.0.1:19211 \
  --rpclisten=127.0.0.1:19210 --nodnsseed --disable-upnp --connect=127.0.0.1:19111
```

```bash
# Terminal C
"$KASPAD" --simnet --appdir=./phase6-node-c --listen=127.0.0.1:19311 \
  --rpclisten=127.0.0.1:19310 --nodnsseed --disable-upnp --connect=127.0.0.1:19111
```

Use the corresponding loopback gRPC port for each node. Query `GetCurrentNetwork`, `GetBlockDagInfo`, and `GetConnectedPeerInfo` and record the network, selected sink, pruning point, and peer topology. Before submitting blocks, verify the local genesis agrees with `SIMNET_GENESIS` at the pinned source. A fresh data directory's selected tip must agree on all nodes. [CLI options](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/15f99ca97799b414e57fb20b8f53f1d85feb1f6c/kaspad/src/args.rs) · [Genesis definitions](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/15f99ca97799b414e57fb20b8f53f1d85feb1f6c/consensus/core/src/config/genesis.rs)

## Restart and convergence evidence

1. Save network, selected tip, pruning point, and peer state before stopping a follower. Stop it cleanly with Ctrl-C and preserve its data directory.
2. Run the identical follower command again. Confirm recovery on the same network and with its expected connection. Record the current selected tip and pruning point on every node.
3. Use the existing three-node partition/rejoin integration fixture for competing blocks, shielded state comparisons, and post-restart block advancement. It drives those checks automatically. The ordinary manual startup commands do not enable test mining or unsafe RPC. [Partition and persistent reopen fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/15f99ca97799b414e57fb20b8f53f1d85feb1f6c/testing/integration/src/phase6_partition_rejoin.rs) · [RPC safe-mode evidence](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093817955)

## Separate acceptance gates

The [five-minute timing evidence](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195/artifacts/14787499841) records 1,500 accepted transparent Simnet submissions at approximately 5 BPS with simulated PoW and matching final tips. Record CPU/RSS from the resource-sampling run separately. Require real native/AuxPoW relay evidence and representative load and disconnect testing before claiming mining or public readiness.

Public testnet startup still needs owner-approved network identity, genesis, bootstrap endpoints, expected protocol and RPC exposure, observable peer and byte budgets, and authenticated production issuance and recovery. Keep this runbook's loopback configuration scoped to Simnet. The 29B issuance candidate remains inactive.

## Automated command exercise

The Linux Actions fixture starts the three documented commands with temporary paths and the same loopback ports. It checks RPC network identity, common fresh genesis, and the explicit peer topology. In a second pass, it restarts A with the additional test-only `--enable-unsynced-mining` flag. Default Simnet simulated PoW is used to advance two blocks, one while C is stopped. C reopens the same path and must catch up to the new tip and match A's pruning point and virtual DAA. All processes must exit successfully after SIGINT. CI uploads the exact commands, test driver observations, and per-process logs. The simulated-PoW exercise passed on the exact #42 source in [Actions run 45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320).

## Native-PoW acceptance on isolated Simnet

Create one absolute-path override file visible to all three processes:

```bash
printf '%s\n' '{"skip_proof_of_work":false,"shielded_coinbase":false}' > "$(pwd)/phase6-native-pow.json"
export NATIVE_PARAMS="$(pwd)/phase6-native-pow.json"
```

Use the three startup commands above with `--override-params-file="$NATIVE_PARAMS" --perf-metrics --perf-metrics-interval-sec=1` appended to each. Append `--enable-unsynced-mining` only on node A for this fixture. Reopen node C with the same command, binary, data directory and overrides. Use fresh directories when switching from the simulated-PoW exercise. The override enables normal proof validation and keeps the fixture transparent with the same Simnet genesis.

For a reproducible block source and complete acceptance check, run the focused Linux driver from the pinned checkout. Reserve the six loopback ports shown above and let the driver start the processes:

```bash
cargo build -p kaspad
export KASCADE_CLI_KASPAD="$PWD/target/debug/kaspad"
export KASCADE_CLI_LOGS="$(mktemp -d)"
cargo nextest run -p kaspa-testing-integration --test phase6_loopback_cli_runbook \
  --run-ignored all --test-threads=1 --success-output immediate \
  -E 'test(=phase6_loopback_cli_real_pow_recovery)'
```

Pass requires a common fresh genesis and two followers connected to A, an above-target nonce rejected, exactly 12 valid blocks accepted, unchanged DAG accounting after an exact duplicate submission, and six missing blocks recovered by C after reopening. All three final selected tips, block counts, pruning points and virtual DAA scores must agree. Record reopen-to-RPC and reopen-to-tip times, per-node CPU and RSS, storage and P2P counters, and three clean exits. Inspect `driver.log`, the daemon stdout/stderr logs, `metrics.jsonl` and `recovery.json` in the Actions artifact. A loopback pass is evidence for this native-PoW workload; representative load and competing native-PoW tips remain separate gates.

New native-PoW Actions artifacts also include the exact override file and `provenance.json`. The manifest records the source actually checked out by Actions, run and attempt identifiers, the daemon SHA-256, and SHA-256 digests of the native-PoW logs and observations. On pull requests the checked-out commit can be GitHub's synthetic merge commit. Use the commit printed by the workflow's checkout step for verification.

```bash
python3 scripts/phase6-native-pow-evidence.py verify "$EVIDENCE_DIRECTORY" \
  --checkout-sha "$CHECKED_OUT_SHA"
```

Pass `--binary "$KASPAD"` to compare a retained binary against the recorded SHA-256. Use the artifact supplied by the expected Actions run as the reference.

## First native-PoW evidence

On source `13dc263c9883b858ca30173d5087f954bc46b58d`, the focused native-PoW test passed in 4.878 seconds. All three processes accepted the expected 12-block DAG, rejected the invalid nonce, left the DAG unchanged on a duplicate submit, and shut down cleanly. C recovered six blocks in 312 ms from process reopen, with RPC ready after 310 ms. The raw artifact was uploaded. The matrix's metrics validator failed because Nextest appended an ANSI reset to the final JSON line, and early process metrics included zero RSS before the sampler's first observation. The follow-up strips terminal control sequences before JSON validation and waits up to ten seconds for each process sample. [Run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379) · [Raw artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379/artifacts/14803614244)

## Paced native-PoW run with follower catch-up

Use the same built daemon, isolated ports and environment variables as the recovery run. Select the paced case:

```bash
cargo nextest run --locked -p kaspa-testing-integration --test phase6_loopback_cli_runbook \
  --run-ignored all --test-threads=1 --success-output immediate \
  -E 'test(=phase6_loopback_cli_sustained_native_pow)'
```

The driver waits at least one second between accepted blocks. It requires all three daemons to select 61 consecutive native-PoW blocks, stops C for 10 blocks, reopens C on its original directory and verifies catch-up, then checks five further blocks on all three peers. The summary reports observed selected-block cadence measured from the first to the last acceptance in the connected phase, RPC and catch-up latency, and final tip, block count, pruning point and virtual DAA. Actions retain raw logs, individual block observations, per-node resource metrics and the pinned provenance manifest. The one-block-per-second submission pace leaves the Simnet network configuration at its defaults. Higher-rate representative transaction load and sustained retarget coverage remain separate gates.

Set `KASCADE_CLI_CONNECTED_BLOCKS=1000` to run the extended connected phase before the same interruption and restart. Allow about 17 minutes for pacing alone. The block records distinguish mining and submission time from RPC-observed tip convergence. Every 100 connected blocks, the driver records point-in-time CPU, RSS, network and storage metrics for each node. The summary also sums the logical file sizes under each node's own data directory without following symbolic links. It records these at genesis and final convergence while the nodes are running and again after clean shutdown. The resulting totals include other files in those directories and are independent of allocated filesystem size.
