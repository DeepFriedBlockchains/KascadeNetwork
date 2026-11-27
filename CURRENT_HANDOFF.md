# KASCADE public development handoff

Updated November 27, 2026.

## November 27 verified recovery and native acceptance

| Draft | Exact head | Evidence |
| --- | --- | --- |
| [#86 signed fee settlement](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/86) | `44984d3b646009ef0bd1383fdaf33c76ad726895` | [45755786331](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755786331) passed 8/8, including the focused signed fee/issuance/RocksDB case in 0.242 s, 36 authority and 1,603 workspace tests. Carrying the accepted transaction wrapper fixed the compile failure and preserved its contextual storage mass. |
| [#87 managed launcher](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/87) | `c8981013c5edf1b20bf4933410eb7ea2175f7aaf` | [45755694915](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755694915) passed 8/8. The launcher waits for the seed listener before starting outbound-only followers. The checked [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755694915/artifacts/15088566731) shows 2/1/1 live peer counts, common genesis, and clean exit in both starts using the same data directories. This was a genesis-stage startup check. |
| [#88 operational integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/88) | `5c55d1a3eccd9a235d7b33647fa2288dd9c62501` | [45755686377](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755686377) passed 8/8. Both management cleanup and guarded nonzero publication process-exit regressions passed, along with 43 authority, 1,617 workspace and 32 no-asm tests. |
| [#89 whole-block rate preflight](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/89) | `a1c8a8dfec21c3052b31c538330f4d88e8726769` | [45755801921](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755801921) passed 8/8, including the effective operator subsidy rate boundary in its focused step (0.007 s). |
| [#90 paced native-PoW](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/90) | `679ab9abb053763c85848bae1a0378d5e97a606b` | [45755839747](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755839747) passed 8/8, with actual native-focused runtime 78.488 s. Three processes accepted 76 blocks overall. During all-connected operation, 61 blocks spanned 60.419 s between first and last accepts, yielding 0.9930515 BPS against the 1 s pacing input. Max observed all-node selected-tip delay was 107 ms. C reopened the same directory, reached RPC in 310 ms and the exact agreed tip in 313 ms after missing 10 blocks. Five further blocks reached all three nodes. [Verified artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755839747/artifacts/15088877595). |
| [#91 multi-page pruning](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/91) | `936d89c004281da62814adb7a6cd196f30528ce8` | [45756192380](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45756192380) passed 8/8. The new focused authenticated retry ran in 15.624 s and re-executed in the 41-case authority selection. 1,606 workspace tests passed. |
| [#92 private peer discovery](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/92) | `a47a9fe0c50587c6669d00ab86b713621a1a7323` | [45756304425](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45756304425) passed 8/8 with real handshake and address exchange on an isolated RFC1918 namespace. C learned B through its sole explicit seed A and established outbound connections to both. The [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45756304425/artifacts/15088483249) records the 2.47 s focused case and the isolated interface layout. |
| [#93 bundled native miner and launcher](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/93) | `0a2d11ee5f469ba2b51dff34e24765c052d1d175` | Recovers the prior unpushed operator branch. The optional accepted-block limit leaves default mining unbounded. The new case requires four native blocks, exact agreement, and a same-directory restart to the same non-genesis state. [45971999397](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45971999397) pending. |
| [#94 settlement and pruning integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/94) | `6987852b440861367097776e67d5a37289749eda` | Stacks green #86/#89/#91 on #88 and enables their named checks in the combined run. Source and workflow checks passed. [45972050245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45972050245) pending. |

The #90 archive SHA-256 matched GitHub's digest. It reports final RSS samples of 140,021,760, 136,425,472 and 128,212,992 bytes. RPC storage size reported zero at all stages and is unavailable for growth analysis. The next native stage should measure actual on-disk bytes and the longer paced workload. The native result uses a 1-second test pacing target. The earlier 4.99344 BPS result is the simulated-PoW signed traffic rehearsal in #83. Native 5 BPS, mixed workload, representative resource limits and launch qualification remain open.

Independent tracks are longer native operation, native signed transactions, discovered-peer reconnection, actual process-exit multi-page recovery and fresh-receiver issuance design review. Public activation and production issuance stay disabled.

## November 25 operational integration and launch rehearsal

| Draft | Exact head | Verified result or next action |
| --- | --- | --- |
| [#81 Timing preflight](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/81) | `2935e49f` | [45707440494](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707440494) passed 8/8. The positive Simnet and zero timing/sample override cases also passed together locally. |
| [#82 Operational integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/82) | `406535f2` | [45707467797](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707467797) passed 8/8. Native 139533132342 used checkout `0c04199b` and passed 43 authority, 1,614 workspace and 32 no-asm tests. Named passes include legacy staging reopen, live interrupted pruning retry/reorg, native partition/rejoin and DAA reopen, nonzero settlement, and durable final evidence rejection. |
| [#83 Signed transaction smoke](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/83) | `a24fef83` | [45707623704](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707623704) passed 8/8. The focused 10.270 s test covered eight measured blocks and 16 signed transactions with matching settled outputs on three loopback nodes. The [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707623704/artifacts/15067398989) yields 4.99344 submitted BPS across seven intervals under simulated PoW. |
| [#84 Actual cleanup exits](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/84) | `ca6e7c3f` | [45707670756](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707670756) passed 8/8. The dedicated case ran in 0.056 s and asserts real exit 73 before and 74 after the atomic metadata batch, followed by reopening the same RocksDB, verifying the old-or-new pointer/row pair, preserving the active entry and retrying. The same matrix separately passed the live reorganization fixture. Native totals were 40 authority, 1,607 workspace and 32 no-asm tests. |
| [#85 Read-only Simnet health probe](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/85) | `ea52eb1d` | [45707905412](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707905412) passed 8/8. Four local focused probe cases passed. Healthy live samples are tracked by the launcher rehearsal. |
| [#86 Signed fee settlement](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/86) | `50a773f5` | [45708143714](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45708143714) passed 5/8. Check, Lints and native compilation all reported the same transaction ownership-type mismatch. The isolated correction is being validated. |
| [#87 Supervised native launcher](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/87) | `c8981013` | The [original run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45708250316) passed 7/8. C had no peer, and the launcher expected listeners on the outbound-only followers. The new head waits for the seed listener, checks actual listeners, and uses nonblocking signal handling. Local supervision fixture passed. [45755694915](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755694915) is pending for actual topology and restart. |
| [#88 Cleanup-exit and startup integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/88) | `5c55d1a3eccd9a235d7b33647fa2288dd9c62501` | Stacks the green #84 and #81 sources on #82. Formatting, workflow parsing and diff checks passed on tree `db39eb457aabc0a9aa5310d8ba459d3d88b4a19b`. [45755686377](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755686377) is pending for the new combined tree. |

### Work in progress on exact new heads

- [#86](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/86) now points to `44984d3b646009ef0bd1383fdaf33c76ad726895`. The type correction carries the exact mempool-validated transaction, including its contextual storage mass, into the block. The new signed fee and persisted issuance test passed locally in 0.26 s. [Replacement matrix 45755786331](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755786331) is in progress.
- [#89](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/89), on #81 at head `a1c8a8dfec21c3052b31c538330f4d88e8726769`, checks positive whole-block subsidy rates for the active timing phases of an operator-supplied override. An otherwise unused nonzero phase follows the existing fork semantics. Formatting and workflow checks passed. [45755801921](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755801921) is in progress.
- [#90](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/90) at `679ab9abb053763c85848bae1a0378d5e97a606b` adds the dedicated three-process native Simnet acceptance run. It targets 61 connected blocks at one second spacing, 10 during the offline follower gap, and five after same-directory catch-up. It records acceptance timestamps, agreement, process metrics, and source/configuration provenance. The local evidence parser passed synthetic-format validation. [Exact native run 45755839747](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45755839747) is in progress. The configured pacing is an input. Actual throughput and recovery latency require the completed artifact.
- [#91](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/91) at `936d89c004281da62814adb7a6cd196f30528ce8` on #79 passed its new focused Simnet regression locally in 17.50 s. The committed exporter and replayed receiver each had 1,302 authenticated pruning UTXOs. The test interrupted after the first nonempty page, verified closed readiness across restart, required complete retry and exact state, then checked live reorganization and offline reopen again. [45756192380](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45756192380) is in progress. The earlier exploratory long-chain candidate remains separate.

**Current priorities.** Verify #87's repaired exact-source rehearsal and inspect both health artifacts. Verify #88's cleanup process exit on the operational stack. Resolve #86's focused consensus and RocksDB case. Exercise a minute of paced real native-PoW mining across three separate daemons, then stop the follower, mine an offline gap, reopen its existing directory, prove exact catch-up and record measured throughput and final chain agreement. Continue the honest multi-page pruning fixture with signed mature coinbase fanout and the active timing profile preflight independently. A representative sustained mixed workload and explicit network and fresh-receiver issuance authority approvals are still required.

## November 25 Phase 6 session recovery

GitHub `main` remains at `f34a2f03e87c57b36ccdaf51072ee360e0fa3d15`. These are the completed draft results, confirmed from the eight Actions jobs and native job logs. The table supersedes the pending statuses in the dated records below.

| Draft | Head | Actions | Native evidence |
| --- | --- | --- | --- |
| [#71](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/71) | `9d91cf6` | [45473741398](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473741398) · 8/8 | Restarted native difficulty calculation and offline mining followed by real two-peer relay. 1,579 workspace tests. |
| [#72](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/72) | `396599f` | [45473757936](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473757936) · 8/8 | Independent inactive replay vectors and duplicate developer release rejection. 1,602 workspace tests. |
| [#73](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/73) | `d5cf25c` | [45498114481](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45498114481) · 8/8 | Real pruning UTXO transfer interruption after a committed page, reopen, retry, live reorganization and offline reopen. 40 authority cases, 1,605 workspace and 32 no-asm tests. |
| [#74](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/74) | `cc45a08` | [45473814935](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473814935) · 8/8 | Receiver page bounds and empty/full behavior. 1,605 workspace tests. |
| [#75](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/75) | `a583b3d` | [45473893184](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473893184) · 8/8 | Closed durable readiness across reopening after final evidence rejection. 1,601 workspace tests. |

PR #73 now targets #74. The earlier combined validation ref at `84589bb`, the current #73 head `d5cf25c` and Actions checkout `7afe67415751dd550455ff4f64bef2ba7f73e735` have the identical Git tree `7cb86c6b369a0f1dc5c3125a3388efa4af82e7a4`. Run 45498114481 verifies both changes together. Its cancelled duplicate 45498114718 ran no jobs.

The surviving prior workspace has the #73 staging-cleanup checkout clean at `d5cf25c`, the native DAA partition checkout clean at #71, and the operator checkout clean at the validated #48 head `9132237`. The issuance checkout on #72 contains an uncommitted `consensus/src/processes/coinbase.rs` patch to reconcile reward-claim counts with settlement, including eligible zero-subsidy claims. Focused results for that patch were absent at recovery. These branch and working-tree states are the last recoverable checkpoint.

### November 25 verified continuation

| Draft | Exact head | GitHub Actions and named evidence |
| --- | --- | --- |
| [#76 Cross-stack integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/76) | `20eb29f66af594c99ff728f0da4434863755afc4` | [45622780578](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622780578), 8/8. Native job 139235203390 checked out `85f1a795` and passed 42 authority, 1,613 workspace and 32 no-asm tests. Named results cover native DAA reopen and relay, interrupted pruning recovery and reorganization, guarded nonzero publication recovery after process exit, duplicate developer releases and durable readiness closure. |
| [#77 Issuance claim counts](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/77) | `6e729a8f32f6108a1493d726fd54a740e451d202` | [45622845484](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622845484), 8/8. Native job 139235416536 passed the expanded coinbase/RocksDB regression, 36 authority, 1,602 workspace and 32 no-asm tests. |
| [#78 Operator evidence](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/78) | `b6662a96d2f2d3e12585860f5bc92f68fed5875c` | [45622905196](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622905196), 8/8. The offline evidence verifier was exercised locally against clean and changed artifacts. The scoped CI proves the source matrix. |
| [#79 Startup cleanup](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/79) | `c570592670398b28e3f45e58890cb7c1df866c71` | [45623057634](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45623057634), 8/8. Native job 139236001844 passed the dedicated legacy persisted-state cases, the live interruption/retry/reorg case, 40 authority, 1,606 workspace and 32 no-asm tests. The legacy-state test reconstructs durable states at the old write boundaries and reopens the database. |
| [#80 Native partition/rejoin](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/80) | `c3aac243f16943bb9dd7dea32f2e380892863805` | [45623236895](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45623236895), 8/8. Native job 139236627124 passed three real native-PoW tests, including the three-daemon competing-branch partition, rollback and renewed relay. It also passed 38 authority, 1,579 workspace and 32 no-asm tests. |

| [#81 Local override preflight](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/81) | `2935e49f8566dd5a1940573f8d07a86866db7a61` | Rejects effective zero block times or sample rates before daemon initialization changes disk state. [45707440494](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707440494) pending. |
| [#82 Operational integration](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/82) | `406535f2e0354d76a506307e75ae61fb8bad16ea` | Stacks #77, #79 and #80 on green #76. Exact source tree `86d718a843b27fde0e6967c698d85667efe57447` passed formatting, diff and workflow structure checks. [45707467797](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45707467797) pending. |

The new session's executor does not contain the unpushed multi-page rehearsal, local health probe, transaction workload or config preflight. The handoff's recorded published heads were recovered directly from GitHub. Continue those four distinct assignments on new branches. Use #79 to cover a real multi-page retry under the repaired cleanup path and #76 to integrate the green #77, #79 and #80 changes. A crash/reopen case through the actual management cleanup path remains a distinct follow-up to the legacy-state fixture. Preserve launch and issuance activation gates.

### November 25 resumed engineering

| Draft | Exact head | Parent and validation status |
| --- | --- | --- |
| [#76 cross-stack validation](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/76) | `20eb29f66af594c99ff728f0da4434863755afc4` | Combines #71, #72, #75 and the #73/#74 tree. The merged source was reviewed for unchanged native relay, inactive codec and committed recovery behavior. [New combined matrix 45622780578](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622780578) running. |
| [#77 issuance claim count](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/77) | `6e729a8f32f6108a1493d726fd54a740e451d202` | On #72. Recovered prior work and passed focused coinbase/RocksDB test locally. [45622845484](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622845484) running. |
| [#78 operator evidence provenance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/78) | `b6662a96d2f2d3e12585860f5bc92f68fed5875c` | On #48. Local fixtures verify source, daemon and evidence hashes and detect modified artifacts. [45622905196](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45622905196) running. |
| [#79 abandoned staging cleanup](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/79) | `c570592670398b28e3f45e58890cb7c1df866c71` | On #73. Clears the staging pointer and entry atomically and handles incomplete cleanup from older versions. Local focused compilation and [45623057634](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45623057634) underway. |
| [#80 native partition/rejoin](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/80) | `c3aac243f16943bb9dd7dea32f2e380892863805` | On #71. New loopback real-kHeavyHash competing-branch test passed locally in 5.57 s. Confirm Actions on this head. |

The multi-page pruning retry case is staged locally at `11a50d1505dbcd9826acf226cb1e0b0347e35d8f` on #73. Its exact new test is `authority_integration_preview_retries_multi_page_pruning_utxos`. It reuses the verified exporter/receiver fixture with a larger chain and verifies the source actually exceeds one wire page before interrupting. Local focused execution is underway. Publish on a separate #73 draft once its result is understood.

### Next executable gates

1. Finish and validate the recovered settlement count patch on a separate draft based on #72. Record the exact focused and full CI evidence.
2. Make abandoned staging cleanup atomic and allow restart to recover a legacy staging pointer whose entry or directory has already been removed. Then cover a real multi-page pruning transfer followed by retry.
3. Test competing native-work branches during a live loopback partition, then verify selection, convergence and continued relay after reconnection. Reuse the #71 source and distinguish it from the earlier simulated-PoW partition and idle-follower link-loss checks.
4. Bind operational native peer recovery evidence to its source and effective configuration, then stage cross-stack validation of new code once its isolated results and dependencies are known.
5. The owner still needs to approve the final network identity/profile, issuance commitment carrier, migration and fresh-pruned-receiver bootstrap trust. Keep public activation and production issuance disabled. Preserve the 29B KCADE cap and 21B main / 8B tail budgets.

## November 24 parallel Phase 6 continuation

| Existing draft | Exact head | Actions at checkpoint |
| --- | --- | --- |
| [#67](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/67) · [45458883349](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883349) | `89574d460bd87eb038e2aefb841175389fb43743` | 8/8. The named independent issuance vectors and both replay endpoints passed in native job 138675767413. 1,600 workspace tests passed. |
| [#68](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/68) · [45458954460](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458954460) | `da7c931922bad9078f26f06ebcdc8de59b2de0c3` | 8/8. The committed-position test and production-path exporter reopen passed in native job 138675949106. 1,603 workspace tests passed. |
| [#70](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/70) · [45469685474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45469685474) | `bbbee69ca52c08c38276e8648fe1ac18e08d53cc` | 8/8. Native job 138713943899 names the committed-position guard and fixed-point reorganization/reopen as PASS. It also passed all 39 authority cases, all 1,603 workspace tests and all 32 no-asm tests. |

| New draft | Exact head and base | Scope and focused result |
| --- | --- | --- |
| [#71](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/71) · [45473741398](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473741398) | `9d91cf60ebf38ca68cf8a54395c532bdb53704c8` on #66 | Reopen the native-PoW daemon after retarget, independently calculate its next rolling target, derive work while offline, and relay it after reconnection. Named local test passed in 5.70 seconds. |
| [#72](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/72) · [45473757936](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473757936) | `396599fffea16d5d28e5498bb49defa4c571a7c9` on #67 | Inactive replay rejects a duplicate developer release keyed by settlement block, including repeated block hashes at different indexes. Temporal fixture verifies earlier releases cannot spend later accrual. Ten focused Rust tests passed. |
| [#73](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/73) · [45473758359](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473758359) | `acef2d08270a54133b997377e1dec3fea4e87b59` on #70 | Disconnect after a real nonempty P2P pruning UTXO write. Assert no active promotion before retry and after reopen, then require exact state equality and real reorganization with a second offline reopen. Named local test passed in 17.41 seconds. |
| [#74](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/74) · [45473814935](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473814935) | `cc45a08fc6d06f367464420925548efbc8bd166b` on #70 | The receiver rejects pages larger than the existing 1,000-entry exporter limit before converting UTXO entries. Full and empty pages preserve existing behavior, and missing row fields remain invalid. Both focused Rust tests passed. |
| [#75](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/75) · [45473893184](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45473893184) | `a583b3deb377aebe7aed741cb3336f026054be84` on #62 | A final evidence recheck failure closes readiness durably, preserves prior authority state, revokes sibling leases and requires fresh verification after reopening. Focused RocksDB regression passed. |

**Engineering continuation.** The #67, #68 and #70 matrices are complete at their listed exact heads. Require full Actions on #71–#75. [Validation head 84589bb](https://github.com/DeepFriedBlockchains/KascadeNetwork/commit/84589bb306b1f3657df87e8f146fa73e4930367f) combines #73 and #74 with their original trees preserved. The combined P2P interruption, retry, fixed-point reorganization and offline reopen passed locally in 18.56 seconds. Use `validation/phase6-pruning-page-and-retry-20261124` for continuation and require combined authority CI after both drafts validate. No completed job failed in the new matrices at handoff. Checks passed so far were #71 4/8 (native 138728321137), #72 4/8 (native 138728387768), #73 2/8 (native 138728528994), #74 2/8 (native 138728564101), #75 1/8 (native 138729008160). Keep production issuance and public launch disabled.

**Owner decisions.** The [fresh-receiver design](docs/development/PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md) recommends evaluating a versioned selected-parent commitment in the selected child's coinbase. A header carrier reduces inclusion-proof work and has broader header, miner and serialization effects. An accumulator still requires a consensus-validated carrier. Full settlement replay supplies independent evidence and has higher bootstrap cost. An approved migration anchor avoids full replay and needs explicit historical audit and reorganization rules. Selection, activation, bootstrap trust and the network profile still need approval. The monetary direction remains the 29B KCADE cap with 21B primary and 8B finite tail budgets.


## November 24 Phase 6 recovery and acceptance

| PR and Actions | Exact head | Result | Focused evidence |
| --- | --- | --- | --- |
| [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) · [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865) | `6df90717b0fdc41a6f85bc0bbb6a25709d688aa5` | 8/8 | Nonzero guarded publication and reopen |
| [#58](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/58) · [45241630500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500) | `42d6937163511280276964400e0bf8fab1844664` | 8/8 | Guarded exit 71/72 and complete native retry |
| [#59](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/59) · [45241635579](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241635579) | `c327d527b6dc9c586048ef4450cb57bc26899de7` | 8/8 | Live retained-checkpoint replacement |
| [#60](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/60) · [45243944211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243944211) | `068c49622d572f39f6a64fa99322b12ca3f05a23` | 8/8 | Inactive v1 codec and context |
| [#61](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/61) · [45251578576](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45251578576) | `d8a6c833420f751a204aa18055806267af00a586` | 8/8 | Live developer accrual and release replacement |
| [#62](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/62) · [45243940522](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522) | `b99dd88562ac810e2ecb9090ddd345a9b0e233eb` | 8/8 | Fresh-process lost-ack retry exit 0 |
| [#63](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/63) · [45254033624](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45254033624) | `3cb3d7f79504c850811382a715e6d5dc6b9924cb` | 8/8 | Live pruning eligibility and selected-child authentication |
| [#64](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/64) · [45251521642](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45251521642) | `ab3835b056c5f90b3a2e3a0793ff5039f95e9f18` | 8/8 | Independent settlement and inactive field |

[#58 first attempt](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500/attempts/1) lost a separate abrupt-exit worker before it reported an exit code. The exact native retry passed all 1,600 workspace tests, 38 authority cases and 32 no-asm cases. The worker signal from the first attempt is unknown. The crash artifacts show [#58 exit 71/72](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500/artifacts/14858151003) and [#62 exit 71/72/0](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522/artifacts/14854234556). Before commit the reopened checkpoint had 2 claims, 3,000 minted, 2,850 paid and 150 deferred, with readiness closed. At and after commit, the reopened ready checkpoint had 4 claims, 6,000 minted and paid, and no liability. The retry verified the complete checkpoint remained identical.

[#65 original run](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45252226323) failed the named exporter-to-receiver fixture at exact pruning UTXO equality: 180 rows captured on the exporter and 198 on the receiver. The virtual UTXO, SMT and shielded evidence agreed. The pruning processor publishes the point before adding its per-block UTXO diffs. The former export API checked the advertised hash around the read while the row set was still advancing. Two separate changes address the observed boundary: a commitment-based producer wait in #65 and an exporter API check in #68 under the same metadata lock as the row read.

| Active draft and Actions | Exact head | Base | Milestone |
| --- | --- | --- | --- |
| [#65](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/65) · [45458788829](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458788829) | `81a8feb6c74dbbf1ac030c74782954b7fc423c9a` | #63 | Waits for a point-scoped full UTXO MuHash before fixture import. |
| [#66](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/66) · [45458883372](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883372) | `79fe770b6f50e5d471b4d2b0ce99c963160fe858` | #21 | Two-node sampled DAA crossing with native PoW and exact target rejection. |
| [#67](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/67) · [45458883349](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883349) | `89574d460bd87eb038e2aefb841175389fb43743` | #64 | Inactive replay endpoint binding and cross-checked v1 commitment vectors. |
| [#68](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/68) · [45458954460](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458954460) | `da7c931922bad9078f26f06ebcdc8de59b2de0c3` | #65 | Rejects pruning UTXO chunks while the committed table position lags or the table is unstable. |
| [#69](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/69) · [45458953963](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458953963) | `720ee756dc582a7f7425e77947087e93a033b275` | #62 | Verifies an abandoned worker cannot persist a different readiness point and records status. |

[Draft #70](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/70) advances the staged fixed-point branch `cc709a3a15f2fd6f2ce6bc568ecb87ac961b8ab8` with the #65 fixture correction and a second offline reopen after branch replacement. Exact head `ee2a81d9c008aadc40fdd8bf6022a818ea862db5`, [run 45459339790](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45459339790). Require proof-based promotion, two accepted branches above shared finality, retained evidence equality after convergence, and durable same-directory reopen before peers can repair state.

#67's exact issuance module passed 8 local Rust tests. Its regression rejected the foreign-genesis path that the previously published inactive derivation accepted. Full workspace CI is separate.

**Next executable checks.** Validate #65 and #68 against the real exporter fixture, then run the corrected fixed-point reorganization branch. Verify the named DAA, inactive-issuance and process-exit tests plus their full matrices. Use the [classified acceptance checklist](docs/development/PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md#november-24-classified-testnet-gates) to distinguish isolated evidence from approval and workload gaps. Fresh issuance publication from pruned receivers requires an approved carrier, activation/migration and independent bootstrap verification. The 29B KCADE cap, 21B main and 8B tail budgets remain the policy inputs. The curve and tail duration require approval. Keep production issuance and public launch disabled.


### Verified native results and combined validation

| Draft | Tested head | Actions | Observed evidence |
| --- | --- | --- | --- |
| [#65](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/65) | `81a8feb6` | [45458788829](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458788829) · 8/8 | Named exporter/receiver reopen passed, with 38 authority cases, 1,602 workspace tests and 32 no-asm tests. |
| [#66](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/66) | `79fe770b` | [45458883372](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883372) · 8/8 | `authority_integration_native_pow_relays_across_difficulty_retarget` passed in both native PoW and authority selections. 1,579 workspace tests passed. |
| [#67](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/67) | `89574d46` | [45458883349](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458883349) · 7/8 | Eight focused local Rust tests passed. The CI authority step passed, and its workspace step was running at checkpoint. |
| [#68](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/68) | `da7c9319` | [45458954460](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458954460) · 7/8 | The focused production query regression passed locally. The CI authority step passed, and its workspace step was running at checkpoint. |
| [#69](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/69) | `720ee756` | [45458953963](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45458953963) · 8/8 | Both named abrupt-worker readiness tests passed. The native job passed 1,600 workspace and 32 no-asm tests. |
| [#70](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/70) | `ee2a81d9` | [45459339790](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45459339790) · 8/8 | Named fixed-point P2P reorganization and second offline reopen passed, with 38 authority and 1,602 workspace cases. |

#70 now stacks on #68. Its fast-forward head `bbbee69ca52c08c38276e8648fe1ac18e08d53cc` combines the production pruning-UTXO guard and the verified fixed-point reorganization fixture without changing either implementation. [Combined PR run 45469685474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45469685474) started. The superseded duplicate is 45469682749. Retain the individual 8/8 evidence above and require the named regression and full matrix on the combined head before considering integration. Read the final #67 and #68 native logs for the independent replay vectors and committed-position test.

The exact combined head passed `daemon_integration_tests::authority_integration_preview_reopens_imported_pruning_evidence` locally in 13.79 seconds. It used the guarded exporter, adopted the replacement branch above the common finality point, and matched the retained and virtual state after a second offline reopen. This was the transparent Simnet fixture with an authenticated empty shielded state.

## Exporter-to-receiver recovery session

The original stacks are #56 → #57 → #58 → #62, #57 → #59 → #61 → #63 → #65, and #53 → #60 → #64. All implementation PRs remain drafts. #65 also contains #61's DAA fixture correction.

| Exact implementation | Actions evidence at checkpoint |
| --- | --- |
| `6df90717b0fdc41a6f85bc0bbb6a25709d688aa5` | [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) · [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865). Both guarded-publication regressions, the authority selection and workspace tests passed. Native follow-up steps were still running. |
| `42d6937163511280276964400e0bf8fab1844664` | [#58](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/58) · [45241630500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500). The focused guarded subprocess regression and 38 authority tests passed. Workspace run passed 1,599 tests and failed the separate `interrupted_import_remains_closed_after_abrupt_writer_exit` when that worker terminated without an exit code. Only the failed job was requested again. |
| `c327d527b6dc9c586048ef4450cb57bc26899de7` | [#59](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/59) · [45241635579](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241635579). `live_reorg_replaces_selected_nonzero_issuance_checkpoint` passed. Workspace pending at checkpoint. |
| `068c49622d572f39f6a64fa99322b12ca3f05a23` | [#60](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/60) · [45243944211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243944211). Both focused codec and context tests passed. Workspace pending. |
| `d8a6c833420f751a204aa18055806267af00a586` | [#61](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/61) · [45251578576](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45251578576). Replaces [45243938117](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243938117), where the live test required zero liability one DAA tick too soon. The isolated Simnet test now uses interval 2 and checks DAA scores 0, 1, 2, and 3. Native regression pending. |
| `b99dd88562ac810e2ecb9090ddd345a9b0e233eb` | [#62](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/62) · [45243940522](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522). Fresh-process retry passed with the same authenticated checkpoint and durable readiness. Workspace pending. |
| `0d869e26697327d7de85c00e1cbb098bdf9c4236` | [#63](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/63) · [45245782694](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45245782694). `live_pruning_candidate_authenticates_selected_child_after_reorg` passed. It uses a prospective point during fork selection and checks pruning eligibility after the replacement. Workspace pending. |
| `ab3835b056c5f90b3a2e3a0793ff5039f95e9f18` | [#64](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/64) · [45251521642](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45251521642). The named settlement regression passed in [45244523522](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45244523522); Lints required one rustfmt change. Replacement pending. |
| `c9380e742f74c11d67f6ff3726f8874ec0aed5bc` | [#65](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/65) · [45252226323](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45252226323). New real exporter-to-receiver fixture. Native pass has yet to be observed. |

[Artifact 14853226458](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500/artifacts/14853226458) prints exit 71 before the guarded commit and exit 72 after commit before acknowledgement. The parent asserted the old checkpoint and closed marker before commit, then the successor checkpoint, paid/deferred conservation, and durable ready marker after commit. [Artifact 14854234556](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522/artifacts/14854234556) prints 71, 72, and retry 0. Its test requires the identical full authenticated checkpoint and ready marker after the retry, including claim count, minted amount and released liability.

**Next executable checks.** Inspect the #61 developer test and #64 formatting and settlement steps on their replacement heads. Inspect #58's failed-job retry for an actual worker status, then the remaining native jobs. On #65, inspect the named `authority_integration_preview_reopens_imported_pruning_evidence` result in the Phase 6 authority step and debug the first failed evidence comparison if present. The [runbook](docs/development/PHASE6_LOOPBACK_OPERATOR_RUNBOOK_2026-11-23.md) and [acceptance checklist](docs/development/PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md) describe the fixture and the remaining recovery conditions.


## Live pruning eligibility and inactive settlement verification — November 23, 2026

| PR | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#56](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/56) | `ac9bacc0d60b2fc6c9e13f72aeb6626c1ead0b10` | [45236866011](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236866011). 8/8. Native log confirms `authority_integration_reopen_before_promotion_rejects_incomplete_evidence`, 36 authority, 1,598 workspace, 32 no-asm. |
| [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) | `6df90717b0fdc41a6f85bc0bbb6a25709d688aa5` | [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865). 7/8. Exact two-test step passed after dropping the database handle. The full native job is still running. |
| [#58](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/58) | `42d6937163511280276964400e0bf8fab1844664` | [45241630500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500). 6/8. Exact real subprocess step passed, whose assertions require exit 71 before commit and 72 after commit. Artifact [14853226458](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500/artifacts/14853226458) exists. Raw exit lines await closed native log. |
| [#59](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/59) | `c327d527b6dc9c586048ef4450cb57bc26899de7` | [45241635579](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241635579). 6/8. Exact live reorganization step running. |
| [#60](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/60) | `068c49622d572f39f6a64fa99322b12ca3f05a23` | [45243944211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243944211). Replacement native toolchain starting. Original run 45242682557 failed E0308: Hash::as_bytes returns the array by value; encoder now borrows it for copy_from_slice. |
| [#61](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/61) | `9bb003af4ddcc653e121a50fe7e4df213c46fe30` | [45243938117](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243938117). Replacement 2/8; named developer-release reorganization step running after exact rustfmt correction from 45242939737. |
| [#62](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/62) | `b99dd88562ac810e2ecb9090ddd345a9b0e233eb` | [45243940522](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243940522). Replacement 1/8; native queued after exact rustfmt correction from 45243050035. Retry exit 0 remains unverified. |
| [#63](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/63) | `2bfcfe3cd9179d06b6d1291806391c714bbab180` | [45244354526](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45244354526). Queued. Named live pruning-candidate and selected-child prerequisite. |
| [#64](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/64) | `2686b0aed000deb397fedb13b17a11058df918c2` | [45244523522](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45244523522). Queued. Exact production-builder settlement fixture extended with inactive field replay verification. |

#63 `test/phase6-live-pruning-child-eligibility-20261123` stacks on corrected #61. It uses accepted archival Simnet blocks and a test-only shortened depth. The shared non-genesis candidate crosses a finality sample, fails early pruning-depth eligibility, then meets depth after consensus selects the competing descendants. The production header/body coinbase check authenticates the new child, and live reachability rules disqualify the old child. Wrong selected parent, missing body and substituted transaction root are rejected. The test requires authority to remain unready without import. It does not advance a receiver through a committed import proof or atomically publish issuance.

The next joint integration test needs an actual exporter and isolated receiver. Keep a legitimate pruning point as the common ancestor of competing descendants; do not attempt a reorganization across the committed pruning/finality boundary. Use the exporter pruning proof and trusted data, materialize UTXO, SMT and shielded import rows on the receiver, validate the selected child from authenticated headers, and resolve the committed virtual base at the pruning point. The receiver needs independent local replay evidence for issuance before guarded publication. Then reorg descendants through consensus and verify captured off-chain child evidence never authorizes restart, while constant-point checkpoint state is kept when it remains compatible. A competing prospective pruning point may be tested before promotion; once committed, use the existing eligibility/finality rules rather than forcing a point change.

#64 `test/phase6-inactive-field-verified-settlement-20261123` stacks on corrected #60. Its inactive interface independently derives a bounded checkpoint from the configured network, selected path, claims and developer releases. The production coinbase builder's test checks that canonical bytes agree with real payout attribution and rejects equal-total digest substitution, wrong network, off-selected releases and duplicates. No block, production import or bootstrap path calls the interface.

**Next executable checks.** Inspect #57 and #58 raw native logs for individual PASS and process exit lines once available. Verify #59, corrected #60–#62 and #63–#64 named focused steps. Add the real exporter-to-receiver pruning fixture above and exercise guarded publication from valid verified state, including the fixed pruning point across a selected-child change. Review carrier, activation, migration and bootstrap trust with the owner before any active code. Production issuance and public launch remain disabled.

## Extended validation, live developer releases and commitment codec — November 23, 2026

| PR | Exact head | Actions evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690). 8/8. Named `reconciles_coinbase_settlement_checkpoint_with_red_fees_and_deferred_payout` and 36/36 authority tests passed. All three workspace gRPC server tests passed; 1,595 workspace and 32 no-asm passed. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761). 8/8. Named `commits_developer_releases_without_minting_or_replaying_stale_liabilities` and coinbase settlement test both passed. |
| [#54](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/54) | `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5` | [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768). 8/8. Named `issuance_checkpoint_replay_anchor_rejects_substitution_and_reorg` and coinbase settlement both passed; 1,597 workspace and 32 no-asm passed. |
| [#55](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/55) | `4b6e14876e0d84eee86241c8e58db2843d63a87b` | [45235594152](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45235594152). 8/8. Native log confirms `retained_issuance_anchor_recovers_after_pruning_and_rejects_stale_selection`; 36 authority, 1,598 workspace and 32 no-asm passed. |
| [#56](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/56) | `ac9bacc0d60b2fc6c9e13f72aeb6626c1ead0b10` | [45236866011](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236866011). 7/8. Guarded readiness and native authority both passed; workspace running. |
| [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) | `6df90717b0fdc41a6f85bc0bbb6a25709d688aa5` | [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865). 5/8. Both named tests still running after fixture RocksDB handle correction. |
| [#58](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/58) | `42d6937163511280276964400e0bf8fab1844664` | [45241630500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500). 4/8. Named process-exit test in progress; log artifact not yet verified. |
| [#59](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/59) | `c327d527b6dc9c586048ef4450cb57bc26899de7` | [45241635579](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241635579). 3/8. Native runner preparing the live Simnet test. |
| [#60](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/60) | `97678d76f6b96e986e06b150b1b4e695972b8a99` | [45242682557](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45242682557). Pending. Focused selectors name both codec tests. |
| [#61](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/61) | `51b0022b78404482793493d7db6c9cb24ba0aaf2` | [45242939737](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45242939737). Queued. Focused selector names the developer-release live Simnet test. |
| [#62](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/62) | `6ae99782a256f540e25ef200a1f3230e842fafc6` | [45243050035](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45243050035). Queued. Runs real three-worker crash/acknowledgement retry test and uploads the log. |

#60 `feat/phase6-inactive-issuance-field-20261123` stacks on #53 and adds a canonical 34-byte codec only. Dedicated tests exercise exact lengths, unsupported versions, wrong network, wrong selected parent/pruning point and equal-total claim substitution. No block builder, parser, activation predicate or network has selected the proposed field.

#61 `test/phase6-live-developer-release-reorg-20261123` stacks on #59. It checks the actual accepted coinbase payments and the persisted developer accrual on the original and competing Simnet branches, rejects stale/duplicate releases, and checks deferred liability and retained checkpoint after reopening. Guarded readiness remains a separate gate. The live fixture's short branches do not advance a real pruning point and lack locally validated proof, committed point-scoped pruning UTXO/SMT/virtual evidence and a real selected child at that point. A test that marks the sink ready or rewrites the selected index would not exercise production verification. Build an actual pruning-boundary fixture exposing those inputs and complete selected-chain settlement evidence before connecting it to `finish_verified_ibd_import_with_issuance`.

#62 `test/phase6-publication-retry-after-lost-ack-20261123` stacks on #58. A new process repeats the same evidence after the postcommit/preacknowledgement exit. A final reopen must show the identical full checkpoint and ready bit. This tests accounting idempotence under retry. The same-db fixture seeds selected ancestry; a live pruning-boundary reorg plus subprocess reopen is the remaining joint gate.

**Next executable checks.** Confirm completed native logs for #56–#57. Inspect #58 exit codes 71/72 and artifact; inspect #62 for the additional retry exit 0 and final reopen. Validate both live reorg steps #59/#61 and codec tests #60; fix actual failures first. Build the real pruning-point fixture independently of activation. Review the [owner decision record](docs/development/PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md) before adding any active wire field or bootstrap rule. Production issuance stays disabled.

## Process-exit recovery, selected-chain reorganization and issuance decision — November 23, 2026

| PR | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690). 7/8. Focused settlement/RocksDB and serial native authority steps passed, including the suite with the port-reservation fix. Workspace pending. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761). 8/8. Native job 137895693585 logs individual PASS for `commits_developer_releases_without_minting_or_replaying_stale_liabilities` and `reconciles_coinbase_settlement_checkpoint_with_red_fees_and_deferred_payout`. |
| [#54](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/54) | `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5` | [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768). 7/8. Focused `issuance_checkpoint_replay_anchor_rejects_substitution_and_reorg` and broader authority passed. Workspace pending. |
| [#55](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/55) | `4b6e14876e0d84eee86241c8e58db2843d63a87b` | [45235594152](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45235594152). 7/8. Focused `retained_issuance_anchor_recovers_after_pruning_and_rejects_stale_selection` and broader authority passed. Workspace pending. |
| [#56](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/56) | `ac9bacc0d60b2fc6c9e13f72aeb6626c1ead0b10` | [45236866011](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236866011). 7/8. Focused `authority_integration_reopen_before_promotion_rejects_incomplete_evidence` passed the actual bundled readiness path; broader native authority running. |
| [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) | `6df90717b0fdc41a6f85bc0bbb6a25709d688aa5` | [45241439865](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241439865). 2/8. Initial corrected-format [45238570956](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45238570956) native job 137916863730 ran both tests and found an in-process RocksDB lock at reopen. The fixture's selected-chain store retained a database Arc. Released that handle before reopening. Replacement native started. |
| [#58](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/58) | `42d6937163511280276964400e0bf8fab1844664` | [45241630500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241630500) running. Initial [45241106989](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241106989) failed rustfmt. Exact formatting corrected and #57's scoped-handle fix carried forward. |
| [#59](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/59) | `c327d527b6dc9c586048ef4450cb57bc26899de7` | [45241635579](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241635579) pending. Initial [45241337716](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45241337716) failed rustfmt. Exact formatting corrected and #57's reopen fix carried forward. |

#58 `test/phase6-nonzero-guarded-publication-exit-20261123` and #59 `test/phase6-live-issuance-reorg-20261123` are separate drafts on #57. #58 arms a test-only process exit on each side of the actual guarded RocksDB readiness batch. It retains a prior checkpoint with developer liability, exits before commit, reopens to check the prior state, and retries with exit after commit but before acknowledgement. It checks the successor's unique claim totals, completed release and durable readiness after reopen. The production coinbase builder supplies settlements. The seeded chain proof limits it to testing the publication and persistence boundary. The workflow uploads `nonzero-pruning-process-exit.log`.

#59 builds competing valid Simnet blocks with proof-of-work bypass, waits for consensus selection, derives nonzero claims from verified coinbase outputs on both branches, and tests stale checkpoint rejection and replacement through RocksDB reopen. It uses the real selected index and the retained checkpoint store. It excludes developer payouts and guarded readiness, which must still be tested together under live consensus reorganization.

The [issuance decision record](docs/development/PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md) defines a proposed 34-byte versioned coinbase field after the miner script and before miner extra data. It separates selected-child inclusion from full validation of accounting and records the approval decisions for activation, genesis, migration, bootstrap trust and old miners/peers. There is no carrier code or consensus activation.

**Next executable gates.** Read the completed native logs for #52 and #54–#56; confirm exact passes and summary. On #57 check both named tests after the handle fix. On #58 inspect the two exit codes and published artifact, and on #59 inspect the live selected-chain result. Then add a live reorganization with developer release and readiness invalidation once the complete actual settlement replay/guarded pruning fixture is available. Wait for owner approval of the commitment layout and bootstrap assumptions before implementation of the active carrier or a fresh receiver. Keep production issuance and network launch disabled.

## Nonzero guarded publication and fresh-receiver design — November 23, 2026

| PR | Exact implementation head | CI and outstanding evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690). 7/8. Named settlement + RocksDB step passed; serial native authority suite running, including port-reservation validation. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761). 7/8. Combined release and coinbase step and broader native authority step passed. Workspace tests running; individual PASS lines await completed native logs. |
| [#54](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/54) | `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5` | [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768). 7/8. Replay-derived issuance step passed; native devnet-prealloc building. |
| [#55](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/55) | `4b6e14876e0d84eee86241c8e58db2843d63a87b` | [45235594152](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45235594152). 6/8. Retained replay across actual selected-index pruning step passed; native and Linux release running. |
| [#56](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/56) | `ac9bacc0d60b2fc6c9e13f72aeb6626c1ead0b10` | [45236866011](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236866011). 5/8. Focused production guarded-publication step running after formatting correction. |
| [#57](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/57) | `1450d6ac03843622487c3bf78b7840595bb777fd` | [45238570956](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45238570956). Replacement running with dedicated nonzero and zero-settlement selectors. Initial [45238200733](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45238200733) failed rustfmt on two expressions. Correction is formatting-only. |

#57 branch `feat/phase6-nonzero-pruning-publication-20261123` stacks on #56 (`feat/phase6-atomic-local-issuance-readiness-20261123`). It runs two production coinbase-builder settlement transitions through the opt-in guarded publisher. The seeded selected-index fixture covers a fee-only red block in each transition, accrued and later released developer liability, wrong network and stale replay path, substituted root, duplicate sources/releases, insufficient liability, abandoned import, committed RocksDB reopen, and stale checkpoint rejection after selected-index replacement. It still requires a running CI result, and the seeded index replacement is narrower than a full consensus reorganization and atomic replacement. A process exit during nonzero publication remains to be exercised.

Fresh receivers need a consensus-authenticated issuance digest. The [design](docs/development/PHASE6_FRESH_RECEIVER_ISSUANCE_DESIGN_2026-11-23.md) compares carriage choices, documents canonical digest fields and independent evidence, and recommends reviewing a versioned selected-parent digest carried by the selected child's coinbase. No placement was added to consensus. The owner must approve the carrier, activation and what bootstrap or migration trust a fresh receiver can use. Production issuance and network launch remain disabled.

**Next executable gates.** Inspect completed #52 and #53 native logs for the authority/port outcome and individual named assertions. Confirm #54–#56 final matrices, then confirm both named #57 regressions. After #57 passes, exercise process exit before the nonzero readiness batch and actual consensus-driven stale owner replacement on the local route. Resolve the owner-controlled parameters, implement the approved commitment's full-validator path, then prove a fresh pruned receiver without a complete local replay prefix can verify and publish authenticated nonzero issuance atomically.

## Four-assignment validation and readiness publication — November 23, 2026

| PR | Exact head | Actions evidence |
| --- | --- | --- |
| #52 | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690), 7/8. Named settlement + RocksDB step passed. Native devnet-prealloc build running; serial authority and port result pending. |
| #53 | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761), 7/8. Combined selected-release/coinbase step passed; native devnet-prealloc running. Check the completed log for both exact PASS lines. |
| #54 | `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5` | [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768), 5/8. Exact focused replay/settlement step succeeded; native job and two builds still running. |
| #55 | `4b6e14876e0d84eee86241c8e58db2843d63a87b` | [45235594152](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45235594152), 3/8. Named retained replay pruning step running. Initial [45234797231](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234797231) failed rustfmt; correction is formatting-only. |
| [#56](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/56) | `ac9bacc0d60b2fc6c9e13f72aeb6626c1ead0b10` | [45236866011](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236866011) started, no jobs concluded at checkpoint. Initial [45236408364](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45236408364) at `b6466c5a823988ac7b1837588ce4c7c57940b8de` failed rustfmt. Correction is formatting-only. Branch `feat/phase6-atomic-local-issuance-readiness-20261123` stacked on the corrected #55. |

#56 stages the verified local pair at the first durable readiness write in `PersistentBranchAuthority::complete_import_with_staging`. The IBD completion reuses actual pruning proof, SMT, shielded, pruning UTXO and virtual UTXO verification, pins selected-chain ownership, and scopes writes to the authority namespace. The opt-in regression covers absent/substituted issuance, abandoned import and successful guarded completion and reopen. The fixture uses zero-issuance genesis replay, so crash and nonzero settlement/reorg publication still need the full integration case. The legacy path remains active with issuance disabled.

Reusable selected-child coinbase membership verifies stored header, selected parent and transaction Merkle root. No current field supplies consensus-validated cumulative issuance. The next consensus increment must validate a separate versioned issuance digest against applied settlement for the selected parent, independently of peer checkpoint bytes. Once the owner approves placement, activation and migration, an authenticated selected child can supply that digest for a fresh pruned receiver. Feed it into the same staged completion; prove nonzero claims/releases and readiness publish atomically after interruption, reopen and reorg. Do not activate the proposed 29B KCADE cap or public network here.

**Next:** Read exact named test and authority evidence on #52–#55, and the named guarded IBD test on #56. The updated [operator runbook](docs/development/PHASE6_LOOPBACK_OPERATOR_RUNBOOK_2026-11-23.md) and [acceptance checklist](docs/development/PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md) track local replay, fresh pruned authentication, atomic readiness and owner approval independently.

## Four-assignment Phase 6 checkpoint — November 23, 2026

- #52 [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690) 7/8, exact settlement step passed and native authority pending. #53 [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761) 6/8, focused step passed and individual named evidence pending completed logs. #54 [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768) active in focused replay step.
- #55 initial [45234797231](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234797231) failed rustfmt. Applied the exact formatting correction at `4b6e14876e0d84eee86241c8e58db2843d63a87b`. [Replacement 45235594152](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45235594152) active. No healthy workflow was restarted.
- Next safe increment: stage verified local issuance in the real guarded readiness publisher while leaving remote activation disabled. Fresh pruned receivers still lack a consensus-validated cumulative issuance commitment. Refactor reusable selected-child authentication and update existing acceptance evidence without claiming remote end-to-end success.

## Retained issuance anchor after selected-index pruning — November 23, 2026

| PR | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690), 7/8. Exact named settlement+RocksDB step succeeded. Native 137892269956 running descendant recovery before the serial authority suite. Port reservation remains pending full authority evidence. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761), 5/8. Combined selected-release and coinbase focused step succeeded. Native 137895693585 advanced into descendant recovery. Confirm both individual PASS lines from the completed job log. |
| [#54](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/54) | `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5` | [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768), running. Replaced initial [45233347194](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45233347194), which failed rustfmt on a replay fixture initializer. Only formatting changed. |
| [#55](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/55) | `878a8a32bc5c687688ddbe99471f0c03d01580bc` | [45234797231](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234797231) pending. Branch `feat/phase6-retained-issuance-anchor-20261123`, stacked on exact corrected #54. Dedicated retained-anchor pruning and reorg test configured. |

#55 stages the separately derived local replay anchor and its verified checkpoint in one authority batch. It reads them at one RocksDB sequence after pruning and verifies trusted genesis, point, cap and current selected ownership. Its regression rejects a checkpoint without an anchor, digest substitution, abandoned writes, wrong configuration, and stale selection after a committed reorg. It calls the actual selected-chain index pruning method before reopening RocksDB. There is no production readiness or reward activation in this change.

**Exact boundary:** Header transaction Merkle proofs bind a body, the accepted-ID root binds transaction identity, and the UTXO commitment binds the UTXO set. The current selected-child coinbase verifier reads the shielded state root. These fields do not supply a validated cumulative issuance digest. #55 helps a node that replayed locally before pruning. A fresh pruned receiver still needs an independently authenticated issuance commitment that consensus checked against actual payout/claim history. Pass that typed evidence to the existing guarded publisher and verify real issuance and pruning readiness publish atomically under rejection, interruption, reopen and reorganization. Proposed 29B KCADE cap (21B primary and 8B finite tail) and production issuance remain inactive.

**Next:** Read completed #52 and #53 native logs for the named test count and authority result. Check the exact focused tests on #54 and #55. If any bind error recurs, inspect the port/worker from the failed log before changing the fixture. No healthy workflow was restarted.

## Pruned receiver authentication checkpoint — November 23, 2026

- #52 and #53 remain active on [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690) and [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761). No reruns requested. Exact focused and authority conclusions are pending.
- #54 initial [45233347194](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45233347194) failed rustfmt on the replay fixture initializer. The formatting-only correction is `c1442b2d6f81cb9e7b9a470dd023154ffd755ef5`. Replacement [45234460768](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45234460768) active.
- The committed header binds the transaction Merkle root, accepted transaction IDs and UTXO commitment. The selected child verifier authenticates the shielded coinbase root. None currently authenticates a cumulative issuance checkpoint including claims and developer releases. The next safe increment will retain a local replay-derived anchor before pruning, scope it to the verified network, cap and selected pruning point, and recover the pair after the historical index prefix is removed. Production issuance remains inactive.

## Selected replay anchor and pending issuance validation — November 23, 2026

| PR | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690), 6/8 passed. The named settlement + RocksDB regression was running; the broader authority result, including port reservations, remains pending. No restart requested. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761), 2/8 passed. Focused selected developer release and coinbase settlement step running. No restart requested. |
| [#54](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/54) | `3863d17c66f7d237a965011a564164b9b3f8ac7f` | [45233347194](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45233347194) pending. Branch `feat/phase6-local-issuance-replay-anchor-20261123` stacked on exact #53. The focused step selects the new named replay/reorg regression and the actual coinbase settlement fixture. |

#54 computes an expected issuance digest from separately provided local consensus replay events and checks the full replay path against a pinned, same-database selected-chain guard. It checks both hash-to-index and index-to-hash, configured genesis and cap, and derives the canonical claim/release root and balances without reading the imported record. The store then verifies the candidate against that expected digest. The regression rejects equal-total source substitution and stale ancestry, verifies rejected/abandoned RocksDB batches and committed reopen, and replaces the stale checkpoint after a selected-chain reorganization. The existing coinbase fixture now feeds its attributed source claims into the replay-derived anchor before staging a reconciled successor.

**Next:** Inspect the exact named #52/#53 focused and completed authority logs. Verify #54's two-test replay and settlement step and matrix. The local replay interface requires trusted claim/release provenance and complete committed ancestry. A pruned receiver cannot use this as its remote proof: `import_shielded_readiness::authenticated_commitment` authenticates only the shielded state root. Consensus still needs a validated issuance commitment sourced from independently authenticated selected evidence or a verified replay checkpoint that can initialize a pruned suffix. Then pass the typed anchor to `commit_staging_promotion_checked` so the same guarded authority batch commits the real #51 settlement row and ready revision. Test invalid proof, interrupted publication, reopen and reorg in that path. The current replay regression uses a batch probe, and production readiness remains unproven. Proposed supply remains 29B KCADE (21B primary and 8B finite tail), with production issuance and public activation disabled.

## Local selected-chain replay checkpoint — November 23, 2026

- #52 `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` remains active on [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690). Four jobs passed; focused settlement, Clippy and the authority/port regression were still pending.
- #53 `844b9d8c2ad53733d314a33f74532dd216bebf5b` remains active on [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761). One job passed at the initial read; native setup was in progress.
- Investigation confirmed the current selected-child commitment extractor verifies the stored header, selected parent and coinbase Merkle membership, then extracts the shielded state root. The existing format supplies no independently verified issuance digest. The next increment will verify local replay against the committed selected index and compute the expected issuance digest separately from the restore candidate. Production issuance stays disabled.

## Selected developer payout commitments — November 23, 2026

| PR | Exact head | Actions evidence |
| --- | --- | --- |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690) running. Check, no-std, WASM check and WASM test passed. Rustfmt passed inside Lints; Clippy still running at the check. The named settlement+RocksDB step was executing; authority/port-reservation result pending. |
| [#53](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/53) | `844b9d8c2ad53733d314a33f74532dd216bebf5b` | Replacement [45232526761](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232526761) pending after two formatting-only fixes. Initial [45232322897](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45232322897) failed rustfmt. This is on branch `feat/phase6-issuance-developer-release-20261123`, stacked on exact #52 head. Focused issuance release and actual coinbase payout regressions pending. |

#53 uses the existing canonical root to commit a distinct developer-release leaf for a selected block. Source claims sort by source hash within each block, then the developer release spends available deferred liability. Releases add only to paid issuance. The verified path gates each event, duplicate/zero/overdrawn releases fail, and the release-enabled API rejects embedding prior payments in new source claims. Existing claim-only roots are preserved. The actual coinbase payout fixture derives its 330-unit release and checks its cumulative checkpoint against the prior liability. The dedicated core regression covers equal-total tampering, selected sibling replacement, duplicate releases, and payout at the issuance cap.

**Next:** Verify #52's exact named settlement+RocksDB focused result and the serial authority logs on 45231556690 before marking it validated. Verify #53's focused two-test step and matrix on 45232526761. The next production interface is an independently verified issuance digest in selected-chain evidence, separate from the current shielded coinbase root. The guarded promotion must consume that proof and stage the #51 checkpoint and readiness in the same authority batch. Run a real interruption/reopen/reorganization acceptance test with attributed source claims and a developer release. Production issuance stays disabled at the proposed 29B KCADE cap (21B primary and 8B finite tail).

## Developer release integration checkpoint — November 23, 2026

- #52 at `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` remains under [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690). Formatting and initial check passed at the first read; native setup and strict Clippy were still running. No reruns requested.
- The next increment binds developer release events to their selected block and settles them against existing deferred balances. It will stay on a separate stacked draft while #52 runs. Production issuance and public activation remain disabled.

## Eligible source attribution and integration port isolation — November 23, 2026

| PR | Exact head | Verified evidence |
| --- | --- | --- |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `232dca6e685cc7081537a8ed5e588f891bb05966` | [45216092833](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216092833), **8/8 passed**. The named `authority_integration_reopen_before_promotion_rejects_incomplete_evidence` ran in focused, authority and workspace phases. Eight focused, 36 authority, 1,592 native and 32 no-asm passed. |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `b34198310232e72f4289ceb8562ec22d9a05bf96` | [45216624402](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216624402), 6/8. Exact `reconciles_coinbase_settlement_checkpoint_with_red_fees_and_deferred_payout` passed 1/1 in 0.037 s. It rejects missing predecessors and altered settlement, retains the prior checkpoint across an abandoned write, and reopens the committed replacement. Rustfmt found fixture wrapping differences. The serial authority step failed on gRPC wildcard port 51898 with AddrInUse and then connection refused during the interrupted UTXO fixture. The prior run failed on a different P2P listener, so serial scheduling alone did not solve the test harness race. |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `2f8536199feeca31c3e5514dc11e6bc9f165fbfa` | Replacement [45231556690](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231556690) started. Initial [45231358062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45231358062) failed rustfmt on the new lease initializer; formatting-only fix pushed. Addresses both failures, adds source attribution and retains all test assertions. |

#47 tests the actual guarded publisher using a fixture issuance row and digest. An invalid selected-child body leaves the promotion marker, readiness and fixture row unpublished. A valid batch survives reopen. Corrupting the persisted fixture split then fails independent verification. Reorganization coverage is separate on [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720) and [#51](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45154174813); #47 does not prove real issuance replacement.

#52 derives unique, ordered eligible-source claims from the same coinbase inputs used to reconcile the actual template. A DAA-eligible blue or red becomes a claim; a non-DAA red only contributes recycled fees. The regression sends actual accrual-case claims through #50 derivation and the #51 RocksDB successor boundary. It separately checks the release of deferred developer funds on a payout block, without attributing that payout to a new reward source. The test harness allocates test listeners outside the runner's ordinary ephemeral range, probes the wildcard address used by the servers, and retains advisory port leases across cooperating test processes. The revised fixture and allocator remain pending CI.

**Next:** Read #52's named focused test and serial authority logs on 45231556690. Extend the canonical issuance representation with a selected-block developer-release event before deriving payout-bearing digests. Add an independently authenticated issuance commitment to the verified selected-chain evidence; the current selected-child payload verifier authenticates a shielded state root. The guarded publisher must then stage the real verified #51 checkpoint alongside the ready revision and prove rejection, interruption, restart and branch replacement in the production path. The proposed 29B KCADE (21B primary and 8B finite tail) and production issuance remain inactive.

## Atomic settlement checkpoint — November 23, 2026

- #47 at `232dca6e685cc7081537a8ed5e588f891bb05966` passed 8/8 in [45216092833](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216092833), including eight focused tests and the named guarded RocksDB reopen/publication case. Its issuance row and digest are fixtures. #45 supplies separate stale-branch coverage.
- #52 at `b34198310232e72f4289ceb8562ec22d9a05bf96` executed and passed the expanded settlement+RocksDB test in [45216624402](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216624402). Six of eight jobs passed overall. Lints found three formatting differences. The serial native authority stage failed on a gRPC AddrInUse followed by a refused connection, so serial execution did not eliminate the test-port race.
- Next work: reserve isolated test ports across test workers, and derive real eligible source claims from checked coinbase inputs with developer payout release accounted separately. Keep production issuance and pruning activation disabled.

## Validated recovery and reconciled checkpoint succession — November 23, 2026

| PR | Exact head | Verified evidence |
| --- | --- | --- |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `900ced8a25a2e704a93ee2f7c26b2d67147bd7ff` | [45153573720](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720), **8/8 passed**. Named stale-child/selected-branch case passed; seven focused, 36 authority, 1,592 native, 32 no-asm. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `232dca6e685cc7081537a8ed5e588f891bb05966` | [45216092833](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216092833) active on validated #45. Rustfmt passed; Clippy and focused guarded publication pending. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `8b1a015b5de1d4a533a2e793bd67e17b977b9ca6` | [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030), **8/8 passed**. The exact real Simnet eligible-red payout regression ran and passed; 39 authority, 1,582 native, 32 no-asm. |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `9132237fd4d2db68b3b4ed56cfa9d767c8990af6` | [45156258308](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308), **8/8 passed**. Focused live peer test passed in 5.202 s; 39 authority, 1,579 native, 32 no-asm. |
| [#50](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/50) | `e01899558e2e5746c8dd3650af42e997b9f282bc` | [45152422597](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152422597), **8/8 passed**. Named `derives_canonical_commitment_from_unique_selected_claims` passed in focused and workspace phases; three focused, 36 authority, 1,593 native, 32 no-asm. |
| [#51](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/51) | `ed32a290dff167b4c6e67397fe9080e5523833c0` | [45154174813](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45154174813), **8/8 passed**. Named stale-owner/reopen storage case passed; 36 authority, 1,594 native, 32 no-asm. |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `b34198310232e72f4289ceb8562ec22d9a05bf96` | [45216624402](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216624402) pending. Earlier `44da60377d7ef13557264f25e41f908797cacee0` passed the named coinbase settlement regression on [45156348643](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156348643). That matrix ended 7/8 on a P2P listener bind AddrInUse, then a descendant-connectivity assertion in the authority stage. |

#48's live trial kept the follower process running through a 442 ms link interruption. Reconnect took 105 ms, catch-up of three blocks took 106 ms, and offline reopen took 315 ms. Reconnected A/B/C RSS was 127,377,408 / 121,192,448 / 121,081,856 bytes. Final tip `472c7730ccc0cf9ce78764105f2902159f17f180740859f6429dfed290c7d02b`. [Artifact with metrics and process logs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308/artifacts/14809571770)

#52 now stages a reconciled successor only after reopening and authenticating the stored predecessor under the same pinned selected-chain view. It verifies consecutive indices and equal cap, compares the exact actual coinbase-issued/paid/deferred transition, and rejects the entire batch on failure. The expanded focused test checks missing predecessor, a modified paid/deferred split even when given its matching fixture digest, an abandoned write through reopen, and a committed successor through another reopen. Its anchor and selected path are fixtures. The proposal is not connected to active payout or readiness rules. The same commit serializes the focused authority suite's workers after the previous bind collision while retaining all checks. The probe-then-bind fixture allocator still exists and repeat collisions would need a test-port reservation fix.

**Next:** Inspect #52's expanded settlement+RocksDB focused test and native authority result, and #47's guarded proof/batch result. To connect actual consensus issuance, expose verified per-source reward attribution for selected blocks and represent release of prior deferred payouts separately from new source subsidies. Then derive the checkpoint root, authenticate its expected digest through verified selected-chain commitment evidence, and stage it with the real readiness revision under the same guarded promotion batch. Prove interrupted publication, restart and branch replacement using the real integration path. Production issuance and public activation remain disabled. The proposed cap is 29B KCADE, 21B primary and 8B finite tail.

## Settlement and recovery checkpoint — November 23, 2026

- All eight jobs passed on #45 [45153573720](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720), #46 [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030), #48 [45156258308](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308), #50 [45152422597](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152422597), and #51 [45154174813](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45154174813) at the exact heads in the table below.
- #47 advanced onto validated #45 at `232dca6e685cc7081537a8ed5e588f891bb05966`, [45216092833](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45216092833) active.
- #48's focused native peer test reconnected in 105 ms, caught up three blocks in 106 ms, and reopened offline in 315 ms. Reconnected A/B/C RSS was 127,377,408 / 121,192,448 / 121,081,856 bytes. [Resource and daemon artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308/artifacts/14809571770)
- #50 ran and passed the named canonical-claim regression, with three focused tests and 1,593 native tests. #46's real payout reopen and #51's authenticated RocksDB reopen/reorganization regressions both ran and passed.
- #52's named settlement case passed on `44da60377d7ef13557264f25e41f908797cacee0` in [45156348643](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156348643). The native authority step failed because its P2P listener hit AddrInUse during descendant recovery; its failed connectivity assertion followed that bind failure. The next increment checks the reconciled transition against the prior persisted checkpoint before staging a replacement. Production issuance remains disabled.

## Settlement reconciliation and validation — November 23, 2026

| Draft | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `0b89ddcd462fe5dcd526ee42ef5cc25d296293d4` | [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224), **8/8 passed**. Both focused CLI cases, usable per-node resource samples, 39 authority, 1,579 native and 32 no-asm passed. [Metrics artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224/artifacts/14805407543). |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `9132237fd4d2db68b3b4ed56cfa9d767c8990af6` | [45156258308](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156258308) queued. Launched after #43 validated. Original [45155787562](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45155787562) failed signature formatting, fixed without changing tests. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `900ced8a25a2e704a93ee2f7c26b2d67147bd7ff` | [45153573720](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720), 6/8. Native prerequisites running. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `1e486d66f9d7657387cb09bf0182d5f57d2ed6fe` | Staged behind #45. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `8b1a015b5de1d4a533a2e793bd67e17b977b9ca6` | [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030), 7/8. The named real Simnet red-block payout reopen regression passed. Native authority suite running. |
| [#49](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/49) | `a4e79bb486418f6ff96b192370aac4db0f67867e` | [45146860835](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146860835), **8/8 passed**. Two focused verifier cases and 36 authority, 1,592 native and 32 no-asm passed. |
| [#50](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/50) | `e01899558e2e5746c8dd3650af42e997b9f282bc` | [45152422597](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152422597), 7/8. Focused issuance step passed; individual canonical-case line awaits completed native log. |
| [#51](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/51) | `ed32a290dff167b4c6e67397fe9080e5523833c0` | [45154174813](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45154174813), 5/8. Named RocksDB checkpoint test passed. |
| [#52](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/52) | `44da60377d7ef13557264f25e41f908797cacee0` | [45156348643](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156348643) pending. Initial [45156178173](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45156178173) failed rustfmt on an assertion initializer; formatting-only correction pushed. Branch `feat/phase6-coinbase-settlement-checkpoint-20261123`. |

#52 reconciles existing expected coinbase outputs and merged reward inputs into new subsidy, net paid subsidy, recycled fees, and developer deferred amounts. It checks the resulting cumulative checkpoint transition, rejects duplicates and overflow, and covers eligible red rewards, non-DAA red fees, deferred accrual/release and altered totals. It is an opt-in reconciliation component, with no change to active reward construction. [Design](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/44da60377d7ef13557264f25e41f908797cacee0/docs/development/PHASE6_COINBASE_SETTLEMENT_CHECKPOINT_2026-11-23.md)

Next run the #52 named test to confirm the adapter; inspect #48's reconnect artifact and #50's completed native log for the individual canonical-claim result. Advance #47 only when #45 validates. Production still needs ordered claim attribution bound to independently authenticated selected-chain evidence. The guarded publisher must retain the selected and virtual guards, then stage verified settlement alongside readiness in one #51 authority batch. Exercise the actual publication path across interruption, RocksDB reopen, and branch replacement before activation. The proposed 29B KCADE finite cap and 21B/8B allocations remain inactive.

## Native-PoW and issuance integration checkpoint — November 23, 2026

- #43 `0b89ddcd462fe5dcd526ee42ef5cc25d296293d4` passed 8/8 on [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224). #48 advanced on that exact base, head `8bb5b6799f85c0c3986612355c47ec11f5d19857`, [45155787562](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45155787562).
- #49 `a4e79bb486418f6ff96b192370aac4db0f67867e` passed 8/8 on [45146860835](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146860835).
- #46's named real Simnet settlement reopen step passed on `8b1a015b5de1d4a533a2e793bd67e17b977b9ca6` in [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030). Full matrix still active. #45, #50 and #51 remain active on the exact runs below. #47 awaits green #45.
- Next implementation target is checked settlement amounts from the existing coinbase template, before any authenticated authority-batch publication. Production issuance remains disabled.

## Persisted checkpoint and selected ancestry — November 23, 2026

| Draft | Exact head | Verified status |
| --- | --- | --- |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `724687069469e0ad18e9e97c32a8549513fe6442` | [45146341474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146341474), **8/8 passed**. Six focused checkpoint tests including selected-child verification; 36 authority, 1,591 native (25 skipped), and 32 no-asm passed. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `29d78741ff436ede003f69f3d1a558069d846890` | [45144884093](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144884093), **8/8 passed**. Six focused tests including process exit without RocksDB close; 36 authority, 1,591 native (26 skipped), and 32 no-asm passed. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `900ced8a25a2e704a93ee2f7c26b2d67147bd7ff` | [45153573720](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720) active on validated #41. Focused selected-ancestry result pending. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `1e486d66f9d7657387cb09bf0182d5f57d2ed6fe` | Staged behind #45. Guarded fixture uses a test-only settlement row. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `0b89ddcd462fe5dcd526ee42ef5cc25d296293d4` | [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224), 7/8. Focused CLI and resource checks passed. Native workspace tests running. [Metrics artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224/artifacts/14805407543) preserved. |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `3eb8661274319e6edb17d27d27bffe1bb4cb4ee6` | Staged until #43 full matrix passes. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `8b1a015b5de1d4a533a2e793bd67e17b977b9ca6` | [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030), 6/8. Native Simnet recovery prerequisites running before the corrected real-settlement step. |
| [#49](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/49) | `a4e79bb486418f6ff96b192370aac4db0f67867e` | [45146860835](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146860835), 7/8. Focused verifier and no-asm passed; documentation running. |
| [#50](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/50) | `e01899558e2e5746c8dd3650af42e997b9f282bc` | [45152422597](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152422597), 4/8. Focused core issuance filter passed. It selects the canonical-claim regression with the other two verifier tests; download the native log on completion for the individual PASS line and count. |
| [#51](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/51) | `ed32a290dff167b4c6e67397fe9080e5523833c0` | Replacement [45154174813](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45154174813) active. Original [45153971670](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153971670) failed rustfmt on a fixture initializer; the correction changes formatting only. Branch `feat/phase6-issuance-checkpoint-store-20261123` stacked on #50. The exact focused storage selector precedes broader native tests. |

#51 adds a single versioned issuance summary under the existing deferred authority batch. Stage and load require the same-database selected-chain guard and a separately supplied expected commitment. Stage rejects the entire batch on verification failure. The RocksDB regression covers abandoning the batch, rejecting an altered paid/deferred split, reopening the committed data, and rejecting and replacing a stale owner after reorg. The associated claim root and count commit unique claims from #50. Per-source acceptance still belongs to consensus. [Storage design](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/ed32a290dff167b4c6e67397fe9080e5523833c0/docs/development/PHASE6_ISSUANCE_CHECKPOINT_STORE_2026-11-23.md)

**Next:** Inspect #51's named storage test and #46's named payout test results. Start #48 when #43 is 8/8 and #47 when #45 is 8/8. Preserve full matrix and focused logs. The production integration still needs an authoritative reorg-aware claim/deferred view derived from real settlement, an independent digest from verified consensus evidence, and a guarded promotion test that proves the new row and readiness are atomic on actual subsidy, recycled fees, deferred obligation, restart and interrupted writes. #51 uses a batch probe and does not prove production readiness publication. The proposed 29B KCADE cap with 21B primary and 8B finite tail remains inactive.

## Validation and persistent reward accounting checkpoint — November 23, 2026

- #41 at `724687069469e0ad18e9e97c32a8549513fe6442` passed all eight jobs, six focused checkpoint tests, 1,591 workspace tests and 32 no-asm tests. [45146341474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146341474)
- #45 at `900ced8a25a2e704a93ee2f7c26b2d67147bd7ff` now includes the validated #41 parent. [45153573720](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45153573720) pending. #47 remains gated on #45.
- #43 and #44 remained 7/8 with their focused steps green. #48 remains gated on full #43. #46's corrected payout selector is pending. #49's focused verifier and #50's focused canonical claim step passed, with full matrices active. Their exact prior runs are recorded below.
- Independent work is limited to a single authenticated settlement checkpoint record in the existing RocksDB authority batch namespace. Production activation, monetary parameters and launch remain gated.

## Ordered issuance claims and corrected native-PoW evidence — November 23, 2026

| Draft | Exact head | Verified Actions state |
| --- | --- | --- |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `724687069469e0ad18e9e97c32a8549513fe6442` | [45146341474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146341474), 7/8. Focused authenticated checkpoint and regular workspace steps passed. No-asm tests running. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `29d78741ff436ede003f69f3d1a558069d846890` | [45144884093](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144884093), 7/8. Focused checkpoint suite passed. Regular workspace tests running. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b` | Staged behind full green #41. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `1e486d66f9d7657387cb09bf0182d5f57d2ed6fe` | Staged behind full green #45. Guarded batch coverage is test-only. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `8b1a015b5de1d4a533a2e793bd67e17b977b9ca6` | [45151879030](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45151879030), running. Prior [45143838285](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143838285) passed seven jobs, then selected zero tests in the focused step. Corrected the Nextest exact selector to include the full Rust module path. Regression source and assertions unchanged. |
| [#49](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/49) | `a4e79bb486418f6ff96b192370aac4db0f67867e` | [45146860835](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146860835), 7/8. Focused issuance verifier and native authority tests passed. Regular workspace tests running. |
| [#50](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/50) | `e01899558e2e5746c8dd3650af42e997b9f282bc` | [45152422597](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152422597), replacement pending. [45152207805](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45152207805) failed rustfmt on one assertion; no behavior changed. Branch `feat/phase6-ordered-issuance-claims-20261123` stacked on #49. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `0b89ddcd462fe5dcd526ee42ef5cc25d296293d4` | [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224), 7/8. Both CLI cases, per-node resource validation, and [corrected artifact upload](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224/artifacts/14805407543) passed. All seven expected RSS samples were positive. At convergence A/B/C RSS was 126,566,400 / 128,303,104 / 125,984,768 bytes. The corrected artifact reports 12 accepted blocks, 6 recovered blocks, 309 ms reopen-to-RPC, and 311 ms reopen-to-tip. Final tip `381bd3ff487b6692c62d43aa0c735ed87fb5634799eb1b6505c55f7ce30f4494`. Native authority tests running. |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `3eb8661274319e6edb17d27d27bffe1bb4cb4ee6` | Staged behind full green #43. |

#50 derives a candidate checkpoint from ordered, unique reward sources and a separately verified selected path. It checks the selected settlement anchor, sorts by selected index then source hash, hashes with existing consensus primitives, and balances paid/deferred issuance at settlement boundaries with checked cap and overflow checks. The focused regression covers input permutation, duplicate sources, stale anchors, wrong point, modified payout, and arithmetic failure. The independent digest remains required.

Production acceptance and persistence still need an authoritative incremental claim and deferred-payout view from actual settlement, a separately authenticated digest from selected-chain evidence, and a guarded batch that includes the verified state and ready revision. The next integration test must use actual subsidy, recycled fees, deferred obligations, selected-chain replacement, and restart or interrupted writes. #47's test-only row is evidence of the batch boundary. The 29B KCADE candidate with 21B primary and 8B finite tail remains inactive.

**Next:** Review #50's focused derivation and #46's corrected focused payout result. On 8/8 #43, start #48 and inspect reconnect and offline reopen logs. On 8/8 #41, integrate its formatting-only change into #45 and start selected ancestry CI. Run #47 after #45 validates. Preserve all passing runs.

## Issuance commitment and native-PoW evidence — November 23, 2026

| Draft | Exact head | GitHub Actions evidence |
| --- | --- | --- |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), 8/8 passed. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `724687069469e0ad18e9e97c32a8549513fe6442` | [45146341474](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146341474), 3/8 passed, native pending. Original [45144878404](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144878404) failed rustfmt on a trailing blank line. Only formatting changed. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `29d78741ff436ede003f69f3d1a558069d846890` | [45144884093](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144884093), 6/8 passed. Native prerequisite descendant recovery running. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b` | Staged. Run after #41 validates. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `7119ef849289553e0e1b72e818f8e6ba003260bb` | [45143838285](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143838285), 7/8 passed. Native prerequisite Simnet recovery/archive/partition running before the settlement regression. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `1e486d66f9d7657387cb09bf0182d5f57d2ed6fe` | Staged. Run after #45 validates. |
| [#49](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/49) | `a4e79bb486418f6ff96b192370aac4db0f67867e` | [45146860835](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146860835), running; focused issuance primitive result pending. Initial [45146560495](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146560495) failed rustfmt. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `0b89ddcd462fe5dcd526ee42ef5cc25d296293d4` | Replacement [45146978224](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45146978224) queued. On source `13dc263c9883b858ca30173d5087f954bc46b58d`, [45143950379](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379) was 7/8, with both CLI regressions passing and the metrics parser failing. |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `3eb8661274319e6edb17d27d27bffe1bb4cb4ee6` | Staged behind corrected #43. Includes structured reconnect timing, offline reopen timing, last-observed timeout diagnostics, and an artifact validator. |

**Observed native-PoW evidence:** #43's exact earlier source passed the three-process focused test in 4.878 s. It accepted 12 blocks, rejected the invalid nonce, accepted duplicate submission without a DAG accounting change, and recovered C's six missed blocks. RPC was ready in 310 ms and DAG convergence took 312 ms from process reopen. Final selected tip was `36c95c5e7007df91052b7ac8f4c817a2299cc52ab42a2c6dbcc257ec78dd893d`. The regression passed all process shutdown assertions. Preserve the [raw CLI artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379/artifacts/14803614244). Metrics parsing failed on a Nextest ANSI reset appended to the final JSON row. A and B showed final RSS of 128,520,192 and 127,254,528 bytes. C's sample still reported zero before the sampler updated. The replacement normalizes terminal output and requires usable per-node samples. Full native-PoW acceptance awaits that run.

**Issuance foundation:** #49 on `feat/phase6-issuance-commitment-20261123` branches from validated #39. It commits network and pruning-point identity, the ordered-claim root and count, and minted, paid and deferred amounts. The verifier requires an independently supplied expected digest, applies checked arithmetic, and checks the supply cap. The candidate is not wired into block, pruning proof, persistence or payout rules. The next production gate is to derive the root from unique ordered reward claims, bind the digest to independently verified selected-chain evidence, and atomically publish verified deferred settlement with readiness. Keep #46's live legacy settlement and #47's test-only guarded publisher evidence distinct. The proposed 29B finite maximum with 21B primary and 8B tail remains inactive.

**Next:** Inspect #43's replacement metrics and focused results before launching #48. On green #41, carry any required parent fixes into #45 and start its regression. On green #45 start #47. Review #44's process-exit test, #46's real payout reopen, and #49's core verifier as their focused steps complete. Preserve the draft stack and keep production issuance and public launch disabled.

## Authenticated issuance and live-peer checkpoint — November 23, 2026

| Draft | Branch | Exact head | Actions evidence |
| --- | --- | --- | --- |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), **8/8 passed**. Five focused checkpoint tests, 36 authority regressions, 1,590 native tests (25 skipped), and 32 no-asm tests passed. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `e7767be32084943bf648f6b1f70bc426444a7b4a` | [45144878404](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144878404) triggered against validated #39. Tested sources inherited from `b997d82bc6e72384a5ba747485812698cce425dd`; result pending. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `test/phase6-issuance-checkpoint-process-exit-20261123` | `29d78741ff436ede003f69f3d1a558069d846890` | [45144884093](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45144884093) triggered against validated #39. Tested sources inherited from `e87401b6bca55a5684f513beb0fa893fed862d24`; result pending. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `test/phase6-issuance-current-selected-path-20261123` | `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b` | Staged. Launch after #41 is green. |
| [#47](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/47) | `test/phase6-guarded-issuance-publication-20261123` | `1e486d66f9d7657387cb09bf0182d5f57d2ed6fe` | Code pushed and staged behind #45. Dedicated focused test configured. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `test/phase6-persisted-red-settlement-20261123` | `7119ef849289553e0e1b72e818f8e6ba003260bb` | [45143838285](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143838285), 5/8 so far. Native 137588250219 was compiling/running the shielded persistent reopen prerequisite. Focused settlement pending. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `test/phase6-cli-real-pow-recovery-20261123` | `13dc263c9883b858ca30173d5087f954bc46b58d` | [45143950379](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379), 4/8 so far. Native 137588618046 executing the focused CLI step. Artifact not yet available. |
| [#48](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/48) | `test/phase6-native-pow-live-peer-reconnect-20261123` | `f5c34b0a53f2a6e690f95727b37617c905488ffb` | Code pushed and staged behind #43. Dedicated CLI regression and artifact configured. |

#47 factors the existing guarded publisher and exercises it with a test-only settlement row in the same deferred RocksDB batch as the promotion marker and ready revision. The test rejects malformed selected-child body input without publishing the row or marker, then reopens after a valid promotion and checks an equal-total paid/deferred alteration against an independent fixture digest. The production path continues to use existing proof, shielded, SMT, UTXO and lease checks. Production still needs a consensus-authenticated issuance digest and a bounded persisted claim/deferred-balance view. #46 supplies the separate real legacy payout reconciliation. The proposed 29B KCADE finite policy remains 21B primary and 8B tail, inactive in production.

#48 adds a one-session loopback TCP forwarder for C's sole upstream link. It cuts the live connection with C still running, checks three-block native-PoW catch-up and agreement across A/B/C, then reopens C while it is still isolated and checks its persisted state. The test awaits offline DAG recovery without re-enabling the link and declares the Tokio network and I/O features directly. Per-node observations and daemon logs are staged. #43's operator runbook remains authoritative for the original three-node test. [Runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/13dc263c9883b858ca30173d5087f954bc46b58d/docs/development/PHASE6_LOOPBACK_OPERATOR_RUNBOOK_2026-11-23.md)

**Next:** Read #43's focused CLI and metrics steps and inspect its artifact when available. Start #48 on validated #43, resolving any required parent fixes on the stacked branch first. Observe #46's focused real-payout result and matrix. On #41 green start #45, then #47 after #45 green. Record #44's exact process-exit result separately. Remaining production gates include authenticated mixed subsidy/fee/deferred settlement, real-PoW competing-tip and representative resource tests, and owner-approved network configuration.

## Two-task checkpoint — November 23, 2026

| Draft | Exact source SHA | Actions and result |
| --- | --- | --- |
| #32 | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), 8/8 passed. |
| #34 | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), **8/8 passed**. Both focused payout regressions passed. 39 authority, 1,582 native and 32 no-asm passed. |
| #39 | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), 7/8. Focused checkpoint, regular workspace and no-asm passed. Native 137563153013 running documentation. |
| #40 | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), **8/8 passed**. 35 authority, 1,579 native and 32 no-asm passed. Five-minute simulated-PoW resource [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666/artifacts/14801585581). |
| #42 | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320), **8/8 passed**. 39 authority, 1,579 native and 32 no-asm passed. Three-process simulated-PoW startup and restart with [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320/artifacts/14800768777). |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `13dc263c9883b858ca30173d5087f954bc46b58d` | [45143950379](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143950379), replacement matrix running, 1/8 passed. Native CLI step pending runner setup. Initial [45143642718](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143642718) failed rustfmt on the node launcher signature. Corrected on the replacement commit. |
| [#46](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/46) | `7119ef849289553e0e1b72e818f8e6ba003260bb` | [45143838285](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45143838285), running, 1/8 passed. Focused legacy Simnet payout reconciliation scheduled before broad native build. |

#43 branch is `test/phase6-cli-real-pow-recovery-20261123`, based on the exact green #42 commit. It tests exact duplicate submission against unchanged DAG counters and adds native-PoW commands, diagnostics and acceptance criteria to the [operator runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/13dc263c9883b858ca30173d5087f954bc46b58d/docs/development/PHASE6_LOOPBACK_OPERATOR_RUNBOOK_2026-11-23.md). Validate the dedicated CLI steps and the per-node artifact before claiming native-PoW acceptance.

#46 branch is `test/phase6-persisted-red-settlement-20261123`, based on the exact green #34 commit. The focused regression uses the real accepted red payout through the consensus persistence path. It reconciles all payouts against independently read source subsidies, checks exactly-once coinbase acceptance and verifies body commitment and selected ancestry before and after RocksDB reopen. The complete fixture UTXO set must be preserved. [Regression scope](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/7119ef849289553e0e1b72e818f8e6ba003260bb/docs/development/PHASE6_PERSISTED_RED_SETTLEMENT_2026-11-23.md).

Production import lacks a consensus-authenticated issuance commitment, a versioned checkpoint codec, and a bounded persisted claim and deferred-balance view. Next integrate a mixed subsidy/fee/deferred settlement fixture at the guarded promotion boundary after #46 validates. Production issuance stays disabled under the proposed 29B KCADE maximum with 21B primary and 8B finite tail.

On #39 full green, trigger #41 at `b997d82bc6e72384a5ba747485812698cce425dd` and #44 at `e87401b6bca55a5684f513beb0fa893fed862d24` using their focused tests and full matrices. After #41 passes, trigger #45 at `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b`. No duplicate workflow was manually started. The CI workflow supersedes obsolete attempts on new commits.

## Previous checkpoint — November 23, 2026

| Draft | Branch | Exact commit | Actions evidence |
| --- | --- | --- | --- |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), **8/8 passed**. Focused Simnet genesis and issuance conservation passed; 39 authority, 1,580 regular native (25 skipped), and 32 no-asm passed. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), 7/8 passed. Both focused payout regressions passed. Native 137560443988 passed regular workspace tests and advanced into no-asm checks. |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), 7/8 passed. Focused checkpoint persistence passed. Native 137563153013 advanced past the devnet-prealloc build into authority regressions. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), 7/8 passed. Focused five-minute sample and [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666/artifacts/14801585581) passed. Native 137559872891 passed regular workspace and no-asm tests and advanced into `cargo doc`. |
| [#42](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320), 7/8 passed. Three-process CLI check and [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320/artifacts/14800768777) passed with simulated PoW. Native 137568406534 advanced into authority regressions. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged behind #39. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `test/phase6-issuance-checkpoint-process-exit-20261123` | `e87401b6bca55a5684f513beb0fa893fed862d24` | Staged behind #39. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `test/phase6-cli-real-pow-recovery-20261123` | `6d03bcb4b55bf75234be6322b2878f27801843e6` | Native-PoW test and artifact checks extended. Staged behind #42 and unvalidated. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `test/phase6-issuance-current-selected-path-20261123` | `7fe95e5d3e6cf5e1cb994dbba0835f8c74069c5b` | Test-only code pushed. Staged behind #41 and unvalidated. |

The remaining native jobs showed step progress. GitHub's Actions API did not return their live logs. Two completed comparable native jobs took approximately 49 and 58 minutes from the first Cargo command to the last. Timestamped logs show separate feature builds and 6m31s–8m13s in regular Nextest execution itself. No failed, queued, cancelled or stalled job was observed in these runs. See the [dated development report](docs/development/PHASE6_NATIVE_CI_TIMING_AND_REOPEN_2026-11-23.md).

PR #45 now reruns the child-body and selected-chain checks against a reopened RocksDB handle. It rejects the stale sibling with either tip, checks that descendant state includes only the selected sibling's fixture nullifier, and verifies the settled checkpoint commitment before testing paid/deferred corruption. The test uses fixture-seeded ancestry and illustrative commitments.

PR #43 checks the exact 12-block native-PoW count from genesis, confirms all three nodes agree on the selected tip, block count, pruning point and virtual DAA score after restart, and adds the final DAG fields to the recovery artifact. No native-PoW result is claimed before Actions exercises the updated head.

**Next:** On full green #39, run #41 and #44 with their focused checks and full matrices. On full green #42, run #43, retain the native-PoW test output and record restart/catch-up and per-node resource data. On full green #41, run #45. Authenticated pruning restoration tied to real settlement and atomic readiness, production hard-cap accounting, representative native/AuxPoW load, and approved network parameters remain gates. The proposed finite policy remains 29B KCADE maximum with a 21B primary allocation and 8B tail. Production issuance remains disabled. The dated report records the next live-settlement fixture at the guarded IBD completion and atomic promotion boundaries.

## Validation stack

PR [#1](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/1) →
[#2](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/2) →
[#3](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/3) remains open.
The combined tip `f64839c` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245)
with 1,518 native tests and 25 skipped. Goref regeneration is complete.

## Phase 6 continuation

- The [authenticated import candidate](docs/development/PHASE6_AUTHENTICATED_IMPORT_2026-11-08.md)
  at `a6dd7fe` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489992639),
  1,552 native tests with 25 skipped, and 32 no-asm tests. The readiness
  reopen, failed-import reservation, abrupt WAL recovery, and shielded
  proof and nullifier regressions passed.
- The [staging drain candidate](docs/development/PHASE6_STAGING_DRAIN_2026-11-08.md)
  at `cf098af` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489994109),
  with 1,553 native tests, 25 skipped, and 32 no-asm tests. The staging
  cleanup and both abrupt WAL recovery regressions passed.
- The [lease write scope](docs/development/PHASE6_AUTHORITY_WRITE_SCOPE_2026-11-08.md)
  rechecks committed evidence on lease issuance and commit. Its source
  `c4a5aab` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493733661)
  and 1,556 native tests. The [pruning session retention change](docs/development/PHASE6_PRUNING_SESSION_RETENTION_2026-11-08.md)
  at `efd3c5c` passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493736508)
  and 1,557 native tests.
- The [processor ownership candidate](docs/development/PHASE6_PROCESSOR_AUTHORITY_OWNER_2026-11-08.md) binds the inactive
  adapter to the owner created by the processor itself. Its exact
  [tested source](https://github.com/DeepFriedBlockchains/KascadeNetwork/commit/1e3cf487d66da46dfb35b04c027f1126e923070b)
  passed [all eight jobs](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42493738676),
  1,557 native tests with 25 skipped, and 32 no-asm tests.

## Authority integration

The public integration branch builds on `f436a1938def7370bf54d99d4f40606dff3f87ce`,
the documentation-only continuation of green source `1e3cf48`.
It preserves the PR #1 → #2 → #3 stack and brings the verified authority
candidates together with focused cross-component regressions. A processor
worker is detached, drained and reopened against the same RocksDB path.
An import token that observes a durable revision replacement stays revoked
until its worker releases the reservation.

See the [dated integration report](docs/development/PHASE6_AUTHORITY_INTEGRATION_2026-11-08.md).
Draft [PR #4](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/4) is stacked on PR #3 at `f64839c`.
The exact implementation head is `c44805170fb44dcd51e645e50341072c2a38e1e3` on
`feature/phase6-authority-integration-20261108`.
[PR run 42500808732](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500808732) is the authoritative combined run
and passed all eight jobs. Both focused regressions passed, along with
1,559 workspace tests (25 skipped), 32 no-asm tests, and strict Clippy.
The superseded
[push run 42500792929](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42500792929) was cancelled by the PR run.
PR #4 retains its tested implementation head.

## Inactive staging coordinator

The [staging coordinator report](docs/development/PHASE6_STAGING_COORDINATOR_2026-11-08.md) describes the next stacked
candidate. One processor-owned session covers proof, SMT, shielded and
UTXO batches. A failed or interrupted component keeps readiness closed.
Authenticated completion issues the exact lease used by the inactive
promotion gate.

Restart on a database with descendants pins committed virtual and
selected-chain state, verifies the applied tip and pruning-point
shielded commitments, and checks the post-pruning nullifier projection.
Unavailable proof, a missing authenticated tip child or an inconsistent
committed view leaves authority closed.

## Candidate validation

[Draft PR #5](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/5) is stacked on PR #4.
The source checkpoint is `4eb422549fa822b804968f9c2911018b518286f0` on `feature/phase6-staging-coordinator-20261108`.
[PR run 42506751029](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506751029) passed all eight jobs.
All four focused regressions and 1,561 native workspace tests passed
(25 skipped). The no-asm suite passed 32 tests. Formatting and strict
Clippy passed.
The same-SHA [push run 42506746483](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42506746483) was superseded.

[Run 42503818037](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42503818037)
reported the missing `kaspa_hashes::Hash` test import in Check and
Clippy. Its correction `6e5c4ae` passed both jobs and five other platform
and build checks, with seven of eight jobs passing overall in
[run 42504801081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42504801081).
The focused suite passed three of four regressions, including committed
descendant recovery. The staging cancellation fixture expected an absent
anticone row, but consensus initialization creates it. The corrected test
requires both scoped rows to equal their pre-operation bytes after the
rejected batch. The staging and restart implementation remains unchanged.

## Typed UTXO adapter

[Draft PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6)
is stacked on PR #5. The original typed chunk source
`2afec236ac42e03bdeaf21d06c0191a70bba99f7` passed all eight jobs in
[run 42509613211](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42509613211).
Four focused regressions, 1,561 native workspace tests (25 skipped), and
32 no-asm tests passed.

## Owner-bound completion and promotion receiver

The [November 10 report](docs/development/PHASE6_PROMOTION_RECEIVER_2026-11-10.md)
records the continuation on the PR #6 branch. The typed UTXO completion
worker verifies committed UTXO rows before publishing the position and
stable flag in the same authority-checked batch. The consensus manager has
a separate verified commit entry point. It asks the exact staging receiver
to consume the completion receipt before swapping consensus. At that checkpoint the receipt issuer and receiver were inactive test
code. The later preview wires them into production IBD behind a
default-off Simnet feature. The initial source `bc393eae2d9bd03df2f684d06e53ec78e865da60` reached
[run 42894808941](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42894808941).
Formatting reported two line wraps in the regression test. The correction
`72af6c818be80c8a0365036e23c4744da8b4bf94` passed formatting in
[run 42895086028](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42895086028).
Check and strict Clippy also passed. The full workspace result was
pending when the follow-up source was prepared.

## IBD proof receipt and promotion recheck

IBD now passes the exact proof accepted by the defender to the staging
receiver in a typed receipt. This uses the existing validation and
application logic. The inactive promotion commit now independently
rechecks the committed UTXO, SMT, proof, and shielded evidence before
spending the lease. A regression removes committed UTXOs after receipt
issuance and requires the write to fail with readiness closed. The follow-up passed all eight jobs in run 42896198192, recorded below.

## Owner-bound proof application

The candidate on `feature/phase6-owned-proof-import-20261110` builds on draft
[PR #6](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/6)
at `42476e2661b5e579f665407d18352a36af09c301`. The baseline
[run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
had passed six jobs at candidate preparation. The Linux release build and
workspace Test Suite were still running. The candidate uses a separate
branch to preserve that run.

The [dated report](docs/development/PHASE6_OWNED_PROOF_IMPORT_2026-11-10.md) records the receiver-owned proof worker.
The coordinator binds the defender's exact receipt before its first
database write and keeps the proof evidence through completion. Both IBD
proof branches use the same proof and history application entry point.
The coordinator-specific entry point remains inactive. The initial proof candidate `863ec4fd7293fa3c54916b229c0f9c5f9a6ba659` passed formatting and Check in
[run 42897872282](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42897872282)
before the streaming follow-up.

The streaming follow-up also drains the SMT and shielded writer on a
stream error before returning to the staging caller. A focused test
covers error propagation only after worker release. The completed combined CI results are recorded below.

The combined source is `ecf0aacafa9cfc3028eb363021c5aebf733c62e1` on
`feature/phase6-owned-proof-import-20261110`. Its exact validation is
[run 42898217206](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42898217206).
The unchanged PR #6 baseline
[run 42896198192](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42896198192)
passed all eight jobs at `42476e2661b5e579f665407d18352a36af09c301`.
It passed six focused regressions, 1,563 native workspace tests
(25 skipped), and 32 no-asm tests. Combined run 42898217206 passed all eight jobs.

## Production IBD preview

See [the dated development report](docs/development/PHASE6_PRODUCTION_IMPORT_PREVIEW_2026-11-10.md). The continuation adds a
default-off `kaspa-consensus/phase6-staging-preview` feature for an
isolated Simnet staging receiver. It admits the exact defender-accepted proof
before writes, carries the same reservation through production IBD
workers, and verifies durable virtual UTXOs before verified promotion.
[Draft PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is stacked on the green `feature/phase6-owned-proof-import-20261110`
branch at `ecf0aacafa9cfc3028eb363021c5aebf733c62e1`. The initial head was
`ebcb3ce00043b0d3f73a2ee7508f0cc68a363a69` on `feature/phase6-production-import-preview-20261110`.
[Run 42909699012](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42909699012)
passed seven jobs and failed the two-daemon shielded preview on exact
descriptor verification. The correction and green replacement run are
recorded below. The parent passed eight focused regressions, 1,565
workspace tests (25 skipped), and 32 no-asm tests in run 42898217206.

## Descriptor preservation correction

[PR #7](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/7)
is validated at `2ff50a53e727511a738c04fec6ddcd432c389790` on
`feature/phase6-production-import-preview-20261110`.
[Run 42930280069](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42930280069)
passed all eight jobs, including formatting, strict Clippy, 11 focused
regressions, 1,567 workspace tests (25 skipped), and 32 no-asm tests.
Workspace doc tests, no-asm doc tests, cargo doc and kip10 example checks
also passed. The same-SHA push run 42930270803 was superseded.

The production sanity rebuild had replaced the authenticated external
descriptor with an optimized local descriptor. It now reconstructs
without descriptor or proof-cache writes. Exact descriptor verification
is unchanged. The original two-daemon shielded promotion regression
passed. See [the descriptor report](docs/development/PHASE6_STAGING_PROOF_DESCRIPTOR_2026-11-10.md).

## Peer interruption and active receiver role

[Draft PR #8](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/8)
is stacked on PR #7. Its exact source is
`bc886b6aaf9df3f2deceac75691f1a1bb932f9fa` on
`feature/phase6-preview-peer-interruption-20261110`.
[PR run 42935509499](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42935509499)
passed all eight jobs. Formatting, strict Clippy, all 15 focused
regressions, 1,568 workspace tests (25 skipped), and 32 no-asm tests passed.
Workspace doc tests, no-asm doc tests, cargo doc and the kip10 example
also passed. Same-SHA push run 42935435587 was superseded.
This documentation-only continuation changes no tested implementation.

The tests pause actual peer transfers, disconnect a temporary peer,
require reservation retention while paused and worker draining before
retry, then require verified promotion. The UTXO case reopens the same
data directory before retry. A separate persistent database regression
abandons a receipt before promotion, reopens incomplete SMT evidence and
requires fresh authentication.

Predecessor `b510f585f3e746e7541a58cc5e09b69c988189d4` passed seven jobs
and 14 of 15 focused regressions in
[run 42932087535](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42932087535).
The remaining test exposed a real bug. Existing active databases skip
genesis initialization when reopened, but that flag was also used to
identify staging receivers. Fresh staging allocation now explicitly
assigns a volatile role, activation revokes it and active reopen starts
without it. The original rejection assertion remains. Successful import
tests additionally require a durable verified promotion record and a
closed active staging role.

See [the interruption report](docs/development/PHASE6_PEER_STREAM_INTERRUPTION_2026-11-10.md).
Shielded interruption occurs before metadata because this fixture has
zero spent nullifiers. Its transparent UTXO stream is empty and pauses
before the done frame. Nonempty payload interruption and abrupt process
termination before promotion remain additional coverage.

## Nonempty payload and abrupt restart validation

[Draft PR #9](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/9)
is stacked on PR #8. Its tested implementation is
`110d1625763c9fecd2cf2a226e5e7fb25add99bf` on
`feature/phase6-payload-crash-recovery-20261111`.
[Run 43068789717](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43068789717)
passed all eight jobs. Formatting, strict Clippy, 21 focused regressions,
1,568 workspace tests (25 skipped), 32 no-asm tests, doc tests,
cargo doc and the kip10 example passed. No implementation CI remains
pending for this milestone.

The fixtures mine a real Orchard spend and nonempty transparent UTXOs.
Ten signed user lanes are refreshed inside the pruning-point finality
window. Cancellation pauses after a real native SMT batch, nonempty
UTXO append or verified atomic shielded seed. The UTXO done frame is
held back until disconnect. Real SIGKILL cases reopen the same sender
and receiver directories, reject incomplete staging, then require fresh
authentication, verified promotion and successful post-IBD blocks.
All six new regressions passed with their safety assertions retained.

See [the dated report](docs/development/PHASE6_PAYLOAD_CRASH_RECOVERY_2026-11-11.md).
This documentation-only continuation changes no tested implementation.

## Active receiver admission validation

[PR #10](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/10)
retains documentation head `a0885ad19eac58edec15283394e9ad78cacc98d2`.
Implementation `509e54b5af6a686b24086eea30fe97e8dc02af23` passed
all eight jobs in [run 43076086692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43076086692).
The completed job log confirms 21 focused tests, 1,568 workspace tests
(25 existing skips), and 32 no-asm tests. Formatting, strict Clippy,
documentation, example and platform/build checks passed.
Real authenticated receipts were rejected by actual admission calls
on ordinary active receivers before import and after promotion,
including the restart scenarios. Do not repeat this completed work.

## Tampered shielded stream validation

Draft [PR #11](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/11)
is validated at `9612aac2ebca91f384591c3c135fa9df57dbc6d3` on
`feature/phase6-tampered-shielded-stream-20261112`.
[Run 43309997530](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43309997530)
passed all eight jobs, independently rechecked November 13, 2026.
Logs confirm 23 focused tests, 1,568 workspace tests (25 skips), and
32 no-asm tests.
The original tampered-stream, same-directory recovery, active-admission
and verified-promotion assertions passed. Do not reimplement these.
Initial run 43308475970 failed five formatter hunks; the validated
correction changed formatting only.

## Bounded shielded ingestion — PR #12

Draft [PR #12](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/12)
is stacked directly on PR #11 at
`9612aac2ebca91f384591c3c135fa9df57dbc6d3`.
Branch: `feature/phase6-bounded-shielded-ingestion-20261113`.
Bounded ingestion implementation: `acba224032ab7613ac805bcb33160fc696436f2c`.
Current reopen test/CI correction: `f1808f1231697474de66bf3f99bad0c2f01ead0d`.
Later checkpoint commits contain documentation only.

Spent-nullifier ingestion no longer accumulates the full set or one
full-set membership batch. Frames are capped at 4,096 entries (128 KiB
raw payload). Each deferred synchronous RocksDB batch checks the exact
processor-owned session, database, revision and allowed shielded prefixes.
The existing proxy pruning guard and worker reservation remain pinned
through the blocking call; network waits hold no store locks.

Partial rows remain unready. Declared count, received accumulator and
streamed committed database evidence must all agree. Verified metadata
and readiness publish together in one atomic deferred batch, with cache
invalidation after persistence. Failed owned imports poison their session.
The raw adapter rejects isolated receivers lacking an authenticated permit.
Isolated completion/promotion checks also stream committed membership.

Eight new bounded_shielded_ regressions cover lazy large streams,
persistent close/reopen/retry, interrupted and corrupt payloads, duplicate
membership, changed committed evidence, rejected final publication,
stale authority, foreign ownership and deferred metadata/shared anchors.
Existing production shielded disconnect and SIGKILL controls now stop
after a real membership batch, before metadata publication. The original
23 authority integration cases remain selected, including tampered retry,
restart, activation rejection and verified successful promotion.

## PR #12 reopen correction and current validation

Run [43511658558](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43511658558)
finished with seven of eight jobs passing. Native job 132183262907 failed
bounded_shielded_partial_stream_reopens_unready_and_retries_fresh with
HashAlreadyExists while trying to insert committed genesis GHOSTDAG data.
Two focused cases passed, one failed, 28 were not run; full workspace
runtime validation did not execute. PR #12 is not validated.

This is a fixture initialization error: TestConsensus::with_db re-used
process_genesis=true when reopening a populated database. Production
Factory::new_active_consensus disables genesis processing for an Existing
entry. The correction uses skip_adding_genesis only on the fixture's
second open, matching that production contract. All prior assertions
remain, with an additional byte-for-byte persisted unready-marker check.
No production header or readiness check has been changed.

The native CI job now runs that focused regression first, before its
workspace build and broader 31-case selection. Require all eight jobs.
See [the correction report](docs/development/PHASE6_BOUNDED_SHIELDED_REOPEN_FIX_2026-11-13.md) for evidence and details.
Replacement source SHA and Actions run IDs will be recorded after push.

## Exact next executable checkpoint

Verify the corrected focused persistent reopen regression, then the
31 authority/bounded cases and full eight-job run. Fix genuine failures
without weakening readiness, authentication, worker draining, restart or
promotion assertions. Do not repeat completed earlier milestones.

Only after PR #12 independently passes, create a new stacked draft PR
for bounded failed-import cleanup: replace the legacy full-set deletion
batch with bounded durable deletions under the same owner permit. Keep
readiness false through interruption/restart and preserve the
no-validated-descendants precondition. Descendant recovery/export
snapshots remain subsequent independently validated milestones.

PUBLIC repository and GitHub Actions only. PRs #1–#12 remain intact,
draft and unmerged. Preview stays default-off and Simnet-only. No merge,
release, force-push, private-history import or consensus activation.

## Pushed correction checkpoint

Correction source: `f1808f1231697474de66bf3f99bad0c2f01ead0d`.
Remote PR #12 head was verified after push. Authoritative replacement
PR run: [43515652940](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43515652940).
Duplicate push run 43515643677 is being cancelled by workflow concurrency;
the PR run is pending while that cancellation finishes. No replacement
focused or full validation result is claimed. Native job order is the
single persistent-reopen regression, then workspace build, 31 focused
cases and full workspace validation. Resume this run rather than rerunning
the prior known-bad source. PR #12 stays draft and unvalidated.

## Isolated validation branch for stalled Actions cancellation

The push run 43515643677 has seven cancelled jobs but Lints job
132196802751 still reports in_progress with no steps. PR run 43515652940
remains pending with no jobs. The GitHub connector has no cancellation
action; no local or browser access is used as a workaround.

Branch `validation/phase6-bounded-reopen-20261113` carries the exact
same corrected Rust source and eight-job workflow as f1808f1, plus this
documentation checkpoint. Its push uses a different existing concurrency
group, avoiding the stuck group without modifying the workflow or its
checks. PR #12 is fast-forwarded to the same commit so that the exact
latest head is tested. No history rewrite or additional PR is involved.
The new validation push run is authoritative once its exact SHA and all
eight results are verified. Source and workflow equivalence will be
checked through GitHub's commit comparison. No pass is claimed yet.

## Active authoritative replacement run

Validation commit: `fa229e6524a78bb3bcd032d08e861f62a926637e`.
Both the PR #12 feature branch and `validation/phase6-bounded-reopen-20261113`
were verified at that commit. All eight jobs have started in push run
[43516463729](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43516463729).
GitHub's comparison f1808f1...fa229e6 lists only CURRENT_HANDOFF.md and
this correction report: all Rust source, tests and workflow are identical.
The original PR group's newer run 43516476491 remained pending; feature
push 43516466968 was cancelled. Use 43516463729 for actual validation,
not those duplicates. No replacement test result is claimed yet.
Any later handoff head is documentation-only unless explicitly recorded.

## Focused reopen regression passed

Authoritative run 43516463729 at validation commit
`fa229e6524a78bb3bcd032d08e861f62a926637e`, native job
132199538931, completed the dedicated persistent-reopen step successfully.
This was independently confirmed through the job-steps endpoint. All
retained restart, dirty-row, retry and readiness assertions, plus the
exact persisted-marker check, executed in that step. This confirms the
fixture initialization correction. The job has continued to workspace
build; the broader 31-case selection and full workspace suite are still
pending. PR #12 remains unvalidated until all eight jobs finish
successfully. Bounded cleanup remains gated on that result.

## PR #12 verified eight-job result

Validation commit `fa229e6524a78bb3bcd032d08e861f62a926637e` passed
all eight jobs in [run 43516463729](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43516463729).
The completed native job 132199538931 confirms the dedicated persistent
reopen regression passed first, all 31 authority/bounded cases passed,
1,576 workspace tests passed (25 existing skips), and all 32 no-asm tests
passed. Workspace and no-asm doc tests, cargo doc, the kip-10 example,
formatting, strict Clippy and platform/build checks also passed.

This independently validates the fixture-only correction at
`f1808f1231697474de66bf3f99bad0c2f01ead0d`: populated-database reopen
skips duplicate genesis initialization exactly as production Factory does.
No assertion, authentication check, header insertion rule or restart
guarantee was weakened. The exact persisted false marker assertion passed.

GitHub comparison from the validating commit through handoff head
`3ef81662656decfb6ba89595c0c82cdc4c38bf29` contains only the two
handoff documents. This milestone commit also changes documentation only.
PR #12 remains draft, unmerged and inactive. Its source is validated.

Next: a new stacked draft PR for bounded failed-import cleanup.
Keep descendant recovery and export snapshots as later milestones.

## Bounded failed-import cleanup candidate

Branch: `feature/phase6-bounded-failed-import-cleanup-20261113`,
stacked on the validated PR #12 handoff checkpoint
`3734e6fa30d2d5e4c82b30034f42dbedcf30bbd6`.
See [the cleanup report](docs/development/PHASE6_BOUNDED_FAILED_IMPORT_CLEANUP_2026-11-13.md).

The candidate persists false readiness before deleting membership in
synchronous batches of at most 4,096 nullifiers under the same authenticated
worker permit. It uses no full-set snapshot or deletion batch. Deferred
typed deletes invalidate caches only after persistence. Wrong owner,
point, component, busy worker or stale revision rejects writes; failure
requires a fresh session after worker drain. Existing validated-descendant
refusal is retained. Matching old snapshot roots cannot bypass the false
readiness left by interrupted cleanup.

Three new cleanup regressions cover durable interruption/reopen/retry,
rejected marker and snapshot batches, cache behavior and exact permit
ownership/reservations. The existing F-15 test retains every assertion.
All eight CI jobs remain required; cleanup source is not yet validated.
Exact PR, source and run IDs will be appended after push. Descendant
recovery and export snapshots remain subsequent milestones. No merge,
release or consensus activation.

## Initial cleanup validation checkpoint

Initial source `dafe86c7cde41e613a7666b9e2068c58fdda902c` is in draft
[PR #13](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/13),
stacked directly on PR #12.
[Run 43522414095](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43522414095)
reported four Rustfmt layout hunks. Their exact output is applied here,
without changing assertions or behavior. Replacement focused and complete
eight-job validation are required; no cleanup pass is claimed.
The PR #12 validation recorded above remains complete and unchanged.

## Cleanup compiler correction

Check job 132219833598 in run 43522414095 reported E0609 at the two
new batch constructors: ShieldedStateManager owns typed stores, not a
database field. The bounded helper now accepts the processor's existing
database handle explicitly; every typed deferred delete still verifies
database identity before any commit. The four formatter hunks are also
applied. No assertions, authentication or restart checks were relaxed.
Focused cleanup and full eight-job validation remain pending.

## PR #13 verified cleanup milestone

Verified November 22, 2026. Draft [PR #13](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/13) implements bounded
failed shielded-import cleanup and is stacked on draft PR #12.
Exact tested source: `8084f0e2edff50741e1781bd7a8b2495f7bebd01`.
[Actions run 43523739668](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43523739668)
completed successfully in all eight jobs. In native job 132224353099,
the dedicated persistent-reopen test and three cleanup regressions passed
before the workspace build. The 34 focused tests, all 1,579 workspace
tests (25 skipped), and 32 no-asm tests passed. Strict Clippy, formatting,
Check, Linux release, WASM checks and tests, WASM SDK, no-std, and docs passed.

The runtime continues to reject destructive cleanup when validated
descendants exist. Next executable task: trace the persisted shielded
readiness and post-pruning nullifier evidence through descendant recovery
and IBD, then add a fail-closed reconciliation path and focused restart
coverage on a draft branch stacked on PR #13. Authenticated export
snapshots and testnet integration remain pending. Keep preview default-off
and Simnet-only. Keep all PRs unmerged.

## Descendant recovery candidate

November 22, 2026. Branch `feature/phase6-descendant-recovery-20261122` is stacked on draft PR #13
at documentation checkpoint `f74a4421c0f48f71931ee2d5be646d2872567056`.
See [the development report](docs/development/PHASE6_DESCENDANT_RECOVERY_2026-11-22.md).

When IBD encounters an unready shielded state with validated descendants,
the Simnet preview can verify the existing state and durably restore the
shielded readiness flag without running a destructive import. It requires
the authenticated pruning child and a selected-header child that commits the
pinned virtual tip. The committed selected chain, descendant diffs, tip
membership, and pruning-point projection must all agree. The full
historical membership set is streamed, with exact duplicate detection
over selected descendant diffs. On error readiness remains closed.

The persistent reopen fixture now checks rejected evidence, changed tip,
wrong commitment child, gated readiness publication and promotion record
preservation. GitHub Actions must pass the dedicated regression and all
eight jobs before advancing to authenticated export snapshots. The preview
remains default-off and Simnet-only.

Draft [PR #14](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/14) first ran at
`e9dd9376c6bc8d8e0fbfe1c354315133a4c5a715` in [run 45048948814](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45048948814).
Rustfmt reported three layout changes, applied in this revision. The
revision also pins the latest selected header tip through the final ancestry
check and marker commit. Complete validation is pending.

[Run 45049094412](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45049094412) at
`a2cdd7a21fc2283af7e2f0e41c787b1592e6e2f2` reported one import layout change in Lints job 137290088805.
This revision applies the exact Rustfmt output. Verify the replacement
run for this branch head before treating descendant recovery as validated.

## Verified descendant recovery milestone

Verified November 22, 2026. Draft [PR #14](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/14)
passed all eight jobs in [run 45049268981](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45049268981)
at exact source `e312306bce7c603a20c0e9251c7e784ab1c7e715`. Native job 137290622323 passed the persistent
reopen control, all three cleanup controls, the dedicated descendant
recovery regression, 34 focused regressions, 1,579 workspace tests
(25 skipped), 32 no-asm tests, docs and the kip-10 example. Strict Clippy,
formatting, Check, Linux release, WASM checks and tests, the WASM SDK,
and no-std passed.

Readiness reconciliation accepts only a corroborated committed state. If
the selected tip lacks an authenticated child or committed data diverges
from its chain commitments, readiness stays closed. Next executable task:
capture and verify shielded metadata, count and spent-nullifier rows in
one server-side export snapshot before sending any frames. Stack this as a
new draft PR on PR #14 and retain the Simnet-only preview gate. Remaining
work includes consensus integration and end-to-end testnet validation.

## Authenticated shielded export snapshot candidate

November 22, 2026. Branch `feature/phase6-authenticated-shielded-export-20261122` is stacked on
draft [PR #14](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/14) at documentation-only
checkpoint `e178963969ea9da2bc5b0a26feb9d3e5243a5252`. See [the dated report](docs/development/PHASE6_AUTHENTICATED_SHIELDED_EXPORT_2026-11-22.md).

The Simnet preview captures the pruning-point metadata and projected
nullifier rows from one pinned virtual/selected-chain view. It verifies
the selected child's header and coinbase body and checks the projected
membership against the resulting proof-of-work commitment before any frame
is sent. Missing or corrupt evidence, an unready point, and inconsistent
descendant diffs fail closed. Network backpressure holds only owned export
data. The rest of the network uses the existing transfer path.

The persistent regression now requires an authenticated stored child,
rejects wrong or unready points and divergent membership, and checks that
an already captured response and the promotion record survive later
state changes. The preview retains one projected nullifier vector per
active export. All eight jobs and the focused regression are pending.
Next, validate this candidate, then continue consensus integration and
end-to-end testnet readiness. Keep preview default-off and Simnet-only.

Draft [PR #15](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/15) initial source
`47a4f69a42e955a85036b5089ca7b24d4aa0ce72` reached [run 45055027648](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45055027648).
Lints job 137308268988 identified two Rustfmt layout changes. This
revision applies both formatter hunks. Review the focused regression and
the full workflow on the replacement head before validating this milestone.

## Verified shielded export milestone

Verified November 22, 2026. Draft [PR #15](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/15)
passed all eight jobs at exact source `6f3466f2e030934ffae9d926a2478fe25e1d7b60` in
[run 45055166221](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45055166221). Native job
137308724299 passed the dedicated recovery/export regression, 34 focused
regressions, 1,579 workspace tests (25 skipped), 32 no-asm tests, docs
and the kip-10 example. Strict Clippy, formatting, Linux release,
WASM checks/tests and SDK build, Check, and no-std passed.

Next executable task: normal IBD on an active consensus must obtain the
verified header of the committed tip's selected child before attempting
shielded readiness reconciliation. Add a two-node Simnet regression that
reconnects an unready receiver with committed descendants and requires it
to resume validation while preserving the promotion record. The preview
remains default-off and Simnet-only. Testnet activation remains pending.

## Simnet descendant IBD integration candidate

November 22, 2026. Branch `feature/phase6-simnet-descendant-ibd-20261122`, based on
draft [PR #15](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/15) at `8bd8fff703f612618b151358800f54c71cc636c7`.
See [the dated report](docs/development/PHASE6_SIMNET_DESCENDANT_IBD_2026-11-22.md).

The active Simnet preview obtains missing headers before shielded
reconciliation when the receiver already has a committed descendant
tip and stable SMT and UTXO state. It authenticates the committed tip
against its selected child before any missing descendant body is applied.
An already UTXO-validated peer pruning point on the local selected chain
also permits normal IBD when only shielded readiness is unstable.

A two-node Simnet regression performs an authenticated IBD promotion,
disconnects the receiver, sets its shielded readiness false, mines a
header gap, and requires the receiver to reconnect and resume validating
blocks while preserving the promotion record and active role. The
focused regression and all eight jobs are required. After validation,
continue with the remaining consensus and testnet acceptance coverage in
KASCADE-SPEC.md. Keep the preview default-off and Simnet-only.

Draft [PR #16](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/16) first ran at
`e147119245e990531fc77f74051b5ceab7b75e08` in [run 45058650015](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45058650015).
Seven jobs passed. Native job 137319441954 reached successful authenticated
promotion, then the focused test panicked when the fixture tried to ban a
permanent connection. The descendant fixture now opens its first peer as
temporary and still requires actual disconnect, readiness reconciliation,
and validated catchup. Production code is identical. Verify the replacement
focused regression and all eight jobs before validating this milestone.

[Run 45060976100](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45060976100) at `55cad3df2695ba27f8286a24ebb2a30f207a6d88`
passed the focused two-node test and seven jobs. In native job 137326618985
the test's combined-suite run logged successful recovery IBD and 20
validated blocks, then timed out waiting for the additional announcing
block. That block can be announced while the preceding IBD is already
processing a bounded request. The fixture now checks the exact pre-reconnect
gap tip by its UTXO-valid status, plus readiness and promotion preservation.
The subsequent finality-depth catch-up assertions still verify live relay
of newly mined blocks. The production patch is unchanged. Rerun all jobs.

## Verified Simnet descendant IBD milestone

Verified November 22, 2026. Draft [PR #16](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/16) passed all eight jobs
at exact implementation SHA `c9f378177c0ddd25f3152b55c8d34b593c3cfcc4` in
[Actions run 45063994270](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45063994270).
Native job 137335973190 passed the focused two-node recovery test and
the 35-case Phase 6 suite, which independently reran the two-node case.
The persistent reopen, cleanup, and export regressions passed, as did
1,579 workspace tests (25 existing skips), 32 no-asm tests, documentation,
and the kip-10 example. Formatting, strict Clippy, Check, no-std, Linux
release, and all WASM checks passed.

The two-node case verifies the exact committed gap tip, restored
shielded readiness, promotion preservation, and live block validation
after reconnect. The preview stays default-off and Simnet-only.

## Concurrent Phase 6 work

The [testnet acceptance matrix](docs/development/PHASE6_TESTNET_ACCEPTANCE_2026-11-22.md) records current
Simnet evidence and the remaining acceptance criteria. [Draft PR #17](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17)
tracks Simnet shielded archive backfill and status verification. [Draft PR #18](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/18)
contains post-launch post-quantum wallet and transaction research on a separate branch.

The [testnet profile audit](docs/development/PHASE6_TESTNET_PROFILE_AUDIT_2026-11-22.md) records inherited testnet defaults and identifies explicit network, address, work-rate and issuance checks before approval. This documentation branch has no consensus or launch changes. Next executable task: run the three-node Simnet partition/rejoin regression and separately verify shielded archive backfill on its draft PR.

## GitHub Actions billing blocker

November 22, 2026. The last fully validated implementation is still
draft [PR #16](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/16) at
`c9f378177c0ddd25f3152b55c8d34b593c3cfcc4`, with eight successful
jobs in [run 45063994270](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45063994270).
Its documentation-only checkpoint is `049ffaf17d3fdc74c93a36b399edb0b4b66cb0b3`.

- Archive backfill [PR #17](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) is pinned at
  `11ab68a3be26fc9616a91d3d06ac5d99bc23679b`.
  [Run 45069228114](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114)
  retains six successful jobs. The replacement native and Linux release
  jobs did not start. Documentation checkpoint
  [81ce77b](https://github.com/DeepFriedBlockchains/KascadeNetwork/commit/81ce77bd7ca2593a84be294f37c50f88b651e424)
  includes the parent handoff.
- Three-node partition/rejoin and restart [PR #19](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/19)
  is pinned at `978584bbd91e81b8b637bbebb6655426fe207d9d`.
  [Run 45069680867](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867)
  retains four successful jobs. Linux release, WASM SDK, Check and the
  native suite did not start on retry.

GitHub Actions annotations identify an account billing lock and report
that the jobs did not start. Neither implementation has full validation.
Once the account owner has resolved the lock, rerun the incomplete jobs
on the exact SHAs, inspect the native regressions and full matrix, and
record the results on each draft. Diagnose any actual test failures
from job logs before changing implementation. Keep both drafts unmerged.

## Validated checkpoints after Actions resumed

November 22, 2026. CI is running again. The stack remains draft and
unmerged, with the Phase 6 preview default-off and Simnet-only.
Documentation-only checkpoints preserve each exact tested source.

| PR | Validated source | Actions result |
| --- | --- | --- |
| [#17 archive backfill](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/17) | `11ab68a3be26fc9616a91d3d06ac5d99bc23679b` | [45069228114, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069228114/attempts/3), 8/8. Both focused tests, 36 Phase 6 cases, 1,579 workspace tests (25 skipped), 32 no-asm. Docs checkpoint `f2aeedfeeabd4b9e9a6be75992ced2420ca48446`. |
| [#19 three-node partition/rejoin](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/19) | `978584bbd91e81b8b637bbebb6655426fe207d9d` | [45069680867, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45069680867/attempts/3), 8/8. The three-node case ran in 26.487 seconds, alongside the rest of the 36 Phase 6 cases, 1,579 workspace tests (25 skipped), and 32 no-asm. Docs checkpoint `80d94f049c976944df511e949afa101443940119`. |
| [#20 testnet profile audit](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/20) | `50b0d19290c54ffef5df3e0919d99645bb528e19` | [45078524217, attempt 3](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45078524217/attempts/3), 8/8. Native evidence from attempt 2 includes 35 Phase 6 cases, 1,579 workspace tests (25 skipped) and 32 no-asm. Attempt 2's WASM job stalled on txscript, then failed with an unavailable log. The single-job retry succeeded on the exact SHA. |
| [#21 native proof-of-work relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/21) | `bd4f65851c0267f70aced3602f61391c5fdfc578` | [45081005178](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45081005178), 8/8. The dedicated native mining test, 36 Phase 6 cases, 1,579 workspace tests (25 skipped), and 32 no-asm passed. Docs checkpoint `8ac6725cc4fc69c1c688a8cf2398415b3ddfe5eb`. |

## Current Phase 6 work and CI

| Draft | Branch | Implementation SHA | Actions |
| --- | --- | --- | --- |
| [#22 AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `test/phase6-auxpow-relay-20261122` | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439), 7/8. The dedicated native/AuxPoW relay step passed. Native job `137401920040` is running the workspace build. Canceled duplicate push run `45085111236`. |
| [#23 5-BPS loopback](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/23) | `docs/phase6-five-bps-evidence-20261122` | `d042358d92a8ab0ae99b549d18dad1c5254533a2` | [45086577123](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086577123), 6/8. Linux release and the native Test Suite remain in progress. Native job `137406503498` running. |
| [#24 bundled AuxPoW miner](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/24) | `test/phase6-miner-auxpow-interop-20261122` | `436a2ba92f619212576dcae41ca3d73b0a908edc` | [45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062), 3/8. no-std, WASM check, and WASM tests passed. The focused miner step passed in native job `137407465112`. |
| [#25 combined acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) | `test/phase6-acceptance-convergence-20261122` | `25cfeed643d40b35f906332e4b024dce23632ae0` | Staged #17, #19, and the focused-passing #22 source. Combined CI deferred until #22 finishes. |

No job failures were reported on the active runs at this checkpoint.
Review #22's dedicated relay test output first. Collect #23's checked
configuration and per-block timing artifacts and review #24's bundled
miner regressions in parallel. Use the [dated checkpoint report](docs/development/PHASE6_RELAY_EVIDENCE_CHECKPOINT_2026-11-22.md)
for the acceptance candidate sequence and remaining gates. Draft
[#18](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/18) covers post-launch quantum research. Preserve
the separate draft heads. Phase 6 remains default-off and Simnet-only,
with shielded consensus activation disabled.

The [Kaspa parent interface report](docs/development/PHASE6_KASPA_PARENT_INTERFACE_2026-11-22.md) narrows the next external
interop fixture to upstream gRPC, including the coinbase payload budget,
independent targets, and stale jobs. The remaining full CI runs still
determine whether the new implementation commits are validated.

## November 22, 2026 · 29B monetary design and acceptance update

| Draft | Branch | Source SHA | Latest evidence |
| --- | --- | --- | --- |
| [#22](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `test/phase6-auxpow-relay-20261122` | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439), 7/8. Focused relay and authority suites passed. Native workspace tests running. |
| [#23](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/23) | `docs/phase6-five-bps-evidence-20261122` | `999f3e1342d82c1feebfeb47d55023807be17276` | [45089622439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439), 1/8 at latest check. Native job `137416068338`. Prior [45086577123](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086577123) on `d042358d92a8ab0ae99b549d18dad1c5254533a2` passed the loopback at 4.9996 BPS and failed only in artifact extraction. The correction `33803e13550f87f227114de579002c62b281d4b6` and current head preserve the fixture's assertions. |
| [#24](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/24) | `test/phase6-miner-auxpow-interop-20261122` | `436a2ba92f619212576dcae41ca3d73b0a908edc` | [45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062), 7/8. Focused miner and authority suites passed. Native workspace tests running. |
| [#25](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) | `test/phase6-acceptance-convergence-20261122` | `25cfeed643d40b35f906332e4b024dce23632ae0` | CI staged. Trigger when #22 finishes 8/8. |
| [#26](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/26) | `design/29b-emission-validation-20261122` | `51b747ae11b11b636612245b756135a564dde4e4` | [45089866749](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089866749), pending with native job `137416754428`. Retargeted to validated #21 after initial [45089528345](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089528345) failed WASM in the inherited getrandom dependency on the old docs base. |
| [#27](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/27) | `test/phase6-rpc-safe-mode-20261122` | `6cc206dce844cdfd43d115f90c3af31d1f03e4c3` | Staged with CI deferred. Binds test listeners to loopback and checks default gRPC safe-mode denial. |

The public [policy documentation branch](https://github.com/DeepFriedBlockchains/KascadeNetwork/tree/docs/29b-monetary-policy-alignment) is at `fcccc2af4904e8505bb2326bc9f0abb331223635`. The owner approved 29B total, 21B primary and 8B finite long-term tail. The precise schedule, duration, start, accounting for genesis and issuance already present, and the unique rewarded-slot rule require approval. Existing per-transaction bounds already use 29B. The inherited live emission code still has a perpetual tail. The superseded 23B recommendation is historical and the cumulative-cap enforcement finding remains open.

Cleanup, persistent reopen, descendant reconciliation and authenticated export are already validated on #13–#16. Preserve those sources. The [checkpoint report](docs/development/PHASE6_29B_AND_ACCEPTANCE_CHECKPOINT_2026-11-22.md) lists their exact runs, public safety review evidence, and remaining tests. Phase 6 remains default-off and Simnet-only. Shielded consensus activation is disabled.

A separate integer calculation confirmed the legacy proposed cascade sums to 21B and all four finite-tail examples sum to 8B with zero post-tail issuance. This checks the proposed arithmetic only. GitHub Actions still needs to run the reference vectors, and production supply enforcement remains open. The audit baseline was reviewed privately. The continuation archive is pending a readable review. See [#26's design](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/51b747ae11b11b636612245b756135a564dde4e4/docs/development/PHASE6_29B_EMISSION_DESIGN_2026-11-22.md).

## Recovered 35-year work and current validation

| Draft | Branch | Exact head | Verified result |
| --- | --- | --- | --- |
| [#22 AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `test/phase6-auxpow-relay-20261122` | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439) 8/8. Two relay, 37 Phase 6, 1,579 workspace, 32 no-asm. |
| [#23 5-BPS evidence](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/23) | `docs/phase6-five-bps-evidence-20261122` | `999f3e1342d82c1feebfeb47d55023807be17276` | [45089622439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439) 7/8. Focused test and artifact upload passed, artifact 14781544000. Native workspace running. |
| [#24 Bundled miner](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/24) | `test/phase6-miner-auxpow-interop-20261122` | `436a2ba92f619212576dcae41ca3d73b0a908edc` | [45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062) 8/8. Two focused miner, 36 authority, 1,581 workspace, 32 no-asm. |
| [#25 Combined acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) | `test/phase6-acceptance-convergence-20261122` | `c542833e385f1f222538f42bc72a9843a1cd2ca4` | [45093748286](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286) 5/8. Implementation is `25cfeed643d40b35f906332e4b024dce23632ae0`; current report-only head triggered the unchanged combined source. Native job 137428872019. |
| [#26 35-year emission](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/26) | `design/29b-emission-validation-20261122` | `20c9d2d6528eabeefe341a11e642b3d901dc1a66` | [45094682311](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094682311) active, 1/8. First candidate [45094494258](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094494258) failed rustfmt; this head applies the reported formatting. Older [45089866749](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089866749) failed compilation in the superseded test. |
| [#27 RPC safety](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/27) | `test/phase6-rpc-safe-mode-20261122` | `9b7c2f07dac1172355979ff3adb4918c7505aaa5` | [45093817955](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093817955) 6/8. Native job 137429089738 is running the focused regression. |
| [#28 KRN economics](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/28) | `docs/phase6-krn-35-year-economics-20261122` | `be39e21123143e72d10fc26fbe868daa551d56a4` | Docs-only. Includes participation, variance, operating break-even and terminal-cutoff analysis. |
| [#18 Post-quantum research](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/18) | `docs/post-quantum-wallet-research-20261122` | `29d34d425a22ae23379cc602065aa5aa0fd7a619` | Docs-only. Versioned witness paths, ML-DSA byte/mass estimates and native/WASM acceptance criteria. |
| [#29 Mining interoperability](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/29) | `docs/phase6-upstream-mining-acceptance-20261122` | `2be05be8b5233e4941fcb33ccd794ecb315edc95` | Docs-only. Pinned bridge interface review, direct parent-node RPC and pool-adapter acceptance gaps. |
| [#30 DAG reward claim fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `test/phase6-reward-claim-accounting-20261122` | `ca9c498709c3c5d4c9d664806adc9c83b767c666` | [45094765692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094765692) 1/8. Real Simnet templates and coinbase validation across equal-DAA competing blocks and a heavier reorg and join. |

The [35-year candidate](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/82f4eac3ae5e6ddfaabdd499e19f44373718f43e/docs/development/PHASE6_35_YEAR_EMISSION_CANDIDATE.md) assumes 5 uniquely rewarded slots per second. Exact schedule counts are 1,577,880,000 primary and 3,944,700,000 tail. The independently reproduced schedules pay exactly 21B and 8B, and no subsidy afterward. The test/preview module has no runtime consumer. Historical 23B guidance is superseded. Production hard-cap enforcement remains open.

The DAG integration needs a production-path trace of credited source block hashes and coinbase outputs along competing selected-parent chains. Persist and authenticate the credited ordinal with accepted state before considering any schedule wiring. The preferred clock remains a hypothesis until duplicate prevention, reorg/reopen/IBD and genesis accounting are demonstrated. Modelled five-slot throughput must be measured.

The authorized private continuation was reviewed against the combined source. Public readiness gates include supply enforcement, shielded activation review, public network parameters, P2P load and resource budgets, and external RPC access boundaries. Durable readiness and atomic writes are present in the preview and require combined confirmation. Current public-facing security work is #27. Preserve the validated #13–#17 and #19–#24 checkpoints and keep all Phase 6 branches draft, default-off and Simnet-only.

## Next execution order

1. Review any completed #25, #26 and #27 runs. For #26, check the six dedicated integer tests and inspect formatting, strict Clippy and all eight jobs.
2. Continue the reward claim fixture using native coinbase/mergeset observation and compare accepted outputs after a reorg. Report independently verifiable proof before any activation discussion.
3. Retrieve #23 artifact evidence and record the sustained-rate and external Kaspa/pool interoperability gaps. Preserve #24's validated source when expanding mining docs.

[Dated recovery report](docs/development/PHASE6_35_YEAR_RECOVERY_2026-11-22.md).


### Reward ownership findings

The mergeset begins with the selected parent and adds blocks in the current block's past outside that parent's past. On a selected-parent chain the past sets are nested, so those per-step set differences are disjoint. The eligible subsidy sources are a subset of each difference and include both blue and qualified red blocks. Non-DAA red blocks can carry fee rebates without a subsidy claim. This identifies a candidate source identity for duplicate exclusion. It does not prove the implementation across reduced parents, pruning, snapshots, and alternative selected chains. The new #30 fixture examines the transparent branch and join case first.

Coinbase construction commits the current block's reward entitlement and includes outputs for source claims. The virtual processor applies the selected parent's coinbase to accepted state. Keep assigned claims, paid outputs, and accrued balances distinct. Any production ordinal must be authenticated with a specific accepted chain point, reconstructed through sync, and reverted before replay. Exercise genesis's nominal subsidy payload separately from spendable outputs. At four unique rewarded slots per second the proposed 35-year count takes 43.75 nominal years; at six it takes about 29.17. The cap is independent of realized rate if each slot is consumed once. Measure credited source counts with block rate, lag, fee totals and reorg rollback on a real mining run.

## Phase 6 validated CI and reward replay checkpoint

All eight jobs passed on each listed source. Follow-up report commits for #23 and #25 preserve the tested code.

| Draft | Branch | Exact tested source | Actions |
| --- | --- | --- | --- |
| [#22 AuxPoW relay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/22) | `test/phase6-auxpow-relay-20261122` | `3aae9a87e353b2a164fadc77528b0871500e34ee` | [45085126439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45085126439) |
| [#23 5-BPS measurement](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/23) | `docs/phase6-five-bps-evidence-20261122` | `999f3e1342d82c1feebfeb47d55023807be17276` | [45089622439](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45089622439), 4.9997 BPS over 50 blocks, peer p95 12.410/12.325 ms. Evidence-only head `aec563aa421ae9348ddb08cd88d9d37feb3302ee`. |
| [#24 Bundled miner](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/24) | `test/phase6-miner-auxpow-interop-20261122` | `436a2ba92f619212576dcae41ca3d73b0a908edc` | [45086888062](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45086888062) |
| [#25 Combined Simnet acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/25) | `test/phase6-acceptance-convergence-20261122` | `c542833e385f1f222538f42bc72a9843a1cd2ca4` | [45093748286](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093748286), 39 targeted, 1,579 workspace passes. Evidence-only head `15f99ca97799b414e57fb20b8f53f1d85feb1f6c`. |
| [#26 35-year candidate](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/26) | `design/29b-emission-validation-20261122` | `20c9d2d6528eabeefe341a11e642b3d901dc1a66` | [45094682311](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094682311), six candidate tests, 36 focused and 1,585 workspace passes. |
| [#27 RPC safe mode](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/27) | `test/phase6-rpc-safe-mode-20261122` | `9b7c2f07dac1172355979ff3adb4918c7505aaa5` | [45093817955](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45093817955), focused, 36 Phase 6 and 1,580 workspace passes. |
| [#30 Reward claims](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `test/phase6-reward-claim-accounting-20261122` | `ca9c498709c3c5d4c9d664806adc9c83b767c666` | [45094765692](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45094765692), focused and workspace fixture passed. |

#30 follow-up `29707a918033aa62a0d763509f8629ee0e6562c4` adds a real RocksDB close/reopen and a clean replay with a different branch arrival order. It compares the complete UTXO set and accepted coinbase records, checks exact join payouts, and checks removal of an abandoned branch's payout. [Run 45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081) is pending.

The 35-year model and the 90/8/2 KRN reference analysis conserve the approved 21B primary and 8B tail budgets. The KRN review checked 147 economic scenarios and 832 exact arithmetic vectors, covering bootstrap, KRN and KCF fallback. The production KRN payout path and live issuance enforcement remain open. Authenticated restoration of any new ordinal must bind it to consensus-verified state and follow accepted reward replay. [Restart and issuance gates](docs/development/PHASE6_ISSUANCE_REPLAY_AND_RESTORE_2026-11-22.md)

Next: verify #30's pending replay CI, then extend real fixture coverage to qualified red and non-DAA fee-only rewards and authenticated pruning restoration. Use sustained sampling of credited source hashes to measure the realized schedule duration. External mining interoperability, sustained operation, external RPC/resource policy and authorized network parameters remain public readiness gates. The preview is default-off and Simnet-only.

## Production-path fee and genesis follow-up

[Draft #31](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31) isolates the real coinbase builder's treatment of qualified red and non-DAA red sources. Exact source `5228e73e9b8a3981f611a5cee512fbe24b50f5f3`, branch `test/phase6-merge-fee-accounting-20261122`, [run 45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993). It checks output scripts and amounts and distinguishes 3,004 Kazmi of new subsidy from 67 Kazmi of recycled fees. It supplies merge membership explicitly. A real DAG forcing qualified and non-DAA red membership remains a follow-up to the green transparent join baseline.

The Simnet genesis transaction starts with zero outputs, but its coinbase payload declares 100,000,000 Kazmi and carries an OP-FALSE reward script. Trace the first accepted payment independently before treating genesis as excluded from the budget. The shielded verification path already checks net coinbase growth minus fees against eligible source subsidy, with a non-DAA blue fee adjustment and a deferred developer entitlement adjustment. That check is per transition. An authenticated ordinal with an activation point is still required for global issuance accounting.

## Reward-conservation continuation

The tested baselines above remain valid on their exact sources. Current runs and follow-ups:

| Draft | Branch | Exact head | Validation |
| --- | --- | --- | --- |
| [#30 Persistence and replay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `test/phase6-reward-claim-accounting-20261122` | `29707a918033aa62a0d763509f8629ee0e6562c4` | [45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081) running, 5/8 at check. Native job 137461585802. |
| [#31 Red subsidy and fees](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31) | `test/phase6-merge-fee-accounting-20261122` | `5228e73e9b8a3981f611a5cee512fbe24b50f5f3` | [45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993) running, 3/8 at check. Native job 137460666926. |
| [#32 Genesis and total issuance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `5ef10a79dcde735e2540e1e5309b498bac3765b8` | Staged; CI deferred until the replay parent validates. |

Draft #32 preserves #30's reorg, RocksDB reopen and arrival-order assertions, then adds checks for zero output at genesis and at its first child's settlement. It requires all seven eligible source claims to account for every UTXO and every atomic unit in the fee-free transparent fixture. A nominal genesis payload value alone does not constitute issuance. The [fixture notes](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/5ef10a79dcde735e2540e1e5309b498bac3765b8/docs/development/PHASE6_GENESIS_ISSUANCE_CONSERVATION_2026-11-22.md) identify the paths outside this proof.

Next: check the focused native jobs before reporting #30 or #31 validated. Resolve any failures from their logs. Once #30 is green, run the dedicated #32 test by enabling its branch in the existing focused step, then run its matrix. Bound and authenticate any future issuance counter before using it across pruning or IBD. Full cap enforcement remains blocked on that and on payout settlement proof. Schedule and public activation remain approval gates.

## Parallel acceptance and owner review

| Draft | Branch | Exact head | Checkpoint |
| --- | --- | --- | --- |
| [#33 Sustained loopback](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/33) | `test/phase6-five-bps-sustained-20261122` | `e48bf578e32eecbd40b6db2f7e596899c0c370e7` | [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195) pending. 1,500 blocks, three nodes, 4.95–5.05 accepted BPS, exact CSV and latency stats. |
| [#34 Red eligibility and payout boundary](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `d059c33cbdf5b49ed4165b19605fa81b1c5d9423` | Staged behind #31. Builder vectors isolate recycled fees and developer accrual/settlement. The live k=1 Simnet case must produce one eligible red from three competing siblings and pay it to the merging miner. |
| [#35 Issuance clock and restore](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/35) | `design/phase6-issuance-clock-restore-20261122` | `4ec79a6930a5d51880a73fe23db46b565e981cf2` | Docs-only owner review. |
| [#36 P2P resource acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/36) | `docs/phase6-p2p-resource-acceptance-20261122` | `24c68c77e392bc067c014da92671b3d80866998b` | Docs-only. Approved envelopes, instrumentation and actual measurements remain open. |

Recommend a fresh zero-issuance genesis and a branch-specific ordinal for eligible, unique subsidy-source claims in existing deterministic mergeset order. Use DAA progression as a cross-check and exclude fee-only sources; DAA cannot distinguish individual sibling claims. A snapshot must authenticate the ordinal and deferred balances as committed consensus state, and reject an inconsistent checkpoint. Resolve replay, atomicity, and pruning/IBD proof before staging production cap enforcement. No emission activation is authorized. The approved 21B/8B budgets remain unchanged.

Check #30 and #31 native jobs first. If they pass, trigger the corresponding staged #32 or #34 follow-up with its focused tests enabled, retaining the validated parent checkpoint. Check #33's artifact for 1,500 actual accepted blocks before making sustained-rate claims. This paced fixture uses simulated PoW, so native work and representative network load remain acceptance gates. Keep private audit material in authorized review. The preview stays default-off and Simnet-only.

## Authenticated issuance model and CI continuation

- [#30](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30), `test/phase6-reward-claim-accounting-20261122`, `29707a918033aa62a0d763509f8629ee0e6562c4`: [45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081) at 5/8. The dedicated competing-tip test passed with the real RocksDB reopen and clean replay assertions. Native job 137461585802 is building the workspace. Keep #32 staged until the full parent matrix passes.
- [#31](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31), `test/phase6-merge-fee-accounting-20261122`, `5228e73e9b8a3981f611a5cee512fbe24b50f5f3`: [45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993) at 6/8, native job 137460666926 still before the dedicated fee/subsidy test. Keep #34 staged until the full parent matrix passes.
- [#33](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/33), `test/phase6-five-bps-sustained-20261122`, `e48bf578e32eecbd40b6db2f7e596899c0c370e7`: [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195) started, 1/8. Native prepares disk space; no five-minute samples yet.
- [#37](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/37) is the separate in-process P2P queue acceptance draft. Its current source is `4ae10e0732f53de7752db86a7737ff1eb3db4fb8`; the original [45105054542](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105054542) was on prior source `d6e401db1a7fc8e5490d4efcaea1e2f1840e1b2b`. Replacement [45105129333](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105129333) belongs to the current source.
- [#38](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/38), `test/phase6-authenticated-issuance-model-20261122`, `82dfd474f5ce1e31112bc7f7eab00db42db42d21`: [45105132180](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105132180) started, 1/8. Test-only bounded model binds network, point, ordered claims, paid and deferred balances to a commitment supplied through separately authenticated context. It exercises duplicate and genesis rejection, branch rollback and replay, fee recycling, deferred settlement, changes to the order or payout split, cap and overflow. It has no production caller. [Model scope](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/82dfd474f5ce1e31112bc7f7eab00db42db42d21/docs/development/PHASE6_AUTHENTICATED_ISSUANCE_MODEL_2026-11-22.md).

Next: finish and record exact #30/#31 matrices, then enable the focused branches and trigger #32/#34. Check #33's dedicated measurement and uploaded artifacts. Verify #37/#38 on their exact current heads. The #38 in-memory proof is a reference for production code, where eligibility, order and uniqueness must come from verified DAG state and the expected commitment must be part of verified pruning ancestry and body data. A live authenticated checkpoint and durable issuance transition remain launch gates. Keep the preview default-off and Simnet-only.

#32 now has a workflow-only staged head `2296bfd82903111bb629641c316c6f8484bc354d` on `test/phase6-genesis-issuance-conservation-20261122`. Its dedicated reward-claim step selects both #30 and #32. The #32 test source is unchanged from `5ef10a79dcde735e2540e1e5309b498bac3765b8`. Once #30 is 8/8, push its normal evidence update on #32 to start CI. #34's dedicated tests are staged at `d059c33cbdf5b49ed4165b19605fa81b1c5d9423` and can run after #31 passes. Actual authenticated RocksDB and pruning-restore issuance remain pending.

## Atomic checkpoint and sustained resource follow-ups

| Draft | Exact source | Branch | Evidence |
| --- | --- | --- | --- |
| [#30 replay](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `29707a918033aa62a0d763509f8629ee0e6562c4` | `test/phase6-reward-claim-accounting-20261122` | [45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081), 7/8. Focused RocksDB reopen and alternate arrival replay passed. Native job 137461585802 is running broader authority tests. |
| [#31 red rewards](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31) | `5228e73e9b8a3981f611a5cee512fbe24b50f5f3` | `test/phase6-merge-fee-accounting-20261122` | [45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993), 7/8. Native job 137460666926 has not yet reached the focused fee case. |
| [#32 genesis and conservation](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `2296bfd82903111bb629641c316c6f8484bc354d` | `test/phase6-genesis-issuance-conservation-20261122` | Staged behind #30. Dedicated step already enabled for this branch. |
| [#34 DAG and deferred payout](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `d059c33cbdf5b49ed4165b19605fa81b1c5d9423` | `test/phase6-merge-payout-boundary-20261122` | Staged behind #31. Dedicated builder and real Simnet tests enabled for this branch. |
| [#33 sustained loopback](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/33) | `e48bf578e32eecbd40b6db2f7e596899c0c370e7` | `test/phase6-five-bps-sustained-20261122` | [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195), 3/8. The five-minute fixture and artifacts have yet to run. Native job 137463920152. |
| [#36 resource acceptance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/36) | `29adc15278428b38f10cb1e5c160cd62fb316818` | `docs/phase6-p2p-resource-acceptance-20261122` | Documents validated loopback dependency and available RPC metrics. No resource-budget proof. |
| [#37 P2P queue tests](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/37) | `4ae10e0732f53de7752db86a7737ff1eb3db4fb8` | `test/phase6-p2p-router-resources-20261122` | [45105129333](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105129333), 3/8. Native job 137464600471 is running focused in-process queue tests. |
| [#38 checkpoint model](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/38) | `82dfd474f5ce1e31112bc7f7eab00db42db42d21` | `test/phase6-authenticated-issuance-model-20261122` | [45105132180](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105132180), 3/8. Native job 137464568170 is before the focused issuance model. |
| [#39 checkpoint persistence](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `cc5d73dd3628c3b1bb1cc96c901dbdf3722fd40d` | `test/phase6-issuance-checkpoint-persistence-20261122` | Staged behind #38. Atomic checkpoint and readiness batch, clean restart/unsubmitted replacement, changed deferred payment and missing readiness checks. |
| [#40 sustained resource sampling](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `89f81b1c2b719c0223d11acbaaf5187613f5691c` | `test/phase6-sustained-resource-sampling-20261122` | Staged behind #33. CPU and RSS of the shared Linux test process and original sustained-rate evidence. |

No failure was reported on the active exact heads at this checkpoint. Historical green results remain pinned above. Preserve branch history and run #32, #34, #39 and #40 when their parents validate, each using the branch-specific step already staged. The next real restore gate must authenticate the expected checkpoint through validated pruning ancestry, commit it with readiness and actual settlement, and verify restart and post-pruning reorg against replay. #39 uses an external fixture anchor and does not establish those production guarantees or crash-during-write behavior. Perform network recovery and native/AuxPoW testing separately from paced simulated-PoW loopback. Keep the existing 29B maximum, 21B primary and 8B finite tail, test-only model, and default-off Simnet-only preview.

[Detailed checkpoint](docs/development/PHASE6_ATOMIC_ISSUANCE_AND_SIMNET_CHECKPOINT_2026-11-22.md).

## Verified matrices and four active follow-ups

| Draft | Branch | Exact source | Result |
| --- | --- | --- | --- |
| [#30](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/30) | `test/phase6-reward-claim-accounting-20261122` | `29707a918033aa62a0d763509f8629ee0e6562c4` | [45103683081](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103683081), 8/8. Focused replay and RocksDB reopen; 39 authority, 1,580 workspace. |
| [#31](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/31) | `test/phase6-merge-fee-accounting-20261122` | `5228e73e9b8a3981f611a5cee512fbe24b50f5f3` | [45103888993](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45103888993), 8/8. Focused fee-only red subsidy; 39 authority, 1,581 workspace. |
| [#33](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/33) | `test/phase6-five-bps-sustained-20261122` | `e48bf578e32eecbd40b6db2f7e596899c0c370e7` | [45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195), 8/8. Five minutes, 1,500 accepted and matching final tips; 35 authority, 1,579 workspace. |
| [#37](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/37) | `test/phase6-p2p-router-resources-20261122` | `4ae10e0732f53de7752db86a7737ff1eb3db4fb8` | [45105129333](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105129333), 8/8. Two focused P2P queue and cleanup cases; 39 authority, 1,581 workspace. |
| [#38](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/38) | `test/phase6-authenticated-issuance-model-20261122` | `82dfd474f5ce1e31112bc7f7eab00db42db42d21` | [45105132180](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105132180), 8/8. Three issuance commitment, replay and cap model cases; 36 authority, 1,588 workspace. |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `8612ee121513905c830a26cbf700a15b12b3d55d` | [45135461607](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135461607), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), queued or pending. Parent passed all eight jobs; new commit changes only its report. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `3b02227faec756934e1c140148f5480cd1ef2cfd` | Staged behind #39; focused selected-child body and persistence fixture enabled. |

## Sustained measurement

One isolated transparent simulated-PoW run accepted all 1,500 sequential submissions. First-to-last interval was **299.801365 seconds**. Observed rate over 1,499 intervals was **4.9999772349 BPS**. All three daemons selected tip `1f4fae8828a71efd7b6ae3ea7270853cb212e957683d06dfd6006c68792ac4d6`. Compared with the 50-block baseline, observed rate remained close to 5 BPS and p95 remained near 12.4 ms.

| Peer validation latency | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: |
| First | 6.436 ms | 12.419 ms | 12.625 ms | 12.977 ms |
| Second | 6.412 ms | 12.399 ms | 12.607 ms | 13.024 ms |

[Artifact 14787499841](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195/artifacts/14787499841) contains raw CSV, settings, tip, and aggregate JSON. This shows transparent loopback validation under pacing. CPU/RSS measurements are pending #40. Native PoW, AuxPoW, mixed transactions and network partition load remain separate acceptance conditions.

## Binding and next gates

#41 connects the real selected-child verifier to the test-only issuance checkpoint reopen fixture. It exercises the stored header and selected-parent row, child body Merkle binding and a changed paid/deferred split. The shielded field is a test stand-in. Pruning ancestry trust, a production issuance field, atomic consensus promotion and crash coverage still need proof. Preserve the approved 29B cap and 21B/8B budgets without enabling the schedule.

Review #32 and #34 focused results, then the rest of their matrices. Review #39 for real test-database atomic checkpoint and restart coverage, and trigger the already-staged #41 focused test after #39 validates. Retrieve #40's raw CPU/RSS once complete. #37 was the count-based in-process acceptance gate. Keep byte budgets, node-wide resource measurements and private security evidence in their separate review paths. Keep branches draft and the preview default-off and Simnet-only.

[Dated evidence report](docs/development/PHASE6_VALIDATED_MATRICES_AND_FOLLOWUPS_2026-11-23.md).

## Checkpoint decoder and loopback startup

Verified against GitHub. Prior exact sources and runs for #30, #31, #33, #37 and #38 still show 8/8. The #33 loopback evidence remains 1,500 accepted simulated-PoW blocks over 299.801365 seconds with a common selected tip and peer p95 times of 12.419 and 12.399 ms. [Run 45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195) · [Artifact 14787499841](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195/artifacts/14787499841)

| Draft | Branch | Exact head | Result |
| --- | --- | --- | --- |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 3/8. Native 137560421199 compiling or running persistent reopen, own focused case pending. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 2/8. Native 137560443988 still ahead of the targeted builder and Simnet membership tests. |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) queued. Prior [45135461607](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135461607) at `8612ee121513905c830a26cbf700a15b12b3d55d` failed cargo fmt on the checkpoint decoder's iterator chain. This revision uses the exact rustfmt output. No assertion, parsing logic or activation changed. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 3/8. Native 137559872891 has not published the CPU/RSS artifact. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged merge includes #39's exact formatter fix. Test-only child-body verifier fixture, waiting on #39. |
| [#42](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `8e49b5a7873d0c833088c827bc89d36e87174fa1` | Source-reviewed, docs-only operator procedure for three isolated loopback daemons, with validated integration baselines. The manual commands have not been executed. |

Next: review #39 replacement focused issuance tests and the full matrix. When green, trigger #41's already configured focused test and full matrix with a normal report update. Check #32/#34 for actual focused failures and #40 for measured CPU/RSS. Continue to a real pruning-proof restore and live settlement binding after #41. Public startup still needs approved network identity/genesis/bootstrap, resource limits, and representative real mining, churn and recovery acceptance. Preserve the 29B total, 21B primary and 8B finite tail and keep the preview default-off and Simnet-only.

[Dated report](docs/development/PHASE6_CHECKPOINT_DECODER_AND_LOOPBACK_RUNBOOK_2026-11-23.md).

## Three-node CLI exercise and current matrices

| Draft | Branch | Exact source | CI at checkpoint |
| --- | --- | --- | --- |
| [#39 Atomic checkpoint](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) 4/8; bounded reopen running, focused model pending. |
| [#41 Selected-child binding](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged behind #39. |
| [#32 Genesis and issuance](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 6/8; focused genesis/replay pending. |
| [#34 Red-block and deferred payout](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 6/8; focused builder/live merge pending. |
| [#40 Five-minute CPU/RSS](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 6/8; measurement pending. |
| [#42 Actual CLI runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320) pending runner allocation. Dedicated native step exercises real processes. |

No failures were reported on these active runs at the checkpoint. #42 now exercises the original three documented loopback commands with isolated temporary paths. The added simulated-PoW recovery pass only restarts A with its explicitly documented test mining flag. It requires advancing two blocks, C catching up from its existing path after missing the second block, matching pruning point/virtual DAA values, and clean SIGINT process shutdown. Collect `phase6-cli-runbook` logs before marking it verified. Earlier multi-node in-process and simulated-PoW evidence stays pinned.

Next actions: verify #39's focused persistence test and all eight jobs before triggering #41. Check #32 and #34's own focused assertions and record CPU/RSS from #40. Review #42's real process driver and per-node logs first on a failure. Keep the 29B total, 21B primary, 8B tail and disabled production issuance. Representative real mining and authenticated pruning restore with actual settlement remain acceptance gates.

[Checkpoint report](docs/development/PHASE6_CLI_RUNBOOK_EXERCISE_CHECKPOINT_2026-11-23.md).

## Active native CI and staged real-PoW recovery

| Draft | Exact source | Actions status |
| --- | --- | --- |
| [#39 Atomic checkpoint](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) 6/8. Native 137563153013 before the focused model. |
| [#41 Selected-child fixture](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged. Trigger after #39 passes. |
| [#32 Genesis conservation](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 7/8. Focused test passed. |
| [#34 Red rewards and deferred payout](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 7/8. Both the fee/subsidy builder and real Simnet eligible-red steps passed. |
| [#40 CPU/RSS](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 7/8. Focused five-minute run and artifact upload passed; broad native build running. |
| [#42 CLI runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320) 4/8. Native 137568406534 is executing the dedicated CLI step, evidence upload pending. |
| [#43 Real native PoW CLI](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `1c03bce8d4948b35bf9a9898d2990d2b6dac3753` | Staged on `test/phase6-cli-real-pow-recovery-20261123` behind #42. No CI yet. |

#43 adds a separate easy native PoW case with three daemon processes, validates rejection of an above-target nonce, mines and relays twelve blocks, and requires a follower to catch up by exactly six blocks after reopening. It records the final DAG tip, pruning point and virtual DAA agreement, per-node CPU/RSS, disk I/O and storage size, and P2P byte counters. This fixture uses fresh Simnet directories and explicit test-only parameters; production consensus is unchanged. [Exact scope](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/1c03bce8d4948b35bf9a9898d2990d2b6dac3753/docs/development/PHASE6_CLI_REAL_POW_RECOVERY_2026-11-23.md)

Keep #39 and #42 stable under test. When #39 is green, trigger #41's focused selected-child check before the broad matrix. When #42 is green, collect the CLI artifact, preserve its tested source, and trigger #43. Fetch #40's five-minute artifact when it exists. Public or private activation still requires authenticated pruning restore tied to settlement, representative mining/load evidence and approved network parameters. Preserve the inactive 29B KCADE schedule and the 21B/8B allocations.

[Dated checkpoint](docs/development/PHASE6_REAL_POW_AND_ACTIVE_CI_2026-11-23.md).

## Verified five-minute resource artifact

[Artifact 14801585581](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666/artifacts/14801585581) was decoded from #40's exact run. It contains 1,500 accepted blocks over 299.800755 seconds, 4.9999874083 BPS, and final tip `91a2469d8c1186daf74b069cb19afbd021b494edc5543258926e01211cd3a381`. The peer p95 validation times are 12.506 ms and 12.483 ms. The single shared three-daemon process used 26.59 CPU seconds over a 299.820118-second observation window (0.08869 average CPU cores). RSS was 317,360 KiB at the start and 472,384 KiB at the end. The process-lifetime peak, which includes startup, was 472,384 KiB. These are transparent loopback simulated-PoW measurements, not per-daemon or real-mining budgets. #40's full native suite remains in progress.

#43's staged workflow now checks the seven node/stage records and saves them as `metrics.jsonl` alongside the CLI logs. It remains gated on #42's exact result.

## Verified CLI run and current follow-ups

| Draft | Branch | Exact head | Current result |
| --- | --- | --- | --- |
| [#39 checkpoint persistence](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500) 7/8, focused checkpoint model passed. Native 137563153013 running the broader workspace build. |
| [#41 selected-child binding](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged behind the complete #39 matrix. |
| [#32 genesis conservation](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319) 7/8, focused passed; regular suite running. |
| [#34 red and deferred payouts](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390) 7/8, both focused cases passed; broader native gates running. |
| [#40 CPU/RSS](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666) 7/8, focused and artifact passed. Measured numbers remain pinned above. |
| [#42 CLI runbook](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320) 6/8. Dedicated three-process CLI test and artifact upload passed in job 137568406534. Broader native suite running. |
| [#43 native-PoW CLI](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `test/phase6-cli-real-pow-recovery-20261123` | `0b037914b7e9299302e09192f75236d8165ba5cc` | Staged behind full #42. Retains twelve real-native-PoW blocks, invalid nonce rejection and six-block recovery. Adds structured `recovery.json` capturing offline time, RPC-ready time, convergence time and exact tip alongside the seven per-node resource observations. |
| [#44 process-exit persistence](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `test/phase6-issuance-checkpoint-process-exit-20261123` | `e87401b6bca55a5684f513beb0fa893fed862d24` | Staged behind #39. Before-submit and after-success DB worker exit boundaries bypass clean RocksDB close. Parent reopens and verifies the complete readiness/checkpoint pair against its expected commitment. This does not test power loss or interruption during a write. |

## CLI evidence

[Artifact 14800768777](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320/artifacts/14800768777) SHA-256 `a88c5682b1d68a974538a441c15bdf788e566090badf0fb193b900c0db13a597` was downloaded and verified. It contains the driver and five separate process logs. The three loopback startup commands selected the same Simnet genesis and explicit two-follower topology. In the separate simulated-PoW recovery pass, C reopened the original data directory and caught up to `01d0a9180e6083448d40e9604f9a0c3f979715a6953f5cde6b79c12068072394`. Pruning point and virtual DAA matched, and all three processes exited successfully after SIGINT. This focused result does not validate native PoW or authorize a public or private network outside the fixture.

## Next executable gates

Finish #39's exact full matrix, then trigger #41 before the broad suite and #44's focused process-exit fixture. Finish #42's full matrix, then trigger #43 on its latest staged head and collect both CLI test cases, `metrics.jsonl` and `recovery.json`. Compare with the simulated-PoW five-minute baseline only within each test's scope. #32/#34/#40 still need their native conclusions. Authenticated pruning restore tied to actual consensus settlement, representative native/AuxPoW workloads, resource budgets and approved network parameters remain open. Preserve the approved 29B maximum with 21B primary and 8B tail and keep production issuance disabled.

[Dated evidence](docs/development/PHASE6_CLI_PASS_AND_PERSISTENCE_FOLLOWUPS_2026-11-23.md).

## Selected-path restore follow-up

| Draft | Branch | Exact source | Result |
| --- | --- | --- | --- |
| [#39](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/39) | `test/phase6-issuance-checkpoint-persistence-20261122` | `db7d221116af81a2b7cbba9f9646d760052a56f2` | [45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500), 7/8. Focused model passed; native 137563153013 is in the workspace build. |
| [#41](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/41) | `test/phase6-selected-child-issuance-binding-20261123` | `b997d82bc6e72384a5ba747485812698cce425dd` | Staged, base is the exact #39 head. |
| [#44](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/44) | `test/phase6-issuance-checkpoint-process-exit-20261123` | `e87401b6bca55a5684f513beb0fa893fed862d24` | Staged, base is the exact #39 head. |
| [#42](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/42) | `docs/phase6-loopback-operator-runbook-20261123` | `bba19b05e8659eb2b58bc0921c816070598be67e` | [45137198320](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320), 6/8. Dedicated CLI and [artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45137198320/artifacts/14800768777) passed; native 137568406534 is in the broader suite. |
| [#43](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/43) | `test/phase6-cli-real-pow-recovery-20261123` | `0b037914b7e9299302e09192f75236d8165ba5cc` | Staged, base is the exact #42 head. |
| [#32](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/32) | `test/phase6-genesis-issuance-conservation-20261122` | `f4d1d4f80c39b2e1c269f97473c53a8762b8bd5b` | [45135385319](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135385319), 7/8. Focused genesis pass, native 137560421199 regular suite running. |
| [#34](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/34) | `test/phase6-merge-payout-boundary-20261122` | `0c8cb06e2cc00dd2d1d4d3633a0e2604e9a53fdc` | [45135392390](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135392390), 7/8. Both focused payout cases passed, native 137560443988 regular suite running. |
| [#40](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/40) | `test/phase6-sustained-resource-sampling-20261122` | `45ce4935738cd30721544e6b37ae99a8cbf52804` | [45135241666](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666), 7/8. Focused and [resource artifact](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45135241666/artifacts/14801585581) passed, native 137559872891 authority suite running. |
| [#45](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/45) | `test/phase6-issuance-current-selected-path-20261123` | `b423c3cbe8be65e460f67b180a5032d08232c135` | Staged behind #41. Checks stale child rejection after selected-path replacement using the existing body and descendant verifiers. |

No failed job was reported in these active runs. The staged #45 test keeps two distinct child headers with identical checkpoint commitments. Switching the fixture's committed selected path rejects the old child under both the current tip and its old tip, despite the old body still authenticating in isolation. Only the selected sibling supplies the expected commitment for the issuance checkpoint reopen fixture. The test seeds ancestry and uses an illustrative commitment. Production still needs real authenticated pruning input and atomic settlement/readiness integration. [Fixture scope](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/b423c3cbe8be65e460f67b180a5032d08232c135/docs/development/PHASE6_ISSUANCE_SELECTED_PATH_2026-11-23.md)

Next: when #39 passes its full matrix, trigger #41 and #44 with their dedicated steps; trigger #43 when #42 is fully green. Then trigger #45 after #41 validates. Review regular native conclusions for #32, #34, and #40. Preserve the earlier 1,500-block simulated-PoW artifact independently of any later real mining outcomes. Representative native and AuxPoW load, measured resource budgets, authenticated pruning restore tied to live settlement, and owner-approved network parameters remain gates. Production issuance stays inactive under the 29B maximum, 21B primary and 8B tail policy.

[Dated checkpoint](docs/development/PHASE6_SELECTED_PATH_AND_CI_CONTINUATION_2026-11-23.md).
