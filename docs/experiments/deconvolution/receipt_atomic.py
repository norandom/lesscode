"""Append-only, atomic receipt writer (same JSON shape as the E1b/E1c/E3 receipts).

A receipt is one JSON file {"commands": [...], "facts": [...], "notes": [...]}. Writers take an
exclusive lock file (O_CREAT|O_EXCL, retry with backoff), re-read the current JSON, append one
entry, write a temp file in the same directory and os.replace it onto the receipt, then release
the lock. Concurrent steps therefore cannot lose entries. Existing entries are never removed
or reordered. A lock older than STALE_S (crashed holder) is broken.

  receipt_atomic.py RECEIPT run LABEL [--cwd D] [--env K=V] [--timeout S] [--no-probe] -- CMD...
  receipt_atomic.py RECEIPT note KEY JSON
  receipt_atomic.py RECEIPT facts [JSON]
Logs for `run` go to <receipt dir>/logs/LABEL.log.
"""
from __future__ import annotations

import datetime
import json
import os
import subprocess
import sys
import time
from pathlib import Path

STALE_S = 120.0
EMPTY = {"commands": [], "facts": [], "notes": []}


def now() -> str:
    return datetime.datetime.now(datetime.UTC).isoformat()


def _acquire(lock: Path, timeout_s: float = 300.0) -> None:
    t0, delay = time.monotonic(), 0.005
    while True:
        try:
            os.close(os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY))
            return
        except FileExistsError:
            try:
                if time.time() - lock.stat().st_mtime > STALE_S:
                    lock.unlink()
                    continue
            except OSError:
                continue
            if time.monotonic() - t0 > timeout_s:
                raise TimeoutError(f"receipt lock busy: {lock}")
            time.sleep(delay)
            delay = min(delay * 1.5, 0.1)


def _append(path: Path | str, collection: str, entry: dict) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    lock = path.with_name(path.name + ".lock")
    _acquire(lock)
    try:
        data = json.loads(path.read_text(encoding="utf-8")) if path.exists() else {k: [] for k in EMPTY}
        for k in EMPTY:
            data.setdefault(k, [])
        data[collection].append(entry)
        tmp = path.with_name(f"{path.name}.{os.getpid()}.tmp")
        tmp.write_text(json.dumps(data, indent=1), encoding="utf-8")
        for attempt in range(50):  # Windows: replace can briefly fail while a reader holds the file
            try:
                os.replace(tmp, path)
                break
            except PermissionError:
                if attempt == 49:
                    raise
                time.sleep(0.02)
    finally:
        try:
            lock.unlink()
        except OSError:
            pass


def add_command(path: Path | str, entry: dict) -> None:
    _append(path, "commands", entry)


def add_note(path: Path | str, key: str, value) -> None:
    _append(path, "notes", {"time_utc": now(), "key": key, "value": value})


def add_fact(path: Path | str, fact: dict) -> None:
    _append(path, "facts", {"time_utc": now(), **fact})


def run(path: Path | str, label: str, cmd: list[str], cwd: str | None = None, env: dict | None = None,
        timeout: float | None = None, probe: bool = True, logs: Path | None = None) -> int | str:
    """Run cmd with output to logs/LABEL.log and append one command entry."""
    path = Path(path)
    logs = Path(logs) if logs else path.parent / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    adaptive = None
    if probe:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import adaptive as _a
        adaptive = _a.probe()
    start, t0 = now(), time.perf_counter()
    with open(logs / f"{label}.log", "w", encoding="utf-8") as log:
        try:
            code: int | str = subprocess.run(cmd, cwd=cwd, env=dict(os.environ, **(env or {})), stdout=log,
                                             stderr=subprocess.STDOUT, timeout=timeout).returncode
        except subprocess.TimeoutExpired:
            code = "timeout"
    seconds = round(time.perf_counter() - t0, 2)
    add_command(path, {"label": label, "cmd": cmd, "cwd": cwd, "env": env or {}, "start_utc": start,
                       "end_utc": now(), "seconds": seconds, "exit": code, "adaptive": adaptive})
    print(f"{label}: exit {code} in {seconds:.1f}s")
    return code


def main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if argv else 2
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    path, a = Path(argv[0]), argv[1:]
    if a[0] == "note":
        add_note(path, a[1], json.loads(a[2]))
    elif a[0] == "facts":
        add_fact(path, json.loads(a[1]) if len(a) > 1 else {})
    elif a[0] == "run":
        label, rest = a[1], a[2:]
        opts, cmd = rest[:rest.index("--")], rest[rest.index("--") + 1:]
        o: dict = {"env": {}}
        i = 0
        while i < len(opts):
            if opts[i] == "--no-probe":
                o["no_probe"] = True
                i += 1
                continue
            k, v = opts[i], opts[i + 1]
            if k == "--env":
                o["env"].update([v.split("=", 1)])
            else:
                o[k.lstrip("-")] = v
            i += 2
        code = run(path, label, cmd, cwd=o.get("cwd"), env=o["env"],
                   timeout=float(o["timeout"]) if "timeout" in o else None, probe=not o.get("no_probe"))
        return 0 if code == 0 else 1
    else:
        print(f"unknown command {a[0]}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
