"""E1c receipt (E1b pattern): `run LABEL [--cwd D] [--env K=V] [--timeout S] -- CMD` runs CMD with
stdout+stderr to results/logs/LABEL.log and appends argv, cwd, env, UTC times, seconds, exit;
`note KEY JSON` appends a note; `facts` records versions, hashes and line counts."""
import datetime, hashlib, json, os, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECEIPT, LOGS = HERE / "results" / "receipt.json", HERE / "results" / "logs"
now = lambda: datetime.datetime.now(datetime.UTC).isoformat()
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()


def main(a):
    data = json.loads(RECEIPT.read_text()) if RECEIPT.exists() else {"commands": [], "facts": [], "notes": []}
    if a[0] == "note":
        data["notes"].append({"time_utc": now(), "key": a[1], "value": json.loads(a[2])})
    elif a[0] == "facts":
        py = [p.name for p in sorted(HERE.glob("*.py"))]
        data["facts"].append({"time_utc": now(), "apparatus_sha256": {f: sha(HERE / f) for f in py + ["plan.md"]},
                              "apparatus_lines": {f: len((HERE / f).read_text().splitlines()) for f in py},
                              **json.loads(a[1] if len(a) > 1 else "{}")})
    else:
        label, rest = a[1], a[2:]
        opts, cmd = rest[:rest.index("--")], rest[rest.index("--") + 1:]
        o = {"--env": []}
        for k, v in zip(opts[::2], opts[1::2]):
            o[k] = o[k] + [v] if k == "--env" else v
        extra = dict(e.split("=", 1) for e in o["--env"])
        LOGS.mkdir(parents=True, exist_ok=True)
        start, t0 = now(), time.perf_counter()
        with open(LOGS / f"{label}.log", "w", encoding="utf-8") as log:
            try:
                code = subprocess.run(cmd, cwd=o.get("--cwd"), env=dict(os.environ, **extra), stdout=log,
                                      stderr=subprocess.STDOUT, timeout=float(o["--timeout"]) if "--timeout" in o else None).returncode
            except subprocess.TimeoutExpired:
                code = "timeout"
        data["commands"].append({"label": label, "cmd": cmd, "cwd": o.get("--cwd"), "env": extra, "start_utc": start,
                                 "end_utc": now(), "seconds": round(time.perf_counter() - t0, 2), "exit": code})
        print(f"{label}: exit {code} in {time.perf_counter() - t0:.1f}s")
    RECEIPT.write_text(json.dumps(data, indent=1))


if __name__ == "__main__":
    main(sys.argv[1:])
