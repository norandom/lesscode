"""Three-reading scope measures, structure validation, and edit-scope checks (ScopeMeasurer).

Counts as-written, separated-form (rule v1), and canonical (pinned ruff)
readings for a trial scope, validates declared pure/shell structure against a
pluggable ``EffectProfile``, and enforces edit scope by hash identity.
Generalizes the frozen E3 apparatus (``measures.py``, ``pureshare.py``) with
explicit ``scope_files`` instead of a module-global ``SET``.
"""
from __future__ import annotations

import ast
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Sequence

__all__ = [
    "MEASURE_VERSION",
    "DEFAULT_EFFECT_PROFILE",
    "EffectProfile",
    "check_edit_scope",
    "compute_measure_gaps",
    "measure_all",
    "measure_canonical",
    "measure_pure_share",
    "measure_separated",
    "measure_written",
]

MEASURE_VERSION = "scope-measures-v1"


@dataclass(frozen=True)
class EffectProfile:
    """Language/domain effect vocabulary for structure validation.

    ``write_pattern`` matches write-site call text, ``bad_param_pattern``
    matches forbidden parameter names in pure functions, and
    ``sql_attr_names`` are attribute names whose calls are forbidden in pure
    functions. Separated from the walker so non-SQLAlchemy domains plug in
    without touching measurement logic.
    """

    name: str
    write_pattern: str
    bad_param_pattern: str
    sql_attr_names: frozenset[str]
    version: str = field(default="effect-v1")


DEFAULT_EFFECT_PROFILE = EffectProfile(
    name="sqlalchemy",
    write_pattern=r"\.(execute|add|add_all|commit|flush|delete|merge)\(|\b(insert|update|delete)\(",
    bad_param_pattern=r"^(engine|conn|connection|session|bind)$|_(engine|conn|connection|session)$",
    sql_attr_names=frozenset(
        {"execute", "begin", "connect", "commit", "add", "flush", "merge", "delete"}
    ),
)


def _defs(node: ast.AST, prefix: str = "") -> list[tuple[str, ast.AST]]:
    """(qualname, def node) pairs; nested defs and methods included (frozen walker)."""
    out: list[tuple[str, ast.AST]] = []
    for child in ast.iter_child_nodes(node):
        qualname = (
            f"{prefix}{child.name}"
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
            else None
        )
        if qualname and not isinstance(child, ast.ClassDef):
            out.append((qualname, child))
        out.extend(_defs(child, f"{qualname}." if qualname else prefix))
    return out


def measure_written(
    root: Path,
    scope_files: Sequence[str],
    structure: dict | None = None,
    effect_profile: EffectProfile = DEFAULT_EFFECT_PROFILE,
) -> dict:
    """Count lines/stmts/funcs/qualnames/writes/edges/module_hops; validate structure.

    Output shape follows the frozen E3 measurer (files keyed by basename,
    ``total``, sorted ``edges``, ``write_sites`` with innermost enclosing def,
    optional ``structure`` block). Every write-site match, forbidden pure
    parameter, and forbidden attribute call comes from ``effect_profile``.
    """
    write_re = re.compile(effect_profile.write_pattern)
    bad_re = re.compile(effect_profile.bad_param_pattern)
    src = {Path(f).name: (root / f).read_text(encoding="utf-8") for f in scope_files}
    trees = {f: ast.parse(s) for f, s in src.items()}
    defmap = {f: _defs(t) for f, t in trees.items()}
    names: dict[str, set[str]] = {}
    for f, defs in defmap.items():
        for _, node in defs:
            names.setdefault(node.name, set()).add(f)
    edges: set[tuple[str, str, str, str]] = set()
    sites: list[dict] = []
    for f, text in src.items():
        for qualname, node in defmap[f]:
            for call in (c for c in ast.walk(node) if isinstance(c, ast.Call)):
                callee = getattr(call.func, "id", getattr(call.func, "attr", None))
                if callee in names and callee != node.name:
                    edges |= {(f, node.name, g, callee) for g in names[callee]}
        for match in write_re.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            inner = [(n.lineno, q) for q, n in defmap[f] if n.lineno <= line <= n.end_lineno]
            sites.append(
                {
                    "file": f,
                    "line": line,
                    "match": match.group(0),
                    "in": f"{f}::{max(inner)[1]}" if inner else None,
                }
            )
    files: dict[str, dict] = {}
    for f, text in src.items():
        files[f] = {
            "lines": len(text.splitlines()),
            "stmts": sum(isinstance(x, ast.stmt) for x in ast.walk(trees[f])),
            "funcs": len({n.name for _, n in defmap[f]}),
            "qualnames": len(defmap[f]),
            "writes": sum(s["file"] == f for s in sites),
        }
    total = {k: sum(v[k] for v in files.values()) for k in ("lines", "stmts", "funcs", "qualnames", "writes")}
    total |= {
        "edges": len(edges),
        "module_hops": len({(a, c) for a, _, c, _ in edges if a != c}),
    }
    out: dict = {
        "measure_version": MEASURE_VERSION,
        "effect_profile": effect_profile.name,
        "root": str(root),
        "files": files,
        "total": total,
        "edges": sorted(map(list, edges)),
        "write_sites": sites,
    }
    if structure is not None:
        out["structure"] = _check_structure(src, defmap, sites, structure, effect_profile)
    return out


def _check_structure(
    src: dict[str, str],
    defmap: dict[str, list[tuple[str, ast.AST]]],
    sites: list[dict],
    structure: dict,
    effect_profile: EffectProfile,
) -> dict:
    bad_re = re.compile(effect_profile.bad_param_pattern)
    nodes = {f"{f}::{q}": n for f, defs in defmap.items() for q, n in defs}
    pure: dict[str, dict] = {}
    shell = set(structure.get("shell", []))
    for key in structure.get("pure", []):
        node = nodes.get(key)
        args = node.args if node else None
        why = (
            ["not defined"]
            if node is None
            else [
                f"param {p.arg}"
                for p in (
                    args.posonlyargs
                    + args.args
                    + args.kwonlyargs
                    + [x for x in (args.vararg, args.kwarg) if x]
                )
                if bad_re.search(p.arg)
            ]
            + [
                f"call .{call.func.attr} line {call.lineno}"
                for call in ast.walk(node)
                if isinstance(call, ast.Call)
                and isinstance(call.func, ast.Attribute)
                and call.func.attr in effect_profile.sql_attr_names
            ]
        )
        pure[key] = {"pass": not why, "why": why}
    checked_sites = [dict(site, **{"pass": site["in"] in shell}) for site in sites]
    declared = set(structure.get("pure", [])) | shell
    return {
        "pure": pure,
        "write_sites": checked_sites,
        "undeclared": sorted(set(nodes) - declared),
        "declared_unknown": sorted(declared - set(nodes)),
        "pure_share": round(len(pure) / max(1, len(nodes)), 3),
        "pass": all(v["pass"] for v in pure.values()) and all(s["pass"] for s in checked_sites),
    }


def measure_pure_share(root: Path, scope_files: Sequence[str], structure: dict) -> dict:
    """Attribute every ``ast.stmt`` to its innermost enclosing def (frozen pureshare)."""
    pure, shell = set(structure["pure"]), set(structure["shell"])
    totals = {"pure": 0, "shell": 0, "undeclared": 0, "outside_defs": 0}
    per_function: dict[str, int] = {}
    for rel in scope_files:
        name = Path(rel).name
        tree = ast.parse((root / rel).read_text(encoding="utf-8"))
        defs = _defs(tree)
        for stmt in (n for n in ast.walk(tree) if isinstance(n, ast.stmt)):
            inner = [(n.lineno, q) for q, n in defs if n.lineno <= stmt.lineno <= n.end_lineno]
            key = f"{name}::{max(inner)[1]}" if inner else None
            cls = (
                "outside_defs"
                if key is None
                else "pure"
                if key in pure
                else "shell"
                if key in shell
                else "undeclared"
            )
            totals[cls] += 1
            if key:
                per_function[key] = per_function.get(key, 0) + 1
    in_defs = totals["pure"] + totals["shell"] + totals["undeclared"]
    all_stmts = sum(totals.values())
    return {
        "measure_version": MEASURE_VERSION,
        "root": str(root),
        "stmts": totals,
        "stmts_in_defs": in_defs,
        "stmts_all": all_stmts,
        "pure_share_by_stmts_in_defs": round(totals["pure"] / max(1, in_defs), 3),
        "pure_share_by_stmts_all": round(totals["pure"] / max(1, all_stmts), 3),
        "pure_share_by_funcs": round(len(pure) / max(1, len(per_function)), 3),
        "per_function": per_function,
    }


def measure_separated(root: Path, scope_files: Sequence[str]) -> dict:
    """Compute separated-form statement counts under rule v1 (rules 1-9)."""
    raise NotImplementedError("task 3.2")


def measure_canonical(root: Path, scope_files: Sequence[str], fmt_dir: Path) -> dict | None:
    """Format isolated copies with pinned ruff and return canonical counts (None if unavailable)."""
    raise NotImplementedError("task 3.2")


def check_edit_scope(
    original: Path,
    candidate: Path,
    scope_files: Sequence[str],
    dirs: Sequence[str] = ("src", "tests", "packages"),
    mentions_files: Sequence[Path] = (),
) -> dict:
    """Verify SHA-256 identity outside ``scope_files`` and scan mentions for out-of-scope tokens."""
    raise NotImplementedError("task 3.3")


def compute_measure_gaps(written: dict, canonical: dict | None, separated: dict, targets: dict) -> dict[str, dict]:
    """Return unmet target deltas; empty dict iff all preregistered targets pass."""
    raise NotImplementedError("task 3.3")


def measure_all(root: Path, scope_files: Sequence[str], fmt_dir: Path | None = None) -> dict:
    """One call returning written, separated, canonical (None if unavailable), and gaps."""
    raise NotImplementedError("task 3.3")
