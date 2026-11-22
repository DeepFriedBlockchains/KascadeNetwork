# Isolated Simnet native proof-of-work acceptance

November 22, 2026. Draft test on top of validated PR #16 documentation checkpoint `049ffaf17d3fdc74c93a36b399edb0b4b66cb0b3`.

The [integration test](../../testing/integration/tests/phase6_native_pow_relay.rs) uses two loopback daemons and a temporary Simnet override. It checks a nonce that exceeds the target is rejected over the ordinary mining RPC and that valid native kHeavyHash blocks become the selected tip on both nodes in each relay direction. This does not validate AuxPoW or sustained throughput. Both remain separate acceptance checks.

The [workflow](../../.github/workflows/ci.yaml) runs the focused test before building the workspace, and its existing combined Phase 6 filter includes the same test. Validation is pending. The preview remains default-off and Simnet-only.
