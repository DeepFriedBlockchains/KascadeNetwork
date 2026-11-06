#!/bin/bash
# Regenerate the goref JSON DAG fixtures under KASCADE consensus rules.
#
# WHY: the three fixtures in testing/integration/testdata/dags_for_json_tests/ are
# upstream Kaspa KIP-0004 vectors. Their coinbase payloads predate KASCADE's
# unconditional 32-byte shielded commitment, so KASCADE's coinbase parser rejects
# them (see docs/development/GOREF_FIXTURE_REGENERATION_2026-11-06.md). Regenerating
# with `simpa` rebuilds every block through KASCADE's own consensus/coinbase path,
# producing valid payloads, hashes and parent references.
#
# REQUIREMENTS (not available in the 2026-11-06 sandbox; see the doc for evidence):
#   * Rust 1.91.0 (rust-version in Cargo.toml)
#   * a C/C++ toolchain: cc, g++, make, cmake, pkg-config, libclang (rocksdb/secp256k1)
#   * >= 12 GB free disk and >= 8 GB RAM for a release build of the workspace
#
# USAGE:
#   ./simpa/regenerate-goref-fixtures.sh            # regenerate + verify
#   ./simpa/regenerate-goref-fixtures.sh --verify   # verify only (no build)
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$DIR/.." && pwd)"
cd "$ROOT"

FIXTURES="testing/integration/testdata/dags_for_json_tests"
ORIGINALS="$FIXTURES/upstream-kaspa-originals"
VERIFY="scripts/verify-goref-fixtures.py"

verify_only=false
[[ "${1:-}" == "--verify" ]] && verify_only=true

if ! $verify_only; then
    command -v cargo >/dev/null || { echo "FATAL: cargo not found (need Rust 1.91.0)"; exit 1; }
    command -v cc    >/dev/null || { echo "FATAL: cc not found (need a C/C++ toolchain)"; exit 1; }
    command -v cmake >/dev/null || { echo "FATAL: cmake not found"; exit 1; }

    echo "==> Preserving current fixtures (if not already preserved)"
    for n in goref-1060-tx-265-blocks goref-notx-5000-blocks goref_custom_pruning_depth; do
        if [[ -f "$FIXTURES/$n/blocks.json.gz" && ! -f "$ORIGINALS/$n/blocks.json.gz" ]]; then
            mkdir -p "$ORIGINALS/$n"
            cp "$FIXTURES/$n/blocks.json.gz" "$ORIGINALS/$n/blocks.json.gz"
        fi
    done

    echo "==> Building simpa (release)"
    cargo build --bin simpa --release

    # --disable-launch-schedule zeroes low_difficulty_start_blocks/difficulty_ramp_blocks
    # so the vectors exercise the sampled DAA, matching the json_test configuration.
    # A DAG shorter than the 50k-block low-difficulty window would otherwise have a
    # constant difficulty and validate nothing.
    echo "==> Regenerating goref-notx-5000-blocks (0 txs/block)"
    cargo run --bin simpa --release -- -n 5000 -t 0 \
        --disable-launch-schedule \
        --blocks-json-gz-output-path "$FIXTURES/goref-notx-5000-blocks/blocks.json.gz"

    echo "==> Regenerating goref-1060-tx-265-blocks (4 txs/block)"
    cargo run --bin simpa --release -- -n 265 -t 4 \
        --disable-launch-schedule \
        --blocks-json-gz-output-path "$FIXTURES/goref-1060-tx-265-blocks/blocks.json.gz"

    echo "==> Regenerating goref_custom_pruning_depth (1 tx/block, pruning)"
    cargo run --bin simpa --release -- -n 5000 -t 1 \
        --disable-launch-schedule \
        --test-pruning --retention-period-days 100 \
        --blocks-json-gz-output-path "$FIXTURES/goref_custom_pruning_depth/blocks.json.gz"
fi

echo "==> Independent structural verification"
rc=0
for n in goref-1060-tx-265-blocks goref-notx-5000-blocks goref_custom_pruning_depth; do
    echo "---- $n"
    python3 "$VERIFY" "$FIXTURES/$n/blocks.json.gz" || rc=1
done

echo
echo "==> Next: run the Rust integration tests that consume these fixtures"
echo "    cargo test -p kaspa-testing-integration --release goref -- --nocapture"
echo "    (or the full suite: cargo test --workspace --release)"
echo
if [[ $rc -ne 0 ]]; then
    echo "VERIFICATION FAILED — inspect the report above before committing."
    exit $rc
fi
echo "All fixtures passed independent structural verification."
