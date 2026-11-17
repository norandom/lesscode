# Task generation readiness

2026-11-17, 12:19 Europe/Berlin (clock: 11:19 UTC).
Owner request: "approved, produce the tasks". Requirements approval recorded.
Production task generation: not ready. No tasks.md was generated.

## Missing prerequisites

| Prerequisite | Current evidence |
| --- | --- |
| Approved requirements | Present: 11 areas, 83 criteria; owner approval recorded |
| Design describing the solution and its contracts | design.md absent |
| Frozen bounded prototype gate | prototype-gate.md absent |
| Real-code behavioral and useful-reduction evidence | No trials for this feature; R114 is static analysis only |
| Qualifying proceed decision and design approval | Neither exists |

The [task-generation skill](../../../.agents/skills/kiro-spec-tasks/SKILL.md)
requires a `proceed` decision linked to real-code evidence before production tasks.
The [prototype-first policy](../../steering/prototype-first.md) permits provisional
design and bounded research planning before that decision. Approval of the
requirements does not supply missing design or empirical evidence.

## Bounded prerequisite checklist

This is a research/design readiness checklist, not an executable production task
plan. Items remain open. Execution cannot begin until the prototype plan has concrete
frozen inputs, targets, budgets and the authorization required by project policy.

- [ ] **Prepare a provisional design and resolve contract tensions.**
  Use [research.md](research.md) and the approved requirements to define the
  supported branch/state measurements, responsibility and identity matching,
  requested ranking behavior, independent report outcomes, proposal/check handoff,
  provenance and pack/ledger compatibility. Include a contradictions-and-disposition
  table. Reconcile inherited solution-unlock guidance with this feature's approved
  branch/state objective; do not import historical line-reduction targets as its
  simplicity measure. Output: design.md marked provisional, design approval false.

- [ ] **Freeze a bounded prototype gate for both named cases.**
  Depends on the provisional design. Inspect complete responsibilities around
  exporter.py and tools.py, preserve the source clone, and identify the code a
  candidate could replace plus the support it would need. Freeze revisions,
  dependencies, tests/pins, measurement definitions, numerical absolute/relative
  usefulness targets, comparison arms, adverse controls, call/run budgets and stop
  conditions. Resolve the specimen test-runtime route under current execution
  rules. Output: prototype-gate.md, initial status not-run, with an explicit review
  and execution-authorization boundary. No candidate generation before the freeze.

- [ ] **Run the qualified disposable prototype within that gate.**
  Depends on the frozen plan and its execution authorization. Qualify measurements
  against the anti-gaming controls; establish behavioral checks on unchanged code;
  attempt the state and branching cases through the seam and step flow. Retain
  failures, unresolved coverage, raw model answers and manual interventions.
  Output: reproducible artifacts, candidate diffs, checks and same-profile
  before/after measurements, plus a dated research entry and reciprocal index links.
  This item is not permission to begin production implementation.

- [ ] **Assess evidence and revise the design to the demonstrated scope.**
  Depends on recorded prototype results. Assess proceed/revise/stop against the
  frozen gate and the applicable evidence-review rules. At least one real
  responsibility must meet its useful-reduction target and behavioral controls;
  both named cases must have recorded attempts. Unsupported scope stays excluded.
  Output: evidence-linked gate decision and revised design ready for owner review.
  A failed gate returns to hypothesis/scope revision, not production task generation.

- [ ] **Generate the production task plan after design approval.**
  Depends on qualifying evidence and owner approval of the revised design. Map
  all approved criteria and demonstrated design contracts to bounded tasks with
  observable completion conditions and explicit dependencies. Perform the required
  task-plan review using the owner's permitted review mode. Output: tasks.md and
  task metadata, initially pending owner approval. Only this step completes the
  requested production task generation.

## Current next action

`$kiro-spec-design model-simplification` to prepare a provisional design and
prototype gate. Requirements are already approved; do not request their approval
again. Design/tasks remain ungenerated and unapproved. No experiment ran during
this readiness check.
