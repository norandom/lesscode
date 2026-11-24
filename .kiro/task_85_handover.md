# Handover 85: brownfield test stage, group 1 done (`step protect begin`)

Written 2026-11-24, 14:10 Europe/Berlin. Predecessor: [task_84_handover.md](task_84_handover.md).
Spec: [brownfield-test-stage](specs/brownfield-test-stage/tasks.md). Record:
[R185](../docs/research/2026-11-24/06-step-protect-begin.md).

## What was done

- **1.1** (`43ec698`): `lesscode-core step protect begin --plan --config --function <file>:<name>
  --spec <name> [--branch] [--specs-dir] [--repo]`. Reads the plan, the receipt of its run, the
  run configuration and the code. Refuses with exit 3 before anything is written.
- **1.2** (`016d8d0`): writes `step.json` (a declaration `step check` reads) and `stage.json`
  (`step-stage-v1`) into `<repo>/<specs-dir>/<name>/` and nowhere else.
- **1.3** (`600ca46`): prints what is chosen with each unnoticed fault, what is left alone, what
  is left out, and the cost of running the faults again from the seconds of the plan.
- The owner's direction of 2026-11-24, given after the approval of the tasks: **the web dashboard
  is off by default and the existing flows can be automated in CI/CD.** It is a rule in
  `tasks.md`. Checked: `--page` and `--open` do nothing unless given; the new command opens
  nothing, asks nothing and answers by exit code. Still to do under this rule: the skill (group 3)
  and the guide (group 4) say how the stage runs in a pipeline, without `--open`.

## Files

`native/src/step/`: `protect.rs` (dispatch; `begin` only), `protect_begin.rs` (arguments and
checks; 2739 of 2800 tokens, so nothing more goes into it), `protect_stage.rs` (shape of
`stage.json`, the id of a fault), `protect_write.rs` (the two files), `protect_says.rs` (the
output). Tests: `native/tests/step_protect.rs`, nine, each on a real plan of the toy repository.
Budgets entries for all five. The file plan of the design is amended.

## State

- Windows at `600ca46`: core suite 487 of 487, 1 skipped; lints, format, doctests, size budgets,
  `tools\bin\kiro-cli.exe code check --changed HEAD~2` with 0 errors and 4 warnings
  (`protect_stage.rs` has 9 types against 8; `chosen` there has long statements; `run` in
  `step/mod.rs` is at cyclomatic 9 of 10).
- Linux through Dagger at `600ca46`: overall 0; core suite 486 of 486, 1 skipped.
- Nothing was run on a real project. That is task 5.1.
- `origin/main` is at `9f6aab7`. Local and not pushed: `504cfa3`, `409f6bf`, the three task
  commits and the commit of this handover. Push only on the owner's word.

## Before task 2.1: one decision

The design says a fault's id can be read from the verdict. It cannot for every fault: the
verdict's label is `#N line L <operator>`, and only some operators carry a place of their own
(`arg_literal:string->empty@481:47`). The id written into `stage.json` is
`<operator>@<line>:<column>` of the site, or the operator as it stands where it already names its
place. Two ways for `step protect result`:

1. Match plain operators by file, line and operator, and say so when two faults cannot be told
   apart. Needs only the verdict, so the result can be computed anywhere the verdict is,
   also in a pipeline. On the receipt of the renderer's plan (268 faults) no two collide.
2. Read the receipt of the check's run, which has the column. Exact, but the receipt lies in the
   work directory and is not committed.

Recommended: 1, because it keeps the result computable from committed files.

## Deviations and limits (all in R185)

- `result` is not yet a command; no stub was committed.
- Five refusals beyond the task's five (source root is not the repository; no test path; every
  named function left out; `--spec`/`--specs-dir` leading outside; no fault planted in the file).
- The command does not say how much disk a copy took: a plan's receipt does not record it.
- Not checked: that a chosen file is committed at `HEAD` as the plan had it (line-ending
  conversion makes the comparison refuse rightly made plans).
- Unnoticed faults of a same-named function that is not built here are listed unmarked.

## Storage

D: has 120.0 GiB free. No isolated target directory was made; builds used the shared
`D:\BuildArtifacts\cargo-target`. Kept: `D:\BuildArtifacts\lesscode\brownfield-stage` (three
budget reports and `linux-g1`, 0.4 MiB). The tests' repositories under the temp directory remove
themselves.

## Next

Group 2, tasks 2.1 to 2.3 (`step protect result`, new file `protect_result.rs`, the command added
in `protect.rs`), in the foreground with explicit task numbers, one commit per task. Read first:
the design's section on `step protect result`, `step/guard.rs` (how a verdict is matched to the
committed content; task 2.1 reuses it), `step/verdict.rs`, `protect_stage.rs`. Next research
record R186; next handover `task_86_handover.md`.

Carried, unchanged from handover 84: the end screen of the run page; the clone's branch
`protect/display-path` is merged nowhere; release 2.2.0 is not tagged; the state-metrics extension
in openidea-kiro and the stopping-rule test of R163; the hook test that fails under load. Drift
hardening (Slice B): closed by the owner, only the drift of one model over time is unmeasured.
