"""Pure share by statement count (req 5.4), complementing the measurer's share by function count.
`pureshare.py ROOT --structure S.json --out O`. Every ast.stmt of the five files (the frozen `stmts` definition) is
attributed to its innermost enclosing def (measures.defs, same qualnames as the structure check) and classed as
pure, shell or undeclared by the declaration; statements outside every def are reported separately. The tree is read only."""
import ast, json, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "e3-evidence-regeneration"))
from measures import defs  # noqa: E402
from surface import SET  # noqa: E402


def share(root: Path, st: dict) -> dict:
    pure, shell, per, tot = set(st["pure"]), set(st["shell"]), {}, {"pure": 0, "shell": 0, "undeclared": 0, "outside_defs": 0}
    for f in SET:
        name, tree = Path(f).name, ast.parse((root / f).read_text(encoding="utf-8"))
        D = defs(tree)
        for s in (n for n in ast.walk(tree) if isinstance(n, ast.stmt)):
            inner = [(n.lineno, q) for q, n in D if n.lineno <= s.lineno <= n.end_lineno]
            key = f"{name}::{max(inner)[1]}" if inner else None
            cls = "outside_defs" if key is None else "pure" if key in pure else "shell" if key in shell else "undeclared"
            tot[cls] += 1
            if key:
                per[key] = per.get(key, 0) + 1
    in_defs, all_ = tot["pure"] + tot["shell"] + tot["undeclared"], sum(tot.values())
    return {"root": str(root), "stmts": tot, "stmts_in_defs": in_defs, "stmts_all": all_,
            "pure_share_by_stmts_in_defs": round(tot["pure"] / max(1, in_defs), 3),
            "pure_share_by_stmts_all": round(tot["pure"] / max(1, all_), 3),
            "pure_share_by_funcs": round(len(pure) / max(1, len(per)), 3), "per_function": per}


if __name__ == "__main__":
    a = sys.argv[1:]
    st = json.loads(Path(a[a.index("--structure") + 1]).read_text(encoding="utf-8"))
    res = share(Path(a[0]).resolve(), st)
    Path(a[a.index("--out") + 1]).write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps({k: v for k, v in res.items() if k != "per_function"}))
