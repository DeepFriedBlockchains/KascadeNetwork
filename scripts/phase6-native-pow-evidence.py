#!/usr/bin/env python3
"""Record or verify source and file digests for a native-PoW Simnet artifact."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


MANIFEST = "provenance.json"
SCENARIO_FILES = {
    "recovery": ("recovery.json", "a-real-pow.log", "b-real-pow.log", "c-real-pow.log", "c-real-pow-reopen.log"),
    "reconnect": ("peer-recovery.json", "a-reconnect.log", "b-reconnect.log", "c-reconnect.log", "c-reconnect-reopen.log"),
    "sustained": ("sustained.json", "blocks.jsonl", "a-sustained.log", "b-sustained.log", "c-sustained.log", "c-sustained-reopen.log"),
}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def artifact_files(directory, scenario):
    params_file = directory / "native-pow-params.json"
    params = json.loads(params_file.read_text(encoding="utf-8"))
    expected_keys = {"skip_proof_of_work", "shielded_coinbase"}
    if not isinstance(params, dict) or params.keys() != expected_keys or any(value is not False for value in params.values()):
        raise ValueError("unexpected native-PoW Simnet overrides")
    filenames = (params_file.name, "driver.log", "metrics.jsonl", *SCENARIO_FILES[scenario])
    return {filename: sha256(directory / filename) for filename in filenames}


def checkout_sha():
    if subprocess.run(["git", "diff", "--quiet", "HEAD", "--"], check=False).returncode:
        raise ValueError("cannot record source provenance with modified tracked files")
    return subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()


def record(args):
    manifest = {
        "version": 1,
        "network": "simnet",
        "scenario": args.scenario,
        "checkout_sha": checkout_sha(),
        "daemon_sha256": sha256(args.binary),
        "files": artifact_files(args.directory, args.scenario),
    }
    if os.getenv("GITHUB_ACTIONS") == "true":
        manifest["github_actions"] = {
            "repository": os.environ["GITHUB_REPOSITORY"],
            "run_id": os.environ["GITHUB_RUN_ID"],
            "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
            "job": os.environ["GITHUB_JOB"],
            "event_sha": os.environ["GITHUB_SHA"],
        }
    with (args.directory / MANIFEST).open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


def verify(args):
    manifest = json.loads((args.directory / MANIFEST).read_text(encoding="utf-8"))
    if manifest["version"] != 1 or manifest["network"] != "simnet":
        raise ValueError("unsupported native-PoW evidence manifest")
    if not re.fullmatch("[a-f0-9]{40}", args.checkout_sha) or manifest["checkout_sha"] != args.checkout_sha:
        raise ValueError("checkout SHA differs from the expected source")
    if not re.fullmatch("[a-f0-9]{64}", manifest["daemon_sha256"]):
        raise ValueError("invalid daemon SHA-256")
    if args.binary is not None and sha256(args.binary) != manifest["daemon_sha256"]:
        raise ValueError("daemon SHA-256 mismatch")
    files = artifact_files(args.directory, manifest["scenario"])
    recorded_files = manifest["files"]
    if not isinstance(recorded_files, dict):
        raise ValueError("invalid artifact digest list")
    if recorded_files != files:
        mismatches = sorted(
            filename for filename in files.keys() | recorded_files.keys()
            if files.get(filename) != recorded_files.get(filename)
        )
        raise ValueError("artifact SHA-256 mismatch for " + ", ".join(mismatches))
    print(f"Verified {len(files)} {manifest['scenario']} evidence files at {args.checkout_sha}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    writer = commands.add_parser("record", help="record evidence after the focused test passes")
    writer.add_argument("directory", type=Path)
    writer.add_argument("--scenario", choices=SCENARIO_FILES, required=True)
    writer.add_argument("--binary", type=Path, required=True)
    reader = commands.add_parser("verify", help="verify a previously recorded artifact")
    reader.add_argument("directory", type=Path)
    reader.add_argument("--checkout-sha", required=True, help="exact checkout SHA obtained from the run")
    reader.add_argument("--binary", type=Path, help="optional reference daemon binary")
    args = parser.parse_args()
    try:
        if args.command == "record":
            record(args)
        else:
            verify(args)
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
