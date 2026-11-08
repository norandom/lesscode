"""Functional core static purity verification and bounded equivalence."""
from __future__ import annotations

import ast
from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any, Optional, Set


class PurityViolationType(str, Enum):
    STATE_MUTATION = "state_mutation"
    GLOBAL_LOOKUP = "global_lookup"
    IO_INVOCATION = "io_invocation"


@dataclass
class PurityViolation:
    function_name: str
    violation_type: PurityViolationType
    location: str
    witness: str
    line: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


_PURE_BUILTINS: Set[str] = {
    "abs", "all", "any", "bool", "bytes", "bytearray", "chr", "complex",
    "dict", "divmod", "enumerate", "filter", "float", "format", "frozenset",
    "getattr", "hasattr", "hash", "hex", "int", "isinstance", "issubclass",
    "iter", "len", "list", "map", "max", "min", "next", "oct", "ord", "pow",
    "range", "repr", "reversed", "round", "set", "slice", "sorted", "str",
    "sum", "tuple", "type", "zip",
    # Constants
    "True", "False", "None",
    # Standard exceptions
    "Exception", "ValueError", "TypeError", "KeyError", "IndexError",
    "AttributeError", "RuntimeError", "StopIteration", "ArithmeticError",
    "ZeroDivisionError", "OverflowError", "LookupError", "AssertionError",
}

_DISALLOWED_IO_CALLS: Set[str] = {
    "print", "open", "input", "exec", "eval", "breakpoint",
}

_DISALLOWED_IO_ATTRS: Set[str] = {
    "write", "read", "readline", "readlines", "flush", "seek", "truncate",
    "send", "recv", "connect", "close", "commit", "rollback", "execute",
    "executemany", "log", "debug", "info", "warning", "warn", "error",
    "critical", "exception",
}


def _extract_assigned_names(target: ast.AST) -> Set[str]:
    names: Set[str] = set()
    if isinstance(target, ast.Name):
        names.add(target.id)
    elif isinstance(target, (ast.Tuple, ast.List)):
        for elt in target.elts:
            names.update(_extract_assigned_names(elt))
    return names


class StaticPurityVisitor(ast.NodeVisitor):
    def __init__(self, core_functions: Set[str], allowed_builtins: Optional[Set[str]] = None):
        self.core_functions = core_functions
        self.allowed_builtins = allowed_builtins or _PURE_BUILTINS
        self.violations: list[PurityViolation] = []
        self._current_core_func: Optional[str] = None
        self._local_names: Set[str] = set()
        self._params: Set[str] = set()

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        if node.name in self.core_functions:
            prev_func = self._current_core_func
            prev_locals = set(self._local_names)
            prev_params = set(self._params)

            self._current_core_func = node.name
            self._local_names = set()
            self._params = {arg.arg for arg in node.args.args}
            if node.args.vararg:
                self._params.add(node.args.vararg.arg)
            if node.args.kwarg:
                self._params.add(node.args.kwarg.arg)
            for arg in node.args.kwonlyargs:
                self._params.add(arg.arg)

            # Pre-scan local variable assignments in this function
            for child in ast.walk(node):
                if isinstance(child, ast.Assign):
                    for t in child.targets:
                        self._local_names.update(_extract_assigned_names(t))
                elif isinstance(child, (ast.AugAssign, ast.AnnAssign)):
                    self._local_names.update(_extract_assigned_names(child.target))
                elif isinstance(child, ast.For):
                    self._local_names.update(_extract_assigned_names(child.target))
                elif isinstance(child, ast.With):
                    for item in child.items:
                        if item.optional_vars:
                            self._local_names.update(_extract_assigned_names(item.optional_vars))
                elif isinstance(child, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                    for gen in child.generators:
                        self._local_names.update(_extract_assigned_names(gen.target))

            self.generic_visit(node)

            self._current_core_func = prev_func
            self._local_names = prev_locals
            self._params = prev_params
        else:
            self.generic_visit(node)

    visit_AsyncFunctionDef = visit_FunctionDef

    def visit_Global(self, node: ast.Global) -> None:
        if self._current_core_func:
            for name in node.names:
                self.violations.append(
                    PurityViolation(
                        function_name=self._current_core_func,
                        violation_type=PurityViolationType.GLOBAL_LOOKUP,
                        location=name,
                        witness=f"Declared global '{name}' in core function '{self._current_core_func}'",
                        line=node.lineno,
                    )
                )
        self.generic_visit(node)

    def visit_Nonlocal(self, node: ast.Nonlocal) -> None:
        if self._current_core_func:
            for name in node.names:
                self.violations.append(
                    PurityViolation(
                        function_name=self._current_core_func,
                        violation_type=PurityViolationType.GLOBAL_LOOKUP,
                        location=name,
                        witness=f"Declared nonlocal '{name}' in core function '{self._current_core_func}'",
                        line=node.lineno,
                    )
                )
        self.generic_visit(node)

    def visit_Assign(self, node: ast.Assign) -> None:
        if self._current_core_func:
            for target in node.targets:
                if isinstance(target, ast.Attribute):
                    loc = ast.unparse(target) if hasattr(ast, "unparse") else target.attr
                    self.violations.append(
                        PurityViolation(
                            function_name=self._current_core_func,
                            violation_type=PurityViolationType.STATE_MUTATION,
                            location=loc,
                            witness=f"Attribute mutation '{loc}' inside core function '{self._current_core_func}'",
                            line=node.lineno,
                        )
                    )
        self.generic_visit(node)

    def visit_AugAssign(self, node: ast.AugAssign) -> None:
        if self._current_core_func:
            if isinstance(node.target, ast.Attribute):
                loc = ast.unparse(node.target) if hasattr(ast, "unparse") else node.target.attr
                self.violations.append(
                    PurityViolation(
                        function_name=self._current_core_func,
                        violation_type=PurityViolationType.STATE_MUTATION,
                        location=loc,
                        witness=f"Augmented attribute mutation '{loc}' inside core function '{self._current_core_func}'",
                        line=node.lineno,
                    )
                )
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        if self._current_core_func:
            # Check direct I/O calls: e.g. print(...)
            if isinstance(node.func, ast.Name):
                if node.func.id in _DISALLOWED_IO_CALLS:
                    self.violations.append(
                        PurityViolation(
                            function_name=self._current_core_func,
                            violation_type=PurityViolationType.IO_INVOCATION,
                            location=node.func.id,
                            witness=f"External I/O call '{node.func.id}' inside core function '{self._current_core_func}'",
                            line=node.lineno,
                        )
                    )
            # Check attribute I/O calls: e.g. sys.stdout.write(...), log.info(...)
            elif isinstance(node.func, ast.Attribute):
                if node.func.attr in _DISALLOWED_IO_ATTRS:
                    loc = ast.unparse(node.func) if hasattr(ast, "unparse") else node.func.attr
                    self.violations.append(
                        PurityViolation(
                            function_name=self._current_core_func,
                            violation_type=PurityViolationType.IO_INVOCATION,
                            location=loc,
                            witness=f"External I/O invocation '{loc}' inside core function '{self._current_core_func}'",
                            line=node.lineno,
                        )
                    )
        self.generic_visit(node)

    def visit_Name(self, node: ast.Name) -> None:
        if self._current_core_func and isinstance(node.ctx, ast.Load):
            name = node.id
            # Allowed if it is a param, local var, pure builtin, or the function itself (for recursion)
            if (
                name not in self._params
                and name not in self._local_names
                and name not in self.allowed_builtins
                and name != self._current_core_func
            ):
                self.violations.append(
                    PurityViolation(
                        function_name=self._current_core_func,
                        violation_type=PurityViolationType.GLOBAL_LOOKUP,
                        location=name,
                        witness=f"Global lookup of undeclared '{name}' inside core function '{self._current_core_func}'",
                        line=node.lineno,
                    )
                )
        self.generic_visit(node)


def check_static_purity(
    source_code: str,
    core_functions: Set[str],
    allowed_builtins: Optional[Set[str]] = None,
) -> list[PurityViolation]:
    """Statically verifies that core functions have 0 state writes, 0 global lookups, and 0 external I/O invocations."""
    tree = ast.parse(source_code)
    visitor = StaticPurityVisitor(core_functions, allowed_builtins=allowed_builtins)
    visitor.visit(tree)
    return visitor.violations
