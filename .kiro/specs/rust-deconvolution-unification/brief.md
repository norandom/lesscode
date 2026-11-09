# Specification Brief: Rust Deconvolution Unification

## 1. Problem Statement
While Lesscode's native apparatus is implemented in Rust and self-governed via `syn`/`proc-macro2` syntax unit budgets, automated code deconvolution (state-boundary isolation, pure-core extraction, adverse mutation gates, and dynamic execution tracing) currently operates only against external Python specimens.

To achieve complete self-application maturity and eliminate structural complexity within Rust codebases, Rust must be brought to full deconvolution parity.

## 2. Proposed Approach
1. **Rust State-Boundary Analyzer:** Detection of `mut` variable bindings, `&mut` borrows, and mutable interior pointers (`RefCell`, `Mutex`, `RwLock`) via `syn`.
2. **Rust Pure Core Verifier:** Static purity auditing of functions (no `mut` arguments, no static/global accesses, no I/O calls), with bounded domain equivalence verification across Rust primitive and composite types.
3. **Rust Adverse Mutation Engine:** Expression-level and statement-level mutation operators targeting error-handling matches (`Err(_)`, `None`), buffer flushes, and state assignment expressions.
4. **Rust Specimen Runner:** Execution driver using `cargo-nextest` with process isolation and JSON test event streaming.

## 3. Scope & Boundaries
- **In Scope:**
  - Rust AST state-boundary analysis via `syn`.
  - Rust pure core static purity and bounded domain checking.
  - Rust adverse mutation generation and kill-rate gating.
  - `cargo-nextest` dynamic specimen execution runner.
  - Bounded self-application trial on Lesscode's own native modules.
- **Out of Scope:**
  - Unbounded formal verification of unsafe Rust or external C FFI.
  - Modifying the existing Python, TypeScript, or Java adapters.

## 4. Constraints
- Must respect the 400-line per-file complexity ceiling.
- Must pass a bounded prototype gate before production task approval.
