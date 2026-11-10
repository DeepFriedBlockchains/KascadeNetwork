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
Run 42909275640 reported a line wrap in the guarded ancestry call.
This revision fixes both and restricts preview selection to Simnet.
