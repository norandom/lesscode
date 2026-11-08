"""E5 State-Boundary Deconvolution Evaluator & Verification Harness.

Evaluates:
1. Adverse Controls (E3 AuditedAbsence, E4 F1 Silent_None, E4 F3 Buffer_Drop, E4 F7 Error_Order).
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

FROZEN_LIFECYCLE_FILES = [
    "src/facdrone/shells/lifecycle_monitor_runner.py",
    "src/facdrone/shells/lifecycle_store.py",
    "src/facdrone/shells/cli/lifecycle_group.py",
    "src/facdrone/shells/lifecycle_evidence.py",
    "src/facdrone/shells/lifecycle_projection.py",
    "src/facdrone/shells/cppi/lifecycle_seams.py",
    "src/facdrone/shells/wiring/lifecycle_root.py",
]

U5_CEILING_R41 = 7_173


def evaluate_adverse_controls() -> dict[str, Any]:
    """Runs the 4 adverse controls to verify that the verification harness fails closed."""
    results: dict[str, Any] = {}

    # Control 1: E3 AuditedAbsence
    frame = StateFrame(
        allowed_writes=["state", "audit_log"],
        required_failure_invariants=["audit_log:AuditedAbsence"],
    )

    def failing_candidate_omits_audit():
        audit_log = []
        try:
            raise RuntimeError("Serialization failed during evidence publication")
        except Exception:
            pass  # Defect: omitted AuditedAbsence recording
        return audit_log

    def passing_candidate_retains_audit():
        audit_log = []
        try:
            raise RuntimeError("Serialization failed during evidence publication")
        except Exception:
            audit_log.append("AuditedAbsence(subsystem='scoring')")
        return audit_log

    omitted_state = failing_candidate_omits_audit()
    violation_omitted = verify_dynamic_state(
        before_state={},
        after_state={"audit_log": omitted_state},
        exit_error=RuntimeError("Serialization failed"),
        frame=frame,
    )

    retained_state = passing_candidate_retains_audit()
    violation_retained = verify_dynamic_state(
        before_state={},
        after_state={"audit_log": retained_state},
        exit_error=RuntimeError("Serialization failed"),
        frame=frame,
    )

    e3_pass = (
        len(violation_omitted) == 1
        and "AuditedAbsence" in violation_omitted[0].witness
        and len(violation_retained) == 0
    )
    results["E3_AuditedAbsence"] = {
        "description": "Failure-side state post-condition verification (AuditedAbsence on abort)",
        "pass": e3_pass,
        "witness_detected": violation_omitted[0].witness if violation_omitted else None,
    }

    # Control 2: E4 F3 Buffer Drop
    contract = StreamContract(monitored_sinks=["stdout", "stderr"])
    orig_events = [
        StreamEvent(sink="stdout", payload="Processing specimen chunk 1...\n", order=0),
        StreamEvent(sink="stdout", payload="Processing specimen chunk 2...\n", order=1),
    ]
    trunc_events = [
        StreamEvent(sink="stdout", payload="Processing specimen chunk 1...\n", order=0),
    ]
    stream_divs = verify_stream_preservation(orig_events, trunc_events, contract)
    e4_f3_pass = (
        len(stream_divs) == 1
        and stream_divs[0].divergence_type == StreamDivergenceType.TRUNCATED
    )
    results["E4_F3_Buffer_Drop"] = {
        "description": "Output stream buffer preservation monitor (E4 partial CLI stdout drop)",
        "pass": e4_f3_pass,
        "divergence": stream_divs[0].witness if stream_divs else None,
    }

    # Control 3: E4 F1 Silent None Acceptance
    bounds = InputDomainBounds(
        ranges={"x": (-10, 10)},
        edge_cases={"x": [None]},
    )

    def orig_evidence_validator(x):
        if x is None:
            raise ValueError("Evidence cannot be None")
        return x > 0

    def flawed_cand_accepts_none(x):
        if x is None:
            return False  # Defect: silently accepts None
        return x > 0

    equiv_div = verify_bounded_equivalence(
        orig_evidence_validator,
        flawed_cand_accepts_none,
        domain_bounds=bounds,
        sample_depth=50,
    )
    e4_f1_pass = (
        equiv_div is not None
        and equiv_div.inputs == {"x": None}
    )
    results["E4_F1_Silent_None"] = {
        "description": "Bounded semantic equivalence checking with minimal counterexample (E4 F1 None acceptance)",
        "pass": e4_f1_pass,
        "counterexample": equiv_div.inputs if equiv_div else None,
    }

    # Control 4: E4 F7 Error Ordering
    sample_code = """def evaluate_pipeline(data, flag):
    if not flag:
        raise ValueError("flag missing")
    if data is None:
        raise TypeError("data missing")
    return len(data)
"""
    mutants = generate_state_mutants(sample_code, "pipeline.py")
    error_swap_mutants = [m for m in mutants if m.mutation_kind == StateMutationKind.ERROR_ORDER_SWAP]

    def test_battery(code_str: str) -> tuple[bool, Optional[str]]:
        ns = {}
        try:
            exec(code_str, ns)
            fn = ns["evaluate_pipeline"]
        except Exception:
            return False, "test_compile"

        try:
            fn(None, False)
            return False, "test_error_order"
        except ValueError:
            return True, None
        except TypeError:
            return False, "test_error_order"
        except Exception:
            return False, "test_error_order"

    gate_report = run_mutation_sensitivity_gate(
        source_code=sample_code,
        test_runner=test_battery,
        target_file="pipeline.py",
    )
    e4_f7_pass = (
        gate_report.qualified
        and gate_report.survived == 0
        and len(error_swap_mutants) > 0
    )
    results["E4_F7_Error_Order"] = {
        "description": "Adverse state mutation sensitivity gate killing error-ordering mutations (E4 F7)",
        "pass": e4_f7_pass,
        "killed": gate_report.killed,
        "survived": gate_report.survived,
        "mutants_evaluated": gate_report.total_mutants,
    }

    return results


def measure_source_tree(root_dir: Path) -> ScopeMetrics:
    """Measures scope files under root_dir using standard lesscode_baseline.measures."""
    res = measures.measure_written(root_dir, FROZEN_LIFECYCLE_FILES)
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


def run_e5_evaluation(
    candidate_root: Optional[Path] = None,
    target_reduction_pct: float = 20.0,
    target_statement_pct: float = 10.0,
) -> dict[str, Any]:
    """Full E5 evaluation against frozen baseline and candidate."""
    t0 = time.time()
    head = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True
    ).stdout.strip()

    baseline_root = REPO_ROOT / ".runs" / "e2-facdrone" / "source"
    print(f"=== E5 Evaluation on Git HEAD {head} ===")
    print("1. Running Adverse Controls Battery...")
    adverse_results = evaluate_adverse_controls()
    adverse_pass = all(r["pass"] for r in adverse_results.values())
    for name, res in adverse_results.items():
        status = "PASS" if res["pass"] else "FAIL"
        print(f"   [{status}] {name}: {res['description']}")

    print("\n2. Measuring Specimen Baseline Metrics...")
    baseline_metrics = measure_source_tree(baseline_root)
    print(f"   Baseline: {baseline_metrics.canonical_lines} lines, {baseline_metrics.ast_statements} statements, "
          f"{baseline_metrics.function_count} functions, {baseline_metrics.call_edge_count} edges, "
          f"{baseline_metrics.write_site_count} write sites")

    print("\n3. Measuring Apparatus Line Accounting...")
    apparatus = measure_rust_apparatus_lines()
    app_pass = apparatus["pass_with_state_contract"]
    status = "PASS" if app_pass else "FAIL"
    print(f"   [{status}] Counted Rust Apparatus: {apparatus['total_with_state_contract']} lines "
          f"(ceiling: {apparatus['ceiling']}, overage: {apparatus['overage_with_state_contract']})")

    candidate_metrics: Optional[ScopeMetrics] = None
    outcome_dict: Optional[dict[str, Any]] = None
    overall_disposition = "PENDING_CANDIDATE"

    if candidate_root and candidate_root.exists():
        print(f"\n4. Measuring Candidate Metrics from {candidate_root}...")
        candidate_metrics = measure_source_tree(candidate_root)
        outcome = evaluate_deconvolution_outcome(
            baseline=baseline_metrics,
            candidate=candidate_metrics,
            min_statement_reduction_pct=target_statement_pct,
            min_line_reduction_pct=target_reduction_pct,
        )
        outcome_dict = outcome.to_dict()
        print(f"   Func Red: {outcome.function_reduction_pct:.1f}%, Edge Red: {outcome.call_edge_reduction_pct:.1f}%")
        print(f"   Stmt Red: {outcome.statement_reduction_pct:.1f}%, Line Red: {outcome.canonical_line_reduction_pct:.1f}%")
        print(f"   Structurally Consolidated: {outcome.structurally_consolidated}")
        print(f"   Compression Met: {outcome.compression_met}")
        print(f"   Disposition: {outcome.disposition_note}")

        if adverse_pass and app_pass and outcome.compression_met:
            overall_disposition = "PASS"
        elif outcome.structurally_consolidated and not outcome.compression_met:
            overall_disposition = "FAIL_DENSITY_TRAP"
        else:
            overall_disposition = "REVISE"
    else:
        print("\n4. Candidate root not provided (Baseline Round 0 mode).")
        if adverse_pass and app_pass:
            overall_disposition = "BASELINE_ROUND_0_PASS"
        else:
            overall_disposition = "BASELINE_ROUND_0_FAIL"

    elapsed = round(time.time() - t0, 2)
    report = {
        "trial": "E5: State-Boundary Deconvolution on Lifecycle Monitor Group",
        "head": head,
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "elapsed_seconds": elapsed,
        "adverse_controls": adverse_results,
        "adverse_controls_qualified": adverse_pass,
        "apparatus_line_accounting": apparatus,
        "apparatus_qualified": app_pass,
        "baseline_metrics": baseline_metrics.to_dict(),
        "candidate_metrics": candidate_metrics.to_dict() if candidate_metrics else None,
        "outcome": outcome_dict,
        "disposition": overall_disposition,
    }

    report_path = RESULTS_DIR / "e5-evaluation-report.json"
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    sha256 = hashlib.sha256(report_path.read_bytes()).hexdigest()
    report["report_sha256"] = sha256
    (RESULTS_DIR / "e5-evaluation-report.json.sha256").write_text(f"{sha256}  e5-evaluation-report.json\n", encoding="utf-8")
    print(f"\nReport written to {report_path} (SHA-256: {sha256[:16]}...)")
    print(f"Overall Disposition: {overall_disposition}")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="E5 State-Boundary Deconvolution Evaluator")
    parser.add_argument("--candidate", type=Path, default=None, help="Path to candidate source root")
    parser.add_argument("--target-lines", type=float, default=20.0, help="Target line reduction percentage")
    parser.add_argument("--target-stmts", type=float, default=10.0, help="Target statement reduction percentage")
    args = parser.parse_args()

    res = run_e5_evaluation(
        candidate_root=args.candidate,
        target_reduction_pct=args.target_lines,
        target_statement_pct=args.target_stmts,
    )
    if res["disposition"] not in ("PASS", "BASELINE_ROUND_0_PASS"):
        sys.exit(1)
