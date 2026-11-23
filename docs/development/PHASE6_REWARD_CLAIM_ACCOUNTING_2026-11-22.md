# Phase 6 competing-tip reward claim fixture

The test observes the existing template builder, block insertion, selected-tip transitions and coinbase outputs on an isolated Simnet DAG. It assigns distinct payout scripts to track each source block in a fee-free fixture. Two sources share a DAA score, a heavier branch replaces the first branch, and the join must credit the eligible sources outside the new selected parent's past. The next block must credit only the join and cannot repeat the earlier claims.

This evidence is scoped to a transparent blue-source DAG. Before a rewarded ordinal becomes consensus state, extend the fixture to non-DAA and red reward aggregation, shielded minting, fee and deferred-entitlement accounting, genesis qualification, authenticated pruning snapshots, persistent reopening and competing-tip replay from a common checkpoint. Use the sustained-rate harness to measure uniquely credited events and compare them with generated blocks. Keep schedule arithmetic and production activation separate.

## Persistent replay extension

The original fixture passed 8/8 on `ca9c498709c3c5d4c9d664806adc9c83b767c666` in [run 45094765692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094765692). The follow-up retains its original assertions, compares accepted coinbase records and the complete virtual UTXO set across a RocksDB close/reopen, and replays the exact same blocks with the other branch arriving first. It also checks each join payout and the absence of the abandoned branch's coinbase in accepted UTXOs. The extension uses a transparent fee-free fixture. Authenticated pruning restoration, other reward paths, and full issuance accounting remain open. CI for the extension is pending.
