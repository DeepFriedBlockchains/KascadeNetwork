# CI Repair Handoff — 2026-11-06

Branch: `fix/public-ci-validation-20261106`
Base: `feature/public-development-20261106`
Head commit: `4b493c0d15d87e2efc2da87b9089cbc5ed133357`
Pull request: <https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/2>
Validation run: <https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42206485413>

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

## 6. GitHub Actions status — run 42206485413

| Job | Status |
| --- | --- |
| Check | success |
| Check no_std | success |
| Check WASM32 | success |
| Test WASM32 | success |
| Build WASM32 SDK | success |
| Lints | success |
| Build Linux Release | success |
| Test Suite | in progress at handoff time |

`Test Suite` is the only check not yet confirmed green. If it fails, pull the
logs with:

```bash
gh run view 42206485413 --repo DeepFriedBlockchains/KascadeNetwork --log-failed
```

## 7. Remaining work for ChatGPT Work

1. Confirm `Test Suite` on run `42206485413`; if it fails, diagnose from
   `--log-failed` and fix on the same branch (normal commit + push, no force).
2. Run a full `cargo nextest run --workspace` on a machine with adequate disk.
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
