"""Three-reading scope measures, structure validation, and edit-scope checks (ScopeMeasurer).

Counts as-written, separated-form (rule v1), and canonical (pinned ruff)
readings for a trial scope, validates declared pure/shell structure against a
pluggable ``EffectProfile``, and enforces edit scope by hash identity.
Generalizes the frozen E3 apparatus (``measures.py``, ``pureshare.py``) with
explicit ``scope_files`` instead of a module-global ``SET``.
"""
from __future__ import annotations

import ast
import hashlib
import re
import shutil
import subprocess
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

    def _norm_key(key: str) -> str:
        # Declarations may qualify the file with its scope path ("src/cart.py::f");
        # that names the same (file, qualname) pair as the frozen basename form
        # ("cart.py::f"), so normalize before lookup (G1-R live finding, 2026-11-08).
        head, sep, qual = key.rpartition("::")
        return f"{Path(head).name}::{qual}" if sep else key

    declared_pure = sorted({_norm_key(k) for k in structure.get("pure", [])})
    shell = {_norm_key(k) for k in structure.get("shell", [])}
    pure: dict[str, dict] = {}
    for key in declared_pure:
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
    """Separated-form statement counts under rule v1 (rules 1-9, frozen text).

    Every ``ast.stmt`` counts 1 (rule 1); rules 2-8 add extras so that
    ``separated - sum(extra) == written`` holds exactly. Rule 9 lists what is
    deliberately not unpacked. Keyed per file (basename) with totals.
    """
    files = {
        Path(rel).name: _separated(ast.parse((root / rel).read_text(encoding="utf-8")))
        for rel in scope_files
    }
    return {
        "measure_version": MEASURE_VERSION,
        "files": files,
        "total": {
            "stmts": sum(v["stmts"] for v in files.values()),
            "extra": {k: sum(v["extra"][k] for v in files.values()) for k in SEPARATED_KEYS},
        },
        "rule": SEPARATED_RULE,
    }


SEPARATED_KEYS = (
    "import",
    "from_import",
    "tuple_assign",
    "multi_target",
    "chained_compare",
    "del",
    "global_nonlocal",
    "with_items",
)

SEPARATED_RULE = {
    "version": "1",
    "text": (
        "1. Every ast.stmt node counts 1, exactly as the frozen `stmts`; rules 2-8 only add extras to it.\n"
        "2. `import a, b, c` counts one statement per name (extra len(names) - 1).\n"
        "3. `from m import a, b` counts one statement per name (extra len(names) - 1); `from m import *` counts 1.\n"
        "4. An assignment with one target that is a tuple or list, and a tuple or list value of the same length, "
        "neither holding a starred element, counts one statement per paired leaf, recursively.\n"
        "5. An assignment with several targets `a = b = v` counts len(targets); rule 4 is then not applied to its targets.\n"
        "6. A comparison with k comparators counts k, wherever it occurs.\n"
        "7. `del a, b` counts len(targets); `global a, b` and `nonlocal a, b` count len(names).\n"
        "8. A `with`/`async with` statement with n items counts n.\n"
        "9. Not unpacked (count only as rule 1): augmented and annotated assignments; a tuple target with a "
        "non-literal or starred value; boolean operators; conditional expressions; comprehensions; lambdas; "
        "walrus; for-loop targets; multi-type `except`; tuple returns, yields and call arguments; decorators; "
        "semicolon-joined lines."
    ),
}


def _leaves(target: ast.AST, value: ast.AST) -> int:  # rule 4
    seq = (ast.Tuple, ast.List)
    if (
        isinstance(target, seq)
        and isinstance(value, seq)
        and len(target.elts) == len(value.elts)  # type: ignore[attr-defined]
        and not any(isinstance(e, ast.Starred) for e in target.elts)  # type: ignore[attr-defined]
        and not any(isinstance(e, ast.Starred) for e in value.elts)  # type: ignore[attr-defined]
    ):
        return sum(_leaves(a, b) for a, b in zip(target.elts, value.elts))  # type: ignore[attr-defined]
    return 1


def _separated(tree: ast.AST) -> dict:
    extras: dict[str, int] = dict.fromkeys(SEPARATED_KEYS, 0)
    base = 0
    for node in ast.walk(tree):
        base += isinstance(node, ast.stmt)
        if isinstance(node, ast.Import):
            extras["import"] += len(node.names) - 1
        elif isinstance(node, ast.ImportFrom):
            extras["from_import"] += len(node.names) - 1
        elif isinstance(node, ast.Assign):
            if len(node.targets) > 1:
                extras["multi_target"] += len(node.targets) - 1
            else:
                extras["tuple_assign"] += _leaves(node.targets[0], node.value) - 1
        elif isinstance(node, ast.Compare):
            extras["chained_compare"] += len(node.comparators) - 1
        elif isinstance(node, ast.Delete):
            extras["del"] += len(node.targets) - 1
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            extras["global_nonlocal"] += len(node.names) - 1
        elif isinstance(node, (ast.With, ast.AsyncWith)):
            extras["with_items"] += len(node.items) - 1
    return {"stmts": base + sum(extras.values()), "extra": extras}


FMT = {
    "name": "ruff format",
    "version": "0.17.3",
    "settings": {
        "isolated": True,
        "line-length": 100,
        "target-version": "py313",
        "other": "ruff defaults (double quotes, 4-space indent, magic trailing comma respected)",
    },
}
RUFF = [shutil.which("uv") or "uv.exe", "tool", "run", "--offline", "--python", "3.13", f"ruff=={FMT['version']}"]
RUFF_ARGS = ["format", "--isolated", "--line-length", "100", "--target-version", "py313", "--no-cache"]


def measure_canonical(root: Path, scope_files: Sequence[str], fmt_dir: Path) -> dict | None:
    """Format isolated copies with the pinned ruff and return canonical counts.

    Returns ``None`` (never zeros) when the formatter is unavailable, fails,
    or reports a version other than the pinned one (Req 6.3).
    """
    shutil.rmtree(fmt_dir, ignore_errors=True)
    for rel in scope_files:
        (fmt_dir / rel).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / rel, fmt_dir / rel)
    try:
        version = subprocess.run(
            RUFF + ["--version"], capture_output=True, text=True, check=True
        ).stdout.split()
        if version[-1] != FMT["version"]:
            return None
        subprocess.run(
            RUFF + RUFF_ARGS + list(scope_files), cwd=fmt_dir, check=True, capture_output=True
        )
    except (OSError, subprocess.SubprocessError):
        return None
    measured = measure_written(fmt_dir, scope_files)
    return {
        "formatter": dict(FMT),
        "files": {
            f: {k: v[k] for k in ("lines", "stmts")} for f, v in measured["files"].items()
        },
        "total": {k: measured["total"][k] for k in ("lines", "stmts")},
    }


def check_edit_scope(
    original: Path,
    candidate: Path,
    scope_files: Sequence[str],
    dirs: Sequence[str] = ("src", "tests", "packages"),
    mentions_files: Sequence[Path] = (),
) -> dict:
    """Verify SHA-256 identity outside ``scope_files`` and scan mentions for out-of-scope tokens."""
    raise NotImplementedError("task 3.3")


def check_edit_scope(
    original: Path,
    candidate: Path,
    scope_files: Sequence[str],
    dirs: Sequence[str] = ("src", "tests", "packages"),
    mentions_files: Sequence[Path] = (),
    source_extensions: Sequence[str] = (".py",),
    ignore_dirs: Sequence[str] = ("node_modules", "target"),
) -> dict:
    """Verify SHA-256 identity outside ``scope_files``; scan mentions for out-of-scope tokens.

    Generalizes the frozen edit-scope checker: ``source_extensions`` selects
    which file suffixes count as source tokens in ``mentions_files`` (so a
    Rust or TypeScript scope needs no rewrite), and ``ignore_dirs`` adds
    build/dependency trees the hash walk skips.
    """
    skip_dirs = {"__pycache__", ".pytest_cache", ".git"} | set(ignore_dirs)
    skip_suffix = {".pyc"}

    def digest(path: Path) -> str:
        return hashlib.sha256(path.read_bytes()).hexdigest()

    def walk(root: Path) -> dict[str, str]:
        out = {
            p.name: digest(p)
            for p in root.iterdir()
            if p.is_file() and p.suffix not in skip_suffix
        }
        for d in dirs:
            base = root / d
            if not base.is_dir():
                continue
            for p in sorted(base.rglob("*")):
                rel = p.relative_to(root)
                if (
                    p.is_file()
                    and p.suffix not in skip_suffix
                    and not skip_dirs & set(rel.parts)
                ):
                    out[rel.as_posix()] = digest(p)
        return out

    a, b = walk(original), walk(candidate)
    scope = sorted(scope_files)
    common = a.keys() & b.keys()
    differing = sorted(k for k in common if a[k] != b[k])
    only_original = sorted(a.keys() - b.keys())
    only_candidate = sorted(b.keys() - a.keys())
    outside = sorted(set(differing) - set(scope))
    edit_scope = "pass" if not only_original and not only_candidate and not outside else "fail"
    report: dict = {
        "original": str(original),
        "candidate": str(candidate),
        "dirs": list(dirs),
        "skip_dirs": sorted(skip_dirs),
        "skip_suffix": sorted(skip_suffix),
        "source_extensions": list(source_extensions),
        "files_original": len(a),
        "files_candidate": len(b),
        "files_compared": len(common),
        "only_in_original": only_original,
        "only_in_candidate": only_candidate,
        "differing": differing,
        "scope": scope,
        "differing_outside_scope": outside,
        "scope_unchanged": sorted(set(scope) - set(differing)),
        "scope_missing": sorted(set(scope) - common),
        "original_hashes": {k: a.get(k) for k in scope},
        "candidate_hashes": {k: b.get(k) for k in scope},
        "edit_scope": edit_scope,
        "verdict": "pass" if edit_scope == "pass" and differing == scope else "fail",
    }
    if mentions_files:
        report["mentions"] = {
            "files": [str(f) for f in mentions_files],
            "outside_scope": _mention_outside(mentions_files, scope, source_extensions),
        }
    return report


def _mention_outside(
    mentions_files: Sequence[Path], scope: Sequence[str], source_extensions: Sequence[str]
) -> list[str]:
    token_re = re.compile(
        r"[\w./\\-]*\w(?:"
        + "|".join(re.escape(ext) for ext in source_extensions)
        + r")\b"
    )
    allowed = {Path(s).name for s in scope} | set(scope)
    found = {
        token
        for f in mentions_files
        for token in token_re.findall(Path(f).read_text(encoding="utf-8", errors="replace"))
    }
    return sorted(t for t in found if t not in allowed and Path(t).name not in allowed)


def compute_measure_gaps(
    written: dict,
    canonical: dict | None,
    separated: dict,
    targets: dict,
) -> dict[str, dict]:
    """Return unmet target deltas; empty dict iff every preregistered target passes.

    ``*_max`` targets pass at exactly the target value. Canonical targets with
    a ``None`` reading stay unmet and are marked unavailable — never zero
    (Req 6.3).
    """
    totals = {
        "written": written["total"],
        "separated": separated["total"],
        "canonical": None if canonical is None else canonical["total"],
    }
    gaps: dict[str, dict] = {}
    for key, bound in targets.items():
        if not key.endswith("_max"):
            gaps[key] = {"target": bound, "error": "unknown target kind (only *_max supported)"}
            continue
        stem = key[: -len("_max")]
        source = next((s for s in ("written", "separated", "canonical") if stem.startswith(f"{s}_")), None)
        if source is not None:
            metric = stem[len(source) + 1 :]
        else:
            source, metric = "written", stem  # bare metrics (stmts_max, edges_max, ...) read as written
        if not source or not metric:
            gaps[key] = {"target": bound, "error": "unknown target source"}
            continue
        totals_for_source = totals[source]
        if totals_for_source is None or metric not in totals_for_source:
            gaps[key] = {
                "target": bound,
                "actual": None,
                "unavailable": True,
            }
            continue
        actual = totals_for_source[metric]
        if actual > bound:
            gaps[key] = {"target": bound, "actual": actual, "excess": actual - bound}
    return gaps


def measure_all(
    root: Path,
    scope_files: Sequence[str],
    fmt_dir: Path | None = None,
    structure: dict | None = None,
    targets: dict | None = None,
) -> dict:
    """One call returning written, separated, canonical (None if unavailable), and gaps."""
    written = measure_written(root, scope_files, structure=structure)
    separated = measure_separated(root, scope_files)
    canonical = measure_canonical(root, scope_files, fmt_dir) if fmt_dir is not None else None
    gaps = compute_measure_gaps(written, canonical, separated, targets or {})
    return {
        "measure_version": MEASURE_VERSION,
        "root": str(root),
        "as_written": written,
        "separated": separated,
        "canonical": canonical,
        "gaps": gaps,
    }
