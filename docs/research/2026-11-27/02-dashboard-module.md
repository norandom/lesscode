# R201: The run page's code is a module of its own; the move changed no page byte, and `lib.rs` needed one line over its ceiling

- Report date/time and timezone: 2026-11-27, about 11:00 Europe/Berlin (from the clock)
- Execution interval: 2026-11-27, about 10:00 to 11:00 Europe/Berlin
- Status: completed
- Spec: [report-set](../../../.kiro/specs/report-set/tasks.md), task group 1 (tasks 1.1 and 1.2);
  design decisions D7 to D9
- Hypothesis ID and predecessor: the "can fail" point of task 1.1; predecessor
  [R200](01-report-set-prototype-gate.md)
- Type: preregistered (the stop point and the byte comparison were written into the task before
  the move)

## Decision

The ten files of the run page are `native/src/dashboard/` now, apart from core, and a test keeps
the border. Nothing the tool writes changed. The one thing the task said could fail did:
`lib.rs` is at its size ceiling and the module's one line put it over. The owner granted one more
registration-only line. Group 2 of the spec may start.

## Hypothesis and falsification

Task 1.1 said: a pure move changes no behaviour, which is rejected if the suite, the browser checks
of the run page and the list, or the bytes of the pages the tool writes differ before and after.
It also named the stop: if `lib.rs` cannot take the line under its ceiling, the task stops and the
owner decides.

## Frozen inputs and method

- lesscode at `8aacae7` before, `2db0f8d` after the move, `a6a8098` after the border
  test. Rust 1.98.1, cargo-nextest, target directory `D:\BuildArtifacts\lesscode-cargo-test`.
- The move was made by a script (`move.ps1`): `git mv` of the ten files, then text replacements of
  module paths only. `cargo fmt` then reordered four imports.
- The byte comparison (`pages.ps1`): `step reports` on three real report pages of R198
  (`check.html`, `check-4-notjudged.html`, `plan.html`), once as they are and once with
  `--without-names-of lesscode-renderer-rework`, which makes the tool rewrite each page's data.
  Eight files: six report copies and two lists, the lists with their written moment masked. Two
  runs of the binary before the move gave the same bytes, so the comparison is repeatable.
- Evidence: `D:\BuildArtifacts\lesscode\dashboard-move\`.

## Evidence and results

| Check | Before | After |
| --- | --- | --- |
| Rust suite, Windows | 552 passed, 1 skipped | 552 passed, 1 skipped (after the owner's decision); 555 with the border test |
| Browser checks of the run page and the list | not run before | 67 passed, none skipped, with the fixture reports of `run_page_check` and `run_page_list` written |
| Pages from `step reports`, 8 files | — | the same bytes |
| Budgets | 100 within, 7 disposed | 101 within, 7 disposed, 0 exceeded |
| fmt, Clippy with warnings denied | — | pass |
| Linux through Dagger | — | "All native checks passed" on `2db0f8d` in 707 s; on the border test's commit see the handover |

The stop point: with `pub mod dashboard;`, `lib.rs` measured 198 logical units against a ceiling of
197 (4,112 tokens against 4,127). Three tests failed, all of them the budgets check run on the
repository itself. The owner chose, of three options put to them, one more registration-only line,
as 18 earlier module lines were added; the budget entry records it.

The border test was shown to fail in three trials, each undone: a core file (`compare.rs`)
importing the dashboard's report types; a dashboard file (`list.rs`) importing `step`; an empty
`page_trial.rs` in the driver's directory.

Found on the way: the browser checks of the run page skip 5 of their 67 tests unless
`LESSCODE_CHECK_REPORTS` names an existing directory into which both `run_page_check` and
`run_page_list` have written. Set for only one of them, one check fails instead of skipping. That
is how they were built in `run-dashboard`, not a result of the move; group handovers should say
which way they ran.

## Limits and interpretation

- The byte comparison covers the list page and the rewriting of a report's data. It does not cover
  a report written by `step plan --page` or `step check --page`, whose data carries the time of the
  run; for those the page template is the same file, taken in by `include_str!` from a path one
  directory shorter.
- The border test reads text. A path to the dashboard written another way (a `use` with braces
  across lines that splits `dashboard::`) would not be seen. None exists today.
- The crossing from core to the dashboard, the runners telling the watch, stays. Taking it out would
  change core and is not part of this spec.

## Tool/spec consequence and next gate

The dashboard module is where the report set's code goes (group 2 on). Next: task 2.1, the set page
in the tool's own page, which the gate did not show.

## COBESY check

- Outcome first: the title says what moved, what did not change, and what failed.
- Replay: the two scripts and the logs are in the evidence directory; the commits are named.
- Disproof: the stop point was written before the move and fired; it is reported with its numbers.
- Facts and interpretation: the table is what ran; the limits say what it does not cover.
