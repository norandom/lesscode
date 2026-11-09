# Implementation Tasks: Rust Deconvolution Adapter

Status: **approved for foreground implementation**, 2026-11-09.
Governed by [prototype-first](../../steering/prototype-first.md), [R52](../../../docs/research/2026-11-09/10-rust-adapter-prototype-gate.md), and [adapter-contract.md](../native-deconvolution-closure/adapter-contract.md).
Owner rules in effect: foreground TDD execution only (no subagents), Windows PowerShell only, one commit per completed task on `main`, R41 line ceiling $\le 7,173$, and the 4 preserved owner WIP files remain strictly untouched.

---

### Task 1: Rust AST & Dynamic Gap Analyzer [x]
- **Requirements:** REQ-RUST-1.1, REQ-RUST-1.2, REQ-RUST-1.3, REQ-RUST-1.4
- **Dependencies:** None (builds on `native/src/adapter_traits.rs`)
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test rust_analysis` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/rust/analysis.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/adapters/rust/mod.rs` (new module root)
  - `native/src/adapters/mod.rs` (adapter registration: `pub mod rust;`)
  - `native/complexity-budgets.json` (register modules)
  - `native/tests/rust_analysis.rs` (new test suite)
- **Implementation:**
  1. Define `RustEffectAnalyzer` using `syn`: parse declared Rust AST, identify source-anchored write sites, field mutations, dereference reassignments, and external function calls.
  2. Implement effect resolution: detect unsupported dynamic forms (unresolved trait dispatch, `unsafe` blocks, raw pointers, interior mutability `Cell`/`RefCell`/`Mutex`/`RwLock` without explicit state roots, unmonitored thread dispatch) and emit `CoverageGap` with source location.
  3. Validate observed mutations against frozen `FrameSpec`: verify allowed field write sites and report violations.
  4. Register modules in `adapters/mod.rs` and `complexity-budgets.json`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_analysis`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Delete `native/src/adapters/rust/`, revert `adapters/mod.rs` and `complexity-budgets.json`.

---

### Task 2: Rust Adverse Mutation Operators [x]
- **Requirements:** REQ-RUST-3.1, REQ-RUST-3.2, REQ-RUST-3.3
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test rust_mutation` (5/5 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/rust/mutation.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `mutation.rs`)
  - `native/tests/rust_mutation.rs` (new test suite)
- **Implementation:**
  1. Define `RustMutantGenerator` implementing `AdverseMutantGenerator`.
  2. Implement supported mutation operators:
     - `DeadlineKillOmission`: omit process tree termination on timeout (`taskkill` omitted).
     - `InvertedVerdictCondition`: invert refusal condition in `verdict` (`arm_exit != 0` -> `arm_exit == 0`).
     - `InvertedErrorClassification`: bypass level2 differences check without query evidence.
     - `DroppedEnvironmentIsolation`: drop environment key from `arm_env` (`PYTHONPATH` omitted).
  3. Bind each operator to source span, SHA-256 hash, and independent expected behavioral fault.
  4. Compute complete denominator accounting ($K / N$).
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_mutation`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/rust/mutation.rs` and `complexity-budgets.json`.

---

### Task 3: Rust Scope Metrics & Density Scorer [x]
- **Requirements:** REQ-RUST-5.1, REQ-RUST-5.2
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test rust_metrics` (5/5 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/rust/metrics.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `metrics.rs`)
  - `native/tests/rust_metrics.rs` (new test suite)
- **Implementation:**
  1. Implement Rust source metric extractors: physical lines, canonical lines (non-empty, non-comment), and AST statement counting per `rust-statements-v1` using `syn`.
  2. Implement whole-responsibility compression evaluation using `crate::density::evaluate_density_gate`:
     - $\ge 20\%$ canonical line reduction.
     - $\ge 10\%$ AST statement reduction.
     - $\ge 50$ canonical lines and $\ge 10$ statements removed.
  3. Ensure no exported complexity to caller files.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_metrics`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/rust/metrics.rs` and `complexity-budgets.json`.

---

### Task 4: Rust Runtime Adapter Supervisor & IPC Protocol [ ]
- **Requirements:** REQ-RUST-2.1 through REQ-RUST-2.4, REQ-RUST-4.1 through REQ-RUST-4.4
- **Dependencies:** Tasks 1, 2, 3
- **Owned Files:**
  - `native/src/adapters/rust/mod.rs` (implement 4 adapter traits)
  - `native/complexity-budgets.json` (register `mod.rs`)
  - `native/tests/rust_adapter.rs` (runner integration test)
- **Implementation:**
  1. Implement `StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, and `SpecimenTraceRunner` for profile `rust-nextest-v1`.
  2. Implement subprocess runner executing cargo / nextest or compiled probe.
  3. Parse line-delimited JSON IPC events (`start`, `state`, `case_failed`, `complete`).
  4. Implement TaggedValue conversion, preserving IEEE-754 double precision (NaN, Infinity, -0.0), UTF-8 text strings, and object/array structures.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_adapter`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/rust/mod.rs` and `complexity-budgets.json`.

---

### Task 5: End-to-End Specimen Integration & Adverse Sensitivity Verification [ ]
- **Requirements:** REQ-RUST-2.1, REQ-RUST-3.2, REQ-RUST-4.4, REQ-RUST-5.4
- **Dependencies:** Task 4
- **Owned Files:**
  - `native/tests/rust_e2e.rs` (end-to-end specimen verification)
  - `native/complexity-budgets.json`
- **Implementation:**
  1. Execute differential comparison against native process-lifecycle responsibility:
     - 100 Cartesian finite domain test cases with **0 divergences**.
  2. Verify 100% kill rate on mandatory adverse controls (`RUST_ADV_1` through `RUST_ADV_4`).
  3. Verify regression test preservation (Nextest battery 314 passed, 0 failures).
  4. Verify witness shrinking and causal diagnosis formatting.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_e2e`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/tests/rust_e2e.rs`.

---

### Task 6: Full System Verification, Budget Audit & GO Disposition [ ]
- **Requirements:** REQ-RUST-5.3, REQ-RUST-5.4, REQ-RUST-5.5
- **Dependencies:** Tasks 1–5
- **Owned Files:**
  - `.kiro/specs/rust-deconvolution-unification/validation-report.md`
  - `.kiro/specs/rust-deconvolution-unification/spec.json`
  - `.kiro/task_23_handover.md`
- **Implementation:**
  1. Run complete native test suite via `cargo-nextest`: verify 100% passing across all binaries.
  2. Run complexity budget audit: verify all files comply with $\le 400$ lines, $\le 300$ syntax units, $\le 80$ lines/function, and total apparatus $\le 7,173$ canonical lines.
  3. Verify the 4 preserved owner WIP files remain strictly untouched.
  4. Author Feature Integration Validation report certifying **GO** disposition.
  5. Update `spec.json` to `phase: "completed"`, `validation_disposition: "GO"`.
  6. Author `task_23_handover.md`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
  - Python tests: `.runs/e2-facdrone/venv/Scripts/pytest.exe -n auto tests/test_state_boundary.py`
- **Rollback:** N/A (final certification task).
