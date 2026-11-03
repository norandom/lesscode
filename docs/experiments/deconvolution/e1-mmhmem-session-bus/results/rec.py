"""E1a receipt recorder: runs one command, retains raw output, appends to receipt.json.

Usage: py -I rec.py <label> <cwd> [--env K=V ...] -- <cmd> [args...]
"""
import datetime as dt
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
RECEIPT = os.path.join(HERE, "receipt.json")
RAW = os.path.join(HERE, "raw")


def now():
    return dt.datetime.now(dt.timezone.utc).astimezone().isoformat(timespec="seconds")


def load():
    if os.path.exists(RECEIPT):
        with open(RECEIPT, encoding="utf-8") as f:
            return json.load(f)
    return {"experiment": "E1a mmhmem session bus", "commands": []}


def main():
    label, cwd, rest = sys.argv[1], sys.argv[2], sys.argv[3:]
    env = dict(os.environ)
    extra = {}
    while rest and rest[0] == "--env":
        k, v = rest[1].split("=", 1)
        env[k] = v
        extra[k] = v
        rest = rest[2:]
    assert rest[0] == "--"
    cmd = rest[1:]
    os.makedirs(RAW, exist_ok=True)
    log = os.path.join(RAW, label + ".log")
    start, t0 = now(), time.monotonic()
    with open(log, "w", encoding="utf-8", errors="replace") as f:
        p = subprocess.run(cmd, cwd=cwd, env=env, stdout=f, stderr=subprocess.STDOUT)
    secs = round(time.monotonic() - t0, 2)
    entry = {
        "label": label, "cwd": cwd, "cmd": cmd, "env": extra,
        "start": start, "end": now(), "seconds": secs,
        "exit_code": p.returncode, "raw_log": os.path.relpath(log, HERE).replace("\\", "/"),
    }
    r = load()
    r["commands"].append(entry)
    with open(RECEIPT, "w", encoding="utf-8") as f:
        json.dump(r, f, indent=2)
    print(json.dumps(entry))
    sys.exit(p.returncode)


if __name__ == "__main__":
    main()
