# Task group 13 handover: state boundary deconvolution implementation and prototype feasibility gate

Written 2026-11-08. Group scope: execute the complete Kiro implementation and qualification workflow for
`.kiro/specs/state-boundary-deconvolution`, covering tasks 1.1 through 4.2.
All specification, implementation, and verification ran in the foreground. No
subagents were dispatched, per owner rule.

## What was done

1. **Task 1: Foundation — state frame conditions and stream buffer monitoring:**
   - **Task 1.1 (commit `62ff486`):** Implemented `StateFrame`, `StateViolation`, `StaticWriteSiteVisitor`, and `check_static_frame` in `src/lesscode_baseline/state_boundary/frame.py` to identify permitted global, instance, and environment write locations, detecting unauthorized mutations outside declared shell functions.
   - **Task 1.2 (commit `35cd16e`):** Implemented dynamic state snapshotting and `verify_dynamic_state` in `frame.py` to evaluate post-condition state assertions on unhandled exceptions and controlled aborts, catching omitted failure-side state transitions (such as E3 `AuditedAbsence`).
   - **Task 1.3 (commit `dff8ca1`):** Implemented `StreamEvent`, `StreamContract`, `StreamDivergence`, `StreamCapture`, and `verify_stream_preservation` in `src/lesscode_baseline/state_boundary/stream_monitor.py` to intercept stdout, stderr, and logging sinks, catching truncated or dropped pre-abort buffers (such as E4 F3) while honoring commutative vs non-commutative sinks.

2. **Task 2: Core — pure core formal equivalence and adverse state mutation generator:**
   - **Task 2.1 (commit `db6761f`):** Implemented `StaticPurityVisitor` and `check_static_purity` in `src/lesscode_baseline/state_boundary/pure_verifier.py` to statically verify that declared pure core functions have zero state mutations, zero global lookups, and zero external I/O invocations.
   - **Task 2.2 (commit `10affea`):** Implemented bounded semantic equivalence checking and automatic counterexample minimization (`verify_bounded_equivalence`, `InputDomainBounds`, `PureDivergenceWitness`) in `pure_verifier.py`, emitting minimal counterexamples upon output divergence.
   - **Task 2.3 (commit `0e52c22`):** Implemented targeted adverse state mutation generator (`generate_state_mutants`, `evaluate_mutant`, `StateMutant`, `StateMutationKind`) in `src/lesscode_baseline/state_boundary/mutant_generator.py` targeting exception handlers (`DROP_FAILURE_STATE`), output buffer flushes (`BUFFER_TRUNCATION`), and error check ordering (`ERROR_ORDER_SWAP`).

3. **Task 3: Integration — mutation sensitivity gates, causal refinement bridge, and density defense:**
   - **Task 3.1 (commit `20ea970`):** Implemented `run_mutation_sensitivity_gate` and `MutationGateReport` in `mutant_generator.py`, requiring candidate test batteries to achieve a 100% kill rate on non-equivalent planted state mutants with mandatory kills on exception-side mutations and buffer flushes.
   - **Task 3.2 (commit `9bf5a3b`):** Extended refinement controllers in Python (`src/lesscode_baseline/regenerator.py`) and Rust (`native/src/regenerator.rs`) to capture `state_violations`, `pure_divergences`, `stream_divergences`, and `surviving_mutants` into turn feedback and retain them in `unresolved_obligations` across turns until verified.
   - **Task 3.3 (commit `c78b9df`):** Implemented `ScopeMetrics`, `DeconvolutionOutcome`, `measure_scope_source`, and `evaluate_deconvolution_outcome` in Python (`density.py`) and lean native structures in Rust (`native/src/state_contract.rs`), explicitly decoupling `structurally_consolidated` from `compression_met` to defend against the E4 density trap.

4. **Task 4: Validation — adversarial qualification suite and prototype feasibility gate:**
   - **Task 4.1 (commit `ca0fe44`):** Implemented offline adversarial qualification suite in `tests/test_state_boundary.py` (32 unit and integration tests) verifying 100% detection of frame condition leaks, failure-side state omissions, truncated streams, and surviving mutants.
   - **Task 4.2 (commit `00905f9`):** Executed the end-to-end prototype feasibility gate runner (`.kiro/specs/state-boundary-deconvolution/run_gate.py`) against Facdrone adverse controls (E3 `AuditedAbsence`, E4 F1/F3/F7) and performed exact production Rust apparatus canonical line accounting against the R38 7,004 ceiling. Recorded evidence in `results/prototype-gate-report.json`.

## Fresh evidence

- **Adverse Controls and Verification Capabilities (100% PASS):**
  - **E3 `AuditedAbsence`:** Dynamic frame evaluator detected missing `AuditedAbsence` post-condition on abort (`StateViolationType.MISSING_FAILURE_INVARIANT`).
  - **E4 F1 Silent `None`:** Bounded pure core verifier emitted minimal reproducible counterexample `{"x": None}`.
  - **E4 F3 Stream Preservation:** Monitor detected 31 dropped bytes on `stdout` prior to abort point (`StreamDivergenceType.TRUNCATED`).
  - **E4 F7 Error Ordering:** Adverse mutation sensitivity gate killed 100% of planted error-order-swap mutants (0 survived).
  - **Density Trap Defense:** Whole-scope measurement on Facdrone `lifecycle` monitor scope (69 functions, 89 edges, 1,495 lines) confirmed structural consolidation (-24.64% functions, -29.21% edges, 51.92% pure core) while correctly failing `compression_met` (-6.02% LLOC vs 10% target) and setting `density_trap_caught: True`.

- **Test Suite Results:**
  - Python adversarial state boundary suite (`tests/test_state_boundary.py`): **32 passed** in 0.12s.
  - Rust regenerator suite (`native/tests/regenerator.rs`): **50 passed, 0 skipped**.
  - Rust recording suite (`native/tests/recording.rs`): **14 passed (3 slow), 0 skipped** in 247.6s.
  - Rust CLI & budget enforcement suite (`native/tests/cli.rs`): **7 passed, 0 skipped** in 0.89s (all complexity budgets exit 0).

- **Apparatus Canonical Line Accounting & Feasibility Gate Disposition:**
  - Measured under unchanged R37 canonical method (`rustfmt 1.98.1` + `ruff==0.17.3` + net git-diff hooks vs `f2fe982`):
    - `native/src/regenerator.rs`: 2,790 lines (+74 lines vs R37: +38 lines from R39 correctness fixes, +36 lines from state obligation bridge)
    - `native/src/measures.rs`: 1,422 lines
    - `native/src/surface.rs`: 1,151 lines
    - `native/src/recording.rs`: 971 lines
    - `native/src/jsonval.rs`: 130 lines
    - `native/src/admission.rs`: 274 lines
    - `native/src/receipt.rs`: 124 lines
    - `native/shim/recording_shim.py`: 178 lines
    - Registration hooks (`lib.rs`, `main.rs`, `Cargo.toml`): 39 lines (16 frozen + 23 net added)
    - **Total R37 Counted Set:** **7,079 canonical lines** (Ceiling: 7,004; Overage: +75 lines, +1.07%)
    - With `native/src/state_contract.rs` (94 canonical lines): **7,173 canonical lines** (Overage: +169 lines, +2.41%)
  - **Prototype Gate Disposition:** **`PASS / PROCEED`** under the R41 amended ceiling. The initial run evaluated to `REVISE` (+169 lines over 7,004); the owner exercised Option (a) on 2026-11-08 ([R41](../docs/research/2026-11-08/08-state-boundary-ceiling-amendment.md)), amending the production Rust ceiling to 7,173 canonical lines. The gate report now passes with `disposition: PROCEED`.

## Untouched owner WIP

All 4 preserved owner WIP files remain unstaged and untouched:
- `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
- `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
- `src/lesscode_baseline/rules/evaluate.py`
- `tests/test_rule_evaluation.py`

## Current state and next steps

The specification `state-boundary-deconvolution` is marked completed in `spec.json`.
The prototype has established:
1. Formal state boundary mechanisms eliminate the oracle blindness problems exposed by E3 and E4.
2. The whole-responsibility density scorer reliably prevents the density trap by distinguishing function consolidation from net statement and line reduction.
3. The apparatus line volume has expanded to 7,173 canonical lines and is officially ratified under the R41 no-growth ceiling.
4. Prototype gate has officially PASSED (`PROCEED`).

The state boundary deconvolution capabilities are now available to govern subsequent deconvolution trials without oracle blindness on scattered state.

Per owner rule, please clear the conversation context before starting the next task group.
