# Phase 6 guarded issuance publication boundary

The test reuses the actual authenticated staging recovery, pruning UTXO/SMT and virtual checks, persistent lease consumption, and atomic promotion publisher. A malformed selected-child body must leave both the promotion key and the fixture settlement row absent and close readiness. With the valid child, the same deferred batch carries the marker and a test-only settlement row. RocksDB reopening must recover both plus the ready revision, and the exact checkpoint must verify against the locally derived fixture digest.

Production still needs an independently verified consensus issuance digest and a persisted reward-claim/deferred-balance view. The fixture digest remains outside the block and pruning proof. The test establishes the publication boundary and does not validate production settlement, power-loss behavior or issuance activation. #46 separately exercises real Simnet payout persistence.
