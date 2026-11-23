# Phase 6 real Simnet eligible red reward membership

The new fixture uses a small GHOSTDAG k=1 cluster and three sibling blocks. It runs real template construction, block validation, selected-tip transition and stored GHOSTDAG retrieval. The merge must retain two blue sources and one red source and advance DAA by three because all three are eligible. It compares every payout, including the eligible red subsidy routed to the merging miner, with independent source payload values and distinct scripts.

The builder-level tests on this branch independently cover the non-DAA red fee-only path and developer settlement at an accrual boundary. The real DAG fixture covers the eligible red path with no fees on an isolated Simnet. Creating a real non-DAA red with eligible fees and testing mixed settlement across a persistent replay remain later gates. CI is deferred while the #31 baseline runs.
