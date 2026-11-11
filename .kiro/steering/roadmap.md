# Roadmap

## Work queue, owner order of 2026-11-10 (do not drop the second item)

1. **Agnostic driver** (`agnostic-annotation-driver`): requirements, design and tasks approved 2026-11-10.
   Task groups 1 to 7 done by 2026-11-11: the driver serves `analyze`, `mutate`, `run` and `admit-pack` for four
   languages, the six text-matching adapter files are deleted, and the specimen results are restated
   ([R75](../../docs/research/2026-11-11/01-specimen-controls-rerun-through-driver.md)). No release contains
   it yet; a release is the owner's decision.
2. **Slice B: hardening against LLM inference drift** (`constraint-guided-compression`, requirements
   Amendment 5; blueprint in `.kiro/task_26_handover.md`). The owner's second priority, explicitly not to be
   forgotten. Started 2026-11-10: the pack-authoring baseline ran on the current endpoint
   ([R68](../../docs/research/2026-11-10/14-slice-b-pack-authoring-drift-baseline.md)): admission 20 of 20, but Rust
   packs differ between runs unnoticed. Rerun with stronger controls
   ([R69](../../docs/research/2026-11-10/15-slice-b-baseline-rerun-stronger-admission.md)): Rust 0 of 5 on the first
   reply, repair works twice in five, Java differs in 3 of 5. Optimisation of the guide with a copied GEPA engine
   (R70 to R74): on Opus every first reply is admitted afterwards, Rust still differs from the hand pack in four
   fixed points, not adopted; on Mistral Large 4 the baseline admits 5 of 20 first replies and optimisation
   finds no improvement. Owner decisions of 2026-11-11: none of the four Rust points goes into the shipped
   pack; the node kind list is tried on both models; the stop for model spend is raised to 40 USD and the
   `MIPROv2` comparison is run. Results of 2026-11-11: the node kind list removes invalid kinds but not the
   admission gap on Mistral, and is not adopted
   ([R78](../../docs/research/2026-11-11/04-slice-b-node-kinds-result.md)); a MIPROv2-style search on Opus
   does not fix Rust admission where GEPA did
   ([R79](../../docs/research/2026-11-11/05-slice-b-miprov2-opus.md)). Annotation-layer baseline run on both models
   ([R83](../../docs/research/2026-11-11/09-slice-b-annotation-baseline.md)): Opus runs agree on 0.69 to 0.92 of their
   sites, three of five Mistral calls give no annotations. Two selection rules over three runs tested
   ([R85](../../docs/research/2026-11-11/11-slice-b-annotation-hardening.md)): the majority of three is more stable
   than one run in four of five files but misses the 0.80 target. Recorded spend about 32.9 of 40 USD.
   Status on 2026-11-11: measured for pack authoring and for annotation, between runs and between models, and
   under two optimisers for pack authoring; not measured: one model over time. Dated next step: the owner
   decides by the next session whether annotations get a hardening (sites named by at least two of three
   runs, or the union), whether drift over time is scheduled, or whether Slice B closes with the finding
   "the mechanical barriers decide soundness; a model's choice decides only how much is tried". Earlier dated step (2026-11-10, at the approval of the driver design): the preregistration of a drift baseline for pack authoring is driver task 2.4 and is due with the handover of driver task group 2. Five authoring runs per language for four languages with the frozen pack guide, current endpoint first, Mistral second; measures are admission rate, repair attempts and identity of the mutants. Running it needs the owner's go.
   Preregistered 2026-11-10 as [R66](../../docs/research/2026-11-10/12-slice-b-pack-authoring-drift-preregistration.md); start condition: driver task group 3 done, guide amendment hashed, owner's go.
   It must have a dated next step before `agnostic-annotation-driver` is reported complete
   (that spec's Requirement 9.6). What it inherits from the driver work:
   - The artefacts to harden are now small and checkable: the language pack a model writes once, admitted by
     deterministic checks, and the optional annotation layer.
   - Baseline to beat (R64): two identical annotation runs agree on 0.60 to 0.94 of their sites; annotation
     validity is 81% to 100%; packs were admitted on the first reply for four languages, so the repair path is
     untested.
   - Method named by the owner: an annotation-guidance process with bounded, verified `MIPROv2`, `GEPA` if
     needed. Opus endpoint now, Mistral later; switching models is itself a drift test.
   - Known endpoint behaviour to design for: one request was refused until its purpose was stated (gate
     Amendment 7).
   - Correction to the task 26 blueprint: its static reward gate assumed `deconvolute analyze`, which had no
     Python implementation; it becomes usable for all four languages only with the native driver.
## Current priority: demonstrate the solution first (2026-11-07)
[Slice](../specs/deconvolution-native-core/slice-proposal.md) and
[gate](../specs/deconvolution-native-core/prototype-gate.md) govern next work.
Requirements unapproved; documentation amendment only. Provisional planning and
authorized disposable prototypes precede unlock; production design/tasks/code do
not. Broader native suite deferred. Existing core has recorded Windows/Linux/macOS
GO in spec.json; older pending-platform statements below are superseded.
Constraint-guided compression is historical, not an active prerequisite.

## Proposed follow-up: constraint-guided compression

Owner backend selection, 2026-11-02: use CodeQL for the first graph/query-driven
research prototype on an eligible specimen. Reuse its database and language
libraries; no separate graph database or mandatory Joern comparison initially.
This selects a research backend without approving the remaining spec phases.

Owner direction, 2026-11-01: the next proposed mechanism is an LLM acting as
constraint modeler across connected code scopes, with solvers and executable
checks challenging candidate representations and feeding counterexamples back
into model revision. Discovery alone is insufficient. See the
[follow-up brief](../specs/constraint-guided-compression/brief.md).
This spec is initialized, not approved for implementation. The exclusions below
still apply to the delivered core; this proposal does not silently reactivate
retired solver or MCP layers. Marginal local deletions remain rejected evidence,
not a demonstration of substantial compression.

## Active delivery scope

Owner decision, 2026-11-01: drop layers first; extend only after demonstrated need.
The product path is the existing agent skill plus a narrow native comparison core
and existing project checks. The Rust design and all 22 tasks are approved and
implemented with independent task review. See [requirements](../specs/capability-compression-core/requirements.md)
and the [implementation proposal](../../docs/reports/native-compression-direction.md).

Current outcome: Windows native workflow verified; fresh ATS, NPU and TypeScript
trials accepted under explicit source targets and adverse controls detected.
The supported Python self-trial is reviewed and adopted: 29 statements and 32
physical lines removed, with paired regression evidence. Full cross-platform
readiness remains pending current Linux/macOS CI results; do not declare the
feature complete from local checks. See [readiness](../specs/capability-compression-core/readiness.md).
Use cargo-nextest for Rust and pytest-xdist for Python via `docs/testing.md`.

1. Replay ATS01 and its negative control with a supplied complete capability scope.
2. Challenge accounting and evidence using NPU01 and rejection fixtures: moved
   helpers, fragmented functions, missing checks and changed source identities.
   After this first working slice, run the self-application maturity gate below.
3. Exercise the same workflow on an unseen capability. The skill inventories
   installed capabilities once per repository snapshot, proposes a concrete
   replacement, and records all attempts, including failures.
4. Establish TypeScript evidence separately. Python evidence does not imply parity.

## Self-application maturity gate

Build and verify the first usable workflow, then use it on a complete Lesscode
capability. This is a maturity/release-claim gate, not a prerequisite to bootstrap
or continue development. Freeze the evaluator, source baseline, scope and checks;
work on an isolated candidate. Require unchanged relevant checks and the existing
project regression suite before adoption. Changes to evaluator logic additionally
need independent expected-result fixtures and an adverse case.

Record the trial and its net implementation change, structural tradeoffs and
behavioral evidence. Adopt only a demonstrated safe improvement. A well-supported
rejection may complete the workflow gate, but is not successful self-compression;
missing evidence leaves the gate open. No mandatory deletion percentage, weakened
tests, feature cuts or repeated refactors merely to obtain a green result.

Use a supported Lesscode capability first (the retained Python implementation is
eligible). Do not add Rust source-analysis support just to unblock this gate;
report native-core coverage separately and do not claim it from Python results.
Formal design/tasks must carry this gate into final validation.

Measure the whole replacement, including helpers, tables and changed callers.
Preserve behavior, APIs, useful types, names and safeguards. Count production,
checks, fixtures and research support separately. No byte/gzip benefit scores,
hardware validation or performance benchmarking. Tests are behavioral evidence,
not a default reduction target.

## Active specifications

- `agnostic-annotation-driver`: initialized 2026-11-10 as the owner's top priority (R56); brief and prototype gate written; gate attempt 1 was revise (R61); attempt 2 the same day met every mechanism row for the rules-as-data form on four languages with model-written packs for the held-out ones (R64); the apparatus ceiling is the open row; no requirements, design or tasks yet. R55 narrowed the dogfood claims below: the porcelain mutation operators are specimen-bound and the hook enforces budgets only.
- `dogfood-self-application`: requirements, design, tasks, and full clean-clone dogfooding validation completed with GO decision on 2026-11-09; verified across 352 native tests (100% pass), CRLF byte preservation in `.gitattributes`, deconvolution porcelain CLI, and intelligent Git pre-push hook integration (R54).
- `rust-deconvolution-unification`: requirements, design, tasks, prototype gate (R52), foreground TDD implementation (6 tasks), and validation completed with GO decision on 2026-11-09; adapter verified across 338 native tests (24 Rust adapter tests) and native lifecycle supervisor candidate (−40.32% lines, −15.66% statements, 100% adverse sensitivity).
- `java-deconvolution-unification`: requirements, design, tasks, prototype gate (R51), foreground TDD implementation (6 tasks), and validation completed with GO decision on 2026-11-09; adapter verified across 314 native tests (24 Java adapter tests) and real specimen `apache/commons-csv` `CSVPrinter.java` candidate (−38.46% lines, −27.68% statements, 100% adverse sensitivity).
- `typescript-deconvolution-adapter`: requirements, design, tasks, prototype gate (R50), foreground TDD implementation (6 tasks), and validation completed with GO decision on 2026-11-09; adapter verified across 290 native tests (20 TypeScript adapter tests) and real specimen `OpenUltraCode` degradation candidate (−43.78% lines, −40.38% statements, 100% adverse sensitivity).
- `native-deconvolution-closure`: requirements, design, tasks, prototype gate (R49), foreground TDD implementation (7 tasks), and validation completed with GO decision on 2026-11-09; substrate closure certified across 270 native tests.
- `state-boundary-deconvolution`: requirements, design, tasks, prototype gate, dual-specimen trials (E5, E6), capability adoption, COBESY synthesis report, and integration validation completed with GO decision on 2026-11-09. All 11 tasks implemented and certified.
- `deconvolution-loop`: requirements, design and tasks approved 2026-11-04; research
  tasks executed through E4 (R26); four R26 amendments proposed, NOT adopted (owner
  ratification pending).
- `deconvolution-native-core`: initialized 2026-11-06; on 2026-11-07 the arbitrated
  fusion plan was drafted into [requirements](../specs/deconvolution-native-core/requirements.md)
  (amendment 2026-11-07, generated, NOT approved) with [gates](../specs/deconvolution-native-core/gates.md),
  [verification](../specs/deconvolution-native-core/verification.md),
  [spikes](../specs/deconvolution-native-core/spikes.md) and the scope guard
  [breath.toml](../specs/deconvolution-native-core/breath.toml); project-level
  [breadth.toml](../breadth.toml), [complexity.md](complexity.md) and
  [../complexity-budgets.json](../complexity-budgets.json) added. Plan-only until the
  pre-kiro owner gate passes; no Rust code.
- `capability-compression-core`: requirements, design and [tasks](../specs/capability-compression-core/tasks.md)
  approved through the explicit task-generation `-y` invocation. Independent task
  graph review passed: all 22 executable tasks in 7 groups are implemented and
  independently reviewed; broader platform validation remains outstanding.
  One native library/CLI, not a port of the Python evaluator.
- `deconvolute-skill`: evolve the existing experimental discovery skill alongside
  that core; no dependency on the broad rule engine or integration platform.

## Removed from the delivery scope

The broad agent-quality-rules platform, generic transactional rewrite engine,
MCP/plugin/hook delivery, separate pattern adviser, autonomous synthesis platform,
and multiple specialized rewrite backends are dropped from the active plan.
They are not queued milestones or prerequisites. Z3 remains research evidence;
there is no solver integration task or extension interface in this slice.

Preserved specs are historical records, not instructions to resume. A later
extension requires an observed unmet need and a new scope/design decision; do
not pre-build adapters, registries or abstractions for these removed layers.
Simple scope-level targets and frozen checks stay in the comparison workflow.

## Retained evidence and implementation

The completed efficiency-baseline and existing Python CLI remain the reference.
Agent-quality-rules has 9/25 tasks marked complete; its remaining plan is retired
from this delivery scope. Preserve existing source and uncommitted work. No public
CLI contract is removed by this roadmap change. Frozen ATS01/NPU01 and rejected
trials remain evidence; two manually selected successes do not establish automatic
candidate discovery efficacy or measured maintenance savings.

Facdrone, ATS and NPU are specimens, not upstream deliverables. Use isolated trial
copies and leave source projects unchanged. Lesscode studies less code with the
same behavior; feature deletion and domain redesign cannot manufacture success.
