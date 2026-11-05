"""E3 measures v2: frozen `measures.measure` (imported, unchanged) as written, plus statements and lines after one pinned
formatter on copies of the five files. `measures2.py --root R [--structure S] --fmt-dir D --out O` | `--check FROZEN.json`
(as-written only; exit 1 and the first difference on any mismatch). The trees are read only; copies are formatted in D."""
import json, shutil, subprocess, sys
from pathlib import Path
from measures import measure
from surface import SET
FMT = {"name": "ruff format", "version": "0.17.3", "settings": {"isolated": True, "line-length": 100, "target-version": "py313",
       "other": "ruff defaults (double quotes, 4-space indent, magic trailing comma respected)"}}
RUFF = [shutil.which("uv") or "uv.exe", "tool", "run", "--offline", "--python", "3.13", f"ruff=={FMT['version']}"]
ARGS = ["format", "--isolated", "--line-length", "100", "--target-version", "py313", "--no-cache"]
def diff(a, b, path="$"):  # first difference between two JSON values, or None
    if type(a) is not type(b) or not isinstance(a, (dict, list)):
        return None if a == b else f"{path}: {json.dumps(a)[:200]} != {json.dumps(b)[:200]}"
    if isinstance(a, dict) and a.keys() != b.keys():
        return f"{path}: keys {sorted(a.keys() ^ b.keys())} differ"
    if isinstance(a, list) and len(a) != len(b):
        return f"{path}: length {len(a)} != {len(b)}"
    return next((d for k in (a if isinstance(a, dict) else range(len(a))) if (d := diff(a[k], b[k], f"{path}[{k!r}]"))), None)
def canonical(root: Path, fmt_dir: Path) -> dict:
    shutil.rmtree(fmt_dir, ignore_errors=True)
    for f in SET:
        (fmt_dir / f).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / f, fmt_dir / f)
    ver = subprocess.run(RUFF + ["--version"], capture_output=True, text=True, check=True).stdout.split()
    assert ver[-1] == FMT["version"], ver
    subprocess.run(RUFF + ARGS + SET, cwd=fmt_dir, check=True, capture_output=True)
    m = measure(fmt_dir, None)
    return {"files": {f: {k: v[k] for k in ("lines", "stmts")} for f, v in m["files"].items()},
            "total": {k: m["total"][k] for k in ("lines", "stmts")}}
if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or "--help" in a or "-h" in a:
        sys.exit(print(__doc__) or 0)
    opt = lambda k: a[a.index(k) + 1] if k in a else None
    st = json.loads(Path(opt("--structure")).read_text(encoding="utf-8")) if opt("--structure") else None
    root = Path(opt("--root")).resolve()
    written = measure(root, st)
    if opt("--check"):
        d = diff(written, json.loads(Path(opt("--check")).read_text(encoding="utf-8")))
        print(f"as_written != {opt('--check')}: {d}" if d else f"as_written == {opt('--check')}")
        sys.exit(1 if d else 0)
    res = {"as_written": written, "canonical": canonical(root, Path(opt("--fmt-dir")).resolve()),
           "formatter": FMT | {"command": RUFF + ARGS + ["<the five files>"], "cwd": "<fmt-dir copy of the five files>"}}
    Path(opt("--out")).write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps({"as_written": {k: written["total"][k] for k in ("lines", "stmts")}, "canonical": res["canonical"]["total"]}))
