"""Adaptive budgets for a shared machine (owner request, 2026-11-04).

Fixed wall-clock caps turn host load into false timeouts. This helper senses the
host and scales every deadline by a measured factor, so a budget means "this
much work on an idle machine", not "this many seconds whatever else runs".

Three measurements, stdlib only, native Windows:
  cpu_other  fraction of total CPU used by processes other than ours (PowerShell
             Get-Counter; falls back to None if unavailable)
  ram_free   free physical memory in GB
  bench_s    seconds for a fixed CPU-bound task; compared with a stored idle
             reference, this is the most direct load signal

factor = max(bench_s / ref_bench_s, 1 / (1 - cpu_other)), clamped to [1.0, MAX_FACTOR].
scaled(base) = base * factor.   wait_for_idle() blocks (bounded) until the host is quiet.

Policy for consumers: record probe() in the receipt before each heavy step; use
scaled(base) as the deadline; a timeout observed while factor > 1.5 is "rerun once
when idle", never a counted outcome on the first occurrence.

CLI:  python adaptive.py probe | calibrate [path] | factor [path] | wait [max_s] [cpu_threshold]
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

MAX_FACTOR = 4.0
REF_PATH = Path(__file__).with_name("adaptive-reference.json")


def bench() -> float:
    """Fixed CPU-bound task, about one second on an idle core."""
    t0 = time.perf_counter()
    h = b"lesscode"
    for _ in range(400_000):
        h = hashlib.sha256(h).digest()
    return time.perf_counter() - t0


def _counter(path: str) -> float | None:
    cmd = ["powershell", "-NoProfile", "-Command",
           f"(Get-Counter '{path}' -SampleInterval 1 -MaxSamples 2).CounterSamples | "
           "Select-Object -Last 1 -ExpandProperty CookedValue"]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=30).stdout.strip()
        return float(out.replace(",", ".")) if out else None
    except (subprocess.SubprocessError, ValueError, OSError):
        return None


def probe() -> dict:
    total = _counter(r"\Processor(_Total)\% Processor Time")
    mem_mb = _counter(r"\Memory\Available MBytes")
    own = _own_cpu_fraction()
    cpu_other = None if total is None else max(0.0, min(0.99, total / 100.0 - own))
    return {"time": time.strftime("%Y-%m-%dT%H:%M:%S"), "cpu_total": total,
            "cpu_other": cpu_other, "ram_free_gb": None if mem_mb is None else round(mem_mb / 1024, 2),
            "bench_s": round(bench(), 3), "pid": os.getpid()}


def _own_cpu_fraction() -> float:
    """CPU fraction of this process over a short window; the rest is 'other'."""
    t0, c0 = time.perf_counter(), time.process_time()
    time.sleep(0.2)
    return min(1.0, (time.process_time() - c0) / ((time.perf_counter() - t0) * (os.cpu_count() or 1)))


def calibrate(path: Path = REF_PATH) -> dict:
    """Store the idle reference: run on a quiet machine; the median of five benches."""
    runs = sorted(bench() for _ in range(5))
    ref = {"ref_bench_s": round(runs[2], 3), "runs": [round(r, 3) for r in runs],
           "time": time.strftime("%Y-%m-%dT%H:%M:%S"), "cpu_count": os.cpu_count()}
    path.write_text(json.dumps(ref, indent=1))
    return ref


def factor(path: Path = REF_PATH, p: dict | None = None) -> tuple[float, dict]:
    p = p or probe()
    ref = json.loads(path.read_text())["ref_bench_s"] if path.exists() else None
    f_bench = (p["bench_s"] / ref) if ref else 1.0
    f_cpu = 1.0 / (1.0 - p["cpu_other"]) if p.get("cpu_other") is not None else 1.0
    f = max(1.0, min(MAX_FACTOR, max(f_bench, f_cpu)))
    return round(f, 2), {**p, "ref_bench_s": ref, "factor_bench": round(f_bench, 2),
                         "factor_cpu": round(f_cpu, 2), "factor": round(f, 2)}


def scaled(base_seconds: float, path: Path = REF_PATH) -> tuple[float, dict]:
    f, info = factor(path)
    return round(base_seconds * f, 1), info


def wait_for_idle(max_wait_s: float = 1800, cpu_threshold: float = 0.35,
                  ram_floor_gb: float = 4.0, path: Path = REF_PATH) -> dict:
    """Block until other load is below threshold and memory is above the floor, or max_wait."""
    t0 = time.perf_counter()
    while True:
        f, info = factor(path)
        quiet = (info.get("cpu_other") is None or info["cpu_other"] < cpu_threshold) and \
                (info.get("ram_free_gb") is None or info["ram_free_gb"] > ram_floor_gb) and f < 1.5
        info["waited_s"] = round(time.perf_counter() - t0, 1)
        if quiet or info["waited_s"] >= max_wait_s:
            info["idle"] = quiet
            return info
        time.sleep(min(60, 10 + info["waited_s"] / 10))


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "probe"
    if cmd == "probe":
        print(json.dumps(probe()))
    elif cmd == "calibrate":
        print(json.dumps(calibrate(Path(sys.argv[2]) if len(sys.argv) > 2 else REF_PATH)))
    elif cmd == "factor":
        print(json.dumps(factor(Path(sys.argv[2]) if len(sys.argv) > 2 else REF_PATH)[1]))
    elif cmd == "wait":
        mw = float(sys.argv[2]) if len(sys.argv) > 2 else 1800
        th = float(sys.argv[3]) if len(sys.argv) > 3 else 0.35
        print(json.dumps(wait_for_idle(mw, th)))
    else:
        sys.exit(f"unknown command {cmd}")
