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
