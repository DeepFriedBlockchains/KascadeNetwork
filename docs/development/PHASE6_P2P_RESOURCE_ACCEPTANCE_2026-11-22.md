# Phase 6 P2P resource acceptance

Source reviewed: `15f99ca97799b414e57fb20b8f53f1d85feb1f6c` (draft #25). Scope: loopback Simnet with the existing development-only Phase 6 preview. Current code has transport decoding limits and bounded count-based route queues. Neither control establishes a memory or CPU budget for a node with several peers.

## Baseline to measure

Use the existing fixtures in the same Actions runner class. Record the exact source, runner configuration, fixture name, elapsed time, peak process-tree RSS, peak active peers, and transfer counts. Instrument the transport and router to distinguish encoded traffic, decoded message sizes, and retained queue bytes. Start with valid fixtures so the resource budget accommodates the largest supported message and ordinary synchronization.

| Fixture | Evidence |
| --- | --- |
| Pruning sequence commitment and UTXO sync | Largest valid proof and UTXO payloads, chunk counts, memory peak, and verified completion |
| SMT and shielded archive recovery | Largest valid chunk for each state lane, bytes retained per active import, and readiness after validation |
| Three-node partition, rejoin, and restart | Recovery latency, active peers and import count, peak memory, and converged checkpoints after reopening |

Run these existing functional baselines in GitHub Actions:

```sh
cargo nextest run -p kaspa-testing-integration --lib --features phase6-staging-preview \
  -E 'test(daemon_pruning_seqcommit_sync_test) | test(daemon_ibd_smt_state_sync_test) | test(authority_integration_preview_verifies_shielded_archive_backfill) | test(authority_integration_preview_three_node_partition_rejoin)'
```

This command verifies fixture functionality. The measurements require resource instrumentation and an Actions artifact before this gate can be marked passed.

## Budgeted acceptance

1. Agree on decoded bytes per message category, retained bytes and active imports per peer, and a total node budget. Include handshake and decode work in the aggregate budget. Choose values after measuring valid traffic and checking protocol compatibility.
2. Add in-process accounting tests with test-sized quotas. Check exactly-at-budget acceptance, release after completion and cancellation, and quota isolation between peers. Bytes and work must be charged before additional work proceeds, with checked arithmetic. Test that a closed stream returns all capacity and allows a normal replacement peer to progress.
3. Re-run the valid fixtures with the candidate policy. Require observed memory and queue bytes within the declared envelopes, live accounting released after disconnect, and no persistent growth across repeated connect/sync/disconnect cycles. Require the original content, readiness, and restart assertions to pass, including the durable checkpoint after reopening.
4. Save one JSON record per fixture with `source_sha`, `runner`, `fixture`, `sample_count`, `peak_process_rss_bytes`, `max_decoded_bytes_by_category`, `peak_retained_queue_bytes`, `peak_active_peers`, `elapsed_ms`, `budget`, and `status`. Include the raw measurements and logs as a CI artifact. Repeat the comparison on the same source and runner class. Final acceptance requires reviewed bounds and evidence from the full Phase 6 matrix.

Keep the existing wire compatibility and activation defaults until these gates are satisfied. Public RPC exposure and authentication have separate acceptance criteria.

Public code reviewed: [P2P transport](../../protocol/p2p/src/core/connection_handler.rs), [router](../../protocol/p2p/src/core/router.rs), [daemon fixtures](../../testing/integration/src/daemon_integration_tests.rs), and [partition/rejoin fixture](../../testing/integration/src/phase6_partition_rejoin.rs).
