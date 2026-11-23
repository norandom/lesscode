# R179: the run page, group 7: a list of several reports

- Report date/time and timezone: 2026-11-23, 20:50 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, 20:20 to 20:50 (the start is the reading of handover 81, the end is the end of the Linux run)
- Status: task 7.1 completed on Windows and on Linux
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), task 7.1
- Hypothesis ID and predecessor: [R178](15-run-page-group-6.md)
- Type: implementation record with its deviations

## Decision

Group 7 is done. `lesscode-core step reports <report>... --out <directory>` reads the run inside
each report and writes two things into a new or empty directory: a copy of each report
(`report-01.html`, `report-02.html`, ...) and `index.html`, one page that lists them.

Each report is one block of the list, and every block is built the same way: the project, the
outcome with a mark and a word, the date, the scope, the two commits, the reasons of a refusal or
the last sentence of the run, and one row per measure of a lens with before, after, the direction
expected and whether it moved that way. The name of the project is the link to the report.

`--without-names-of <project>` leaves the names of that project's files and functions out. The
names are taken out of the copy of the report, and the list is made from that copy, so a name
that is not in the report cannot be in the list. The report the person named is not changed.

## What was built

| Part | What it does |
| --- | --- |
| `native/src/step/reports.rs` | the command: reads every report first, then writes; refuses a directory that is not new or empty |
| `native/src/adapters/driver/page_list.rs` | one row of the list from the run inside a report: project, scope, date, outcome, reasons, lenses |
| `native/src/adapters/driver/page_names.rs` | a number for every name of a file, function or type, wherever it stands in the run |
| `native/src/adapters/driver/page_text.rs` | the run inside a written page, the same page with other data inside, and the list page |
| `native/page/list.html` | the list page: the colours and type of the run page, no toolbar and no tabs |
| `native/tests/run_page_list.rs` | three tests: two on a run written by hand, one through the binary on three fixture steps |
| `native/page/list.spec.mjs` | seven tests in a browser; `verification/playwright.config.mjs` now runs both files |

### What the list takes from a report

The project is the name of the directory of the repository (`subject.repository`). The scope is
the files whose functions the run counted or planted faults in. The date is when the run began.
The rows of the lenses are those of the report, unchanged. The list computes nothing: where a
report has no lenses, as that of `step plan`, the block says that the run compared nothing.

### What "without names" does

- Every file, function and type the run names gets a number (`file 1`, `function 2`, `type 1`),
  the same one wherever the name stands. The drawing of functions and calls stays, with numbers.
- In a text of the run (a reason of a refusal, the reason at a stage) a file's name is replaced
  wherever it stands, and the name of a function or type where it stands as a word of its own.
- Left out as a whole: the commands of the project, the files the run wrote into the repository,
  the shapes of a type, and the line of source of the example fault.
- A command of the tool keeps its words; only a file's name in it is replaced.
- The place of the repository becomes the project's name and the sentence that its names are left
  out. That is where the report itself says what was done to it (tab "Your code").
- A project named with the option that no report is of is an error, exit code 3, and nothing is
  written. A name written wrongly would otherwise leave every name in.

## Evidence and results

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, on the code of `b41d903` | 471 of 471, 1 skipped (the replay of a recorded run); 468 before the group |
| Lints with warnings as errors; format check; doctests; size budgets | pass |
| `tools\bin\kiro-cli.exe code check --changed 507eb46` | passes; two errors that were there before (`run` in `main.rs`: 61 lines, complexity 12) |
| The pages in Chromium through Playwright | 65 of 65, none skipped: the 58 of group 6 and seven for the list |
| Linux through Dagger at `b41d903` | overall 0, every step 0; core suite 470 of 470, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-b41d903`) |
| The browser check on Linux | not run; the pages were opened on Windows only |

The fixture: one repository, three steps, checked by the tool with `--page`.

| Step | Outcome | In the list |
| --- | --- | --- |
| a pin, then a smaller function | Accepted | tokens 42 to 37, moved "yes" |
| more faults unnoticed after | Refused | the reason in the box; protection "2; 3; 0" to "2; 4; 0", moved "no" |
| the scope grows beyond the declaration | Refused before any run | the reason in the box; tokens 42 to 47, moved "no"; behaviour and protection "not measured" |

What the tests say about it:

- All three rows of the list have the same fields, and in the browser the three blocks have the
  same parts in the same order. The copies of the reports are the reports, byte for byte.
- Each link opens its report, by keyboard, and the report shows the outcome the list showed.
- With `--without-names-of repo`, none of `app.py`, `expect.txt`, `helper` and `toy_check` is in
  the data of the list or of the three reports, nor in what the browser shows of the list and of
  every tab of every report. The reason of the refusal reads "file 1: function 2: unnoticed
  faults rise from 3 to 4". The page around the data of a report is as it was written.
- With two projects and one of them named, only that one loses its names; the other report is
  copied byte for byte.
- A file that is no report, and a directory with something in it, give exit code 3 and write
  nothing.
- The list page at 400 wide does not scroll sideways as a whole; the table of a block scrolls
  inside its block, as on the run page.

I looked at the pictures `list-1280`, `list-400` and `list-without-names`.

Outputs: `D:\BuildArtifacts\lesscode\run-dashboard\states-g7-1` (the pictures and the record of
the browser check), `reports-g7` (the seven reports of group 6 written again, and the two lists
with their reports), `budgets-g7.json`.

To write them again:

```powershell
$env:LESSCODE_CHECK_REPORTS = 'D:\BuildArtifacts\lesscode\run-dashboard\reports-g7'
cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml -E 'binary(run_page_check) | binary(run_page_list)'
$env:LESSCODE_PAGE_CHECK_OUT = 'D:\BuildArtifacts\lesscode\run-dashboard\<new name>'
node verification/node_modules/@playwright/test/cli.js test --config verification/playwright.config.mjs
```

## Deviations and my faults

1. **Two refusals in the fixture, where the task says one of three.** The second refusal is the
   step that is refused before any run; it is the one in which a size got worse and two lenses
   are not measured.
2. **The reports are copied.** The task says the list links to each report. The list links to a
   copy beside it, because a report without names has to be another file than the one the
   person keeps, and a directory with the list and its reports can be handed on as it is.
3. **"Project" is the name of the repository's directory.** A run records nothing else about
   whose code it is. Two clones of one project in directories of different names are two
   projects for the option. All fixture repositories are named `repo`; the test with two
   projects changes the name inside one written report.
4. **More than files and functions is left out**: the names of types, the shapes of a type (they
   are made of field names), the commands of the project (they name its test files), the files
   the run wrote, and the line of source. Requirement 8.4 names files and functions only.
5. **What stays**: the name of the project, the name of the branch, the two commits, and the
   directory of the copies. A branch or a directory named after a file would show that name.
6. **A function named like a common word.** The fixture's function is `step`. The tool's own
   sentences are not searched, so "the commits of the step" stays. A reason of the run is
   searched, so "test step, exit 1" in a reason would become "test function 2, exit 1"; the
   fixture has no such reason, the test on a run written by hand has one of that kind. Nothing
   of the project is shown by that; a sentence reads worse. Which sentences are the tool's own is
   decided by the field they stand in, and for the last sentence of the run by its exit code 0.
7. **A report keeps the page it was written with.** Only its data is replaced. A report written
   by an older version of the tool therefore still opens with that version's page; the list
   refuses a report whose data has another version than 1.
8. **Not seen**: the list in Firefox or Safari, with a screen reader, or with reports of a real
   project. A list of many reports was not tried; every block shows its whole table.
9. **The review was inline, by me.** No one else read the code.

## Storage

D: had 134.8 GiB free at 20:20 and 122.4 GiB at 20:46, after the Linux run. The shared Cargo
target directory grew from 75.9 to 76.0 GiB, so the 12.4 GiB are not there. I did not find out
where they went; the Linux run writes into the shared Podman storage, and other work runs on this
machine. No target directory was added. Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`:
`states-g7-1` (8.9 MiB), `reports-g7` (0.9 MiB), `budgets-g7.json`, and what handover 81 names;
under `native-evidence`, `linux-b41d903` (0.4 MiB). Nothing was removed. No container of this
project was left; the Dagger engine is shared and stays.
