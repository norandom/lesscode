# R174: the run page, groups 1 and 2: the snapshot, and `step plan` telling it

- Report date/time and timezone: 2026-11-23, 16:21 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, 15:45 to 16:21 (from the commits `89d0665` to `84e380b`)
- Status: completed on Windows; the Linux run for group 2 was under way when this was written
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 1.1, 1.2, 2.1, 2.2, 2.3
- Hypothesis ID and predecessor: the [prototype gate](../../../.kiro/specs/run-dashboard/prototype-gate.md) of the same day
- Type: implementation record with its deviations; nothing was preregistered beyond the tasks' own "done when"

## Decision

Groups 1 and 2 are done as far as Windows shows. A run can now tell a listener what happens in it,
and `step plan` can keep that as the value the page will draw from. No command sets a listener yet,
so nothing a user runs has changed. Group 3 (serving the run, and the options) is next.

## What exists

| File | What |
| --- | --- |
| `native/src/adapters/driver/page_snapshot.rs` | the one value the page shows, version 1 |
| `native/src/adapters/driver/page_log.rs` | what a run tells, kept as that value; no input or output |
| `native/src/adapters/driver/page_subject.rs` | repository, branch, commit, uncommitted work, main branch, asked from Git with three read-only commands |
| `native/src/adapters/driver/page_watch.rs` | the events the batch runner tells, one listener for the process, and a heart |
| `native/src/step/plan_page.rs` | the listener of `step plan`: each fault under its function, the stages, the end |
| `native/tests/run_page_*.rs` | 25 tests in four files |

Changed in existing code: three lines in the batch runner and one in the run command that tell the
listener; `step plan` hands the fault run a listener when its arguments carry one, and ends the
snapshot when the plan is written or the run could not judge.

## The "can fail" item of task 2.1 did not fail

The tree of the page needs the function of a fault while the run goes on. The runner knows only the
place of a fault. But the listener can read the functions of each file before the run, and the place
then gives the function (`FileFunctions::owner`). The test compares the per-function counts with the
receipt's total.

## Deviations from the tasks, each for a stated reason

1. **Where the code sits.** The task asked for a module of its own in the core. `lib.rs` is at its
   frozen ceiling (197 units), and one more declaration failed the budgets check. A directory under
   the driver then failed a guard test that reads the driver's directory as flat. Neither the ceiling
   nor the test was changed; the files sit flat in the driver.
2. **The runner tells three things, not every stage.** The faults planned, the unchanged tests of a
   copy, each judged fault. Copying and the first run of the tests are one stage on the page. A copy
   that fails before its tests run is not told while the run goes on; it is in the receipt.
3. **One listener for the process, not a parameter.** `runner.rs` had 63 tokens under its ceiling
   of 2,800. Passing a listener through would have gone over. It now has 1 token left: the next
   change to that file needs it split first.
4. **A fifth class.** A fault whose run failed twice has no class in the driver; the snapshot of task
   1.1 had no place for it. It has one now.
5. **The heart is tested on the log, at 100 milliseconds**, not with a command that sleeps for more
   than five seconds. No command starts a heart yet.
6. **Only the calls of an unprotected function** are carried into the snapshot, not its state writes.

## Faults of mine, found by the checks and corrected

- `kiro-cli` found `RunLog::classed` over the limit for cyclomatic complexity, 11 where 10 is
  allowed, and that was already so in the commit of task 1.1, which I had not checked with it. The
  counting is one function now, used three times.
- The same check found a test of mine 87 lines long where 60 are allowed. It is three tests now.
- The first attempt at task 2.3 put `run_plan` over its ceiling for one function, 578 tokens of 560;
  two hook tests failed with it. The fault run moved into a function of its own.
- The first fixture for task 2.3 had no unprotected function, so the test could not show what it
  was written for. The fixture has one now.
- This record first said 27 new tests and 410 before the feature. Counted in the files it is 25,
  and 437 less 25 is 412. Corrected at 16:25.
- A date arithmetic exists twice: in the step ledger and in the snapshot. The driver may not reach
  up into the step commands, and I did not change the ledger.

## Evidence

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, at `84e380b` | 437 of 437 (412 before the feature) |
| Lints with warnings as errors | clean |
| Format check | clean |
| Size budgets | pass; `runner.rs` 2,799 of 2,800 tokens, `run_run` 552 of 560 |
| `kiro-cli code check --changed 801fa20` | passes; one finding that was there before (`run_batch` has 6 parameters) |
| Linux through Dagger at `a34cebf` (group 1) | overall 0; 424 of 424 |
| Linux through Dagger at `84e380b` (group 2) | under way at 16:21 |

Budget receipts: `D:\BuildArtifacts\lesscode\run-dashboard\`. Linux evidence:
`D:\BuildArtifacts\lesscode\native-evidence\linux-a34cebf`.

## Limits

- Nothing here was seen in a browser. The page itself is group 4.
- A run of a real project with a listener has not happened; the tests use the toy project of the
  driver's own suites.
- One listener for the process is safe because the driver's tests run one test per process. A
  caller that ran two fault runs at once in one process would have them share it.

## Next

Group 3: the listener on `127.0.0.1` with the standard library, and `--page` and `--open`. Before
anything else touches `runner.rs`, it has to be split.
