# Validation Report: Dogfood Self-Application & Pre-Push Hook

## 1. Executive Summary

- **Specification:** `dogfood-self-application`
- **Date:** 2026-11-09
- **Status:** **PASSED / COMPLETE (GO)**
- **Scope:** Dogfooding `lesscode-core` substrate and `deconvolute` skill on an isolated specimen clone (`..\lesscode-dogfood-copy`), correcting operational rough edges, implementing CLI porcelain (`analyze`, `mutate`, `density`), establishing clean Git pre-push hook governance (`pre-push`, `install-hook`), and achieving 100% test reproducibility on fresh clean checkouts.

---

## 2. Requirement Verification Matrix

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| **REQ-1** | Isolated Specimen Workspace Sandbox | **PASS** | Dogfood trial executed in sibling clone `..\lesscode-dogfood-copy`; active repository remained unmutated; build artifacts directed to external directory `D:\BuildArtifacts\`. |
| **REQ-2** | Deconvolution CLI Porcelain Commands | **PASS** | `lesscode-core deconvolute` subcommands implemented (`analyze`, `mutate`, `density`) in `native/src/cli_deconvolute.rs`; verified across Rust, Java, TypeScript, and Python; emits schema-valid `-v1` JSON receipts. |
| **REQ-3** | Cross-Workspace Path Resolution | **PASS** | `checked_path` in `native/src/main.rs` updated to allow canonical parent pops (`..`) while rejecting directory traversal escapes; verified with `tests/cli.rs::relative_parent_paths_resolve_cleanly_when_valid_and_reject_root_escapes`. |
| **REQ-4** | Repository Test Fixture Integrity | **PASS** | Tracked 11 missing turn fixtures under `docs/experiments/.../turns/` and 14 historical s2 fixtures under `.runs/`; configured `text eol=lf` in `.gitattributes` to prevent CRLF corruption; `historical_s2_bindings_drift_reproduces_counts` passes in clean checkout. |
| **REQ-5** | Specimen Test Execution Isolation | **PASS** | Injected isolated `CARGO_TARGET_DIR` in `native/src/adapters/rust/mod.rs` preventing file lock contention on global cargo targets. |
| **REQ-6** | Agent Skill Ergonomics & Receipts | **PASS** | Updated `.agents/skills/deconvolute/SKILL.md` with binary location discovery, CLI porcelain commands, JSON receipt schemas, pre-push hook usage, and the clean-clone branch refactoring workflow. |
| **REQ-7** | Intelligent CLI Git Pre-Push Hook | **PASS** | Implemented `native/src/cli_hook.rs` providing `pre-push` and `install-hook`; bypasses deletions and `LESSCODE_SKIP_HOOK=1`; enforces complexity budgets; treats `refactor/*` and `deconvolute/*` branches as major refactorings. |

---

## 3. Test Suite & Evidence Log

### Test Execution Summary
- **Target Environment:** Windows 10 x64, PowerShell, Rust 1.98.1, `cargo-nextest` 0.9.143
- **Test Command (Specimen Clone):**
  ```powershell
  cargo nextest run --manifest-path ..\lesscode-dogfood-copy\native\Cargo.toml --target-dir D:\BuildArtifacts\lesscode-cargo-test
  ```
- **Outcome:** **352 passed, 0 failed, 0 skipped** across 35 test binaries.

### Subsystem Verification Runs
1. **Pre-Push Hook Suite (`native/tests/cli_hook.rs`):**
   - `pre_push_skips_when_env_flag_set` -> **ok**
   - `pre_push_skips_on_branch_deletion` -> **ok**
   - `pre_push_passes_clean_codebase_budgets` -> **ok**
   - `pre_push_rejects_violated_budgets` -> **ok**
   - `install_hook_creates_portable_git_hook_script` -> **ok**
   - `refactor_branch_detection_triggers_refactoring_path` -> **ok**
   - **Result:** 6/6 passed.

2. **Deconvolution CLI Suite (`native/tests/cli_deconvolute.rs`):**
   - `deconvolute_help_lists_all_subcommands` -> **ok**
   - `analyze_subcommand_flags_dynamic_gaps` -> **ok**
   - `analyze_subcommand_reports_clean_pure_rust` -> **ok**
   - `mutate_subcommand_generates_adverse_mutants` -> **ok**
   - `density_subcommand_evaluates_structural_reduction` -> **ok**
   - `unsupported_language_fails_with_exit_code_3` -> **ok**
   - **Result:** 6/6 passed.

3. **Admission Suite (`native/tests/admission.rs`):**
   - `historical_s2_bindings_drift_reproduces_counts` -> **ok** (11 verified, 5 mismatch, 1 missing, 4 no_bound_sidecar) in both host and clean dogfood checkout.
   - **Result:** 9/9 passed.

---

## 4. Complexity Budget Conformance Audit

Audited via `lesscode-core budgets --config native/complexity-budgets.json --root native/src`:

| Governed File | Physical Lines | Unit Budget | Longest Fn Lines | Status |
|---------------|----------------|-------------|------------------|--------|
| `native/src/cli_deconvolute.rs` | 365 / 400 | 107 / 300 | 50 / 80 (`run_analyze`) | **within_budget** |
| `native/src/cli_hook.rs` | 245 / 400 | 104 / 300 | 38 / 80 (`run_install_hook`) | **within_budget** |
| `native/src/main.rs` | 262 / 265 | 81 / 82 | 60 / 65 (`run`) | **disposed (within ceiling)** |
| `native/src/budgets.rs` | 330 / 330 | 95 / 300 | 44 / 80 (`check`) | **disposed (within ceiling)** |

- **Total Governed Files:** 38
- **Files Within Budget:** 30
- **Disposed / Frozen Ceilings:** 8
- **Files Exceeded:** **0**
- **Unlisted Files:** **0**
- **Missing Files:** **0**

---

## 5. Architectural Disposition

The `dogfood-self-application` specification is complete and verified against live code on both the active tree and an isolated checkout. All 7 requirements are fulfilled. The Git pre-push hook provides automated local enforcement of size ceilings and refactoring discipline prior to push.
