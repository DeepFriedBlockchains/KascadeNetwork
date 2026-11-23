# Phase 6 issuance fixture and selected ancestry

November 23, 2026

This staged test composes the existing selected-child body verifier, selected-chain descendant verifier and #41 issuance reopen fixture. Two sibling child bodies carry the same test commitment but different headers. The fixture changes the committed selected path to the alternate child. It checks that the previous body still authenticates in isolation but fails the current-chain gate, even if its old tip is supplied. The selected replacement supplies the expected checkpoint commitment.

The test uses fixture-seeded chain data and the existing shielded slot as a stand-in in the test only. It does not add issuance data to pruning proofs, authenticate a network-supplied pruning chain, or wire a production issuance path. CI is staged behind #41. Production still needs authenticated ancestry, atomic settlement promotion and post-restore consensus proof.
