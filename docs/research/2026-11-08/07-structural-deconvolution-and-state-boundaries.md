# R40: structural consolidation without line reduction — the scattered state and density boundary

- Report date/time and timezone: 2026-11-08, UTC
- Execution interval: retrospective analysis across E1–E4, R11, R25, R26, R34–R39, and the refinement repair
- Status: completed retrospective synthesis
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), [deconvolution-loop](../../../.kiro/specs/deconvolution-loop/requirements.md); informs follow-up spec `state-boundary-deconvolution`
- Hypothesis ID and predecessors: H8(c) ([R19](../2026-11-03/01-dynamic-deconvolution-direction.md)), E3 ([R25](../2026-11-04/02-e3-evidence-regeneration.md)), E4 ([R26](../2026-11-06/01-e4-lifecycle-regeneration.md)), R38 ([05-u5-ceiling-amendment.md](05-u5-ceiling-amendment.md)), R39 ([06-refinement-correctness.md](06-refinement-correctness.md))
- Type: retrospective synthesis & COBESY evidence audit

## Decision

The empirical evidence establishes that an LLM guided by structural measures can successfully de-fragment scattered code, reducing function counts by ~25% and intra-scope call edges by ~29%, while partitioning stateful logic into a verified pure core (>50% pure functions). 

However, this structural consolidation did **not** produce the expected whole-replacement compression:
1. **The Density Trap:** Merging functions reduced caller-to-callee hops, but canonical lines only dropped by 6.0% (missing the 10–20% target; T2 failed). Code statements were compacted into fewer, denser definitions rather than eliminated.
2. **Scattered State and Oracle Blindness:** Trace-based execution oracles verify standard happy paths, but miss subtle state transitions on edge paths. Consolidated candidates routinely dropped unmonitored exception-side state (E3 `AuditedAbsence`), partial CLI outputs (E4 F3), and error-ordering invariants (E4 F7).
3. **Apparatus Expansion:** Across all substrates, the refinement apparatus itself expanded to 7,004 canonical lines (R38 no-growth cap), failing the 20% reduction target (2,270 line cap).

What this permits: we must stop treating trace re欧play and line-count metrics as sufficient proxies for semantic equivalence and compression. A new specification utilizing formal software verification techniques—explicit state frame conditions, exception-side invariants, bounded SMT equivalence on the pure core, and adverse mutant suites—is required to soundly govern state-boundary deconvolution.

## Hypothesis and falsification

- **Structural Hypothesis Under Audit:** Reducing call edges and consolidating scattered functions yields equal observable behavior with at least 10–20% fewer lines.
- **Falsification Observed:** In E4, function count fell by 24.6% and call edges fell by 29.2%, but lines fell by only 6.0% (T2 failed). Simultaneously, independent adversarial probes exposed 7 behavioral differences (F1–F7) that the trace oracle could not observe.
- **New Falsifiable Proposal:** Software verification techniques can establish sound state boundaries by:
  1. Enforcing explicit frame conditions and post-conditions on all state locations across both success and exception paths.
  2. Bounding symbolic/SMT equivalence on the extracted pure core.
  3. Rejecting any candidate that fails to kill adverse mutation controls planted on exception and state-mutation sites.

## Frozen inputs and method

- **Specimen Evidence:**
  - Facdrone `lifecycle` monitor (7 files, 1,495 lines, 69 functions, 89 intra-scope call edges).
  - Facdrone `evidence_publication` (E3).
  - G1 admission screen across 15 files in 4 repositories (Lesscode, NPU RAG, Facdrone, ATS Agent).
- **Tooling & Provenance:**
  - Python baseline (`regenerator.py`) and Native Rust substrate (`lesscode-core`, 2,752 physical lines in `regenerator.rs`).
  - Trace oracle recorder, AST call-graph analyzer, `ruff==0.17.3`, `rustfmt 1.98.1`, `cargo-nextest 0.9.150`, `pytest-xdist 3.8.0`.
  - Adversarial probe batteries from R39 and E4 independent review (128 probes).

## Evidence and results

### 1. Structural Measures vs. Line Reductions

| Metric | Original Scope | E4 Consolidated Candidate | Measured Change | Target / Status |
|---|---|---|---|---|
| **Function Count** | 69 functions | 52 functions | **−24.6%** | Held (predicted $\le 55$) |
| **Intra-Scope Call Edges** | 89 edges | 63 edges | **−29.2%** | Held (predicted $\le 66$) |
| **Write Sites per State Location** | Scattered across files | 11 write sites in 25 shell functions | **Constrained** | Held (predicted $\le 12$) |
| **Pure Function Share** | 0% formally verified | 27 of 52 functions (51.9%) | **>50% Pure Core** | Held (predicted $\ge 0.50$) |
| **Canonical Lines (LLOC)** | 1,495 lines | 1,405 lines | **−6.0%** | **FAILED** (predicted 10–20%, $\le 1,345$) |

### 2. Scattered State & Unobserved Divergences (Oracle Blindness)

Trace oracles comparing normal execution outputs failed to catch critical stateful discrepancies:
* **E3 Exception-Side State:** Candidate dropped `AuditedAbsence` state recording during failed serialization.
* **E4 Hidden Divergences (F1–F7):**
  * *F1:* Silent acceptance of `None` evidence on typed inputs.
  * *F3:* Lost partial CLI stdout output when processing crashed midway.
  * *F7:* Inverted error evaluation order between validation stages.
  * *Call-time Binding:* Changed global lookup times during dynamic patching.

## Limits and interpretation

1. **De-fragmentation is Not Deletion:** Consolidating micro-functions improves locality and reduces hops, but unless underlying responsibility requirements or duplicate logic trees are deleted, AST statement volume remains invariant.
2. **Traces are Incomplete Specifications:** Replaying recorded traces verifies observed interactions, but leaves exception recovery, partial output buffers, and latent state invariants unchecked.
3. **Apparatus Line Ceiling:** The apparatus ceiling of 7,004 lines cannot absorb large external verification frameworks without violating the R38 no-growth cap. Verification techniques must be bounded, modular, and lightweight.

## Tool/spec consequence and next gate

The findings motivate a dedicated specification: **`state-boundary-deconvolution`**.
This spec moves beyond purely empirical trace matching by introducing:
1. **Formal State Invariants & Frame Conditions:** Declaring exact mutable sets and post-conditions on exception exits.
2. **Pure Core Bounded SMT Equivalence:** Verifying mathematical equivalence of extracted pure functions.
3. **Targeted Adverse Mutation Killing:** Forcing candidates to kill deliberate state-mutation faults before acceptance.

## COBESY check

- **Outcome Visible First:** Negative results on LLOC and oracle blindness stated explicitly; structural gains acknowledged without inflation.
- **Replayability:** Every metric mapped to recorded E3/E4 artifacts, receipts, and line numbers.
- **Disproof Explicit:** Disproved the assumption that reducing call edges and functions automatically produces proportional line compression.
- **Negative Results Safe:** Falsifications (T2 line failure, R38 apparatus cap, F1–F7 oracle blindness) documented as essential scientific boundaries.
