# Task group 16 handover: E6 generalization trial preregistration & baseline census (Report Envelope)

Written 2026-11-09. Group scope: establish and qualify the **Experiment E6: State-Boundary Deconvolution Generalization Trial** on an unseen multi-file Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 canonical lines), following owner selection of Option 1.
All execution, measurement, verification, and script creation ran in the foreground. No subagents were dispatched, per owner rule.

---

## What was done

1. **Target Subsystem Selection & Scope Freezing:**
   - Selected the **Report Envelope & Snapshot Store** group (`case-report-envelope.json` from CodeQL discovery run-02), spanning 6 files across kernel domain logic and reporting persistence shells:
     - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py` (553 lines, 256 stmts, 23 funcs, 0 writes)
     - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py` (215 lines, 92 stmts, 9 funcs, 4 writes)
     - `packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py` (453 lines, 154 stmts, 11 funcs, 0 writes)
     - `packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py` (249 lines, 148 stmts, 21 funcs, 0 writes)
     - `packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py` (66 lines, 34 stmts, 3 funcs, 0 writes)
     - `packages/facdrone-kernel/facdrone_kernel/canonical.py` (402 lines, 241 stmts, 35 funcs, 0 writes)
   - Created `docs/experiments/deconvolution/e6-report-envelope/scope.py` recording exact SHA-256 byte hashes and verifying 0 baseline discrepancies against the frozen specimen snapshot.

2. **Preregistered Plan with COBESY Falsifiable Framing:**
   - Authored `docs/experiments/deconvolution/e6-report-envelope/plan.md` locking numerical prediction targets before code changes:
     - **T2 (Lines):** $\le 1,550$ canonical lines ($\ge 20\%$ whole-responsibility compression).
     - **T3 (Statements):** $\le 832$ AST statements ($\ge 10\%$ reduction; Density Trap defense).
     - **T4 (Structural):** $\le 81$ functions ($\ge 20\%$) and $\le 108$ intra-scope call edges ($\ge 25\%$).
     - **T5 (Sensitivity):** 100% kill rate on adverse state mutants (E6-C1 to E6-C4).
     - **T6 (Purity):** 0 counterexamples across bounded domain $[-10, 10]$ and edge cases.
     - **T7 (Apparatus):** Rust substrate line volume $\le 7,173$ canonical lines (R41 no-growth cap).
     - **S (Contract):** Confined write sites ($\le 4$), pure share $\ge 50\%$, 0 replay divergences, 100% test pass.

3. **Measurement Apparatus & Adverse Controls Qualification:**
   - Implemented `docs/experiments/deconvolution/e6-report-envelope/e6_evaluator.py`:
     - 4/4 adverse controls passed (`[PASS] E6_C1_MissingFailureState`, `[PASS] E6_C2_BufferTruncation`, `[PASS] E6_C3_OutOfDomainPurity`, `[PASS] E6_C4_ErrorOrderInversion`).
     - Rust apparatus measured at 7,173 canonical lines (ceiling 7,173, overage 0).
     - Emitted baseline evaluation report at `results/e6-evaluation-report.json` with written SHA-256 sidecar.

4. **Acceptance Test Battery:**
   - Authored `docs/experiments/deconvolution/e6-report-envelope/run_e6.py` covering 46+ tests across reporting and root suites (`test_store_faults.py`, `test_record_simulation_request.py`, `test_export_on_request.py`, `test_factor_basket_request.py`, `test_cli_sim.py`, `test_legacy_run_read_path.py`).
   - Verified that the acceptance battery passes with exit 0 on the baseline snapshot.

5. **Research Log Synchronization:**
   - Authored `docs/research/2026-11-09/02-e6-generalization-trial-setup.md` (R44).
   - Updated `docs/research/README.md` to index R44 as Latest.

---

## Fresh Evidence

### 1. E6 Baseline Metrics (`measures.measure_written`)

| Metric | Measured Baseline | E6 Target Bar | Required Reduction |
|---|---:|---:|---:|
| Canonical lines | 1,938 | $\le 1,550$ | $\ge 20\%$ (−388 lines) |
| AST statements | 925 | $\le 832$ | $\ge 10\%$ (−93 statements) |
| Function count | 102 | $\le 81$ | $\ge 20\%$ (−21 functions) |
| Call edges | 145 | $\le 108$ | $\ge 25\%$ (−37 call edges) |
| State write sites | 4 | $\le 4$ | Imperative shell only |
| Adverse mutant kill rate | — | 100% | 4/4 controls killed |
| Rust apparatus lines | 7,173 | $\le 7,173$ | 0 overage |

### 2. Verification Harness Status

- **Adverse Controls:** 4/4 detected and killed (100% sensitivity).
- **Rust Apparatus Lines:** 7,173 canonical lines (ceiling 7,173, overage 0).
- **Acceptance Battery:** Reporting suite and root unit suites passed (exit 0).

---

## Untouched Owner WIP

All 4 preserved owner WIP files remain strictly unstaged and untouched:
- `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
- `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
- `src/lesscode_baseline/rules/evaluate.py`
- `tests/test_rule_evaluation.py`

---

## Current State and Next Steps

1. **Commit Task Group 16 Artifacts:** Stage and commit preregistration and baseline apparatus to `main`.
2. **Context Reset / Turn 1 Execution:** Clear conversation context per owner rule (6) or proceed with Candidate Generation (Turn 1).
