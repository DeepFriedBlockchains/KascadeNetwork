# Phase 6 RPC safe-mode regression

November 22, 2026

The production daemon defaults to loopback gRPC and requires configuration for wRPC listeners. Its gRPC peer administration and shutdown handlers deny requests when unsafe RPC is disabled. The random-port integration harness explicitly bound its listeners to every interface despite using local clients. Bind all four test listeners to loopback so multi-node fixtures do not expose extra interfaces. No production listener or permission behavior changes.

The Simnet regression makes a loopback client call peer addition and shutdown in the daemon's default safe mode and requires the exact API denial. The same connection must continue to serve a public chain query. Run `cargo nextest run -p kaspa-testing-integration --test rpc_safe_mode` in Actions and then validate the full matrix. Coverage of external RPC authentication and per-client resource limits remains separate. Validation is deferred while the already-active Phase 6 matrices run.
