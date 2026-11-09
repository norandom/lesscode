# Brief: Dogfooding Self-Application and Tooling Corrections

## 1. Problem Statement
The native deconvolution substrate (`lesscode-core`) and agent skill (`deconvolute`) have achieved cross-language parity across Python, TypeScript, Java, and Rust. However, the tooling has not yet been exercised end-to-end on `lesscode` itself in an actual dogfooding trial. Running on real production code in an isolated clone of this repository will surface rough edges, missing CLI porcelain, path resolution quirks (`checked_path`), test runner execution friction (`CARGO_TARGET_DIR` lock contention), and manual orchestration burdens in the skill.

## 2. Approach
1. **Isolated Repository Clone**: Provision an isolated copy of the repository (`C:\Users\mariu\Source\lesscode-dogfood-copy`) so that dogfooding mutations, experimental candidate rewrites, and test executions never dirty or compromise the active `main` workspace.
2. **Empirical Dogfooding Trial**: Exercise the deconvolution toolchain on concrete candidate responsibilities within `native/` in the copy (e.g. `budgets.rs`, `admission.rs`, or adapter supervisors):
   - AST frame & forbidden syntax analysis (`syn` AST scanning).
   - Adverse mutant generation (4 mandatory operators).
   - Bounded domain Cartesian evaluation.
   - Mechanical density & line reduction calculation.
3. **Rough Edge Inventory**: Document every observed point of friction, including CLI ergonomics, path validation issues, command verb gaps, and error diagnostics.
4. **Tooling Corrections**: Formalize the required fixes into EARS acceptance criteria, architectural design, and prioritized TDD tasks.

## 3. Boundary Candidates
- **In-Scope**:
  - Provisioning and configuring the isolated dogfood repository copy.
  - Executing deconvolution analysis, mutation, and verification on the copy.
  - Cataloging all rough edges and operational friction.
  - Adding first-class CLI commands/porcelain for deconvolution (`analyze`, `mutate`, `verify`) to eliminate manual glue code.
  - Hardening path handling, build locks, and test runner supervision.
- **Out-of-Scope**:
  - In-place mutation of the active `lesscode` repository during trial runs.
  - Rewriting native core functionality unrelated to deconvolution tooling and CLI ergonomics.
  - Lowering complexity budgets or weakening adverse mutation kill thresholds.

## 4. Constraints
- Windows PowerShell execution only (never `cmd.exe` or WSL).
- Pinned Rust 1.98.1 toolchain and `cargo-nextest` 0.9.143.
- Substrate line budgets strictly enforced: $\le 400$ physical lines, $\le 300$ syntax units, $\le 80$ lines per function.
- All 338 native tests must remain passing without regression.
