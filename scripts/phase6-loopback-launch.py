#!/usr/bin/env python3
"""Launch and supervise three isolated native-PoW Simnet daemons."""

import argparse
import fcntl
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import threading
import time


NODES = {"a": (19111, 19110), "b": (19211, 19210), "c": (19311, 19310)}
PARAMS = '{"skip_proof_of_work":false,"shielded_coinbase":false}\n'


def pin_file(path, content):
    try:
        with path.open("x", encoding="utf-8") as stream:
            stream.write(content)
    except FileExistsError:
        if path.read_text(encoding="utf-8") != content:
            raise ValueError(f"{path} differs from the launcher configuration")


def listen_addresses():
    return (("127.0.0.1", port) for ports in NODES.values() for port in ports)


def ports_available():
    for address in listen_addresses():
        try:
            with socket.socket() as listener:
                listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                listener.bind(address)
        except OSError as error:
            raise OSError(f"loopback port {address[1]} is unavailable: {error}") from error


def wait_for_listeners(processes, stop, timeout):
    pending = set(listen_addresses())
    limit = time.monotonic() + timeout
    while pending and not stop.is_set():
        check_processes(processes)
        for address in tuple(pending):
            try:
                with socket.create_connection(address, timeout=0.15):
                    pending.remove(address)
            except OSError:
                pass
        if pending and time.monotonic() >= limit:
            raise TimeoutError(f"listeners did not start on ports {sorted(port for _, port in pending)}")
        stop.wait(0.1)
    return not stop.is_set()


def check_processes(processes):
    for name, process in processes.items():
        if process.poll() is not None:
            raise RuntimeError(f"node {name} exited with status {process.returncode}; inspect node-{name}.log")


def stop_processes(processes):
    for process in processes.values():
        process.send_signal(signal.SIGINT)
    deadline = time.monotonic() + 15
    for process in processes.values():
        try:
            process.wait(timeout=max(0, deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
    return all(process.returncode == 0 for process in processes.values())


def launch(args, stop):
    root = args.directory.resolve()
    root.mkdir(parents=True, exist_ok=True)
    with (root / ".launcher.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        config = root / "launcher.toml"
        params = root / "native-pow.json"
        pin_file(config, "# Intentionally empty. The launcher owns the Simnet arguments.\n")
        pin_file(params, PARAMS)
        ports_available()
        daemon = args.daemon.resolve(strict=True)
        env = {key: value for key, value in os.environ.items() if not key.startswith("KASPAD_")}
        processes = {}
        try:
            for name, (p2p, rpc) in NODES.items():
                data = root / f"node-{name}"
                if data.is_symlink():
                    raise ValueError(f"expected a separate directory at {data}")
                invocation = [
                    str(daemon),
                    "--simnet",
                    f"--configfile={config}",
                    f"--appdir={data}",
                    f"--override-params-file={params}",
                    f"--listen=127.0.0.1:{p2p}",
                    f"--rpclisten=127.0.0.1:{rpc}",
                    "--nodnsseed",
                    "--disable-upnp",
                ]
                invocation += ["--outpeers=0", "--enable-unsynced-mining"] if name == "a" else ["--connect=127.0.0.1:19111"]
                with (root / f"node-{name}.log").open("ab") as log:
                    processes[name] = subprocess.Popen(invocation, stdout=log, stderr=subprocess.STDOUT, env=env, start_new_session=True)
            if wait_for_listeners(processes, stop, args.timeout):
                check_processes(processes)
                print(json.dumps({"status": "listeners_ready", "rpc_ports": {name: rpc for name, (_, rpc) in NODES.items()}}), flush=True)
                if not args.check:
                    while not stop.wait(0.5):
                        check_processes(processes)
        finally:
            stopped = stop_processes(processes)
        if not stopped:
            raise RuntimeError(f"one or more daemons did not exit cleanly; inspect {root}")
        print(json.dumps({"status": "stopped"}), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--daemon", type=Path, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30, help="listener startup deadline in seconds")
    parser.add_argument("--check", action="store_true", help="stop after startup and verify clean shutdown")
    args = parser.parse_args()
    if not 0 < args.timeout <= 300:
        parser.error("--timeout must be greater than 0 and at most 300 seconds")
    stop = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())
    try:
        launch(args, stop)
    except (OSError, ValueError, RuntimeError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
