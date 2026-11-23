# Task handover 77: the run page, groups 1 and 2

Written 2026-11-23, 16:21 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md).

## Done

Groups 1 and 2 of the run page: the snapshot, what a run says about the person's code, the batch
runner telling a listener, a sign of life, and `step plan` ending the snapshot with its unprotected
functions. Five commits on main, `89d0665` to `84e380b`. Details and every deviation:
[R174](../docs/research/2026-11-23/11-run-page-groups-1-and-2.md).

## State

- Windows: core suite 437 of 437, lints, format and budgets pass at `84e380b`; `kiro-cli` passes.
- Linux through Dagger: group 1 passed at `a34cebf` (overall 0, 424 of 424). The run for group 2 at
  `84e380b` was under way at 16:21; its evidence goes to
  `D:\BuildArtifacts\lesscode\native-evidence\linux-84e380b`. **Read `overall.exit-code` there before
  calling group 2 closed.**
- `origin/main` is at `a34cebf`; three commits after it are local, plus this file and R174.
- No command sets a listener yet. A user's run is what it was.

## What the next group has to know

- `native/src/adapters/driver/runner.rs` is at 2,799 of 2,800 tokens. It must be split before any
  further change to it. The owner has not been asked how.
- `run_run` is at 552 of 560 tokens.
- The files of the feature sit flat in the driver (`page_*.rs`) and in `step/plan_page.rs`, not in a
  module of their own: `lib.rs` is at its ceiling, and a guard test reads the driver's directory as
  flat.
- One listener for the process (`page_watch::Watching`). Safe under `cargo nextest`; two fault runs
  at once in one process would share it.
- `PlanArgs.page` carries the listener; no option sets it. Group 3 adds `--page` and `--open`, makes
  `step plan` build the listener, start the heart and serve the snapshot.
- The prototype of the page and its gate script are in `docs/experiments/run-dashboard/`; the view
  library is not in the repository yet (task 4.1: one file with its licence and hash).

## Next

Group 3, tasks 3.1 and 3.2. After it, group 4 brings the page into the binary.

## Storage

D: had 124.1 GiB free at 16:21. Nothing heavy was built in these groups. Kept: budget receipts under
`D:\BuildArtifacts\lesscode\run-dashboard\`, Linux evidence under `…\native-evidence\`.
