"""Whole-responsibility density scorer and outcome gate."""
from __future__ import annotations

import ast
from dataclasses import asdict, dataclass, field
from typing import Any, Optional, Set


@dataclass
class ScopeMetrics:
    function_count: int
    call_edge_count: int
    write_site_count: int
    ast_statements: int
    canonical_lines: int
    pure_functions: int = 0

    @property
    def pure_function_pct(self) -> float:
        return (self.pure_functions / self.function_count * 100.0) if self.function_count > 0 else 0.0

    def to_dict(self) -> dict[str, Any]:
        d = asdict(self)
        d["pure_function_pct"] = round(self.pure_function_pct, 2)
        return d


@dataclass
class DeconvolutionOutcome:
    baseline_metrics: ScopeMetrics
    candidate_metrics: ScopeMetrics
    function_reduction_pct: float
    call_edge_reduction_pct: float
    write_site_reduction_pct: float
    statement_reduction_pct: float
    canonical_line_reduction_pct: float
    pure_function_pct: float
    structurally_consolidated: bool
    compression_met: bool
    min_statement_reduction_target: float = 0.0
    min_line_reduction_target: float = 0.0
    disposition_note: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "function_reduction_pct": round(self.function_reduction_pct, 2),
            "call_edge_reduction_pct": round(self.call_edge_reduction_pct, 2),
            "write_site_reduction_pct": round(self.write_site_reduction_pct, 2),
            "statement_reduction_pct": round(self.statement_reduction_pct, 2),
            "canonical_line_reduction_pct": round(self.canonical_line_reduction_pct, 2),
            "pure_function_pct": round(self.pure_function_pct, 2),
            "structurally_consolidated": self.structurally_consolidated,
            "compression_met": self.compression_met,
            "disposition_note": self.disposition_note,
        }


def _count_canonical_lines(source_code: str) -> int:
    lines = source_code.splitlines()
    count = 0
    for line in lines:
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            count += 1
    return count


def measure_scope_source(
    source_code: str,
    pure_functions: Optional[Set[str]] = None,
    shell_functions: Optional[Set[str]] = None,
) -> ScopeMetrics:
    """Measures function count, intra-scope call edges, state write sites, canonical lines, and AST statements."""
    tree = ast.parse(source_code)
    pure_set = pure_functions or set()
    shell_set = shell_functions or set()

    # 1. Collect all declared function names in the scope
    func_names: Set[str] = set()
    func_count = 0
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            func_names.add(node.name)
            func_count += 1

    # 2. Count AST statements
    stmt_count = 0
    for node in ast.walk(tree):
        if isinstance(node, ast.stmt):
            stmt_count += 1

    # 3. Count write sites
    write_count = 0
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            for t in node.targets:
                if isinstance(t, (ast.Attribute, ast.Name)):
                    write_count += 1
        elif isinstance(node, (ast.AugAssign, ast.AnnAssign)):
            if isinstance(node.target, (ast.Attribute, ast.Name)):
                write_count += 1
        elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Call):
            if isinstance(node.value.func, ast.Attribute):
                if node.value.func.attr in {"append", "extend", "insert", "remove", "pop", "clear", "update", "add"}:
                    write_count += 1

    # 4. Count intra-scope call edges
    call_edges = 0
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            target_name: Optional[str] = None
            if isinstance(node.func, ast.Name):
                target_name = node.func.id
            elif isinstance(node.func, ast.Attribute):
                target_name = node.func.attr
            if target_name and target_name in func_names:
                call_edges += 1

    # 5. Pure functions count
    pure_count = len(func_names & pure_set)

    canonical_lines = _count_canonical_lines(source_code)

    return ScopeMetrics(
        function_count=func_count,
        call_edge_count=call_edges,
        write_site_count=write_count,
        ast_statements=stmt_count,
        canonical_lines=canonical_lines,
        pure_functions=pure_count,
    )


def evaluate_deconvolution_outcome(
    baseline: ScopeMetrics,
    candidate: ScopeMetrics,
    min_statement_reduction_pct: float = 0.0,
    min_line_reduction_pct: float = 0.0,
) -> DeconvolutionOutcome:
    """Distinguishes structural consolidation from genuine compression, enforcing whole-responsibility targets."""
    func_red = (
        ((baseline.function_count - candidate.function_count) / baseline.function_count * 100.0)
        if baseline.function_count > 0
        else 0.0
    )
    edge_red = (
        ((baseline.call_edge_count - candidate.call_edge_count) / baseline.call_edge_count * 100.0)
        if baseline.call_edge_count > 0
        else 0.0
    )
    write_red = (
        ((baseline.write_site_count - candidate.write_site_count) / baseline.write_site_count * 100.0)
        if baseline.write_site_count > 0
        else 0.0
    )
    stmt_red = (
        ((baseline.ast_statements - candidate.ast_statements) / baseline.ast_statements * 100.0)
        if baseline.ast_statements > 0
        else 0.0
    )
    line_red = (
        ((baseline.canonical_lines - candidate.canonical_lines) / baseline.canonical_lines * 100.0)
        if baseline.canonical_lines > 0
        else 0.0
    )

    structurally_consolidated = func_red > 0.0 or edge_red > 0.0 or write_red > 0.0
    compression_met = stmt_red >= min_statement_reduction_pct and line_red >= min_line_reduction_pct

    disposition = ""
    if structurally_consolidated and not compression_met:
        disposition = (
            f"Density trap: structural consolidation achieved (functions -{func_red:.1f}%, "
            f"edges -{edge_red:.1f}%) but statement reduction ({stmt_red:.1f}%) or line reduction "
            f"({line_red:.1f}%) failed targets ({min_statement_reduction_pct:.1f}%, {min_line_reduction_pct:.1f}%)."
        )
    elif compression_met:
        disposition = (
            f"Whole-responsibility compression met: statements -{stmt_red:.1f}%, lines -{line_red:.1f}%."
        )

    return DeconvolutionOutcome(
        baseline_metrics=baseline,
        candidate_metrics=candidate,
        function_reduction_pct=func_red,
        call_edge_reduction_pct=edge_red,
        write_site_reduction_pct=write_red,
        statement_reduction_pct=stmt_red,
        canonical_line_reduction_pct=line_red,
        pure_function_pct=candidate.pure_function_pct,
        structurally_consolidated=structurally_consolidated,
        compression_met=compression_met,
        min_statement_reduction_target=min_statement_reduction_pct,
        min_line_reduction_target=min_line_reduction_pct,
        disposition_note=disposition,
    )
