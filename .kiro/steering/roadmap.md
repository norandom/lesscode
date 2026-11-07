# Roadmap

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
