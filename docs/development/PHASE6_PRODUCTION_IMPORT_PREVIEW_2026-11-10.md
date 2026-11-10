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

The existing two-daemon shielded IBD test is also exercised with preview
enabled. It requires the active receiver to require verified promotion
and checks the imported frontier and post-IBD block acceptance. The
additional peer-driven cancellation and restart cases remain open.
Candidate CI is pending.
The default consensus settings and default IBD promotion route remain
in effect.

The initial draft [run 42908162867](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42908162867)
reported formatting differences and two shielded return-type mismatches.
The follow-up fixes those errors and uses one stop signal per managed
staging lifecycle before joining workers and removing the staging DB.

[Draft PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is stacked on `feature/phase6-owned-proof-import-20261110`.
The exact source is `20db28f7dbd8e228d695202f6129fbbc2fd0586a` on
`feature/phase6-production-import-preview-20261110`. Authoritative
[PR run 42908550005](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42908550005)
is pending. Its same-SHA push run 42908537162 was superseded.
The green parent passed eight focused tests, 1,565 workspace tests
(25 skipped), and 32 no-asm tests.
