"""E1c-2's step2.py (imported, unchanged) with its receipt rebound to E3 (receipt3): one heavy step with a
probed, scaled deadline. `step3.py LABEL BASE [receipt opts] -- CMD`; '{T}' in CMD becomes scaled(BASE).
A timeout-type outcome at factor > 1.5 waits for idle (max 30 min) and reruns once as LABEL-rerun."""
import json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE), str(HERE.parent / "e1c2-concolic-rerun")]
import step2  # noqa: E402  (binds receipt to E1c-2 ...)
import receipt3  # noqa: E402,F401  (... and this rebinds the same module object to E3)
if __name__ == "__main__":
    a = sys.argv[1:]
    i = a.index("--")
    entry, f, timed_out = step2.step(a[0], step2.base_of(a[1]), a[2:i], a[i + 1:])
    if timed_out and f > 1.5:
        step2.receipt.main(["note", f"{a[0]}-wait", json.dumps(step2.adaptive.wait_for_idle(1800))])
        entry, f, timed_out = step2.step(a[0] + "-rerun", step2.base_of(a[1]), a[2:i], a[i + 1:])
    print(json.dumps({"label": entry["label"], "exit": entry["exit"], "factor": f, "timeout_type": timed_out}))
