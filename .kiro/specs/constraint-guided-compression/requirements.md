# Requirements Document

## Correctness finding, 2026-11-08 (R39)

[R39](../../../docs/research/2026-11-08/06-refinement-correctness.md) reproduces
violations of the intended candidate/evidence, feedback, obligation and outcome
contracts despite passing existing regenerator suites. The
[corrective spec](../refinement-evidence-integrity/requirements.md) and
[adjusted concept](../refinement-evidence-integrity/concept.md) are pending review.
Requirements 4.2-4.6, 5.5-5.8, 6.3-6.7 and 7.11 govern the correction. This notice
does not approve new phases, alter R38's cap or schedule Slice B adoption.

## Introduction

**Amendment, 2026-11-07 ([R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md)):**
Following the empirical falsification of static CodeQL state models and LLM-authored SMT equivalence models in `R08`–`R18` ([R19](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md)), the completion of [`capability-compression-core`](../capability-compression-core/spec.json) and [`deconvolution-native-core`](../deconvolution-native-core/spec.json) (`native/src/{scope,metrics,compare,model,admission,receipt,budgets}.rs` and [`src/lesscode_baseline/recording.py`](../../../src/lesscode_baseline/recording.py)), and the owner's 2026-11-07 direction, Requirements 1–8 below (all 56 approved numeric criteria `1.1`–`8.6`) are realized through the **H8/H9 dynamic deconvolution substrate** plus **H10 DSPy-style probabilistic multi-turn refinement (`Refine` / `BestOfN`)**:
- **Queryable program evidence (Req 2)** is supplied by deterministic AST surface derivation (`surface.py` Contract B: tree + recorder imports per [R29](../../../docs/research/2026-11-07/03-s9-surface-contracts.md)), intra-scope call-edge, write-site and pure/shell structural checks (`measures.py`), and static test-infrastructure detection (`testinfra.py` per [R32](../../../docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md)); CodeQL remains an optional external locator, not a required runtime database or state model.
- **Behavioral modeling and challenge (Req 3, Req 5)** are grounded in recorded Level 1/Level 2 runtime trace oracles (`lesscode_baseline.recording`), mutation sensitivity scoring (`cosmic-ray` / `cargo-mutants`), and pure-core differential checks (`CrossHair` behind the 4 solver-encoding verifier gates in [`docs/symbolic-execution-tooling.md`](../../../docs/symbolic-execution-tooling.md) as a counterexample finder only, never as an unbounded equivalence proof).
- **Bounded multi-pass modeling and revision (Req 4, Req 7.3–7.5)** are realized by a **DSPy-style probabilistic turn controller (`Refine` / `BestOfN`)** with a two-tier programmatic reward function (`StaticAssertionGate` in $[0.0, 0.49]$ for fast syntax/editscope/surface/pure-shell assertions without DB spin-up; `RuntimeOracleGate` in $[0.50, 1.0]$ for trace replay and 3-stratum canonical measures), enabling weaker or non-deterministic models to recover across bounded turns while preserving every rollout hash and enforcing strict training vs. held-out scope separation (`7.5`) and the prototype-first gate (`7.9–7.11`).

**Amendment 2, 2026-11-07 (language-scope disposition after the independent fusion design review):**
The owner records a staged scope disposition for the bilingual language clauses (Boundary Context "Python and TypeScript source"; `Req 7.1` labeled bilingual cases): **this spec implements and evaluates Python only; TypeScript extraction evaluation is deferred to a named later stage**, and other target languages remain explicit TODO items. The G0-era TypeScript fixtures (oracle 12/13, prototype-gate.md) are preserved as the first evaluation material for that stage, subject to revalidation under the mechanism chosen then (they predate CodeQL's demotion to optional locator). `Req 7.1` is satisfied for Python and reads "partially satisfied / deferred (TypeScript)" until that stage. Extensibility requirement (owner, 2026-11-07): the design and its generated tasks must keep every language-specific analysis step behind a per-language extractor seam so that adding a target language (TypeScript first, then others) is an additive change, never a rewrite; recorded in design.md ("Deferred Target Languages") and binding on task generation.

**Amendment 3, 2026-11-08 (U5 disposition, [R35](../../../docs/research/2026-11-08/02-u5-disposition.md)):**
The owner records the U5 disposition for the whole-replacement apparatus
ceiling ([prototype-gate.md](prototype-gate.md) frozen checklist): **U5 is
accepted as failing on the Python stage-1 prototype substrate** (3,783 vs
2,270 canonical lines; ceiling unchanged, `Req 7.8` negative retained). Python
is the stage-1 prototype; Rust is the target substrate. Slice B production
adoption (`Req 7.10`) remains withheld until a Rust-port re-measurement of U5
under the same expanded-baseline accounting passes the frozen ceiling, or a
further dated owner decision records a different disposition.

**Amendment 4, 2026-11-08 (U5 ceiling amendment, [R38](../../../docs/research/2026-11-08/05-u5-ceiling-amendment.md)):**
The owner records the disposition on the R37 negative by exercising
preregistered option (a) of the R37 outcome rule: **the Rust-substrate U5
ceiling is amended from 2,270 to 7,004 canonical lines** — exactly the
measured R37 total on the R37 counted set, effective as a no-growth cap
(growth beyond it, or any baseline change, requires a further dated owner
decision). The 20% whole-replacement reduction claim is retired as
falsified; the Python 3,783 and Rust 7,049/7,004 negatives stay on record
(`Req 7.8`) and the R36/R37 gate records remain FAIL under the frozen 2,270
— no retroactive pass. Slice B production adoption (`Req 7.10`) moves from
withheld to **adoption planning unlocked**; production adoption execution
still requires the `Req 7.9` production-design gate and `Req 7.11`
revalidation of affected claims, and stays unscheduled until the owner
requests an adoption plan.

Lesscode shall support a bounded research workflow in which an LLM models a
complete code responsibility, proposes a smaller representation and revises it
using queryable source evidence, counterexamples and executable checks. The goal
is less maintained implementation with the same behavior, not smaller function
scores or reduced features. Prior experiments show both useful scoped reductions
and marginal or invalid replacements.

The [brief](brief.md), [research protocol](research.md) and
[dated research log](../../../docs/research/README.md) provide the rationale.
Requirements approved by the owner on 2026-11-02. Design generated in [design.md](design.md)
on 2026-11-07 with demonstrated vs. provisional slice separation per [prototype-gate.md](prototype-gate.md).
Criterion identifiers use section.item notation, for example 2.3.

## Boundary Context

- In scope: Python and TypeScript source; persisted, source-linked program queries;
  cross-function and shared-state modeling; bounded multi-pass LLM revision;
  supported constraint verification; isolated candidates; whole-scope measurement;
  staged experiments and retained evidence.
- Out of scope: feature reduction, application-domain redesign, unrestricted
  whole-program equivalence or global minimality claims, arbitrary language
  translation, Java, hardware/performance optimization, byte-based benefit scores,
  mandatory multi-backend support, and MCP/plugin delivery.
- Adjacent expectations: reuse the existing comparison core's measurement and
  supplied-check contracts; project runtimes and externally executed checks remain
  necessary. Comparator acceptance is not independent behavioral proof.
- Recorded design constraint: the owner selected CodeQL for the initial research
  backend. Tool licensing and source eligibility must be recorded. A separate
  graph database and orchestration framework are not prerequisites.
- Research success: a reproducible negative or unsupported result may complete a
  stage, but shall not be described as successful compression. Thresholds and
  budgets are declared for each scope before candidate generation; no universal
  reduction percentage is imposed.

## Requirements

### Requirement 1: Reproducible scope and research boundary

**Objective:** As a researcher, I want each trial bound to explicit source and
behavior, so that changing inputs cannot manufacture a successful result.

#### Acceptance Criteria
1. When a trial is registered, the Lesscode research workflow shall record its hypothesis, predecessor, source snapshot identity, license, language, entry points, owned implementation, affected caller boundary and exclusions.
2. When candidate generation is requested, the Lesscode research workflow shall record behavioral invariants, the frozen evaluator and checks, absolute and relative reduction targets, structural constraints and execution budgets.
3. The Lesscode research workflow shall classify production, tests, fixtures, dependencies and experiment apparatus separately.
4. If required provenance or source-use eligibility is unresolved, the Lesscode research workflow shall report the missing prerequisite and prevent the dependent analysis from starting.
5. While a specimen trial runs, the Lesscode research workflow shall preserve the original checkout and produce candidates only in an isolated trial scope.

### Requirement 2: Persistent program evidence and explicit uncertainty

**Objective:** As a modeler, I want queryable relationships across a repository,
so that I can reason about distributed behavior with source evidence.

#### Acceptance Criteria
1. When source is analyzed, the Lesscode research workflow shall retain a reloadable representation and record the source identity, extractor and analysis versions, configuration and diagnostics.
2. The Lesscode research workflow shall support queries for callable and caller boundaries, argument/return flow, control dependencies, shared-state reads/writes and observable effects, reporting unsupported relationship kinds explicitly.
3. When a query returns evidence, the Lesscode research workflow shall retain its query identity, source anchors, result and analysis assumptions.
4. The Lesscode research workflow shall distinguish extracted facts, inferred potential relationships, observed executions and LLM-proposed relationships.
5. If calls, aliases, callbacks, exceptions or state effects cannot be resolved, the Lesscode research workflow shall expose that uncertainty and shall not interpret missing edges as proof of unreachability or safe deletion.
6. When a retained representation is reloaded with the same recorded inputs, the Lesscode research workflow shall support comparison of normalized query results and report discrepancies.

### Requirement 3: Complete responsibility and behavioral models

**Objective:** As a developer, I want the LLM to model behavior spanning functions
and state, so that simplification can remove distributed implementation.

#### Acceptance Criteria
1. When a cluster is proposed, the Lesscode research workflow shall record its evidence, intended responsibility, relevant callers, shared state and boundary obligations; similarity alone shall not establish interchangeability.
2. When a behavioral model is proposed, the Lesscode research workflow shall record its input/output domains, state transitions, effect ordering, exception behavior, numeric assumptions and unresolved obligations.
3. The Lesscode research workflow shall link each modeled obligation to source or execution evidence, or label it as an unverified assumption.
4. When a replacement representation is proposed, the Lesscode research workflow shall identify the implementation it removes, preserved variant differences and all added support.
5. When local replacement generation is requested, the Lesscode research workflow shall establish a shared snapshot-bound inventory of available repository and framework capabilities.
6. If a proposal needs a dependency change, the Lesscode research workflow shall record it as a repository-level decision and shall not silently modify the frozen trial dependency set.

### Requirement 4: Bounded multi-pass modeling and revision

**Objective:** As a researcher, I want repeated evidence-driven passes, so that
the system can repair incomplete identification, modeling and simplification.

#### Acceptance Criteria
1. The Lesscode research workflow shall retain separately identifiable modeler, critic, candidate-author and evaluator outputs, including agent/model identity and execution settings.
2. When a pass requests more evidence or proposes a revision, the Lesscode research workflow shall identify the unresolved obligation or counterexample motivating that action.
3. When a model or candidate changes, the Lesscode research workflow shall retain its predecessor, changed assumptions, supporting evidence and revision reason without overwriting prior attempts.
4. If source, dependencies or shared contracts change, the Lesscode research workflow shall invalidate affected evidence and dependent claims before reuse.
5. If the declared pass, query, solver or inference budget is exhausted, the Lesscode research workflow shall stop the affected search and report exhaustion separately from rejection or equivalence.
6. If the declared no-progress condition is met, the Lesscode research workflow shall stop the loop and retain its unresolved obligations.
7. When an interrupted trial resumes, the Lesscode research workflow shall check retained input identities before continuing from its recorded state.

### Requirement 5: Model fidelity and behavioral challenge

**Objective:** As a developer, I want constraints challenged against source
behavior, so that a correct solution to a wrong model is not accepted.

#### Acceptance Criteria
1. When a solver-backed claim is prepared, the Lesscode research workflow shall declare the modeled semantics, supported operations, abstraction assumptions and source-model fidelity evidence.
2. When a candidate can be checked within the supported model, the Lesscode research workflow shall seek a behavioral difference and retain the exact query, solver version, bounds, result and available counterexample.
3. If a counterexample is found, the Lesscode research workflow shall distinguish a realizable source mismatch from an abstraction artifact or unresolved witness before attributing a source defect.
4. If a solver returns unknown, times out or checks only a bounded domain, the Lesscode research workflow shall report that limitation without promoting it to unbounded equivalence.
5. When an executable candidate is assessed, the Lesscode research workflow shall compare it against the frozen original using declared project checks and applicable differential or property checks.
6. If baseline checks fail or required observations are missing, the Lesscode research workflow shall preserve those outcomes and shall not report a fully passing behavioral gate.
7. While candidates are revised, the Lesscode research workflow shall preserve frozen checks and invariants; any amended evaluation protocol shall create a separately identified trial or dated amendment with recomparison.
8. When a meaningful behavioral mutation is introduced as an adverse control, the Lesscode research workflow shall record whether the evaluator rejects it and shall withhold evaluator-adequacy claims if it does not.

### Requirement 6: Net compression and honest acceptance

**Objective:** As a maintainer, I want whole-replacement accounting, so that
fragmentation, relocation or new scaffolding cannot masquerade as compression.

#### Acceptance Criteria
1. When a candidate is measured, the Lesscode research workflow shall count the complete owned implementation, added helpers, wrappers, tables, configuration, integration and changed callers, including additions outside the original files.
2. The Lesscode research workflow shall report defined physical-line and logical/statement measures separately, together with aggregate decisions, callable distribution and available call-layer evidence.
3. If a metric is unavailable, the Lesscode research workflow shall report it as unavailable rather than zero.
4. If the candidate misses the preregistered usefulness target, the Lesscode research workflow shall label it marginal or rejected rather than successful compression.
5. If behavior or required evidence fails, the Lesscode research workflow shall reject or leave the candidate inconclusive regardless of its size reduction.
6. When a candidate meets measured targets, the Lesscode research workflow shall retain a review of role-revealing names, useful types, explicit invariants and added indirection before recommending adoption.
7. The Lesscode research workflow shall report candidate validity, measured compression and actual adoption as separate outcomes, without summing overlapping candidate savings.

### Requirement 7: Staged empirical evaluation

**Objective:** As a researcher, I want small-to-large experiments with controls,
so that tool expansion follows evidence and can fail early.

#### Acceptance Criteria
1. When real-project modeling readiness is assessed, the Lesscode research workflow shall evaluate extraction on labeled Python and TypeScript cases containing cross-function flow, shared state, callbacks, aliases and negative controls.
2. When moving from extraction to small responsibilities, medium connected scopes or large-repository slices, the Lesscode research workflow shall record the previous gate's evidence and a continue, revise or stop decision.
3. When evaluating the modeling mechanism, the Lesscode research workflow shall compare source-only, graph-grounded and constraint-feedback arms under recorded comparable source, evaluator, model and resource budgets.
4. When an arm runs multiple passes, the Lesscode research workflow shall retain per-pass obligations, counterexamples, candidate results and cumulative research cost.
5. When a general efficacy claim is assessed, the Lesscode research workflow shall record repeated runs and held-out scopes separated from prompt/program optimization inputs.
6. When a large repository is sampled, the Lesscode research workflow shall report the selection method, covered population, unsupported population and per-scope outcomes without extrapolating savings to unexamined code.
7. When the first working loop is available, the Lesscode research workflow shall perform a supported-scope Lesscode self-trial with frozen evaluator checks and project regressions before claiming self-application maturity.
8. If a stage yields no useful reduction, the Lesscode research workflow shall retain the negative result without weakening its targets or requiring further infrastructure merely to continue.
9. When a production design is proposed for approval, the Lesscode research workflow shall require a linked real-code prototype result demonstrating the proposed mechanism, preserved declared behavior and a net reduction meeting the predeclared usefulness target.
10. If that feasibility gate is not-run, revise or stop, the Lesscode research workflow shall withhold dependent production task generation and execution while permitting a bounded isolated prototype plan.
11. When a material assumption supporting the feasibility gate changes, the Lesscode research workflow shall invalidate affected claims and require revalidation before dependent production work continues.

### Requirement 8: Dated research log and spec traceability

**Objective:** As a continuing researcher or agent, I want evidence-linked history,
so that later work preserves intermediate results and the reasons for revisions.

#### Acceptance Criteria
1. When a trial, failed attempt or material hypothesis decision occurs, the Lesscode research workflow shall create or amend a dated entry under docs/research and update its index.
2. The Lesscode research workflow shall record in each research entry its discussion/report date, actual execution interval when known, retrospective status when applicable, hypothesis, method, evidence links, limitations, decision and next gate.
3. When a research result affects specification scope, the Lesscode research workflow shall link the dated record to the relevant spec and add a reciprocal spec link without silently changing phase approvals.
4. When an interpretation changes, the Lesscode research workflow shall append a dated correction that preserves the prior result and identifies the superseded interpretation.
5. The Lesscode research workflow shall retain durable evidence sufficient to inspect results and replay the recorded procedure, with confidential credentials excluded from published artifacts.
6. When publishing progress, the Lesscode research workflow shall distinguish planned work, setup, completed execution and measured outcomes, and shall generate quantitative charts only from retained measurements.

