"""Contract-B public-surface derivation and candidate verification (SurfaceDeriver).

All Python AST knowledge lives behind this seam: surface derivation, syntax
verification, and candidate verification. Controllers and the regenerator
never import ``ast`` directly; adding a target language means adding a sibling
extractor module, not rewriting callers.

The derivation generalizes the frozen E3/E4 apparatus (``surface.py`` /
``surface4.py``, scanned uniformly by ``s9_surface_contracts.py`` for R26
amendment 1): Contract A scans the tree for imports, module-alias attribute
reads and monkeypatch targets of the scope modules; Contract B additionally
scans recorder (oracle) files with the same walker, so symbols imported only
by the recorder join the contract.
"""
from __future__ import annotations

import ast
from pathlib import Path
from typing import Sequence

__all__ = ["EXTRACTOR_VERSION", "check_syntax", "derive_surface", "verify_candidate_surface"]

EXTRACTOR_VERSION = "python-ast-v1"

_SCAN_DIRS = ("src", "packages", "tests")
_TEST_MARKERS = ("tests/", "/tests/", "facdrone_testing")
_SUPPORTED_LANGUAGES = ("python",)


def _module_name(rel: str) -> str:
    """Map a scope file path to its dotted module name (frozen convention)."""
    body = rel[:-3] if rel.endswith(".py") else rel
    first, _, rest = body.partition("/")
    if first in _SCAN_DIRS and rest:
        body = rest
    return body.replace("/", ".")


def check_syntax(
    cand_root: Path,
    scope_files: Sequence[str],
    language: str = "python",
) -> list[str]:
    """Return line-anchored syntax errors for ``scope_files`` under ``cand_root``.

    Never imports or executes candidate code. An empty list means every file
    parsed cleanly. Languages without a sibling extractor are reported as an
    explicit unsupported-language error.
    """
    if language not in _SUPPORTED_LANGUAGES:
        return [f"check_syntax: unsupported language {language!r}"]
    errors: list[str] = []
    for rel in scope_files:
        path = cand_root / rel
        if not path.is_file():
            errors.append(f"{rel}: file missing from candidate tree")
            continue
        try:
            ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            errors.append(f"{rel}:{exc.lineno}:{exc.offset}: {exc.msg}")
        except (OSError, UnicodeDecodeError) as exc:
            errors.append(f"{rel}: unreadable: {exc}")
    return errors


def _kinds(root: Path, rel: str) -> dict[str, str]:
    """Top-level callable/constant/re-export kinds of one scope module."""
    out: dict[str, str] = {}
    for node in ast.parse((root / rel).read_text(encoding="utf-8")).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            out[node.name] = "class" if isinstance(node, ast.ClassDef) else "function"
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            targets = getattr(node, "targets", [getattr(node, "target", None)])
            out.update({t.id: "constant" for t in targets if isinstance(t, ast.Name)})
        elif isinstance(node, ast.ImportFrom):
            out.update({a.asname or a.name: f"reexport:{node.module}" for a in node.names})
    return out


def _scan(tree: ast.Module, modules: dict[str, str], add) -> None:
    """Walk one parsed file, reporting imports, alias attribute reads, setattr targets."""
    alias: dict[str, str] = {}
    for node in ast.walk(tree):
        if not isinstance(node, (ast.ImportFrom, ast.Import)):
            continue
        for alias_node in node.names:
            if isinstance(node, ast.ImportFrom) and node.module in modules:
                add(node.module, alias_node.name, "import")
            elif (
                candidate := alias_node.name
                if isinstance(node, ast.Import)
                else f"{node.module}.{alias_node.name}"
            ) in modules:
                add(candidate, "<module>", "module")
                alias[alias_node.asname or alias_node.name] = candidate
    for node in ast.walk(tree):
        if (
            isinstance(node, ast.Attribute)
            and isinstance(node.value, ast.Name)
            and node.value.id in alias
        ):
            add(alias[node.value.id], node.attr, "attribute")
        elif (
            isinstance(node, ast.Call)
            and getattr(node.func, "attr", getattr(node.func, "id", "")) == "setattr"
            and len(node.args) > 1
        ):
            first, second = node.args[:2]
            if isinstance(first, ast.Name) and first.id in alias and isinstance(second, ast.Constant):
                add(alias[first.id], second.value, "patched")
            elif isinstance(first, ast.Constant) and str(first.value).rpartition(".")[0] in modules:
                dotted = str(first.value).rpartition(".")
                add(dotted[0], dotted[2], "patched")


def derive_surface(
    root: Path,
    scope_files: Sequence[str],
    recorder_files: Sequence[str] = (),
    scan_dirs: Sequence[str] = _SCAN_DIRS,
) -> dict:
    """Derive the Contract-B public surface report for ``scope_files``.

    Contract B: symbols imported or used only by the recorder oracle are part
    of the contract (tagged ``recorder``). Attribute reads of names a scope
    module lacks are recorded under ``unresolved`` — never silently absent
    (Req 2.5); imports of names the module does not define keep kind
    ``missing``.
    """
    scope = list(scope_files)
    modules = {_module_name(rel): rel for rel in scope}
    kind = {m: _kinds(root, rel) for m, rel in modules.items()}
    scope_set = set(scope)
    recorder_set = set(recorder_files)
    files = [
        p
        for d in scan_dirs
        for p in sorted((root / d).rglob("*.py"))
    ]
    report: dict = {
        "extractor_version": EXTRACTOR_VERSION,
        "root": str(root),
        "scope_files": scope,
        "recorder_files": sorted(recorder_set),
        "files_scanned": len(files),
        "modules": {m: {"file": rel, "names": {}, "importers": {}} for m, rel in modules.items()},
        "unresolved": {m: {} for m in modules},
        "intra_set": {m: {} for m in modules},
    }

    def make_add(rel: str, tag: str):
        def add(module: str, name: str, via: str) -> None:
            resolved = "module" if name == "<module>" else kind[module].get(name)
            if rel in scope_set:
                report["intra_set"][module].setdefault(name, []).append(rel)
            elif resolved is None and via == "attribute":
                entries = report["unresolved"][module].setdefault(name, [])
                if rel not in entries:
                    entries.append(rel)
            else:
                entry = report["modules"][module]["names"].setdefault(
                    name, {"kind": resolved or "missing", "tags": [], "via": [], "importers": []}
                )
                for key, value in (("tags", tag), ("via", via), ("importers", rel)):
                    if value not in entry[key]:
                        entry[key].append(value)
                report["modules"][module]["importers"][rel] = tag

        return add

    def tag_for(rel: str) -> str:
        if rel in recorder_set:
            return "recorder"
        if rel.startswith("tests/") or "/tests/" in rel or "facdrone_testing" in rel:
            return "test"
        return "prod"

    for path in [*files, *(root / rel for rel in sorted(recorder_set))]:
        rel = path.relative_to(root).as_posix()
        add = make_add(rel, tag_for(rel))
        _scan(ast.parse(path.read_text(encoding="utf-8")), modules, add)
    return report


def verify_candidate_surface(
    cand_root: Path,
    surface_report: dict,
    extra_patched: Sequence[str] = (),
) -> dict:
    """Verify a candidate tree against the Contract-B surface report.

    Reports missing symbols, kind mismatches, and local rebinding of
    ``patched`` module globals; never imports or executes candidate code.
    (Implemented by task 2.2.)
    """
    raise NotImplementedError("task 2.2")
