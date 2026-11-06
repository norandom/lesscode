"""E4 public surface: `surface4.py [ROOT] [--out PATH]` (default: the frozen copy, results/public-surface.json).
E3's surface.py with the lifecycle monitor group as the set (seven modules). Imports, module aliases' attribute reads and monkeypatch targets over src, packages, tests;
tags prod/test; kind in the original. Among the seven: `intra_set`; reads of names a module lacks: `unresolved`."""
import ast, json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
SET = [f"src/facdrone/shells/{m}.py" for m in ("lifecycle_monitor_runner", "lifecycle_store", "cli/lifecycle_group",
       "lifecycle_evidence", "lifecycle_projection", "cppi/lifecycle_seams", "wiring/lifecycle_root")]
MOD = {f[4:-3].replace("/", "."): f for f in SET}
def kinds(root: Path, f: str) -> dict:
    out = {}
    for n in ast.parse((root / f).read_text(encoding="utf-8")).body:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            out[n.name] = "class" if isinstance(n, ast.ClassDef) else "function"
        elif isinstance(n, (ast.Assign, ast.AnnAssign)):
            out.update({t.id: "constant" for t in getattr(n, "targets", [getattr(n, "target", None)]) if isinstance(t, ast.Name)})
        elif isinstance(n, ast.ImportFrom):
            out.update({a.asname or a.name: f"reexport:{n.module}" for a in n.names})
    return out
def scan(tree, add) -> None:
    alias = {}  # local name -> regenerated module
    for n in (n for n in ast.walk(tree) if isinstance(n, (ast.ImportFrom, ast.Import))):
        for a in n.names:
            if isinstance(n, ast.ImportFrom) and n.module in MOD:
                add(n.module, a.name, "import")
            elif (m := a.name if isinstance(n, ast.Import) else f"{n.module}.{a.name}") in MOD:
                add(m, "<module>", "module")
                alias[a.asname or a.name] = m
    for n in ast.walk(tree):
        if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id in alias:
            add(alias[n.value.id], n.attr, "attribute")
        elif isinstance(n, ast.Call) and getattr(n.func, "attr", getattr(n.func, "id", "")) == "setattr" and len(n.args) > 1:
            a0, a1 = n.args[:2]
            if isinstance(a0, ast.Name) and a0.id in alias and isinstance(a1, ast.Constant):
                add(alias[a0.id], a1.value, "patched")
            elif isinstance(a0, ast.Constant) and str(a0.value).rpartition(".")[0] in MOD:
                add(a0.value.rpartition(".")[0], a0.value.rpartition(".")[2], "patched")
def main(root: Path, out: Path) -> dict:
    kind, files = {m: kinds(root, f) for m, f in MOD.items()}, [p for d in ("src", "packages", "tests") for p in sorted((root / d).rglob("*.py"))]
    body = {"root": str(root), "files_scanned": len(files), "modules": {m: {"file": f, "names": {}, "importers": {}} for m, f in MOD.items()},
            "unresolved": {m: {} for m in MOD}, "intra_set": {m: {} for m in MOD}}
    for p in files:
        rel = p.relative_to(root).as_posix()
        tag = "test" if rel.startswith("tests/") or "/tests/" in rel or "facdrone_testing" in rel else "prod"
        def add(m, name, via, rel=rel, tag=tag):
            k = "module" if name == "<module>" else kind[m].get(name)
            if rel in SET or (k is None and via == "attribute"):
                return body["intra_set" if rel in SET else "unresolved"][m].setdefault(name, []).append(rel)
            e = body["modules"][m]["names"].setdefault(name, {"kind": k or "missing", "tags": [], "via": [], "importers": []})
            [e[x].append(v) for x, v in (("tags", tag), ("via", via), ("importers", rel)) if v not in e[x]]
            body["modules"][m]["importers"][rel] = tag
        scan(ast.parse(p.read_text(encoding="utf-8")), add)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=1, sort_keys=True), encoding="utf-8")
    return body
if __name__ == "__main__":
    a = sys.argv[1:]
    b = main(Path(a[0] if a and a[0] != "--out" else HERE.parents[3] / ".runs/e2-facdrone/source"),
             Path(a[a.index("--out") + 1]) if "--out" in a else HERE / "results" / "public-surface.json")
    [print(m, {n: (e["kind"], e["tags"]) for n, e in sorted(s["names"].items())}) for m, s in b["modules"].items()]
