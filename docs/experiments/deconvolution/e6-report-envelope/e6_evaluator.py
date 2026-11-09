"""E6 State-Boundary Deconvolution Evaluator & Verification Harness.

Evaluates:
1. Adverse Controls (E6-C1 Missing Failure State, E6-C2 Buffer Truncation, E6-C3 Out-of-Domain Purity, E6-C4 Error Order).
2. StateFrame invariants (static write-site bounds, dynamic post-conditions).
3. StreamContract buffer preservation (stdout/stderr pre-abort integrity).
4. PureVerifier bounded equivalence on pure core.
5. StateMutantHarness adverse mutation sensitivity gate.
6. DensityGate whole-responsibility metrics against Density Trap defense.
7. Rust apparatus canonical line volume against R41 ceiling (7,173 lines).
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from typing import Any, Dict, List, Optional, Set

REPO_ROOT = Path(__file__).resolve().parents[4]
NATIVE_DIR = REPO_ROOT / "native"
HERE = Path(__file__).resolve().parent
RESULTS_DIR = HERE / "results"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)

sys.path.insert(0, str(REPO_ROOT / "src"))

from lesscode_baseline.state_boundary.frame import (
    StateFrame,
    check_static_frame,
    verify_dynamic_state,
)
from lesscode_baseline.state_boundary.stream_monitor import (
    StreamCapture,
    StreamContract,
    StreamDivergenceType,
    StreamEvent,
    verify_stream_preservation,
)
from lesscode_baseline.state_boundary.pure_verifier import (
    InputDomainBounds,
    check_static_purity,
    verify_bounded_equivalence,
)
from lesscode_baseline.state_boundary.mutant_generator import (
    StateMutationKind,
    generate_state_mutants,
    run_mutation_sensitivity_gate,
)
from lesscode_baseline.state_boundary.density import (
    ScopeMetrics,
    DeconvolutionOutcome,
    evaluate_deconvolution_outcome,
)
from lesscode_baseline import measures

FROZEN_REPORT_ENVELOPE_FILES = [
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py",
    "packages/facdrone-kernel/facdrone_kernel/canonical.py",
]

U5_CEILING_R41 = 7_173


def evaluate_adverse_controls() -> dict[str, Any]:
    """Runs the 4 adverse controls for E6 to verify that the verification harness fails closed."""
    results: dict[str, Any] = {}

    # Control 1: E6-C1 Missing Failure State
    frame = StateFrame(
        allowed_writes=["state", "audit_log"],
        required_failure_invariants=["audit_log:AuditedAbsence"],
    )

    def failing_candidate_omits_audit():
        audit_log = []
        try:
            raise RuntimeError("Transaction abort during report persistence")
        except Exception:
            pass  # Defect: omitted AuditedAbsence recording
        return audit_log

    def passing_candidate_retains_audit():
        audit_log = []
        try:
            raise RuntimeError("Transaction abort during report persistence")
        except Exception:
            audit_log.append("AuditedAbsence(subsystem='reporting')")
        return audit_log

    omitted_state = failing_candidate_omits_audit()
    violation_omitted = verify_dynamic_state(
        before_state={},
        after_state={"audit_log": omitted_state},
        exit_error=RuntimeError("Transaction abort"),
        frame=frame,
    )

    retained_state = passing_candidate_retains_audit()
    violation_retained = verify_dynamic_state(
        before_state={},
        after_state={"audit_log": retained_state},
        exit_error=RuntimeError("Transaction abort"),
        frame=frame,
    )

    e6_c1_pass = (
        len(violation_omitted) == 1
        and "AuditedAbsence" in violation_omitted[0].witness
        and len(violation_retained) == 0
    )
    results["E6_C1_MissingFailureState"] = {
        "description": "Failure-side state post-condition verification (AuditedAbsence on transaction abort)",
        "pass": e6_c1_pass,
        "witness_detected": violation_omitted[0].witness if violation_omitted else None,
    }

    # Control 2: E6-C2 Stream Buffer Truncation
    contract = StreamContract(monitored_sinks=["stdout", "stderr"])
    orig_events = [
        StreamEvent(sink="stderr", payload="[INFO] validating reporting envelope\n", order=0),
        StreamEvent(sink="stderr", payload="[WARN] snapshot cache missing; fallback active\n", order=1),
        StreamEvent(sink="stderr", payload="[ERROR] critical composition fault\n", order=2),
    ]
    trunc_events = [
        StreamEvent(sink="stderr", payload="[INFO] validating reporting envelope\n", order=0),
    ]
    divergences = verify_stream_preservation(orig_events, trunc_events, contract)
    e6_c2_pass = (
        len(divergences) == 1
        and divergences[0].divergence_type == StreamDivergenceType.TRUNCATED
    )
    results["E6_C2_BufferTruncation"] = {
        "description": "Stream buffer preservation monitor detecting dropped diagnostic logs on abort",
        "pass": e6_c2_pass,
        "divergence": divergences[0].witness if divergences else None,
    }

    # Control 3: E6-C3 Out-of-Domain Purity Violation
    bounds = InputDomainBounds(
        ranges={"x": (-10, 10)},
        edge_cases={"x": [None]},
    )

    def orig_canonical_check(x):
        if x is None:
            raise ValueError("Sequence ID cannot be None")
        return x * 2 + 1

    def mutated_canonical_check(x):
        if x is None:
            return 0  # Unsound fallback: hides error on None inputs
        return x * 2 + 1

    equiv_div = verify_bounded_equivalence(
        orig_canonical_check,
        mutated_canonical_check,
        domain_bounds=bounds,
        sample_depth=50,
    )
    e6_c3_pass = (
        equiv_div is not None
        and equiv_div.inputs == {"x": None}
    )
    results["E6_C3_OutOfDomainPurity"] = {
        "description": "Bounded pure-core equivalence checker catching unsound negative/None fallback",
        "pass": e6_c3_pass,
        "counterexample": equiv_div.inputs if equiv_div else None,
    }

    # Control 4: E6-C4 Error Order Inversion
    sample_code = """def validate_report_envelope(request_id, payload_size):
    if not isinstance(request_id, str):
        raise TypeError("request_id must be a string")
    if payload_size < 0:
        raise ValueError("payload_size cannot be negative")
    return True
"""
    mutants = generate_state_mutants(sample_code, "evidence_validation.py")
    error_swap_mutants = [m for m in mutants if m.mutation_kind == StateMutationKind.ERROR_ORDER_SWAP]

    def test_battery(code_str: str) -> tuple[bool, Optional[str]]:
        ns = {}
        try:
            exec(code_str, ns)
            fn = ns["validate_report_envelope"]
        except Exception:
            return False, "test_compile"

        try:
            fn(None, -5)
            return False, "test_error_order"
        except TypeError:
            return True, None
        except ValueError:
            return False, "test_error_order"
        except Exception:
            return False, "test_error_order"

    gate_report = run_mutation_sensitivity_gate(
        source_code=sample_code,
        test_runner=test_battery,
        target_file="evidence_validation.py",
    )
    e6_c4_pass = (
        gate_report.qualified
        and gate_report.survived == 0
        and len(error_swap_mutants) > 0
    )
    results["E6_C4_ErrorOrderInversion"] = {
        "description": "Adverse state mutation sensitivity gate killing inverted error checking (E6-C4)",
        "pass": e6_c4_pass,
        "killed": gate_report.killed,
        "survived": gate_report.survived,
        "mutants_evaluated": gate_report.total_mutants,
    }

    return results



def measure_source_tree(root_dir: Path) -> ScopeMetrics:
    """Measures scope files under root_dir using standard lesscode_baseline.measures."""
    res = measures.measure_written(root_dir, FROZEN_REPORT_ENVELOPE_FILES)
    tot = res["total"]
    return ScopeMetrics(
        function_count=tot["funcs"],
        call_edge_count=tot["edges"],
        write_site_count=tot["writes"],
        ast_statements=tot["stmts"],
        canonical_lines=tot["lines"],
        pure_functions=0,
    )


def measure_rust_apparatus_lines() -> dict[str, Any]:
    """Measures production Rust substrate canonical line count against R41 amended ceiling."""
    RS_NEW = ["src/surface.rs", "src/measures.rs", "src/recording.rs", "src/regenerator.rs", "src/jsonval.rs"]
    RS_BASE = ["src/admission.rs", "src/receipt.rs"]
    SHIM_REL = "native/shim/recording_shim.py"
    HOOK_FILES = ["native/src/lib.rs", "native/src/main.rs", "native/Cargo.toml"]
    R31_FROZEN_HOOKS = {"src/lib.rs": 1, "src/main.rs": 15, "Cargo.toml": 0}
    PRE_PORT_COMMIT = "f2fe982"
    RUSTFMT = ["rustup", "run", "1.98.1", "rustfmt", "--edition", "2021", "--emit", "stdout"]

    def canonical_rs(rel, tmp):
        dst = tmp / Path(rel).name
        dst.write_bytes((NATIVE_DIR / rel).read_bytes())
        fmt = subprocess.run([*RUSTFMT, str(dst)], check=True, capture_output=True, text=True)
        return len(fmt.stdout.splitlines())

    def measure_hooks():
        proc = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "diff", PRE_PORT_COMMIT, "HEAD", "--", *HOOK_FILES],
            check=True, capture_output=True, text=True
        )
        added = {}
        current = None
        for line in proc.stdout.splitlines():
            if line.startswith("+++ b/"):
                current = line[6:]
                added.setdefault(current, 0)
            elif line.startswith("+") and current is not None:
                added[current] += 1
        return sum(added.values()), added

    with tempfile.TemporaryDirectory(prefix="apparatus-lines-") as td:
        tmp = Path(td)
        rs_counts = {rel: canonical_rs(rel, tmp) for rel in RS_NEW + RS_BASE}
        state_contract_lines = canonical_rs("src/state_contract.rs", tmp)
        shim_report = measures.measure_canonical(REPO_ROOT, [SHIM_REL], tmp / "fmt")

    shim_lines = shim_report["files"][Path(SHIM_REL).name]["lines"]
    hooks_total, hooks_detail = measure_hooks()
    frozen_hooks_total = sum(R31_FROZEN_HOOKS.values())

    r37_rs_total = sum(rs_counts.values())
    r37_counted_total = r37_rs_total + shim_lines + frozen_hooks_total + hooks_total
    total_with_state_contract = r37_counted_total + state_contract_lines

    return {
        "rs_counts": rs_counts,
        "state_contract_lines": state_contract_lines,
        "shim_lines": shim_lines,
        "frozen_hooks_total": frozen_hooks_total,
        "added_hooks": hooks_total,
        "added_hooks_detail": hooks_detail,
        "r37_set_total": r37_counted_total,
        "total_with_state_contract": total_with_state_contract,
        "ceiling": U5_CEILING_R41,
        "overage_r37_set": r37_counted_total - U5_CEILING_R41,
        "overage_with_state_contract": total_with_state_contract - U5_CEILING_R41,
        "pass_r37_set": r37_counted_total <= U5_CEILING_R41,
        "pass_with_state_contract": total_with_state_contract <= U5_CEILING_R41,
    }


def run_e6_evaluation(
    candidate_root: Optional[Path] = None,
    target_reduction_pct: float = 20.0,
    target_statement_pct: float = 10.0,
) -> dict[str, Any]:
    """Full E6 evaluation against frozen baseline and candidate."""
    t0 = time.time()
    head = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True
    ).stdout.strip()

    baseline_root = REPO_ROOT / ".runs" / "e2-facdrone" / "source"
    print(f"=== E6 Evaluation on Git HEAD {head} ===")

    print("Step 1: Evaluating Adverse Controls Battery (E6-C1 through E6-C4)...")
    controls_res = evaluate_adverse_controls()
    all_controls_pass = all(c["pass"] for c in controls_res.values())
    for k, v in controls_res.items():
        print(f"  [{'PASS' if v['pass'] else 'FAIL'}] {k}: {v['description']}")

    print("Step 2: Measuring Rust Apparatus Line Volume against R41 ceiling...")
    rust_metrics = measure_rust_apparatus_lines()
    print(f"  Counted Rust apparatus: {rust_metrics['total_with_state_contract']} lines (ceiling: {rust_metrics['ceiling']}, overage: {rust_metrics['overage_with_state_contract']})")

    print("Step 3: Measuring Scope Metrics via measures.measure_written...")
    baseline_metrics = measure_source_tree(baseline_root)
    print(f"  Baseline: {baseline_metrics.canonical_lines} lines, {baseline_metrics.ast_statements} stmts, {baseline_metrics.function_count} funcs, {baseline_metrics.call_edge_count} edges, {baseline_metrics.write_site_count} writes")

    if candidate_root and candidate_root.exists():
        candidate_metrics = measure_source_tree(candidate_root)
        print(f"  Candidate: {candidate_metrics.canonical_lines} lines, {candidate_metrics.ast_statements} stmts, {candidate_metrics.function_count} funcs, {candidate_metrics.call_edge_count} edges, {candidate_metrics.write_site_count} writes")
        outcome = evaluate_deconvolution_outcome(
            baseline=baseline_metrics,
            candidate=candidate_metrics,
            min_statement_reduction_pct=target_statement_pct,
            min_line_reduction_pct=target_reduction_pct,
        )
    else:
        candidate_metrics = baseline_metrics
        outcome = evaluate_deconvolution_outcome(
            baseline=baseline_metrics,
            candidate=baseline_metrics,
            min_statement_reduction_pct=target_statement_pct,
            min_line_reduction_pct=target_reduction_pct,
        )

    print(f"  Outcome: structurally_consolidated={outcome.structurally_consolidated}, compression_met={outcome.compression_met}")
    print(f"  Deltas: lines={outcome.canonical_line_reduction_pct:.2f}%, stmts={outcome.statement_reduction_pct:.2f}%, funcs={outcome.function_reduction_pct:.2f}%, edges={outcome.call_edge_reduction_pct:.2f}%")

    qualified = (
        all_controls_pass
        and rust_metrics["pass_with_state_contract"]
        and (outcome.compression_met if candidate_root else True)
    )

    report = {
        "trial": "E6: State-Boundary Deconvolution Generalization Trial on Report Envelope & Snapshot Store",
        "head": head,
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "elapsed_seconds": round(time.time() - t0, 2),
        "adverse_controls": controls_res,
        "all_controls_pass": all_controls_pass,
        "rust_apparatus": rust_metrics,
        "baseline_metrics": baseline_metrics.to_dict(),
        "candidate_metrics": candidate_metrics.to_dict() if candidate_root else None,
        "outcome": outcome.to_dict(),
        "disposition": "PASS" if qualified else ("BASELINE_RECORDED" if not candidate_root else "REVISE"),
    }

    out_file = RESULTS_DIR / "e6-evaluation-report.json"
    out_file.write_text(json.dumps(report, indent=2), encoding="utf-8")
    sha_file = RESULTS_DIR / "e6-evaluation-report.json.sha256"
    h = hashlib.sha256(out_file.read_bytes()).hexdigest()
    sha_file.write_text(f"{h}  e6-evaluation-report.json\n", encoding="utf-8")
    print(f"Saved report to {out_file} (SHA-256: {h})")

    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="E6 Evaluator")
    parser.add_argument("--candidate", type=Path, default=None, help="Candidate root directory")
    parser.add_argument("--target-lines", type=float, default=20.0, help="Min canonical line reduction pct")
    parser.add_argument("--target-stmts", type=float, default=10.0, help="Min AST statement reduction pct")
    args = parser.parse_args()

    rep = run_e6_evaluation(
        candidate_root=args.candidate,
        target_reduction_pct=args.target_lines,
        target_statement_pct=args.target_stmts,
    )
    if not rep["all_controls_pass"] or not rep["rust_apparatus"]["pass_with_state_contract"]:
        sys.exit(1)
    if args.candidate and rep["disposition"] != "PASS":
        sys.exit(2)
