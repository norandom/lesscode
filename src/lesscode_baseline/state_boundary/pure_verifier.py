"""Functional core static purity verification and bounded equivalence."""
from __future__ import annotations

import ast
from dataclasses import asdict, dataclass, field
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


@dataclass
class InputDomainBounds:
    ranges: dict[str, tuple[int | float, int | float]] = field(default_factory=dict)
    edge_cases: dict[str, list[Any]] = field(default_factory=dict)
    custom_samples: list[dict[str, Any]] = field(default_factory=list)


@dataclass
class PureDivergenceWitness:
    function_name: str
    inputs: dict[str, Any]
    original_output: Any
    candidate_output: Any
    divergence_kind: str  # "value_mismatch" | "exception_mismatch" | "unhandled_exception"
    witness: str

    def to_obligation_string(self) -> str:
        return f"PureCoreDivergence:{self.function_name}:{self.inputs}:{self.divergence_kind}:{self.witness}"


def _safe_call(fn: Any, kwargs: dict[str, Any]) -> tuple[Any, Optional[Exception]]:
    try:
        return fn(**kwargs), None
    except Exception as ex:
        return None, ex


def _is_divergent(original_fn: Any, candidate_fn: Any, kw: dict[str, Any]) -> bool:
    orig_res, orig_exc = _safe_call(original_fn, kw)
    cand_res, cand_exc = _safe_call(candidate_fn, kw)
    if (orig_exc is None) != (cand_exc is None):
        return True
    if orig_exc is not None and cand_exc is not None:
        return type(orig_exc) is not type(cand_exc)
    return orig_res != cand_res


def _minimize_counterexample(
    original_fn: Any,
    candidate_fn: Any,
    kw: dict[str, Any],
) -> dict[str, Any]:
    """Shrinks counterexample inputs to minimal reproducible bounds."""
    minimized = dict(kw)
    for p, val in kw.items():
        if isinstance(val, int) and val != 0:
            trial = dict(minimized, **{p: 0})
            if _is_divergent(original_fn, candidate_fn, trial):
                minimized[p] = 0
                continue
            low = 0 if val > 0 else val
            high = val if val > 0 else 0
            best = val
            while abs(high - low) > 1:
                mid = (low + high) // 2
                trial = dict(minimized, **{p: mid})
                if _is_divergent(original_fn, candidate_fn, trial):
                    best = mid
                    if val > 0:
                        high = mid
                    else:
                        low = mid
                else:
                    if val > 0:
                        low = mid
                    else:
                        high = mid
            minimized[p] = best
        elif isinstance(val, str) and len(val) > 1:
            trial = dict(minimized, **{p: ""})
            if _is_divergent(original_fn, candidate_fn, trial):
                minimized[p] = ""
                continue
            for length in range(1, len(val)):
                trial = dict(minimized, **{p: val[:length]})
                if _is_divergent(original_fn, candidate_fn, trial):
                    minimized[p] = val[:length]
                    break
        elif isinstance(val, list) and len(val) > 1:
            trial = dict(minimized, **{p: []})
            if _is_divergent(original_fn, candidate_fn, trial):
                minimized[p] = []
                continue
            for length in range(1, len(val)):
                trial = dict(minimized, **{p: val[:length]})
                if _is_divergent(original_fn, candidate_fn, trial):
                    minimized[p] = val[:length]
                    break
    return minimized


def verify_bounded_equivalence(
    original_fn: Any,
    candidate_fn: Any,
    domain_bounds: Optional[InputDomainBounds] = None,
    sample_depth: int = 50,
) -> Optional[PureDivergenceWitness]:
    """Compares candidate pure logic against original function over bounded inputs, returning minimal divergence witness."""
    import inspect
    import itertools

    if domain_bounds is None:
        domain_bounds = InputDomainBounds()

    sig = inspect.signature(original_fn)
    param_names = list(sig.parameters.keys())
    fn_name = getattr(candidate_fn, "__name__", getattr(original_fn, "__name__", "pure_core"))

    input_candidates: list[dict[str, Any]] = []

    # 1. Add custom samples
    if domain_bounds.custom_samples:
        input_candidates.extend(domain_bounds.custom_samples)

    # 2. Add edge case combinations
    edge_cases_by_param: dict[str, list[Any]] = {}
    for p in param_names:
        if p in domain_bounds.edge_cases:
            edge_cases_by_param[p] = domain_bounds.edge_cases[p]
        else:
            edge_cases_by_param[p] = [0, 1, -1, "", None]

    # Generate edge combinations
    edge_keys = [p for p in param_names if p in edge_cases_by_param]
    if edge_keys:
        val_lists = [edge_cases_by_param[k] for k in edge_keys]
        for combo in itertools.product(*val_lists):
            kw = dict(zip(edge_keys, combo))
            if kw not in input_candidates:
                input_candidates.append(kw)

    # 3. Add range-bounded samples
    range_params = [p for p in param_names if p in domain_bounds.ranges]
    if range_params:
        for p in range_params:
            low, high = domain_bounds.ranges[p]
            step = max(1, int((high - low) / max(1, sample_depth)))
            for v in range(int(low), int(high) + 1, step):
                kw = {p: v}
                for other in param_names:
                    if other != p and other not in kw:
                        kw[other] = edge_cases_by_param.get(other, [0])[0]
                if kw not in input_candidates:
                    input_candidates.append(kw)

    # Evaluate each input candidate
    for raw_kw in input_candidates:
        if _is_divergent(original_fn, candidate_fn, raw_kw):
            kw = _minimize_counterexample(original_fn, candidate_fn, raw_kw)
            orig_res, orig_exc = _safe_call(original_fn, kw)
            cand_res, cand_exc = _safe_call(candidate_fn, kw)

            if orig_exc is None and cand_exc is not None:
                return PureDivergenceWitness(
                    function_name=fn_name,
                    inputs=kw,
                    original_output=orig_res,
                    candidate_output=f"{type(cand_exc).__name__}: {cand_exc}",
                    divergence_kind="unhandled_exception",
                    witness=f"Candidate raised unexpected {type(cand_exc).__name__}: {cand_exc}",
                )

            if orig_exc is not None and cand_exc is None:
                return PureDivergenceWitness(
                    function_name=fn_name,
                    inputs=kw,
                    original_output=f"{type(orig_exc).__name__}: {orig_exc}",
                    candidate_output=cand_res,
                    divergence_kind="exception_mismatch",
                    witness=f"Original raised {type(orig_exc).__name__} but candidate succeeded with {cand_res}",
                )

            if orig_exc is not None and cand_exc is not None:
                if type(orig_exc) is not type(cand_exc):
                    return PureDivergenceWitness(
                        function_name=fn_name,
                        inputs=kw,
                        original_output=f"{type(orig_exc).__name__}: {orig_exc}",
                        candidate_output=f"{type(cand_exc).__name__}: {cand_exc}",
                        divergence_kind="exception_mismatch",
                        witness=f"Exception type divergence: original raised {type(orig_exc).__name__}, candidate raised {type(cand_exc).__name__}",
                    )

            if orig_exc is None and cand_exc is None:
                if orig_res != cand_res:
                    return PureDivergenceWitness(
                        function_name=fn_name,
                        inputs=kw,
                        original_output=orig_res,
                        candidate_output=cand_res,
                        divergence_kind="value_mismatch",
                        witness=f"Output value divergence on {kw}: original returned {orig_res!r}, candidate returned {cand_res!r}",
                    )

    return None


