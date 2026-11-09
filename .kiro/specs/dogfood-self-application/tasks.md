# Implementation Tasks: Dogfooding Self-Application and Tooling Corrections

Status: **approved for foreground implementation**, 2026-11-09.
Governed by [requirements.md](requirements.md) and [design.md](design.md).
Owner rules in effect: foreground TDD execution only (no subagents), Windows PowerShell only, one commit per completed task on `main`, pinned Rust 1.98.1 and `cargo-nextest` 0.9.143, complexity budgets strictly maintained ($\le 400$ physical, $\le 300$ units, $\le 80$ lines/fn).

---

### Task 1: Repository Test Fixture Integrity & Clean-Clone Reproducibility [x]
- **Requirements:** REQ-4.1, REQ-4.2, REQ-4.3
- **Dependencies:** None
- **Status:** Completed in commit `4a79d57`. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test admission historical_s2_bindings_drift_reproduces_counts` (PASS) and verified in clean clone `..\lesscode-dogfood-copy` (PASS).
- **Owned Files:**
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn1/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn1/user.txt`
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn2/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn2/user.txt`
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn3/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/e4-turn3/user.txt`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn1/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn1/user.txt`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn1-retry/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn1-retry/user.txt`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn2/response.json`
  - `docs/experiments/deconvolution/regenerator/results/turns/t4-turn2/user.txt`
- **Implementation:**
  1. Add and track the 11 regenerator turn fixtures in git.
  2. Verify that `tests/admission.rs` (`historical_s2_bindings_drift_reproduces_counts`) finds all bound files and reproduces the expected verified count (11).
  3. Pull changes into `..\lesscode-dogfood-copy` and verify that the clean clone now passes 100% of the test suite.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test admission historical_s2_bindings_drift_reproduces_counts`
  - `cargo nextest run --manifest-path ..\lesscode-dogfood-copy\native\Cargo.toml`
- **Rollback:** Untrack the turn files with `git reset HEAD docs/experiments/...`.

---

### Task 2: Cross-Workspace Path Resolution Normalization [x]
- **Requirements:** REQ-3.1, REQ-3.2, REQ-3.3, REQ-3.4
- **Dependencies:** Task 1
- **Status:** Completed in commit `a75389b`. Verified with `cargo test --manifest-path native/Cargo.toml --test cli` (8/8 passed) and `cargo test --manifest-path native/Cargo.toml --test budgets` (clean, 0 overage).
- **Owned Files:**
  - `native/src/main.rs` (update `checked_path`)
  - `native/tests/cli.rs` (add test for relative parent paths)
- **Implementation:**
  1. In `native/src/main.rs:checked_path`, update `Component::ParentDir` handling: pop the last component from `resolved` instead of immediately aborting with `"path traversal is not allowed"`.
  2. Ensure root escapes (popping past the drive prefix or empty root) still return an explicit traversal error.
  3. Keep symlink/reparse point checks intact on the canonicalized path.
  4. Add integration test verifying that relative paths pointing to sibling directories (e.g. `..\lesscode-dogfood-copy`) resolve cleanly.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test cli`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `native/src/main.rs` and `native/tests/cli.rs`.

---

### Task 3: Specimen Test Execution Isolation & Build Concurrency [x]
- **Requirements:** REQ-5.1, REQ-5.2, REQ-5.3, REQ-5.4
- **Dependencies:** Task 1
- **Status:** Completed. Verified with `cargo nextest run --manifest-path native/Cargo.toml --test rust_adapter` (7/7 passed).
- **Owned Files:**
  - `native/src/adapters/rust/mod.rs`
  - `native/tests/rust_adapter.rs`
- **Implementation:**
  1. In `native/src/adapters/rust/mod.rs`, ensure that when spawning `cargo nextest` or `cargo test`, `CARGO_TARGET_DIR` is set to an isolated target directory under the specimen root (`target/isolated-specimen`).
  2. Verify that concurrent executions do not lock the host `CARGO_TARGET_DIR`.
  3. Add a test in `native/tests/rust_adapter.rs` validating environment variable injection.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test rust_adapter`
- **Rollback:** Revert `native/src/adapters/rust/mod.rs`.

---

### Task 4: Deconvolution CLI Porcelain Commands (`analyze`, `mutate`, `density`) [x]
- **Requirements:** REQ-1.4, REQ-2.1, REQ-2.2, REQ-2.3, REQ-2.4, REQ-2.5, REQ-6.1, REQ-6.3
- **Dependencies:** Task 2, Task 3
- **Status:** Completed. Verified with `cargo test --manifest-path native/Cargo.toml --target-dir D:\BuildArtifacts\lesscode-cargo-test --test cli_deconvolute` (6/6 passed) and `cargo test --manifest-path native/Cargo.toml --target-dir D:\BuildArtifacts\lesscode-cargo-test --test budgets` (10/10 passed).
- **Owned Files:**
  - `native/src/cli_deconvolute.rs` (new: $\le 400$ physical, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/main.rs` (add `Deconvolute` variant to `Command`)
  - `native/complexity-budgets.json` (register `cli_deconvolute.rs`)
  - `native/tests/cli_deconvolute.rs` (new test suite)
- **Implementation:**
  1. Define `DeconvoluteCommand` enum with subcommands:
     - `Analyze`: `--lang <lang> --source <file> [--frame-spec <file>] [--out <file>]`
     - `Mutate`: `--lang <lang> --source <file> --out-dir <dir> [--out <file>]`
     - `Density`: `--lang <lang> --baseline <base> --candidate <cand> [--out <file>]`
  2. Implement language dispatch supporting `rust`, `java`, `typescript`, and `python`.
  3. For `Analyze`: call adapter `StateFrameAnalyzer`, output dynamic gaps and write sites receipt.
  4. For `Mutate`: call adapter `AdverseMutantGenerator`, write generated mutants to `--out-dir`, output mutation receipt.
  5. For `Density`: call adapter metrics functions, compute canonical line and statement reductions, check against mechanical density gate ($\ge 20\%$ lines, $\ge 50$ lines, $\ge 10\%$ stmts, $\ge 10$ stmts), output density receipt.
  6. Register `cli_deconvolute.rs` in `complexity-budgets.json`.
  7. Add integration tests covering CLI options, stdout/file output, and exit codes (0 for success, 3 for invalid input).
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml --test cli_deconvolute`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Delete `cli_deconvolute.rs`, revert `main.rs` and `complexity-budgets.json`.

---

### Task 5: Agent Skill Ergonomics & Clean-Clone Branch Workflow Documentation [x]
- **Requirements:** REQ-1.1, REQ-1.2, REQ-1.3, REQ-6.1, REQ-6.2, REQ-6.3
- **Dependencies:** Task 4
- **Status:** Completed. Updated `.agents/skills/deconvolute/SKILL.md` with binary location discovery, CLI porcelain commands (`analyze`, `mutate`, `density`), and the clean-clone branch-based refactoring workflow.
- **Owned Files:**
  - `.agents/skills/deconvolute/SKILL.md`
- **Implementation:**
  1. Update `SKILL.md` with exact CLI porcelain commands (`lesscode-core deconvolute analyze|mutate|density`).
  2. Add binary location discovery guidance (locating `lesscode-core.exe` in `target/debug/` or `$env:CARGO_TARGET_DIR/debug/`).
  3. Document the standardized developer workflow:
     - Clone/checkout clean specimen copy.
     - Execute deconvolution, mutation, and test verification on the copy.
     - Push candidate branch (`refactor/deconvolute-...`).
     - Treat the branch as a major refactoring (running full test suite and reviewing adverse mutation evidence).
- **Verification:**
  - Inspect `SKILL.md` for completeness and accuracy.
- **Rollback:** Revert `.agents/skills/deconvolute/SKILL.md`.

---

### Task 6: CLI Pre-Push Hook and Hook Installer (`pre-push`, `install-hook`) [x]
- **Requirements:** REQ-7.1, REQ-7.2, REQ-7.3, REQ-7.4, REQ-7.5
- **Dependencies:** Task 4, Task 5
- **Status:** Completed. Implemented `native/src/cli_hook.rs` (245 lines, within budget), wired `PrePush` and `InstallHook` in `native/src/cli_deconvolute.rs`, registered in `complexity-budgets.json`, documented in `SKILL.md`, and verified with `cargo test --test cli_hook` (6/6 passed) and `cargo test --test budgets` (10/10 passed).
- **Owned Files:**
  - `native/src/cli_hook.rs` (new: $\le 400$ physical lines, $\le 300$ units, $\le 80$ fn lines)
  - `native/src/cli_deconvolute.rs` (add `PrePush` and `InstallHook` subcommands)
  - `native/complexity-budgets.json` (register `cli_hook.rs`)
  - `native/tests/cli_hook.rs` (new integration test suite)
  - `.agents/skills/deconvolute/SKILL.md` (document hook usage and install command)
- **Implementation:**
  1. Create `native/src/cli_hook.rs`:
     - `run_pre_push`: inspects stdin lines or arguments (`local_ref`, `local_sha`, `remote_ref`, `remote_sha`).
     - Bypasses check if deleting a branch (`local_sha == 00000000...` or `(delete)`) or if `LESSCODE_SKIP_HOOK=1` is set.
     - Performs complexity budget verification (`complexity-budgets.json`) against native source root; rejects push with code 1 if violated.
     - If pushing a refactoring branch (`refactor/*`, `deconvolute/*`), checks AST state boundaries on modified files and verifies zero forbidden dynamic gaps.
     - `run_install_hook`: writes `.git/hooks/pre-push` script invoking `lesscode-core deconvolute pre-push "$@"`.
  2. Wire `PrePush` and `InstallHook` in `native/src/cli_deconvolute.rs`.
  3. Register `cli_hook.rs` in `native/complexity-budgets.json`.
  4. Add integration tests in `native/tests/cli_hook.rs`.
  5. Update `SKILL.md` with hook instructions.
- **Verification:**
  - `cargo test --manifest-path native/Cargo.toml --test cli_hook`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Delete `cli_hook.rs`, `cli_hook.rs` test, and revert `cli_deconvolute.rs`, `complexity-budgets.json`, and `SKILL.md`.

---

### Task 7: End-to-End Dogfooding Validation & Verification on Isolated Copy [x]
- **Requirements:** REQ-1.1..1.4, REQ-2.1..2.5, REQ-3.1..3.4, REQ-4.1..4.3, REQ-5.1..5.4, REQ-6.1..6.3, REQ-7.1..7.5
- **Dependencies:** Tasks 1, 2, 3, 4, 5, 6
- **Status:** Completed. Verified end-to-end on clean clone `..\lesscode-dogfood-copy`: full nextest suite passed 352/352 tests (100%), complexity budgets verified clean across 38 files (0 exceeded, 0 unlisted, 0 missing), pre-push hook installed and verified (6/6 tests passed), and validation report recorded in `validation-report.md`.
- **Owned Files:**
  - `.kiro/specs/dogfood-self-application/validation-report.md`
  - `.kiro/specs/dogfood-self-application/spec.json`
- **Implementation:**
  1. Run the new `lesscode-core deconvolute` commands against the dogfood copy (`..\lesscode-dogfood-copy`).
  2. Verify pre-push hook behavior (`pre-push`, `install-hook`).
  3. Run the entire native test suite (`cargo nextest run --manifest-path native/Cargo.toml`) and verify 100% pass rate.
  4. Run native complexity budget validation (`cargo test --manifest-path native/Cargo.toml --test budgets`).
  5. Write `validation-report.md` recording all evidence, receipts, and final disposition.
  6. Update `spec.json` to complete.
- **Verification:**
  - `cargo nextest run --manifest-path native/Cargo.toml`
  - `cargo test --manifest-path native/Cargo.toml --test budgets`
- **Rollback:** Revert `validation-report.md` and `spec.json`.
