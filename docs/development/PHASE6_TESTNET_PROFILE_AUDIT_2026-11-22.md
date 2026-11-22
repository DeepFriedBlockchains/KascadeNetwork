# KASCADE testnet profile audit

November 22, 2026. Reviewed validated Phase 6 implementation `c9f378177c0ddd25f3152b55c8d34b593c3cfcc4` and documentation checkpoint `049ffaf17d3fdc74c93a36b399edb0b4b66cb0b3`. This is a source inventory. The public testnet profile requires a separate design decision and an isolated acceptance run.

| Area | Existing source | Required KASCADE candidate evidence |
| --- | --- | --- |
| Network identity and genesis | The inherited testnet is `testnet-10` using `TESTNET_GENESIS`. Testnet DNS seeders are empty. The chain has separate constants for main, test, and simulated networks. | Record the candidate ID, magic/domain, full genesis and genesis derivation, and explicit bootstrap peers. Check that peers on the inherited and candidate profiles reject cross-network connections in both directions. |
| Addresses and ports | The codec uses `zkastest:` for testnet and `zkassim:` for Simnet. Inherited testnet suffix 10 uses assumed P2P port 16821 and default gRPC port 16820. | Reserve the candidate receive prefixes and P2P/RPC ports, check wallet and daemon agree, and verify CLI/API rejection for other network address types. |
| Rate and proof of work | The inherited testnet uses 1 BPS, enforces PoW, and enables inherited merged mining from genesis. Default Simnet targets 10 BPS and simulates PoW. | Rehearse native PoW and AuxPoW on a separate isolated profile. For the 5-BPS target use a 200 ms target plus measured difficulty, finality, pruning, propagation, and verification limits. |
| Shielded configuration | The inherited testnet sets `shielded_coinbase: true`. Default Simnet sets it false. Phase 6 integration fixtures use explicit Simnet overrides. | Keep KASCADE shielded activation disabled. Record each allowed test override and its exact configuration hash. |
| Issuance and rewards | `CoinbaseManager::calc_block_subsidy` currently retains the inherited positive perpetual tail. The KASCADE spec calls for an exact 23B KCADE cap. This source state cannot certify the cap. | Supply independent integer boundary vectors and cumulative issuance proof for the approved KASCADE schedule before public-profile acceptance. Verify reward splits, KRN and KCF transitions separately. |

Sources: [network identifiers and port defaults](../../consensus/core/src/network.rs), [genesis constants](../../consensus/core/src/config/genesis.rs), [network parameters](../../consensus/core/src/config/params.rs), [address codec](../../crypto/addresses/src/lib.rs), [subsidy](../../consensus/src/processes/coinbase.rs), and the [protocol draft](../../KASCADE-SPEC.md).

## Isolated profile acceptance

Use separate app directories and explicit loopback peer and RPC bindings for Actions. Keep each node's configuration digest with the run. For the three-node partition exercise, record the exact branch tips and common ancestor, both active peer sets at each partition stage, and the selected tip, pruning point, and state commitments after rejoin and restart. Treat any lingering inherited prefix, net ID, genesis, or reward policy as an open acceptance item. Execute the profile and negative checks only in isolated GitHub Actions until a public profile has owner approval. See the [acceptance matrix](PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md).

## Documentation validation

Exact documentation source `50b0d19290c54ffef5df3e0919d99645bb528e19` passed all eight jobs in [run 45078524217, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45078524217/attempts/3). The single WASM retry succeeded. This confirms the documentation checkpoint's CI result. The production profile and quantitative testnet gates remain open.
