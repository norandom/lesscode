# Implementation Tasks: TypeScript / JavaScript Deconvolution Adapter

Status: **approved for foreground implementation**, 2026-11-09.
Governed by [prototype-first](../../steering/prototype-first.md), [R50](../../../docs/research/2026-11-09/08-typescript-adapter-prototype-gate.md), and [adapter-contract.md](../native-deconvolution-closure/adapter-contract.md).
Owner rules in effect: foreground TDD execution only (no subagents), Windows PowerShell only, one commit per completed task on `main`, R41 line ceiling $\le 7,173$, and the 4 preserved owner WIP files remain strictly untouched.

---

### Task 1: TypeScript AST & Effect Analyzer [x]
- **Requirements:** REQ-TS-1.1, REQ-TS-1.2, REQ-TS-1.3, REQ-TS-1.4
- **Dependencies:** None (builds on `native/src/adapter_traits.rs`)
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test typescript_analysis` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget, exit code 0).
- **Owned Files:**
  - `native/src/adapters/typescript/analysis.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/adapters/typescript/mod.rs` (new module root)
  - `native/src/adapters/mod.rs` (adapter registration)
  - `native/src/lib.rs` (register `pub mod adapters;`)
  - `native/complexity-budgets.json` (register modules)
  - `native/tests/typescript_analysis.rs` (new test suite)
- **Implementation:**
  1. Define `TypeScriptEffectAnalyzer`: parse declared TypeScript AST constructs, identify source-anchored write sites, property reassignments, and external function calls.
  2. Implement effect resolution: detect unsupported dynamic forms (proxies, `eval()`, dynamic imports, worker threads) and emit `CoverageGap` with source location.
  3. Validate observed mutations against frozen `FrameSpec`: verify allowed write sites and report violations.
  4. Register modules in `adapters/mod.rs`, `lib.rs`, and `complexity-budgets.json`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test typescript_analysis`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Delete `native/src/adapters/typescript/`, revert `adapters/mod.rs`, `lib.rs`, and `complexity-budgets.json`.

---

### Task 2: TypeScript Adverse Mutation Operators [ ]
- **Requirements:** REQ-TS-3.1, REQ-TS-3.2, REQ-TS-3.3
- **Dependencies:** Task 1
- **Owned Files:**
  - `native/src/adapters/typescript/mutation.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `mutation.rs`)
  - `native/tests/typescript_mutation.rs` (new test suite)
- **Implementation:**
  1. Define `TypeScriptMutantGenerator` implementing `AdverseMutantGenerator`.
  2. Implement supported mutation operators:
     - `SeverityInversion`: mutate severity values (e.g. `warning` -> `notice`, `blocked` -> `warning`).
     - `SlugifyDrop`: strip regex sanitization in slugify helpers.
     - `ActionOmission`: drop primary safe next action from failure responses.
     - `OrderInversion`: reverse error array sequences.
  3. Bind each operator to source span, SHA-256 hash, and independent expected behavioral fault.
  4. Compute complete denominator accounting ($K / N$).
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test typescript_mutation`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/typescript/mutation.rs` and `complexity-budgets.json`.

---

### Task 3: TypeScript Scope Metrics & Density Scorer [ ]
- **Requirements:** REQ-TS-5.1, REQ-TS-5.2
- **Dependencies:** Task 1
- **Owned Files:**
  - `native/src/adapters/typescript/metrics.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `metrics.rs`)
  - `native/tests/typescript_metrics.rs` (new test suite)
- **Implementation:**
  1. Implement TypeScript source metric extractors: physical lines, canonical lines (non-empty, non-comment), and AST statement counting.
  2. Implement whole-responsibility compression evaluation using `crate::density::evaluate_density_gate`:
     - $\ge 20\%$ canonical line reduction.
     - $\ge 10\%$ AST statement reduction.
     - $\ge 50$ canonical lines and $\ge 10$ statements removed.
  3. Ensure no exported complexity to caller files.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test typescript_metrics`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/typescript/metrics.rs` and `complexity-budgets.json`.

---

### Task 4: TypeScript Runtime Adapter Supervisor & IPC Protocol [ ]
- **Requirements:** REQ-TS-2.1 through REQ-TS-2.4, REQ-TS-4.1 through REQ-TS-4.4
- **Dependencies:** Tasks 1, 2, 3
- **Owned Files:**
  - `native/src/adapters/typescript/mod.rs` (implement 4 adapter traits)
  - `native/complexity-budgets.json` (register `mod.rs`)
  - `native/tests/typescript_adapter.rs` (runner integration test)
- **Implementation:**
  1. Implement `StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, and `SpecimenTraceRunner` for profile `ts-node-v1`.
  2. Implement subprocess runner executing `native/adapters/typescript/runner.mjs` with Node 24 and tsx loader.
  3. Parse line-delimited JSON IPC events (`start`, `state`, `case_failed`, `complete`).
  4. Implement TaggedValue conversion, preserving IEEE-754 double precision (NaN, Infinity, -0.0) and object/array structures.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test typescript_adapter`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/typescript/mod.rs` and `complexity-budgets.json`.

---

### Task 5: End-to-End Specimen Integration & Adverse Sensitivity Verification [ ]
- **Requirements:** REQ-TS-2.1, REQ-TS-3.2, REQ-TS-4.4, REQ-TS-5.4
- **Dependencies:** Task 4
- **Owned Files:**
  - `native/tests/typescript_e2e.rs` (end-to-end specimen verification)
  - `native/complexity-budgets.json`
- **Implementation:**
  1. Execute differential comparison against `OpenUltraCode` degradation responsibility:
     - 42 Cartesian finite domain test cases with **0 divergences**.
  2. Verify 100% kill rate on mandatory adverse controls (TS_ADV_1 through TS_ADV_4).
  3. Verify regression test preservation (`tests/degradation.test.ts` passes 3/3, `tsc --noEmit` exit 0).
  4. Verify witness shrinking and causal diagnosis formatting.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test typescript_e2e`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/tests/typescript_e2e.rs`.

---

### Task 6: Full System Verification, Budget Audit & GO Disposition [ ]
- **Requirements:** REQ-TS-5.3, REQ-TS-5.4, REQ-TS-5.5
- **Dependencies:** Tasks 1–5
- **Owned Files:**
  - `.kiro/specs/typescript-deconvolution-adapter/validation-report.md`
  - `.kiro/specs/typescript-deconvolution-adapter/spec.json`
  - `.kiro/task_21_handover.md`
- **Implementation:**
  1. Run complete native test suite via `cargo-nextest`: verify 100% passing across all binaries.
  2. Run complexity budget audit: verify all files comply with $\le 400$ lines, $\le 300$ syntax units, $\le 80$ lines/function, and total apparatus $\le 7,173$ canonical lines.
  3. Verify the 4 preserved owner WIP files remain strictly untouched.
  4. Author Feature Integration Validation report certifying **GO** disposition.
  5. Update `spec.json` to `phase: "completed"`, `validation_disposition: "GO"`.
  6. Author `task_21_handover.md`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
  - Python tests: `.runs/e2-facdrone/venv/Scripts/pytest.exe -n auto tests/test_state_boundary.py`
- **Rollback:** N/A (final certification task).
