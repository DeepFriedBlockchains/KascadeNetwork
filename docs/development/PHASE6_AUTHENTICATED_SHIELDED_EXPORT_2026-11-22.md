# Phase 6 authenticated shielded export, November 22, 2026
November 22, 2026. Branch `feature/phase6-authenticated-shielded-export-20261122` is stacked on
draft [PR #14](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/14) at documentation-only
checkpoint `e178963969ea9da2bc5b0a26feb9d3e5243a5252`. See [the dated report](docs/development/PHASE6_AUTHENTICATED_SHIELDED_EXPORT_2026-11-22.md).

The Simnet preview captures the pruning-point metadata and projected
nullifier rows from one pinned virtual/selected-chain view. It verifies
the selected child's header and coinbase body and checks the projected
membership against the resulting proof-of-work commitment before any frame
is sent. Missing or corrupt evidence, an unready point, and inconsistent
descendant diffs fail closed. Network backpressure holds only owned export
data. The rest of the network uses the existing transfer path.

The persistent regression now requires an authenticated stored child,
rejects wrong or unready points and divergent membership, and checks that
an already captured response and the promotion record survive later
state changes. The preview retains one projected nullifier vector per
active export. All eight jobs and the focused regression are pending.
Next, validate this candidate, then continue consensus integration and
end-to-end testnet readiness. Keep preview default-off and Simnet-only.

Draft [PR #15](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/15) initial source
`47a4f69a42e955a85036b5089ca7b24d4aa0ce72` reached [run 45055027648](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45055027648).
Lints job 137308268988 identified two Rustfmt layout changes. This
revision applies both formatter hunks. Review the focused regression and
the full workflow on the replacement head before validating this milestone.
