# CI Repair Handoff — 2026-11-06

Branch: `fix/public-ci-validation-20261106`
Base: `feature/public-development-20261106`
Head commit: `aa00ff1` (see §6 for follow-up commits)
Pull request: <https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/2>
Latest validation run: <https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42215101257>

This document records the public-repository CI repair work performed while ChatGPT
Work was unavailable. It is intended to let Work resume without re-deriving any of
the root causes below.

## 1. Scope and constraints honoured

- No repository sanitization or migration was restarted.
- The original private repository was not accessed or modified.
- No monetary-policy or consensus rule was changed to make a test pass.
- No test was disabled, weakened, or suppressed. Four tests that had silently
  never been registered were actually re-enabled (see §2.6).
- No force-push, no merge into `main`, no merge into the development branch.
- Work was done on a separate branch off `feature/public-development-20261106`
  and proposed via a separate PR (#2). PR #1 was left intact.

## 2. Exact failures and root causes

The failing run on PR #1 reported 6 failed, 8 cancelled, 2 successful checks.
Eight distinct causes were established.

### 2.1 `Check no_std` — `num_cpus` requires `std`

`crypto/hashes` unconditionally depended on `num_cpus`, which links `std`. The
`thumbv7em-none-eabi` check therefore failed with `E0463: can't find crate for
'std'`. Only the `std`-gated FishHash dataset builder uses `num_cpus`, so the
dependency was made optional and added to the `std` feature via `dep:num_cpus`.

### 2.2 WASM32 jobs — `getrandom` 0.4 has no default wasm backend

`rand` 0.10 pulls `getrandom` 0.4, whose `wasm32-unknown-unknown` backend is not
selected by default and emits a `compile_error!`. `shielded-core` now declares the
same `getrandom` version for `cfg(target_arch = "wasm32")` with the `wasm_js`
feature, which unifies the feature across the whole wasm build graph. This fixed
`Check WASM32`, `Test WASM32`, and `Build WASM32 SDK`.

### 2.3 `Lints` — rustfmt drift

`cargo fmt --all -- --check` reported 181 diffs against the pinned rustfmt
1.93.0. Resolved by running `cargo fmt --all` with that toolchain. No semantic
changes.

### 2.4 `Lints` — clippy `-D warnings` backlog

A long series of clippy findings was cleared, including `type_complexity` (named
type aliases at each site), `too_many_arguments` (`DifficultyLaunchSchedule`
struct), `large_enum_variant` on generated protowire (scoped `#[allow]` on the
generated module only — the wire API is unchanged), `implied_bounds_in_impls`
(`CryptoRng` already implies `Rng` in rand_core 0.10, so the bound was reduced to
`impl CryptoRng`), `needless_range_loop`, `manual_is_multiple_of`,
`needless_borrow`/`slice::from_ref`, `unnecessary_map_or`, `let_and_return`,
`collapsible_if`/`let_chains`, and several never-read fields and dead constants.

### 2.5 `Test Suite` — mislabelled legacy HRP fixtures

`check_legacy_firecash_hrp_still_decodes` panicked with `BadChecksum`. Two
fixtures were labelled `zkas:` but carried checksums computed for the `firecash`
HRP. This was confirmed with an independent Python bech32 implementation. Only
the fixture labels were corrected; the decoder is untouched.

### 2.6 `Test Suite` — four tests were never registered

`duplicate_macro_attributes` revealed that doc comments had been placed between
two `#[test]` attributes, so four test functions were never compiled as tests.
The stray attribute was removed and `#[test]` added to the function in each case:
`history_status_is_incomplete_after_a_backfill_until_verified`,
`serde_default_does_not_make_bincode_records_forward_compatible`,
`scan_block_store_roundtrip`, and
`history_replay_reproduces_frontier_and_detects_tampering`.

### 2.7 `Build Linux Release`

Already passing; no change was required. It remains green.

### 2.8 Cancelled jobs — benign

The 8 cancellations were same-SHA concurrency-group collisions between the push
run `42173914791` and the pull_request run `42173919830` on head `ba4ce30`. The
workflow uses `cancel-in-progress: true`, so the older run is cancelled when the
newer run starts on the same commit. No investigation beyond confirming the SHA
overlap was needed; a rerun is sufficient. Optionally the concurrency group could
be made to distinguish `push` from `pull_request`, but that is a workflow policy
change and was deliberately left out of this PR.

## 3. Commits

| SHA | Subject |
| --- | --- |
| `374e2dd` | `fix(build): restore no_std and wasm32 builds` |
| `85cd647` | `test(addresses): correct two mislabelled legacy HRP fixtures` |
| `4b493c0` | `chore(lint): clear rustfmt drift and the clippy -D warnings backlog` |

58 files changed, +1552 / −1520. Working tree clean.

## 4. Local validation performed

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --tests --benches --examples -- -D warnings` | clean, 0 warnings |
| no_std check (`thumbv7em-none-eabi`) | OK |
| wasm32 clippy | OK |
| `--tests` compile for all five wasm-pack crates | OK |
| `kaspa-addresses` | 5 passed |
| `kaspa-shielded-core` | 125 passed, 2 ignored |
| `kaspa-pow` | 15 passed |
| `kaspa-mining` | 63 passed |
| `kaspa-consensus` | 139 passed, 1 ignored |

## 5. Sandbox limitations encountered

The repair sandbox has an 8.8 GB root filesystem and a 2 GB `/dev/shm` tmpfs,
with a ~240 s hard command timeout. Disk exhaustion during `cargo test` was
worked around by relocating `target/debug/build` and 1258 `*.rmeta` files to
`/root/kascade-build-cache` via symlinks and pruning cargo registry caches. Long
commands were run with `nohup ... &` plus polling. This is why a full
`cargo nextest run --workspace` was not completed locally; GitHub's `Test Suite`
job is the authoritative full run.

## 6. Follow-up `Test Suite` failures and fixes

The first `Test Suite` run (42206485413, head `4b493c0`) failed at
`goref_custom_pruning_depth_test`. Two further causes were found and fixed on
the same branch; the remaining one is a genuine blocker.

### 6.1 Stale goref `blockrate` schema — fixed (`55a3436`)

`BlockrateParams` gained required `shielded_anchor_depth` and
`max_shielded_anchor_age` fields. The three goref fixtures still carried the old
`blockrate` object, so `OverrideParams` deserialization failed and `json_test`
fell back to parsing the params line as a block, panicking with
`missing field 'header'`. The fixtures now include the two fields with values
that satisfy `shielded_anchor_depth < max_shielded_anchor_age <
pruning_depth - finality_depth` for each fixture's depth.

### 6.2 DEVNET launch schedule vs upstream DAA vectors — fixed (`aa00ff1`)

With parsing fixed, the goref tests failed with a difficulty mismatch
(`505505215` computed vs `505527324` expected). `DEVNET_PARAMS` enables the
KASCADE launch schedule (`low_difficulty_start_blocks = 50_000`,
`difficulty_ramp_blocks = 200_000`), which pins difficulty to the genesis target
and then ramps a ceiling, intentionally overriding the pure DAA. The goref DAGs
are upstream KIP-0004 vectors whose bits were produced by the pure DAA. The
launch schedule is now disabled **only inside `json_test`'s synthetic-DAG
config**; no production parameter or consensus rule is changed, and the launch
schedule keeps its own dedicated tests.

### 6.3 GENUINE BLOCKER: coinbase payload format vs upstream goref data

After 6.2, the goref tests fail with:

```
bad coinbase payload: coinbase payload length is 53 bytes but it needs to be
at least 129 bytes long in order to accommodate the script public key
```

KASCADE's coinbase payload unconditionally includes a 32-byte shielded state
commitment (`LENGTH_OF_SHIELDED_COMMITMENT` in
`consensus/src/processes/coinbase.rs`). `MIN_PAYLOAD_LENGTH` is therefore 51
bytes, and the parser reads the script-public-key length from offset 50. The
upstream goref blocks carry 53-byte pre-shielded payloads, so the parser reads a
script-key byte as the length and rejects the block. There is no legacy or
fork-gated payload path (`serialize_coinbase_payload` / `deserialize_coinbase_payload`
always include the commitment; `shielded_coinbase: false` on simnet does not
change the layout).

This cannot be fixed without either (a) changing consensus rules to accept the
legacy payload (explicitly forbidden, and unsafe), or (b) regenerating the goref
DAG fixtures under KASCADE rules — which changes every block hash and parent
reference and requires running a KASCADE node to produce a valid DAG. Neither is
in scope for a CI-repair change, and the sandbox lost its Rust toolchain before
this could be attempted. **The three goref tests therefore remain red and
`Test Suite` cannot go green until the fixtures are regenerated or the tests are
retired by the owners.**

Affected tests: `goref_custom_pruning_depth_test`, `goref_notx_test`,
`goref_notx_concurrent_test` (and the other `goref_*` tests share the same
fixtures). The secondary `DB is expected to have no strong references` panic is
just teardown noise after the primary failure.

## 7. GitHub Actions status — run 42215101257 (head `aa00ff1`)

| Job | Status |
| --- | --- |
| Check | success |
| Check no_std | success |
| Check WASM32 | success |
| Test WASM32 | success |
| Build WASM32 SDK | success |
| Lints | success |
| Build Linux Release | success |
| Test Suite | failure — 1129 passed, 3 failed (goref coinbase blocker) |

## 8. Remaining work for ChatGPT Work

1. Resolve the coinbase-payload blocker (§6.3): regenerate the goref DAG
   fixtures under KASCADE rules, or retire the upstream goref tests with owner
   sign-off. Do not weaken the coinbase parser to accept legacy payloads.
2. Run a full `cargo nextest run --workspace` on a machine with adequate disk
   (the sandbox lost its Rust toolchain mid-task, so later validation is CI-only).
3. Run `wasm-pack test --node` end-to-end for the five wasm crates.
4. After PR #2 merges into `feature/public-development-20261106`, rerun PR #1.
5. Consider distinguishing `push` from `pull_request` in the concurrency group
   so same-SHA runs do not cancel each other.
6. Do not duplicate this repair work; it is already committed and pushed.

## 8. Reproduction commands

```bash
git fetch origin
git checkout fix/public-ci-validation-20261106
cargo fmt --all -- --check
cargo clippy --workspace --tests --benches --examples -- -D warnings
cargo check -p kaspa-hashes --no-default-features --target thumbv7em-none-eabi
cargo check -p kaspa-wasm --target wasm32-unknown-unknown
cargo test -p kaspa-addresses -p kaspa-shielded-core -p kaspa-pow -p kaspa-mining -p kaspa-consensus
```
