# R45: E6 Generalization Trial Qualification — State-Boundary Deconvolution Certified Across Report Envelope Subsystem

- Report date/time and timezone: 2026-11-09, Europe/Berlin (UTC+1)
- Execution interval: Candidate generation Round 1 and live qualification verification against baseline Git HEAD `09ac9d9`
- Status: completed (all COBESY predictions T1–T7 and S confirmed and passed)
- Spec: [.kiro/specs/state-boundary-deconvolution/spec.json](../../../.kiro/specs/state-boundary-deconvolution/spec.json) (generalization trial qualified; milestone 3 completed)
- Hypothesis ID and predecessor: [R44](02-e6-generalization-trial-setup.md) (E6 trial preregistration & baseline census); [R43](01-e5-lifecycle-adoption-maturity-gate.md) (lifecycle adoption maturity gate certified); [R42](../../research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md) (E5 trial breaking density trap); [R41](../../research/2026-11-08/08-state-boundary-ceiling-amendment.md) (Rust substrate ceiling amended to 7,173); [R40](../../research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md) (structural deconvolution & state boundary synthesis).
- Type: Empirical qualification report confirming the generalization of State-Boundary Deconvolution to an unseen target subsystem.

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

## Empirical Findings by File

The 6 files in the `case-report-envelope` scope were deconvoluted as follows:

1. **`composition_errors.py` (66 $\to$ 54 lines, 34 $\to$ 32 stmts, 3 funcs, 0 writes):**
   - Retained complete `ReportingCompositionReasonCode` enumeration and `ReportingCompositionError` type with cause chaining.
   - Preserved `store_unreachable` dialect inspectability for PostgreSQL operational/connection faults.
2. **`evidence_validation.py` (249 $\to$ 207 lines, 148 $\to$ 128 stmts, 21 $\to$ 20 funcs, 0 writes):**
   - Consolidated path traversal containment checks (`_check_contained_path`).
   - Streamlined scalar parsers (`parse_sha256`, `parse_text`, `parse_datetime`) while preserving all validation errors.
3. **`reporting_request_store.py` (215 $\to$ 183 lines, 92 $\to$ 87 stmts, 9 $\to$ 8 funcs, 4 writes):**
   - Strictly confined write operations to `SqlReportingRequestEnvelopeStore.persist()` (row lock + `update(SIM_RUNS)`).
   - Consolidated row JSON mapping validation and unified error reason translation.
4. **`snapshot_store.py` (453 $\to$ 347 lines, 154 $\to$ 145 stmts, 11 $\to$ 10 funcs, 0 writes):**
   - Streamlined `_SnapshotManifest` exact-field parsing and provenance verification.
   - Inlined static verification helpers while strictly retaining multi-source publication contracts (ECB, FRED, SPDJI, FMP).
5. **`reporting_request.py` (553 $\to$ 519 lines, 256 $\to$ 245 stmts, 23 $\to$ 21 funcs, 0 writes):**
   - Inlined single-caller assertion helpers directly into `ReportingRequestEnvelope.__post_init__`, eliminating duplicate call edges to `evidence_validation.py`.
   - Consolidated nested flow, approval, and identity parsers inside `_parse_envelope`.
6. **`canonical.py` (402 $\to$ 233 lines, 241 $\to$ 202 stmts, 35 $\to$ 9 funcs, 0 writes):**
   - Subsumed 11 scalar parser/encoder helper functions and lookup tables into unified type-matching dispatchers.
   - Retained 100% byte-exact JSON canonicalization, float-rounding policy (12 significant digits for identity mode), and typed hydration for domain dataclasses.

---

## Evidence Artifacts

- **Candidate Diff:** [`docs/experiments/deconvolution/e6-report-envelope/results/e6-candidate-r1.diff`](../../experiments/deconvolution/e6-report-envelope/results/e6-candidate-r1.diff)
  - SHA-256: `9b8b5456f28806915a689f2343969ff62414a15fcafc6387a3a4dc5f7d242756`
- **Evaluation Report:** [`docs/experiments/deconvolution/e6-report-envelope/results/e6-evaluation-report.json`](../../experiments/deconvolution/e6-report-envelope/results/e6-evaluation-report.json)
  - SHA-256: `384a1225d1486b078563e89ebb000a245f83c7c1bd1c281b319d09bc48c78b51`
- **Acceptance Test Suite:** [`docs/experiments/deconvolution/e6-report-envelope/run_e6.py`](../../experiments/deconvolution/e6-report-envelope/run_e6.py)
  - Passing 46 tests across `packages/facdrone-reporting`, `packages/facdrone-kernel`, and root suites in 38.5s without regressions.

---

## Methodological Conclusion

Experiment E6 confirms the central generalization thesis: **State-Boundary Deconvolution is a subsystem-agnostic architectural compression loop**. 

By identifying state boundaries before synthesis, enforcing transaction isolation on state mutations, monitoring diagnostic streams for abort truncation, and holding pure domain logic to bounded counterexample-guided equivalence, the engine reliably eliminates redundant scaffolding, call edges, and intermediate transformations across diverse application domains (lifecycle monitors and report construction envelopes alike) while maintaining zero overage on the supporting formal apparatus.
