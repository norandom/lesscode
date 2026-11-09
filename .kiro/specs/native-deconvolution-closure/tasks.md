# Implementation Tasks: Native Deconvolution Closure

Status: **approved for foreground implementation**, 2026-11-09.
Governed by [prototype-first](../../steering/prototype-first.md), [R49](../../../docs/research/2026-11-09/07-native-closure-prototype-gate.md), and [adapter-contract.md](adapter-contract.md).
Owner rules in effect: foreground TDD execution only (no subagents), Windows PowerShell only, one commit per completed task on `main`, R41 line ceiling $\le 7,173$, and the 4 preserved owner WIP files remain strictly untouched.

---

### Task 1: Common Adapter Traits & Wire Contracts [x]
- **Requirements:** REQ-NAT-1.1, REQ-NAT-3.1, REQ-NAT-5.1, REQ-NAT-6.3
- **Dependencies:** None (foundation module)
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test adapter_contract` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget).
- **Owned Files:**
  - `native/src/adapter_traits.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/lib.rs` (registration only, $+1$ line)
  - `native/complexity-budgets.json` (register `adapter_traits.rs`)
  - `native/tests/adapter_contract.rs` (new test suite)
- **Implementation:**
  1. Define core result enums: `VerificationResult` (Verified, Divergent, Unverified, ExecutionFailed), `TaggedValue` (Int64, UInt64, FloatBits, Bytes, String, Bool, Null, Array, Object), `CoverageGap`, `ExecutionFailure`.
  2. Define four core traits per normative contract: `StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, `SpecimenTraceRunner`.
  3. Define request/response carrier types: `AnalysisRequest`, `AnalysisResult`, `ComparisonRequest`, `DomainSpec`, `MutationRequest`, `MutationBatch`, `RunRequest`, `RunResult`.
  4. Register module in `lib.rs` and `complexity-budgets.json`.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test adapter_contract`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/adapter_traits.rs`, remove `pub mod adapter_traits;` from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 2: Native State Frame Analyzer & Density Engine [x]
- **Requirements:** REQ-NAT-1.1 through REQ-NAT-1.5, REQ-NAT-2.1 through REQ-NAT-2.5
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test state_frame` (5/5 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget).
- **Owned Files:**
  - `native/src/state_frame.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/density.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/lib.rs` (registration only, $+2$ lines)
  - `native/complexity-budgets.json` (register `state_frame.rs`, `density.rs`)
  - `native/tests/state_frame.rs` (new test suite)
- **Implementation:**
  1. Implement `StateFrameAnalyzer` in `state_frame.rs`: parse declared scope write sites, resolve effect profiles, evaluate frame specifications against baseline/candidate state dictionaries.
  2. Implement source-anchored violation reporting for unauthorized state writes, global mutations, and broken failure invariants.
  3. Implement `density.rs`: integrate with existing `measures.rs` to compute physical lines, canonical lines, AST statements, functions, call edges, write sites, and verify the $\ge 20\%$ line / $\ge 10\%$ statement reduction thresholds.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test state_frame`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `state_frame.rs`, `density.rs`, remove from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 3: Pure Core Bounded Domain Scheduler & Witness Engine [x]
- **Requirements:** REQ-NAT-3.1 through REQ-NAT-3.5
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test pure_verifier` (4/4 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget).
- **Owned Files:**
  - `native/src/pure_verifier.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/lib.rs` (registration only, $+1$ line)
  - `native/complexity-budgets.json` (register `pure_verifier.rs`)
  - `native/tests/pure_verifier.rs` (new test suite)
- **Implementation:**
  1. Implement `PureCoreVerifier` in `pure_verifier.rs`: schedule Cartesian product of parameter domains across `DomainSpec`.
  2. Implement special tagged value comparisons (exact IEEE-754 bit parity, NaN, signed zeros, large ints, byte arrays).
  3. Implement minimal counterexample shrinking and witness generation when candidate diverges from baseline.
  4. Ensure incomplete domains or unsupported types return `Unverified`, and harness failures return `ExecutionFailed` (never false `Verified`).
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test pure_verifier`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `pure_verifier.rs`, remove from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 4: Ordered Stream Monitor & Adverse Mutant Generator [x]
- **Requirements:** REQ-NAT-4.1 through REQ-NAT-4.5
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test stream_monitor --test mutant_generator` (9/9 passed) and `cargo run --manifest-path native/Cargo.toml -- budgets` (within budget).
- **Owned Files:**
  - `native/src/stream_monitor.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/mutant_generator.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/lib.rs` (registration only, $+2$ lines)
  - `native/complexity-budgets.json` (register `stream_monitor.rs`, `mutant_generator.rs`)
  - `native/tests/stream_monitor.rs` (new test suite)
  - `native/tests/mutant_generator.rs` (new test suite)
- **Implementation:**
  1. Implement `stream_monitor.rs`: record monotonic sequence, step, kind (write, flush, log, abort, complete), and verify pre-abort byte preservation and stream ordering against baseline.
  2. Reject truncated, out-of-order, or dropped bytes with source-anchored witness.
  3. Implement `mutant_generator.rs`: apply AST mutation operators to target state structures, track mutation status (killed, survived, invalid, timeout), and verify complete denominator accounting without dropped controls.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test stream_monitor`
  - `cargo nextest run --manifest-path native/Cargo.toml --test mutant_generator`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert new modules, remove from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 5: Counted Specimen Runner & Child Process Supervisor
- **Requirements:** REQ-NAT-5.1 through REQ-NAT-5.5
- **Dependencies:** Task 1, Task 4
- **Owned Files:**
  - `native/src/specimen_runner.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/lib.rs` (registration only, $+1$ line)
  - `native/complexity-budgets.json` (register `specimen_runner.rs`)
  - `native/tests/specimen_runner.rs` (new test suite)
- **Implementation:**
  1. Implement `SpecimenTraceRunner` in `specimen_runner.rs`: spawn target specimen runtime (e.g., Python subprocess) with line-delimited JSON IPC on reserved stream descriptors.
  2. Manage isolated workspace, deadline monitoring with process-tree termination, and resource cleanup.
  3. Ensure byte-identical frozen LF-body digest reproduction and integration with existing `recording.rs` without modifying golden files.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test specimen_runner`
  - `cargo nextest run --manifest-path native/Cargo.toml --test recording`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `specimen_runner.rs`, remove from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 6: Token Guard & Refinement Feedback Integration
- **Requirements:** REQ-NAT-6.1, REQ-NAT-6.2
- **Dependencies:** Task 1, Task 2, Task 3
- **Owned Files:**
  - `native/src/token_guard.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/regenerator.rs` (integrate token guard and causal witness feedback)
  - `native/src/lib.rs` (registration only, $+1$ line)
  - `native/complexity-budgets.json` (register `token_guard.rs`)
  - `native/tests/token_guard.rs` (new test suite)
- **Implementation:**
  1. Implement `TokenGuard`: provider input counting, strict 120,000 token limit enforcement, streaming chunk decoder with UTF-8 boundary management.
  2. Connect causal failure witnesses from `state_frame`, `pure_verifier`, and `stream_monitor` into `regenerator.rs` refine prompt generation.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test token_guard`
  - `cargo nextest run --manifest-path native/Cargo.toml --test regenerator`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `token_guard.rs`, revert changes to `regenerator.rs`, remove from `lib.rs`, revert `complexity-budgets.json`.

---

### Task 7: Full System Verification, Budget Audit & Handover
- **Requirements:** REQ-NAT-6.3 through REQ-NAT-6.5
- **Dependencies:** Tasks 1–6
- **Owned Files:**
  - `.kiro/specs/native-deconvolution-closure/validation-report.md`
  - `.kiro/specs/native-deconvolution-closure/spec.json`
  - `.kiro/task_20_handover.md`
- **Implementation:**
  1. Run full native test battery: `cargo nextest run --manifest-path native/Cargo.toml`.
  2. Run budget checker: `cargo test --manifest-path native/Cargo.toml --test budgets`.
  3. Verify apparatus canonical lines under unchanged R37 method against R41 ceiling ($\le 7,173$).
  4. Run Python acceptance suites (`pytest -n auto tests/test_state_boundary.py`, `run_e5.py`, `run_e6.py`).
  5. Author validation report, update `spec.json`, and write `.kiro/task_20_handover.md`.
- **Verification:**
  - All test commands exit 0; 0 budget overages; 0 git diff on preserved WIP files.
