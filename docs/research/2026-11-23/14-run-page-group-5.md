# R177: the run page, group 5: what the page explains, and faults in code that is not built here

- Report date/time and timezone: 2026-11-23, 19:00 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, about 18:20 to 19:00 (the start is the first check of the group, the end is the end of the Linux run)
- Status: tasks 5.1, 5.3 and 5.4 completed on Windows and on Linux; task 5.2 is written and waits for the owner to read its wording
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 5.1 to 5.4
- Hypothesis ID and predecessor: [R176](13-run-page-group-4.md); corrects a count of [R164](01-renderer-baseline-faults.md)
- Type: implementation record with its deviations; task 5.3 was a question that could fail, and its answer is below

## Decision

Group 5 is done but for one reading. The page says at its top and in a tab what the run touches,
explains a planted fault with one of the run, and gives its notices where tests are missing or do
not pass. Each sentence is drawn only where the snapshot holds its fact.

The question of task 5.3 has the answer yes: a language pack can name the code that is not built on
a system, and a fault there is counted apart. On the recorded run of R164 this finds 66 of 268
faults, not the 64 R164 counted.

Task 5.2 is not ticked. Its "done when" asks that the owner has read the wording.

## Faults in code that is not built here (task 5.3)

**The question.** Can the packs mark a site that lies under a condition of compilation, for Rust
first? If not, the page was to say nothing.

**What was built.** A pack may carry `not_built_markers`: per operating system, the texts of
attributes under which code is not built on that system. The Rust pack names whole conditions only,
such as `#[cfg(target_os = "linux")]` for Windows. An item with such an attribute directly in front
of it is not built, and neither is anything inside it. The search is the one that already finds test
code by its attribute.

The page's log then counts a fault at such a place apart, whatever its run gave. Such a fault is
never the example. A function that holds only such faults is neither protected nor unprotected and
is left out of the share.

**The check that could fail.** The record of R164 (268 faults on the renderer of `drift-rdp` at
`09b4ce8`, run on Windows) was told to the page's log again, with the sources of that commit:

| | R164's count | This count |
| --- | --- | --- |
| In code not built on Windows | 64 | 66 |
| Built here: noticed | 72 | 72 |
| Built here: unnoticed | 130 | 130 |
| Built here: did not build | 2 | 0 |
| Functions with no fault noticed | 21 | 16 of 38 |

The two more are single statements under a condition: `#[cfg(not(windows))] configure();` in
`gpu.rs` line 474 and `#[cfg(target_os = "linux")] linux::collect(&mut facts);` in `display_path.rs`
line 484. R164 counted by the line ranges of two modules and took these two for built code whose
fault did not build. They are not built on Windows. Of the 66, the run had classed 63 as unnoticed
and 3 as not building.

The first run of the check gave 66 where I had written 64 into it, and failed. I looked at the two
places before I changed the number.

**What it cost.** The Rust pack was at the admission limit (40 lines, about 2,480 of 2,500
characters). The markers need 9 lines and about 650 characters. Four tests of admission failed.
The owner decided to raise the limit to 50 lines and 3,200 characters for every pack.

**What it does not do.**

- The run and the plan are unchanged. They still plant these faults, spend the time on them (a
  quarter of the renderer run) and count them as unnoticed. The page and the plan therefore differ
  on such a function: the plan calls `attach` unprotected, the page says nothing is known about it.
  Leaving such places out of a run is a change to what a run measures and was not decided.
- A condition the pack does not name is not found: `any(...)`, `all(...)`, features, a condition
  written with other spacing. Those faults are counted as before.
- A module made conditional in another file (`#[cfg(windows)] pub mod dcomp;`) is not found from
  inside its own file. On Linux the faults of `dcomp.rs` would be counted as before.
- The system is the one the tool runs on. A test command that builds for another system is not
  seen.
- Only the Rust pack has markers.

## What the page says now

| Where | What | When |
| --- | --- | --- |
| A line at the top, for the whole run | the tool works in copies, and where | the place only if known |
| | "It writes nothing into your repository", or the files it writes "and nothing else" | only if the repository is known |
| | the branch, and "this is the main branch" | only if it is |
| | "Your working tree had uncommitted changes when the run began" | only if it had |
| | "The copies take ..." | at the end, if measured |
| Tab "Your code" | the same in full, the project's commands, and how to remove the copies | removal at the end |
| Tab "Planted faults" | what a planted fault is; one of this run; what a noticed one shows and does not; three things an unnoticed one can mean; what is counted apart | the example once a fault is judged |
| Notices | no test command; the test command ran no test; the tests do not pass on the unchanged code; a copy dropped out | each alone |
| Result | the share of unprotected functions as its first line | when the run has ended |
| Tab "Unprotected" | the sentence on code that needs a device | only where a function's reaching out is listed |

## Deviations

1. **Task 5.1 went beyond the page.** Its boundary is the page. Two facts the page needs were never
   set by a run: the space the copies take, and what the run writes into the repository. `step plan`
   now measures the copies at its end and names those of its own outputs (the plan, the page, the
   directory of the copies) that lie inside the repository.
2. **The warning on the main branch** is shown whenever the run reads the main branch. Requirement
   4.5 asks for it where the run judges a change. Whether a run judges a change is not in the
   snapshot before `step check` tells the log (task 6.1).
3. **A test command that runs no test** has its notice in the page, but `step plan` does not count
   the tests that ran, so a real run never shows it.
4. **Task 5.3 shows 66 and not 64**, see above.
5. **The pack limit was raised**, by the owner's decision in this session. The guides of the earlier
   experiments (`docs/experiments/annotation-driver/pack-guide-v0.md` and `-v1.md`) still name 40
   lines and 2,500 characters; they are records and were left.
6. **A test that is not run by default.** The replay of the recorded run needs files that are not
   in the repository and is marked to be left out; the suite reports it as skipped.

## Faults of mine

- For 5.1 (the Rust part) and 5.2 I wrote the test and the code in one step and did not see the
  test fail first. For the page of 5.1 and for 5.4 the check failed first (10 of 33 and 8 of 39).
- A fixed situation said "1 of 3 functions" where its own data gave 2 of 3. The new check of the
  order found it.
- The first replay showed "No test command was given" on the recorded run: the replay had not told
  the log that there was one. Corrected in the replay.

## Evidence

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, at `30f2959` | 460 of 460, 1 skipped (the replay); 456 before the group |
| Lints with warnings as errors; format check; doctests; size budgets, at `30f2959` | pass |
| `kiro-cli code check --changed fa245a6` | passes; one error that was there before (`sites_of`, nesting 4) |
| The page in Chromium through Playwright, at `30f2959` | 41 of 41: 19 situations at 1,280 and at 400, the drawing made to fail, the walk, reduced motion |
| The replay of R164's record | 66 apart, 72 noticed, 130 unnoticed, 0 not built, of 268 |
| The report of that replay, opened in Chromium | the 66 are a hatched part of the bar; `collect` has the mark for "nothing is known" |
| Linux through Dagger at `30f2959` | overall 0; core suite 459 of 459, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-30f2959`) |

Records under `D:\BuildArtifacts\lesscode\run-dashboard\`: `states-g5-final` (the last check, a
picture of every page and tab), `r164-renderer-report.html` and `.png`, `r164-source` (the renderer
sources at `09b4ce8`, 0.1 MiB, read from `D:\Source\drift-rdp` and not changed there).

To run the replay:

```powershell
$env:LESSCODE_RECORDED_RUN = 'D:\BuildArtifacts\lesscode\lesscode-metrics\r164-renderer-baseline\work\plan-run.json'
$env:LESSCODE_RECORDED_SOURCE = 'D:\BuildArtifacts\lesscode\run-dashboard\r164-source'
$env:LESSCODE_RECORDED_PAGE = 'D:\BuildArtifacts\lesscode\run-dashboard\r164-renderer-report.html'
cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml -E 'binary(run_page_not_built)' --run-ignored only --no-capture
```

## Limits

- The replay tells a record to the page's log. No run was made with the new pack; a real run on
  the renderer was not repeated.
- Measuring the copies walks their directory once at the end of a run. On a large project this was
  not timed.
- One browser engine, no screen reader, no real run followed in a real browser, as in R176.
- The fixed situations and the wording are mine. The owner has not read the wording.
- The review of each task was mine, inline; no second reader.

## Next

The owner reads the wording of the tab "Planted faults" (task 5.2). Then group 6: `step check` tells
the log, and the report.
