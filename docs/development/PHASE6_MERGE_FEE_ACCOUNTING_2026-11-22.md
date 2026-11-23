# Phase 6 eligible subsidy and fee-only merge accounting

The fixture calls the production coinbase builder with one eligible blue, one eligible red, and one non-DAA red. It checks the exact outputs and the 3,004 Kazmi net subsidy after subtracting all 67 Kazmi of recycled fees. The non-DAA red contributes only its fee. The merging miner receives the red aggregate, so an output count cannot serve as a count of newly rewarded units.

This test supplies membership explicitly in order to isolate settlement behavior. Producing those classes through live DAG validation, authenticating a credited ordinal, and testing snapshot restoration remain separate checks. The validated transparent join fixture in #30 is the baseline. The current branch changes tests and CI only.

The mixed mergeset also runs with the inherited developer accrual enabled. Independent samples below and across a payout boundary check exact output order, fees, and carry. Adding the ending accrued entitlement, then subtracting prior entitlement and recycled fees from gross outputs yields the same 3,004 Kazmi subsidy in both cases. A payout of prior entitlement does not credit a new reward slot. The inherited 5% fee is used as a fixture setting to exercise carry behavior under the current builder.
