#!/usr/bin/env python3
"""Exercise the bundled native miner against the supervised three-node loopback Simnet."""

import argparse
from contextlib import contextmanager
import json
from pathlib import Path
import re
import signal
import subprocess
import sys
import time


BLOCKS = 4
# Simnet-only fixture address with a zeroed public-key payload.
PAYOUT = "zkassim:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqrdtwestq"
ENDPOINTS = ("127.0.0.1:19110", "127.0.0.1:19210", "127.0.0.1:19310")
STATE_FIELDS = ("selected_tip", "block_count", "pruning_point", "virtual_daa_score")
LAUNCHER = Path(__file__).with_name("phase6-loopback-launch.py")


@contextmanager
def launch(args, phase):
    log_path = args.directory / f"launcher-{phase}.log"
    with log_path.open("w") as log:
        process = subprocess.Popen(
            [sys.executable, str(LAUNCHER), "--daemon", str(args.daemon), "--directory", str(args.directory)],
            stdout=log,
            stderr=subprocess.STDOUT,
        )
        try:
            yield process, log_path
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
                try:
                    process.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    process.terminate()
                    process.wait(timeout=20)
                    raise RuntimeError(f"launcher shutdown timed out; see {log_path}") from None
            if process.returncode != 0 or '"status": "stopped"' not in log_path.read_text():
                raise RuntimeError(f"launcher did not stop cleanly; see {log_path}")


def wait_for_state(args, launcher, phase, tip=None):
    process, log_path = launcher
    output = args.directory / f"health-{phase}.json"
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"launcher exited during {phase}; see {log_path}")
        if '"status": "listeners_ready"' in log_path.read_text():
            sample = subprocess.run(
                [str(args.health), "--minimum-peers", "1", "--timeout-ms", "2000", *ENDPOINTS],
                capture_output=True,
                text=True,
                timeout=5,
            )
            output.write_text(sample.stdout)
            if sample.returncode == 0:
                states = [node["state"] for node in json.loads(sample.stdout)["nodes"]]
                checkpoints = [tuple(state[key] for key in STATE_FIELDS) for state in states]
                if (
                    [state["connected_peers"] for state in states] == [2, 1, 1]
                    and len(set(checkpoints)) == 1
                    and (tip is None or checkpoints[0][0] == tip)
                ):
                    return checkpoints[0]
        time.sleep(0.25)
    raise TimeoutError(f"three nodes did not agree during {phase}; see {log_path} and {output}")


def mine(args):
    log_path = args.directory / "miner.log"
    with log_path.open("w") as log:
        subprocess.run(
            [
                str(args.miner),
                "--rpc-server", ENDPOINTS[0],
                "--mining-address", PAYOUT,
                "--threads", "1",
                "--max-blocks", str(BLOCKS),
            ],
            stdout=log,
            stderr=subprocess.STDOUT,
            timeout=30,
            check=True,
        )
    hashes = re.findall(r"BLOCK ACCEPTED\s+nonce=\d+\s+hash=([0-9a-f]{64})", log_path.read_text())
    if len(hashes) != BLOCKS or len(set(hashes)) != BLOCKS:
        raise RuntimeError(f"miner did not report {BLOCKS} distinct accepted native blocks; see {log_path}")
    return hashes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--daemon", type=Path, required=True)
    parser.add_argument("--miner", type=Path, required=True)
    parser.add_argument("--health", type=Path, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    args.directory.mkdir(parents=True)
    with launch(args, "fresh") as launcher:
        genesis = wait_for_state(args, launcher, "genesis")
        params = json.loads((args.directory / "native-pow.json").read_text())
        if params.get("skip_proof_of_work") is not False:
            raise RuntimeError("launcher did not enable native proof verification")
        hashes = mine(args)
        mined = wait_for_state(args, launcher, "mined", hashes[-1])
        if mined[0] == genesis[0] or mined[1] - genesis[1] != BLOCKS:
            raise RuntimeError("observed chain growth differs from the miner's accepted blocks")
    with launch(args, "reopen") as launcher:
        reopened = wait_for_state(args, launcher, "reopened", mined[0])
        if reopened != mined:
            raise RuntimeError("reopened consensus state differs from the mined checkpoint")
    evidence = {
        "accepted_hashes": hashes,
        "genesis": dict(zip(STATE_FIELDS, genesis)),
        "mined": dict(zip(STATE_FIELDS, mined)),
        "reopened": dict(zip(STATE_FIELDS, reopened)),
    }
    (args.directory / "miner-check.json").write_text(json.dumps(evidence) + "\n")
    print(json.dumps(evidence))


if __name__ == "__main__":
    main()
