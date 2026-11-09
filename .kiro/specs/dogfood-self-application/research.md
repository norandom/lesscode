# Research Log: Dogfooding Self-Application Trial

## 1. Context & Objective
Exercise the native `lesscode-core` tooling and deconvolution workflow on an isolated copy of the `lesscode` repository to identify friction points, edge-case failures, and CLI usability gaps before expanding usage to external projects.

## 2. Experimental Setup
- **Baseline Repository**: `C:\Users\mariu\Source\lesscode` (commit `ceb89bc`).
- **Isolated Dogfood Workspace**: `C:\Users\mariu\Source\lesscode-dogfood-copy` cloned via `git clone . ..\lesscode-dogfood-copy`.
- **Target Substrate**: Native Rust substrate (`native/Cargo.toml`, 338 nextest tests across 33 test binaries).

## 3. Empirical Observations & Findings

### Observation 1: Missing Fixtures in Clean Checkouts
- **Action**: Executed `cargo nextest run --manifest-path ..\lesscode-dogfood-copy\native\Cargo.toml`.
- **Result**: Failed 1 test out of 338: `historical_s2_bindings_drift_reproduces_counts` in `tests/admission.rs`.
- **Diagnosis**: 11 regenerator turn fixtures in `docs/experiments/deconvolution/regenerator/results/turns/` were present in the host working tree as untracked files, but missing from git. A clean clone failed because these fixtures were absent.
- **Resolution**: Track and commit all 11 turn files to ensure 100% clean clone reproducibility.

### Observation 2: Missing CLI Porcelain for Deconvolution
- **Action**: Inspected `lesscode-core --help` and `main.rs`.
- **Result**: `lesscode-core` provides subcommands for `prepare`, `compare`, `admission`, `budgets`, `surface`, and `regenerate`. It lacks subcommands for `analyze`, `mutate`, and `density`.
- **Diagnosis**: The deconvolution functionality exists only as Rust library modules (`adapters::rust::analysis`, `adapters::rust::mutation`, `adapters::rust::metrics`, `density`). An agent or developer following `.agents/skills/deconvolute/SKILL.md` cannot execute these without writing ad-hoc code.
- **Resolution**: Introduce `lesscode-core deconvolute` (or direct subcommands `analyze`, `mutate`, `density`) in a modular `cli_deconvolute.rs` file.

### Observation 3: Path Normalization Rejection in `checked_path`
- **Action**: Analyzed cross-workspace invocation (`..\lesscode-dogfood-copy`).
- **Result**: `checked_path` immediately rejects `Component::ParentDir` (`..`) with `"path traversal is not allowed"`.
- **Diagnosis**: When joined with `current_dir()`, unnormalized relative paths contain `..`, triggering false-positive traversal rejections even for valid sibling directory paths.
- **Resolution**: Normalize parent directory components safely (`resolved.pop()`) while ensuring traversal does not escape above the root prefix.

### Observation 4: Global Build Lock Contention
- **Action**: Running `cargo nextest` across specimen copies shared `D:\BuildArtifacts\cargo-target`.
- **Result**: Cargo blocked on `Blocking waiting for file lock on build directory`.
- **Diagnosis**: Specimen runners must use isolated target directories (`--target-dir` or setting `CARGO_TARGET_DIR`) to prevent lock collisions during concurrent verification.

## 4. User Workflow Direction
Developers and agents will run deconvolution on a clean git clone/worktree, generate the pure kernel and compressed imperative shell, verify all tests and adverse controls, and push a dedicated branch (`refactor/deconvolute-...`). The branch is then reviewed and validated like a major refactoring prior to merging into main.
