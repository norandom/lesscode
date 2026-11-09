# Implementation Tasks: Java Deconvolution Adapter

Status: **approved for foreground implementation**, 2026-11-09.
Governed by [prototype-first](../../steering/prototype-first.md), [R51](../../../docs/research/2026-11-09/09-java-adapter-prototype-gate.md), and [adapter-contract.md](../native-deconvolution-closure/adapter-contract.md).
Owner rules in effect: foreground TDD execution only (no subagents), Windows PowerShell only, one commit per completed task on `main`, R41 line ceiling $\le 7,173$, and the 4 preserved owner WIP files remain strictly untouched.

---

### Task 1: Java AST & Class Effect Analyzer [x]
- **Requirements:** REQ-JAVA-1.1, REQ-JAVA-1.2, REQ-JAVA-1.3, REQ-JAVA-1.4
- **Dependencies:** None (builds on `native/src/adapter_traits.rs`)
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test java_analysis` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget, exit code 0).
- **Owned Files:**
  - `native/src/adapters/java/analysis.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/adapters/java/mod.rs` (new module root)
  - `native/src/adapters/mod.rs` (adapter registration)
  - `native/complexity-budgets.json` (register modules)
  - `native/tests/java_analysis.rs` (new test suite)
- **Implementation:**
  1. Define `JavaEffectAnalyzer`: parse declared Java constructs, identify source-anchored write sites, field reassignments, synchronized locks, and external function calls.
  2. Implement effect resolution: detect unsupported dynamic forms (reflection, JNI/native methods, dynamic classloading, unsynchronized thread dispatch) and emit `CoverageGap` with source location.
  3. Validate observed mutations against frozen `FrameSpec`: verify allowed field write sites and report violations.
  4. Register modules in `adapters/mod.rs` and `complexity-budgets.json`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test java_analysis`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Delete `native/src/adapters/java/`, revert `adapters/mod.rs` and `complexity-budgets.json`.

---

### Task 2: Java Adverse Mutation Operators [x]
- **Requirements:** REQ-JAVA-3.1, REQ-JAVA-3.2, REQ-JAVA-3.3
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test java_mutation` (5/5 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/java/mutation.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `mutation.rs`)
  - `native/tests/java_mutation.rs` (new test suite)
- **Implementation:**
  1. Define `JavaMutantGenerator` implementing `AdverseMutantGenerator`.
  2. Implement supported mutation operators:
     - `StateMutationOmission`: omit state mutation flag reset (e.g. `newRecord = false`).
     - `LockCleanupOmission`: omit lock unlock in finally block (e.g. `lock.unlock()`).
     - `DelimiterSequencingInversion`: invert delimiter printing sequence relative to value.
     - `DroppedStreamFlush`: omit buffer flush call in flush/close methods.
  3. Bind each operator to source span, SHA-256 hash, and independent expected behavioral fault.
  4. Compute complete denominator accounting ($K / N$).
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test java_mutation`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/java/mutation.rs` and `complexity-budgets.json`.

---

### Task 3: Java Scope Metrics & Density Scorer [x]
- **Requirements:** REQ-JAVA-5.1, REQ-JAVA-5.2
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test java_metrics` (5/5 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/java/metrics.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/complexity-budgets.json` (register `metrics.rs`)
  - `native/tests/java_metrics.rs` (new test suite)
- **Implementation:**
  1. Implement Java source metric extractors: physical lines, canonical lines (non-empty, non-comment), and AST statement counting per `java-statements-v1`.
  2. Implement whole-responsibility compression evaluation using `crate::density::evaluate_density_gate`:
     - $\ge 20\%$ canonical line reduction.
     - $\ge 10\%$ AST statement reduction.
     - $\ge 50$ canonical lines and $\ge 10$ statements removed.
  3. Ensure no exported complexity to caller files.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test java_metrics`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/java/metrics.rs` and `complexity-budgets.json`.

---

### Task 4: Java Runtime Adapter Supervisor & IPC Protocol [x]
- **Requirements:** REQ-JAVA-2.1 through REQ-JAVA-2.4, REQ-JAVA-4.1 through REQ-JAVA-4.4
- **Dependencies:** Tasks 1, 2, 3
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test java_adapter` (6/6 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/src/adapters/java/mod.rs` (implement 4 adapter traits)
  - `native/complexity-budgets.json` (register `mod.rs`)
  - `native/tests/java_adapter.rs` (runner integration test)
- **Implementation:**
  1. Implement `StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, and `SpecimenTraceRunner` for profile `java-junit-v1`.
  2. Implement subprocess runner executing `java` with `native/adapters/java/ProbeMain.java` or compiled probe with Maven classpath.
  3. Parse line-delimited JSON IPC events (`start`, `state`, `case_failed`, `complete`).
  4. Implement TaggedValue conversion, preserving IEEE-754 double precision (NaN, Infinity, -0.0), UTF-8 text strings, and object/array structures.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test java_adapter`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapters/java/mod.rs` and `complexity-budgets.json`.

---

### Task 5: End-to-End Specimen Integration & Adverse Sensitivity Verification [x]
- **Requirements:** REQ-JAVA-2.1, REQ-JAVA-3.2, REQ-JAVA-4.4, REQ-JAVA-5.4
- **Dependencies:** Task 4
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test java_e2e` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets --config native/complexity-budgets.json --root native/src --out ...` (clean, exit code 0).
- **Owned Files:**
  - `native/tests/java_e2e.rs` (end-to-end specimen verification)
  - `native/complexity-budgets.json`
- **Implementation:**
  1. Execute differential comparison against `apache/commons-csv` `CSVPrinter` responsibility:
     - 100 Cartesian finite domain test cases with **0 divergences**.
  2. Verify 100% kill rate on mandatory adverse controls (`JAVA_ADV_1` through `JAVA_ADV_4`).
  3. Verify regression test preservation (`CSVPrinterTest` passes 168 tests, 0 failures, 0 errors, 7 skipped).
  4. Verify witness shrinking and causal diagnosis formatting.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test java_e2e`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/tests/java_e2e.rs`.

---

### Task 6: Full System Verification, Budget Audit & GO Disposition [x]
- **Requirements:** REQ-JAVA-5.3, REQ-JAVA-5.4, REQ-JAVA-5.5
- **Dependencies:** Tasks 1–5
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml` (314/314 passed across 28 binaries in 105.67s), `pytest -n auto tests/test_state_boundary.py` (32/32 passed in 1.53s), `cargo test --test budgets` (10/10 passed), and CLI budget audit clean. Validation report authored certifying GO disposition.
- **Owned Files:**
  - `.kiro/specs/java-deconvolution-unification/validation-report.md`
  - `.kiro/specs/java-deconvolution-unification/spec.json`
  - `.kiro/task_22_handover.md`
- **Implementation:**
  1. Run complete native test suite via `cargo-nextest`: verify 100% passing across all binaries.
  2. Run complexity budget audit: verify all files comply with $\le 400$ lines, $\le 300$ syntax units, $\le 80$ lines/function, and total apparatus $\le 7,173$ canonical lines.
  3. Verify the 4 preserved owner WIP files remain strictly untouched.
  4. Author Feature Integration Validation report certifying **GO** disposition.
  5. Update `spec.json` to `phase: "completed"`, `validation_disposition: "GO"`.
  6. Author `task_22_handover.md`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
  - Python tests: `.runs/e2-facdrone/venv/Scripts/pytest.exe -n auto tests/test_state_boundary.py`
- **Rollback:** N/A (final certification task).
