---
name: deconvolute
description: Deconvolute coupled imperative and stateful code into isolated pure computational cores and thin imperative shells across Python, TypeScript, Java, and Rust. Drives native Rust lesscode-core for state-frame analysis, bounded finite-domain Cartesian verification, adverse mutation sensitivity, and mechanical density gating.
---

# State-Boundary Deconvolution Skill

Deconvolute bloated, coupled, or highly stateful implementations by cleanly separating pure data transformations from mutable state and I/O shells. Driven natively by `lesscode-core` across **Python** (`pytest-v1`), **TypeScript** (`ts-node-v1`), **Java** (`java-junit-v1`), and **Rust** (`rust-nextest-v1`).

---

## 1. The Deconvolution Pattern

```
┌────────────────────────────────────────────────────────┐
│  IMPERATIVE SHELL (Outer Boundary)                     │
│  - Receives inputs, reads environment, handles I/O     │
│  - Confined, source-anchored write sites               │
│  - Calls Pure Core, writes state, emits events         │
└───────────────────────────┬────────────────────────────┘
                            │ (Immutable inputs)
                            ▼
┌────────────────────────────────────────────────────────┐
│  PURE CORE (Computational Kernel)                      │
│  - Deterministic transformations                       │
│  - Zero side effects, zero I/O, zero global mutations  │
│  - Exhaustively verifiable over finite input domains   │
└────────────────────────────────────────────────────────┘
```

---

## 2. Step-by-Step Procedure

### Phase 1: Responsibility Discovery & Scope Boundary
1. **Target Selection:** Identify a whole implementation responsibility (e.g. protocol encoding, state transitions, validation rules, message normalization). Avoid micro-optimizing isolated single functions.
2. **State Inventory:** Locate every state variable, dictionary mutation, field write (`self.x = ...`, `this.x = ...`, `*ptr = ...`), and external side-effect call.
3. **Draft the Frame Spec:** Declare exact permitted write sites and forbidden dynamic forms in `FrameSpec`.

### Phase 2: Shell / Core Partitioning
1. **Extract Pure Kernel:** Pull computational logic, condition trees, and data shaping into standalone pure functions taking immutable values and returning immutable results.
2. **Compress Imperative Shell:** Replace redundant intermediate variables, layered loops, and duplicate caches in the outer shell with a direct call to the pure kernel followed by atomic state assignment.

### Phase 3: Native State-Frame & Gap Analysis
Run native AST analysis to verify that the candidate does not leak side effects or rely on unanalyzable dynamic features:
- **Python:** flag `eval`, `exec`, `globals()`, unmonitored threading.
- **TypeScript:** flag `eval`, `Function()`, prototype mutations, dynamic `require`.
- **Java:** flag `Class.forName`, custom `ClassLoader`, `native`, unmonitored threads.
- **Rust:** flag `unsafe`, `dyn`, raw pointer dereference, unmonitored threads, hidden `Cell`/`Mutex`.

### Phase 4: Equivalence Verification Over Finite Domains
1. Define a bounded parameter domain `DomainSpec` covering representative inputs, empty values, zeros, extremes, and boundaries.
2. Generate Cartesian combinations (typically 100 cases).
3. Evaluate baseline vs candidate: require **0 divergences**. If divergences appear, shrink the witness to identify the minimal causal defect.
4. Run the comparison on the real baseline and the real candidate, in the project's own language and test framework. No `lesscode-core` command does this for you. A comparison of two re-implementations written for the check proves nothing about the code under review.

### Phase 5: Adverse Mutation Sensitivity
1. Let the driver plant faults in the candidate and run the project's own tests against each one (`deconvolute run`, from release 1.1.0; see `docs/driver.md`). The operators come from the language pack: negate a condition, swap a paired operator, remove or rewrite a failure signal, drop a state write, swap two adjacent checks.
2. Read the classes in the receipt. `killed` means the tests failed. `survived` means they passed: look at each survivor and decide whether a test is missing or the change makes no difference. `invalid`, `timeout` and `error` say nothing about the tests. A void run says nothing at all.
3. A mutant is killed only when a test run says so. A changed text, a hand-written witness or a mutant that did not compile is not a kill. The earlier gates of this project counted such things (R75, `docs/research/2026-11-11/01-specimen-controls-rerun-through-driver.md`); do not repeat that.
4. Faults the pack has no operator for (a changed argument, a changed literal inside a call, the order of a list, what a process prints) are not covered. If one of them matters, write the test for it by hand and say so in the review.

### Phase 6: Mechanical Density Gate
Evaluate structural reduction using `lesscode-core` (from release 1.1.1 by syntax: at least 20% fewer tokens, at least 400 tokens removed and at least 10% fewer named nodes; lines do not count, because layout changes them). The line-based rule below describes releases up to 1.1.0 and overstated reductions (R87):
- $\text{LineReduction} \ge 20\%$ and $\Delta \text{Lines} \ge 50$ canonical lines.
- $\text{StmtReduction} \ge 10\%$ and $\Delta \text{Stmts} \ge 10$ AST statements.
- Any change below these absolute or relative thresholds is rejected as marginal.

---

## 3. Native CLI Porcelain Commands

The native `lesscode-core` binary exposes first-class deconvolution commands across all supported runtimes:

### Known limits of releases 1.0.1 and 1.0.2 (R55, 2026-11-10)
Check `lesscode-core --version` first. These apply only to a binary older than 1.1.0; see `docs/research/2026-11-10/01-released-binary-adopter-dogfood.md`.
- `analyze --lang python` performs no analysis. Release 1.0.2 writes no receipt, prints a note to stderr and exits 2. Version 1.0.1 reports `"passed": true`. Do not use either as evidence.
- `mutate` uses operators bound to the experiment specimens. On other code it usually writes no mutants. Release 1.0.2 then exits 1 (the receipt shows `"mutants_generated": 0`); version 1.0.1 exits 0, so read `mutants_generated`. There is no Python generator.
- There is no `run` and no `admit-pack` command.

Still true in 1.1.0:
- The pre-push hook enforces complexity budgets only and treats every branch the same.

### From release 1.1.0: the agnostic driver
`analyze` and `mutate` are served by one engine driven by language packs for Rust, Python, TypeScript and Java, and two commands are new. Full description: `docs/driver.md`.
```powershell
# Run the project's own tests against every planted fault, in copies of the source tree:
lesscode-core deconvolute run --config <run-config.json> --work <work-dir> [--workers N] [--no-reuse] --out <receipt.json>

# Check a language pack before it is used:
lesscode-core deconvolute admit-pack --pack <pack.json> --lang <language> [--controls <controls.json> --source-root <dir>]
```
- `run` is the only command that executes a project's commands. It works in copies under `--work` and refuses a work directory that is the source root, lies inside it or contains it. Exit 0 complete, 2 errors remain, 3 void or bad input.
- `analyze` exits 1 on a declared gap or a write outside the frame specification; `mutate` exits 1 when a file has no mutant. A source that does not parse is refused with exit 3.

### Binary Discovery
Prefer an installed release binary on `PATH` (`lesscode-core --version`; install with `scripts/install.sh` or `scripts/install.ps1`). Otherwise locate the compiled binary in standard build locations:
- `$env:CARGO_TARGET_DIR/debug/lesscode-core.exe` (when custom target dir is configured)
- `native/target/debug/lesscode-core.exe` (or `target/debug/lesscode-core.exe`)
- Alternatively, invoke via Cargo: `cargo run --manifest-path native/Cargo.toml -- deconvolute <subcommand>`

### `analyze` Subcommand
Scans source code for dynamic gaps, AST syntax patterns, and write sites against an optional frame specification:
```powershell
lesscode-core deconvolute analyze --lang <rust|java|typescript|python> --source <path-to-source> [--frame-spec <path-to-spec>] [--out <receipt.json>]
```
- **Inputs:** Target source file and language identifier (`rust`, `java`, `typescript`, `python`).
- **Optional Frame Spec:** JSON file specifying permitted write sites and forbidden dynamic forms.
- **Output:** Emits structured JSON detailing dynamic gaps, write sites count, and conformance status.

### `mutate` Subcommand
Generates mandatory adverse mutants to stress-test candidate test oracle sensitivity:
```powershell
lesscode-core deconvolute mutate --lang <rust|java|typescript|python> --source <path-to-source> --out-dir <output-dir> [--out <receipt.json>]
```
- **Inputs:** Target source file and target output directory.
- **Output:** Writes individual mutant source files (`mutant_1_...`, `mutant_2_...`) and outputs a JSON receipt summarizing generated mutations.
- **Verification Rule:** `mutate` only writes mutant files. Running them against the test suite is `deconvolute run` or your own harness. Review every survivor; do not report a kill that no test run produced.

### `density` Subcommand
Computes canonical and AST statement reductions between baseline and candidate implementations:
```powershell
lesscode-core deconvolute density --lang <rust|java|typescript|python> --baseline <baseline-file> --candidate <candidate-file> [--out <receipt.json>]
```
- **Inputs:** Path to baseline source and path to deconvoluted candidate source.
- **Output:** JSON receipt evaluating physical line delta, canonical line delta/percentage, and AST statement delta/percentage.
- **Decision:** Emits `pass` if thresholds ($\ge 20\%$ and $\ge 50$ lines, $\ge 10\%$ and $\ge 10$ statements) are met, or `marginal`/`fail` otherwise.

### Intelligent Git Pre-Push Hook (`pre-push` & `install-hook`)
Interprets push refs and verifies complexity ceilings before a remote push:
```powershell
# Install the pre-push hook into .git/hooks:
lesscode-core deconvolute install-hook [--git-dir .git] [--force]

# Execute pre-push validation directly:
lesscode-core deconvolute pre-push [--local-ref <ref>] [--local-sha <sha>]
```
- **Automatic Bypasses:** Deletions of remote branches and `LESSCODE_SKIP_HOOK=1` exit 0 immediately.
- **Complexity Budgets:** Intercepts regressions in `complexity-budgets.json` before code reaches the remote.
- **Refactoring Branches (`refactor/*`, `deconvolute/*`):** From release 1.0.2, treated like any other branch: budget check only (version 1.0.1 announces them and prints a note when tracked files are uncommitted). No state-boundary or mutation check runs in the hook, so run `analyze` and the test oracle yourself.

### General & Budget Commands
```powershell
# Verify codebase complexity ceilings:
lesscode-core budgets --config native/complexity-budgets.json --root native/src --out target/budgets.json

# Compare baseline and candidate trees under a frozen manifest:
lesscode-core compare --frozen frozen.json --baseline <base> --candidate <cand> --evidence <evidence> --out target/comparison-out.json
```

---

## 4. Standardized Developer Dogfooding & Branch Review Workflow

To prevent workspace corruption and guarantee reproducible refactorings, deconvolution follows a clean-clone, branch-based workflow:

```
┌─────────────────────────────────┐
│ Clean Working Clone (External)  │
│ `git clone . ../specimen-copy`  │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│ Deconvolution Execution         │
│ 1. `deconvolute analyze`        │
│ 2. Extract Core / Shell         │
│ 3. `deconvolute run` (review)   │
│ 4. `deconvolute density` (Pass) │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│ Candidate Refactoring Branch    │
│ `git checkout -b refactor/...`  │
│ `git push origin refactor/...`  │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│ Major Refactoring Review        │
│ - Full CI / nextest run         │
│ - Surviving mutants reviewed    │
│ - Mechanical density audit      │
└─────────────────────────────────┘
```

### 1. Work on a Clean Isolated Clone
Never perform destructive mutations or experimental deconvolution in the active host repository:
```powershell
git clone . ..\lesscode-dogfood-copy
cd ..\lesscode-dogfood-copy
```

### 2. Execute Deconvolution & Verification
1. Run `analyze` on the targeted module to verify write sites and AST gaps.
2. Refactor into pure core and imperative shell.
3. Run `deconvolute run` and review every mutant the test suite does not kill.
4. Run `density` to ensure structural simplification exceeds the mechanical thresholds.
5. Run the full specimen test suite using an isolated target directory:
   ```powershell
   cargo nextest run --target-dir target/isolated-specimen
   ```

### 3. Push Candidate Refactoring Branch
Commit the verified deconvolution and push to a dedicated candidate branch:
```powershell
git checkout -b refactor/deconvolute-<component>
git add .
git commit -m "refactor(deconvolute): isolate pure computational core in <component>"
# When installed, the intelligent pre-push hook gates the push automatically:
git push origin refactor/deconvolute-<component>
```

### 4. Treat as a Major Refactoring
Candidate deconvolution branches must pass full team / CI review:
- Full test suite passes across all platforms.
- The `deconvolute run` receipt is not void, and every surviving mutant has a recorded reason (missing test added, or the change makes no difference).
- Density receipt proves non-marginal complexity reduction.
- All complexity budgets ($\le 400$ physical lines, $\le 300$ syntax units, $\le 80$ lines/fn) are met.

---

## 5. Governance & Constraints

- **Never invent fake equivalence:** passing existing tests does not establish equivalence without adverse controls and bounded domain coverage.
- **Preserve public interfaces:** public caller contracts, exceptions, and observable behavior must remain intact.
- **Maintain complexity ceilings:** new adapters and commands must conform to the $\le 400$ physical lines, $\le 300$ syntax units, $\le 80$ lines per function complexity budget.
- **Fail-safe path validation:** tools must validate paths safely without rejecting legitimate sibling specimen directories.
