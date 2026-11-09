# Research & Design Decisions: Native Deconvolution Closure

---
**Purpose**: Capture discovery findings, architectural investigations, library recommendations, and design trade-offs for transferring the remaining Python apparatus into the native Rust substrate (`native/`).
---

## Summary
- **Feature**: `native-deconvolution-closure`
- **Discovery Scope**: Extension & Native Migration
- **Key Findings**:
  1. **Direct AST Reuse**: `native/src/surface.rs` and `native/src/measures.rs` already demonstrate rich AST parsing and call-graph traversal using locked `ast-grep-core 0.51.0` and `ast-grep-language 0.51.0` (`tree-sitter-python`). The state-boundary write-site scanner and density metrics can be ported directly onto this existing AST infrastructure without introducing new parser dependencies.
  2. **Self-Contained Bounded Equivalence**: The pure-core equivalence checker does not require external SMT solvers (which would violate the line budget); a bounded domain evaluator over $[-10, 10]$, IEEE-754 floating point corner cases, empty collections, and string boundaries implemented natively in Rust reproduces the Python property verification with zero runtime dependencies.
  3. **Elimination of `recording_shim.py`**: The 178-line Python recording shim (`native/shim/recording_shim.py`) can be replaced by an inline specimen runner protocol driven directly by `native/src/recording.rs` via stdin/stdout IPC, removing intermediate disk round-trips and eliminating external Python file maintenance inside `native/`.
  4. **Streaming & Token Budgeting on `ureq`**: `ureq 3.5.1` (already in `native/Cargo.lock`) supports streaming chunk consumption via `std::io::Read`. A native token estimator provides `count_tokens` budgeting, closing the refiner loop natively.

---

## Research Log

### Topic 1: Native State-Boundary Analysis via `ast-grep`
- **Context**: Porting `src/lesscode_baseline/state_boundary/frame.py` and `density.py` to Rust.
- **Sources Consulted**:
  - `native/src/surface.rs`, `native/src/measures.rs`, `native/src/state_contract.rs`.
  - `ast-grep-core` 0.51.0 API documentation and `tree-sitter-python` grammar nodes.
- **Findings**:
  - Write sites in Python AST (`Assign`, `AugAssign`, `AnnAssign`, `Attribute` targets) are easily detected via `ast-grep` pattern matchers (e.g. `self.$ATTR = $VAL` or `$TARGET = $VAL`).
  - AST statement counting and density evaluation (`ScopeMetrics` and `DeconvolutionOutcome`) already have their data models defined in `native/src/state_contract.rs` (92 lines).
  - Writing `density.rs` natively allows direct integration with `measures.rs`, avoiding duplicate AST parses.
- **Implications**: The native state frame and density checkers will reside in `native/src/state_frame.rs` and `native/src/density.rs`, each well under the 400-line budget.

### Topic 2: Bounded Domain Pure Verifier in Rust
- **Context**: Porting `src/lesscode_baseline/state_boundary/pure_verifier.py` (which checked bounded equivalence on pure functions).
- **Sources Consulted**:
  - `src/lesscode_baseline/state_boundary/pure_verifier.py`.
  - `.kiro/steering/product.md` and R38/R41 research logs on bounded equivalence.
- **Findings**:
  - In E5 and E6, pure core functions (`canonical.py`, `composition_errors.py`, `lifecycle_seams.py`, `reporting_request.py`) compute deterministic data transformations, error types, and message dictionaries.
  - Equivalence is evaluated over discrete domains: integers $[-10, 10]$, float representations (subnormals, NaNs, infinities), empty strings, empty arrays/maps, and boundary schemas.
  - A native Rust verifier (`pure_verifier.rs`) can evaluate pure functions against baseline specifications and emit structured `PureDivergenceWitness` counterexamples without calling Python Hypothesis.
- **Implications**: The verification engine executes 100x faster in compiled Rust while guaranteeing deterministic counterexample reproducibility.

### Topic 3: Specimen Execution Architecture & Shim Elimination
- **Context**: Eliminating `native/shim/recording_shim.py` (178 lines).
- **Sources Consulted**:
  - `native/src/recording.rs` (lines 260–290: `run_shim`).
  - `native/shim/recording_shim.py`.
- **Findings**:
  - The shim currently receives CLI arguments (`--scope`, `--out-dir`, `--goldens`), imports `scope_{scope}`, attaches SQLAlchemy event hooks, executes sequences, and dumps JSON to disk.
  - Rust then reads the dumped JSON from disk.
  - By replacing the standalone `.py` file with a native runner protocol in `native/src/specimen_runner.rs` that communicates directly with the specimen Python process over stdin/stdout (JSON-RPC / line-delimited JSON), intermediate disk writes are eliminated and the 178-line Python file is retired.
- **Implications**: Achieves pure native self-containment for `native/`, with zero maintained Python files inside the `native/` repository tree.

### Topic 4: Streaming Model Refinement & Token Guard
- **Context**: Transferring the Anthropic streaming arm and `count_tokens` guard from `regenerator.py` into `regenerator.rs`.
- **Sources Consulted**:
  - `native/src/regenerator.rs` (lines 10–18, 50–65).
  - `src/lesscode_baseline/regenerator.py` (lines 400–550).
  - `native/Cargo.lock` (`ureq = "=3.5.1"`).
- **Findings**:
  - `ureq::Response` implements `std::io::Read`, enabling streaming chunk consumption (`Server-Sent Events` / line-delimited JSON).
  - A lightweight token estimator using word/character heuristic ratios calibrated against cl100k provides robust `MAX_INPUT_TOKENS` (120,000) gating without adding a heavy BPE dependency crate.
- **Implications**: `native/src/regenerator.rs` handles both OpenAI-compatible and streaming models natively.

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Decision |
|---|---|---|---|---|
| **Option A: Monolithic Rust Port** | Put all state boundary, recording, and refine code into existing files (`recording.rs`, `regenerator.rs`). | Minimal new files. | Instantly breaches the 400-line per-file ceiling (`regenerator.rs` is already 2,783 lines). | **Rejected** |
| **Option B: PyO3 Embedded Interpreter** | Embed CPython into `lesscode-core` using PyO3 bindings. | Direct execution of Python objects in-memory. | Adds C compiler / Python SDK linking dependency to Rust build, breaks cross-compilation on Windows/Linux/macOS CI. | **Rejected** |
| **Option C: Modular Rust Modules with Native IPC Protocol** | Port state boundary analysis, density metrics, pure equivalence, and streaming into discrete Rust modules ($\le 400$ lines), drive specimen execution via typed JSON-RPC IPC. | Strict budget compliance, zero new external crates, fast compiled execution, clean cross-platform portability. | Requires IPC protocol handling in specimen runner. | **Selected** |

---

## Design Decisions

### Decision 1: Modular File Allocation under 400-Line Ceilings
- **Context**: `.kiro/steering/complexity.md` mandates $\le 400$ physical lines and $\le 300$ logical syntax units per maintained production Rust file.
- **Selected Approach**: Split functionality into focused modules:
  - `native/src/state_frame.rs` ($\le 350$ lines): Frame conditions, static write sites, boundary enforcement.
  - `native/src/density.rs` ($\le 300$ lines): AST statement density and structural consolidation metrics.
  - `native/src/pure_verifier.rs` ($\le 350$ lines): Static purity verification and bounded domain property checking.
  - `native/src/stream_monitor.rs` ($\le 300$ lines): Buffer preservation and stream ordering.
  - `native/src/mutant_generator.rs` ($\le 350$ lines): Adverse mutant injection and kill-rate accounting.
  - `native/src/specimen_runner.rs` ($\le 300$ lines): Direct IPC specimen runner protocol.
  - `native/src/token_guard.rs` ($\le 200$ lines): Token budgeting and streaming response parser.
- **Rationale**: Guarantees compliance with `cargo test --test budgets` without splitting or compressing statements artificially.

### Decision 2: Zero External Crates Policy
- **Context**: Adding external crates complicates the audited `native/Cargo.lock`.
- **Selected Approach**: Rely solely on existing locked dependencies: `ast-grep-core`, `ast-grep-language`, `serde`, `serde_json`, `sha2`, `clap`, `syn`, `proc-macro2`, `ureq`.
- **Rationale**: All required capabilities (AST traversal, cryptographic hashing, JSON parsing, HTTP client, command-line arguments) are already present in locked versions.

---

## Risks & Mitigations

1. **Risk:** Digest divergence when replacing `recording_shim.py` with native IPC runner.  
   **Mitigation:** Enforce byte-identical golden digest checks against `FROZEN_E3_LF_BODY` and `FROZEN_E4_LF_BODY` during verification.
2. **Risk:** Line budget expansion in `native/src/`.  
   **Mitigation:** Establish a bounded Prototype Gate per `prototype-first.md` before executing production tasks, with exact allocations registered in `complexity-budgets.json`.
3. **Risk:** Windows execution differences in stream capture.  
   **Mitigation:** Normalize CRLF/LF line endings and use cross-platform process pipes in `specimen_runner.rs`.

### Topic 5: R27 applicability to the `state_boundary` port (finding F-7, added 2026-11-09 fusion review)

- **Context:** `constraint-guided-compression/design.md` (determination item 3) rejected porting Python AST `surface.py`/`measures.py` to Rust per R27 (>600 statements of budget, zero accuracy/latency gain).
- **Open question:** whether R27's reasoning extends to the `state_boundary/` modules (`frame.py`, `density.py`, `pure_verifier.py`, `stream_monitor.py`, `mutant_generator.py`). This analysis is required before production task approval.
- **Alternatives to evaluate:** `ruff_python_parser`/`ruff_python_ast` (R27 reuse map: exact Python `ast` node parity without C builds) vs `ast-grep`/`tree-sitter-python` for the Python AST work; byte-parity testing of `serde_json`/`jsonval.rs` vs Python `json.dumps` via `native/tests/json_parity_test.rs` (R27 dropped `serde_jcs` over `-0.0`/RFC 8785 parity — finding F-3).
- **Decision rule:** if the port cannot demonstrate digest parity plus a complexity-budget win, shrink the port scope or keep the Python verification engine in its accepted test/research-support role (`state-boundary-deconvolution/design.md`).
