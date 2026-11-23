# Phase 6 native-PoW live peer reconnection

Three independent loopback Simnet processes use the same native-PoW override described in the operator runbook. Only C's upstream link traverses a local TCP forwarder. The forwarder serves one connection at a time and is suspended by dropping both sockets. C must remain alive, stay on the old tip, and reconnect to A after forwarding resumes. It must catch up three native-PoW blocks and later reopen on the same tip while still offline. A's original block is resubmitted through the existing duplicate accounting assertion. A and B remain connected throughout. The test records per-node metrics and daemon logs. It applies only to this workload; it does not establish a sustained production resource budget or reward-claim conservation.

For the runbook's automated command, replace the Nextest expression with `test(=phase6_loopback_cli_live_peer_reconnect)`. The driver additionally reserves an ephemeral loopback port. The dedicated Actions step stores process output, peer metrics, and convergence observations.

The driver emits a `phase6_peer_recovery` record with the link interruption duration, time from forwarding to peer reconnect and DAG catch-up, offline reopen duration, synchronized blocks, and final tip. A peer-count or DAG timeout reports the last observed state rather than only the failed condition.
