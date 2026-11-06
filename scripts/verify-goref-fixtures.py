#!/usr/bin/env python3
"""Independent structural verifier for goref JSON DAG fixtures.

This tool intentionally does NOT depend on the Rust toolchain. It validates the
invariants that can be checked from the serialized JSON alone:

  * the params line parses and carries the required blockrate fields;
  * every block hash is unique;
  * every parent reference resolves to a block in the file;
  * blocks are written in topological order (parents precede children);
  * exactly one genesis block exists (empty parentsByLevel);
  * coinbase payload lengths are compatible with the *KASCADE* coinbase layout
    (blue_score 8 | subsidy 8 | shielded_commitment 32 | spk_version 2 | spk_len 1 | spk),
    i.e. len >= 51 + spk_len, where spk_len is read at byte offset 50;
  * the upstream layout is reported for comparison (spk_len at offset 18);
  * DAA `bits` values are present and non-zero for non-genesis blocks;
  * the observed blue-score span is consistent with the configured pruning depth.

Block-hash recomputation (kHeavyHash / blake2b header hashing) is deliberately
out of scope here: it requires `kaspa-hashes` and is performed by the Rust
integration tests and by `simpa` itself. See
docs/development/GOREF_FIXTURE_REGENERATION_2026-11-06.md for the full runbook.

Exit codes:
  0  all structural invariants hold and payloads are KASCADE-compatible
  1  one or more structural invariants failed
  2  usage / IO error
"""

from __future__ import annotations

import argparse
import gzip
import json
import sys
from collections import Counter

KASCADE_COMMITMENT_LEN = 32
KASCADE_FIXED_PREFIX = 8 + 8 + KASCADE_COMMITMENT_LEN + 2 + 1  # 51
UPSTREAM_FIXED_PREFIX = 8 + 8 + 2 + 1  # 19
REQUIRED_BLOCKRATE_FIELDS = (
    "target_time_per_block",
    "ghostdag_k",
    "past_median_time_sample_rate",
    "difficulty_sample_rate",
    "max_block_parents",
    "mergeset_size_limit",
    "merge_depth",
    "finality_depth",
    "pruning_depth",
    "coinbase_maturity",
    "shielded_anchor_depth",
    "max_shielded_anchor_age",
)


def die(msg: str, code: int = 2) -> "None":
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(code)


def load(path: str):
    try:
        with gzip.open(path, "rt", encoding="utf-8") as fh:
            lines = [ln for ln in fh.read().splitlines() if ln.strip()]
    except OSError as exc:
        die(f"cannot read {path}: {exc}")
    if len(lines) < 2:
        die(f"{path}: expected a params line plus at least one block line")
    try:
        params = json.loads(lines[0])
    except json.JSONDecodeError as exc:
        die(f"{path}: params line is not valid JSON: {exc}")
    blocks = []
    for i, ln in enumerate(lines[1:], start=2):
        try:
            blocks.append(json.loads(ln))
        except json.JSONDecodeError as exc:
            die(f"{path}: line {i} is not valid JSON: {exc}")
    return params, blocks


def coinbase_payload(block: dict) -> bytes:
    txs = block.get("transactions") or []
    if not txs:
        return b""
    payload_hex = txs[0].get("payload") or ""
    try:
        return bytes.fromhex(payload_hex)
    except ValueError:
        return b""


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("fixture", help="path to blocks.json.gz")
    ap.add_argument("--quiet", action="store_true", help="only print the final verdict")
    args = ap.parse_args()

    params, blocks = load(args.fixture)
    problems: list[str] = []
    notes: list[str] = []

    # --- params ---------------------------------------------------------
    blockrate = params.get("blockrate")
    if not isinstance(blockrate, dict):
        problems.append("params line has no 'blockrate' object")
        blockrate = {}
    missing = [f for f in REQUIRED_BLOCKRATE_FIELDS if f not in blockrate]
    if missing:
        problems.append(f"blockrate missing required fields: {', '.join(missing)}")

    # --- hashes & uniqueness -------------------------------------------
    hashes: list[str] = []
    for idx, blk in enumerate(blocks):
        hdr = blk.get("header") or {}
        h = hdr.get("hash")
        if not h:
            problems.append(f"block #{idx} has no header.hash")
            h = f"<missing-{idx}>"
        hashes.append(h)
    dupes = [h for h, n in Counter(hashes).items() if n > 1]
    if dupes:
        problems.append(f"{len(dupes)} duplicate block hash(es), e.g. {dupes[:3]}")

    index_of = {h: i for i, h in enumerate(hashes)}

    # --- genesis --------------------------------------------------------
    genesis = [i for i, b in enumerate(blocks) if not (b.get("header") or {}).get("parentsByLevel")]
    if len(genesis) != 1:
        problems.append(f"expected exactly one genesis block (empty parentsByLevel), found {len(genesis)}")

    # --- parents & topological order ------------------------------------
    missing_parents = 0
    order_violations = 0
    total_edges = 0
    for idx, blk in enumerate(blocks):
        levels = (blk.get("header") or {}).get("parentsByLevel") or []
        for level in levels:
            for parent in level or []:
                total_edges += 1
                if parent not in index_of:
                    missing_parents += 1
                    if missing_parents <= 3:
                        problems.append(f"block #{idx} references unknown parent {parent}")
                    continue
                if index_of[parent] >= idx:
                    order_violations += 1
                    if order_violations <= 3:
                        problems.append(f"block #{idx} appears before its parent {parent}")
    if missing_parents > 3:
        problems.append(f"... {missing_parents} unresolved parent references in total")
    if order_violations > 3:
        problems.append(f"... {order_violations} topological-order violations in total")

    # --- coinbase payload compatibility --------------------------------
    payload_lens = Counter()
    kascade_bad = 0
    upstream_bad = 0
    for idx, blk in enumerate(blocks):
        payload = coinbase_payload(blk)
        n = len(payload)
        payload_lens[n] += 1
        # KASCADE layout: spk_len lives at offset 50.
        if n > 50:
            spk_len = payload[50]
            if n < KASCADE_FIXED_PREFIX + spk_len:
                kascade_bad += 1
        else:
            kascade_bad += 1
        # Upstream layout: spk_len lives at offset 18.
        if n > 18:
            up_spk_len = payload[18]
            if n < UPSTREAM_FIXED_PREFIX + up_spk_len:
                upstream_bad += 1
        else:
            upstream_bad += 1
    if kascade_bad:
        problems.append(
            f"{kascade_bad}/{len(blocks)} coinbase payloads are NOT KASCADE-compatible "
            f"(need >= 51 + spk_len at offset 50); this is the known pre-shielded-vector blocker"
        )
    if upstream_bad:
        notes.append(f"{upstream_bad}/{len(blocks)} payloads also fail the upstream layout check (unexpected)")

    # --- DAA / blue score / pruning ------------------------------------
    bits = Counter()
    blue_scores = []
    zero_bits_non_genesis = 0
    for idx, blk in enumerate(blocks):
        hdr = blk.get("header") or {}
        b = hdr.get("bits")
        bits[b] += 1
        if isinstance(hdr.get("blueScore"), int):
            blue_scores.append(hdr["blueScore"])
        if idx not in genesis and not b:
            zero_bits_non_genesis += 1
    if zero_bits_non_genesis:
        problems.append(f"{zero_bits_non_genesis} non-genesis blocks have zero/missing DAA bits")

    span = (max(blue_scores) - min(blue_scores)) if blue_scores else 0
    pruning_depth = blockrate.get("pruning_depth")
    pruning_note = "n/a"
    if isinstance(pruning_depth, int) and pruning_depth > 0:
        # A fixture may legitimately span more than pruning_depth: it records every
        # block produced, while the node prunes old ones as virtual advances. This is
        # exactly what goref_custom_pruning_depth exercises, so it is informational.
        pruning_note = f"span {span} vs pruning_depth {pruning_depth} " + (
            "within" if span <= pruning_depth else "spans-more (expected for pruning fixtures)"
        )

    # --- report ---------------------------------------------------------
    if not args.quiet:
        print(f"fixture:            {args.fixture}")
        print(f"blocks:             {len(blocks)}")
        print(f"edges:              {total_edges}")
        print(f"genesis index:      {genesis[0] if len(genesis) == 1 else genesis}")
        print(f"blue-score range:   {min(blue_scores) if blue_scores else '-'}..{max(blue_scores) if blue_scores else '-'}")
        print(f"pruning:            {pruning_note}")
        print(f"distinct bits:      {len(bits)} -> {dict(list(bits.items())[:5])}")
        print(f"payload lengths:    {dict(payload_lens)}")
        print(f"kascade-incompat:   {kascade_bad}/{len(blocks)}")
        for n in notes:
            print(f"note:               {n}")

    if problems:
        print("\nVERDICT: FAIL")
        for p in problems:
            print(f"  - {p}")
        return 1

    print("\nVERDICT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
