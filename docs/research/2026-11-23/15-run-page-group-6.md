# R178: the run page, group 6: `step check` on the page, and its report

- Report date/time and timezone: 2026-11-23, 20:15 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, 19:10 to 20:13 (the start is the reading of handover 80, the end is the end of the second Linux run)
- Status: tasks 6.1, 6.2 and 6.3 completed on Windows and on Linux
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 6.1 to 6.3
- Hypothesis ID and predecessor: [R177](14-run-page-group-5.md); uses the numbers of [R166](03-reached-shapes-walk.md), [R170](07-holders-read-while-the-tests-run.md) and [R173](10-shared-frame-rework.md)
- Type: implementation record with its deviations

## Decision

Group 6 is done. `step check` takes `--page` and `--open` like `step plan`. While it runs the page
shows its six stages; when it has ended the page is the report of the step: the outcome in one box,
what was compared, one row per measure of a lens, the planted faults per function on both sides,
a drawing of the functions and calls before and after, and the shapes of a type's state where
someone counted them.

The check itself is unchanged. With and without the page it writes the same verdict, byte for
byte, and returns the same exit code.

Two of the six lenses are not measured by the tool: state and local complexity. The report says
"not measured" for them, unless a file with their numbers is handed in. What is handed in is
marked as handed in.

## What was built

| Part | What it does |
| --- | --- |
| `native/src/step/check_page.rs` | the listener of `step check`: stages, the faults of each side under their function, the end with the verdict's result and reasons |
| `native/src/step/check_show.rs` | the options: where the report goes, what the check reads, what it writes into the repository |
| `native/src/step/check_report.rs` | the rows of the lenses, the faults per function, and what the page draws of functions and calls |
| `native/src/step/check_handed.rs` | the file named with `--measures`: rows of the two lenses the tool does not measure, and shapes |
| `native/src/adapters/driver/page_report.rs` | the report part of the snapshot |
| `native/src/step/plan_show.rs` | the showing now serves any command; `--page-source` |
| `native/page/run.html` | the result of a judged step, the tree by side, the two drawings |
| `native/tests/run_page_check.rs` | eight tests through the binary on the fixture steps |
| `native/page/states.spec.mjs`, `shared-frame.json` | seven fixed situations of a check, three tests of their own |

### The stages (task 6.1)

Six, in the order things happen: the checks that need no run; the tests on the code before the
step; the faults before; the tests after; the faults after; the comparison and the verdict. The
task lists four kinds. The run does base first and step second, so the page shows six stages and
none of them goes on twice.

A step refused before any run ends the page at once: the first stage is marked failed with the
reason, the other five are marked left out, nothing is copied.

Each fault is counted under its file and function **on its side**. The snapshot's rows for files
and functions carry `side: before` or `side: after`. A run of `step plan` has no sides and its
snapshot is unchanged.

The subject of a check is the step's branch and commit, whatever is checked out, and the commit
it is compared with. That is the field Requirement 4.5 needed: the page now warns "the change that
is judged is on the main branch" only where a change is judged, and keeps the other wording for a
plan.

The verdict and, where criteria are tied, the evidence are named as written into the repository.
Without a named place the page would lie beside the verdict, among the repository's files; that
is refused before anything is judged.

### What an acceptance is worth (Requirement 6.4)

Where no test noticed any fault in a function before the step, the acceptance says so: "No test
noticed any fault before the step in: helper. There the acceptance means only that nothing got
worse; nothing was protected before." The per-function table repeats it in the row.

### The report (task 6.2)

| Lens | Measure | Where the numbers are from |
| --- | --- | --- |
| Size | tokens and syntax nodes of the files in scope | the verdict |
| Structure | functions; calls from one function to another; pairs of files with a call between them | the two commits, counted by the code of `deconvolute measure-scope` |
| State | not measured, or the rows handed in | `--measures` |
| Local complexity | not measured, or the rows handed in | `--measures` |
| Behaviour kept | the project's tests with no fault planted: pass or do not pass, on each side | how the two stages of the tests ended |
| Protection | planted faults of the step's functions: noticed; unnoticed; neither | the verdict |

Each row has the direction that was expected, whether the measure moved that way, and the command
its numbers are from. The result names the two commits, the branch, the tool and its version.

A refusal and an acceptance stand in the same box at the top of the result, in the same type, at
the same place. The reasons of a refusal are inside the box. This is checked in a browser by
comparing the two boxes.

The page shows names of files and functions and no source text. The one place that had source
text, the line of code of the example fault, is now left out unless the run is started with
`--page-source`. This also changes the page of `step plan`.

### The two drawings (task 6.3)

**Functions and calls.** The report carries, for each side, the functions of each file by name and
the calls among them. The page draws each side on one rim: a dot per function, a line per call, a
dashed line for a call across files. A function takes the same part of the rim on both sides, so
that the two are to one scale. A scope of more than 400 functions or 2,500 calls is not drawn; the
page then says so and points to the rows of the lens.

**Shapes.** Per type, the shapes its state can be written in, and for each whether the code reaches
it and whether the tests see it. The tool does not count shapes. They come only from the file
named with `--measures`; "reached" and "seen" may each be left out and are then shown as not
measured.

Where the numbers are absent the page says "not measured" under each heading.

## Evidence and results

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, at `d2495e1` | 468 of 468, 1 skipped (the replay of a recorded run); 460 before the group |
| Lints with warnings as errors; format check; doctests; size budgets, at `d2495e1` | pass |
| `kiro-cli code check --changed 22cae1c` | passes; three errors that were there before (`compare` with six parameters, two long tests of `cli_deconvolute.rs`) |
| The page in Chromium through Playwright, at `d2495e1` | 58 of 58: 26 situations at 1,280 and at 400, six tests of their own |
| The seven reports the tool wrote for the fixture steps, opened in the browser | each shows all six lenses, "not measured", what was compared, its outcome in the box, two drawings |
| Linux through Dagger at `6373133` | every step of the check ended with 0 (`D:\BuildArtifacts\lesscode\native-evidence\linux-6373133`) |
| Linux through Dagger at `d2495e1` | overall 0; core suite 467 of 467, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-d2495e1`) |

The fixture steps and what their reports say:

| Step | Outcome | What the report shows |
| --- | --- | --- |
| a pin, then a smaller function | Accepted, exit 0 | tokens 42 to 37; faults of `step` on both sides equal to the verdict's |
| the same, with a file handed in | Accepted | the rows for state and local complexity marked as handed in; two shapes |
| a pin on a function no test looks at | Accepted | "nothing was protected before" |
| the scope grows beyond the declaration | Refused, exit 1, before any run | the reason at the failed stage and in the box; no copy |
| more faults unnoticed after | Refused | all six stages ended; protection "moved: no" |
| the tests fail after the step | Refused | the stage of those tests failed; behaviour kept "pass" to "do not pass" |
| a test command that cannot run | Not judged, exit 2 | behaviour kept "not measured" |

**The numbers of `SharedFrame`.** The browser check uses `native/page/shared-frame.json`:

- the file `src-tauri/src/renderer/shared_frame.rs` of drift-rdp at `e4d3b2c` and at `e02154e`,
  counted now with `deconvolute measure-scope`: 15 functions and 8 calls before, 17 and 12 after;
- the first run on the type: 8 shapes of `dirty`, `damage` and `full`, 6 reached (R166), 6 seen by
  the tests (R170).

R173 gives 11 calls after. That was the first of the step's two commits (`18c9043`, measured again
now: 11). The second commit (`e02154e`) has 12. R173's table is therefore about the first commit
in that row; this entry does not amend it, because R173 says which outputs it used.

The function names in that file were first read from the two source files, because no command
printed the names of the functions of a file. The owner then decided that `measure-scope` lists
them (commit `d2495e1`, receipt `scope-calls-v2`). The command's names for both commits are the
same as the fixture's, 15 and 17, in the same order; the report of a check now takes its names
from there.

Outputs: `D:\BuildArtifacts\lesscode\run-dashboard\states-g6-final` (a picture of every situation
and tab), `reports-g6` (the seven reports), `shared-frame` (the two sources and their
measurements), `budgets-g6.json`.

To write the reports again and check them in a browser:

```powershell
$env:LESSCODE_CHECK_REPORTS = 'D:\BuildArtifacts\lesscode\run-dashboard\reports-g6'
cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml -E 'binary(run_page_check)'
$env:LESSCODE_PAGE_CHECK_OUT = 'D:\BuildArtifacts\lesscode\run-dashboard\<new name>'
node verification/node_modules/@playwright/test/cli.js test --config verification/playwright.config.mjs
```

## Deviations and my faults

1. **Six stages where the task names four kinds.** See above.
2. **Task 6.1 was committed with two test routines over the limits of `kiro-cli`** (189 lines
   where 60 are allowed). `kiro-cli code check --changed` does not see a file that Git does not
   know yet, and the test file was new. I saw it at the check of task 6.2 and cut the tests into
   routines within the limits in that commit. Before the check, new files now get `git add -N`.
3. **The page was changed in task 6.2, not in 6.1.** Between the two commits the page of a check
   mixed the faults of both sides. No release lies between them.
4. **"Structure is measured with `deconvolute measure-scope` on both sides."** The report calls the
   code of that command on the texts of the two commits; it does not start the command. So the
   structure is also measured for a step that is refused before any copy exists.
5. **`--measures` and `--page-source` are new options** that the requirements name only as "a
   file with those numbers is handed in" and "unless the person asked for source text". The
   format of the file is in the head of `check_handed.rs`. It is this tool's own; the evidence
   format of the co-project was read and is about test runs, not about measures.
6. **`--page-source` changes `step plan`.** Its page showed the line of code of the example fault
   since task 5.2. It does not any more without the option. This touches the wording the owner
   has still to read for 5.2.
7. **Not drawn from the page "Two Bowls"**: the mark for functions that write stored state, the
   ring for local complexity, the counts strip of that page. The snapshot has no numbers for the
   first two; the strip is the table of lenses.
8. **The first drawing has no fixture from the tool itself for a real project.** The seven real
   reports are of a toy with two functions and no call. The drawing with 15 and 17 functions is
   from the fixture file.
9. **A file's functions were shown as going on after the end** where their faults were planned
   and never run (the tests failed after the step). Seen on the first real report; the page now
   shows them as left out, and counts no share of unprotected functions for them.
10. **Reviews were inline, by me.**

## Limits and interpretation

- **"Moved that way" is a comparison of two numbers.** A count that stays the same has not fallen,
  and the row says "no". For structure on a step that touches no call this reads harsher than it
  is.
- **Behaviour kept is "the tests pass".** The page says nothing about decisions in total, which the
  lens also names.
- **When the tests do not pass before the step, the check refuses** ("the project's tests do not
  pass on the base"), and the page's notice says the run cannot judge anything. Both are shown.
  What the check returns in that case is not this spec's to change.
- **The reasons of a refusal are the check's own words**, among them "unmutated" and
  `exit Some(1)`.
- **A check that ends with an error** (exit 3) has no report rows; the page says "Could not judge"
  with the error.
- **Handed-in numbers are not checked.** The page says where they are from and that the run did
  not measure them.
- **The copies that drop out are not told apart by side** in the notices.
- Still not seen: `--open` starting a real browser; the page following a real run; a real run on a
  project of size; Firefox, Safari, a screen reader.

## Tool/spec consequence and next gate

Group 7 (a list of several reports) can read the report rows from the data inside each page:
lens, measure, before, after, moved, measured, handed in.

Decided by the owner on 2026-11-23, after the group, from choices put to them:

1. **Source text stays off by default**, for `step check` and `step plan` alike; `--page-source`
   shows it (deviation 6 stands as built).
2. **`measure-scope` lists the names of the functions of a file.** Done in `d2495e1`; its receipt
   is `scope-calls-v2`.
3. The wording of the tab "Planted faults" (task 5.2) was shown to the owner in the conversation,
   with the line on source text as it is now. The answer was "ok"; 5.2 and group 5 are ticked.
4. After this record the conversation is cleared, and group 7 starts from handover 81. Nothing is
   pushed.

Slice B, the hardening against drift of the model: closed by the owner; only the drift of one
model over time is unmeasured. Nothing in this group touches it.

## COBESY check

The outcome is first. Every number has its command or its file. What failed and what I did wrong
is in its own section. What the page does not show is listed. The numbers of `SharedFrame` are
said to be from a fixture file, and the difference to R173 is explained and not smoothed.
