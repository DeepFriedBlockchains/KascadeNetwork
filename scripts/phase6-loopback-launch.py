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
import time


LOOPBACK = "127.0.0.1"
SEED_PORT = 19111
RPC_PORTS = {"a": 19110, "b": 19210, "c": 19310}
PARAMS = '{"skip_proof_of_work":false,"shielded_coinbase":false}\n'


def pin_file(path, content):
    try:
        with path.open("x", encoding="utf-8") as stream:
            stream.write(content)
    except FileExistsError:
        if path.read_text(encoding="utf-8") != content:
            raise ValueError(f"{path} differs from the launcher configuration")


def ports_available():
    for port in (SEED_PORT, *RPC_PORTS.values()):
        try:
            with socket.socket() as listener:
                listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                listener.bind((LOOPBACK, port))
        except OSError as error:
            raise OSError(f"loopback port {port} is unavailable: {error}") from error


def wait_for_listeners(processes, ports, should_stop, deadline):
    pending = set(ports)
    while pending and not should_stop():
        check_processes(processes)
        for port in tuple(pending):
            try:
                with socket.create_connection((LOOPBACK, port), timeout=0.15):
                    pending.remove(port)
            except OSError:
                pass
        if pending:
            if time.monotonic() >= deadline:
                raise TimeoutError(f"listeners did not start on ports {sorted(pending)}")
            time.sleep(0.1)
    return not should_stop()


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


def launch(args, should_stop):
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
        deadline = time.monotonic() + args.timeout
        try:
            for name, rpc in RPC_PORTS.items():
                if should_stop():
                    break
                data = root / f"node-{name}"
                if data.is_symlink():
                    raise ValueError(f"expected a separate directory at {data}")
                invocation = [
                    str(daemon),
                    "--simnet",
                    f"--configfile={config}",
                    f"--appdir={data}",
                    f"--override-params-file={params}",
                    f"--rpclisten={LOOPBACK}:{rpc}",
                    "--nodnsseed",
                    "--disable-upnp",
                ]
                if name == "a":
                    invocation += [f"--listen={LOOPBACK}:{SEED_PORT}", "--outpeers=0", "--enable-unsynced-mining"]
                else:
                    invocation.append(f"--connect={LOOPBACK}:{SEED_PORT}")
                with (root / f"node-{name}.log").open("ab") as log:
                    processes[name] = subprocess.Popen(invocation, stdout=log, stderr=subprocess.STDOUT, env=env, start_new_session=True)
                # The permanent peers back off after a failed first connection.
                if name == "a" and not wait_for_listeners(processes, (SEED_PORT,), should_stop, deadline):
                    break
            if wait_for_listeners(processes, RPC_PORTS.values(), should_stop, deadline):
                check_processes(processes)
                print(json.dumps({"status": "listeners_ready", "rpc_ports": RPC_PORTS}), flush=True)
                if not args.check:
                    while not should_stop():
                        time.sleep(0.5)
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
    stop_requested = False

    def request_stop(*_):
        nonlocal stop_requested
        stop_requested = True

    signal.signal(signal.SIGINT, request_stop)
    signal.signal(signal.SIGTERM, request_stop)
    try:
        launch(args, lambda: stop_requested)
    except (OSError, ValueError, RuntimeError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
