# Prototype Feasibility Gate: Native Deconvolution Closure

Status: **Provisional proposal and research feasibility gate**, 2026-11-09. Governed by [`.kiro/steering/prototype-first.md`](../../steering/prototype-first.md). Real-code behavior, LF-body digest parity, and adverse mutation evidence are required before production task generation or execution.

---

## 1. Context & Hypothesis Under Test

* **Predecessors:** E3–E6 ([R25](../../docs/research/2026-11-04/02-e3-evidence-regeneration.md), [R26](../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md), [R40](../../docs/research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md), [R41](../../docs/research/2026-11-08/08-state-boundary-ceiling-amendment.md), [R43](../../docs/research/2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md), [R46](../../docs/research/2026-11-09/04-e6-report-envelope-adoption-maturity-gate.md)).
* **Core Hypothesis:** Transferring the remaining Python apparatus (`state_boundary/`, `recording_shim.py`, and regenerator streaming/token guards) into compiled Rust in `native/src/` produces a self-contained native deconvolution core that eliminates external runtime shim dependencies while strictly maintaining 100% adverse mutant sensitivity, 0 replay divergence, and byte-exact golden digest parity.
* **Concrete Removal & Transfer Plan:**
  1. Retire `native/shim/recording_shim.py` (178 lines freed from counted apparatus).
  2. Implement native state-boundary analysis, bounded pure verification, stream monitoring, and adverse mutant injection in modular Rust files under `native/src/`.
  3. Enforce that every new Rust file complies with the 400-line physical / 300-unit logical complexity ceiling.

---

## 2. Frozen Specimen Inputs & Scope

* **Primary Specimen Scope:** Facdrone `lifecycle` (E5) and `report-envelope` (E6) adopted baselines and frozen golden sequences.
* **Frozen Verification Targets:**
  1. *Frozen Double-Record Digests:* `FROZEN_E3_LF_BODY` (`92a48f77...`) and `FROZEN_E4_LF_BODY` (`0b2e383d...`).
  2. *Adverse State Mutants:* Exception-side mutants, buffer flush mutants, and state write-site mutants (must achieve 100% kill rate).
  3. *Pure Core Equivalence:* 0 counterexamples over bounded domains ($[-10, 10]$, floats, empty collections) across all extracted pure functions.
  4. *AST Statement Density:* Enforce $\ge 10\%$ AST statement reduction and $\ge 20\%$ line reduction on candidate scopes.

---

## 3. Preregistered Feasibility Targets & Stop Conditions

| Metric | Target Bar | Falsifier / Stop Condition |
|---|---|---|
| **Golden Digest Parity** | 100% byte-exact match on `FROZEN_E3_LF_BODY` and `FROZEN_E4_LF_BODY` | Any divergence in golden body bytes or LF-body SHA-256 |
| **Pure Core Equivalence** | 0 divergences on bounded integer, float, and edge schema inputs | Discrepancy on any bounded domain input without minimal counterexample |
| **Adverse Mutant Kill Rate** | 100% (all planted adverse mutants killed) | Any planted adverse state mutant survives |
| **Stream Buffer Preservation** | Byte-exact partial output captured prior to aborts | Dropped, truncated, or reordered stream bytes |
| **Per-File Complexity Ceilings** | $\le 400$ physical lines, $\le 300$ logical units, $\le 80$ lines/fn | Any new production file breaches complexity caps |

---

## 4. Required Evidence for "Proceed"

1. **Native Replay Parity:** Proof that the native specimen execution engine replays frozen sequences and matches golden digests without `recording_shim.py`.
2. **Native State Verification Proof:** Verification logs showing native `state_frame`, `pure_verifier`, `stream_monitor`, and `mutant_generator` executing against E5/E6 candidates with 100% mutant kill rate and 0 divergences.
3. **Complexity Budget Compliance:** Clean execution of `cargo test --test budgets` verifying all new modules against `complexity-budgets.json`.

If the native apparatus fails digest parity, leaks state, or breaches complexity caps, the gate verdict is **`revise`** or **`stop`**. No production implementation tasks may be approved or executed without passing this gate. Provisional, unapproved task drafts (`tasks.approved = false`, `ready_for_implementation = false`) may be generated for planning purposes only (finding F-2, 2026-11-09 fusion review).
