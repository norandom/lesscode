# Task handover 78: the run page, group 3

Written 2026-11-23, 16:45 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_77_handover.md](task_77_handover.md).

## Done

Group 3: the listener (task 3.1) and the options `--page` and `--open` of `step plan` (task 3.2).
Commits `aa27060`, `76c9329`, `c46ce7a` on main. Details, deviations and limits:
[R175](../docs/research/2026-11-23/12-run-page-group-3.md).

## State

- Windows at `c46ce7a`: core suite 451 of 451, lints, format, budgets and `kiro-cli` pass.
- Linux through Dagger at `c46ce7a`: overall 0, 450 of 450. Group 2 at `84e380b` also passed
  (436 of 436); that was open in handover 77.
- `origin/main` is at `bb10ad7`. The three commits of this group and the records are local.

## What the next group has to know

- The page served and written is a stand-in in `native/src/adapters/driver/page_text.rs`. Task 4.1
  replaces the constant `PAGE` with the page of the prototype (`docs/experiments/run-dashboard/page.html`)
  taken in with `include_str!`. The markers `/*EVENTS*/null` and `/*DATA*/null` are what `live` and
  `report` fill; the tests in `run_page_options.rs` read the snapshot back after `const data = `.
- The page file must not sit in `native/src/adapters/driver` or `native/src/step`: a guard test
  reads every file there and forbids the names of the shipped languages, and "javascript" holds one.
- A second guard forbids `.replacen(` under `native/src/adapters`.
- No browser has been started yet by anything. The first real `--open` is still to be seen, on
  Windows at least; the owner's desktop is where it would open.
- `runner.rs` is still at 2,799 of 2,800 tokens and was not touched.
- The view library (Preact with htm, 13,296 bytes) is not in the repository yet.

## Next

Group 4, tasks 4.1 to 4.4: the page and its library inside the binary, the six states, the tree of
files and functions, and the browser checks.

## Storage

D: had 123.9 GiB free at 16:45. Nothing heavy was built. Kept: budget receipts and Dagger logs
under `D:\BuildArtifacts\lesscode\run-dashboard\`, Linux evidence under `…\native-evidence\`.
