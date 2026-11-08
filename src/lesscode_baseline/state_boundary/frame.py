"""State frame condition and write-site analysis."""
from __future__ import annotations

import ast
from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any, Optional, Set


class StateViolationType(str, Enum):
    UNAUTHORIZED_WRITE = "unauthorized_write"
    MISSING_FAILURE_INVARIANT = "missing_failure_invariant"
    TYPE_CONFLICT = "type_conflict"


@dataclass
class StateViolation:
    location: str
    violation_type: StateViolationType
    witness: str
    line: Optional[int] = None


@dataclass
class StateFrame:
    allowed_writes: list[str] = field(default_factory=list)
    required_failure_invariants: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StateFrame:
        return cls(
            allowed_writes=list(data.get("allowed_writes", [])),
            required_failure_invariants=list(data.get("required_failure_invariants", [])),
        )


def _get_target_expr_str(node: ast.AST) -> Optional[str]:
    """Helper to stringify a target expression (e.g. self.buffer or GLOBAL_VAR)."""
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        val = _get_target_expr_str(node.value)
        if val:
            return f"{val}.{node.attr}"
    return None


class StaticWriteSiteVisitor(ast.NodeVisitor):
    def __init__(self, frame: StateFrame, shell_functions: Set[str]):
        self.frame = frame
        self.shell_functions = shell_functions
        self.violations: list[StateViolation] = []
        self._current_function: Optional[str] = None
        self._global_names_in_scope: Set[str] = set()

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        prev_func = self._current_function
        prev_globals = self._global_names_in_scope
        self._current_function = node.name
        self._global_names_in_scope = set()

        # Collect declared global/nonlocal names in this function
        for child in ast.walk(node):
            if isinstance(child, ast.Global):
                self._global_names_in_scope.update(child.names)
            elif isinstance(child, ast.Nonlocal):
                self._global_names_in_scope.update(child.names)

        self.generic_visit(node)
        self._current_function = prev_func
        self._global_names_in_scope = prev_globals

    visit_AsyncFunctionDef = visit_FunctionDef

    def _check_target(self, target: ast.AST, line: int) -> None:
        expr_str = _get_target_expr_str(target)
        if not expr_str:
            return

        is_state_write = False
        # If it's an attribute (e.g. self.attr or obj.field) or declared global/nonlocal, it's a state write
        if isinstance(target, ast.Attribute):
            is_state_write = True
        elif isinstance(target, ast.Name) and target.id in self._global_names_in_scope:
            is_state_write = True

        if not is_state_write:
            return

        func_name = self._current_function or "<module>"
        is_shell = func_name in self.shell_functions

        if not is_shell:
            self.violations.append(
                StateViolation(
                    location=expr_str,
                    violation_type=StateViolationType.UNAUTHORIZED_WRITE,
                    witness=f"State write to '{expr_str}' inside non-shell function '{func_name}'",
                    line=line,
                )
            )
        elif expr_str not in self.frame.allowed_writes:
            self.violations.append(
                StateViolation(
                    location=expr_str,
                    violation_type=StateViolationType.UNAUTHORIZED_WRITE,
                    witness=f"State write to undeclared location '{expr_str}' in shell function '{func_name}'",
                    line=line,
                )
            )

    def visit_Assign(self, node: ast.Assign) -> None:
        for t in node.targets:
            self._check_target(t, node.lineno)
        self.generic_visit(node)

    def visit_AugAssign(self, node: ast.AugAssign) -> None:
        self._check_target(node.target, node.lineno)
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        if node.value is not None:
            self._check_target(node.target, node.lineno)
        self.generic_visit(node)

    def visit_Expr(self, node: ast.Expr) -> None:
        # Check method mutations like GLOBAL_LOG.append(...) or self.list.extend(...)
        if isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Attribute):
            call_attr = node.value.func.attr
            if call_attr in {"append", "extend", "insert", "remove", "pop", "clear", "update", "add"}:
                receiver = _get_target_expr_str(node.value.func.value)
                if receiver:
                    func_name = self._current_function or "<module>"
                    is_shell = func_name in self.shell_functions
                    if not is_shell:
                        self.violations.append(
                            StateViolation(
                                location=receiver,
                                violation_type=StateViolationType.UNAUTHORIZED_WRITE,
                                witness=f"State mutation method '{call_attr}' on '{receiver}' in non-shell '{func_name}'",
                                line=node.lineno,
                            )
                        )
                    elif receiver not in self.frame.allowed_writes:
                        self.violations.append(
                            StateViolation(
                                location=receiver,
                                violation_type=StateViolationType.UNAUTHORIZED_WRITE,
                                witness=f"State mutation method '{call_attr}' on undeclared location '{receiver}' in '{func_name}'",
                                line=node.lineno,
                            )
                        )
        self.generic_visit(node)


def check_static_frame(source_code: str, frame: StateFrame, shell_functions: Set[str]) -> list[StateViolation]:
    """Statically verifies that all write sites reside strictly within declared shell functions."""
    tree = ast.parse(source_code)
    visitor = StaticWriteSiteVisitor(frame, shell_functions)
    visitor.visit(tree)
    return visitor.violations
