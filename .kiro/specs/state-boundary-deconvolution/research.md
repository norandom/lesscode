# Research & Discovery Log: State Boundary Deconvolution

## Summary

This research log captures the architectural analysis, technology decisions, and integration patterns for `state-boundary-deconvolution`.
Empirical trials in E3 ([R25](../../docs/research/2026-11-04/02-e3-evidence-regeneration.md)) and E4 ([R26](../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md)) demonstrated that while call-edge and function-count deconvolution effectively collapses scattered functions (−24.6% functions, −29.2% edges, 51.9% pure core), it fails to achieve whole-responsibility line compression (−6.0% LLOC; the "Density Trap"). Furthermore, trace execution oracles proved blind to exception-side state mutations (`AuditedAbsence`), partial CLI output buffers, and error-evaluation ordering (E4 F1–F7).

This specification introduces software verification techniques to soundly verify state boundaries and pure-core logic without relying solely on sparse happy-path trace replay.

---

## Research Topics

### 1. State Frame Conditions & Failure-Side Post-Conditions
- **Finding:** In E3, an extracted pure-core candidate dropped the mutation of an internal `AuditedAbsence` state object during serialization failure. The trace oracle missed this because test cases only evaluated return values on successful runs.
- **Approach:** Introduce explicit state frame condition contracts. A frame condition specifies:
  1. `allowed_writes`: set of mutable paths (globals, object attributes, database handles, buffer sinks).
  2. `success_invariants`: post-condition assertions evaluated when `result.is_ok()`.
  3. `failure_invariants`: post-condition assertions evaluated when `result.is_err()`.
- Any mutation to a location outside `allowed_writes` is flagged as an unauthorized side effect. Any omitted post-condition mutation on failure is flagged as a broken failure invariant.

### 2. Formal Bounded Equivalence for Extracted Pure Cores
- **Finding:** In E4, 27 of 52 functions (51.9%) were partitioned into a side-effect-free functional core. Testing them with recorded trace sequences only exercised a fraction of reachable input space.
- **Approach:** Pure functions (deterministic mappings with 0 write sites and 0 I/O) can be verified with bounded mathematical rigor:
  - Lightweight property-based testing and bounded symbolic exploration (using Hypothesis / bounded Z3 SMT encodings for arithmetic/relational decisions).
  - Evaluates equivalence across boundary conditions, negative integers, null/empty values, and malformed inputs.
  - Automatically emits minimal counterexamples when candidate output diverges from original output.

### 3. Adverse State Mutation Sensitivity (The Verifier Challenge)
- **Finding:** R39 demonstrated that passing suites can conceal broken controller logic if tests do not deliberately challenge adverse conditions.
- **Approach:** Introduce planted state mutations as an acceptance gate:
  - *Exception Mutation:* Comment out or short-circuit exception-side state mutations.
  - *Buffer Mutation:* Prevent partial stdout/stderr buffer flushing before an abort.
  - *Order Mutation:* Swap error evaluation rungs.
  - A candidate's test battery must reject 100% of non-equivalent planted mutants before acceptance.

### 4. Density Trap Defense & Whole-Responsibility Accounting
- **Finding:** In E4, functions dropped by ~25% and edges dropped by ~29%, but canonical lines dropped by only 6% because code density increased.
- **Approach:** Decouple structural de-fragmentation from whole-responsibility compression:
  - Candidate reporting must explicitly distinguish `structurally_consolidated` from `compressed`.
  - Compression requires satisfying a non-zero AST statement and canonical line reduction threshold across the complete containing scope (including helpers, tables, and wrappers).

---

## Technology & Boundary Decisions

1. **Lightweight Native Core vs. Test-Harness Separation:**
   - In accordance with the R38 7,004 canonical line ceiling, heavy verification runners and solver integrations must NOT be embedded into the production Rust apparatus (`lesscode-core`).
   - The native library exposes lean verification hooks (`FrameCondition`, `StateVerdict`, `StreamAssertion`), while property generators, SMT bindings, and mutation test drivers run in the test/research harness.
2. **Integration with `refinement-evidence-integrity`:**
   - Leverages the repaired controller's causal feedback and obligation tracking. State frame violations and pure-core divergences become active obligations that cannot be marked discharged upon discovery.
