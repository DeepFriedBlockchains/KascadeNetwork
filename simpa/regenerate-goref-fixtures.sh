#!/bin/bash
# Regenerate current-format DAGs using the real KASCADE template builder.
# The archived upstream gzip files are read-only inputs.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

fixtures="testing/integration/testdata/dags_for_json_tests"
input="$fixtures"

case "${1:-}" in
    --verify) ;;
    "")
        # Stage beside the destination so final renames stay on the same filesystem.
        input="$(mktemp -d "$root/$fixtures/.regeneration.XXXXXXXX")"
        trap 'rm -f "$input"/goref*/blocks.json.gz; rmdir "$input"/goref* "$input" 2>/dev/null || true' EXIT
        GOREF_FIXTURE_OUTPUT="$input" cargo test --locked -p kaspa-testing-integration --release \
            generate_goref_fixtures -- --ignored --nocapture
        ;;
    *) echo "usage: $0 [--verify]" >&2; exit 2 ;;
esac

python3 scripts/verify-goref-fixtures.py "$input/goref-1060-tx-265-blocks/blocks.json.gz" --expect 265 224 6
python3 scripts/verify-goref-fixtures.py "$input/goref-notx-5000-blocks/blocks.json.gz" \
    --expect 5000 0 125 --require-daa-change
python3 scripts/verify-goref-fixtures.py "$input/goref_custom_pruning_depth/blocks.json.gz" \
    --expect 5000 4790 125 --require-daa-change --require-pruning-span

if [[ "$input" != "$fixtures" ]]; then
    for name in goref-1060-tx-265-blocks goref-notx-5000-blocks goref_custom_pruning_depth; do
        mv "$input/$name/blocks.json.gz" "$fixtures/$name/blocks.json.gz"
    done
fi
