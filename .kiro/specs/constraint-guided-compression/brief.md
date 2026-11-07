# Constraint-guided compression

## Amendment, 2026-11-07: H8/H9 apparatus alignment and DSPy-style probabilistic turn refinement ([R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md))

Owner direction, 2026-11-07: reconcile `constraint-guided-compression` with the completed
detection and verification cores ([`capability-compression-core`](../capability-compression-core/spec.json)
in `native/src/{scope,metrics,compare,model}.rs` and [`deconvolution-native-core`](../deconvolution-native-core/spec.json)
in `native/src/{admission,receipt,budgets}.rs` + [`src/lesscode_baseline/recording.py`](../../../src/lesscode_baseline/recording.py)),
promote the proven [`deconvolution-loop`](../deconvolution-loop/design.md) detection and measurement
modules (`surface.py` with Contract B from [R29](../../../docs/research/2026-11-07/03-s9-surface-contracts.md),
`measures.py`/`measures2.py`/`measures3.py`/`pureshare.py`, `editscope.py`, `testinfra.py` from
[R32](../../../docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md), and `regen_turn.py`
with the four [R26](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md) amendments) into
`src/lesscode_baseline/`, and adopt **DSPy-style probabilistic prompt iterations (`dspy.Refine` / `dspy.BestOfN`)**
with a two-tier programmatic reward function (`StaticAssertionGate` $\to$ `RuntimeOracleGate`) so that
weaker models or non-deterministic sampling responses (such as E4 Turn 1's thinking-budget exhaustion
or E4 Turn 2's missing surface symbol) recover automatically across bounded additional turns without
weakening the frozen acceptance oracle.

- **Withdrawn from the active design surface**: H7's static CodeQL state-flow models, LLM-authored
  SMT equivalence models, and in-place family factorings ([R08](../../../docs/research/2026-11-02/08-codeql-state-boundary.md)–[R19](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md)).
- **Demonstrated slice (`proceed`)**: Static surface Contract B (`surface.py`), 3-stratum canonical
  and separated-form AST/edge/write-site/pure-share measures (`measures.py`), edit-scope and patch
  sentinel checks (`editscope.py`), static test-infrastructure detection (`testinfra.py`), and
  single-turn recorded regeneration (`regen_turn.py`) wired to `lesscode_baseline.recording` and
  `lesscode-core` (`admission`, `receipt`, `budgets`, `compare`).
- **Provisional slice (`revise` — gated by spike `G1-R` in [prototype-gate.md](prototype-gate.md))**:
  Multi-rollout DSPy-style `Refine`/`BestOfN` recovery (`H10`) and optional offline DSPy prompt
  compilation (`Req 7.5`). See [design.md](design.md) and [R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md).
  Earlier sections below are retained as historical context.

---

Research evidence: [dated lab log](../../../docs/research/README.md),
[prior-trial retrospective](../../../docs/research/2026-11-02/01-retrospective.md),
and [CodeQL decision/setup](../../../docs/research/2026-11-02/02-codeql-direction.md).
Record subsequent trials there and link their decisions back to this spec.

The [2026-11-01 originating decision](../../../docs/research/2026-11-01/07-llm-constraint-modeler.md)
records why the owner redirected the mechanism from candidate ranking to
LLM-guided constraint modeling. It was backfilled on 2026-11-02.

## Status and intent

Owner design constraint, 2026-11-02: follow the
[prototype feasibility gate](prototype-gate.md). Draft the proposal, test the
risky mechanism on representative real code, and establish useful net compression
before final design approval and production task generation. Fail early and
revise the hypothesis rather than execute an unvalidated task list.

Backend decision, 2026-11-02: the owner selected CodeQL for the first research
prototype. Use its persisted database and existing Python/TypeScript analysis
libraries as the query surface. Export focused source-linked results for the
LLM modeler; do not build a separate graph database first. Joern is a fallback,
not a mandatory parallel implementation. This decision selects the research
backend; it does not approve all requirements, design or implementation tasks.

Follow-up specification requested by the owner on 2026-11-01. Requirements were
generated and approved on 2026-11-02; final design and tasks are not approved.
The [design proposal](design-proposal.md) remains behind the prototype gate. See the
[dated generation record](../../../docs/research/2026-11-02/03-requirements.md).
This is a new scope proposal
after capability-compression-core, not permission to resume retired synthesis,
MCP or generic rewrite infrastructure.

## Problem

Developers using agents accumulate implementation spread across functions,
modules and support layers. Existing Lesscode measurements can evaluate a
replacement but do not drive substantial compression. Measuring manually chosen
local rewrites misses the central problem: constructing a smaller implementation
of the same behavior across a complete scope.

The recent [mvibe trial](../../../docs/experiments/proxy-routing-trial/README.md)
removed 23 of 334 module lines; the
[Facdrone trial](../../../docs/experiments/facdrone-csv-trial/README.md)
removed only 3 of 1,056. These are test-based trials, not SMT results. The latter
is marginal and rejected. Neither establishes that the repositories lack
compression opportunities. They expose a limitation in the implementation and
experimental method. Preserve these outcomes rather than retroactively calling
them constraint-driven synthesis.

## Central hypothesis

An LLM acting as the constraint modeler can drive meaningful scope-wide
compression by proposing behavioral models and smaller representations, then
revising them using solver counterexamples and executable observations.
Discovery and metrics supply evidence to this loop; candidate ranking alone is
not the proposed solution.

The solver checks the encoded problem. It does not discover complete application
semantics automatically or prove that the LLM's model faithfully describes the
source. Model fidelity is a separate obligation.

## Proposed loop

Owner refinement, 2026-11-02: use a serialized, queryable program graph to drive
multi-pass LLM constraint modeling across functions and scattered state. CFGs
alone and isolated if/then/else synthesis are insufficient. Prototype graph
fidelity, semantic clustering, state/effect models and bounded refinement at
small, medium and large scopes. Evaluate existing Joern/CodeQL-style query
surfaces, DSPy and established agent orchestration before building infrastructure.
See [research and staged prototype protocol](research.md) for tool evidence,
state-explosion limits, comparison arms, failure gates and the COBESY record plan.
These are research candidates, not selected production dependencies or approvals.

1. Establish repository-wide framework and dependency knowledge once per source
   snapshot. Reuse that inventory across local scopes; do not introduce a new
   framework independently for each function.
2. Traverse connected code scopes. The LLM models entry points, callers, outputs,
   state transitions, effects, exceptions, public contracts and uncertainties,
   citing source and test evidence. Preserve input ordering and numeric semantics.
3. Describe alternative representations: existing library calls, shared kernels,
   declarative tables or simpler algorithms. State which implementation each
   alternative removes and which distinctions remain as data or behavior.
4. Express hard behavioral constraints and explicit structural objectives across
   the entire replacement. Include helpers, tables, adapters, configuration and
   changed callers, including code moved outside the original boundary.
5. Synthesize or evaluate a candidate under those constraints. For supported
   semantics, seek an input on which the candidate and source model differ.
   Combine this with differential, property and regression checks against actual
   source execution. Return counterexamples to the LLM for model or candidate
   revision; never weaken the frozen behavioral boundary to obtain acceptance.
6. Measure the whole replacement and its wider module/project effects. Accept
   only useful net simplification with adequate behavioral evidence. Record
   rejected, unsupported, unknown and exhausted attempts as distinct outcomes.
7. Continue across connected scopes, invalidating dependent models when source,
   shared contracts or dependency assumptions change. Preserve verified contracts
   as inputs to subsequent scopes, not unconditional proof of their composition.

## Two constraint layers

**Representation selection:** model which implementations and dependencies remain,
which responsibilities can share code, and the cost of each candidate. The LLM
proposes a bounded alternative space; a feasible or optimal solution is relative
to that space and cost model, not a globally minimal program.

**Behavioral preservation:** check equivalence within explicit domains, sorts and
effect models. Use SMT where supported; tests cover executable observations and
remaining boundaries without being described as universal proofs. Report solver
`unknown`, timeouts and bounded search honestly. Real arithmetic does not prove
Python float or TypeScript number equivalence. External library substitutions
need trustworthy contracts and actual integration checks.

Branch depth and complexity caps are adjustable structural constraints, not a
universal mandate for flat dispatch. Reject caps that force indirection, conceal
behavior or merely redistribute decisions across atomic functions.

## First experiment to specify

Use an isolated, substantial responsibility from a grown repository. Facdrone,
mvibe and mmhmem are possible specimens, not deliverables. Do not change or commit
upstream specimens. Target Python and TypeScript initially; do not expand language
support merely to include a specimen.

Freeze source identities, accounting boundaries, dependencies and existing checks.
Before candidate generation, record a scope-relative and absolute usefulness
threshold, structural objectives and revision budget. Three or four lines in a
large module do not qualify as substantial compression; retain such an outcome
as a rejection, not a successful trial.

Compare the existing manually guided method with the LLM-modeler loop on the
same scope and evaluator. Retain each proposed model, its source evidence,
constraints, replacement, counterexample, revision and final decision. Include an
adverse candidate that violates a meaningful behavior and confirm its rejection.
Do not force a successful result or broaden infrastructure to manufacture one.

Report production lines, AST statements or explicitly defined LLOC, aggregate
decisions, call layers and function distribution before and after. Count tests,
fixtures and research machinery separately. Record versions, commands, excluded
semantics, baseline failures and proof coverage. Distinguish test evidence,
bounded checks, SMT equivalence under a model, and source-model fidelity evidence.
Line counts do not establish maintenance-effort reductions.

## Boundaries and decisions for the next phase

- Same features and observable behavior; no requirements reduction or
  application-specific domain redesign.
- Reuse the native Rust measurement core and existing project checks where
  appropriate. Python prototypes use local uv. No MCP packaging prerequisite.
- Open-source tooling; no mandatory commercial analyzers. No hardware or
  performance optimization, and no byte/gzip benefit metrics.
- Requirements must settle the minimum model record, fidelity checks, scope
  traversal, constraint semantics, unsupported-case behavior and empirical
  usefulness criteria before implementation design.
- Design must determine how to represent effects and cross-scope contracts,
  which solver-supported subset to start with, and how much machinery a single
  credible trial actually requires. Do not assume a general Python/TypeScript
  symbolic compiler is necessary.
- Later self-application on Lesscode is a maturity check after a working loop;
  it must preserve tests and functionality and must not block bootstrap.
