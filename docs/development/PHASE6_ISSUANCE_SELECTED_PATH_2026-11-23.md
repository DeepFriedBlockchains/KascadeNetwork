# Phase 6 issuance fixture and selected ancestry

November 23, 2026

This staged test composes the existing selected-child body verifier, selected-chain descendant verifier and #41 issuance reopen fixture. Two sibling child bodies carry the same test commitment but different headers. The fixture changes the committed selected path to the alternate child. It checks that the previous body still authenticates in isolation but fails the current-chain gate, even if its old tip is supplied. After dropping the database handle and reopening the same directory, the test reruns the body and ancestry verification against reopened stores. The old child still authenticates in isolation but fails the current-chain gate with either tip. Only the replacement's nullifier diff is returned. The selected replacement supplies the expected checkpoint commitment from the reopened database, and a changed paid/deferred split is rejected.

The test uses fixture-seeded chain data and the existing shielded slot as a stand-in in the test only. It does not add issuance data to pruning proofs, authenticate a network-supplied pruning chain, or wire a production issuance path. CI is staged behind #41. Production still needs authenticated ancestry, atomic settlement promotion and post-restore consensus proof.
