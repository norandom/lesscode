"""Static test-infrastructure detection for a repository and a scope (TestInfraDetector).

Preserves the cross-language report envelope (runner, fixtures, external_store,
scope_references, seeds, marker_expression, deselections, rule_version) with
pytest-specific keys scoped as runner subfields; non-Python repositories
return ``runner.name = "unknown"`` and zero seeds cleanly (R32).
"""
from __future__ import annotations

from pathlib import Path
from typing import Sequence

__all__ = ["RULE_VERSION", "detect", "render", "main"]

RULE_VERSION = "testinfra-rules-v1"


def detect(repo_root: Path, scope: str, baseline_failures: Sequence[str] | None = None, seeds_file: Path | None = None, site_packages: Path | None = None) -> dict:
    """Detect runner, config, markers, fixtures/plugins, store facts, hazards, and scope references."""
    raise NotImplementedError("task 4.1")


def render(report: dict) -> str:
    """Render the detection report as a readable summary."""
    raise NotImplementedError("task 4.1")


def main(argv: list[str] | None = None) -> int:
    """CLI entry: ``testinfra <repo_root> --scope ... --out ... --stem ...``."""
    raise NotImplementedError("task 4.1")
