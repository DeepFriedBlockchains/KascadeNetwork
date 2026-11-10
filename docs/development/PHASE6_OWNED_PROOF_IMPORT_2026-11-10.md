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
Candidate CI is pending. Continue by passing the coordinator and its
permit through trusted-body processing, SMT, shielded and UTXO writes
before invoking the verified promotion boundary.
