# Implementation Plan: State Boundary Deconvolution

## Implementation Tasks

- [x] 1. Foundation: state frame condition contracts and stream buffer monitoring
  - [x] 1.1 State frame condition declaration and static write-site analysis
    - Implement `StateFrame` declaration parser and AST visitor in `src/lesscode_baseline/state_boundary/frame.py` to identify permitted global, instance, and environment write locations.
    - Statically inspect candidates to verify that all write sites reside strictly within declared shell functions.
    - Done when unauthorized write sites outside declared shell functions are detected and reported with line-anchored locations.
    - _Boundary: FrameChecker_
    - _Requirements: 1.1, 1.2, 1.5_
  - [x] 1.2 (P) Dynamic state observation and failure-side invariant evaluator
    - Implement dynamic state snapshotting before and after execution in `src/lesscode_baseline/state_boundary/frame.py`.
    - Evaluate post-condition state assertions on unhandled exceptions and controlled aborts, asserting that required failure-side state transitions (such as audit logs) are present.
    - Done when omitted failure-side state mutations (such as E3 `AuditedAbsence`) are flagged as invariant violations and rejected.
    - _Boundary: FrameChecker_
    - _Requirements: 1.3, 1.4, 1.5_
  - [x] 1.3 (P) Output stream and buffer preservation monitor
    - Implement stdout, stderr, and logging stream interception in `src/lesscode_baseline/state_boundary/stream_monitor.py` to capture emitted output bytes prior to abort points.
    - Enforce relative effect ordering constraints matching the original execution contract.
    - Done when dropped, un-flushed, or truncated intermediate output streams (such as E4 F3) are caught and reported as stream divergences.
    - _Boundary: StreamMonitor_
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 2. Core: pure core formal equivalence and adverse state mutation generator
  - [x] 2.1 Static purity verification for functional core
    - Implement static purity verification in `src/lesscode_baseline/state_boundary/pure_verifier.py` to ensure core functions have 0 state writes, 0 global lookups, and 0 external I/O invocations.
    - Done when any side-effect attempt or global state lookup inside a core function triggers a static purity rejection.
    - _Boundary: PureVerifier_
    - _Requirements: 2.1, 2.5_
  - [x] 2.2 (P) Bounded semantic equivalence checking and counterexample generation
    - Implement bounded property-based exploration in `src/lesscode_baseline/state_boundary/pure_verifier.py` comparing candidate pure core functions against original un-deconvoluted functions over bounded input domains.
    - Ensure both valid and out-of-domain input combinations are evaluated, automatically emitting minimal reproducible counterexamples upon output divergence.
    - Done when input/output discrepancies between original and candidate pure logic produce a structured counterexample witness.
    - _Boundary: PureVerifier_
    - _Requirements: 2.2, 2.3, 2.4_
  - [x] 2.3 Targeted adverse state mutation injector
    - Implement mutation generator in `src/lesscode_baseline/state_boundary/mutant_generator.py` targeting exception handlers (dropping failure state), output buffers (dropping flush), and error-evaluation order.
    - Classify execution outcomes into behavioral mutant kills, compilation errors, timeouts, and equivalent mutations.
    - Done when planted state-boundary mutants are generated and categorized with explicit target anchors.
    - _Boundary: StateMutantHarness_
    - _Requirements: 3.1, 3.4_

- [x] 3. Integration: mutation sensitivity gates, causal refinement bridge, and density defense
  - [x] 3.1 Adverse state mutation sensitivity gate
    - Wire mutant evaluation into the verification gate, requiring candidate test batteries to kill 100% of non-equivalent planted mutants.
    - Enforce mandatory kills for exception-side state mutations and buffer flushes before granting verification qualification.
    - Done when any candidate whose test suite allows a non-equivalent adverse state mutant to survive is denied acceptance.
    - _Boundary: StateMutantHarness_
    - _Depends: 2.3_
    - _Requirements: 3.2, 3.3, 3.5_
  - [x] 3.2 Refinement controller causal state obligation bridge
    - Extend `lesscode-core::regenerator` and `src/lesscode_baseline/regenerator.py` to integrate state failure kinds (`StateFrameViolation`, `PureCoreDivergence`, `StreamDivergence`).
    - Format failing state witnesses into structured diagnostic feedback for subsequent turn prompts, retaining unresolved obligations across turns.
    - Done when a state violation discovered on turn 1 is injected into the turn 2 prompt and retained in `unresolved_obligations` until re-verified.
    - _Boundary: ObligationBridge_
    - _Depends: 1.2, 1.3, 2.2_
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - [x] 3.3 (P) Whole-responsibility density scorer and outcome gate
    - Implement scope-wide measurement in `src/lesscode_baseline/state_boundary/density.py` and `native/src/state_contract.rs` measuring functions, call edges, write sites, AST statements, and canonical lines across the containing scope.
    - Explicitly distinguish `structurally_consolidated` from `compression_met`, preventing function consolidation without net statement/line reduction from claiming compression.
    - Done when candidate reports separate structural de-fragmentation from net compression, failing `compression_met` if statements merely compacted into denser functions.
    - _Boundary: DensityGate_
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 4. Validation: prototype feasibility gate and qualification battery
  - [x] 4.1 Adversarial state-boundary unit and integration test suite
    - Implement `tests/test_state_boundary.py` covering frame condition leaks, failure-side state omissions, truncated output streams, and surviving mutant rejections.
    - Done when the test suite passes offline with 100% detection of simulated state-boundary defects.
    - _Boundary: FrameChecker, StreamMonitor, StateMutantHarness_
    - _Requirements: 1.2, 1.3, 3.3, 4.2_
  - [x] 4.2 End-to-end prototype feasibility gate execution
    - Execute the feasibility gate against the Facdrone `lifecycle` scope and adverse controls (E3 `AuditedAbsence`, E4 F1/F3/F7) per `prototype-gate.md`.
    - Verify that production apparatus line volume strictly complies with the R38 7,004 canonical line ceiling.
    - Done when all adverse controls are killed, state post-conditions are verified, and the gate report is recorded with explicit line accounting.
    - _Boundary: ObligationBridge, DensityGate_
    - _Depends: 3.1, 3.2, 3.3, 4.1_
    - _Requirements: 3.5, 5.5, 6.1, 6.2, 6.5_

---

## Implementation Notes

Specification implementation completed, qualified, adopted, and validated on 2026-11-09:
- **Tasks 1.1–1.3 (Foundation):** State frame conditions, dynamic invariants, and stream monitoring implemented in `src/lesscode_baseline/state_boundary/{frame.py, stream_monitor.py}`.
- **Tasks 2.1–2.3 (Core):** Static purity checking, bounded property equivalence over $[-10, 10]$, and targeted adverse state mutations implemented in `pure_verifier.py` and `mutant_generator.py`.
- **Tasks 3.1–3.3 (Integration):** Adverse sensitivity gate, causal state obligation tracking across turns, and whole-responsibility density scoring implemented in `density.py` and `native/src/state_contract.rs`.
- **Tasks 4.1–4.2 (Validation):** 32-test adversarial unit suite in `tests/test_state_boundary.py` and prototype feasibility gate passed under amended Rust apparatus ceiling of 7,173 canonical lines (R41).
- **Trial & Adoption Milestones:**
  - Experiment E5 (Lifecycle): Line count −24.8%, statements −12.1%, functions −27.5%, call edges −31.5%, 11 write sites, 50/50 sequences replayed with 0 Level 2 divergences. Formally adopted into `.runs/e5-adopted-lifecycle/` (R42, R43).
  - Experiment E6 (Report Envelope Generalization): Line count −23.01%, statements −10.49%, functions −38.24%, call edges −30.34%, 4 write sites, 4/4 adverse mutants killed. Formally adopted into `.runs/e6-adopted-report-envelope/` (R44, R45, R46).
- **Milestone Synthesis Report:** Published in [`docs/reports/state-boundary-deconvolution-report.md`](../../reports/state-boundary-deconvolution-report.md) following the COBESY research structure.
- **Feature Integration Validation:** Mechanical suite (241 nextest, 32 pytest, E5/E6 acceptance suites) green, 0 TODO/secrets, clean runtime liveness, and final verdict signed off as **GO** in [`validation-report.md`](validation-report.md).
