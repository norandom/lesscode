# Brief: brownfield-test-stage

Written 2026-11-24 from the owner's direction of that day, given while watching a real run of
`step plan --page --open` on the renderer of drift-rdp (run-dashboard task 8.1). A brief, not
approved requirements. Nothing here is built except the page's reference to the stage. The open
questions at the end are the owner's to decide.

## Problem

lesscode is there to work an existing codebase into less code. The tests of a real project are only
half good: on the renderer of drift-rdp they noticed 72 of 268 planted faults, and 21 functions had
no fault noticed at all (R164). A reduction in such code can change behaviour without any test
failing.

The owner, looking at the list of unprotected functions on the run page: "what do we do with
functions which are unprotected? ignore them? that is not clear", and then: "with lesscode the
emphasis is that we work a codebase into being 'less code'. so we should have a test brownfield
stage, where we improve this."

A developer who has read a plan has no stage that leads from "these functions are unprotected" to
"this code is protected well enough to be reduced". The page ends at the list.

## Current State

- `step plan` lists per function the planted faults the tests notice and miss, and what an
  unprotected function reaches out to.
- `step pin` judges one new test against one unnoticed fault of the base: admitted when the tests
  with the pin pass on the unchanged code and fail on the fault. It needs a step declaration and is
  asked fault by fault.
- `step check` judges the reduction and refuses a step that lets unnoticed faults rise.
- The skills `untangle-state` and `refactor-in-steps` hold the loop an agent follows.
- Two trials on `shared_frame.rs` of the renderer (2026-11-23): four tests written from uncovered
  shapes of the state noticed one fault in eight (R171); two tests written from the remaining
  faults noticed three of seven, and the four left were not calls for a test (R172). The rework
  that followed was refused once by the released flow and then accepted (R173).
- The brief [lesscode-metrics](../lesscode-metrics/brief.md) names the wish as item 3, "faults,
  tests and reduction hand in hand", and lists as missing "a command that writes tests for
  unnoticed faults and admits them through `step pin`". Since 2026-11-23 the owner's rule is that
  the tool holds no model: an agent following a skill writes, the tool measures and judges, the
  person decides.

The gap: no stage of its own between plan and reduction. Nothing takes a chosen set of functions,
judges all new tests against all their unnoticed faults at once, shows before and now per function,
records which remaining faults are no call for a test, and says when the reduction may begin.

## Desired Outcome

One path a developer can read as a lane: see what is protected, protect what you will change, make
it less.

For the middle stage:

- The developer names the functions to be reduced. The other unprotected functions are left alone,
  and the page says so.
- A spec in the project under work says what is to be protected, and the person approves it before
  a test is written.
- An agent working the spec's tasks writes the tests, on a branch, in commits that touch tests only.
- A test counts when `step pin` admits it; the stage is through for a function when a second plan
  shows its faults noticed, or a person has written down why a remaining fault is no call for a
  test or cannot be tested as the code stands.
- The page and the guide lead from the list of unprotected functions to this stage, and from it to
  the reduction.

## Approach

The owner's direction, 2026-11-24, on what the page should say about an unprotected function:
"there I'd simply refer to the brownfield and make a spec to produce the tests", and on how:
"kiro allows this. we can use this".

So the stage is a spec, worked with Kiro, whose product is the tests.

- The run page refers to the brownfield test stage where it lists the unprotected functions (done
  on the page on 2026-11-24: the tab Unprotected says to make a spec whose tasks are the tests).
- The developer, or an agent, makes a spec in the project under work: its requirements are the
  chosen functions and what their unnoticed faults say is not protected; its tasks are the tests.
- The tasks are worked the usual way. A test counts when the existing rule admits it: it passes on
  the unchanged code and fails on a planted fault (`step pin`), and a second `step plan` shows the
  function with noticed faults.
- lesscode supplies what the spec starts from: the plan's list of the chosen functions with their
  unnoticed faults, in a form a spec can take in. Whether that is a file the tool writes or a
  chapter of the skill that reads the plan is for the design.

Why this one: nothing new judges anything; the judgement stays with `step plan`, `step pin` and
`step check`, which six real steps went through. The work of writing tests gets the same reviewed
phases as any other work, and the person approves what is to be protected before tests are written.

Considered and not proposed:

- **A command of its own for the stage** that judges all new tests against all faults of the chosen
  functions in one run, keeps before and now per function, and gates the reduction. More to build
  and a second judge beside `step pin`. It can follow if the spec route shows that asking fault by
  fault is too slow; the design should say what it measured.
- **The tool writes the tests through model calls of its own.** Ruled out by the owner's decision
  of 2026-11-23.

## Scope

- **In**: the reference to the stage on the run page and in the guide; what a spec of this stage
  holds (chosen functions, their unnoticed faults, tests as tasks, how a remaining fault is written
  off and by whom); what lesscode hands the spec to start from; the chapters of the skills
  `untangle-state` and `refactor-in-steps`; a first real subject (the renderer of drift-rdp, whose
  numbers exist).
- **Out**: writing tests inside the tool; a new judge beside `step pin`; the reduction itself
  (`step check` stays as it is); splitting code that cannot be tested as it stands (the skill
  `untangle-state` holds that cut); recording which test notices which fault.

## Boundary Candidates

- From the plan to the start of a spec: the chosen functions and their unnoticed faults.
- The shape of the spec: requirements from faults, tests as tasks, the written reason for a fault
  left alone.
- Admission of the tests (`step pin` and a second plan, as they are).
- Where the page and the guide refer to the stage.
- The skills: what an agent does in this stage.

## Out of Boundary

- No model call in the tool.
- No change to how faults are planted or classed.
- No measure of the quality of a test beyond whether it notices a planted fault and passes on the
  unchanged code. Whether the behaviour it fixes is the wanted one stays with the person.

## Upstream / Downstream

- **Upstream**: [gradual-refactoring-steps](../gradual-refactoring-steps/design.md) (`step plan`,
  `pin`, `check`, the ledger); [run-dashboard](../run-dashboard/tasks.md) (the page, its lane and
  snapshot); the fault runner of the driver.
- **Downstream**: the subsystem reworks of [lesscode-metrics](../lesscode-metrics/brief.md); the
  examples a developer can run; the documentation of the whole path.

## Existing Spec Touchpoints

- **Extends**: run-dashboard (the reference on the page); the guide of gradual-refactoring-steps.
- **Adjacent**: lesscode-metrics (its item 3 is this stage; its page work went into run-dashboard);
  state-consolidation (the finder is not touched).

## Constraints

- Rust only; no Python apparatus. The page stays one HTML file embedded in the binary.
- Python, TypeScript, Java and Rust are treated alike.
- Every planted fault costs a build and a test run of the project. On the renderer 268 faults took
  1 hour 43 minutes on two copies; on 2026-11-24 three copies building at once ran out of memory on
  a machine with 8.4 GiB free. The stage has to run only the faults that matter and say how long it
  will take.
- Research-bearing: the design passes the prototype gate of
  [prototype-first](../../steering/prototype-first.md) on real code before production tasks.
- Token budgets of `runner.rs` and `page_log.rs` are at their limits; new code goes into new
  modules.

## Decided by the owner on 2026-11-24

Nothing is open. The answers are requirements for the spec.

- **Protected enough**: a function may be reduced when every planted fault in it is noticed by a
  test or written off with a reason. A share of the faults is not enough, and neither is leaving
  the list of unprotected functions.
- **Who writes a fault off**: the agent proposes the reason, in the spec; it counts when the owner
  of the code has approved it. A fault is written off as "changes nothing a caller can see" or as
  "cannot be tested as the code stands".
- **Where the spec lives**: in the project under work, in its own `.kiro/specs`.
- **When**: after run-dashboard group 8, which is closed.
- **The first run of the stage is watched**: the stage starts with a plan on the renderer of
  drift-rdp, and the owner watches the run page follow it. That is the look at the changed page
  that the validation of run-dashboard names as missing.

## Related

- The run and the owner's words this brief comes from: [R181](../../../docs/research/2026-11-24/02-run-page-a-real-run-watched.md).
