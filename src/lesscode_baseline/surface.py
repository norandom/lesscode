"""Contract-B public-surface derivation and candidate verification (SurfaceDeriver).

All Python AST knowledge lives behind this seam: surface derivation, syntax
verification, and candidate verification. Controllers and the regenerator
never import ``ast`` directly; adding a target language means adding a sibling
extractor module, not rewriting callers.
"""
from __future__ import annotations

from pathlib import Path
from typing import Sequence

__all__ = ["EXTRACTOR_VERSION", "check_syntax", "derive_surface", "verify_candidate_surface"]

EXTRACTOR_VERSION = "python-ast-v1"


def check_syntax(
    cand_root: Path,
    scope_files: Sequence[str],
    language: str = "python",
) -> list[str]:
    """Return line-anchored syntax errors for ``scope_files`` under ``cand_root``.

    Never imports or executes candidate code. An empty list means every file
    parsed cleanly. Languages other than ``python`` are reported as an
    explicit unsupported-language error until a sibling extractor exists.
    """
    raise NotImplementedError("task 2.1")


def derive_surface(
    root: Path,
    scope_files: Sequence[str],
    recorder_files: Sequence[str] = (),
    scan_dirs: Sequence[str] = ("src", "packages", "tests"),
) -> dict:
    """Derive the Contract-B public surface report for ``scope_files``.

    Contract B: symbols imported only by the recorder oracle are part of the
    contract. Unresolvable targets stay in ``unresolved`` (never absent).
    """
    raise NotImplementedError("task 2.1")


def verify_candidate_surface(
    cand_root: Path,
    surface_report: dict,
    extra_patched: Sequence[str] = (),
) -> dict:
    """Verify a candidate tree against the Contract-B surface report.

    Reports missing symbols, kind mismatches, and local rebinding of
    ``patched`` module globals; never imports or executes candidate code.
    """
    raise NotImplementedError("task 2.2")
