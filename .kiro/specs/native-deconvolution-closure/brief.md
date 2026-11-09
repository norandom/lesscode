# Specification Brief: Native Deconvolution Closure

## 1. Problem Statement

Across Experiments E1 through E6, the Lesscode research program validated automated code deconvolution, structural consolidation, and state-boundary isolation, achieving $>23\%$ canonical line reduction, $>10\%$ statement reduction, 100% adverse mutant detection, and 0 behavioral divergences.

However, several critical components of the "native" apparatus currently remain implemented in Python rather than compiled Rust:
1. **State-Boundary Verification Apparatus:** The core verification engine introduced in `state-boundary-deconvolution` (`src/lesscode_baseline/state_boundary/`: `frame.py`, `pure_verifier.py`, `stream_monitor.py`, `mutant_generator.py`, `density.py` — ~1,440 lines) remains entirely in Python. The Rust crate `native/` only defines lightweight data structures in `native/src/state_contract.rs` (92 lines: `ScopeMetrics` and `DeconvolutionOutcome`), relying on Python subprocesses to execute actual property-based equivalence, mutation generation, and stream monitoring.
2. **Embedded Recording Shim:** `native/shim/recording_shim.py` (178 canonical lines) resides inside the `native/` tree but runs as an external Python subprocess inside the specimen virtual environment (`.runs/e2-facdrone/venv`), managing SQLAlchemy listeners, database engine lifecycle, and seed/generated sequence execution.
3. **Regenerator Substrate Extensions:** In `native/src/regenerator.rs`, token counting (`count_tokens`) and Anthropic streaming response handling were deferred to the Python substrate (`src/lesscode_baseline/regenerator.py`), supporting only OpenAI-compatible endpoints natively.

This split architecture creates runtime friction, requires cross-language subprocess serialization, and leaves the native solution incomplete ("unclosed").

## 2. Proposed Approach

Create a unified, self-contained native Rust closure in `native/` that ports the remaining Python apparatus:
1. **Native State-Boundary Engine (`native/src/state_boundary/` or modular Rust files):**
   - Port `frame.py` (state frame extraction and boundary conditions) into native Rust.
   - Port `density.py` (AST statement and call-graph density evaluation) using the existing `ast-grep` / `tree-sitter-python` infrastructure.
   - Port `pure_verifier.py` (pure core bounded equivalence verification) into Rust property-based / domain-checking logic.
   - Port `stream_monitor.py` (side-effect stream and buffer ordering) into native Rust trace verification.
   - Port `mutant_generator.py` (adverse state mutant injection and syntax mutation) into native AST-level mutation operators.
2. **Native Specimen Execution & Recording Bridge:**
   - Eliminate or minimize `recording_shim.py` by transitioning sequence execution, environment resets, and observation capture to direct native execution or a compiled IPC protocol, keeping Python involvement strictly limited to target code execution if required by runtime semantics.
3. **Regenerator Streaming & Token Guard Closure:**
   - Port token budget validation and streaming client capabilities directly into `native/src/regenerator.rs` (or submodule) to achieve full native refine loop independence.
4. **Substrate Budget Governance:**
   - Account for all new modules under `native/complexity-budgets.json` (the authoritative path read by `native/tests/budgets.rs`; finding F-9), respecting per-file caps ($\le 400$ physical lines, $\le 300$ syntax units) and establishing a bounded prototype gate per `prototype-first.md`.

## 3. Scope & Boundaries

- **In Scope:**
  - Full Rust port of state-boundary verification (`frame`, `stream_monitor`, `pure_verifier`, `mutant_generator`, `density`).
  - Native replacement or encapsulation of `native/shim/recording_shim.py`.
  - Native token counting and streaming response handling in `regenerator.rs`.
  - Integration tests in `native/tests/` verifying functional equivalence with the Python baseline.
  - Substrate budget governance and complexity accounting.
- **Out of Scope:**
  - Changing target specimen business logic or modifying frozen golden traces (`e1`–`e6`).
  - Weakening verification criteria, mutation kill requirements, or equivalence bounds.
  - Modifying the 4 preserved owner WIP files.

## 4. Constraints

- Native Windows PowerShell only (no `cmd.exe`, no WSL for development execution).
- Tests must pass cleanly with `cargo-nextest` for Rust and `pytest-xdist` for reference checks.
- Per-file complexity ceilings ($\le 400$ physical lines, $\le 300$ logical syntax units) strictly enforced by `cargo test --test budgets`.
- Research continuity and prototype-first gate compliance under `.kiro/steering/prototype-first.md`.
