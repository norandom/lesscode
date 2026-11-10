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

### Phase 5: Adverse Mutation Sensitivity
1. Apply 4 mandatory adverse controls to the candidate:
   - `MissingFailureState`: omit error classification or timeout kill.
   - `InvertedCondition`: flip branch conditions (`!= 0` to `== 0`).
   - `InvertedErrorOrder`: bypass divergence checks or reorder error precedence.
   - `SilentNone`: drop environment variables, stream flush, or required headers.
2. Run the test oracle: require **100% kill rate** ($K / N = 1.0$). If any mutant survives, the test battery is insufficient and the candidate must not be admitted.

### Phase 6: Mechanical Density Gate
Evaluate structural reduction using `lesscode-core`:
- $\text{LineReduction} \ge 20\%$ and $\Delta \text{Lines} \ge 50$ canonical lines.
- $\text{StmtReduction} \ge 10\%$ and $\Delta \text{Stmts} \ge 10$ AST statements.
- Any change below these absolute or relative thresholds is rejected as marginal.

---

## 3. Native CLI Porcelain Commands

The native `lesscode-core` binary exposes first-class deconvolution commands across all supported runtimes:

### Known limits of the porcelain (R55, 2026-11-10)
Measured with the 1.0.1 binary; see `docs/research/2026-11-10/01-released-binary-adopter-dogfood.md`. Where release 1.0.2 behaves differently, the item says so.
- `analyze --lang python` performs no analysis. From release 1.0.2 it writes no receipt, prints a note to stderr and exits 2. Version 1.0.1 still reports `"passed": true`. Do not use either as evidence.
- `mutate` uses operators bound to the experiment specimens. On other code it usually writes no mutants. From release 1.0.2 it then exits 1 (the receipt shows `"mutants_generated": 0`); version 1.0.1 exits 0, so read `mutants_generated`. There is no Python generator: from release 1.0.2 `mutate --lang python` writes nothing and exits 2.
- `density --lang python` reports the line count in both the line and statement fields.
- The pre-push hook enforces complexity budgets only. From release 1.0.2 it treats `refactor/*` and `deconvolute/*` branches like any other branch; version 1.0.1 announces them but checks nothing more.

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
- **Verification Rule:** Every generated mutant must be executed against the candidate test suite; candidate is accepted only if $K / N = 1.0$ (100% killed).

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
│ 3. `deconvolute mutate` (K/N=1) │
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
│ - Zero adverse surviving mutants│
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
3. Run `mutate` and verify that the test suite kills 100% of generated adverse mutants.
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
- Adverse mutation receipts show $K/N = 1.0$ kill rate.
- Density receipt proves non-marginal complexity reduction.
- All complexity budgets ($\le 400$ physical lines, $\le 300$ syntax units, $\le 80$ lines/fn) are met.

---

## 5. Governance & Constraints

- **Never invent fake equivalence:** passing existing tests does not establish equivalence without adverse controls and bounded domain coverage.
- **Preserve public interfaces:** public caller contracts, exceptions, and observable behavior must remain intact.
- **Maintain complexity ceilings:** new adapters and commands must conform to the $\le 400$ physical lines, $\le 300$ syntax units, $\le 80$ lines per function complexity budget.
- **Fail-safe path validation:** tools must validate paths safely without rejecting legitimate sibling specimen directories.
