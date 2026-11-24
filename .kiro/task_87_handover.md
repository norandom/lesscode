# Handover 87: brownfield test stage, group 3 done (the skill `protect-before-reduce`)

Written 2026-11-24, 15:40 Europe/Berlin. Predecessor: [task_86_handover.md](task_86_handover.md).
Spec: [brownfield-test-stage](specs/brownfield-test-stage/tasks.md). Record:
[R187](../docs/research/2026-11-24/08-protect-skill.md).

## What was done

- **The owner's decision "committed only"** (`d8d1d1d`): `step protect result` reads the verdict,
  `stage.json` and `write-offs.json` as they are committed on the branch and no other copy. No
  committed verdict is exit 2; an approval that is written and not committed waits. This closes
  the open point of handover 86.
- **3.1** (`a87df10`): `.agents/skills/protect-before-reduce/SKILL.md`. Who does what, the eleven
  phases with their gates, the stage in a pipeline without `--open`, what no command checks. Read
  against R184 inline; the owner has read it.
- **The owner's decision "print the disk figure"** (`50cfa39`): `step protect begin` says what one
  copy of the plan takes on disk, measured on the directory `w1` beside the run receipt, or says
  that the copies are gone. The design had the sentence; groups 1 and 2 had not built it.
- **3.2** (`8d4b68b`): `templates/requirements.md` and `templates/tasks.md` in the skill.
- `22630db`: two tests split to stay within the size limit of a routine.
- **A defect in the Linux verification, found and changed** (`38d337e`, `.dagger/main.go`): with
  the kept target directory, two runs today tested the binaries of an earlier run and said overall
  0 (the build finished in 0.10 s, 497 tests, the two new ones missing). The module now gives the
  core's files the time of the run, so the core is built in every run. The owner has not seen
  this change yet.

## Files

`.agents/skills/protect-before-reduce/` (the skill, two templates). `native/src/step/`:
`protect_result.rs` (reads committed files only), `protect_says.rs` (the disk figure; `run_of`,
where the receipt of a plan's run lies, moved here from `protect_begin.rs`, which was at its size
budget), `protect_begin.rs`. Tests: `native/tests/step_protect_result.rs` (eight; the verdict and
each write-off are committed), `native/tests/step_protect.rs` (ten). The design is amended twice,
dated, under its file plan.

## State

- Windows at `22630db`: core suite 500 of 500, 1 skipped; lints, format, doctests, size budgets
  (inside the suite), `tools\bin\kiro-cli.exe code check --changed 296a7e6` with 0 errors and 9
  warnings.
- Linux through Dagger, the code of `22630db` with the module of `38d337e`: overall 0; core
  suite 499 of 499, 1 skipped (`linux-g3c`). `linux-g3` and `linux-g3b` are the two runs that
  built nothing; they are kept as the evidence of the defect and are not results.
- Earlier Linux results (checked at 15:46 on the owner's word, amendment of R187): all 27 earlier
  Linux runs kept under `D:\BuildArtifacts\lesscode` built the core; none is stale. The two stale
  runs of today most likely came from a run of mine that was started before an edit and stopped,
  and built the older tests after the newer files were saved. Do not edit files of `native/` while
  a Dagger run is going, and do not stop one half-way.
- Nothing was run on a real project, and no agent has followed the skill. That is task 5.1.
- `origin/main` is at `9f6aab7`. Local and not pushed: everything after it. Push only on the
  owner's word.

## Three points where the skill says more than the design

The owner read the skill with these named and did not object:

1. The agent never fills `approved_by` and `approved_at`, also not from the owner's answer in the
   conversation. The owner writes them and commits the file.
2. Pin receipts go into the stage spec, `pins/pin-<n>.json`, and are committed with it.
3. The base is run by `step pin --fault 1` before any test exists; exit 1 is expected there.

## For task 5.1 to watch

- In every Linux log, look for `Compiling lesscode-core` and the number of tests before taking
  `overall 0`.
- The first pin of a stage is two runs (the base, then the unnoticed faults with the branch's
  tests). `step protect begin` announces the time of the first. Compare with the clock.
- The disk figure leaves out what a project builds outside the copy.
- Run `step protect begin` before the plan's copies are removed, or it cannot say the size.

## Storage

D: has 119.6 GiB free. No isolated target directory was made; builds used the shared
`D:\BuildArtifacts\cargo-target`. Kept: `D:\BuildArtifacts\lesscode\brownfield-stage` (six budget
reports, `linux-g1`, `linux-g2`, `linux-g3`, `linux-g3b`, `linux-g3c`; 2.55 MiB, up from 1.27 MiB). The tests'
repositories under the temp directory remove themselves.

## Next

Group 4, the path in the guide (`docs/steps.md`, `docs/state-refactoring.md`, one paragraph each
in the skills `untangle-state` and `refactor-in-steps`, the release notes; humanizer pass before a
push; done when the owner has read the section). The guide has to say "committed only" for the
result and that `step protect begin` prints time and disk. Then task 5.1, for which the owner
names a file of the renderer. Next research record R188; next handover `task_88_handover.md`.

Carried, unchanged: the end screen of the run page; the clone's branch `protect/display-path` is
merged nowhere; release 2.2.0 is not tagged; the state-metrics extension in openidea-kiro and the
stopping-rule test of R163; the hook test that fails under load (seen again in this group:
`the_installed_hook_stops_a_push_without_a_matching_verdict` failed in a full run beside a second
build and passed alone). Drift hardening (Slice B): closed by the owner, only the drift of one
model over time is unmeasured.
