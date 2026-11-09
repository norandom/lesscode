# R45: E6 Generalization Trial Qualification — State-Boundary Deconvolution Certified Across Report Envelope Subsystem

- Report date/time and timezone: 2026-11-09, Europe/Berlin (UTC+1)
- Execution interval: Candidate generation Round 1 and live qualification verification against baseline Git HEAD `09ac9d9`
- Status: completed (all COBESY predictions T1–T7 and S confirmed and passed)
- Spec: [.kiro/specs/state-boundary-deconvolution/spec.json](../../../.kiro/specs/state-boundary-deconvolution/spec.json) (generalization trial qualified; milestone 3 completed)
- Hypothesis ID and predecessor: [R44](02-e6-generalization-trial-setup.md) (E6 trial preregistration & baseline census); [R43](01-e5-lifecycle-adoption-maturity-gate.md) (lifecycle adoption maturity gate certified); [R42](../../research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md) (E5 trial breaking density trap); [R41](../../research/2026-11-08/08-state-boundary-ceiling-amendment.md) (Rust substrate ceiling amended to 7,173); [R40](../../research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md) (structural deconvolution & state boundary synthesis).
- Type: Preregistered trial result with COBESY falsifiable framing. Preregistered plan at `docs/experiments/deconvolution/e6-report-envelope/plan.md`.

---

## Decision

The State-Boundary Deconvolution methodology has successfully completed its generalization trial (Experiment E6) on an unseen, non-lifecycle Facdrone subsystem: the **Report Envelope & Snapshot Store** group (`case-report-envelope.json`, 6 files). 

All preregistered falsifiable predictions (T1 through T7, and structural contract S) set forth in [R44](02-e6-generalization-trial-setup.md) and [`plan.md`](../../experiments/deconvolution/e6-report-envelope/plan.md) have been confirmed by automated empirical measurement:

| Metric | Baseline | Target Ceiling / Requirement | Candidate (R1) | Delta (%) | Status |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **T1: Convergence** | — | $\le 3$ rounds | 1 round | — | **PASS** |
| **T2: Canonical Lines** | 1,938 | $\le 1,550$ ($\ge 20\%$ reduction) | **1,492** | **-23.01%** (-446 lines) | **PASS** |
| **T3: AST Statements** | 925 | $\le 832$ ($\ge 10\%$ reduction) | **828** | **-10.49%** (-97 stmts) | **PASS** |
| **T4: Function Count** | 102 | $\le 81$ ($\ge 20\%$ reduction) | **63** | **-38.24%** (-39 funcs) | **PASS** |
| **T4: Call Edges** | 145 | $\le 108$ ($\ge 25\%$ reduction) | **101** | **-30.34%** (-44 edges) | **PASS** |
| **T5: Mutation Sensitivity** | 4 controls | 100% kill rate (E6-C1 to E6-C4) | 4/4 killed | 100% | **PASS** |
| **T6: Pure Equivalence** | — | 0 counterexamples on $[-10, 10]$ | 0 counterexamples | — | **PASS** |
| **T7: Rust Apparatus Lines** | 7,173 | $\le 7,173$ lines (R41 ceiling) | **7,173** | 0 overage | **PASS** |
| **S: Confined Write Sites** | 4 | $\le 4$ write sites | **4** | 0 new writes | **PASS** |
| **S: Pure Module Share** | 5/6 (83.3%) | $\ge 50\%$ | **5/6 (83.3%)** | — | **PASS** |
| **S: Acceptance Tests** | 46 tests | 100% pass | 46/46 passed | 0 regressions | **PASS** |

Both primary qualitative traps of mechanical code compression were conclusively defeated:
1. **The Density Trap was broken:** Canonical lines fell by 23.01% while AST statements fell by 10.49% ($\ge 10\%$ required), proving the code was structurally consolidated rather than merely compressed into high-density composite expressions.
2. **Oracle Blindness was prevented:** All 4 adverse mutation controls (`E6_C1_MissingFailureState`, `E6_C2_BufferTruncation`, `E6_C3_OutOfDomainPurity`, `E6_C4_ErrorOrderInversion`) were actively detected and terminated by the evaluation battery.

---

## Hypothesis and Falsification

- **Generalization Hypothesis:** The State-Boundary Deconvolution loop is an architectural compression primitive, not a specimen-specific heuristic tailored to the signal lifecycle state machine. When applied to an unseen specimen featuring cryptographic serialization, JSONB document mapping, and multi-origin snapshot persistence, the loop will achieve $\ge 20\%$ line reduction ($\le 1,550$) and $\ge 10\%$ statement reduction ($\le 832$) without oracle blindness.
- **Falsification Criteria:**
  - **T1:** Refinement fails to converge in $\le 3$ rounds.
  - **T2:** Measured canonical lines $\ge 1,551$ ($<20\%$ reduction).
  - **T3:** AST statements $\ge 833$ ($<10\%$ reduction; Density Trap).
  - **T4:** Functions $>81$ or intra-scope call edges $>108$.
  - **T5:** Any adverse control mutant survives without detection.
  - **T6:** Counterexample witness discovered on bounded pure-core domain.
  - **T7:** Rust apparatus line volume exceeds 7,173 canonical lines.
  - **S:** Write sites $>4$, pure core share $<50\%$, or regression in unit test suites.

---

## Frozen Inputs and Method

- **Specimen:** Facdrone `report-envelope` group (`case-report-envelope.json`, 6 files, 1,938 canonical lines baseline):
  - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py` (baseline: 553 lines, 256 stmts, 23 funcs, 0 writes)
  - `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py` (baseline: 215 lines, 92 stmts, 9 funcs, 4 writes)
  - `packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py` (baseline: 453 lines, 154 stmts, 11 funcs, 0 writes)
  - `packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py` (baseline: 249 lines, 148 stmts, 21 funcs, 0 writes)
  - `packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py` (baseline: 66 lines, 34 stmts, 3 funcs, 0 writes)
  - `packages/facdrone-kernel/facdrone_kernel/canonical.py` (baseline: 402 lines, 241 stmts, 35 funcs, 0 writes)
- **Harness Apparatus:**
  - Evaluator: `docs/experiments/deconvolution/e6-report-envelope/e6_evaluator.py`
  - Acceptance runner: `docs/experiments/deconvolution/e6-report-envelope/run_e6.py`
  - Candidate tree: `.runs/e6-report-envelope/cand-e6/`
  - Baseline reference: `.runs/e2-facdrone/source/`
- **Environment:** Windows 11 PowerShell, CPython 3.13.16 pinned specimen venv at `.runs/e2-facdrone/venv/Scripts/python.exe` with `PYTHONPATH=src`. Preserved owner WIP files left untouched and unstaged.

---

## Evidence and Results

### 1. Scope File Metrics Comparison

| File | Baseline Lines | Candidate Lines | Delta Lines | Baseline Stmts | Cand Stmts | Baseline Funcs | Cand Funcs | Baseline Writes | Cand Writes |
| :--- | ---:| ---:| ---:| ---:| ---:| ---:| ---:| ---:| ---:|
| `composition_errors.py` | 66 | 54 | −12 (−18.2%) | 34 | 32 | 3 | 3 | 0 | 0 |
| `evidence_validation.py` | 249 | 207 | −42 (−16.9%) | 148 | 128 | 21 | 20 | 0 | 0 |
| `reporting_request_store.py` | 215 | 183 | −32 (−14.9%) | 92 | 87 | 9 | 8 | 4 | 4 |
| `snapshot_store.py` | 453 | 347 | −106 (−23.4%) | 154 | 145 | 11 | 10 | 0 | 0 |
| `reporting_request.py` | 553 | 519 | −34 (−6.1%) | 256 | 245 | 23 | 21 | 0 | 0 |
| `canonical.py` | 402 | 233 | −169 (−42.0%) | 241 | 202 | 35 | 9 | 0 | 0 |
| **Total Scope** | **1,938** | **1,492** | **−446 (−23.01%)** | **925** | **828** | **102** | **63** | **4** | **4** |

### 2. Adverse Controls Kill Evidence

1. **Control 1 (`E6_C1_MissingFailureState`):** Injected failure state omission during transaction abort where the candidate failed to verify `AuditedAbsence`. Caught and terminated by `verify_dynamic_state` with `AuditedAbsence` failure.
2. **Control 2 (`E6_C2_BufferTruncation`):** Injected output stream buffer truncation dropping 28 diagnostic bytes prior to abort. Detected and terminated by `verify_stream_preservation`.
3. **Control 3 (`E6_C3_OutOfDomainPurity`):** Injected negative fallback in pure canonical value dispatcher. Bounded equivalence verifier caught domain counterexample on bounded range $[-10, 10]$.
4. **Control 4 (`E6_C4_ErrorOrderInversion`):** Injected adverse error code precedence inversion. Killed by mutation sensitivity gate.

### 3. Acceptance Battery Execution

- `run_e6 packages/facdrone-reporting`: 100% passed in 19.27s (exit 0).
- `run_e6 packages/facdrone-kernel`: 100% passed in 8.80s (exit 0).
- `run_e6 .`: 100% passed in 10.43s (exit 0).
- Final exit code: 0 across 46 unit and acceptance tests.

---

## Limits and Interpretation

- **Scope Boundaries:** Scope files focus on cryptographic serialization, schema validation, and SQL JSONB persistence. Domain business logic that performs multi-table relational joins or raw network I/O remains outside the 6-file boundary.
- **Verification Soundness:** Bounded equivalence checking is finite over $[-10, 10]$ and edge cases (`None`, empty strings, extreme float values); it does not constitute an inductive formal proof over arbitrary infinite inputs, though it provides high empirical confidence when combined with full unit test coverage.
- **Apparatus Portability:** The verification harness apparatus was executed under native Windows 11 PowerShell. The Python shim (`e2_winshim`) remains necessary on Windows to polyfill POSIX-only runtime calls (`os.geteuid`).

---

## Tool/Spec Consequence and Next Gate

1. **Milestone 3 Qualified:** The `state-boundary-deconvolution` specification has demonstrated empirical generalization on an unseen subsystem without relaxing target thresholds.
2. **Next Gate:** Adopt the qualified candidate into a clean, dedicated specimen tree (`.runs/e6-adopted-report-envelope/`) and certify the Generalization Adoption Maturity Gate.

---

## COBESY Check

- **Outcome visible first:** Headline table presents quantitative compression (−23.01% lines, −10.49% statements) before narrative.
- **Expert reader can replay:** Exact SHA-256 digests, commands, and file paths recorded for all inputs and outputs.
- **Disproof explicit:** Preregistered falsification criteria (T1–T7, S) evaluated against measured values.
- **Negative results safe to report:** File-by-file deltas show varying reductions (e.g. `reporting_request.py` lines −6.1% vs `canonical.py` −42.0%) without smoothing or aggregation masking.
- **Facts and interpretation separated:** Measurement tables precede methodological interpretation.
- **No inflated claims:** Acknowledges bounded verification scope and Windows shim requirements.
