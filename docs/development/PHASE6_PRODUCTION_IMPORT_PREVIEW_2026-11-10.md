# Phase 6 production import preview

November 10, 2026

The starting branch `feature/phase6-owned-proof-import-20261110` at
`ecf0aacafa9cfc3028eb363021c5aebf733c62e1` passed all eight jobs in
[run 42898217206](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42898217206).

The default-off `kaspa-consensus/phase6-staging-preview` feature limits the preview to Simnet staging receivers and connects
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

[Run 42908550005](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42908550005)
passed Check and reported a formatting difference in managed staging.
The next source also pins the virtual read guard through the durable
write and requires committed UTXO-valid status at the pruning point.

The shielded commitment child lookup now reuses the proxy's existing
pruning read guard. The unguarded receiver path still acquires its own
guard. A focused regression checks both proxy paths.

Run 42909011971 reported a test-only Hash import in the feature build.
Run 42909275640 reported a line wrap and an accidental rename of the
unguarded session constructor. The follow-ups fix these issues and
restrict preview selection to Simnet.

[Draft PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is stacked on the green `feature/phase6-owned-proof-import-20261110`
branch at `ecf0aacafa9cfc3028eb363021c5aebf733c62e1`. The PR head is
`ebcb3ce00043b0d3f73a2ee7508f0cc68a363a69` on `feature/phase6-production-import-preview-20261110`.
The authoritative [run 42909699012](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42909699012)
has passed Check, formatting and strict Clippy, Check WASM32, and
Check no_std. Test Suite, Test WASM32, Linux release and the WASM SDK
build are still running. The same-SHA push run 42909689791 was superseded. The green
parent passed eight focused regressions, 1,565 workspace tests (25
skipped), and 32 no-asm tests in run 42898217206.
