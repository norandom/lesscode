"""Host guard for heavy steps (promoted from the coordinator's scratch guard).

Before a heavy step LABEL: adaptive.wait_for_idle(max_wait, cpu_other_max, ram_floor). Proceed (exit 0)
if the host is idle, or if it is not idle but free RAM is above ram_min. Otherwise wait once more
(note LABEL-guard-2) and then stop (exit 3). Every note records the probe AND the thresholds used.

Defaults: max_wait 1800 s, cpu_other_max 0.35, ram_floor 8.0 GB (idle test), ram_min 6.0 GB (busy-but-ok).
The decision depends on host load; callers must treat exit 3 as "do not start the heavy step".

  guard.py LABEL --receipt PATH [--max-wait S] [--cpu F] [--ram-floor GB] [--ram-min GB]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import adaptive  # noqa: E402
import receipt_atomic  # noqa: E402

STOP = 3


def guard(label: str, receipt: Path, max_wait_s: float = 1800, cpu_other_max: float = 0.35,
          ram_floor_gb: float = 8.0, ram_min_gb: float = 6.0) -> int:
    thresholds = {"max_wait_s": float(max_wait_s), "cpu_other_max": cpu_other_max,
                  "ram_floor_gb": ram_floor_gb, "ram_min_gb": ram_min_gb}
    for attempt in (1, 2):
        info = adaptive.wait_for_idle(max_wait_s, cpu_other_max, ram_floor_gb)
        ok = bool(info["idle"]) or (info.get("ram_free_gb") or 0) > ram_min_gb
        decision = "run" if ok else ("wait again" if attempt == 1 else "stop")
        value = {"probe": info, "thresholds": thresholds, "attempt": attempt, "decision": decision}
        receipt_atomic.add_note(receipt, f"{label}-guard" + ("" if attempt == 1 else "-2"), value)
        print(label, json.dumps(value), flush=True)
        if ok:
            return 0
    return STOP


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("label")
    p.add_argument("--receipt", required=True, type=Path)
    p.add_argument("--max-wait", type=float, default=1800)
    p.add_argument("--cpu", type=float, default=0.35)
    p.add_argument("--ram-floor", type=float, default=8.0)
    p.add_argument("--ram-min", type=float, default=6.0)
    a = p.parse_args(argv)
    return guard(a.label, a.receipt, a.max_wait, a.cpu, a.ram_floor, a.ram_min)


if __name__ == "__main__":
    sys.exit(main())
