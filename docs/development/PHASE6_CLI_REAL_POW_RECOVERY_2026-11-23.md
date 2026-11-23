# Phase 6 CLI recovery with native proof of work

November 23, 2026

This isolated Simnet test runs three real kaspad processes using the same explicit loopback topology as the startup runbook. It sets proof-of-work skipping to false on every node and rejects an above-target nonce without weakening block validation. It then mines and relays twelve easy native PoW blocks. One follower stops after six, reopens the same database, and must catch up exactly to the other nodes. It checks final tip, block count, pruning point, and virtual DAA. Both CLI cases run serially because they share ports. The Actions artifact keeps the full driver and per-process logs.

This is an easy native PoW functional acceptance case, with locally submitted templates. It does not establish sustained hashrate, external miner or AuxPoW interoperability, or behavior over public network topologies. CI is staged behind #42's simulated-PoW process baseline. No production or issuance configuration is touched.

The staged real-PoW pass also records each process's RPC CPU usage, RSS, disk I/O and storage bytes, and P2P byte counters at genesis, before stopping C, and after convergence. Process counters for C start a new series when it reopens. The measurements are observations, not resource budgets or sustained mining results.
