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
- Python: flag `eval`, `exec`, `globals()`, unmonitored threading.
- TypeScript: flag `eval`, `Function()`, prototype mutations, dynamic `require`.
- Java: flag `Class.forName`, custom `ClassLoader`, `native`, unmonitored threads.
- Rust: flag `unsafe`, `dyn`, raw pointer dereference, unmonitored threads, hidden `Cell`/`Mutex`.

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

## 3. Native CLI Commands

Execute verification directly using the compiled native binary:

```powershell
# Verify complexity budgets:
lesscode-core budgets --config native/complexity-budgets.json --root native/src --out target/budgets.json

# Compare baseline and candidate trees under a frozen manifest:
lesscode-core compare --frozen frozen.json --baseline path/to/base --candidate path/to/cand --evidence path/to/evidence --out target/comparison-out.json

# Run Rust adapter test suite:
cargo nextest run --manifest-path native/Cargo.toml
```

---

## 4. Governance & Constraints

- Never invent fake equivalence: passing tests do not establish equivalence without adverse controls and bounded domain coverage.
- Preserve public interfaces: public caller contracts, exceptions, and observable behavior must remain intact.
- Keep the apparatus lean: new adapters must conform to the $\le 400$ line, $\le 300$ unit, $\le 80$ lines/fn complexity budget.
