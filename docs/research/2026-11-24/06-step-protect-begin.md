# R185: `step protect begin` is built; on the toy repository it refuses what it should and `step check` reads what it writes

- Report date/time and timezone: 2026-11-24, 14:00 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, about 13:30 to 13:56 (the commits of tasks 1.1, 1.2 and 1.3 at 13:41, 13:51 and 13:56); the run on Linux after that, see below
- Status: completed for group 1 of the tasks; nothing was run on a real project
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), tasks 1.1 to 1.3; requirements 1.1 to 1.7, 2.1, 2.4, 2.8, 5.2, 5.9
- Hypothesis ID and predecessor: [R184](05-brownfield-stage-prototype.md) (the stage by hand)
- Type: implementation record, with two findings about the design

## Decision

Group 2 (`step protect result`) can begin. The first command of the stage exists and is shown on
the toy repository only: every refusal with its sentence and with nothing written, the two files it
writes, and `step check` accepting the written declaration on a branch without changes. Whether it
holds on the renderer of drift-rdp is task 5.1 and is not shown here.

Before task 2.1 one point of the design has to be settled, because the design describes the
verdict inexactly (finding 1).

## What was built

| Commit | Task | What |
| --- | --- | --- |
| `43ec698` | 1.1 | `step protect begin` reads a plan, the receipt of its run, the run configuration and the code, and refuses with exit 3 before anything is written |
| `016d8d0` | 1.2 | it writes `step.json` and `stage.json` into the stage spec's directory and nowhere else |
| `600ca46` | 1.3 | it says what is chosen, what is left alone, what is left out, and what running the faults again costs |

The owner's direction of the same day ("the web dash is off by default and existing flows can be
automated on ci cd") is written into the rules of the tasks. Checked against the code: `--page` and
`--open` of `step plan` and `step check` are options that do nothing unless given
(`native/tests/run_page_options.rs`, the test
`without_either_option_nothing_is_written_or_opened_and_the_plan_is_the_same`). The new command has
no such option, starts no listener, reads nothing from standard input and answers by its exit code;
its tests run it with standard input empty and closed.

## Evidence

- Tests: `native/tests/step_protect.rs`, nine tests through the binary. Each makes a real plan of
  the toy repository with `step plan` first, so a change of the plan's or the receipt's shape
  breaks them.
- Windows at `600ca46`: core suite 487 of 487, 1 skipped; format check, lints with warnings
  denied, doctests, size budgets
  (`D:\BuildArtifacts\lesscode\brownfield-stage\budgets-1.3.json`), `kiro-cli code check` with no
  error.
- Linux through Dagger at `600ca46` (`dagger call evidence`, 348 s): overall 0, every step 0; core
  suite 486 of 486, 1 skipped, with the nine tests of `step_protect` among them. The function left
  out there is `only_windows`, on Windows `only_linux`. Evidence:
  `D:\BuildArtifacts\lesscode\brownfield-stage\linux-g1` (0.4 MiB).
- The fault id on a real receipt: the plan of the renderer of drift-rdp
  (`D:\BuildArtifacts\lesscode\run-dashboard\real-run-8.1\work\plan-run.json`, 268 faults) gives
  268 different ids with the rule below.

## Findings

1. **The verdict does not carry a column for every fault.** The design says a fault's id is "the
   part of the verdict's `#N line L operator@L:C` that does not depend on the run". The verdict's
   label is `#N line L <operator>` (`Fault::label` in `step/faults.rs`). Only some operators name a
   place of their own inside the label (`arg_literal:string->empty@481:47`); `negate`,
   `drop_statement`, `swap_next` and others do not. What was built: the id is
   `<operator>@<line>:<column>` of the fault's site, and an operator that already names its place
   is the id as it stands. For `step protect result` this means a fault with a plain operator can
   be matched to the verdict by line and operator only. On the real receipt no two faults share
   file, line and plain operator, so the match was unique there; that is one file set, not a rule.
   Task 2.1 has to decide: match by line and operator and say so when two faults cannot be told
   apart, or read the receipt of the check's run, which has the column.
2. **A file of `protect_begin.rs` alone went over the size budget** (3915 tokens against 2800).
   The shape of `stage.json` is in `protect_stage.rs`, the writing in `protect_write.rs`, what is
   said in `protect_says.rs`. The file plan of the design is amended.

## Deviations from the tasks, each small

- `protect.rs` has the command `begin` only. `result` is added with task 2.1 and not as a command
  that exists and does nothing.
- Refusals beyond the five of task 1.1, all exit 3: the source root of the run configuration is
  not the repository (a declaration has paths from the repository's root); the configuration names
  no test path; every named function is left out; `--spec` or `--specs-dir` would lead outside the
  repository; the plan planted no fault in the file of a named function.
- `stage.json` carries `other` beside `noticed` and `unnoticed` per chosen function.
- The command does not say how much disk one copy took in the plan (point 5 of "What the prototype
  changed"). The receipt of a plan does not record it. It says the time: the seconds of the faults
  one after another, and with the plan's copies at once plus one run of the tests without a fault.

## Limits

- Not checked: that a chosen file is committed at `HEAD` as the plan had it. The plan's hash is of
  the working file; where git converts line endings the committed bytes differ, so a comparison
  with the commit would refuse rightly made plans. A plan made on uncommitted changes of a chosen
  file is therefore not refused here; the base run of the stage would show other faults than
  `stage.json` lists.
- A name that two functions of one file share is one row of the plan. If one of the two is not
  built on this system, its unnoticed faults are listed with the others and are not marked.
- The toy's faults take less than a tenth of a second; the times the command prints are shown for
  their arithmetic, not for their worth as an estimate. The estimate is first tried in task 5.1,
  against 17 min 50 s of the base's run in R184.

## Next gate

Group 2, after the decision on finding 1.

## COBESY check

The outcome and its limit come first. A reader can replay with
`cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --test step_protect`. What
would reject the claim: a refusal that writes a file, or a declaration `step check` does not read;
both are tests. Nothing is claimed about a real project.
