# R171: tests for the four shapes of `SharedFrame` that the code reaches and no test covers

- Report date/time and timezone: 2026-11-23, 11:19 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R170](07-holders-read-while-the-tests-run.md) (what the tests cover), [R164](01-renderer-baseline-faults.md) (what the tests notice)
- Type: preregistered

## Decision

Open.

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

Not run yet.

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
