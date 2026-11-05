"""E3 measures v3: frozen `measures.measure` (imported, unchanged) as written, plus a separated-form statement count of the
five files. `measures3.py --root R [--structure S] --out O` | `--check FROZEN.json --root R [--structure S]` (as-written only;
exit 1 and the first difference on any mismatch). The trees are read only. Separated-form rule, version 1:
1. Every ast.stmt node counts 1, exactly as the frozen `stmts`; rules 2-8 only add extras to it.
2. `import a, b, c` counts one statement per name (extra len(names) - 1).
3. `from m import a, b` counts one statement per name (extra len(names) - 1); `from m import *` counts 1.
4. An assignment with one target that is a tuple or list, and a tuple or list value of the same length, neither holding a
   starred element, counts one statement per paired leaf, recursively (`[a, (b, c)] = 1, (2, 3)` counts 3).
5. An assignment with several targets `a = b = v` counts len(targets); rule 4 is then not applied to its targets.
6. A comparison with k comparators counts k (`0 < x < 1` adds 1 to its enclosing statement), wherever it occurs.
7. `del a, b` counts len(targets); `global a, b` and `nonlocal a, b` count len(names).
8. A `with` or `async with` statement with n items counts n (the nested form it replaces).
9. Not unpacked (count only as rule 1): augmented and annotated assignments; a tuple target with a non-literal or starred
   value (`a, b = f()`, `a, *b = ...`); boolean operators; conditional expressions; comprehensions; lambdas; walrus; for-loop
   targets; multi-type `except`; tuple returns, yields and call arguments; decorators; semicolon-joined lines (already
   separate ast.stmt nodes)."""
import ast, json, sys
from pathlib import Path
from measures import measure
from measures2 import diff
from surface import SET
RULE = {"version": "1", "text": __doc__[__doc__.index("1. Every"):]}
KEYS = ("import", "from_import", "tuple_assign", "multi_target", "chained_compare", "del", "global_nonlocal", "with_items")
def leaves(t, v) -> int:  # rule 4
    seq, star = (ast.Tuple, ast.List), lambda n: any(isinstance(e, ast.Starred) for e in n.elts)
    if isinstance(t, seq) and isinstance(v, seq) and len(t.elts) == len(v.elts) and not star(t) and not star(v):
        return sum(leaves(a, b) for a, b in zip(t.elts, v.elts))
    return 1
def separated(tree: ast.AST) -> dict:
    x, base = dict.fromkeys(KEYS, 0), 0
    for n in ast.walk(tree):
        base += isinstance(n, ast.stmt)
        if isinstance(n, ast.Import):
            x["import"] += len(n.names) - 1
        elif isinstance(n, ast.ImportFrom):
            x["from_import"] += len(n.names) - 1
        elif isinstance(n, ast.Assign):
            x["multi_target" if len(n.targets) > 1 else "tuple_assign"] += (
                len(n.targets) if len(n.targets) > 1 else leaves(n.targets[0], n.value)) - 1
        elif isinstance(n, ast.Compare):
            x["chained_compare"] += len(n.comparators) - 1
        elif isinstance(n, ast.Delete):
            x["del"] += len(n.targets) - 1
        elif isinstance(n, (ast.Global, ast.Nonlocal)):
            x["global_nonlocal"] += len(n.names) - 1
        elif isinstance(n, (ast.With, ast.AsyncWith)):
            x["with_items"] += len(n.items) - 1
    return {"stmts": base + sum(x.values()), "extra": x}
def separated_tree(root: Path) -> dict:
    files = {Path(f).name: separated(ast.parse((root / f).read_text(encoding="utf-8"))) for f in SET}
    return {"files": files, "total": {"stmts": sum(v["stmts"] for v in files.values()),
                                      "extra": {k: sum(v["extra"][k] for v in files.values()) for k in KEYS}}}
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
    sep = separated_tree(root)
    assert all(sep["files"][f]["stmts"] - sum(sep["files"][f]["extra"].values()) == v["stmts"] for f, v in written["files"].items())
    res = {"as_written": written, "separated": sep, "rule": RULE}
    Path(opt("--out")).write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps({"as_written": written["total"]["stmts"], "separated": sep["total"], "rule_version": RULE["version"]}))
