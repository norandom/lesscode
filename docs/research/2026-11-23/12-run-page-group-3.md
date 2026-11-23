# R175: the run page, group 3: a listener for the length of a run, and `--page` and `--open`

- Report date/time and timezone: 2026-11-23, 16:45 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, 16:26 to 16:45 (push of the groups before, to the end of the Linux run)
- Status: completed on Windows and on Linux
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 3.1 and 3.2
- Hypothesis ID and predecessor: [R174](11-run-page-groups-1-and-2.md)
- Type: implementation record with its deviations; nothing was preregistered beyond the tasks' own "done when"

## Decision

Group 3 is done. `step plan --page [place]` writes a report of the run at its end; `step plan --open`
also serves the run on `127.0.0.1` and starts the default browser once. A run without either option
writes and opens nothing. What the browser shows is a plain stand-in that prints the snapshot; the
page of the design is group 4.

The Linux run for group 2, open in R174, passed at `84e380b`: overall 0, 436 of 436.

## What exists

| File | What |
| --- | --- |
| `native/src/adapters/driver/page_serve.rs` | the listener: the page, the stream of snapshots, nothing else; a secret in the address |
| `native/src/adapters/driver/page_open.rs` | the platform's opener, started directly and not through a shell |
| `native/src/adapters/driver/page_text.rs` | the page as text, with the stream's address or the snapshot inside; the stand-in |
| `native/src/step/plan_show.rs` | the two options: where the report goes, the refusal, the listener, the browser |
| `native/tests/run_page_serve.rs`, `run_page_options.rs` | 6 and 8 tests |

## Deviations from the tasks, each for a stated reason

1. **The page is a stand-in.** Group 3 comes before the page is in the binary (task 4.1). The
   stand-in follows the stream and prints the snapshot as text. Its two markers stay for the real page.
2. **The listener starts only with `--open`.** `--page` alone serves nothing, as the design says a
   run without a browser behaves as before and leaves the report. `--open` alone writes the report
   beside the plan.
3. **"Among the repository's tracked files"** is read as: inside a repository and not ignored by Git,
   asked with `git check-ignore`. A file there would show up as new in `git status`.
4. **A place whose directory does not exist is refused before anything runs.** The task did not ask
   for it. Finding it out after a run of hours would be worse.
5. **Byte-identical** is shown for the plan with and without the options in one run directory. I
   could not compare with a binary from before this group inside a test, and the fault run's own
   receipt carries times.
6. **The exit code with a failing listener** is shown in the tests of 3.2, where a run exists, not
   in those of 3.1.

## Faults of mine, found by the checks and corrected

- The option first took an empty text for "no place"; the argument parser refuses an empty path, and
  two tests through the binary failed. It is an optional value now.
- A guard test forbids replacing the first occurrence of a text anywhere under the adapters. I had
  used that to fill the page. The guard is about fault operators; I did not change it and split the
  page at the marker instead.
- `kiro-cli` found six parameters on `begin` where five are allowed, after I had committed task 3.2.
  A commit of its own corrects it (`c46ce7a`).
- The commit of task 3.1 (`aa27060`) was cut out of a tree that held both tasks. That commit alone
  was not built or tested by me; the tree after 3.2 was.

## Evidence

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, at `c46ce7a` | 451 of 451 (437 before the group) |
| Lints with warnings as errors; format check | clean |
| Size budgets | pass, 85 files listed |
| `kiro-cli code check --changed bb10ad7` | passes, no error |
| Linux through Dagger at `c46ce7a` | overall 0; 450 of 450 |

Linux evidence: `D:\BuildArtifacts\lesscode\native-evidence\linux-c46ce7a`.

## Limits

- **No browser was started**, by a test or by me. The tests use a stand-in that reads the page from
  the address. Whether `rundll32`, `open` and `xdg-open` start a browser on the address is not shown.
- The secret is 32 characters from the standard library's randomly keyed hasher. I did not measure
  its randomness; I rely on the library keying it from the system.
- A very short run can end before a browser has asked for the page. The browser then finds nothing
  at the address; the report is at the printed place.
- The listener checks the secret and that the request reads. It does not check the `Host` line.
- Only `step plan` has the options. `step check` is group 6.

## Next

Group 4: the page of the prototype and its library into the binary, and the six states.
