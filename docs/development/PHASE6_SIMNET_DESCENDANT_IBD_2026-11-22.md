# Phase 6 Simnet descendant IBD integration, November 22, 2026
November 22, 2026. Branch `feature/phase6-simnet-descendant-ibd-20261122`, based on
draft [PR #15](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/15) at `8bd8fff703f612618b151358800f54c71cc636c7`.
See [the dated report](docs/development/PHASE6_SIMNET_DESCENDANT_IBD_2026-11-22.md).

The active Simnet preview obtains missing headers before shielded
reconciliation when the receiver already has a committed descendant
tip and stable SMT and UTXO state. It authenticates the committed tip
against its selected child before any missing descendant body is applied.
An already UTXO-validated peer pruning point on the local selected chain
also permits normal IBD when only shielded readiness is unstable.

A two-node Simnet regression performs an authenticated IBD promotion,
disconnects the receiver, sets its shielded readiness false, mines a
header gap, and requires the receiver to reconnect and resume validating
blocks while preserving the promotion record and active role. The
focused regression and all eight jobs are required. After validation,
continue with the remaining consensus and testnet acceptance coverage in
KASCADE-SPEC.md. Keep the preview default-off and Simnet-only.

Draft [PR #16](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/16) first ran at
`e147119245e990531fc77f74051b5ceab7b75e08` in [run 45058650015](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45058650015).
Seven jobs passed. Native job 137319441954 reached successful authenticated
promotion, then the focused test panicked when the fixture tried to ban a
permanent connection. The descendant fixture now opens its first peer as
temporary and still requires actual disconnect, readiness reconciliation,
and validated catchup. Production code is identical. Verify the replacement
focused regression and all eight jobs before validating this milestone.
