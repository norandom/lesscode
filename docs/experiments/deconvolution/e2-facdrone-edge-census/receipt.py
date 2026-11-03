"""Append one command or one facts record to results/receipt.json.

  receipt.py run LABEL [--cwd DIR] [--env K=V ...] -- CMD ...   run CMD, record times and exit code
  receipt.py facts VENV_PYTHON                                     record versions and hashes
"""
import datetime, hashlib, json, os, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECEIPT = HERE / "results" / "receipt.json"
now = lambda: datetime.datetime.now(datetime.UTC).isoformat()
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
out = lambda *c, **k: subprocess.run(c, capture_output=True, text=True, **k).stdout.strip()

def main(argv):
    data = json.loads(RECEIPT.read_text()) if RECEIPT.exists() else {"commands": [], "facts": []}
    if argv[0] == "facts":
        files = sorted(p.name for p in HERE.glob("*.py")) + ["plan.md", "README.md"]
        data["facts"].append({"time_utc": now(), "uv": out("uv.exe", "--version"),
            "venv_python": out(argv[1], "-c", "import sys; print(sys.version)"),
            "venv_packages": out("uv.exe", "pip", "freeze", "--python", argv[1]).splitlines(),
            "facdrone_head": out("git", "-C", "D:/Source/facdrone", "rev-parse", "HEAD"),
            "facdrone_status_short": out("git", "-C", "D:/Source/facdrone", "status", "--short"),
            "apparatus_sha256": {f: sha(HERE / f) for f in files if (HERE / f).exists()},
            "apparatus_lines": {f: len((HERE / f).read_text().splitlines()) for f in files if f.endswith(".py")}})
    else:
        label, rest = argv[1], argv[2:]
        sep = rest.index("--")
        opts, cmd, cwd, env = rest[:sep], rest[sep + 1:], None, dict(os.environ)
        extra = {}
        for k, v in zip(opts[::2], opts[1::2]):
            if k == "--cwd": cwd = v
            elif k == "--env": extra.update([v.split("=", 1)])
        env.update(extra)
        start, t0 = now(), time.perf_counter()
        code = subprocess.run(cmd, cwd=cwd, env=env).returncode
        data["commands"].append({"label": label, "cmd": cmd, "cwd": cwd, "env": extra, "start_utc": start,
                                 "end_utc": now(), "seconds": round(time.perf_counter() - t0, 2), "exit": code})
    RECEIPT.write_text(json.dumps(data, indent=1))

if __name__ == "__main__":
    main(sys.argv[1:])
