# Specification Brief: State Boundary Deconvolution

## Problem Statement

Experimental deconvolution trials (E3, E4) demonstrated that an LLM guided by structural measures can successfully de-fragment scattered code—reducing function count by 24.6% and call edges by 29.2%, while partitioning logic into an imperative shell and a verified pure core (>50% pure functions). 

However, two major barriers prevent this from achieving safe, verifiable code compression:
1. **The Density Trap:** Merging functions reduces caller-to-callee hops, but does not eliminate underlying statement volume (in E4, canonical lines fell by only 6.0%, failing the predicted 10–20% target). Consolidating functions without removing redundant responsibilities simply yields denser functions.
2. **Scattered State & Oracle Blindness:** Trace execution oracles only observe paths exercised by seed test corpora. When scattered mutable state is consolidated, unmonitored edge paths lose critical side effects—such as exception-side state mutations (`AuditedAbsence` in E3), partial CLI output buffers before aborts (E4 F3), and error-handling evaluation order (E4 F7).

## Proposed Approach: Software Verification of State Boundaries

To resolve these issues, we introduce formal software verification techniques into the deconvolution and refinement pipeline:
1. **Explicit State Frame Conditions & Post-Condition Contracts:**
   - Formalize the mutable state boundary: specify the exact set of state locations (globals, attributes, storage, buffers) that each function is permitted to read or mutate.
   - Enforce exception-side state invariants: verify that aborts and exceptions leave state in the exact contractual post-state expected by the system.
2. **Bounded Formal Equivalence for the Pure Core:**
   - For all logic extracted into the functional core, execute bounded SMT equivalence checks or exhaustive property-based verification (Hypothesis / symbolic execution) to mathematically verify equivalence across both valid and invalid input spaces.
3. **Adverse State Mutation Verification (Sensitivity Gates):**
   - Automatically inject planted mutants into state mutation sites, exception handlers, and output buffers.
   - A candidate cannot be accepted based on green test passes alone; it must actively kill all planted state-boundary mutants.
4. **Causal Multi-Turn State Recovery:**
   - Leverage the repaired `refinement-evidence-integrity` controller to route state-boundary counterexamples causally back into subsequent refinement turns, preserving active obligations until explicitly re-verified.

## Scope & Boundaries

- **In Scope:**
  - Automated extraction of state frame conditions and mutable location inventories.
  - Exception-side post-condition assertions and buffer-flush verification.
  - Bounded property/SMT equivalence verification harness for the pure core.
  - Targeted state mutation generator and verification gate.
  - Integration into the native refinement loop (`regenerator.rs` / `regenerator.py`).
- **Out of Scope:**
  - Full whole-program universal equivalence proofs for arbitrary unbounded loops.
  - Heavy external solver infrastructure that would violate the R38 7,004 canonical line ceiling.
  - Automatic rewrite adoption without maintainer review.

## Constraints

- Rust substrate apparatus must remain within the R38 7,004 canonical line ceiling.
- Strict causal feedback and candidate delivery rules enforced by `refinement-evidence-integrity`.
- Native Windows execution using clean PATH protocol; cargo-nextest and pytest-xdist runners.
