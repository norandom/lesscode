"""Static test-infrastructure detection for a Python repository and a scope (deconvolution-loop task 2.2).

Relocated verbatim into the evaluator package (constraint-guided-compression task 4.1);
the frozen experiment original stays untouched. The report envelope is cross-language:
runner, fixtures, external_store, scope_references, seeds, marker_expression,
deselections and rule_version are language-neutral; pytest-specific settings are
runner subfields, and non-Python repositories return an unknown runner cleanly (R32).

  testinfra.py REPO_ROOT --scope FILE... [--baseline-failures JSON] [--seeds-file TXT]
               [--site-packages DIR] [--out DIR] [--stem NAME]

Writes OUT/NAME.json and OUT/NAME.md (OUT defaults to the cwd, NAME to testinfra). Static only: file reads,
TOML/INI parsing and the ast module; specimen code is never imported or run.

Detected: the pytest configurations (rootdir candidates, testpaths, markers, addopts, python_files,
pythonpath, plugins), conftest fixtures and `pytest_plugins` modules, external-store facts and host
hazards with file and line, references to the scope, seed sets by level, baseline deselections that
fall in the seeds, and a marker expression.

Reference kinds follow E3's surface.py (import, module, attribute, patched), reimplemented here rather
than imported, with three extensions: aliases of `import a.b.c` without `as`, `mock.patch("a.b.name")`
and `patch.object(alias, "name")` string targets (kind patched), and relative imports.

Marker rule: a declared marker is excluded when its description declares it opt-in ("opt-in", "opts
in", "never part of the gate"), or names a hardware, network, broker or port resource and does not say
the tests skip on their own. Store resources (database) are reported but do not exclude: the fixture
plugin may provision the store. Gate scripts' `pytest -m` expressions are reported for comparison.
"""
from __future__ import annotations

import argparse
import ast
import configparser
import fnmatch
import json
import os
import re
import shlex
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

SKIP_DIRS = {"node_modules", "__pycache__", "venv", "site-packages", "build", "dist"}
RULE_VERSION = "testinfra-rules-v1"  # envelope rule stamp: config/marker/store/hazard semantics
CONFIG_NAMES = ("pyproject.toml", "pytest.ini", "setup.cfg", "tox.ini")
SCRIPT_GLOBS = ("*.sh", "*.ps1", "*.bat", "*.cmd", "Makefile", "justfile", "*.yml", "*.yaml", "noxfile.py")
STORE_LIBS = ("pgembed", "pytest_postgresql", "testing.postgresql", "psycopg", "psycopg2", "asyncpg", "sqlalchemy",
              "alembic", "testcontainers", "docker", "redis", "pymongo", "sqlite3")
STORE_NAME = re.compile(r"pg|postgres|dsn|database|_db|db_|^db$|sql", re.I)
FIXTURE_STORE = re.compile(r"pg|postgres|db|database|engine|sql", re.I)
DAEMONS = ("pg_ctl", "postgres", "redis-server", "mongod", "mysqld")
POSIX_ATTRS = {"geteuid", "getuid", "getgid", "getegid", "fork", "symlink", "chown", "killpg", "setsid"}
POSIX_MODULES = {"fcntl", "pwd", "grp", "termios", "resource", "posix"}
OPT_IN = re.compile(r"\bopt[- ]?in\b|\bopts in\b|never part of the gate", re.I)
SELF_SKIP = re.compile(r"\bskips?\b", re.I)
RESOURCES = {"network": r"\bnetwork|\binternet\b|\bsocket|\bremote\b", "broker": r"\bbroker",
             "hardware": r"\bhardware\b|\bgpu\b|\bdevice\b|\bserial port", "port": r"\b(tcp |udp )?ports?\b(?![_-])",
             "store": r"postgres|\bdatabase\b|\bredis\b|\bmongo"}


def rel(root: Path, p: Path) -> str:
    return p.resolve().relative_to(root).as_posix()


def walk(root: Path):
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if not x.startswith(".") and x not in SKIP_DIRS)
        for f in sorted(files):
            yield Path(d) / f


def as_list(v) -> list[str]:
    if v is None:
        return []
    if isinstance(v, str):
        return [x.strip() for x in v.splitlines() if x.strip()] if "\n" in v.strip() else v.split()
    return [str(x) for x in v]


def read_config(path: Path) -> dict | None:
    """The pytest section of one config file, normalised; None when it has none."""
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.name == "pyproject.toml":
        data = tomllib.loads(text)
        tool = data.get("tool", {}).get("pytest")
        ep = data.get("project", {}).get("entry-points", {}).get("pytest11", {})
        if tool is None:
            return {"entry_points": ep} if ep else None
        sect = tool.get("ini_options", tool)
        markers = as_list(sect.get("markers"))
        addopts = " ".join(as_list(sect.get("addopts"))) if not isinstance(sect.get("addopts"), str) else sect["addopts"]
    else:
        cp = configparser.ConfigParser(interpolation=None)
        cp.read_string(text)
        name = "tool:pytest" if path.name == "setup.cfg" else "pytest"
        if not cp.has_section(name):
            return None
        sect, ep = dict(cp[name]), {}
        markers = [x.strip() for x in sect.get("markers", "").splitlines() if x.strip()]
        addopts = " ".join(sect.get("addopts", "").split())
    toks = shlex.split(addopts) if addopts else []
    plugins = [toks[i + 1] for i, t in enumerate(toks[:-1]) if t == "-p"] + [t[2:] for t in toks if t.startswith("-p") and len(t) > 2]
    return {"testpaths": as_list(sect.get("testpaths")), "pythonpath": as_list(sect.get("pythonpath")),
            "python_files": as_list(sect.get("python_files")) or ["test_*.py", "*_test.py"],
            "markers": {m.partition(":")[0].strip(): m.partition(":")[2].strip() for m in markers},
            "addopts": addopts, "plugins": plugins, "required_plugins": as_list(sect.get("required_plugins")),
            "entry_points": ep}


def dotted(n) -> str | None:
    parts = []
    while isinstance(n, ast.Attribute):
        parts.append(n.attr)
        n = n.value
    return ".".join([n.id, *reversed(parts)]) if isinstance(n, ast.Name) else None


def top_names(tree) -> dict[str, str]:
    out = {}
    for n in tree.body:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            out[n.name] = "class" if isinstance(n, ast.ClassDef) else "function"
        elif isinstance(n, (ast.Assign, ast.AnnAssign)):
            out.update({t.id: "constant" for t in getattr(n, "targets", [getattr(n, "target", None)]) if isinstance(t, ast.Name)})
        elif isinstance(n, (ast.ImportFrom, ast.Import)):
            out.update({(a.asname or a.name).split(".")[0]: "reexport" for a in n.names})
    return out


def str_consts(tree) -> dict[str, str]:
    return {t.id: n.value.value for n in tree.body if isinstance(n, ast.Assign) and isinstance(n.value, ast.Constant)
            and isinstance(n.value.value, str) for t in n.targets if isinstance(t, ast.Name)}


class Repo:
    def __init__(self, root: Path, site: Path | None):
        self.root, self.site = root.resolve(), site
        self.py, self.configs, self.scripts, self.trees, self.errors, pkg_dirs = [], [], [], {}, [], set()
        for p in walk(self.root):
            r = rel(self.root, p)
            if p.suffix == ".py":
                self.py.append(r)
            if p.name == "pyproject.toml":
                pkg_dirs.add(Path(r).parent.as_posix())
            if p.name in CONFIG_NAMES and (c := read_config(p)) is not None:
                c.update(file=r, rootdir=Path(r).parent.as_posix())
                self.configs.append(c)
            if any(fnmatch.fnmatch(p.name, g) for g in SCRIPT_GLOBS):
                self.scripts.append(r)
        roots = {"."} | {c["rootdir"] for c in self.configs}
        roots |= {os.path.normpath(Path(c["rootdir"]) / x).replace("\\", "/") for c in self.configs for x in c.get("pythonpath", [])}
        roots |= pkg_dirs
        roots |= {f"{r}/src" if r != "." else "src" for r in list(roots) if (self.root / r / "src").is_dir()}
        self.import_roots = sorted(r for r in roots if (self.root / r).is_dir())
        self.mod_files, self.file_mods = defaultdict(set), defaultdict(set)
        for f in self.py:
            for r in self.import_roots:
                pre = "" if r == "." else r + "/"
                if f.startswith(pre):
                    parts = f[len(pre):-3].split("/")
                    parts = parts[:-1] if parts[-1] == "__init__" else parts
                    if parts and all(x.isidentifier() for x in parts):
                        self.mod_files[".".join(parts)].add(f)
                        self.file_mods[f].add(".".join(parts))

    def tree(self, f: str):
        if f not in self.trees:
            try:
                self.trees[f] = ast.parse((self.root / f).read_text(encoding="utf-8", errors="replace"))
            except SyntaxError:
                self.trees[f] = None
                self.errors.append(f)
        return self.trees[f]

    def site_file(self, mod: str) -> Path | None:
        if self.site is None:
            return None
        for cand in (self.site / (mod.replace(".", "/") + ".py"), self.site / mod.replace(".", "/") / "__init__.py"):
            if cand.is_file():
                return cand
        return None


def imports_of(tree, mods: set[str]) -> list[tuple[str, str | None, str | None]]:
    """(module, imported name or None, local alias) for every import, relative ones resolved."""
    out = []
    for n in ast.walk(tree):
        if isinstance(n, ast.Import):
            out += [(a.name, None, a.asname or a.name) for a in n.names]
        elif isinstance(n, ast.ImportFrom):
            base = n.module or ""
            if n.level and mods:
                pkg = sorted(mods)[0].split(".")[: -n.level] if n.level else []
                base = ".".join([*pkg, *([n.module] if n.module else [])])
            out += [(base, a.name, a.asname or a.name) for a in n.names]
    return out


def scan_refs(tree, mods: set[str], scope: dict[str, dict], add) -> None:
    """surface.py's reference kinds over one file; add(module, name, kind)."""
    alias = {}  # local dotted name -> scope module
    for base, name, local in imports_of(tree, mods):
        if name is not None and base in scope:
            add(base, name, "import")
        full = base if name is None else f"{base}.{name}"
        if full in scope:
            add(full, "<module>", "module")
            alias[local] = full
    for n in ast.walk(tree):
        if isinstance(n, ast.Attribute) and dotted(n.value) in alias:
            add(alias[dotted(n.value)], n.attr, "attribute")
        elif isinstance(n, ast.Call) and n.args:
            fn = getattr(n.func, "attr", getattr(n.func, "id", ""))
            a0 = n.args[0]
            if fn in ("setattr", "delattr", "object") and len(n.args) > 1 and dotted(a0) in alias \
                    and isinstance(n.args[1], ast.Constant):
                add(alias[dotted(a0)], n.args[1].value, "patched")
            elif fn in ("setattr", "delattr", "patch") and isinstance(a0, ast.Constant) and isinstance(a0.value, str):
                mod, _, attr = a0.value.rpartition(".")
                while mod and mod not in scope:
                    mod, _, _ = mod.rpartition(".")
                if mod:
                    add(mod, a0.value[len(mod) + 1:].split(".")[0] or attr, "patched")


def fixtures_in(tree) -> dict[str, ast.FunctionDef]:
    out = {}
    for n in ast.walk(tree):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
            for d in n.decorator_list:
                d = d.func if isinstance(d, ast.Call) else d
                if (dotted(d) or "").split(".")[-1] == "fixture":
                    out[n.name] = n
    return out


def fixture_defs(repo, host: str) -> list[tuple[str, tuple[str, ast.FunctionDef]]]:
    """Fixtures visible in host: its own, and fixtures it re-exports from in-tree modules by import."""
    t = repo.tree(host)
    if t is None:
        return []
    out = [(host, kv) for kv in fixtures_in(t).items()]
    for base, name, local in imports_of(t, repo.file_mods.get(host, set())):
        for src in sorted(repo.mod_files.get(base, ())) if name else []:
            if (st := repo.tree(src)) is not None and (fn := fixtures_in(st).get(name)) is not None:
                out.append((src, (local, fn)))
    return out


def autouse(fn) -> bool:
    return any(isinstance(d, ast.Call) and any(k.arg == "autouse" and getattr(k.value, "value", False) for k in d.keywords)
               for d in fn.decorator_list)


def requested(tree) -> set[str]:
    names = set()
    for n in ast.walk(tree):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test"):
            names |= {a.arg for a in n.args.args + n.args.kwonlyargs}
        if isinstance(n, ast.Call) and (dotted(n.func) or "").endswith("usefixtures"):
            names |= {a.value for a in n.args if isinstance(a, ast.Constant)}
    return names


def plugin_list(tree) -> list[str]:
    for n in tree.body:
        if isinstance(n, ast.Assign) and any(getattr(t, "id", "") == "pytest_plugins" for t in n.targets):
            v = n.value
            return [v.value] if isinstance(v, ast.Constant) else [e.value for e in getattr(v, "elts", []) if isinstance(e, ast.Constant)]
    return []


def detect(root: Path, scope_files: list[str], fails: dict | None, seeds_file: list[str] | None, site: Path | None) -> dict:
    repo = Repo(root, site)
    scope_files = [rel(repo.root, (repo.root / s) if not Path(s).is_absolute() else Path(s)) for s in scope_files]
    scope = {m: top_names(repo.tree(f)) for f in scope_files for m in repo.file_mods.get(f, ())}
    scope_mod_file = {m: f for f in scope_files for m in repo.file_mods.get(f, ())}
    # test files: under a config's testpaths, matching its python_files; owner = innermost rootdir
    tests = {}
    for c in sorted((c for c in repo.configs if "testpaths" in c), key=lambda c: len(c["rootdir"])):
        pre = "" if c["rootdir"] == "." else c["rootdir"] + "/"
        dirs = [os.path.normpath(pre + t).replace("\\", "/") + "/" for t in (c["testpaths"] or ["."])]
        for f in repo.py:
            if any(f.startswith(d) or d == "./" for d in dirs) and f.startswith(pre) \
                    and any(fnmatch.fnmatch(Path(f).name, g) for g in c["python_files"]):
                tests[f] = c["rootdir"]
    test_dirs = {os.path.normpath(("" if c["rootdir"] == "." else c["rootdir"] + "/") + t).replace("\\", "/") + "/"
                 for c in repo.configs if "testpaths" in c for t in c["testpaths"]}
    plugins_decl, conftest = {}, {}
    for f in repo.py:
        t = repo.tree(f)
        if t is not None and (pl := plugin_list(t)):
            plugins_decl[f] = pl
        if Path(f).name == "conftest.py" and t is not None:
            conftest[f] = sorted(fixtures_in(t))
    plugin_mods = sorted({m for v in plugins_decl.values() for m in v} | {p for c in repo.configs for p in c.get("plugins", [])})
    plugin_files = {m: sorted(repo.mod_files.get(m, ())) for m in plugin_mods}
    support = {f for f in repo.py if f not in tests and (any(f.startswith(d) for d in test_dirs) or
               any(re.search(r"(^|[_-])testing$", seg) for seg in f.split("/")[:-1]))} | {x for v in plugin_files.values() for x in v}
    # scope references
    per_file, unresolved, intra = defaultdict(lambda: defaultdict(dict)), defaultdict(set), defaultdict(set)
    for f in repo.py:
        t = repo.tree(f)
        if t is None:
            continue

        def add(m, name, kind, f=f):
            if f in scope_files:
                intra[f"{m}.{name}"].add(f)
                return
            if name != "<module>" and name not in scope[m] and kind in ("attribute", "patched"):
                unresolved[f"{m}.{name}"].add(f)
            ks = per_file[f][m].setdefault(name, [])
            ks.append(kind) if kind not in ks else None
        scan_refs(t, repo.file_mods.get(f, set()), scope, add)
    refs = {f: {m: dict(sorted(v.items())) for m, v in sorted(ms.items())} for f, ms in sorted(per_file.items())}
    # seeds by level
    edges = {f: {m for base, name, _ in imports_of(repo.tree(f), repo.file_mods.get(f, set()))
                 for m in (base, f"{base}.{name}" if name else base) if m in repo.mod_files}
             for f in repo.py if repo.tree(f) is not None}

    def closure(start: set[str], allowed: set[str]) -> set[str]:
        reach, changed = set(start), True
        while changed:
            new = {f for f in allowed - reach if any(x in reach for m in edges.get(f, ()) for x in repo.mod_files[m])}
            changed, reach = bool(new), reach | new
        return reach
    reach_support = closure({f for f in refs if f in support}, support)
    reach_all = closure(set(refs) | set(scope_files), set(repo.py))
    direct = sorted(f for f in refs if f in tests)
    via_support = sorted(f for f in tests if f not in refs and
                         any(x in reach_support for m in edges.get(f, ()) for x in repo.mod_files[m]))
    fix_reach, auto_reach = {}, []  # fixture name -> defining files, when its body uses a scope-reaching import
    hosts = sorted(set(conftest) | {x for v in plugin_files.values() for x in v})
    for f, (name, fn) in sorted({(d, kv) for h in hosts for d, kv in fixture_defs(repo, h)}, key=lambda x: (x[0], x[1][0])):
        bound = {local for base, imp, local in imports_of(repo.tree(f), repo.file_mods.get(f, set()))
                 if base in scope or f"{base}.{imp}" in scope or repo.mod_files.get(base, set()) & (reach_support | set(scope_files))}
        used = {n.id for n in ast.walk(fn) if isinstance(n, ast.Name)}
        if used & bound:
            if autouse(fn):
                auto_reach.append(f"{f}::{name}")
            else:
                fix_reach.setdefault(name, []).append(f)
    via_fixture = sorted(f for f in tests if f not in refs and f not in via_support and requested(repo.tree(f) or ast.Module(body=[], type_ignores=[])) & set(fix_reach))
    via_prod = sorted(f for f in tests if f in reach_all and f not in refs and f not in via_support and f not in via_fixture)
    plugin_set = {x for v in plugin_files.values() for x in v}  # store scan: conftests, plugins, plugins' in-tree imports
    store_files = sorted(set(conftest) | plugin_set | {x for f in plugin_set for m in edges.get(f, ()) for x in repo.mod_files[m]})
    gates = gate_commands(repo)
    ign = {x for g in gates["ignores"] for x in g["resolved"]}
    recommended = sorted((set(direct) | set(via_support) | set(via_fixture)) - ign)
    seeds = {"rule": "direct references, plus test files importing a test-support module whose support-only import "
                     "closure references the scope, plus test files requesting a conftest/plugin fixture whose body uses "
                     "such an import; minus files the gate scripts --ignore. via_production is an upper bound only.",
             "direct": direct, "via_support": via_support, "via_fixture": via_fixture, "via_production": via_prod,
             "ignored_by_gates": sorted(ign & (set(direct) | set(via_support) | set(via_fixture))),
             "recommended": recommended, "counts": {"direct": len(direct), "via_support": len(via_support),
             "via_fixture": len(via_fixture), "via_production": len(via_prod), "recommended": len(recommended)}}
    out = {"repo_root": str(repo.root), "scope": scope_files, "scope_modules": sorted(scope),
           "runner": runner(repo, gates), "fixtures": {"conftest": conftest, "pytest_plugins": plugins_decl,
           "plugin_modules": {m: {"files": fs, "fixtures": sorted({kv[0] for f in fs for _, kv in fixture_defs(repo, f)})}
                              for m, fs in plugin_files.items()},
           "scope_reaching_fixtures": {k: sorted(v) for k, v in sorted(fix_reach.items())},
           "scope_reaching_autouse_fixtures_not_counted": sorted(auto_reach)},
           "external_store": store_facts(repo, store_files),
           "scope_references": {"test_files": {f: v for f, v in refs.items() if f in tests},
                                "support_files": {f: v for f, v in refs.items() if f in support},
                                "production_files": sorted(f for f in refs if f not in tests and f not in support),
                                "unresolved": {k: sorted(v) for k, v in sorted(unresolved.items())},
                                "intra_scope": {k: sorted(v) for k, v in sorted(intra.items())},
                                "by_module": {m: sorted(f for f, ms in refs.items() if f in tests and m in ms) for m in sorted(scope)},
                                "parse_errors": sorted(repo.errors)},
           "seeds": seeds, "marker_expression": markers(repo, gates), "files_scanned": len(repo.py),
           "test_files_total": len(tests), "import_roots": repo.import_roots, "rule_version": RULE_VERSION}
    out["deselections"] = deselections(repo, fails, recommended, seeds_file) if fails is not None else None
    return out


def gate_commands(repo: Repo) -> dict:
    exprs, ignores, implied = [], [], set()
    for s in repo.scripts:
        lines = (repo.root / s).read_text(encoding="utf-8", errors="replace").split("\n")
        i = 0
        while i < len(lines):
            start, text = i, lines[i]
            while text.rstrip().endswith("\\") and i + 1 < len(lines):
                i += 1
                text = text.rstrip()[:-1] + " " + lines[i]
            i += 1
            for m in re.finditer(r"--ignore=(\S+?)[\s\"')]", text + " "):
                p = m.group(1)
                resolved = sorted(f for c in repo.configs if "testpaths" in c for f in
                                  [os.path.normpath(("" if c["rootdir"] == "." else c["rootdir"] + "/") + p).replace("\\", "/")]
                                  if (repo.root / f).is_file())
                ignores.append({"file": s, "line": start + 1, "path": p, "resolved": resolved})
            if "pytest" in text and (m := re.search(r"\s-m\s+(\"[^\"]*\"|'[^']*')", text)):
                own = re.findall(r"--ignore=(\S+?)[\s\"')]", text + " ")
                exprs.append({"file": s, "line": start + 1, "expression": m.group(1)[1:-1], "ignores": own,
                              "plugins": re.findall(r"\s-p\s+(\S+)", text)})
            if "pytest" in text:
                implied |= {p for flag, p in (("-n", "pytest-xdist"), ("--dist", "pytest-xdist"), ("--timeout", "pytest-timeout"),
                                              ("--cov", "pytest-cov")) if re.search(rf"\s{flag}[\s=]", text)}
    return {"expressions": exprs, "ignores": ignores, "implied_plugins": sorted(implied)}


def runner(repo: Repo, gates: dict) -> dict:
    cfgs = [c for c in repo.configs if "testpaths" in c]
    return {"name": "pytest" if cfgs else "unknown",
            "configs": [{k: c[k] for k in ("rootdir", "file", "testpaths", "pythonpath", "python_files", "markers", "addopts", "plugins")}
                        for c in sorted(cfgs, key=lambda c: c["rootdir"])],
            "plugins": sorted({p for c in cfgs for p in c["plugins"] + c["required_plugins"]} |
                              {p for g in gates["expressions"] for p in g["plugins"]}),
            "entry_points_pytest11": {c["file"]: c["entry_points"] for c in repo.configs if c.get("entry_points")},
            "implied_plugins": gates["implied_plugins"]}


def store_facts(repo: Repo, files: list[str]) -> dict:
    facts, hazards, site_mods = [], [], set()
    targets = [(f, repo.root / f) for f in files]
    for f in files:
        for base, name, _ in imports_of(repo.tree(f) or ast.Module(body=[], type_ignores=[]), repo.file_mods.get(f, set())):
            for m in (f"{base}.{name}" if name else base, base):
                if m and m not in repo.mod_files and not m.startswith(("pytest", "_pytest")) and \
                        re.search(r"(^|\.)(testing|tests?)(\.|$)|pg|postgres", m) and (sf := repo.site_file(m)):
                    site_mods.add((m, sf))
                    break
    targets += [(f"<site-packages>/{m.replace('.', '/')}.py", p) for m, p in sorted(site_mods)]
    for label, path in targets:
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        consts, wrappers = str_consts(tree), set()
        for n in ast.walk(tree):
            if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and any(
                    isinstance(c, ast.Call) and (dotted(c.func) or "").startswith("subprocess.") and
                    any(k.arg == "capture_output" and getattr(k.value, "value", False) or
                        k.arg in ("stdout", "stderr") and (dotted(k.value) or "").endswith("PIPE") for k in c.keywords)
                    for c in ast.walk(n)):
                wrappers.add(n.name)
        fx = fixtures_in(tree)
        for name, fn in fx.items():
            if FIXTURE_STORE.search(name):
                facts.append({"kind": "fixture", "detail": name, "file": label, "line": fn.lineno})
        for n in ast.walk(tree):
            line = getattr(n, "lineno", 0)
            if isinstance(n, (ast.Import, ast.ImportFrom)):
                for mod in ([a.name for a in n.names] if isinstance(n, ast.Import) else [n.module or ""]):
                    lib = next((s for s in STORE_LIBS if mod == s or mod.startswith(s + ".")), None)
                    if lib:
                        facts.append({"kind": "import", "detail": lib, "file": label, "line": line})
                    if mod.split(".")[0] in POSIX_MODULES:
                        hazards.append({"kind": "posix-module", "detail": mod, "file": label, "line": line})
            elif isinstance(n, ast.Call):
                d = dotted(n.func) or ""
                if d in ("os.getenv", "os.environ.get", "environ.get") and n.args:
                    a = n.args[0]
                    key = a.value if isinstance(a, ast.Constant) else consts.get(getattr(a, "id", ""))
                    if isinstance(key, str):
                        facts.append({"kind": "env", "detail": key, "file": label, "line": line, "store_related": bool(STORE_NAME.search(key))})
                if d.split(".")[-1] in wrappers or d.startswith("subprocess."):
                    args = [a.value for a in n.args if isinstance(a, ast.Constant) and isinstance(a.value, str)]
                    if "start" in args and any(x.endswith(dm) for x in args for dm in DAEMONS) and (d.split(".")[-1] in wrappers
                       or any(k.arg == "capture_output" for k in n.keywords)):
                        hazards.append({"kind": "captured-pipes-on-daemon-start", "detail": f"{d}({', '.join(args[:2])}, ... 'start')", "file": label, "line": line})
                    if any(x in DAEMONS or x in ("initdb", "createdb") for x in args):
                        facts.append({"kind": "command", "detail": next(x for x in args if x in DAEMONS or x in ("initdb", "createdb")), "file": label, "line": line})
            elif isinstance(n, ast.Subscript) and dotted(n.value) == "os.environ":
                s = n.slice
                key = s.value if isinstance(s, ast.Constant) else consts.get(getattr(s, "id", ""))
                if isinstance(key, str):
                    facts.append({"kind": "env", "detail": key, "file": label, "line": line, "store_related": bool(STORE_NAME.search(key))})
            elif isinstance(n, ast.Attribute) and dotted(n.value) == "os" and n.attr in POSIX_ATTRS:
                hazards.append({"kind": "posix-api", "detail": f"os.{n.attr}", "file": label, "line": line})
            elif isinstance(n, ast.Attribute) and n.attr == "symlink_to":
                hazards.append({"kind": "symlink", "detail": "Path.symlink_to", "file": label, "line": line})
            elif isinstance(n, ast.Constant) and isinstance(n.value, str) and re.search(r"=''|=\"\"", n.value):
                hazards.append({"kind": "quoted-empty-arg", "detail": n.value.strip()[:60], "file": label, "line": line})
    repo_files = [{"kind": "file", "detail": r, "file": r, "line": 0} for r in ("alembic.ini", "migrations", "compose.yaml",
                  "docker-compose.yml", "docker-compose.yaml") if (repo.root / r).exists()]
    uniq = lambda xs: sorted({json.dumps(x, sort_keys=True) for x in xs})  # noqa: E731
    return {"facts": [json.loads(x) for x in uniq(facts + repo_files)], "host_hazards": [json.loads(x) for x in uniq(hazards)],
            "site_packages_followed": sorted(m for m, _ in site_mods),
            "summary": {"embedded_postgres": any(x["detail"] == "pgembed" for x in facts),
                        "store_env": sorted({x["detail"] for x in facts if x["kind"] == "env" and x.get("store_related")}),
                        "migrations": any(x["detail"] in ("alembic", "alembic.ini") for x in facts + repo_files)}}


def markers(repo: Repo, gates: dict) -> dict:
    decl, order = {}, []
    for c in sorted((c for c in repo.configs if "markers" in c), key=lambda c: c["rootdir"]):
        for m, desc in c["markers"].items():
            decl.setdefault(m, []).append({"config": c["file"], "description": desc})
            order.append(m) if m not in order else None
    signals = {}
    for m in order:
        text = " ".join(d["description"] for d in decl[m])
        res = [k for k, rx in RESOURCES.items() if re.search(rx, text, re.I)]
        sig = (["opt-in"] if OPT_IN.search(text) else []) + [f"resource:{r}" for r in res] + (["self-skip"] if SELF_SKIP.search(text) else [])
        signals[m] = sig
    excluded = {m: [s for s in sig if s != "self-skip"] for m, sig in signals.items()
                if "opt-in" in sig or ({f"resource:{r}" for r in ("network", "broker", "hardware", "port")} & set(sig) and "self-skip" not in sig)}
    suggested = " and ".join(f"not {m}" for m in excluded)
    exprs = sorted({g["expression"] for g in gates["expressions"]})
    neg = lambda e: frozenset(re.findall(r"not\s+(\w+)", e))  # noqa: E731
    return {"rule": "exclude a marker whose description declares it opt-in, or names a hardware, network, broker or port "
                    "resource without saying the tests skip on their own; declaration order",
            "declared": decl, "signals": signals, "excluded": excluded, "suggested": suggested,
            "gate_expressions": gates["expressions"], "distinct_gate_expressions": exprs,
            "agrees_with_gates": {e: neg(e) == frozenset(excluded) for e in exprs}}


def load_fails(path: Path) -> dict[str, list[str]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return {"recorded": data}
    if "deselect" in data:
        return {"recorded": data["deselect"]}
    return {k: v for k, v in data.items() if isinstance(v, list)}


def defined(tree, chain: list[str], func: str) -> bool:
    body = tree.body
    for cls in chain:
        body = next((n.body for n in body if isinstance(n, ast.ClassDef) and n.name == cls), None)
        if body is None:
            return False
    return any(isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == func for n in body) or \
        any(isinstance(n, ast.Assign) and any(getattr(t, "id", "") == func for t in n.targets) for n in body)


def resolve_node(repo: Repo, nid: str) -> tuple[str, list[str]]:
    """('node'|'collection', candidate node ids or files) for a path or junit-dotted node id."""
    head, _, tail = nid.partition("::")
    if head.endswith(".py"):
        return "node", [nid] if (repo.root / head).is_file() else []
    collection = head == ""
    dotted_mod, func = (tail, "") if collection else (head, tail)
    parts, base = dotted_mod.split("."), func.split("[")[0]
    out = []
    for c in sorted({c["rootdir"] for c in repo.configs if "testpaths" in c}):
        pre = "" if c == "." else c + "/"
        for k in range(len(parts), 0, -1):
            f = pre + "/".join(parts[:k]) + ".py"
            if (repo.root / f).is_file():
                t, chain = repo.tree(f), parts[k:]
                if collection:
                    out.append(f)
                elif t is not None and defined(t, chain, base):
                    out.append("::".join([f, *chain, func]))
                break
    return ("collection" if collection else "node"), out


def deselections(repo: Repo, fails: dict, seeds: list[str], seeds_file: list[str] | None) -> dict:
    resolved, coll, unres, amb = {}, [], [], []
    for cause, ids in fails.items():
        resolved[cause] = []
        for nid in ids:
            kind, cands = resolve_node(repo, nid)
            if len(cands) != 1:
                (amb if cands else unres).append(nid if not cands else {"id": nid, "candidates": cands})
            elif kind == "collection":
                coll.append(cands[0])
            else:
                resolved[cause].append(cands[0])
    within = lambda ss: {c: sorted(n for n in v if n.split("::")[0] in set(ss)) for c, v in resolved.items()}  # noqa: E731
    total = sum(len(v) for v in fails.values()) - len(coll)
    out = {"total": total, "resolved": sum(len(v) for v in resolved.values()), "collection_errors": sorted(coll),
           "unresolved": unres, "ambiguous": amb, "by_cause": {c: len(v) for c, v in resolved.items()},
           "resolved_ids": {c: sorted(v) for c, v in resolved.items()}, "applicable": within(seeds)}
    out["applicable_count"] = sum(len(v) for v in out["applicable"].values())
    if seeds_file is not None:
        out["applicable_to_supplied_seeds"] = within(seeds_file)
        out["applicable_to_supplied_seeds_count"] = sum(len(v) for v in out["applicable_to_supplied_seeds"].values())
        out["supplied_seed_files"] = len(seeds_file)
    return out


def render(d: dict) -> str:
    r, s, m, x, st = d["runner"], d["seeds"], d["marker_expression"], d["deselections"], d["external_store"]
    lines = ["# Test infrastructure", "", f"- Repository: `{d['repo_root']}`; {d['files_scanned']} Python files, "
             f"{d['test_files_total']} test files.", f"- Scope: {', '.join(f'`{f}`' for f in d['scope'])}.", "",
             "## Runner", "", f"- Runner: {r['name']}; plugins named: {', '.join(r['plugins']) or 'none'}; implied by gate "
             f"flags: {', '.join(r['implied_plugins']) or 'none'}.", "", "| rootdir | config | testpaths | markers |", "|---|---|---|---|"]
    lines += [f"| `{c['rootdir']}` | `{c['file']}` | {', '.join(c['testpaths'])} | {', '.join(c['markers'])} |" for c in r["configs"]]
    f = d["fixtures"]
    lines += ["", "## Fixtures and plugins", "", f"- conftest files: {len(f['conftest'])}; pytest_plugins declared in "
              f"{len(f['pytest_plugins'])} files: {', '.join(sorted({p for v in f['pytest_plugins'].values() for p in v}))}."]
    lines += [f"- Plugin `{k}` ({', '.join(v['files']) or 'not in tree'}): fixtures {', '.join(v['fixtures']) or 'none'}."
              for k, v in f["plugin_modules"].items()]
    lines += ["", "## External store", "", f"- Embedded PostgreSQL: {st['summary']['embedded_postgres']}; migrations: "
              f"{st['summary']['migrations']}; store environment variables: {', '.join(st['summary']['store_env']) or 'none'}."]
    lines += [f"- {h['kind']}: `{h['detail']}` at `{h['file']}:{h['line']}`" for h in st["host_hazards"]]
    lines += ["", "## Scope references and seeds", "", f"- Test files referencing the scope directly: {s['counts']['direct']}; "
              f"via test support: {s['counts']['via_support']}; via fixtures: {s['counts']['via_fixture']}; recommended: "
              f"{s['counts']['recommended']}; production-import upper bound adds {s['counts']['via_production']}.",
              f"- Unresolved references: {len(d['scope_references']['unresolved'])}; parse errors: {len(d['scope_references']['parse_errors'])}.",
              "", "## Marker expression", "", f"- Suggested: `{m['suggested']}` ({m['rule']}).",
              f"- Gate expressions: {', '.join(f'`{e}`' for e in m['distinct_gate_expressions']) or 'none'}; "
              f"agreement: {m['agrees_with_gates']}."]
    if x:
        lines += ["", "## Baseline deselections", "", f"- Recorded node failures: {x['total']} (resolved {x['resolved']}, "
                  f"unresolved {len(x['unresolved'])}, ambiguous {len(x['ambiguous'])}); collection errors: {len(x['collection_errors'])}.",
                  f"- Applicable to the recommended seeds: {x['applicable_count']}; by cause: "
                  f"{ {c: len(v) for c, v in x['applicable'].items()} }."]
        if "applicable_to_supplied_seeds" in x:
            lines.append(f"- Applicable to the {x['supplied_seed_files']} supplied seed files: {x['applicable_to_supplied_seeds_count']}.")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("repo_root", type=Path)
    ap.add_argument("--scope", nargs="+", required=True, help="scope source files (relative to the root)")
    ap.add_argument("--baseline-failures", type=Path, help="recorded failures: list, {cause: [ids]} or {'deselect': [ids]}")
    ap.add_argument("--seeds-file", type=Path, help="a recorded seed list (one file per line) to apply deselections to as well")
    ap.add_argument("--site-packages", type=Path, help="follow store-related third-party modules here (static read)")
    ap.add_argument("--out", type=Path, default=Path.cwd())
    ap.add_argument("--stem", default="testinfra", help="output file stem (default testinfra: .json and .md)")
    a = ap.parse_args(argv)
    fails = load_fails(a.baseline_failures) if a.baseline_failures else None
    seeds = [x.strip() for x in a.seeds_file.read_text(encoding="utf-8").splitlines() if x.strip()] if a.seeds_file else None
    d = detect(a.repo_root, a.scope, fails, seeds, a.site_packages)
    a.out.mkdir(parents=True, exist_ok=True)
    (a.out / f"{a.stem}.json").write_text(json.dumps(d, indent=1, sort_keys=True), encoding="utf-8")
    (a.out / f"{a.stem}.md").write_text(render(d), encoding="utf-8")
    print(f"testinfra: {d['test_files_total']} test files; seeds {d['seeds']['counts']}; marker `{d['marker_expression']['suggested']}`")
    return 0


if __name__ == "__main__":
    sys.exit(main())
