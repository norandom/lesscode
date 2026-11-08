"""Targeted adverse state mutation injector and evaluation harness."""
from __future__ import annotations

import ast
import copy
from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any, Callable, Optional


class StateMutationKind(str, Enum):
    DROP_FAILURE_STATE = "drop_failure_state"
    BUFFER_TRUNCATION = "buffer_truncation"
    ERROR_ORDER_SWAP = "error_order_swap"


class MutationOutcome(str, Enum):
    KILLED = "killed"
    SURVIVED = "survived"
    SYNTAX_ERROR = "syntax_error"
    TIMEOUT = "timeout"
    EQUIVALENT = "equivalent"


@dataclass
class StateMutant:
    mutant_id: str
    target_node: str
    mutation_kind: StateMutationKind
    original_code: str
    mutated_code: str
    line: int
    target_file: str = ""

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StateMutant:
        return cls(
            mutant_id=str(data["mutant_id"]),
            target_node=str(data["target_node"]),
            mutation_kind=StateMutationKind(data["mutation_kind"]),
            original_code=str(data["original_code"]),
            mutated_code=str(data["mutated_code"]),
            line=int(data["line"]),
            target_file=str(data.get("target_file", "")),
        )


@dataclass
class MutationResult:
    mutant_id: str
    outcome: MutationOutcome
    killer_test: Optional[str] = None
    witness: str = ""

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def _has_raise_or_return(stmt: ast.AST) -> bool:
    for child in ast.walk(stmt):
        if isinstance(child, (ast.Raise, ast.Return)):
            return True
    return False


def generate_state_mutants(source_code: str, target_file: str = "") -> list[StateMutant]:
    """Generates targeted adverse state mutants for failure states, output flushes, and error order."""
    tree = ast.parse(source_code)
    mutants: list[StateMutant] = []
    mutant_idx = 1

    # 1. Pattern: DROP_FAILURE_STATE in exception handlers
    for node in ast.walk(tree):
        if isinstance(node, ast.ExceptHandler):
            # Look for statements mutating state or audit log
            for i, stmt in enumerate(node.body):
                stmt_text = ast.unparse(stmt) if hasattr(ast, "unparse") else str(type(stmt))
                # If statement is a state write or method call (and not the re-raise itself)
                if not isinstance(stmt, ast.Raise):
                    cloned_tree = copy.deepcopy(tree)
                    # Find matching except handler in cloned tree
                    for cloned_node in ast.walk(cloned_tree):
                        if (
                            isinstance(cloned_node, ast.ExceptHandler)
                            and cloned_node.lineno == node.lineno
                            and len(cloned_node.body) == len(node.body)
                        ):
                            # Replace the statement with pass
                            cloned_node.body[i] = ast.Pass()
                            mutated_code = ast.unparse(cloned_tree) if hasattr(ast, "unparse") else source_code
                            mutants.append(
                                StateMutant(
                                    mutant_id=f"mut_state_{mutant_idx:03d}",
                                    target_node=stmt_text,
                                    mutation_kind=StateMutationKind.DROP_FAILURE_STATE,
                                    original_code=stmt_text,
                                    mutated_code=mutated_code,
                                    line=stmt.lineno,
                                    target_file=target_file,
                                )
                            )
                            mutant_idx += 1
                            break

    # 2. Pattern: BUFFER_TRUNCATION (dropping flush calls or intermediate output)
    for node in ast.walk(tree):
        if isinstance(node, ast.Expr) and isinstance(node.value, ast.Call):
            call = node.value
            is_flush = (
                isinstance(call.func, ast.Attribute) and call.func.attr == "flush"
            )
            if is_flush:
                orig_text = ast.unparse(node) if hasattr(ast, "unparse") else "flush()"
                cloned_tree = copy.deepcopy(tree)
                for cloned_node in ast.walk(cloned_tree):
                    if (
                        isinstance(cloned_node, ast.Expr)
                        and cloned_node.lineno == node.lineno
                        and isinstance(cloned_node.value, ast.Call)
                    ):
                        # Replace flush call with pass
                        cloned_node.value = ast.Constant(value=None)
                        mutated_code = ast.unparse(cloned_tree) if hasattr(ast, "unparse") else source_code
                        mutants.append(
                            StateMutant(
                                mutant_id=f"mut_flush_{mutant_idx:03d}",
                                target_node=orig_text,
                                mutation_kind=StateMutationKind.BUFFER_TRUNCATION,
                                original_code=orig_text,
                                mutated_code=mutated_code,
                                line=node.lineno,
                                target_file=target_file,
                            )
                        )
                        mutant_idx += 1
                        break

    # 3. Pattern: ERROR_ORDER_SWAP (swapping consecutive error checks)
    for node in ast.walk(tree):
        body = getattr(node, "body", None)
        if isinstance(body, list) and len(body) >= 2:
            for i in range(len(body) - 1):
                s1 = body[i]
                s2 = body[i + 1]
                if (
                    isinstance(s1, ast.If)
                    and isinstance(s2, ast.If)
                    and _has_raise_or_return(s1)
                    and _has_raise_or_return(s2)
                ):
                    cloned_tree = copy.deepcopy(tree)
                    for cloned_node in ast.walk(cloned_tree):
                        cloned_body = getattr(cloned_node, "body", None)
                        if (
                            isinstance(cloned_body, list)
                            and len(cloned_body) == len(body)
                            and getattr(cloned_node, "lineno", None) == getattr(node, "lineno", None)
                        ):
                            # Swap s1 and s2
                            cloned_body[i], cloned_body[i + 1] = cloned_body[i + 1], cloned_body[i]
                            mutated_code = ast.unparse(cloned_tree) if hasattr(ast, "unparse") else source_code
                            mutants.append(
                                StateMutant(
                                    mutant_id=f"mut_swap_{mutant_idx:03d}",
                                    target_node=f"swap(lines {s1.lineno}, {s2.lineno})",
                                    mutation_kind=StateMutationKind.ERROR_ORDER_SWAP,
                                    original_code=f"{ast.unparse(s1)}; {ast.unparse(s2)}" if hasattr(ast, "unparse") else "swap",
                                    mutated_code=mutated_code,
                                    line=s1.lineno,
                                    target_file=target_file,
                                )
                            )
                            mutant_idx += 1
                            break

    return mutants


def evaluate_mutant(
    mutant: StateMutant,
    test_runner: Callable[[str], tuple[bool, Optional[str]]],
    timeout_sec: float = 5.0,
) -> MutationResult:
    """Evaluates a single mutant with the provided test runner, classifying the outcome."""
    try:
        compile(mutant.mutated_code, "<mutant>", "exec")
    except Exception as ex:
        return MutationResult(
            mutant_id=mutant.mutant_id,
            outcome=MutationOutcome.SYNTAX_ERROR,
            witness=f"Mutated code failed syntax check: {ex}",
        )

    try:
        passed, killer = test_runner(mutant.mutated_code)
        if passed:
            return MutationResult(
                mutant_id=mutant.mutant_id,
                outcome=MutationOutcome.SURVIVED,
                witness=f"Mutant '{mutant.mutant_id}' ({mutant.mutation_kind.value}) survived without triggering test failure",
            )
        else:
            return MutationResult(
                mutant_id=mutant.mutant_id,
                outcome=MutationOutcome.KILLED,
                killer_test=killer,
                witness=f"Mutant '{mutant.mutant_id}' killed by test '{killer}'",
            )
    except TimeoutError:
        return MutationResult(
            mutant_id=mutant.mutant_id,
            outcome=MutationOutcome.TIMEOUT,
            witness=f"Mutant '{mutant.mutant_id}' execution timed out",
        )
    except Exception as ex:
        return MutationResult(
            mutant_id=mutant.mutant_id,
            outcome=MutationOutcome.KILLED,
            witness=f"Mutant '{mutant.mutant_id}' caused exception: {ex}",
        )
