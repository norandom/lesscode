# Task handover 81: the run page, group 6

Written 2026-11-23, 20:15 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_80_handover.md](task_80_handover.md).

## Done

Group 6, tasks 6.1 to 6.3: `step check` takes `--page` and `--open`; the page shows its six stages
and, at the end, the report of the step: the outcome in one box, what was compared, one row per
measure of a lens, the planted faults per function on both sides, a drawing of the functions and
calls before and after, and the shapes of a type's state where a file with them is handed in.
Commits `d379cf0` (6.1), `4ab1adb` (6.2), `6373133` (6.3), `d2495e1` (`measure-scope` lists function names, an owner decision after the group) on main, and one more for this record.
Details, ten deviations and my faults:
[R178](../docs/research/2026-11-23/15-run-page-group-6.md).

## State

- Windows at `d2495e1`: core suite 468 of 468, 1 skipped (the replay of a recorded run). Lints,
  format, doctests, budgets and `kiro-cli` pass.
- Linux through Dagger at `d2495e1`: overall 0; core suite 467 of 467, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-d2495e1`)
- The page in Chromium through Playwright at `d2495e1`: 58 of 58, among them the seven reports the
  tool wrote for the fixture steps.
- `origin/main` is at `bb10ad7`. Groups 3 to 6 and their records are local.
- Group 5 is closed: the owner read the wording of the tab "Planted faults" in this conversation
  and let it stand; task 5.2 and the group line 5 are ticked.

## Owner decisions of this group

- Source text stays off the page by default, for `step check` and `step plan` alike;
  `--page-source` shows the line of code of the example fault (Requirement 7.9).
- `deconvolute measure-scope` lists the names of the functions of each file (`names`); its receipt
  is `scope-calls-v2`. The fixture `native/page/shared-frame.json` holds what the command prints.
- After this record the conversation is cleared. Nothing is pushed.

## What the next group has to know

- **Where things are.** `step/check_page.rs` (the listener), `check_show.rs` (the options),
  `check_report.rs` (the rows of the lenses, the faults per function, the drawing of calls),
  `check_handed.rs` (the file of `--measures`; its format is in the head of the file),
  `adapters/driver/page_report.rs` (the report part of the snapshot), `plan_show.rs`
  (`Showing<W>` for any command that has a log; `PageArgs` with `--page`, `--open`,
  `--page-source`).
- **The snapshot gained**, all left out or empty for a plan: `subject.compared_with`; `side` on the
  rows of files and functions; `report` with `lenses`, `functions`, `refusal`, `graph`, `shapes`.
  Group 7 reads `report.lenses` of each page: `lens`, `measure`, `before`, `after`, `expected`,
  `moved`, `measured`, `command`, `handed_in`, `note`.
- **`kiro-cli code check --changed` does not see files Git does not know.** Run `git add -N` on
  new files before it. Its limits hold for test routines too: 60 lines, 40 statements.
- **Tight budgets**: `page_log.rs` 2,790 of 2,800 tokens; `runner.rs` 2,799; `judge_by_runs` 538 of
  560; `check_page.rs` 2,489. New report code belongs in a file of its own.
- **The reports of the fixture steps** are written where `LESSCODE_CHECK_REPORTS` names a
  directory, and the browser check then opens them. Without it the test in the browser is left
  out and says so. The commands are in R178.
- **The page's own files have line feeds only.** `native/tests` reads the data inside a page up to
  `;\n`; a page written with carriage returns breaks that.
- When the tests do not pass before the step, the check refuses and the page's notice says the run
  cannot judge. Both are shown; the check was not changed.
- Still not seen: `--open` starting a real browser; the page following a real run; a real run on a
  project of size with the report; Firefox, Safari, a screen reader.
- Guards as before: nothing with the name of a shipped language in `native/src/adapters/driver` or
  `native/src/step`; no `.replacen(` under `native/src/adapters`; `runner.rs` was not touched.

## Next

1. Group 7, task 7.1 and what follows: a list of several reports.
2. Carried, not rechecked: the `kiro-cli` extension for state metrics in openidea-kiro and the
   stopping-rule test of R163; the branch `drift-rdp` unmerged; the hook test that fails under
   load; the humanizer pass before README or guides are pushed. Slice B, the hardening against
   drift of the model: closed by the owner; only the drift of one model over time is unmeasured.

## Storage

D: had 135.0 GiB free at 20:00, 120.1 GiB at the end of group 5 (the rise is not this project's
doing). No target directory was added; only incremental builds in the shared Cargo target
directory (75.9 GiB at 20:00, shared with other projects, the share of lesscode not separated).
The Linux run uses the shared Podman storage, which is not this project's to clean.

Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`: `states-g6-final` (8.0 MiB, the last check
with a picture of every situation and tab), `reports-g6` (0.5 MiB, the seven reports),
`shared-frame` (0.2 MiB, the two sources of the fixture and their measurements),
`budgets-g6.json`; from before, `states-g5-final` (4.7 MiB) and the R164 files. Linux evidence
under `…\native-evidence\linux-6373133` and `linux-d2495e1`. Removed: five superseded outputs of the check
(`states-g6-1` to `-5`, 34.7 MiB). The temporary repositories of the step tests under
`D:\BuildArtifacts\Temp` were not looked for.
