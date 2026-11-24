# Implementation Plan: the brownfield test stage

Generated 2026-11-24, 13:25 (Europe/Berlin). **Approved by the owner on 2026-11-24** ("ok").
Basis: [requirements](requirements.md) (approved 2026-11-24, amended the same day to one review),
[design](design.md) (approved by the owner on 2026-11-24: "approved, push and write the tasks"),
[prototype gate](prototype-gate.md) (`proceed`, [R184](../../../docs/research/2026-11-24/05-brownfield-stage-prototype.md)).
Task-graph sanity review: inline (no subagent, by the owner's rule), verdict PASS, with one note:
task 2.1 needs to know how `step guard` matches a verdict to the committed content, and reuses it.

Rules for every task group (owner rules of 2026-11-07, 2026-11-10 and 2026-11-17):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format
  check and lints before every push. Rust only.
- The new code runs no fault. `step plan`, `step pin`, `step check`, the declaration and the run
  page keep their behaviour, exit codes and receipts; every group ends with the existing suites
  passing unchanged, on Windows and through Dagger on Linux.
- Owner's direction of 2026-11-24, after the approval ("the web dash is off by default and
  existing flows can be automated on ci cd"): the two new commands open no page, start no listener
  and ask nothing; everything they need is in arguments and files, and what they found is in the
  exit code. `--page` and `--open` of `step plan` and `step check` stay options that are off unless
  given. The skill and the guide say how the stage runs in a pipeline, without `--open`.
- Every new file gets its entry in `native/complexity-budgets.json` at the default ceilings. A file
  over budget is split, not exempted. `runner.rs` and `page_log.rs` are not touched.
- Each group ends with a dated research entry where a result or a deviation exists, and
  `.kiro/task_N_handover.md`.
- Storage: measure D: before a group; remove working copies when a run has ended.
- Tasks marked "can fail" carry something the prototype did not show. A failure is recorded and
  goes back to the design before the group is closed.
- What is reported about a task is what ran and was seen. Times come from the clock.

## Tasks

- [ ] 1. From a plan to the start of a stage: `step protect begin`
- [x] 1.1 The command, and what it refuses
  - `native/src/step/protect.rs` (arguments of `protect begin` and `protect result`, dispatch) and
    `protect_begin.rs`; the subcommand in `step/mod.rs`. Reads a plan, its run receipt and the run
    configuration.
  - Refuses with exit 3 and a sentence that names what is wrong: a named function the plan does
    not have (the plan's functions are listed); a plan that is incomplete or whose run was void; a
    file of a chosen function whose content is not the one the run receipt has a hash for; a stage
    spec directory that already holds a `stage.json`; a test path of the configuration that holds
    no file.
  - A chosen function whose faults all lie in code not built here is left out and named.
  - Done when tests on the toy repository show each refusal with its text, and that a refusal
    writes nothing.
  - _Boundary: protect.rs, protect_begin.rs_
  - _Requirements: 1.3, 1.4, 1.5, 1.6_
- [x] 1.2 The declaration and `stage.json`
  - Writes, into `<specs-dir>/<name>/` only: `step.json` (scope the files of the chosen functions,
    `focus` the chosen functions, `max_token_growth` 0, base the commit at `HEAD`, branch as named
    or `protect/<name>`, commands, test paths and run settings of the configuration) and
    `stage.json` (`step-stage-v1`: plan and its hash, base, test command, per chosen function its
    counts, what it reaches out to and each unnoticed fault with id, place, operator and text; the
    unprotected functions left alone; the ones left out).
  - Done when a test shows that `step check` accepts the written declaration on a branch without
    changes, that the repository holds nothing new outside the stage spec's directory, and that
    `stage.json` of a toy plan matches a written expectation.
  - _Depends: 1.1_
  - _Boundary: protect_begin.rs_
  - _Requirements: 1.1, 1.2, 1.7, 2.1, 2.4, 2.8, 3.8, 5.2_
- [x] 1.3 What it says before anything runs
  - Prints the chosen functions with their unnoticed faults, the unprotected functions that are
    left alone and that nothing has to be done about them, and the cost: how many faults the base
    run and the check will run, about how long each takes from the seconds of the plan, and that
    the base run comes first.
  - Done when a test reads those lines for a toy plan, and the numbers are the plan's.
  - _Depends: 1.2_
  - _Boundary: protect_begin.rs_
  - _Requirements: 1.1, 1.2, 5.9_

- [ ] 2. Protected enough: `step protect result`
- [x] 2.1 The verdict it may use
  - `native/src/step/protect_result.rs`. Takes the verdict only when it is the one for the
    committed content, by the hashes `step guard` compares; otherwise exit 2 and nothing written.
  - If a scope file differs between the base and the branch: exit 2, and the sentence that the
    result does not hold for the functions of that file.
  - Done when tests show exit 2 for a missing verdict, a verdict for other content, and a branch
    that changed a code file, each with its sentence and without a `protection.json`.
  - _Boundary: protect_result.rs_
  - _Requirements: 3.6, 5.1, 5.7, 7.4_
- [x] 2.2 The four numbers, and the exit code
  - Per chosen function, from the verdict and `write-offs.json`: noticed before, noticed now,
    written off, waiting. A write-off counts with one of the two kinds, a reason, an approver and a
    date; without approval its fault is waiting; a kind that is not one of the two is bad input
    (exit 3); a written-off fault that is noticed is shown as noticed. Exit 0 when nothing waits in
    any chosen function, 1 otherwise with every waiting fault named. A verdict that refused because
    unnoticed faults rose gives exit 1 and says so.
  - Done when tests on the toy repository show: every fault noticed (0); one left without a
    proposal (1, named); the same with an unapproved proposal (1, waiting); with an approved one
    (0, shown as written off with its reason); a wrong kind (3); a written-off fault that a test
    notices (shown as noticed).
  - _Depends: 2.1_
  - _Boundary: protect_result.rs_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.6, 5.3, 5.4, 5.5, 5.6, 7.5_
- [x] 2.3 What it says and what it leaves
  - The output names every write-off with its reason, approver and date; for one that cannot be
    tested as the code stands it repeats what the function reaches out to and that a reduction has
    to keep the behaviour there by other means. It says that the stage advises. On exit 0 it names
    the next step: declare the reduction of the function as a step. Writes `protection.json`
    (`step-protection-v1`) with the four numbers, the write-offs, the waiting faults, base,
    revision and the hash of the verdict.
  - Done when a test reads those sentences and the receipt for a toy stage with one fault of each
    standing.
  - _Depends: 2.2_
  - _Boundary: protect_result.rs_
  - _Requirements: 4.5, 4.7, 5.8, 6.4, 6.5_

- [ ] 3. The skill `protect-before-reduce`
- [x] 3.1 The walk
  - `.agents/skills/protect-before-reduce/SKILL.md`, in lesscode's own words (no text of Kiro's
    skills): the eleven phases of the design with their gates. It says: the person names the
    functions; the base runs once first and how long; where tests live inside the file of the code
    (Rust), a test file of its own and one joining line are proposed to the person and committed
    before the plan; a test is committed alone and shown by `step pin` to fail on the fault and
    pass on the unchanged code; a task that would need a change of the code stops and becomes a
    proposal; one review by the owner of the code, who alone fills the two approval fields; the
    project's own Kiro setup is used where there is one and nothing is installed where there is
    none; working copies are removed as the stage goes.
  - Done when the skill is read against the record of the prototype (R184) and every step taken
    there by hand has its line, and the owner has read it.
  - _Boundary: the skill_
  - _Requirements: 2.5, 2.6, 2.7, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 7.1, 7.2, 7.3_
- [ ] 3.2 The form of the stage spec
  - Two templates in the skill: `requirements.md` (the plan, the base and the test command at the
    top; what is left alone; one requirement per chosen function that says what could change
    unseen there, with its faults by id) and `tasks.md` (one task per test with the sentence of
    what it fixes, its faults by id, the boundary "test files only", the place of the pin
    receipts; one task for the proposals; a result section).
  - Done when the stage spec of the prototype (`protect-display-path` in the clone) can be written
    from the templates without leaving a part of it out.
  - _Depends: 3.1_
  - _Boundary: the skill's templates_
  - _Requirements: 2.2, 2.3, 2.4, 3.9_

- [ ] 4. The path in the guide
  - `docs/steps.md`: the two commands with their exit codes and files, and the path in three
    parts, see what is protected, protect what you will change, make it less; what the stage
    costs. `docs/state-refactoring.md`: where the stage comes in the walk. One paragraph each in
    the skills `untangle-state` and `refactor-in-steps`. The run page already refers to the stage.
    An entry in the release notes. Humanizer pass before a push.
  - Done when the owner has read the section.
  - _Depends: 1.3, 2.3, 3.1_
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 5. Closing
- [ ] 5.1 The stage with its commands on another file of the renderer (can fail)
  - The whole skill, with `step protect begin` and `step protect result` in place of what was done
    by hand in R184, on a renderer file the owner names (proposed: `yuv.rs`: `convert` 8 noticed
    and 6 unnoticed, `new` 4 and 5, `uniform_bytes` 4 and 1). In an isolated clone; the owner
    watches the first run on the page and does the one review.
  - Done when the research entry gives the four numbers per function from `protection.json`, the
    times against what `step protect begin` announced, and what the owner said.
  - _Depends: 1.3, 2.3, 3.2_
  - _Requirements: 1.1, 2.5, 3.1, 4.3, 5.3, 5.4, 5.9, 7.3_
- [ ] 5.2 The stage in a project that is not Rust (can fail)
  - The same on one Python or TypeScript project from `D:\Source` that the owner names, where
    tests live in files of their own and no joining line is needed.
  - Done when the research entry says whether anything of the stage had to be done differently
    for that language.
  - _Depends: 5.1_
  - _Requirements: 3.8_
- [ ] 5.3 Validation and handover
  - The suites on Windows and through Dagger on Linux; budgets; lints; `kiro-cli code check`; a
    validation report; the handover.
  - Done when the validation report says GO or names what stands against it.
  - _Depends: 5.2_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5_
