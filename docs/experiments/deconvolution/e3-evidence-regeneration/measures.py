"""E3 measures of the five regenerated files: `measures.py ROOT [--structure S.json] --out results/measures-<label>.json`
Definitions in the README. S = {"pure": ["file.py::qualname", ...], "shell": [...]}: each pure function is checked
(parameters, SQL calls), each write site must lie in a declared shell function (innermost enclosing def)."""
import ast, json, re, sys
from pathlib import Path
from surface import SET
WRITE = re.compile(r"\.(execute|add|add_all|commit|flush|delete|merge)\(|\b(insert|update|delete)\(")
BAD = re.compile(r"^(engine|conn|connection|session|bind)$|_(engine|conn|connection|session)$")
SQL = {"execute", "begin", "connect", "commit", "add", "flush", "merge", "delete"}
def defs(node, pre="") -> list:  # (qualname, def node), nested defs and methods included
    out = []
    for c in ast.iter_child_nodes(node):
        q = f"{pre}{c.name}" if isinstance(c, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) else None
        out += ([(q, c)] if q and not isinstance(c, ast.ClassDef) else []) + defs(c, f"{q}." if q else pre)
    return out
def measure(root: Path, st: dict | None) -> dict:
    src = {Path(f).name: (root / f).read_text(encoding="utf-8") for f in SET}
    T = {f: ast.parse(s) for f, s in src.items()}
    D, names, edges, files, sites = {f: defs(t) for f, t in T.items()}, {}, set(), {}, []
    [names.setdefault(n.name, set()).add(f) for f, ds in D.items() for _, n in ds]
    for f, s in src.items():
        for q, n in D[f]:
            for c in (c for c in ast.walk(n) if isinstance(c, ast.Call)):
                if (callee := getattr(c.func, "id", getattr(c.func, "attr", None))) in names and callee != n.name:
                    edges |= {(f, n.name, g, callee) for g in names[callee]}
        for m in WRITE.finditer(s):
            ln = s.count("\n", 0, m.start()) + 1
            inner = [(n.lineno, q) for q, n in D[f] if n.lineno <= ln <= n.end_lineno]
            sites.append({"file": f, "line": ln, "match": m.group(0), "in": f"{f}::{max(inner)[1]}" if inner else None})
        files[f] = {"lines": len(s.splitlines()), "stmts": sum(isinstance(x, ast.stmt) for x in ast.walk(T[f])),
                    "funcs": len({n.name for _, n in D[f]}), "qualnames": len(D[f]), "writes": sum(x["file"] == f for x in sites)}
    total = {k: sum(v[k] for v in files.values()) for k in ("lines", "stmts", "funcs", "qualnames", "writes")}
    out = {"root": str(root), "files": files, "total": total | {"edges": len(edges), "module_hops": len({(a, c) for a, _, c, _ in edges if a != c})},
           "edges": sorted(map(list, edges)), "write_sites": sites}
    if st is not None:
        nodes, pure, shell = {f"{f}::{q}": n for f, ds in D.items() for q, n in ds}, {}, set(st.get("shell", []))
        for k in st.get("pure", []):
            n = nodes.get(k)
            a = n.args if n else None
            why = ["not defined"] if n is None else [f"param {p.arg}" for p in a.posonlyargs + a.args + a.kwonlyargs + [
                x for x in (a.vararg, a.kwarg) if x] if BAD.search(p.arg)] + [f"call .{c.func.attr} line {c.lineno}" for c in ast.walk(n)
                if isinstance(c, ast.Call) and isinstance(c.func, ast.Attribute) and c.func.attr in SQL]
            pure[k] = {"pass": not why, "why": why}
        w, declared = [s | {"pass": s["in"] in shell} for s in sites], set(st.get("pure", [])) | shell
        out["structure"] = {"pure": pure, "write_sites": w, "undeclared": sorted(set(nodes) - declared),
                            "declared_unknown": sorted(declared - set(nodes)), "pure_share": round(len(pure) / max(1, len(nodes)), 3),
                            "pass": all(v["pass"] for v in pure.values()) and all(x["pass"] for x in w)}
    return out
if __name__ == "__main__":
    a = sys.argv[1:]
    st = json.loads(Path(a[a.index("--structure") + 1]).read_text(encoding="utf-8")) if "--structure" in a else None
    res = measure(Path(a[0]).resolve(), st)
    Path(a[a.index("--out") + 1]).write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps({"total": res["total"], **({"structure_pass": res["structure"]["pass"]} if st else {})}))
