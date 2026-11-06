# goref Fixture Regeneration Under KASCADE Consensus — Feasibility & Blocker Report

**Date:** 2026-11-06
**Branch:** `feature/goref-fixture-regeneration-20261106`
**Branch base:** `fix/public-ci-validation-20261106` (stacked on PR #2, so this PR's
diff contains only the goref work). Pristine originals were extracted from
`feature/public-development-20261106` (`ba4ce3072bd7785e1b3061647f2dacbba044d8e6`).
**Related:** PR #2 (`fix/public-ci-validation-20261106`, head `91da4d8`), PR #1 (untouched)

---

## 1. Objective

Regenerate the three upstream goref DAG fixtures under KASCADE consensus rules so
the `Test Suite` job stops failing on coinbase payload validation, **without**
changing production consensus/monetary rules, weakening tests, or fabricating
vectors.

Affected fixtures (consumed by `testing/integration/src/consensus_integration_tests.rs`):

| Fixture | Test(s) | Blocks |
|---|---|---|
| `goref-notx-5000-blocks` | `goref_notx_test`, `goref_notx_concurrent_test` | 5001 |
| `goref-1060-tx-265-blocks` | `goref_tx_small_test`, `goref_tx_small_concurrent_test` | 266 |
| `goref_custom_pruning_depth` | `goref_custom_pruning_depth_test` | 5001 |

---

## 2. Verified GitHub state (preservation)

Before any work on this branch, the completed CI repairs were confirmed present on
GitHub:

- PR #2 — **OPEN**, base `feature/public-development-20261106`, head
  `fix/public-ci-validation-20261106`, head SHA `91da4d8f728fe147529420dbaaa5e628765fd8f3`,
  `mergeable: MERGEABLE`.
- PR #1 — **OPEN** (base `main`), untouched.
- Latest `Tests` run `42215101257`: 7/8 jobs success; `Test Suite` failed with
  1129 passed / 3 failed / 24 skipped (fail-fast left 386 not run). The 3 failures
  are the goref coinbase-payload rejections documented in §3.

Nothing in this branch modifies or force-pushes those branches.

---

## 3. The blocker (recap)

KASCADE's coinbase payload layout is:

```
blue_score(8) | subsidy(8) | shielded_commitment(32) | spk_version(2) | spk_len(1) | spk | extra
```

`consensus/src/processes/coinbase.rs` parses the 32-byte commitment
**unconditionally** and enforces `MIN_PAYLOAD_LENGTH = 51`. The upstream goref
vectors use the pre-shielded layout:

```
blue_score(8) | subsidy(8) | spk_version(2) | spk_len(1) | spk | extra
```

Independent measurement of the current fixtures (see §5) shows:

- genesis payload = **32 bytes** (< 51 minimum);
- every non-genesis payload = **53 bytes**;
- KASCADE reads `spk_len` at byte offset 50, which in a 53-byte upstream payload
  lands inside script-public-key data, yielding a required length of **129 bytes**
  — hence `coinbase payload length is 53 bytes but it needs to be at least 129 bytes long`.

This is a **data-format** incompatibility, not a logic bug. It cannot be fixed by
tweaking the parser without weakening consensus validation (forbidden).

---

## 4. Feasibility determination: the tooling exists and is correct

The repository ships a first-party generator: `simpa` (`simpa/src/main.rs`,
`simpa/src/blocks_json.rs`), driven by `simpa/generate-json-tests-data.sh`.

Key findings:

1. `simpa` builds blocks through **KASCADE's own consensus path**
   (`build_block_template` in `simpa/src/simulator/miner.rs`), so generated blocks
   carry the correct 32-byte shielded commitment and will pass the parser.
2. `simpa` starts from `DEVNET_PARAMS`, sets `crescendo_activation = always`,
   `skip_proof_of_work()`, and writes `OverrideParams` as the first JSON line
   followed by topologically ordered blocks — exactly the fixture format.
3. Therefore **valid replacement vectors can be generated without modifying
   production consensus rules.** Only test tooling is involved.

### 4.1 Launch-schedule coupling (important)

`DEVNET_PARAMS` sets `low_difficulty_start_blocks = 50_000` and
`difficulty_ramp_blocks = 200_000`. A 5 000-block DAG never leaves the
low-difficulty window, so every block's difficulty would be pinned to the genesis
target — the vectors would no longer exercise the sampled DAA at all.

The `json_test` harness already disables the schedule (commit `aa00ff1`) so the
upstream vectors validate the DAA. To keep generator and consumer consistent, this
branch adds an **opt-in** `--disable-launch-schedule` flag to `simpa` that zeroes
`low_difficulty_start_blocks`/`difficulty_ramp_blocks` (`difficulty_ramp_blocks == 0`
is the documented "schedule disabled" sentinel). Default `simpa` behaviour is
unchanged; production `Params` are untouched.

> ⚠️ This flag is a two-line change in `simpa/src/main.rs`. It **must be compiled
> and exercised in an environment with a working toolchain** before the fixtures it
> produces are trusted. It is not a consensus change.

---

## 5. Independent verification tooling (works without Rust)

`scripts/verify-goref-fixtures.py` validates, from the serialized JSON alone:

- params line parses and carries every required `blockrate` field
  (including `shielded_anchor_depth`, `max_shielded_anchor_age`);
- all block hashes unique;
- every parent reference resolves;
- blocks are in topological order (parents precede children);
- exactly one genesis block;
- coinbase payload length vs. the KASCADE layout (`>= 51 + spk_len` at offset 50),
  and the upstream layout for comparison;
- DAA `bits` present and non-zero for non-genesis blocks;
- blue-score span vs. `pruning_depth` (informational for pruning fixtures).

### 5.1 Current results (evidence)

```
goref-1060-tx-265-blocks : 266 blocks, 1753 edges, genesis ok, 1 distinct bits
                           payloads {32:1, 53:265}, kascade-incompat 266/266 -> FAIL
goref-notx-5000-blocks   : 5001 blocks, 57592 edges, genesis ok, 2096 distinct bits
                           payloads {32:1, 53:5000}, kascade-incompat 5001/5001 -> FAIL
goref_custom_pruning_depth: 5001 blocks, 56422 edges, genesis ok, 2270 distinct bits
                           payloads {32:1, 53:5000}, kascade-incompat 5001/5001 -> FAIL
```

Structural invariants (parents, ordering, genesis, DAA bits) **pass**; the only
failure is coinbase-payload compatibility — confirming the diagnosis is precise and
that regeneration (not parser surgery) is the correct fix.

The pristine upstream fixtures are preserved under
`testing/integration/testdata/dags_for_json_tests/upstream-kaspa-originals/`
(byte-identical to base `ba4ce30`). They additionally lack the two `blockrate`
fields, which commit `55a3436` added to the live fixtures.

---

## 6. Environment blocker: regeneration cannot be executed here

Regeneration requires building `simpa`, which requires a full Rust + C/C++
toolchain and several GB of build space. The 2026-11-06 sandbox has none of this.
The following was measured, not assumed.

### 6.1 No Rust toolchain

```
$ command -v cargo rustc rustup
(no output — all absent)
```

Installing `1.91.0` (minimal profile) succeeds but consumes ~1.2 GB:

```
$ du -sh /root/.rustup /root/.cargo/registry
571M    /root/.rustup
655M    /root/.cargo/registry
```

### 6.2 No C/C++ toolchain

```
$ for t in cc gcc g++ clang cmake make pkg-config; do command -v $t; done
(all MISSING)
```

`rocksdb-sys` and `secp256k1-sys` require these. Installing
`build-essential cmake libclang-dev clang` adds ~700 MB.

The very first build attempt fails immediately:

```
$ cargo build --bin simpa --release
error: linker `cc` not found
  = note: No such file or directory (os error 2)
error: could not compile `proc-macro2` (build script) due to 1 previous error
```

### 6.3 Insufficient disk (the hard blocker)

Sandbox capacity: **8.8 GB root, 5.8 GB baseline used, 2.6 GB free, 3.8 GB RAM,
2 CPUs, no swap.** After toolchain + C toolchain installation only ~668 MB remain.

With the C toolchain installed, the build ran until the filesystem filled:

```
Compiling ... (181 crates compiled)
error: could not compile `js-sys` (lib)
  ... (signal: 9, SIGKILL: kill)     # OOM/disk pressure
/dev/root  8.8G  8.5G  0  100% /
target/ 444M and still growing
```

A release build of this 94-crate workspace (rocksdb, secp256k1, full consensus
graph) needs roughly **5–10 GB** of target space. The 5.8 GB baseline is system
tooling (`/usr` 3.2 GB, `/root` 1.5 GB, `/opt` 947 MB); the repository itself is
only 33 MB, so there is nothing safe to reclaim that would close the gap.

**Conclusion:** regeneration is feasible *in principle* and the exact commands are
known, but **cannot be executed in this sandbox**. Per the assignment, no vectors
were fabricated or substituted.

---

## 7. What this branch delivers

1. `scripts/verify-goref-fixtures.py` — independent, Rust-free structural verifier
   (runs now; §5.1).
2. `simpa/regenerate-goref-fixtures.sh` — hardened runbook: toolchain checks,
   original preservation, regeneration with `--disable-launch-schedule`, and
   automatic verification. Supports `--verify` for verification-only.
3. `simpa/src/main.rs` — opt-in `--disable-launch-schedule` flag (test tooling).
4. `testing/integration/testdata/dags_for_json_tests/upstream-kaspa-originals/` —
   pristine upstream vectors preserved for comparison.
5. This report.

No production consensus/monetary code, no test assertions, and no CI workflow were
weakened or disabled.

---

## 8. Reproducible commands (run in a capable environment)

```bash
# 0. Prerequisites: Rust 1.91.0; cc/g++/make/cmake/pkg-config/libclang; >=12 GB disk
rustc --version            # expect 1.91.0
cc --version && cmake --version

# 1. Regenerate + verify
./simpa/regenerate-goref-fixtures.sh

# 2. Run the consuming integration tests
cargo test -p kaspa-testing-integration --release goref -- --nocapture

# 3. Full suite
cargo test --workspace --release

# 4. Verification only (no build)
./simpa/regenerate-goref-fixtures.sh --verify
```

### 8.1 Post-regeneration checklist

Because regeneration changes every block hash and parent reference, verify:

- [ ] `scripts/verify-goref-fixtures.py` reports `VERDICT: PASS` for all three;
- [ ] block hashes are recomputed and match headers (the Rust tests do this via
      `Block::from_arcs` + `validate_and_insert_block`);
- [ ] parent references resolve and the DAG inserts without `MissingParents`;
- [ ] DAA expectations hold with the launch schedule disabled (matches `aa00ff1`);
- [ ] `goref_custom_pruning_depth_test` still exercises pruning
      (`--test-pruning --retention-period-days 100`, `pruning_depth = 450`);
- [ ] no test hardcodes a fixture hash (confirmed: none do);
- [ ] `cargo test --workspace --release` green.

---

## 9. Verified CI results (PR #3, run 42221177157, head `277790e`)

`Tests` workflow — **7 of 8 jobs success**, `Test Suite` failure (expected):

| Job | Result |
|---|---|
| Check | success |
| Check no_std | success |
| Check WASM32 | success |
| Test WASM32 | success |
| Build WASM32 SDK | success |
| Lints (`-D warnings`) | success |
| Build Linux Release | success |
| Test Suite | failure — same 3 goref tests, same coinbase error |

Importantly, `Lints` and `Build Linux Release` both compile `simpa`, so the new
`--disable-launch-schedule` flag is valid and warning-clean. `Test Suite` fails on
exactly `goref_notx_test`, `goref_notx_concurrent_test`, and
`goref_custom_pruning_depth_test` with the unchanged coinbase-payload error — i.e.
**no new regressions**. The fixtures are intentionally unchanged in this PR; the
failure disappears once §8 is run on a build-capable machine.

---

## 10. Remaining work / handoff

- **Blocked here:** actual regeneration (needs a build-capable runner; see §6).
- **Next owner:** run §8 on a machine with ≥12 GB disk and the full toolchain,
  commit the regenerated `.json.gz` files, and confirm the `Test Suite` job passes.
- **If a build-capable runner is unavailable long-term:** the alternative is to
  retire the three goref tests with owner sign-off (Option B), explicitly *not*
  done here.
- The `simpa` flag change must be compiled and validated before the vectors it
  produces are trusted.
