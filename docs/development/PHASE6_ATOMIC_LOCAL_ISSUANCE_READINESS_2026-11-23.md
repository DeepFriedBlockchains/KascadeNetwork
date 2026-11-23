# Phase 6 issuance and guarded pruning readiness

## Local verified replay

The opt-in IBD completion path checks the established pruning proof, SMT, shielded child, and pruning and virtual UTXO verifiers. Under the same pruning, virtual and selected-chain guards it stages the locally replayed issuance pair with the first durable readiness write. Activation remains disabled. The named fixture tests missing evidence, a substituted equal-total root, abandoned import and a successful reopen under the actual IBD authority. It uses a zero-issuance genesis replay. Claim/release conservation and stale reorganization are covered separately in the issuance store and #53 fixtures. A process-exit fixture spanning real nonzero issuance and this atomic readiness write is still required.

## Fresh pruned receiver

The reusable selected-child coinbase check binds the body to the stored header's transaction root and verifies the selected parent. It provides membership. Consensus must also validate the semantics of any issuance field before accepting it. The header's UTXO root and accepted-transaction root and the currently active shielded root do not commit cumulative issuance history and deferred balances. A peer's checkpoint cannot provide its own expected digest.

The candidate boundary is a separate, versioned issuance commitment in the selected child. It must commit the independently computed checkpoint for its selected parent: network, pruning point, canonical ordered claims and developer releases, minted, paid and deferred balances. Use the existing issuance hash plus the existing transaction Merkle and selected-chain proof verification. The block acceptance path must reject a wrong commitment from its own settlement context; then a fresh receiver can authenticate that required field from the independently proven child. A body field without that consensus check is insufficient. Activation schedule, codec and commitment placement, genesis/migration baseline, and network compatibility require owner approval before enabling. No field, wire change or activation is introduced here.

After that gate, substitute a proof-derived anchor for local replay in the same staged IBD completion. Exercise authentic nonzero claims and releases, wrong network/point, forged and missing evidence, write interruption, reopen, and selected reorganization through the production importing path. Keep readiness closed whenever the selected issuance evidence is missing or stale. Production issuance and the proposed 29B KCADE finite policy remain disabled.
