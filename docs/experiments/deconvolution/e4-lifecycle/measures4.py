"""E4 measures (deconvolution-loop task 6.1): the frozen E3 measurers, unchanged in code, rebound to the seven lifecycle
files. `measures.measure` (as written: lines, ast.stmt, functions, write sites, intra-set call edges, module hops, the
structure check), `measures2.canonical` (ruff format 0.17.3 on copies under --fmt-dir, settings measures2.FMT) and
`measures3.separated_tree` (separated-form statements, rule version 1) read their file list from a module-level SET;
this script rebinds that one name to surface4.SET and calls the functions as they are.
  measures4.py --root R [--structure S.json] --fmt-dir D --label L [--out-dir results]
writes <out-dir>/measures4-<L>.json (as written), measures4-<L>-canonical.json and measures4-<L>-separated.json in the
shapes of measures.py, measures2.py and measures3.py. The trees are read only; copies are formatted in D."""
import json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e3-evidence-regeneration"))
import measures, measures2, measures3  # noqa: E402  (frozen E3 files; only their module-level SET is rebound below)
from surface4 import SET  # noqa: E402  (the seven lifecycle files, relative to a root)
for _m in (measures, measures2, measures3):
    _m.SET = SET


def measure_all(root: Path, st: dict | None, fmt_dir: Path) -> tuple[dict, dict, dict]:
    written = measures.measure(root, st)
    canon = measures2.canonical(root, fmt_dir)
    sep = measures3.separated_tree(root)
    assert all(sep["files"][f]["stmts"] - sum(sep["files"][f]["extra"].values()) == v["stmts"]
               for f, v in written["files"].items()), "separated-form base differs from the as-written count"
    small = {k: written["total"][k] for k in ("lines", "stmts")}
    return (written,
            {"as_written": {"files": {f: {k: v[k] for k in ("lines", "stmts")} for f, v in written["files"].items()}, "total": small},
             "canonical": canon, "formatter": measures2.FMT | {"command": measures2.RUFF + measures2.ARGS + ["<the seven files>"],
                                                                 "cwd": "<fmt-dir copy of the seven files>"}, "set": SET},
            {"as_written": {"total": {"stmts": written["total"]["stmts"]}}, "separated": sep, "rule": measures3.RULE, "set": SET})


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or "--help" in a or "-h" in a:
        sys.exit(print(__doc__) or 0)
    opt = lambda k, d=None: a[a.index(k) + 1] if k in a else d  # noqa: E731
    st = json.loads(Path(opt("--structure")).read_text(encoding="utf-8")) if opt("--structure") else None
    root, label, out = Path(opt("--root")).resolve(), opt("--label"), Path(opt("--out-dir", HERE / "results"))
    w, c, s = measure_all(root, st, Path(opt("--fmt-dir")).resolve())
    for suffix, body in (("", w), ("-canonical", c), ("-separated", s)):
        (out / f"measures4-{label}{suffix}.json").write_text(json.dumps(body, indent=1), encoding="utf-8")
    print(json.dumps({"as_written": w["total"], "canonical": c["canonical"]["total"], "separated": s["separated"]["total"]["stmts"],
                      **({"structure_pass": w["structure"]["pass"], "pure_share": w["structure"]["pure_share"]} if st else {})}))
