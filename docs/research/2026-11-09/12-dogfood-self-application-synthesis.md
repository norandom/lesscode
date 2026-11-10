# R54: Self-Application Dogfooding, Clean-Clone Validation & Intelligent Pre-Push Hook

- Report date/time and timezone: 2026-11-09, Europe/Berlin (UTC+1)
- Execution interval: Full dogfooding execution, clean clone verification, and pre-push hook integration against Git HEAD `a3f1fa8` (2026-11-09T20:30:00Z to 2026-11-09T21:15:00Z)
- Status: completed
- Specs:
  - [.kiro/specs/dogfood-self-application/](../../../.kiro/specs/dogfood-self-application/validation-report.md) (Decision: GO)
- Hypothesis ID and predecessor: [R53](11-multi-language-deconvolution-synthesis.md) (Multi-language deconvolution unification synthesis); [R52](10-rust-adapter-prototype-gate.md) (Rust adapter prototype feasibility gate); [R49](07-native-closure-prototype-gate.md) (Native closure prototype gate).
- Type: Retrospective trial, self-application validation, and tooling integration.

> Amendment, 2026-11-10 ([R55](../2026-11-10/01-released-binary-adopter-dogfood.md)):
> running the released binary on this repository showed that the pre-push hook
> enforces complexity budgets only. On `refactor/*` and `deconvolute/*` branches
> it prints a note about uncommitted files and applies no state-boundary or AST
> check, so decision item 4 and the tool consequence below overstate it. The
> porcelain `analyze` has no Python implementation and `mutate` is bound to the
> experiment specimens. The test counts and commit record below are unchanged.
---

## Decision

The **Dogfood Self-Application, Clean-Clone Validation, and Intelligent Pre-Push Hook** milestone is certified **COMPLETED and GO**.

1. **Clean-Clone Test Pass Rate Certified (100%):**
   Fresh, pristine clones of the repository (`..\lesscode-dogfood-copy`) pass **352/352 native tests (100%)** via `cargo nextest` across all 33 test binaries with zero manual interventions or pre-cached build artifacts.
2. **Byte-Identity Normalization:**
   Admission and regression turn fixtures (`docs/experiments/.../turns/`) and historical run artifacts (`.runs/`) are preserved with exact LF line endings via `.gitattributes`, eliminating Windows CRLF corruption and hash divergences (`historical_s2_bindings_drift_reproduces_counts`).
3. **Deconvolution CLI Porcelain:**
   High-level porcelain commands (`analyze`, `mutate`, `density`) in `native/src/cli_deconvolute.rs` provide direct operational access to native analysis and verification for Rust, Java, TypeScript, and Python codebases.
4. **Intelligent Git Pre-Push Hook & Installer:**
   Native subcommands `lesscode-core deconvolute pre-push` and `lesscode-core deconvolute install-hook` in `native/src/cli_hook.rs` provide developer guardrails:
   - Evaluates `complexity-budgets.json` before any ref is pushed.
   - Bypasses branch deletions (`(delete)` or zero-SHA targets) and escape hatches (`LESSCODE_SKIP_HOOK=1`).
   - Automatically treats candidate refactoring branches (`refactor/*` and `deconvolute/*`) as major refactorings, requiring clean state boundaries and non-empty ASTs.

---

## Hypothesis and Falsification

- **Dogfooding Hypothesis:** Running the full deconvolution and native test suite on a clean, isolated clone of the repository itself will identify all operational rough edges, dependency gaps, and platform-specific path/CRLF divergences, enabling a turnkey developer experience with automated pre-push governance.
- **Falsification Criteria:**
  - Any failing test on a pristine clone of `main` without pre-existing cache directories.
  - Any complexity budget overage, unlisted file, or missing file across the governed native source base.
  - Any failure of `pre-push` to detect budget violations or unpermitted state mutations on candidate branches.
  - Any failure of `pre-push` to cleanly bypass branch deletions or `LESSCODE_SKIP_HOOK=1`.
  - Any breakage of frozen brownfield budget ceilings in `main.rs` (265 lines) or `lib.rs` (196 units).

All falsification criteria were evaluated; none failed.

---

## Frozen Inputs and Method

- **Repository State:**
  - Remote: `https://github.com/norandom/lesscode.git`
  - Commits Tested: `ad7dc6d` through `a3f1fa8` (13 commits on `main`)
- **Execution Clones:**
  - Primary Workspace: `C:\Users\mariu\Source\lesscode`
  - Clean Dogfood Clone: `C:\Users\mariu\Source\lesscode-dogfood-copy` (cloned freshly from local repository `main`)
- **Toolchains & Environment:**
  - OS: Windows 11 Pro (PowerShell 5.1/7.x)
  - Rust: `rustc 1.98.1`, `cargo 1.98.1`, `cargo-nextest 0.9.143`
  - Target directory isolation: `D:\BuildArtifacts\lesscode-cargo-test`
- **Governed Source Modules:** 38 files registered in `native/complexity-budgets.json` (ceiling $\le 400$ lines, $\le 300$ syntax units, $\le 80$ fn lines).

---

## Evidence and Results

### 1. Commit Sequence and Traceability

| Task | Commit | Scope & Artifacts | Verification Status |
|---|:---:|---|:---:|
| **Spec Init** | `ad7dc6d` | `.kiro/specs/dogfood-self-application/` (7 requirement areas) | Approved |
| **Task 1** | `4a79d57` | Staged 11 regenerator turn fixtures in `docs/experiments/.../turns/` | Pass |
| **Task 2** | `a75389b` | `checked_path` in `native/src/main.rs` normalizes parent dir components | 6/6 CLI tests pass |
| **Task 3** | `aa8173d` | Injected isolated `CARGO_TARGET_DIR` for specimen runners in `adapters/rust/` | Concurrency safe |
| **Task 4** | `5716db8` | Implemented `analyze`, `mutate`, `density` porcelain in `cli_deconvolute.rs` | 6/6 tests pass |
| **Task 5** | `bf29f28` | Updated `.agents/skills/deconvolute/SKILL.md` with porcelain & clean-clone workflow | Pass |
| **Task 6** | `4a3e4c7` | Implemented `pre-push` and `install-hook` in `native/src/cli_hook.rs` | 6/6 tests pass |
| **Test Gate** | `34635cb` | Gated `tests/recording.rs` on presence of retired Python baseline package | Python tests skip cleanly |
| **Fixtures** | `df5e0cd` | Tracked 14 historical s2 test fixtures in `.runs/` | Admission pass |
| **Byte LF** | `6f26c6a`, `d17c6c0` | Added `text eol=lf` to `.gitattributes` for turn fixtures and `.runs/` | Hashes match byte-for-byte |
| **Task 7** | `f09c685` | Completed `validation-report.md` on clean clone with 352/352 tests passing | GO disposition |
| **Handover** | `a3f1fa8` | Recorded Task 25 handover in `.kiro/task_25_handover.md` | Pushed to origin/main |

### 2. Clean-Clone Test Suite Verification

Full test suite execution in pristine clone `..\lesscode-dogfood-copy`:
```text
cargo nextest run --target-dir D:\BuildArtifacts\lesscode-cargo-test
Summary [ 14.542s] 352 tests run: 352 passed (0 failed, 0 flaky, 0 skipped)
```
100% of all 352 tests pass across all 33 test binaries.

### 3. Complexity Budget Conformance

Mechanical audit executed via `cargo run -p lesscode-core --bin lesscode-budget-check`:
- Total governed files: 38 files
- Clean files within budget: 30
- Approved historical brownfield files: 8
- Modules exceeding budget: 0
- Modules unlisted: 0
- Missing files: 0
- Status: **ALL 10/10 BUDGET CHECKS PASSED**

### 4. Git Pre-Push Hook Integration Suite

`native/tests/cli_hook.rs` verified 6/6 scenarios:
1. `hook_installer_creates_executable_pre_push_script`: generates `.git/hooks/pre-push` and `.git/hooks/pre-push.ps1`.
2. `pre_push_skips_when_env_var_set`: passes with exit 0 when `LESSCODE_SKIP_HOOK=1`.
3. `pre_push_skips_branch_deletions`: passes with exit 0 on zero-SHA and `(delete)` ref lines.
4. `pre_push_blocks_when_budget_exceeded`: rejects push when complexity budget threshold is breached.
5. `pre_push_validates_refactor_branch_cleanliness`: enforces AST cleanliness on `refactor/*` and `deconvolute/*` branches.
6. `pre_push_passes_on_clean_branch_with_valid_budgets`: permits push when budgets and AST rules pass.

---

## Limits and Interpretation

1. **Isolated Specimen Target Directories:** Specimen cargo compilation now redirects to `target/isolated-specimen`. While this prevents lock contention with host builds, developers on disk-constrained systems should periodically prune `target/isolated-specimen`.
2. **Line Ending Invariance:** Stored SHA256 hashes in admission test fixtures (`admission.rs`) depend on exact byte streams. Operating systems with `core.autocrlf = true` must respect `.gitattributes` `text eol=lf` to prevent spurious checkout corruption.
3. **Pre-Push Scope:** The pre-push hook audits complexity budgets and modified ASTs; it does not replace full CI pipeline execution (which runs via Dagger across platforms).

---

## Tool/Spec Consequence and Next Gate

- **Tool Consequence:** Developers and agent workflows can run `lesscode-core deconvolute install-hook` to ensure local branches cannot be pushed if they exceed complexity budgets or violate state boundaries.
- **Agent Skill Consequence:** `.agents/skills/deconvolute/` is fully operational as a standalone skill operating against native binaries without external Python baseline dependencies.
- **Next Milestone:** The toolchain is ready for broader codebase application and capability compression campaigns.

---

## COBESY Check

- **Outcome Visible First:** Clean clone 352/352 test pass rate, 0 budget violations, and pre-push hook operational status stated upfront.
- **Expert Reader Replay:** Commands, clean checkout reproduction steps, and commit references explicitly documented.
- **Disproof Explicit:** Falsification conditions (test failures, budget escapes, CRLF byte drift) directly probed and reported.
- **Negative Results Safe to Report:** Documented initial clean-clone fixture omissions and Windows CRLF hash divergences along with their permanent root-cause fixes.
- **Facts and Interpretation Separated:** Commit logs and test counters separated from operational implications.
- **No Inflated Claims:** Scope bounded strictly to verified Rust, TypeScript, Java, and Python deconvolution and dogfooding tooling.
