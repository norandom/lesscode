# Requirements: the brownfield test stage

Generated 2026-11-24, 11:10 (Europe/Berlin) from the [brief](brief.md). Approved by the owner on
2026-11-24 ("approved, go on to the design"), with the five points below.
Evidence: [R164](../../../docs/research/2026-11-23/01-renderer-baseline-faults.md) (on the renderer
of drift-rdp the tests notice 72 of 268 planted faults),
[R171](../../../docs/research/2026-11-23/08-tests-for-four-uncovered-shapes.md) and
[R172](../../../docs/research/2026-11-23/09-tests-from-the-remaining-faults.md) (tests written from
what a plan shows; four of seven remaining faults were no call for a test),
[R181](../../../docs/research/2026-11-24/02-run-page-a-real-run-watched.md) (the owner's words this
stage comes from).

## Project description

**Who has the problem.** A developer who wants to make existing code smaller with lesscode, and the
agent that works for them.

**The situation today.** A plan shows in which functions the project's tests notice no planted
fault. The page says what that means. Nothing leads from there to code that is protected well
enough to be reduced: which functions to take, what tests to write, when it is enough, and what to
do with a fault no test should chase.

**What should change.** Between the plan and the reduction there is a stage of its own. The
developer names the functions to be reduced. A spec in their own project says what is to be
protected, and they approve it. An agent writes the tests the way Kiro's implementation already
works, test first, with the planted fault in the place of the missing implementation. The stage
ends per function when every planted fault is noticed or written off with a reason the developer
approved.

## Points proposed for approval with this document

1. **The stage writes tests and nothing else.** No file of the code to be reduced is changed in it.
   A fault that needs a change of the code before a test can see it is written off as "cannot be
   tested as the code stands"; cutting such code is the work of the skill `untangle-state`.
2. **The stage advises, it does not lock.** It says when the reduction of a function may begin.
   Nothing new in lesscode refuses a reduction because the stage was skipped; `step check` judges a
   reduction as it does today.
3. **Kiro's form is copied, not required.** The spec of the stage has the form of a Kiro spec and
   its tasks follow Kiro's test-first protocol. Where the project under work has its own Kiro
   setup, that is used. Where it has none, the stage brings the form with it and installs nothing.
4. **One requirement per function, one task per test.** A test may notice several faults; a fault
   counts when any test notices it.
5. **The first subject** is the renderer of drift-rdp, and the owner watches the run page follow
   the first plan of the stage. That is the prototype gate of the design and the look at the
   changed page that the validation of `run-dashboard` names as missing.

## Amended 2026-11-24, 13:10: one review

The owner, after the plan of the prototype, chose one review at the end over an approval before
the tests and one per write-off. Changed below: 2.5, 4.3, 4.4 and 7.3. Before the change 2.5
forbade a test before the spec was approved, and 7.3 named three stops.

## Amended 2026-11-24, 18:35: the whole scope by default, and what holds the stage back

**Approved by the owner on 2026-11-24** ("approved, start with 6.1"). It comes from task 5.1 ([R188](../../../docs/research/2026-11-24/09-protect-stage-yuv.md)).
That run spent eleven of the owner's approvals on debug labels and wrote one test. The owner, in
their own words: "we do the fail injection brownfielding because we want a better reach for
lesscode" and "the idea of lesscode isn't per function scoping". They chose "whole scope by
default". Asked "whether a fault injection that is successful must always prevent us", they chose
three cases:

- a missed fault that a caller sees, in code the reduction changes, holds the stage back;
- a missed fault no caller can see is not planted;
- a missed fault a caller sees, outside the code the reduction changes, is reported and does not
  hold the stage back.

The stage runs before the reduction, so it cannot know which code the reduction will change. The
chosen functions stand for that code. By default they are every function with an unnoticed fault,
so every missed fault a caller sees holds the stage back. Only when the person names functions
does the third case apply, to the functions they did not name. Whether the reduction later stays
inside the chosen functions is judged by `step check` of the reduction, as today.

The changes, each with the text it replaces:

- **1.1** now reads: "When the stage begins on a plan, the stage shall take as chosen every function
  of the plan that has an unnoticed fault; where the person names functions, the stage shall take
  those functions only. It shall list for each chosen function its noticed and unnoticed faults,
  each unnoticed fault with its place and what was changed." It replaces: "When the person names
  functions of a plan, the stage shall take those functions as the chosen functions and shall list
  …".
- **1.2** now reads: "Where the person names functions, the stage shall list every other function of
  the plan with an unnoticed fault as reported, with the number of its unnoticed faults from the
  plan. It shall say that these do not hold the stage back and that a reduction of them is not
  covered." It replaces: "… that it is left alone and that nothing has to be done about it."
- **5.10** is new: "When the stage says whether the chosen functions are protected enough, the stage
  shall also list the reported functions of 1.2 with their numbers from the plan, and shall not let
  them change the outcome."
- **7.1** now reads: "The stage shall let an agent choose nothing. The scope comes from the plan the
  person asked for, and only the person narrows it to named functions." It replaces: "… the chosen
  functions come from the person."
- **Not planted, outside this spec.** A missed fault no caller can see is a matter for the language
  pack that plants it. The boundary below keeps "how faults are planted" out of scope. So the
  second case is met by a change to the pack of the spec `agnostic-annotation-driver`, recorded in
  the design: the Rust pack plants no empty string in a struct field named `label`. Requirement 4
  (writing a fault off) stays as it is for whatever is still planted.

## Introduction

lesscode works an existing codebase into less code. A reduction is only as safe as the tests that
would notice a mistake in it, and the tests of a real project notice a part of what can go wrong.
This feature gives the path a middle stage: see what is protected, protect what you will change,
make it less.

Terms: a *plan* is the result of `step plan`: per function, the planted faults the tests noticed and
the ones they did not. A *chosen function* is a function of a plan that the person wants to reduce.
The *stage* is the brownfield test stage, for one set of chosen functions. The *stage spec* is the
specification the stage writes into the project under work. A *protection test* is a test written
in the stage. A fault is *written off* when a person has approved a reason why no test is to
notice it. A function is *protected enough* when every planted fault in it is noticed or written
off. The *owner of the code* is the person who approves in the project under work.

## Boundary Context

- **In scope**: choosing functions from a plan; the stage spec in the project under work; writing
  protection tests first against planted faults; writing a fault off; saying when a function is
  protected enough; where the page and the guide lead into the stage and out of it.
- **Out of scope**: writing tests inside the tool, or any model inside the tool; changing the code
  to be reduced; the reduction itself; how faults are planted or classed; a new judge beside the
  existing ones; new views of the run page; recording which test notices which fault.
- **Adjacent expectations**: a plan, the admission of a single test against a single fault, and the
  judgement of a reduction stay as they are and keep their exit codes. The run page shows a plan of
  the stage like any other plan. The skills `untangle-state` and `refactor-in-steps` name the
  stage where they speak of unprotected functions.

## Requirements

### Requirement 1: Choosing what to protect
**Objective:** As a developer, I want to name the functions I am going to reduce, so that tests
are written where I will work and nowhere else.

#### Acceptance Criteria
1. When the person names functions of a plan, the stage shall take those functions as the chosen
   functions and shall list for each its noticed and unnoticed faults, each unnoticed fault with its
   place and what was changed.
2. The stage shall say for every unprotected function of the plan that was not chosen that it is
   left alone and that nothing has to be done about it.
3. If a named function is not in the plan, then the stage shall say so and shall name the functions
   the plan has.
4. If the plan could not judge anything, then the stage shall not begin and shall say that a plan
   with a result is needed first.
5. If a file of a chosen function has changed since the plan was made, then the stage shall say
   that the plan no longer describes the code and shall ask for a new plan before it goes on.
6. Where a chosen function lies wholly in code that is not built on the current system, the stage
   shall say that nothing is known about it here and shall leave it out.
7. Where the plan lists what a chosen function reaches out to, such as a device, a window or a
   network, the stage shall show that with the function.

### Requirement 2: A spec in the project under work
**Objective:** As the owner of the code, I want to read and approve what is going to be protected
before any test is written, so that the tests fix behaviour I want kept.

#### Acceptance Criteria
1. When the chosen functions are settled, the stage shall write a stage spec into the project
   under work, at the place where that project keeps its specs.
2. The stage spec shall hold one requirement per chosen function, which says in words what a
   change could alter unseen there, derived from the unnoticed faults of that function.
3. The stage spec shall hold one task per protection test, which names the faults the test is to
   notice.
4. The stage spec shall name the plan it was made from, the commit of the code, and the command
   that runs the project's tests.
5. The stage shall put the stage spec, the protection tests and the proposals to write faults off
   before the owner of the code in one review, and shall count nothing as approved before it.
6. Where the project under work has its own way of keeping specs in Kiro's form, the stage shall
   use it; where it has none, the stage shall write the stage spec in that form without installing
   anything in the project.
7. The stage shall write the stage spec in the language the project under work uses for its specs.
8. The stage shall write nothing outside the project under work except into the directory of the
   copies.

### Requirement 3: Tests first, against the planted fault
**Objective:** As a developer, I want every protection test shown to fail on the mistake it is
for and to pass on my code as it is, so that I am not handed tests that only look like protection.

#### Acceptance Criteria
1. When a task of the stage spec is worked, the stage shall first show the protection test failing
   on the code with the planted fault, and then show it passing on the unchanged code.
2. The stage shall keep with each task the output of the failing run and of the passing run.
3. If a protection test fails on the unchanged code, then the stage shall not count it and shall
   say that the test contradicts the code as it is.
4. If a protection test passes on the code with the planted fault, then the stage shall not count
   it for that fault and shall say that the test does not notice the fault.
5. When a protection test has been added, the stage shall show that the project's own tests still
   pass with it.
6. The stage shall change test files only; if a task cannot be done without changing the code to
   be reduced, then the stage shall stop that task and propose the fault for writing off.
7. The stage shall keep each protection test in a commit that changes no file of the code to be
   reduced.
8. The stage shall work the same way for every language for which a plan can be made.
9. The stage shall say with each protection test which behaviour it fixes, in one sentence a
   person can check against what they want the code to do.

### Requirement 4: Writing a fault off
**Objective:** As the owner of the code, I want to decide which remaining faults no test has to
chase, so that the stage ends without tests that fix nothing worth keeping.

#### Acceptance Criteria
1. Where no protection test is to notice a fault, the stage shall record a proposal to write it
   off in the stage spec, with the fault's place, what was changed, and a reason.
2. The stage shall take one of two kinds of reason: the fault changes nothing a caller can see, or
   the fault cannot be tested as the code stands.
3. The stage shall count a fault as written off only when the owner of the code has approved the
   proposal in the review, and shall record who approved it and when.
4. While a proposal is not approved, the stage shall count its fault as waiting, and shall not
   call its function protected enough.
5. The stage shall go on showing a written-off fault as written off, with its reason, in every
   later result of the stage.
6. If a test notices a fault that was written off, then the stage shall show the fault as noticed.
7. Where a fault is written off because it cannot be tested as the code stands, the stage shall
   say that the reduction of that function has to keep the behaviour around that fault by other
   means, and shall name what the function reaches out to where the plan lists it.

### Requirement 5: Protected enough
**Objective:** As a developer, I want to be told per function when its reduction may begin, on
the strength of a run and not of a report, so that "enough" is not a matter of opinion.

#### Acceptance Criteria
1. When the tasks of the stage spec are done, the stage shall have the planted faults of the chosen
   functions run again against the project's tests with the protection tests, and shall take the
   outcome of that run and not the outcome of the tasks.
2. When the faults are run again, the stage shall run the faults of the chosen functions and no
   others.
3. The stage shall say for each chosen function how many faults were noticed before the stage, how
   many are noticed now, how many are written off, and how many are waiting.
4. When every planted fault of a chosen function is noticed or written off, the stage shall say
   that the function is protected enough and that its reduction may begin.
5. While a fault of a chosen function is waiting, the stage shall say that the function is not
   protected enough, and shall name each waiting fault.
6. If a fault that was noticed before the stage is no longer noticed, then the stage shall say so
   and shall not call the function protected enough.
7. If the code of a chosen function changed during the stage, then the stage shall say that its
   result does not hold for that function.
8. The stage shall say that it advises: a reduction that begins without it is judged as any other.
9. Before the faults are run, the stage shall say how many will be run and about how long that
   takes, from the times of the plan.

### Requirement 6: Finding the stage, and leaving it
**Objective:** As a developer who has just read a plan, I want to be led to the stage and from it
to the reduction, so that the path is one path.

#### Acceptance Criteria
1. Where the run page lists unprotected functions, the run page shall refer to the stage for
   functions the person wants to change.
2. The guide shall describe the path in three parts in this order: see what is protected, protect
   what you will change, make it less.
3. The guide shall say what the stage costs: one run of the project's commands for every planted
   fault that is run again.
4. When a chosen function is protected enough, the stage shall name the next step, which is to
   declare the reduction of that function as a step.
5. When the stage ends, the stage shall leave in the stage spec what was protected, what was
   written off and why, and what is still waiting, so that a later reader finds it without the run.

### Requirement 7: The agent's part and the person's part
**Objective:** As the owner of the code, I want to know what an agent may do in the stage on its
own and what waits for me, so that I can let it work.

#### Acceptance Criteria
1. The stage shall let an agent choose nothing: the chosen functions come from the person.
2. The stage shall let an agent write the stage spec, write protection tests and propose faults for
   writing off.
3. The stage shall wait for the owner of the code at two points: the one review of the stage
   spec, the protection tests and the proposals to write faults off; and the merge of the tests.
4. If a command of the stage ends without a judgement, then the stage shall say that nothing was
   judged and shall not report the function as protected.
5. The stage shall not report a function as protected enough on an agent's reading of the code.
