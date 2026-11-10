# Phase 6 promotion receiver and typed UTXO completion

November 10, 2026

[Draft PR #5](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/5)
at `4eb422549fa822b804968f9c2911018b518286f0` passed all eight jobs in
[run 42506751029](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506751029).
[Draft PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6)
at `2afec236ac42e03bdeaf21d06c0191a70bba99f7` passed all eight jobs in
[run 42509613211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42509613211).
Each passed four focused regressions, 1,561 native workspace tests
(25 skipped), and 32 no-asm tests.

The typed UTXO worker checks the committed pruning point and recomputes
the UTXO commitment while retaining the pruning and metadata guards.
The owner-bound batch writes the position, the stable flag, and the
closed authority revision together. Store caches are invalidated after
the synchronous RocksDB commit. The coordinator rejects generic UTXO
batches and requires typed completion before authenticated readiness.

The manager verifies a single-use receipt on the staging receiver
before changing the active consensus. A rejection returns the staging
receiver to its caller for worker drain and cleanup. In the inactive
implementation, the processor checks the concrete receipt type and
exact processor identity before consuming the database-bound lease.
The descendant recovery fixture uses the receipt path and the manager
test covers rejection when the receiver does not implement verification.

The IBD UTXO import can be called separately from the active-index reset.
The existing IBD path still uses the legacy ordering. Production proof,
SMT, shielded and UTXO adapter wiring and end-to-end promotion remain
open. This branch does not activate Phase 6 or alter consensus rules.

Candidate `bc393eae2d9bd03df2f684d06e53ec78e865da60` triggered
[run 42894808941](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42894808941),
which reported two formatting differences. Formatting passed on
`72af6c818be80c8a0365036e23c4744da8b4bf94` in
[run 42895086028](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42895086028).
Check and strict Clippy also passed. The full workspace result was
pending when the follow-up source was prepared. Continue at the proof receipt handoff from
the IBD defender to the staging receiver, then carry the same permit
through the component workers before routing IBD through the verified
commit entry point.

## Follow-up

The IBD proof path now receives an opaque proof receipt from the defending
consensus and hands that exact proof to the staging receiver in both
normal and sanity-check mode. The underlying validation and application
methods use the existing rules. The focused test verifies that a rejected
proof produces no receipt and the receiver receives the accepted header.

The inactive promotion receipt retains the authenticated proof and
shielded child. Under the authority and pruning guards, promotion rechecks
the committed proof, SMT, shielded, and UTXO data. The descendant fixture
removes a committed UTXO after lease issuance, verifies that promotion
fails closed, and verifies that restoring the row alone cannot revive
authority. Follow-up Actions results are pending.
