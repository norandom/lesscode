# R173: `SharedFrame` reworked so that what cannot be reached cannot be written

- Report date/time and timezone: 2026-11-23, 12:21 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R172](09-tests-from-the-remaining-faults.md); the state of the type from [R165](02-state-holder-count.md) to [R170](07-holders-read-while-the-tests-run.md)
- Type: preregistered

## Decision

Open.

## What the measurements say about the type as it is

`SharedFrame` in `drift-rdp` keeps four holders: `wake`, and `dirty`, and behind a lock `damage` and
`full`. They can be written in 16 ways; the code reaches 12 (R166, R167, R169). The four it never
reaches are those with `full` set and no `damage`. `dirty` is set by `mark_dirty` and read by one
function that exists in test builds only (R166). Two planted faults that keep the lock while the
render thread is rung cannot be noticed by any test of the type (R172): that the lock is released
first is said by two `drop(guard)` lines and by nothing else.

## The rework, decided before it is written

Inside the file only. The public functions keep their names, parameters and results; the six other
files of the crate that call them are not touched.

1. `damage: Option<Region>` and `full: bool` become one value with three cases: nothing pending, a
   rectangle pending, the whole frame pending. "Full without damage" has no case.
2. `dirty` goes. The test-only `publish` compares the frame's version with the version it saw last,
   kept in a field that exists in test builds only.
3. `update_full` and `update_rect` take the lock for one expression, so that it is released by
   the shape of the code before `mark_dirty` runs. No `drop(guard)` line is left to forget.

Not changed: callers that write through `begin_write` and call `mark_dirty` themselves can still
call it while they hold the guard. Closing that needs a change of the interface, which is the
owner's to decide.

## Hypothesis and falsification

Written before the code is changed.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | The change is one commit that touches one file and no test; the 542 tests pass on it as they are | a test has to change, another file changes, or another count |
| 2 | The released flow accepts the step: `step check` with the commit after the tests as base, growth of the file allowed up to 60 tokens, set now | refused or not judged |
| 3 | Counted by the program of R165: writable shapes of `SharedFrame` go from 8 to 3 (it counts `wake` as one shape; with `wake` by hand, 16 to 6) | another number |
| 4 | Watched while the tests run, the tests see 6 shapes of `wake` and what is pending, and 6 is all that can be written: nothing unreachable is left | fewer than 6 seen, or a shape that cannot be explained |
| 5 | Planted faults in the file: at most 3 go unnoticed after the change (4 before), and the two about releasing the lock have no place to be planted any more | more than 3, or one of the two still exists |
| 6 | Functions and calls in the file (`measure-scope`: 15 and 8 before) | not judged; reported |
| 7 | Size of the file in tokens | not judged beyond row 2; reported |

Row 5 leaves room for one new unnoticed fault in the code I add. If there is one, it is named.

What I do not expect: fewer decisions. The condition for an empty rectangle stays, and the three
cases need a choice where two fields were read. The gain registered here is in the state, 16 ways
to 6, not in size.

## Frozen inputs and method

- The branch `lesscode/shared-frame-wake-tests` of `drift-rdp` at `e4d3b2c` (after the six tests),
  in its worktree. Two more commits: the declaration of the step, then the change. Not pushed.
- `lesscode-core` built at `a60f426` (R172), commands `step check` and `deconvolute measure-scope`;
  `state-holders` revision 3; `scope-watch` revision 1.
- Declaration: `.kiro/specs/shared-frame-pending/step.json` on the branch; base `e4d3b2c`.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r173-rework`.

## Evidence and results

Not run yet.

## Limits and interpretation

- The change is written by hand, by the author of the hypothesis. The tool judges it; it did not
  write it.
- Tests and planted faults show how sharp the tests are, on one thread. They do not show that two
  versions behave the same under two threads.
- Whether the owner wants `dirty` gone is the owner's decision at merge; it is read nowhere in the
  shipped program today.

## Tool/spec consequence and next gate

This is the first turn of the whole loop on real code: measure the state, find where tests do not
look and where they do not bite, write those tests, rework, judge. If the rows hold, it is the
first before-and-after example with all six lenses filled, on one type. A subsystem is still to do.

## COBESY check

The design of the change and what it does not close are written before the code; the allowance in
tokens is set beforehand; no gain in size or decisions is claimed.
