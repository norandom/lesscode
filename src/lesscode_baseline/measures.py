"""Three-reading scope measures, structure validation, and edit-scope checks (ScopeMeasurer).

Counts as-written, separated-form (rule v1), and canonical (pinned ruff)
readings for a trial scope, validates declared pure/shell structure against a
pluggable ``EffectProfile``, and enforces edit scope by hash identity.
"""
from __future__ import annotations

from pathlib import Path
from typing import Sequence

__all__ = ["MEASURE_VERSION", "measure_all", "measure_canonical", "measure_pure_share", "measure_separated", "measure_written", "check_edit_scope", "compute_measure_gaps"]

MEASURE_VERSION = "scope-measures-v1"


def measure_written(root: Path, scope_files: Sequence[str], structure: dict | None = None) -> dict:
    """Count lines, stmts, funcs, qualnames, writes, edges, module_hops and validate structure."""
    raise NotImplementedError("task 3.1")


def measure_pure_share(root: Path, scope_files: Sequence[str], structure: dict) -> dict:
    """Attribute every statement to its innermost enclosing def (pure/shell/undeclared/outside)."""
    raise NotImplementedError("task 3.1")


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
