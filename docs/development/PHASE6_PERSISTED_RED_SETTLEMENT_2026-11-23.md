# Phase 6 persisted Simnet red settlement

November 23, 2026

The validated #34 fixture supplies two blue rewards and one eligible red reward. Its selected successor now accepts the real merge coinbase. Before and after RocksDB reopen, the test checks the selected ancestry, stored body commitment, the exactly-once accepted coinbase source, every committed payout amount and script, and the reconciled sum of the three independently read source subsidies. It compares the complete fixture UTXO snapshot across restart. The focused regression runs before the workspace build.

The fixture is transparent, fee-free, and uses the legacy Simnet schedule with developer accrual disabled. It establishes the real persisted settlement side of the pruning test. There is no production checkpoint codec, independently consensus-authenticated issuance digest or bounded persisted reward-claim/deferred-balance view to pass through import. The remaining integration is to authenticate those rows against the selected child, reconcile actual mixed subsidy/fee/deferred settlement, and publish them atomically with readiness through the guarded import path. The 29B issuance candidate remains inactive.
