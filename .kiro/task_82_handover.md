# Task handover 82: the run page, group 7

Written 2026-11-23, 20:50 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_81_handover.md](task_81_handover.md).

## Done

Group 7, task 7.1: `lesscode-core step reports <report>... --out <directory>` writes a copy of
each report and `index.html`, one page that lists them, into a new or empty directory. Every
report is one block built the same way, accepted or refused. `--without-names-of <project>` puts
numbers where the files, functions and types of that project are named, in the copy of the report
and so in the list. Commit `b41d903` on main, and one more for this record. Details and nine
deviations: [R179](../docs/research/2026-11-23/16-run-page-group-7.md).

## State

- Windows, on the code of `b41d903`: core suite 471 of 471, 1 skipped (the replay of a recorded
  run). Lints, format, doctests, budgets and `tools\bin\kiro-cli.exe code check` pass.
- Linux through Dagger at `b41d903`: overall 0; core suite 470 of 470, 1 skipped
  (`D:\BuildArtifacts\lesscode\native-evidence\linux-b41d903`).
- The pages in Chromium through Playwright: 65 of 65, none skipped; seven of them are the list.
- `origin/main` is at `bb10ad7`. Groups 3 to 7 and their records are local; the owner chose not
  to push.
- Groups 1 to 7 are ticked. Group 8 is open: 8.1, 8.2, 8.3.

## What the next group has to know

- **Where things are.** `step/reports.rs` (the command), `adapters/driver/page_list.rs` (a row of
  the list), `page_names.rs` (numbers for names), `page_text.rs` (`data_of`, `with_data`, `list`),
  `native/page/list.html`, `native/page/list.spec.mjs`, `native/tests/run_page_list.rs`.
- **The list has its own data**: `schema: 1`, `tool`, `written`, `reports`; a report has `link`,
  `project`, `names_left_out`, `scope`, `date`, `tool`, `command`, `branch`, `commit`,
  `compared_with`, `outcome`, `outcome_detail`, `exit_code`, `reasons`, `lenses`.
- **"Project" is the name of the repository's directory.** For task 8.2 the documentation has to
  say so, and that the option leaves out more than files and functions (types, shapes, the
  project's commands, written files, the source line) and keeps the branch, the commits and the
  directory of the copies.
- **A function named like a common word** takes that word out of a reason of the run too. The
  tool's own sentences are not searched; which they are is decided by the field, in the list
  `OWN` of `page_names.rs`. A new field of the snapshot with the tool's own words belongs there;
  a new field that names a file or function is found only under the keys `file`, `function`,
  `functions`, `type` and `edges`.
- **The browser check** runs `states.spec.mjs` and `list.spec.mjs`. The lists it opens are
  written by `run_page_list` into `LESSCODE_CHECK_REPORTS`; without them three of the seven list
  tests are left out and say so. The commands are in R179.
- **`kiro-cli` is `tools\bin\kiro-cli.exe`**; it is not on the path. `git add -N` on new files
  before `code check --changed`.
- **Tight budgets** as before: `page_log.rs` 2,790 of 2,800 tokens; `runner.rs` 2,799;
  `judge_by_runs` 538 of 560; `check_page.rs` 2,489. None was touched.
- For 8.1 the owner has to be at the screen: `step plan --page --open` on a subsystem of a real
  project. For 8.2 the owner's wish stands: recommend `https://github.com/jamesaphoenix/diff-core`
  after reading it, wording for the owner to approve, humanizer pass before a push. `docs/steps.md`
  does not yet name `step reports`.
- Still not seen: `--open` starting a real browser; the page following a real run; a real run on
  a project of size with the report; a list of reports of a real project; Firefox, Safari, a
  screen reader. Reviews were inline, by me.

## Next

1. Group 8: 8.1 a real run, watched by the owner; 8.2 the documentation; 8.3 validation and
   release notes. Next research record R180; next handover `task_83_handover.md`.
2. Carried, not rechecked: the `kiro-cli` extension for state metrics in openidea-kiro and the
   stopping-rule test of R163; the branch `drift-rdp` unmerged; the hook test that fails under
   load; the humanizer pass before README or guides are pushed. Slice B, the hardening against
   drift of the model: closed by the owner; only the drift of one model over time is unmeasured.

## Storage

D: had 134.8 GiB free at 20:20 and 122.4 GiB at 20:46, after the Linux run. The shared Cargo
target directory grew from 75.9 to 76.0 GiB (shared with other projects, the share of lesscode
not separated), so the 12.4 GiB are not there; where they went was not found out. The Linux run
uses the shared Podman storage, which is not this project's to clean. Both limits hold: more than
50 GiB free, and no new lesscode cache.

Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`: `states-g7-1` (8.9 MiB, the pictures and
the record of the browser check), `reports-g7` (0.9 MiB, seven reports and the two lists with
their reports), `budgets-g7.json`; and what handover 81 names. Linux evidence:
`…\native-evidence\linux-b41d903` (0.4 MiB). Nothing was removed. The temporary repositories of
the step tests under `D:\BuildArtifacts\Temp` remove themselves when a test ends; a test that
was stopped leaves its own, and those were not looked for.
