"""E2 static facts over the 781 frozen production files (plan step 3).

usage: static_facts.py SOURCE_ROOT MANIFEST OUT.json
Per callable (key "relpath::qualname::firstlineno", matching code.co_qualname/co_firstlineno):
kind, module, own body lines, own AST statements, write sites (attribute/subscript store paths,
`self`/`cls` replaced by the class), codec-list call sites. Per file: physical lines, statements.
"""
import ast, fnmatch, hashlib, importlib.util, json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("e2_codecs", HERE / "codecs.py")
CODECS = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(CODECS)
DEFS = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)

def module_name(rel):
    parts = rel[:-3].split("/")
    parts = parts[1:] if parts[0] == "src" else parts[2:]
    return ".".join(parts[:-1] if parts[-1] == "__init__" else parts)

def dotted(node, cls, aliases):
    if isinstance(node, ast.Name):
        return cls if node.id in ("self", "cls") and cls else aliases.get(node.id, node.id)
    if isinstance(node, ast.Attribute):
        base = dotted(node.value, cls, aliases)
        return base and f"{base}.{node.attr}"
    if isinstance(node, ast.Subscript):
        base = dotted(node.value, cls, aliases)
        return base and f"{base}[]"
    return None

def own_nodes(node):
    for child in ast.iter_child_nodes(node):
        if not isinstance(child, DEFS):
            yield child
            yield from own_nodes(child)

def local_names(fn):
    """Names bound inside the function body and not parameters: stores through them are temporaries."""
    if isinstance(fn, ast.ClassDef) or fn is None:
        return set()
    params = {a.arg for a in fn.args.posonlyargs + fn.args.args + fn.args.kwonlyargs}
    nodes = list(own_nodes(fn))
    shared = {n for x in nodes if isinstance(x, (ast.Global, ast.Nonlocal)) for n in x.names}
    return {x.id for x in nodes if isinstance(x, ast.Name) and isinstance(x.ctx, ast.Store)} - params - shared

def root(node):
    while isinstance(node, (ast.Attribute, ast.Subscript)):
        node = node.value
    return node.id if isinstance(node, ast.Name) else None

def is_codec(name, call):
    if name in CODECS.NONLITERAL_CTORS:
        return bool(call.args) and not isinstance(call.args[0], ast.Constant)
    last = name.rsplit(".", 1)[-1]
    return last.startswith(CODECS.PREFIX_METHODS) or any(fnmatch.fnmatchcase(name, p) for p in CODECS.CODECS)

def scan(rel, text):
    tree, mod = ast.parse(text), module_name(rel)
    aliases = {}
    for n in ast.walk(tree):
        if isinstance(n, ast.Import):
            aliases.update({a.asname or a.name.split(".")[0]: a.name if a.asname else a.name.split(".")[0] for a in n.names})
        elif isinstance(n, ast.ImportFrom) and n.module:
            aliases.update({a.asname or a.name: f"{n.module}.{a.name}" for a in n.names})
    callables, owner = {}, {}
    mkey = f"{rel}::<module>::1"
    callables[mkey] = {"kind": "module", "module": mod, "lines": [], "stmts": 0, "writes": [], "codec_sites": 0}

    def visit(node, prefix, cls, key, locs=frozenset()):
        for child in ast.iter_child_nodes(node):
            if isinstance(child, DEFS):
                callables[key]["stmts"] += isinstance(child, ast.stmt)
                name = "<lambda>" if isinstance(child, ast.Lambda) else child.name
                first = min([child.lineno] + [d.lineno for d in getattr(child, "decorator_list", [])])
                ckey = f"{rel}::{prefix}{name}::{first}"
                body = child.body if isinstance(child.body, list) else [child.body]
                kind = "class" if isinstance(child, ast.ClassDef) else "function"
                callables[ckey] = {"kind": kind, "module": mod, "lines": [], "stmts": 0, "writes": [], "codec_sites": 0}
                for ln in range(body[0].lineno, child.end_lineno + 1):
                    owner[ln] = ckey
                sub = f"{prefix}{name}." if kind == "class" else f"{prefix}{name}.<locals>."
                visit(child, sub, f"{prefix}{name}" if kind == "class" else cls, ckey, local_names(child))
                continue
            fact = callables[key]
            if isinstance(child, ast.stmt):
                fact["stmts"] += 1
            targets = child.targets if isinstance(child, ast.Assign) else [child.target] if isinstance(child, (ast.AugAssign, ast.AnnAssign)) else []
            for t in targets:
                for el in (t.elts if isinstance(t, ast.Tuple) else [t]):
                    if isinstance(el, (ast.Attribute, ast.Subscript)) and root(el) not in locs and (p := dotted(el, cls, {})):
                        fact["writes"].append(p)
            if isinstance(child, ast.Call):
                name = dotted(child.func, None, aliases) or ""
                if name in ("object.__setattr__", "setattr") and len(child.args) >= 2 and isinstance(child.args[1], ast.Constant):
                    base = root(child.args[0]) not in locs and dotted(child.args[0], cls, {})
                    base and fact["writes"].append(f"{base}.{child.args[1].value}")
                fact["codec_sites"] += bool(name) and is_codec(name, child)
            visit(child, prefix, cls, key, locs)

    visit(tree, "", None, mkey)
    for ln, key in owner.items():
        callables[key]["lines"].append(ln)
    taken = set(owner)
    callables[mkey]["lines"] = [ln for ln in range(1, len(text.splitlines()) + 1) if ln not in taken]
    for fact in callables.values():
        fact["writes"] = sorted(set(fact["writes"]))
    return callables

def main(root, manifest, out):
    files, callables = {}, {}
    for f in json.load(open(manifest))["files"]:
        if f["path"].startswith("packages/facdrone-testing/") or "/testing/" in f["path"] or f["path"].endswith("/testing.py"):
            continue  # test-support modules are test frames (plan amendment 3)
        raw = (Path(root) / f["path"]).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == f["sha256"], f["path"]
        text = raw.decode("utf-8")
        found = scan(f["path"], text)
        callables.update(found)
        files[f["path"]] = {"module": module_name(f["path"]), "physical_lines": len(text.splitlines()),
                            "ast_statements": sum(c["stmts"] for c in found.values())}
    Path(out).write_text(json.dumps({"files": files, "callables": callables}))
    print(len(files), "files", len(callables), "callables")

if __name__ == "__main__":
    main(*sys.argv[1:4])
