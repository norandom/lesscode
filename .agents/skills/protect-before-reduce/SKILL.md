---
name: protect-before-reduce
description: Protect the functions a person is about to make smaller, before any of their code changes. From a plan of `lesscode-core step plan` and the functions the person names, write a small spec in the project, write one test per behaviour that a planted fault shows to be unwatched, let `step pin` show each test failing on the fault and passing on the code as it is, propose the faults no test should chase for writing off, and let `step protect result` say by exit code whether the functions are protected enough. Use between a plan and a reduction, when the plan shows functions whose tests notice little. The stage adds tests and changes no code. The tool measures and judges, the agent writes, the owner of the code decides; the skill cannot call a function protected itself.
---

# Protect before you reduce

A reduction is as safe as the tests that would notice a mistake in it. This walk comes between the
plan and the reduction: see what is protected, **protect what you will change**, make it less. It
ends with a branch that holds new tests, a small spec that says what they are for, and a result
from `lesscode-core step protect result`.

The stage was first walked by hand on five functions of a renderer on 24 November 2026: four tests
turned six of nine unnoticed faults into noticed ones, the owner wrote the other three off, and no
line of the code changed. The step commands are described in [docs/steps.md](../../../docs/steps.md)
and in [refactor-in-steps](../refactor-in-steps/SKILL.md).

## Who does what

| Who | Does | Does not |
| --- | --- | --- |
| The tool (`lesscode-core`) | plants faults, runs the project's tests, counts, answers by exit code | write a test or a spec; call a model |
| The agent | runs the commands, writes the stage spec and the tests, proposes write-offs, reports what the commands said | choose the functions; change the code to be reduced; approve anything; call a function protected on its own reading |
| The person | names the functions | |
| The owner of the code | reviews once, approves each write-off by their own hand, merges | |

The person and the owner of the code are often one person. The stage waits for them at three
places: the choice of functions (phase 2), the one review (phase 9) and the merge (phase 11).
Nowhere else do you stop to ask, and nowhere do you go past one of these three.

**The stage advises.** Nothing in lesscode refuses a reduction because the stage was skipped. Say
so when you report; do not present the result as a lock.

## Words

- *Planted fault*: one small mistake made on purpose in a copy of the code, such as a `<` turned
  into a `<=`. *Noticed*: a test failed on it. *Unnoticed*: every test passed although it was there.
- *Chosen function*: a function of the plan that the person wants to reduce.
- *Stage spec*: the directory this walk writes into the project, `<specs-dir>/<name>/`.
- *Written off*: the owner of the code approved a reason why no test is to notice a fault.
- *Waiting*: unnoticed, and not written off.
- *Protected enough*: nothing of the function is waiting.
- A fault's *id* is `<operator>@<line>:<column>` within its file, for example `negate@68:11`. Its
  *number* is the `#N` that `step pin` prints. The id stays the same between runs and goes into the
  spec; the number is what `step pin --fault` takes.

## Exit codes

Every command here answers by exit code: 0 done or accepted, 1 refused or something waits, 2 not
judged, 3 bad input. Gate every next action on the exit code. Exit 2 is never a pass: find out why
(a command that could not start, a build that failed, a timeout), run again, and until then report
that nothing was judged.

## Before you start

- `lesscode-core` on `PATH`, a Git repository with nothing uncommitted, and a run configuration
  `run.json` with the project's own test command and its test paths.
- A work directory outside the repository and not a drive root. Use two directories under it:
  `<work>/plan` for the plan and `<work>/stage` for everything after it.
- Disk. Every copy is a full build of the project. On the renderer eight copies took 34.5 GiB.
  Measure the free space before the plan and say what one copy takes once the plan has built one.
- Where the project keeps its specs. If it has Kiro's directories (`.kiro/specs`, steering), use
  them, the language its other specs are written in, and its own habit of approval. If it has none,
  ask the person for a directory and pass it as `--specs-dir`; install nothing in the project.
- **Where do the tests of the chosen file live?** The stage may add and change files under the
  configured test paths and nothing else.
  - Tests in files of their own (Python, TypeScript, Java, a Rust crate's public interface): nothing
    to prepare.
  - Tests inside the file of the code (Rust unit tests of private items): the stage needs a test
    file of its own and one line in the code file that joins it, for example
    `#[cfg(test)] mod protect;` with an empty `protect.rs` under a path that is a test path of
    `run.json`. That line is a change of a code file. Propose it to the person, with the file and
    the line, and commit it with the empty test file **before the plan**, so that it is part of the
    base. Tests under a crate's own test directory do not reach private modules, so they are no
    way round the joining line. `step protect begin` refuses a configuration whose test paths hold
    no file.

## The walk

| Phase | Who | What | Gate |
| --- | --- | --- | --- |
| 1 | agent | the plan | exit 0 |
| 2 | person | names the functions | the person |
| 3 | agent | `step protect begin` | exit 0 |
| 4 | agent | writes the stage spec | none |
| 5 | agent | runs the base once, after saying how long it takes | exit 0 or 1 |
| 6 | agent | per task: the test, committed alone, then `step pin` per fault | exit 0 per pin |
| 7 | agent | proposes write-offs | none |
| 8 | agent | `step check`, and commits its verdict | exit 0 |
| 9 | owner of the code | the one review; approves write-offs and commits them | the owner |
| 10 | agent | `step protect result` | its exit code |
| 11 | owner of the code | merges the tests | the owner |

### Phase 1: the plan

```sh
lesscode-core step plan --config run.json --work <work>/plan --out plan.json --page
```

Add `--open` only when a person is at the screen and wants to watch; it starts their browser on a
page that follows the run. Without it nothing is opened and nothing is served. A plan of a file of
a compiled language can take a quarter of an hour or more; say so before you start it.

A plan that is incomplete, or whose run judged nothing, is no ground for a stage. Find out why and
plan again.

### Phase 2: the person names the functions

Show the person the plan's table: per function, noticed and unnoticed faults. Say in plain words
what the choice is: "Which of these functions are you going to make smaller? Tests are written for
those and for no others." Do not describe the stage by its gates, and do not ask several open
questions at once; offer choices.

The choice is the person's. Do not choose, extend or trim it. You may say what the plan shows: a
function that is protected already, a function whose faults all lie in code that is not built on
this system, a function that reaches out to a device, a window or a network and will be hard to
test as it stands.

### Phase 3: begin the stage

On the commit the plan was made from, with nothing uncommitted:

```sh
lesscode-core step protect begin --plan plan.json --config run.json \
    --function <file>:<name> [--function <file>:<name> ...] --spec <name> \
    [--specs-dir <dir>] [--branch <branch>]
```

The file is named as the plan names it. The command writes `step.json` (the declaration the other
commands take) and `stage.json` (what is to be protected) into `<specs-dir>/<name>/` and nowhere
else. It prints the chosen functions with their unnoticed faults, the unprotected functions that
are left alone, any chosen function that is left out because none of its code is built here, and
how many faults will be run again and about how long that takes.

Exit 3 says what is wrong: a function the plan does not have (the ones it has are listed), a plan
without a result, a file that changed since the plan (plan again), a stage that was begun there
already, a test path without a file. Do not work round a refusal.

Then create the branch the command named (`protect/<name>` unless you gave another) and stay on it.
Now remove the copies under `<work>/plan`; keep `plan.json` and its page.

### Phase 4: write the stage spec

Write three files beside `step.json` and `stage.json`, from `stage.json` and from reading the
chosen functions. Use the forms in [templates/requirements.md](templates/requirements.md) and
[templates/tasks.md](templates/tasks.md), in the language the project writes its specs in.

- `requirements.md`: at the top the plan, the base commit and the test command, all from
  `stage.json`; what is left alone and left out; then **one requirement per chosen function**. A
  requirement says in words what could change unseen in that function today, and names the faults
  it comes from by id. Write what a caller would see go wrong, not what the operator did to the
  text.
- `tasks.md`: **one task per test**. A test may be for several faults, and should be where one
  behaviour explains them. Each task has the one sentence that says which behaviour the test fixes,
  its faults by id, the boundary "test files only", and where its pin receipts go. One further task
  holds the proposals to write off. Do not plan more tests than faults.
- `spec.json`: the name of the stage spec, the dates, the language, and the approvals, all not
  approved. Where the project has other specs, give this one the same fields as theirs.

The sentence of a task is what the owner of the code checks against what they want the code to do.
No command reads it. Write it so that it can be wrong: "after a fallback the state names the canvas
and the reason given", not "tests the fallback".

Commit the five files by themselves. Nothing is approved by this commit; the review comes once, in
phase 9.

### Phase 5: run the base once

The first `step pin` of a stage runs the faults of the chosen functions on the code as it is. That
is the long run: 17 min 50 s on the renderer, where each later pin took about two minutes. Say
first how long it will take, with the number `step protect begin` printed, then run it as a step
of its own, before any test is written:

```sh
lesscode-core step pin --declaration <specs-dir>/<name>/step.json --fault 1 --work <work>/stage
```

No test exists yet, so the answer is exit 1, "the tests with the pin pass on the fault". That is
expected here and only here. If number 1 is not an unnoticed fault the command ends with 3 and
lists the ones that are; that list serves as well.

Take from the output the list of unnoticed faults with their numbers (`#N line L operator`). Match
each to its id in `stage.json` by line and operator, and note the number beside the id in
`tasks.md`. A fault that did not build keeps its number, so the numbers have gaps; never count
them off yourself. Report how long the run took against what was announced.

### Phase 6: one test per task, shown by `step pin`

For each task of `tasks.md`, in order:

1. Write the test, in a test file only. Run the project's own test command once yourself; the new
   test and all the old ones have to pass on the code as it is.
2. Commit the test in a commit that touches no file of the code.
3. For each fault the task names:

   ```sh
   lesscode-core step pin --declaration <specs-dir>/<name>/step.json --fault <n> \
       --work <work>/stage --out <specs-dir>/<name>/pins/pin-<n>.json
   ```

4. Keep the receipt and what the command printed (`pins/pin-<n>.log`) as the evidence of the task,
   and mark the task done only when every fault it names was admitted.

What the answers mean:

- Exit 0, admitted: the tests pass on the unchanged code and fail on the fault. The test is shown
  to notice this mistake.
- Exit 1, "the tests with the pin pass on the fault": the test does not notice it. Change the test.
- Exit 1, "the tests with the pin fail on the unchanged code": the test contradicts the code as it
  is. The code is what is to be kept; change the test. This answer costs one more run than the
  other, because a copy tries its first run a second time.
- Exit 2: not judged. Not a pass.

**Change the test, never the code.** If a fault can only be reached by changing the code to be
reduced (making something public, adding a seam, passing in a fake where none can be passed), stop
that task. Take the fault out of it, say in the task why, and propose the fault for writing off in
phase 7 as not testable as it stands. Do the same after honest tries that did not get a test
admitted. Leaving a fault is a normal end of a task; a test that only looks like protection is not.

### Phase 7: propose write-offs

For every fault of a chosen function that no admitted test notices, write an entry into
`<specs-dir>/<name>/write-offs.json`:

```json
{ "schema": 1, "base": "<the base commit of stage.json>",
  "write_offs": [
    { "file": "src/a.rs", "function": "decide", "fault": "negate@68:11",
      "kind": "not_testable_as_it_stands",
      "reason": "The condition is fixed at build time; on this system the other branch is not built.",
      "proposed_by": "agent", "approved_by": null, "approved_at": null } ] }
```

`kind` is one of two, and no third exists:

- `nothing_a_caller_sees`: the fault changes nothing that any caller can observe.
- `not_testable_as_it_stands`: a test cannot reach it without a change of the code. The result will
  repeat what the function reaches out to and say that the reduction has to keep the behaviour
  there by other means; cutting such code so that it can be tested is the work of
  [untangle-state](../untangle-state/SKILL.md).

The reason is one or two sentences a person can check in the code. Where two faults of a function
share line and operator, propose both or neither: the result cannot tell them apart and treats
them together. An entry for a fault the stage does not have, or with another kind, is bad input.

**`approved_by` and `approved_at` stay `null`.** You never fill them: not from an answer in the
conversation, not "as instructed", not marked as filled in on someone's behalf. Only the owner of
the code writes them. Commit the proposals.

### Phase 8: the check

With everything committed:

```sh
lesscode-core step check --declaration <specs-dir>/<name>/step.json --work <work>/stage --page
```

It runs the faults of the chosen functions again, and no others, with the new tests in place. This
is the judgement of the stage; the pins were the evidence of the tasks. Exit 0: accepted. Exit 1:
read the reasons. A stage that changed a code file, or a test committed together with code, is a
stage done wrong; put that right and do not touch the declaration's limits. Exit 2: not judged.

Commit the verdict the command wrote, and the pin receipts if they are not committed yet.
`step protect result` reads the verdict, `stage.json` and `write-offs.json` **as they are committed
on the branch and no other copy**. A verdict that lies in the working tree does not exist for it.
A commit of a test file after the check makes the verdict one for other content: run the check
again.

Then remove the copies under `<work>/stage`. Keep its `outcomes` directory and the `.json`
receipts; they are small and make a repeated run short.

### Phase 9: the one review

Stop here and hand the owner of the code one thing to read, in this order:

1. what was chosen and what was left alone;
2. each test with its sentence, the faults it was admitted for, and the file it is in;
3. each proposal to write off, with its kind and reason, and for `not_testable_as_it_stands` what
   the function reaches out to;
4. what stays unwatched if all proposals are approved, in plain words ("nothing checks that the
   sessions are moved to the canvas");
5. the output of `step protect result` as it stands now, which is exit 1 while proposals wait.

The owner approves a write-off by writing their name into `approved_by` and the date into
`approved_at` of that entry, and **committing the file on the branch**. An approval that is written
and not committed does not count. The owner may instead ask for a test, change a sentence, or
refuse a proposal; then go back to phase 6 or 7, and through phase 8 again where a test file
changed.

Say what you could not judge yourself: whether each sentence names a behaviour the owner wants
kept, and whether the tests you wrote are the ones you would have accepted from someone else. You
wrote them and you judged which faults a test can reach.

### Phase 10: the result

```sh
lesscode-core step protect result --declaration <specs-dir>/<name>/step.json
```

| Exit | Meaning | What you do |
| --- | --- | --- |
| 0 | every chosen function is protected enough | report the output; it names the next step |
| 1 | a fault waits, or the check refused because unnoticed faults rose; each is named | report the output; back to phase 6, 7 or 9 |
| 2 | not judged: no committed verdict, a verdict for other content, a code file the branch changed, a check that did not judge | report that nothing was judged, and why |
| 3 | bad input | report it and put the input right |

Per chosen function the output gives four numbers: noticed before, noticed now, written off,
waiting. It names every write-off with its reason, approver and date. **Report that output and
nothing of your own reading.** Do not round it, do not add a function it does not list, and do not
say "protected" for exit 1 or 2.

On exit 0 and 1 the command writes `protection.json` beside the declaration. Fill the result
section of `tasks.md` from the output, and commit both. `protection.json` is what a later reader
finds without the run.

On exit 0 the next step is the reduction of a chosen function, declared as a step of its own
([refactor-in-steps](../refactor-in-steps/SKILL.md)). It is not part of this walk and not on this
branch.

### Phase 11: the merge

Merging the tests is the owner's. Before it, remove what is left of the copies and say what is
kept and how large it is. The merge is the last place where an approval that was not the owner's
would be seen: the result prints every approver and date for that reason.

## In a pipeline

Every command of the walk takes all it needs from arguments and files, opens no page, starts no
listener, asks nothing and answers by exit code. Leave `--open` away; `--page` only writes a file
and may stay. The dashboard is off unless someone asks for it.

- Phases 1, 3, 5, 6, 8 and 10 run unattended. The names of the functions for phase 3 are arguments
  that a person decided beforehand; a pipeline does not pick them.
- `step protect result` is the job that gates: exit 0 passes, exit 1 means a fault waits, exit 2
  means nothing was judged and must not be read as a pass.
- The result reads committed files only, so a pipeline and a person at a checkout get the same
  answer for the same commit. An approval reaches a pipeline by the owner's commit and by no other
  way.
- A pipeline cannot do phases 2, 9 and 11. A run without the owner ends at exit 1 with the
  proposals named as waiting; that is the correct end, not a failure to be worked round.

## What no command checks

Say these to the owner of the code; do not fill them with your own judgement:

- That a person wrote `approved_by`. The tool sees a name and a date.
- That the sentence of a test names a behaviour worth keeping. A test is admitted when it fails on
  the fault; it can still fix a behaviour nobody wants.
- That a reason for a write-off is true.
- That the reduction will behave the same. Planted faults measure how much the tests notice, not
  equivalence.
- Anything about a function that was left alone or left out.
- A test that depends on the machine (a GPU, an environment variable, a clock) is admitted on the
  machine it ran on. Say what it depends on.
