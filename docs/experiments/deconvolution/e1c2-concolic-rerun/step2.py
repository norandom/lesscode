"""Amendment 3 (adaptive budgets): run one heavy step through receipt2 with a probed, scaled deadline.
  step2.py LABEL BASE [receipt opts] -- CMD    BASE: seconds, or `armB:ROOT` (E1b's measured deadline of ROOT's seed set)
'{T}' in CMD becomes scaled(BASE). The probe and factor go into the step's receipt entry. A timeout-type outcome
(receipt timeout, run_arm "deadline exceeded", or CrossHair 'No differences found' without exhaustion) at factor > 1.5
waits for idle (max 30 min) and reruns once as LABEL-rerun; both attempts stay in the receipt."""
import json, os, sys
from pathlib import Path
import receipt2
from receipt2 import receipt
sys.path.insert(0, str(receipt2.HERE.parent))
import adaptive  # noqa: E402


def base_of(b: str) -> float:
    if not b.startswith("armB:"):
        return float(b)
    import run_e1c2_arm as A
    A.R.run_arm.groups("B", Path(b[5:]).resolve())
    return json.loads(A.R.run_arm.DEADLINES.read_text())[A.R.run_arm.set_key("B")]["deadline"]


def step(label, base, opts, cmd):
    secs, info = adaptive.scaled(base)
    receipt.main(["run", label, *opts, "--timeout", str(secs + 120), "--", *[c.replace("{T}", str(secs)) for c in cmd]])
    data = json.loads(receipt.RECEIPT.read_text())
    entry = data["commands"][-1] | {"adaptive": {"base_s": base, "scaled_s": secs, "probe": info}}
    data["commands"][-1] = entry
    receipt.RECEIPT.write_text(json.dumps(data, indent=1))
    log = (receipt.LOGS / f"{label}.log").read_text(encoding="utf-8", errors="replace")
    return entry, info["factor"], entry["exit"] == "timeout" or "deadline exceeded" in log or ("No differences found" in log and "exhausted" not in log)


if __name__ == "__main__":
    a = sys.argv[1:]
    i = a.index("--")
    entry, f, timed_out = step(a[0], base_of(a[1]), a[2:i], a[i + 1:])
    if timed_out and f > 1.5:
        receipt.main(["note", f"{a[0]}-wait", json.dumps(adaptive.wait_for_idle(1800))])
        entry, f, timed_out = step(a[0] + "-rerun", base_of(a[1]), a[2:i], a[i + 1:])
    print(json.dumps({"label": entry["label"], "exit": entry["exit"], "factor": f, "timeout_type": timed_out}))
