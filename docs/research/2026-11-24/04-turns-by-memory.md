# R183: a copy says why it stopped, tries again, and the copies take turns by free memory

- Report date/time and timezone: 2026-11-24, 10:45 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, 10:05 to 10:45; the two runs on the renderer 10:18:30 to 10:27:23 and 10:31:05 to 10:41:24 (from their start and end files)
- Status: completed on Windows; Linux through Dagger after the commit
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), requirements 3.5 and 6.2; the four changes [R181](02-run-page-a-real-run-watched.md) named as not done. The owner: "push main, then make the four tool changes".
- Hypothesis ID and predecessor: [R181](02-run-page-a-real-run-watched.md), [R182](03-run-page-closing.md)
- Type: implementation record with one expectation that failed

## Decision

The four changes are in the tool. My first version of the fourth was wrong and a real run showed
it: holding back only the first build of each copy is not enough. Every run of the project's
commands now waits for its turn.

## Expectation, written before the first run on the renderer

Three copies on the renderer of drift-rdp, the case that was void on the morning of this day
(R181), get through when the copies take turns for their **first** run by the memory that is free.

**It failed.** The turns for the first run worked: the first build took about 6.7 GiB, the third
copy waited ("copy 3 waits for its first run: one took about 6.7 GiB and 11.1 GiB are free"), and
all three first runs passed. Nine minutes in, the run was void: while the third copy made its full
build, the other two were building planted faults, and in copy 2 the compiler ended with
`0xe0000001` in the run after the faults. A build with one planted fault takes memory too.

## What the tool does now

All in two new files, `native/src/adapters/driver/first_run.rs` and `memory.rs`. `runner.rs`,
which was at its size limit, lost `tail` and `unusable` to the new file and calls it in three places.

1. **Build or test.** A first run that does not get through is one of four kinds: a command could
   not start; the commands ended before any test ran; a test did not pass; a step ran over its
   limit. "Before any test ran" is a failing step other than the test step, or a test step whose
   output has none of the configuration's `runner_evidence_patterns`. Without those patterns the
   two cannot be told apart and the old sentence stays. A sign that the system ended a command
   (an exit status of Windows from `0xC0000000` on, a kill on Unix, "out of memory" and the like)
   is said in the sentence. Where no test ran, the reason no longer begins with "the unmutated
   tests do not pass", so `step check` answers "not judged" and not "refused".
2. **The whole output** of a failed first run is written to `first-run-w<copy>-<attempt>.log`
   under the work directory, and the sentence names the file. The reason is now one line, then the
   end of the output; the void reason of a run joins the lines only.
3. **A second attempt.** A copy whose first run failed tries once more when no other copy is in
   its first run, unless a command could not start or a step ran over its limit. When a second
   attempt fails too, no other copy makes one: the fault is then not the load. The receipt names
   the first attempt under `retried` (absent where there was none).
4. **Turns by memory.** Runs are of two kinds: the first of a copy, and a later one (with a
   planted fault, or the unchanged commands at the end). The first run of each kind goes with no
   other beside it while the tool reads the least memory the system had available during it; the
   difference to before is what such a run takes. After that a run starts when nothing is going
   on, or when what was free with nothing going on holds what is set aside for the runs going on,
   this run, and a quarter of it to spare. Windows: `GlobalMemoryStatusEx`, the smaller of free
   physical memory and what can still be committed. Linux: `MemAvailable`. Elsewhere unknown, and
   nothing is held back.

On the page the tool's sentence is read at once ("The tool says: ..."); only the end of the
command's output is behind the line that opens.

## Evidence

All under `D:\BuildArtifacts\lesscode\run-dashboard\`.

| What | Result |
| --- | --- |
| Three copies on one file of the renderer (`frame_stores.rs`, 4 faults), turns for the first run only | **void**, exit 3, after 8 min 53 s (`first-run-renderer-1`) |
| The same with turns for every run | exit 0 after 10 min 19 s; 4 of 4 faults noticed; all three copies closed with the unchanged tests passing; no second attempt was needed. A full build took about 8.2 GiB, 13.4 GiB were free with nothing going on (`first-run-renderer-2`) |
| A toy project whose test command fails once per copy, two copies | both first attempts kept in `first-run-w1-1.log` and `first-run-w2-1.log`, both second attempts got through, exit 0, `retried` in the receipt (`first-run-smoke`) |
| Unit tests of the new files | 7: the kinds told apart; a second attempt and its kept output; no further second attempt after one failed; turns when memory does not hold two; side by side when it does; the reading of `/proc/meminfo`; this system tells |
| Core suite on Windows | 478 of 478, 1 skipped |
| The pages in Chromium through Playwright | 67 of 67 (`states-g10-1`); the lane test also checks the tool's sentence |
| Lints with warnings as errors, format, size budgets (`budgets-g10.json`), `kiro-cli code check --changed 3fe351b` | pass; `Turns::enter` was at cyclomatic 11 and was split |

The second renderer run used the binary before `Turns::enter` was split into `State::admits` and
`State::says_why`. The split changes no decision; the suite ran after it, the renderer did not.

## Limits

- One machine, one project, one file of four faults. The full renderer (268 faults) was not run
  again with three copies. With 13 GiB free and 8 GiB for a full build the copies of this run
  mostly ran one after another; that three copies are faster than one here was not shown and is
  not expected.
- The memory a run takes is read from the whole system. Another program that takes memory during
  the measured run makes the number too large; one that frees memory makes it too small.
- A later run is measured on the first planted fault. A fault whose build is much larger than that
  one is not allowed for, beyond the quarter to spare.
- The message that a copy waits goes to the terminal. The page does not show it; the stage only
  takes longer.
- A project whose tests fail on the unchanged code now costs one more run of them before the tool
  gives up.
- macOS was not run. Reviews were inline, by me.

## Next gate

The brief [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/brief.md).
