# Prototype Feasibility Gate: State Boundary Deconvolution

Status: **Provisional proposal and research feasibility gate**, 2026-11-08. Governed by [`.kiro/steering/prototype-first.md`](../../steering/prototype-first.md). Real-code behavior and adverse mutation evidence are required before production task approval.

---

## 1. Context & Hypothesis Under Test

* **Predecessors:** E3 ([R25](../../docs/research/2026-11-04/02-e3-evidence-regeneration.md)), E4 ([R26](../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md)), R38 ([05-u5-ceiling-amendment.md](../../docs/research/2026-11-08/05-u5-ceiling-amendment.md)), R39 ([06-refinement-correctness.md](../../docs/research/2026-11-08/06-refinement-correctness.md)), and R40 ([07-structural-deconvolution-and-state-boundaries.md](../../docs/research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md)).
* **Core Hypothesis:** Explicit state frame conditions, bounded pure-core equivalence, and adverse state mutation gates can eliminate oracle blindness on scattered state without increasing apparatus line volume beyond the R38 7,004 canonical line ceiling.
* **Concrete Removal Hypothesis:** In a representative scattered responsibility (e.g., Facdrone `lifecycle` or `evidence_publication`), scattered state mutation boilerplate and redundant intermediate wrapper functions (24% of functions, 29% of call edges) can be replaced by a partitioned functional core and an explicit imperative shell, while preserving 100% of exception-side state transitions, partial stream buffers, and error-ordering invariants.

---

## 2. Frozen Specimen Inputs & Scope

* **Primary Specimen Scope:** Facdrone `lifecycle` monitor (7 files, 1,495 lines, 69 functions, 89 intra-scope call edges).
* **Baseline Known Defect Targets (Adverse Controls):**
  1. *E3 Serialization Exception:* `AuditedAbsence` failure-side state recording.
  2. *E4 F1:* Silent `None` evidence acceptance.
  3. *E4 F3:* Partial CLI stdout output emitted before an abort.
  4. *E4 F7:* Error-ordering sequence divergence.
* **Apparatus Scope & Accounting:**
  - Production substrate: `lesscode-core` (Rust), capped at 7,004 canonical lines (R38 no-growth cap).
  - Test/verification harness: Separately counted in research/test support.

---

## 3. Preregistered Feasibility Targets & Stop Conditions

| Metric | Target Bar | Falsifier / Stop Condition |
|---|---|---|
| **Exception-Side State Post-Conditions** | 100% of failure-side invariants verified (e.g. `AuditedAbsence` present after abort) | Any dropped failure-side state mutation passes the gate |
| **Output Stream & Buffer Preservation** | 100% byte-exact partial stdout/stderr stream preserved before abort points | Any truncated, buffered, or dropped output stream passes the gate |
| **Pure Core Equivalence** | 100% bounded equivalence over pure functions across both valid and invalid input domains | Divergence on any bounded input combination passes the gate |
| **Adverse Mutation Sensitivity** | 100% of non-equivalent planted state mutants killed by test battery | Any surviving adverse state mutant |
| **Apparatus Line Ceiling** | Counted production Rust substrate $\le 7,004$ canonical lines | Substrate canonical lines exceed 7,004 without dated owner decision |

---

## 4. Required Evidence for "Proceed"

1. **Adverse Mutation Kill Proof:** Explicit execution logs showing the harness detecting and killing planted mutants in exception handlers, partial output buffers, and state write sites.
2. **Causal Recovery Log:** Evidence that when a state frame violation is discovered, the controller generates a structured counterexample, routes it into the subsequent prompt, and preserves the unresolved obligation across turns.
3. **Whole-Responsibility Measurement:** Clear accounting of all added shell helpers and tables, demonstrating whether statement volume actually decreased or merely compacted.

If the prototype cannot kill the adverse controls or breaches the 7,004 line apparatus ceiling, the gate outcome is **`revise`** or **`stop`**. No production implementation tasks may be generated without passing this gate.
