# R44: E6 State-Boundary Deconvolution Generalization Trial Preregistration & Baseline Census

- Report date/time and timezone: 2026-11-09, Europe/Berlin (UTC+1)
- Execution interval: Live baseline evaluation and preregistration on Git HEAD `5273a70`
- Status: completed (preregistration and baseline census established)
- Spec: [.kiro/specs/state-boundary-deconvolution/spec.json](../../../.kiro/specs/state-boundary-deconvolution/spec.json) (phase completed; generalization trial opened under roadmap milestone 3)
- Hypothesis ID and predecessor: [R43](01-e5-lifecycle-adoption-maturity-gate.md) (self-application maturity gate certified); [R42](../../research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md) (E5 trial breaking density trap); [R41](../../research/2026-11-08/08-state-boundary-ceiling-amendment.md) (Rust substrate ceiling amended to 7,173); [R40](../../research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md) (structural deconvolution & state boundary synthesis).
- Type: Preregistered trial plan and baseline qualification with COBESY falsifiable framing. Preregistered plan at `docs/experiments/deconvolution/e6-report-envelope/plan.md`.

---

## Decision

Following owner direction to proceed with **Option 1: Experiment E6 Generalization Trial on an Unseen Target Specimen**, the testbed, frozen scope, measurement harness, and adverse controls battery have been formally established and qualified at `docs/experiments/deconvolution/e6-report-envelope/`:
1. **Target Subsystem Selected:** Facdrone `report-envelope` group (`case-report-envelope.json` from CodeQL discovery run-02), spanning 6 files across kernel domain logic and reporting persistence shells:
   - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py` (553 lines, 256 stmts, 23 funcs, 0 writes)
   - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py` (215 lines, 92 stmts, 9 funcs, 4 writes)
   - `packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py` (453 lines, 154 stmts, 11 funcs, 0 writes)
   - `packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py` (249 lines, 148 stmts, 21 funcs, 0 writes)
   - `packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py` (66 lines, 34 stmts, 3 funcs, 0 writes)
   - `packages/facdrone-kernel/facdrone_kernel/canonical.py` (402 lines, 241 stmts, 35 funcs, 0 writes)
   - **Baseline Totals:** **1,938 canonical lines**, **925 AST statements**, **102 functions**, **145 intra-scope call edges**, and **4 write sites** (confined exclusively to `reporting_request_store.py`).
2. **Fixed Target Compression Bars (Predictions T1–T7, S):**
   - **T2 (Canonical Lines):** $\le 1,550$ lines ($\ge 20\%$ whole-responsibility reduction).
   - **T3 (AST Statements):** $\le 832$ statements ($\ge 10\%$ reduction; Density Trap defense).
   - **T4 (Structural De-fragmentation):** $\le 81$ functions ($\ge 20\%$) and $\le 108$ call edges ($\ge 25\%$).
   - **T5 (Adverse Mutation Kill Rate):** 100% kill rate on adverse state mutants (E6-C1 to E6-C4).
   - **T6 (Pure-Core Equivalence):** 0 counterexamples across bounded domains $[-10, 10]$ and edge cases.
   - **T7 (Apparatus Volume):** Counted Rust substrate held $\le 7,173$ canonical lines.
   - **S (Structural Contract):** Confined shell write sites ($\le 4$), pure share $\ge 50\%$, 0 replay divergences, 100% test pass on affected test suites.
3. **Apparatus & Adverse Controls Qualified:**
   - 4/4 adverse controls passed (`[PASS] E6_C1_MissingFailureState`, `[PASS] E6_C2_BufferTruncation`, `[PASS] E6_C3_OutOfDomainPurity`, `[PASS] E6_C4_ErrorOrderInversion`).
   - Rust apparatus measured at 7,173 canonical lines (0 overage against R41 ceiling).
   - Acceptance test battery (`run_e6.py`) passes 100% across reporting and root suites in 21s.

---

## Hypothesis and Falsification

- **Generalization Hypothesis:** The State-Boundary Deconvolution methodology is not overfitted to the `lifecycle` monitor group. When applied to an unseen multi-file subsystem (`report-envelope`), enforcing explicit state frames (restricting writes to 4 store methods), pure-core extraction (`canonical.py`), output buffer tracking, and adverse mutation gates will enable the deconvolution engine to break the Density Trap, achieving $\ge 20\%$ line compression ($\le 1,550$) and $\ge 10\%$ statement compression ($\le 832$) without oracle blindness.
- **Falsification Criteria:**
  - **T1:** Refinement fails to converge in $\le 3$ turns.
  - **T2:** Measured canonical lines $\ge 1,551$ ($<20\%$ reduction).
  - **T3:** AST statements $\ge 833$ ($<10\%$ reduction; Density Trap triggered).
  - **T4:** Functions $>81$ or intra-scope call edges $>108$.
  - **T5:** Any adverse control mutant survives without detection.
  - **T6:** Counterexample witness discovered on bounded pure-core domain.
  - **T7:** Rust apparatus line volume exceeds 7,173 canonical lines.
  - **S:** Write sites $>4$, pure core share $<50\%$, or regression in unit test suites.

---

## Frozen Inputs and Method

- **Specimen:** Facdrone `report-envelope` group (6 files, 1,938 canonical lines, 925 statements, 102 functions, 145 call edges, 4 write sites).
- **Harness Apparatus:**
  - Evaluator: `docs/experiments/deconvolution/e6-report-envelope/e6_evaluator.py`
  - Acceptance runner: `docs/experiments/deconvolution/e6-report-envelope/run_e6.py`
  - Baseline report: `docs/experiments/deconvolution/e6-report-envelope/results/e6-evaluation-report.json`
- **Environment:** Windows 11 PowerShell, CPython 3.13.16 pinned specimen venv at `.runs/e2-facdrone/venv/Scripts/python.exe` with `PYTHONPATH=src`. Preserved owner WIP files left untouched and unstaged.

---

## Tool/Spec Consequence and Next Gate

1. **Preregistration Locked:** Scope, metrics, predictions, and baseline hashes are locked in `scope.py` and `plan.md`.
2. **Next Steps:**
   - Execute Candidate Generation (Round 1) on the candidate tree.
   - Run verification battery via `e6_evaluator.py` and `run_e6.py`.
   - Update handover.
