# Lesscode

[![DOI](https://zenodo.org/badge/1440672292.svg)](https://doi.org/10.5281/zenodo.23958363)
[![Release](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/norandom/lesscode/releases/tag/v1.0.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

The goal is substantial code simplification while preserving features. In grown
codebases, custom implementations, boilerplate and supporting layers can often
be replaced by capabilities already provided by the project's framework,
libraries or language. Lesscode aims to help identify those opportunities and
check the resulting changes.

The work is to move responsibility into suitable existing capabilities: use a
framework shorthand, reuse a library operation, or express the same behavior
with clearer idiomatic code. Success means less code and structure to maintain
for the same features and public contracts. Moving custom code into another
module, hiding it behind a wrapper, or dropping a feature does not establish
that result. A replacement's dependencies, configuration and behavioral
differences also count when judging whether it simplifies the project.

## What works today

Lesscode provides a high-performance native core (`lesscode-core`) written in Rust
that drives static and dynamic deconvolution across four language substrates:
**Python**, **TypeScript**, **Java**, and **Rust**.

The engine deconvolutes coupled imperative and stateful code into isolated pure
computational cores and thin imperative shells. It enforces:
- **State-Frame Analysis**: AST-level detection of dynamic escape hatches, side-effect
  write sites, and frame boundary validation.
- **Bounded Finite-Domain Cartesian Verification**: Concolic / exhaustive verification
  proving state equivalence between original pure cores and candidate rewrites.
- **Adverse Mutation Sensitivity**: High-order mutation operators (operator replacement,
  boundary shifts, exception drops, stream truncations) to verify that regression
  test suites achieve $\ge 80\%$ (or $100\%$) kill rates.
- **Mechanical Density Gating**: Exact canonical token and statement metrics distinguishing
  substantial whole-responsibility compression ($\ge 20\%$ statement reduction, $\ge 50$ physical lines)
  from cosmetic reformatting or line splitting.
- **Repository Complexity Budgets**: Automated CI and pre-push gating enforcing frozen
  line count, syntax unit, and function length ceilings.

Native analysis uses pinned open-source dependencies; no hosted account, commercial
analyzer, or external solver service is required. The core leaves specimens unchanged
and writes versioned JSON receipts outside the source roots.

Native checks pass across Windows, Linux, and macOS. The historical Python
prototype (`lesscode-baseline`) was formally retired and archived at commit `fd0b5b9`.

## Deconvolution CLI (`lesscode-core`)

The `lesscode-core` executable provides porcelain commands for standalone analysis,
mutation generation, density measurement, and automated git hook integration.

### Building `lesscode-core`

Build with Rust 1.85+ (tested on Rust 1.98.1):

```bash
cargo build --release --manifest-path native/Cargo.toml
```

The compiled binary will be placed at `native/target/release/lesscode-core`
(or `lesscode-core.exe` on Windows).

### 1. State-Frame & Boundary Analysis

Inspect an implementation file to detect side-effecting write sites, unauthorized
dynamic forms, and state boundary violations:

```bash
lesscode-core deconvolute analyze --lang rust --file src/lib.rs --out receipt.json
lesscode-core deconvolute analyze --lang java --file src/main/java/App.java --out receipt.json
lesscode-core deconvolute analyze --lang typescript --file src/index.ts --out receipt.json
lesscode-core deconvolute analyze --lang python --file src/module.py --out receipt.json
```

The generated `receipt.json` records detected write sites, purity verdicts, and
structural invariants.

### 2. Adverse Mutant Generation

Generate adverse mutants to test whether your test suite detects semantic regressions:

```bash
lesscode-core deconvolute mutate --lang rust --file src/lib.rs --out-dir ./mutants
lesscode-core deconvolute mutate --lang java --file src/App.java --out-dir ./mutants
lesscode-core deconvolute mutate --lang typescript --file src/index.ts --out-dir ./mutants
lesscode-core deconvolute mutate --lang python --file src/module.py --out-dir ./mutants
```

### 3. Mechanical Density Comparison

Compare an original baseline against a deconvolution candidate:

```bash
lesscode-core deconvolute density \
  --lang typescript \
  --baseline specimens/original.ts \
  --candidate specimens/candidate.ts
```

Outputs canonical statement counts, physical line deltas, and whether the change qualifies
as substantial whole-responsibility compression.

### 4. Complexity Budget Enforcement

Audit the codebase against frozen structural ceilings (`complexity-budgets.json`):

```bash
lesscode-core budgets --config .kiro/complexity-budgets.json --root .
```

Rejects code when physical lines, syntax units, or function lengths exceed frozen limits.

### 5. Git Pre-Push Hook Integration

Install the intelligent pre-push hook:

```bash
lesscode-core deconvolute install-hook
```

The hook automatically:
- Validates complexity budgets before pushing to remote repositories.
- Detects refactoring branches (`refactor/*` and `deconvolute/*`), validating AST state-boundary cleanliness.
- Bypasses branch deletions and honors `LESSCODE_SKIP_HOOK=1` for intentional emergency overrides.

## Developer Refactoring Workflow

When simplifying or deconvoluting code:
1. **Work on a clean checkout / copy**:
   Developers run deconvolution and trials on a clean checkout or isolated worktree of the repository:
   ```bash
   git clone <repo-url> my-refactor-copy
   cd my-refactor-copy
   git checkout -b refactor/simplify-subsystem
   ```
2. **Execute deconvolution & test suites**:
   Run `lesscode-core deconvolute analyze` and `lesscode-core deconvolute mutate`. Ensure test suites kill 100% of adverse mutants and Cartesian verification passes without divergence.
3. **Audit complexity budgets**:
   Verify that candidate changes satisfy `.kiro/complexity-budgets.json`.
4. **Push branch for major refactoring review**:
   Push the refactoring branch. The pre-push hook validates budgets and state cleanliness. The branch is then reviewed as a major refactoring pull request against full test suites.

## What you get

| Review question | Evidence |
| --- | --- |
| Did the declared behavior checks pass? | Baseline and candidate outcomes, zero-divergence test receipts, and diagnostic logs |
| What structural changes occurred? | Native statement, decision and function counts, with pure cores and imperative shells separated |
| How much source changed? | Complete-capability physical and logical counts, with tests, support and dependency changes distinguished |
| Can someone inspect the result? | Versioned JSON, a readable summary, raw outputs and a hash-checked artifact index |
| Was the comparison held constant? | Source fingerprints, frozen profile, check inventory and recorded tool identities |

`accepted` means the candidate passed the frozen target and declared checks. It does not prove
equivalence or certify code quality. Missing required evidence prevents
acceptance; an observed behavior failure cannot be offset by fewer lines.

## Experimental Trials & Research Log

The repository maintains an empirical, dated research record ([docs/research/README.md](docs/research/README.md))
documenting all experiments, negative results, and hypothesis evolutions:

- [Proxy Routing Trial](docs/experiments/proxy-routing-trial/README.md): Removes 23 module lines and 16 statements with 1,172 tests passing.
- [ATS Bilingual Reporting](docs/experiments/ats-report/README.md): Reduces 384 lines to 202 and 298 LLOC to 135 across 8,820 frozen cases.
- [Active CPU/NPU Backend](docs/experiments/npu-backends/README.md): 171 to 101 owned physical lines, 68 to 47 LLOC, 3,168 matching generated cases.
- [Outcome Hotspot Trial](docs/experiments/outcome-hotspot/README.md): CCN 54 to 45, 20,258 differential cases, Z3-verified boundary.
- [Dogfooding Self-Application (R54)](docs/research/2026-11-09/12-dogfood-self-application-synthesis.md): Verified the deconvolution core on Lesscode itself with 100% test pass rate (352/352 nextest tests) and 0 budget violations across 38 governed files.

## Technical Documentation

- [Documentation Index](docs/README.md): Organization of all project documentation.
- [Dated Research Log](docs/research/README.md): Complete chronological record of trials, hypotheses, and evidence.
- [Native Core Documentation](native/README.md): Rust crate architecture, modules, and testing guidelines.
- [Agent Deconvolute Skill](.agents/skills/deconvolute/SKILL.md): Agentic instructions for automated deconvolution.
- [CI Validation](docs/ci-validation.md): Cross-platform Dagger and native runner configurations.

## Where the project is going

Following the v1.0.0 milestone and multi-language substrate delivery:
1. **Slice B — Probabilistic Turn Refinement & MIPROv2 Optimization**:
   Addressing LLM and inference quality drift by wrapping prompt execution in DSPy-style multi-turn `Refine` / `BestOfN` controllers, compiling prompts offline with MIPROv2 against a deterministic two-tier reward function (`StaticAssertionGate` $\to$ `RuntimeOracleGate`).
2. **Deterministic Rewrites & Synthesis**:
   Extending concolic extraction and bounded synthesis across broader language semantics and idiomatic transformations.

## License

Lesscode is licensed under the [MIT License](LICENSE). Third-party dependencies
and fixtures retain their own licenses and notices; see the
[native dependency inventory](native/LICENSES.md).

## Citation

Releases are archived on Zenodo; the DOI badge above resolves to the latest
archived release. Citation metadata is in [CITATION.cff](CITATION.cff).
