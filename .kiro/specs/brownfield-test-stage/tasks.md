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

- [x] 1. From a plan to the start of a stage: `step protect begin`
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

- [x] 2. Protected enough: `step protect result`
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

- [x] 3. The skill `protect-before-reduce`
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
- [x] 3.2 The form of the stage spec
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

- [x] 4. The path in the guide
  - `docs/steps.md`: the two commands with their exit codes and files, and the path in three
    parts, see what is protected, protect what you will change, make it less; what the stage
    costs. `docs/state-refactoring.md`: where the stage comes in the walk. One paragraph each in
    the skills `untangle-state` and `refactor-in-steps`. The run page already refers to the stage.
    An entry in the release notes. Humanizer pass before a push.
  - Done when the owner has read the section.
  - Done 2026-11-24 (`b16826e`); the owner read it: "read it, ok".
  - _Depends: 1.3, 2.3, 3.1_
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 5. Closing
- [x] 5.1 The stage with its commands on another file of the renderer (can fail)
  - The whole skill, with `step protect begin` and `step protect result` in place of what was done
    by hand in R184, on a renderer file the owner names (proposed: `yuv.rs`: `convert` 8 noticed
    and 6 unnoticed, `new` 4 and 5, `uniform_bytes` 4 and 1). In an isolated clone; the owner
    watches the first run on the page and does the one review.
  - Done when the research entry gives the four numbers per function from `protection.json`, the
    times against what `step protect begin` announced, and what the owner said.
  - Done 2026-11-24 ([R188](../../../docs/research/2026-11-24/09-protect-stage-yuv.md)): `step
    protect result` exit 0 after the owner approved eleven label write-offs; eight of ten
    expectations hold. E8 (1 of 12 noticed) and E9 (first pin 11 min 21 s) fail.
  - _Depends: 1.3, 2.3, 3.2_
  - _Requirements: 1.1, 2.5, 3.1, 4.3, 5.3, 5.4, 5.9, 7.3_
- [x] 5.2 The stage in a project that is not Rust (can fail)
  - The same on one Python or TypeScript project from `D:\Source` that the owner names, where
    tests live in files of their own and no joining line is needed.
  - Done when the research entry says whether anything of the stage had to be done differently
    for that language.
  - Done 2026-11-25 ([R190](../../../docs/research/2026-11-25/01-protect-stage-python.md)), on
    scitools-hook's `analysis/structure/cycles.py`, chosen by the owner. Nothing of the stage
    differed for Python; the project's test command runs in a Linux container. One test was
    written and one fault written off, and `step protect result` exited 0.
  - _Depends: 5.1, 6.4_
  - _Requirements: 3.8_
- [x] 5.3 Validation and handover
  - The suites on Windows and through Dagger on Linux; budgets; lints; `kiro-cli code check`; a
    validation report; the handover.
  - Done when the validation report says GO or names what stands against it.
  - Done 2026-11-25: [validation.md](validation.md), GO, with seven points named that stand against it; [task_88_handover.md](../../task_88_handover.md).
  - _Depends: 5.2, 6.4_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 6. The whole scope by default (amendment of 2026-11-24, 18:35; approved by the owner the same day)

  The owner chose the order: the code first, then a full run, then 5.2 and 5.3.
- [x] 6.1 `--function` optional, and reported functions
  - `step protect begin` without `--function` chooses every function of the plan with an unnoticed
    fault, less `left_out`. With nothing unnoticed it writes nothing and exits 0. `left_alone`
    becomes `reported` with the unnoticed count. `step protect result` lists the reported functions,
    writes them into `protection.json`, and leaves them out of the exit code. `begin` prints `/`.
  - Done when tests on the toy repository show: the whole scope without `--function`; a narrowed
    scope with reported functions in `stage.json`, in the output and in `protection.json`; exit 0
    with nothing written when nothing is unnoticed; the earlier refusals unchanged.
  - Done 2026-11-24. Test first: six tests red, then green. The whole native suite passed (504,
    nextest), as did the doctests, `fmt --check`, Clippy with `-D warnings` and the budgets. To stay
    under its token ceiling, `protect_begin.rs` gave the two plan readers to `protect_stage.rs`.
    `result` also prints `/`. A `stage.json` from before keeps working: `left_alone` is read past.
    On the real plan of `yuv.rs` (frozen inputs of R188), `begin` without `--function` chose
    `convert`, `new` and `uniform_bytes` with the same 28 faults as the owner's choice; the output
    was removed.
  - _Boundary: protect_begin.rs, protect_stage.rs, protect_write.rs, protect_says.rs,
    protect_numbers.rs, protect_receipt.rs_
  - _Requirements: 1.1, 1.2, 5.10, 7.1_
- [x] 6.2 No empty-string fault in a Rust `label` field (pack of `agnostic-annotation-driver`)
  - `skip_string_in_fields` and the field initializer's kind in the pack's `call_argument`;
    `argument_edits` skips `string->empty` there; the Rust pack sets `["label"]`.
  - Done 2026-11-24, with one difference named:
    - **Built as** `skip_strings_in: {"kind", "names"}`, without a second field name: a child of the
      node whose text is a name names the field.
    - **Where the code sits.** The check is `ArgumentOps::empty_for` in `pack.rs`. `operators.rs`
      calls it once, and two `_, _` patterns became `..`, so it stays at its ceiling (3,022 of
      3,024 tokens).
    - **Pack size.** The quiet kind is checked against the grammar at admission. The Rust pack is
      3,173 of 3,200 characters; its operator lines were indented less to make room.
    - **Checks.** The test was red first. Then the whole native suite passed (505), as did the
      doctests, `fmt`, Clippy with `-D warnings` and the budgets.
    - **The plan of `yuv.rs`.** Run on `stage-base/yuv` (`4fff14f`) with a frozen release binary of
      this code (`stage-6\bin`, SHA-256 `0B3E529E…A903`), 18:55:35 to 19:09:37, exit 0. Copy 2
      failed its first run once and got through on the second.
    - **Result: 23 faults, not 19.** 18 noticed as before, 5 unnoticed: `drop_statement@114:8`
      (NV12) kept, and the five label faults of `new` and two of `convert` gone. Four remain in
      `convert` (`@296:12`, `@305:20`, `@312:20`, `@322:20`). They are labels too, but passed as
      the first argument of the local helper `plane("yuv-y", …)`, which puts them into `label:`
      in its own body. A rule that reads the syntax only does not see that.
    - **Where the evidence is.** `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6\`:
      `plan-yuv.{json,log,times}` and the run receipt; the copies are removed.
  - Done when a driver test shows `label: Some("x")` unplanted and `name: Some("x")` and
    `f("x")` still planted. A plan of `yuv.rs` in the clone has no `string->empty` on a `label`
    field and keeps the twelfth unnoticed fault. The expected count is 19 where it was 30; any other
    difference is named.
  - _Boundary: native/src/adapters/driver/operators.rs, pack.rs, native/packs/rust.json_
  - _Requirements: amendment of 18:35, the second case_
- [x] 6.3 The skill, the guide and the release notes
  - Phase 2 of `protect-before-reduce` and `docs/steps.md` with the whole scope by default and the
    reported functions; the five findings of 5.1; an entry in `docs/releases/2.2.0.md`. Humanizer
    pass.
  - Done when the owner has read the changed sections.
  - Done 2026-11-24 (`33a6273`). The owner read a summary of the changes, section by section, in
    the conversation: "read it, ok". Also in that commit, from design point 5: `begin` says how
    long to allow (1.4 times its figure) and what twice the plan's copies take on disk, with a
    test. The suite passed (505), as did the doctests, `fmt`, Clippy and the budgets. The owner
    decided the same day to accept the four labels passed through a helper as write-offs and not
    to extend the rule.
  - _Depends: 6.1, 6.2_
  - _Requirements: 1.1, 1.2, 6.2, 6.3, 7.1_
- [x] 6.4 A full run where the stage gets tests written, on another file of drift-rdp (can fail)
  - I propose a file with the numbers of its plan (several unnoticed faults a caller can see), and
    the owner approves it before the run. Then the stage runs with the whole scope by default, from
    the plan to `step protect result`, built from the code of 6.1 and 6.2 and frozen.
  - Done when a preregistered research entry gives the tests written, the four numbers per
    function, the reported functions, the times against `begin`'s figure, and what the owner said.
  - The owner chose `shared_frame.rs` and approved its joining line (clone `a8f3442`) on
    2026-11-24. Preregistered as [R189](../../../docs/research/2026-11-24/10-protect-stage-shared-frame.md).
  - Run 2026-11-24 up to the owner's review. Three tests were written and admitted, and four
    unnoticed faults ended noticed. The result is exit 1: three proposals wait for approval. The
    fourth fault, #6, is one the plan wrongly counted noticed, so it is not in `stage.json` and
    cannot be written off. The owner decided to fix that in the tool the next day (the base run
    sets the stage's unnoticed faults), together with two gaps of the skill: `--page` in phase 8,
    and removing the pins' copies before the check. Open until then.
  - Done 2026-11-25, after 6.5. The second check was accepted, the owner approved the write-off of
    #6, and `step protect result` exited 0 (clone `cd9e2cd`). Three tests were written and four
    faults written off. Eight of eleven expectations of R189 hold.
  - _Depends: 6.1, 6.2, 6.3_
  - _Requirements: 1.1, 3.1, 5.3, 5.4, 5.10_
- [x] 6.5 The base run of the check sets the stage's unnoticed faults (amendment of 2026-11-25)
  - `step check` writes `unnoticed_before` per function. `step protect result` takes into the
    stage each fault unnoticed before and after that `stage.json` lacks, accepts a write-off for
    it by id, and names it (`from_the_base_run` in `protection.json`). In the skill, phase 8
    gives `--page` a place, and the pins' copies are removed before the check. `begin`'s disk
    line says why.
  - Done when tests show:
    - the verdict's `unnoticed_before`;
    - a fault missed before and after but absent from `stage.json` waits, can be written off by
      its id, and ends in exit 0 once approved;
    - a fault missed only after does not join the stage;
    - a verdict without the field behaves as before.
  - Then R189's stage is finished with a new check and result.
  - Done 2026-11-25. Test first: three tests red, then green.
    - Unit tests of the numbers: a fault joins by id, a fault missed only after does not join,
      and a verdict without the field adds nothing.
    - One test on the toy repository: `stage.json` without its fault, the check, exit 1 with
      "From the base run", a write-off by `drop_statement@9`, then exit 0 with
      `from_the_base_run` in `protection.json`.
    - `begin`'s disk line says to remove the pins' copies before the check.
    - The suite passed (508), as did the doctests, `fmt`, Clippy and the budgets.
    - The skill (phases 1 and 8: `--page <place>`, the pins' copies, the miscounted fault),
      `docs/steps.md` and the 2.2.0 notes are updated.
  - _Depends: 6.4_
  - _Boundary: verdict.rs, check_runs.rs, protect_numbers.rs, protect_result.rs,
    protect_receipt.rs, protect_says.rs; the skill and docs/steps.md_
  - _Requirements: 4.1, 4.3, 5.3, 5.5_

- [x] 7. No swap of keyword arguments, and a time figure that counts the tests (amendment of 2026-11-25, afternoon; approved by the owner the same day)

  Points 2 and 3 of handover 88. The owner kept `38d337e` of `.dagger/main.go` as it is.
- [x] 7.1 No swap of keyword arguments in the Python pack (pack of `agnostic-annotation-driver`)
  - `named_kinds` in the pack's `call_argument` operators; the first two arguments are not swapped
    when the first is of a named kind; admission checks the kinds against the grammar; the Python
    pack names `keyword_argument`.
  - Done when tests show:
    - `Finding(kind=k, rule=r)` gets no `arg_swap`, and `send(name, False)` still does;
    - a pack whose `named_kinds` names a kind the grammar lacks is refused.
  - Also: the faults of R190's `cycles.py` at its base, made with the new pack and not run, lack
    `arg_swap@138:8` and keep `arg_swap@145:25`. Any other difference is named.
  - Done 2026-11-25. Test first: both tests red, then green.
    - **Where the code sits.** The check is `ArgumentOps::swappable` in `pack.rs`; `operators.rs`
      asks it for the arguments it may swap, which leaves that file at 3,020 of 3,024 tokens.
    - **The faults of `cycles.py`.** `deconvolute mutate` at `73a736e` with this code made 21
      faults where R190's plan had 22. The one missing is `arg_swap@138:8`. `arg_swap@145:25`
      and the other three swaps are still there, and nothing else differs. Receipt and suite log:
      `D:\BuildArtifacts\lesscode\brownfield-test-stage\group-7\`.
    - **Found on the way, not changed.** The grammar also parses a swap that puts a keyword
      argument before a positional one (`log(level=2, text)`), which Python refuses to compile.
      Such a fault is planted and counted as noticed. The test records that, and the owner
      decides whether the rule should cover it.
    - **Checks.** The suite passed (510) once. In the two runs before it,
      `the_installed_hook_stops_a_push_without_a_matching_verdict` failed, as handovers 52, 75,
      76 and 87 record for runs under load. It passed alone and in the third run. The doctests,
      `fmt`, Clippy with `-D warnings` and the budgets passed.
  - _Boundary: native/src/adapters/driver/pack.rs, operators.rs, admission.rs, native/packs/python.json_
  - _Requirements: amendment of the afternoon of 2026-11-25, point 1_
- [x] 7.2 `begin`'s time figure counts the tests as a run makes them
  - The base run's figure is `3T + S/n + (S - S/n)/c`, or `2T + S` with one copy. The check's
    figure is that plus `3T`. Both are printed, each with the room of 1.4.
  - Done when the test of what `begin` says names both figures. On the frozen plans of R188,
    R189 and R190, `begin` prints the figures that R191 computed. R191 gives the measured runs
    against the old and the new figures.
  - Done 2026-11-25. Test first: the test of what `begin` says was red on the new wording, then
    green. It now also computes both figures from the toy plan's run and finds them in the output.
    - **The frozen plans.** `begin` was run on the plans of R188, R189 and R190, at the bases those
      stages began from, and exited 0 each time. Its figures differ from R191's table by 0 to 3 s,
      because R191 used the rounded numbers `begin` had printed. R191's "After the code" gives them.
      The base runs took 0.91 to 1.00 of the new figure and the checks 0.45 to 1.05.
    - **Clones.** What `begin` wrote was removed, and both clones are back on their branches,
      clean.
    - **Checks.** The suite passed (510), as did the doctests, `fmt`, Clippy with `-D warnings`
      and the budgets (`protect_says.rs` 1,471 of 2,800 tokens).
    - **After the check of 7.4.** `kiro-cli code check` failed on its ratchet: the test of what
      `begin` says had grown from 56 to 81 code lines, over the limit of 60. The figures moved into
      a test of their own, `begin_gives_the_base_run_and_the_check_their_own_figures` (`719ee6b`).
      The suite then passed (511).
  - _Boundary: native/src/step/protect_says.rs, native/tests/step_protect.rs_
  - _Requirements: 5.9; amendment of the afternoon of 2026-11-25, point 2_
- [x] 7.3 The guides and the release notes
  - `docs/driver.md` (`named_kinds`), `docs/steps.md` and the skill (the two figures),
    `docs/releases/2.2.0.md` (both changes, and its known limit that the stage was walked in Rust
    only, which R190 ended). Humanizer pass.
  - Done when the owner has read the changed sections.
  - Done 2026-11-25 (`add72d0`), humanizer pass done. The owner was shown a summary of each changed
    section and answered "Accept as written".
  - _Depends: 7.1, 7.2_
- [x] 7.5 No swap that puts a keyword argument first (owner decision of 2026-11-25, after 7.3)
  - Found in 7.1: the grammar parses `log(level=2, text)`, which Python refuses to compile, so
    the swap of a positional argument with a keyword argument after it is planted and counted as
    noticed. The owner chose "Extend the rule now": the first two arguments are not swapped when
    either of them is of a named kind.
  - Done when the test of 7.1 shows no swap for `log(text, level=2)` and still one for
    `send(name, False)`. The faults of `cycles.py` at `73a736e` are made again, and any change
    against the 21 of 7.1 is named. `docs/driver.md` and the 2.2.0 notes say so, and that known
    limit is removed.
  - Done 2026-11-25. Test first: red on `log(level=2, text)`, then green. `swappable` now looks
    at both of the first two arguments. The first wording of its comment named Python, and
    `the_engine_names_no_language_or_project` refused it. The comment no longer names a language.
    - **The faults of `cycles.py`.** Still 21, the same as in 7.1: none of its swaps has a keyword
      argument second.
    - **Checks.** The suite passed (511), as did the doctests, `fmt`, Clippy with `-D warnings`
      and the budgets. `kiro-cli code check --changed ada2bb6^` passed, with its 3 pre-existing
      errors.
  - _Boundary: native/src/adapters/driver/pack.rs, native/tests/driver_contract.rs, docs/driver.md,
    docs/releases/2.2.0.md_
  - _Depends: 7.1_
- [x] 7.4 Validation and handover
  - The suites on Windows and through Dagger on Linux; budgets; lints; format; doctests; an addition
    to the validation report; `task_89_handover.md`.
  - Done 2026-11-25: the [validation](validation.md) addition says GO;
    [task_89_handover.md](../../task_89_handover.md).
  - _Depends: 7.1, 7.2, 7.3, 7.5_
