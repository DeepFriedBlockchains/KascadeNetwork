# Phase 6 owner-bound proof import

November 10, 2026

The draft PR #6 checkpoint `42476e2661b5e579f665407d18352a36af09c301` reached six
successful jobs in [run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
when this candidate was prepared. Its Linux release build and workspace
Test Suite were still running.

The receiver reserves a processor-owned import session with the
defender-accepted pruning proof. It rejects concurrent reservations,
foreign processors, and mismatching pruning history. The proof worker
pins the pruning guard, verifies the exact durable closed revision
before the writer, corroborates the committed proof afterward, and
rechecks the revision. It leaves the proof phase open for trusted-body
processing. Completion uses the retained evidence and has no
caller-supplied proof parameter.

Both production IBD proof branches now share the same proof and history
application entry point used by the inactive adapter. The active IBD
promotion behavior and consensus rules remain unchanged. The proof
writer continues to use the existing isolated staging stores. An
interrupted write leaves authority closed, and no authority mutex is
held across the legacy writer's point and virtual locks.

Focused coverage exercises competing ownership, a wrong history,
stale durable authority, and the real pruning proof and history writer.
The initial proof candidate `863ec4fd7293fa3c54916b229c0f9c5f9a6ba659` passed formatting and Check in
[run 42897872282](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42897872282)
before the streaming follow-up. Continue by passing the coordinator and its
permit through trusted-body processing, SMT, shielded and UTXO writes
before invoking the verified promotion boundary.

## Streaming writer drain

SMT and shielded IBD now close their chunk sender and join the writer on
both success and stream failure before returning to the caller. A test
pins the writer past a simulated peer failure and checks that the error
returns only after release. Stable flags still require successful stream
and worker completion. 
The combined source is `ecf0aacafa9cfc3028eb363021c5aebf733c62e1` on
`feature/phase6-owned-proof-import-20261110`. Its exact validation is
[run 42898217206](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42898217206).
The unchanged PR #6 baseline
[run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
passed all eight jobs at `42476e2661b5e579f665407d18352a36af09c301`.
It passed six focused regressions, 1,563 native workspace tests
(25 skipped), and 32 no-asm tests. Combined run 42898217206 has passed
Check, strict Clippy, Check no_std, Check WASM32, Test WASM32, and
Linux release. Its WASM SDK build and Test Suite are still running.
