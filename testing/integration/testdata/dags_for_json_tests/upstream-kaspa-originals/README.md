# Pristine upstream goref fixtures (preserved for comparison)

These are the **original upstream Kaspa KIP-0004 DAG vectors**, extracted
byte-for-byte from base branch `feature/public-development-20261106`
(commit `ba4ce3072bd7785e1b3061647f2dacbba044d8e6`, snapshot `6224816`).

They are kept here so the live fixtures in the parent directories can be compared
against the pre-KASCADE originals during and after regeneration.

## Why they are not usable as-is

Their coinbase payloads use the **pre-shielded** layout:

```
blue_score(8) | subsidy(8) | spk_version(2) | spk_len(1) | spk | extra
```

KASCADE parses an unconditional 32-byte `shielded_commitment` and enforces a
51-byte minimum, so these payloads (32 bytes genesis, 53 bytes otherwise) are
rejected. They also predate the `shielded_anchor_depth` / `max_shielded_anchor_age`
fields added to `BlockrateParams`, so their params line fails `OverrideParams`
deserialization (`deny_unknown_fields`).

See `docs/development/GOREF_FIXTURE_REGENERATION_2026-11-06.md` for the full
analysis and the regeneration runbook (`simpa/regenerate-goref-fixtures.sh`).

## Verification

```bash
python3 scripts/verify-goref-fixtures.py \
  testing/integration/testdata/dags_for_json_tests/upstream-kaspa-originals/goref-notx-5000-blocks/blocks.json.gz
```

Expected: structural invariants pass; coinbase-payload check fails (by design),
and the params line is missing the two shielded blockrate fields.
