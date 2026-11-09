# Requirements Document

## Introduction
This specification defines the functional requirements and acceptance criteria for dogfooding the `lesscode-core` substrate and `deconvolute` skill on an isolated copy of the `lesscode` repository itself, cataloging all operational rough edges, and implementing the required tooling corrections. Dogfooding self-application ensures that agents, developers, and operators have an integrated, ergonomic, and reliable CLI and execution toolchain when performing deconvolution across real codebases.

## Boundary Context
- **In scope**:
  - Establishing and maintaining isolated specimen workspace clones for dogfooding trials.
  - Adding unified deconvolution CLI porcelain commands to `lesscode-core` (`analyze`, `mutate`, `density`).
  - Hardening path traversal validation in `lesscode-core` to cleanly support external/sibling specimen roots without false-positive directory traversal rejections.
  - Eliminating build lock contention by isolating build and target directories (`CARGO_TARGET_DIR`) during specimen test runs.
  - Ensuring complete repository fixture integrity so that clean checkouts and isolated clones pass 100% of the test suite.
  - Synchronizing the `.agents/skills/deconvolute/` skill with the production native CLI porcelain and machine-readable JSON receipts.
- **Out of scope**:
  - In-place mutation or destructive trial modifications to the active `main` workspace.
  - Altering the mathematical definitions of complexity budgets or loosening adverse mutant kill thresholds ($K/N = 1.0$).
  - Modifying domain logic in non-tooling native modules outside the identified rough edges.
- **Adjacent expectations**:
  - Pinned Rust toolchain 1.98.1 and `cargo-nextest` 0.9.143 are available in the host environment.
  - The repository's existing 338 native tests remain 100% passing across all changes.

## Requirements

### Requirement 1: Isolated Specimen Workspace and Dogfood Sandbox
**Objective:** As a toolchain operator, I want dogfooding experiments and trial mutations to run in an isolated clone of the repository, so that experimental code changes never corrupt or dirty the active working tree.

#### Acceptance Criteria
1. When initiating a dogfooding trial, the Toolchain Runner shall verify that the target specimen root is isolated from the active repository working tree.
2. If a user or tool targets the active repository root as a candidate mutation destination, the Toolchain Runner shall reject the operation with an error.
3. While operating on the isolated specimen copy, the Toolchain Runner shall preserve all original files in the active repository without modification.
4. The Toolchain Runner shall produce self-application trial receipts and evidence artifacts in external directories outside both source roots.

### Requirement 2: Deconvolution CLI Porcelain Commands
**Objective:** As an agent or developer executing a deconvolution workflow, I want direct CLI subcommands for AST analysis, adverse mutation, and density scoring, so that I can execute deconvolution steps without writing custom harness glue code.

#### Acceptance Criteria
1. When a user runs `lesscode-core analyze --lang <language> --source <file>`, the CLI shall perform static effect and dynamic gap scanning and output a structured JSON analysis report.
2. Where an optional frame specification (`--frame-spec <path>`) is supplied to `analyze`, the CLI shall validate write sites against permitted locations and report any violations.
3. When a user runs `lesscode-core mutate --lang <language> --source <file> --out-dir <dir>`, the CLI shall generate mutants for all mandatory adverse operators and write them to the specified directory.
4. When a user runs `lesscode-core density --lang <language> --baseline <base> --candidate <cand>`, the CLI shall compute canonical line differences, physical line counts, and AST statement differences, emitting an admission decision (`pass`, `fail`, or `marginal`).
5. If an unsupported language identifier is passed to any deconvolution CLI command, the CLI shall exit with an invalid input error code (exit code 3) and list supported runtimes.

### Requirement 3: Cross-Workspace Path Resolution and Traversal Safety
**Objective:** As a developer running tools across adjacent repositories or sibling specimen clones, I want relative and absolute paths to external specimen roots to resolve cleanly, so that valid paths are not falsely rejected as malicious path traversals.

#### Acceptance Criteria
1. When a user provides a relative path to an external specimen root (such as `..\lesscode-dogfood-copy`), the Path Resolver shall normalize the path before component inspection.
2. If a resolved path legitimately points to an accessible directory outside the current working directory, the Path Resolver shall allow the path when it is not a forbidden symlink or reparse point.
3. If an input path attempts path traversal to escape permitted filesystem boundaries via malicious sequences, the Path Resolver shall reject the path with an invalid input error.
4. The Path Resolver shall support Windows drive prefixes, UNC paths, and forward-slash path separators consistently.

### Requirement 4: Repository Test Fixture Integrity
**Objective:** As a maintainer or CI system, I want all required test fixtures and experiment artifacts committed to git, so that clean clones and isolated worktrees pass all tests without missing file errors.

#### Acceptance Criteria
1. When `cargo nextest` runs on a fresh, clean clone of the repository, all tests including `historical_s2_bindings_drift_reproduces_counts` shall pass without missing file errors.
2. The Repository Manifest shall ensure all test-referenced artifacts in `docs/experiments/` are tracked in version control.
3. If an integration test references an external artifact, the test suite shall assert the artifact's presence and report a clear error pointing to the expected path if missing.

### Requirement 5: Specimen Test Execution Isolation and Build Concurrency
**Objective:** As an automated test supervisor, I want specimen test runs to use dedicated, isolated target directories, so that background builds and specimen test executions do not block on global build directory file locks.

#### Acceptance Criteria
1. When the Rust specimen runner invokes `cargo nextest` or `cargo test` against a specimen workspace, the Specimen Supervisor shall set an isolated target directory for that run.
2. While a specimen test execution is in progress, the Specimen Supervisor shall prevent lock collisions with the host workspace's global `CARGO_TARGET_DIR`.
3. If a specimen test process terminates abnormally or exceeds its timeout limit, the Specimen Supervisor shall clean up spawned child processes and report a failure receipt.
4. The Specimen Supervisor shall stream execution progress and write raw test receipts to the external evidence directory.

### Requirement 6: Agent Skill Ergonomics and Machine-Readable Receipts
**Objective:** As an autonomous coding agent, I want the `deconvolute` skill documentation and CLI outputs to provide structured, predictable JSON receipts, so that I can programmatically parse trial results and make gating decisions.

#### Acceptance Criteria
1. When any deconvolution CLI command completes successfully, the CLI shall write a machine-readable JSON receipt containing schema version, timestamp, inputs, and structured metrics.
2. The Agent Skill documentation in `.agents/skills/deconvolute/SKILL.md` shall accurately document all available CLI subcommands, arguments, binary location discovery, and exit code semantics.
3. If a deconvolution candidate fails the mechanical density gate or adverse mutation threshold, the CLI receipt shall explicitly detail which specific threshold was violated.

### Requirement 7: CLI-Driven Git Pre-Push Hook for Size Governance and Refactoring Discipline
**Objective:** As a developer or autonomous agent pushing candidate branches, I want an intelligently designed Git pre-push hook powered by the CLI, so that accidental regressions in complexity budgets or incomplete deconvolution refactorings are intercepted before reaching the remote repository.

#### Acceptance Criteria
1. When invoked via `git push` or directly as `lesscode-core deconvolute pre-push`, the CLI hook shall inspect the incoming ref lines from standard input or CLI arguments.
2. If the push is a branch deletion (`local_sha == 0000000000000000000000000000000000000000` or `(delete)`), or if `LESSCODE_SKIP_HOOK=1` is set, the CLI hook shall exit 0 immediately without performing blocking validations.
3. The CLI hook shall verify codebase complexity budgets against `complexity-budgets.json`; if any physical line, logical unit, or function length ceiling is exceeded, the hook shall emit actionable diagnostics and exit 1 (blocking the push).
4. When pushing candidate refactoring or deconvolution branches (e.g. `refactor/*`, `deconvolute/*`), the CLI hook shall treat the branch as a major refactoring, verifying AST state-boundary cleanliness and test oracle kill discipline.
5. The CLI shall provide a `lesscode-core deconvolute install-hook [--git-dir <path>]` command that writes a standard, portable Git `pre-push` hook executable in the target repository's `.git/hooks/` directory.
