# Isolated Simnet native proof-of-work acceptance

November 22, 2026. Draft test on top of validated PR #16 documentation checkpoint `049ffaf17d3fdc74c93a36b399edb0b4b66cb0b3`.

The [integration test](../../testing/integration/tests/phase6_native_pow_relay.rs) uses two loopback daemons and a temporary Simnet override. It checks a nonce that exceeds the target is rejected over the ordinary mining RPC and that valid native kHeavyHash blocks become the selected tip on both nodes in each relay direction. This does not validate AuxPoW or sustained throughput. Both remain separate acceptance checks.

The [workflow](../../.github/workflows/ci.yaml) runs the focused test before building the workspace, and its existing combined Phase 6 filter includes the same test. The preview remains default-off and Simnet-only.

## Verified native mining acceptance

All eight jobs passed on implementation `bd4f65851c0267f70aced3602f61391c5fdfc578` in [run 45081005178](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45081005178). Native job 137388968702 passed the dedicated two-node native-work test and reran it in the 36-case Phase 6 suite. It also passed 1,579 workspace tests (25 skips), and 32 no-asm tests. The test rejected above-target native PoW and confirmed valid work was accepted and relayed in both directions.

The separately prepared [AuxPoW fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/commit/384d0bd221d05077f1afc66f03a552819a3fd798) is the next focused mining acceptance test. Its result is pending. This checkpoint changes documentation only.
