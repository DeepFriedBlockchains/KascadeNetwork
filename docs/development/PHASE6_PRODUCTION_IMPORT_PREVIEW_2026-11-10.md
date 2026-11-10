# Phase 6 production import preview

November 10, 2026

The starting branch `feature/phase6-owned-proof-import-20261110` at
`ecf0aacafa9cfc3028eb363021c5aebf733c62e1` passed all eight jobs in
[run 42898217206](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42898217206).

The default-off `kaspa-consensus/phase6-staging-preview` feature connects
receiver admission before the production proof writer. Trusted bodies,
headers, SMT, shielded and UTXO work retain that import reservation
through their receiver proxy. The managed staging lifecycle retains the
factory slot while detached tasks drain and stops the receiver when the
import is cancelled. The verified route checks the complete virtual
UTXO set and multiset before issuing a receipt and again at commit.
A receiver advertising preview support rejects promotion without a
receipt.

Tests cover pending receiver work on cancellation and incomplete or
drifted virtual UTXO evidence, as well as verified durable completion
and the existing stale-authority and restart cases. CI enables the
preview for Check, strict Clippy, and focused regressions. The regular
workspace suite continues to exercise the default build.

Full peer-driven integration coverage and candidate CI remain pending.
The default consensus settings and default IBD promotion route remain
in effect.
