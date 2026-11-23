# R171: tests for the four shapes of `SharedFrame` that the code reaches and no test covers

- Report date/time and timezone: 2026-11-23, 11:19 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, tests written and run 11:20 to 11:23; watched again 11:23:52 to 11:26:55; planted faults 11:26:55 to 11:53:43 Europe/Berlin (from the commits and the runs' own records)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R170](07-holders-read-while-the-tests-run.md) (what the tests cover), [R164](01-renderer-baseline-faults.md) (what the tests notice)
- Type: preregistered

## Decision

All five rows hold. Four tests written from the uncovered shapes close the gap in what the tests
cover, 8 of 12 to 12 of 12, and notice one of the eight faults the tests had missed, the one that
has to do with these shapes. Covering a shape and noticing a fault are different things; the loop
needs both as sources of tests.

## What is done

With `wake` the code reaches 12 shapes of `SharedFrame`; the tests of `drift-rdp` see 8. The four
left are those with `wake` set and damage pending: with and without `full`, with and without
`dirty`. Four tests are written for them, by hand, in the test module of `shared_frame.rs`, on a
branch of `drift-rdp` in a separate worktree. Each puts the frame into one of the four shapes
through the type's public calls and asserts what a reader can observe there: whether the render
thread was rung, the version, and the damage a reader would take.

Then two measurements are repeated on the branch:

- the planted faults of `shared_frame.rs` with the released `step plan`, against R164's 34 noticed
  and 8 unnoticed of 42;
- the holders read while the tests run (R170).

## Hypothesis and falsification

Written before the tests exist. I have read the eight unnoticed faults of R164 in this file:
five are about the bounds of a rectangle and the size of the buffer (lines 93, 100, 127, 133),
two drop the release of the lock before the frame is marked (lines 199, 210), one drops
`mark_dirty` at the end of `update_rect` (line 211).

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | The four tests pass on the code as it is, and the suite has 540 passing tests | a test of mine fails on unchanged code, or another count |
| 2 | Watched again, the tests see all 12 reached shapes of `SharedFrame`, and none outside them | fewer than 12, or a 13th |
| 3 | Of the eight unnoticed faults, the new tests notice exactly one: the dropped `mark_dirty` in `update_rect` | none, or more than one |
| 4 | No fault that was noticed before goes unnoticed | one does |
| 5 | The two faults that drop the release of the lock stay unnoticed: on one thread nothing observes them | one is noticed |

Row 3 is the point of the experiment. Covering a shape and noticing a fault are two different
things. I expect tests written from uncovered shapes to close the gap in what is covered and to
notice one fault in eight, because seven of the eight are not about these shapes at all. If that
holds, the two sources of tests, shapes and faults, are both needed and neither replaces the other.

## Frozen inputs and method

- `drift-rdp` at `09b4ce8`; a worktree at `D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`
  on a new local branch `lesscode/shared-frame-wake-tests`. The owner's checkout is not touched.
  Only the test module of `src-tauri/src/renderer/shared_frame.rs` changes. Not pushed.
- Faults: `lesscode-core` 2.1.0 `step plan`, the configuration of R164 narrowed to this one file,
  with the worktree as the source.
- Watching: `scope-watch` revision 1 with the worktree as the subject.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r171-shape-tests`.

## Evidence and results

The tests: branch `lesscode/shared-frame-wake-tests` of `drift-rdp`, commit `08c6f9b`, in the worktree
`D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`; 85 lines added to the test module of
`shared_frame.rs`, nothing else; `cargo fmt --check` clean on the file. Local, not pushed. Outputs in
`D:\BuildArtifacts\lesscode\lesscode-metrics\r171-shape-tests`: `watch` (the second watching),
`plan.json`, `plan.log`, `work\plan-run.json`.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | The four tests pass on the code as it is; 540 passing tests | 540 passed, 6 skipped | yes |
| 2 | Watched again, all 12 reached shapes are seen and none outside them | 12, the same twelve | yes |
| 3 | Exactly one of the eight unnoticed faults is noticed: the dropped `mark_dirty` in `update_rect` | exactly that one (fault 233 of R164) | yes |
| 4 | No fault noticed before goes unnoticed | 34 noticed before, 35 now, of 42 | yes |
| 5 | The two faults that drop the release of the lock stay unnoticed | both unnoticed | yes |

The four shapes are seen in one or two tests each. The watched suite gives the same result as the
unwatched: 540 and 6.

Still unnoticed in this file: the seven that [R172](09-tests-from-the-remaining-faults.md) takes up.

### Two of three copies dropped out at the start

As in R164, and worse: in two of the three copies the unchanged tests did not pass on their first
run ("the unmutated tests do not pass in this copy"), and those copies ran no fault. The one copy
left ran all 42, and closed with the unchanged tests passing; the result is complete. Run by hand
in one of the dropped copies after the run, the same command passed, 540 of 540.

The first runs happen while all three copies build the project at once. A test that fails under
that load is still only my guess: release 2.1.0 keeps no output of the failed first run. What is
shown is that it happens repeatedly on this project, that the run does not say so in its summary,
and that it cost two thirds of the copies. R172 uses a build of the tool from after the release,
which names the step that failed and the end of its output.

## Limits and interpretation

- The tests are written by hand, by the author of the hypothesis. That a tool could write them is
  not shown here.
- One thread. The moment between the write and `mark_dirty`, in which another thread can look, is
  asserted at rest, by leaving out `mark_dirty`, not by racing two threads.
- A fault noticed means the test is sharp at that place, nothing more.

## Tool/spec consequence and next gate

If row 3 holds: the loop needs both inputs. Shapes say where no test looks; planted faults say
where a test does not bite. The next step would be tests written from the seven remaining faults.

## COBESY check

The expected gain in noticed faults is small and stated; the difference between covering and
noticing is the hypothesis, not an afterthought.
