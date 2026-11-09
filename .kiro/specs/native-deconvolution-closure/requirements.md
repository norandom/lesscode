# Requirements Document: Native Deconvolution Closure

## Introduction

Define the requirements for complete native closure of the deconvolution apparatus in Rust.
While Experiments E1 through E6 demonstrated the effectiveness of state-boundary deconvolution and structural consolidation in breaking the Density Trap, the active verification and execution tooling currently remains split across two languages:
1. The State-Boundary Verification Apparatus: `src/lesscode_baseline/state_boundary/` (`frame.py`, `stream_monitor.py`, `pure_verifier.py`, `mutant_generator.py`, `density.py`) remains written in Python, while `native/src/state_contract.rs` only defines lightweight data structures (`ScopeMetrics`, `DeconvolutionOutcome`).
2. The Embedded Recording Shim: `native/shim/recording_shim.py` (178 canonical lines) executes in a Python subprocess inside the specimen virtual environment, maintaining external runtime dependency on Python database drivers, event listeners, and sequence execution.
3. Regenerator Substrate Extensions: In `native/src/regenerator.rs`, token counting (`count_tokens`) and Anthropic streaming response handling were deferred to the Python substrate (`src/lesscode_baseline/regenerator.py`).

This specification establishes requirements to transfer all remaining Python apparatus components into the native Rust substrate (`native/`), producing a fully compiled, self-contained native deconvolution core (`lesscode-core` / `lesscode_core`) governed by strict complexity budgets.

## Boundary Context

- **In scope:**
  - Native Rust port of the State-Boundary verification apparatus:
    - State frame condition derivation, tracking, and boundary violation detection.
    - AST statement, call graph, and function density metrics (`density.rs` / `surface.rs`).
    - Bounded domain equivalence checking for extracted pure functions (`pure_verifier.rs`).
    - Side-effect stream, buffer flush, and call-ordering monitor (`stream_monitor.rs`).
    - Adverse state mutation generator and kill-rate gate (`mutant_generator.rs`).
  - Native Specimen Execution and Recording Architecture:
    - Direct native sequence execution, transaction lifecycle management, and observation capture, replacing the external Python runner `native/shim/recording_shim.py` with compiled native instrumentation or typed IPC protocol.
  - Native Regenerator Closure:
    - Native token budgeting, length guards, and streaming model client handling in `native/src/regenerator.rs` (or modular split).
  - Substrate Size Governance and Prototype-First Gate:
    - Formal registration in `complexity-budgets.json` complying with per-file limits ($\le 400$ physical lines, $\le 300$ syntax units) and bounded prototype evidence under `.kiro/steering/prototype-first.md`.
    - Native test coverage (`cargo nextest`) demonstrating functional parity with the Python baseline.
- **Out of scope:**
  - Modifying specimen application behavior or changing baseline tests.
  - Relaxing verification rigor, reducing mutant sensitivity, or expanding equivalence tolerance.
  - Modifying the 4 preserved owner WIP files:
    - `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
    - `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
    - `src/lesscode_baseline/rules/evaluate.py`
    - `tests/test_rule_evaluation.py`
- **Adjacent expectations:**
  - Existing `lesscode-core` CLI subcommands (`prepare`, `compare`, `admission`, `budgets`, `surface`, `regenerate`) preserve CLI arguments, exit codes, and serialization compatibility.

## Requirements

### Requirement 1: Native State Frame and Side-Effect Boundary Enforcement

**Objective:** As a verification engineer, I want state frame conditions and side-effect boundaries analyzed and checked natively in Rust, so that state isolation verification executes without Python interpreter overhead.

#### Acceptance Criteria
1. When target source code is submitted for deconvolution analysis, the native analysis engine shall extract mutable state frame conditions, identifying all instance attributes, global variables, and external storage sites.
2. When a deconvolution candidate is verified, the native state verification engine shall inspect execution traces against declared frame conditions and shall reject the candidate if any undeclared state location is modified.
3. When an execution exits via an exception or controlled error abort, the native state verification engine shall verify that state post-condition invariants and failure-side cleanup are preserved.
4. If a candidate attempts to mutate a read-only parameter, global configuration, or undeclared state location, the native state verification engine shall emit a structured boundary violation receipt.
5. While verifying state transitions, the native state verification engine shall distinguish between permitted mutations within the imperative shell and prohibited side effects inside the pure core.

### Requirement 2: Native AST Statement Density and Structural Consolidation Metrics

**Objective:** As a maintainer, I want density scoring and structural consolidation calculated natively using ast-grep/tree-sitter, so that line and statement reductions are audited within the compiled core.

#### Acceptance Criteria
1. When evaluating a deconvolution candidate, the native metrics engine shall calculate physical lines, canonical lines, AST statement count, function count, intra-scope call edges, and mutable write sites across the containing scope.
2. When measuring code reduction, the native metrics engine shall require both canonical line reduction ($\ge 20\%$) and AST statement reduction ($\ge 10\%$) to qualify the candidate as compressed.
3. If a candidate achieves function consolidation without satisfying the AST statement reduction threshold, the native metrics engine shall flag the candidate as subject to the Density Trap and reject compression qualification.
4. When reporting deconvolution outcome, the native metrics engine shall output structured metrics matching `DeconvolutionOutcome` contracts.
5. The native metrics engine shall compute all density and structural metrics without delegating to external Python processes.

### Requirement 3: Native Bounded Formal Equivalence Verification

**Objective:** As a researcher, I want pure functions extracted into the functional core verified for bounded equivalence natively in Rust, so that semantic equivalence is mathematically checked without external Python test runners.

#### Acceptance Criteria
1. When a function is identified as pure, the native verification engine shall statically verify that it contains zero write sites, zero global lookups, and zero external I/O invocations.
2. When evaluating pure candidate functions against baseline logic, the native verification engine shall check input/output equivalence across bounded integer domains ($[-10, 10]$), floating-point representations, empty collections, and schema boundaries.
3. If an input combination yields divergent return values, disparate exception types, or non-equivalent termination, the native verification engine shall emit a minimal reproducible counterexample witness.
4. Where a pure function cannot be verified within bounded domains, the native verification engine shall mark the pure equivalence status as unverified and require differential trace coverage.
5. The native verification engine shall prohibit claiming functional core equivalence if pure functions were evaluated solely by happy-path sample inputs.

### Requirement 4: Native Stream Monitoring and Adverse Mutation Gates

**Objective:** As a maintainer, I want stream output ordering verified and adverse state mutants injected natively, so that subtle error-path corruptions and oracle blindness are eliminated.

#### Acceptance Criteria
1. When candidate operations execute, the native stream monitor shall observe intermediate stream writes, log emissions, and buffer flush operations prior to completion or abort.
2. If a candidate drops, buffers without flushing, or alters the sequence of emitted stream events, the native stream monitor shall report a stream divergence violation.
3. When adverse mutation testing is conducted, the native mutation generator shall synthesize targeted state-boundary mutants, exception-path mutants, and buffer mutation operators directly into candidate ASTs.
4. When candidate test suites execute against adverse mutants, the native verification engine shall require a 100% kill rate of all non-equivalent mutants before granting qualification.
5. If any adverse state mutant survives candidate verification, the native verification engine shall withhold candidate qualification and identify the surviving unobserved mutant.

### Requirement 5: Native In-Process Specimen Execution and Recording Engine

**Objective:** As an operator, I want the recording, replay, and sequence execution engine running natively without `recording_shim.py`, so that external Python subprocess shimming is eliminated from the native toolchain.

#### Acceptance Criteria
1. When recording or replaying execution sequences, the native recording engine shall execute sequences, capture observations, and compute canonical digests directly through native process orchestration or direct IPC.
2. When managing database lifecycle and transaction checkpoints, the native engine shall handle server startup, migration checks, and snapshot rollbacks without depending on `native/shim/recording_shim.py`.
3. If sequence execution encounters non-deterministic state, unobserved SQL queries, or Level 1 divergence, the native recording engine shall report the exact diverging step and abort replay.
4. The native recording engine shall preserve byte-identical LF-body digests and SHA-256 sidecar compatibility with existing frozen goldens (`FROZEN_E3_LF_BODY`, `FROZEN_E4_LF_BODY`).
5. When executing native sequence replay, the native recording engine shall capture and diff observations without creating intermediate unmonitored temporary files.

### Requirement 6: Native Refine Loop Integration and Substrate Budget Governance

**Objective:** As a systems architect, I want streaming model interaction, token counting, and all transferred modules governed by strict complexity budgets, so that native code volume remains bounded and auditable.

#### Acceptance Criteria
1. When refining candidate code through LLM interfaces, the native regenerator shall enforce token input limits and process streaming responses directly within `native/src/regenerator.rs`.
2. When state boundary violations or equivalence counterexamples occur, the native regenerator shall format and inject causal feedback witnesses into subsequent refine prompts.
3. When any new or modified module is added to `native/src/`, the native budgets checker (`cargo test --test budgets`) shall enforce that every new or modified production file satisfies its registered ceiling in `native/complexity-budgets.json` (default $\le 400$ physical lines and $\le 300$ logical syntax units); pre-existing legacy files remain under their frozen ceilings and are not re-capped (finding F-1, 2026-11-09 fusion review).
4. When proposed additions exceed the existing substrate ceiling, the workflow shall require an approved prototype gate and explicit budget amendment before production task generation.
5. The native suite shall maintain 100% test pass rates under `cargo-nextest` on Windows without requiring WSL or cmd.exe.
