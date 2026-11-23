# R173: `SharedFrame` reworked so that what cannot be reached cannot be written

- Report date/time and timezone: 2026-11-23, 12:21 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, change written 12:22 to 12:27; first check 12:27:29 to 12:27:30 (refused); second check 12:29:22 to 13:02:06 (accepted); watched again to 13:06:25 Europe/Berlin (from the commits and the commands' output)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R172](09-tests-from-the-remaining-faults.md); the state of the type from [R165](02-state-holder-count.md) to [R170](07-holders-read-while-the-tests-run.md)
- Type: preregistered

## Decision

The rework stands and the released flow accepts it, at the second attempt. The first was refused
in one second for size: 72 tokens more where I had allowed 60. The limit stayed; what the new code
did not need went. Two rows therefore hold only with that said: the change is two commits, not
one, and the acceptance is of the second. The state of the type goes from 16 ways to be written to
6, all 6 seen by the tests; the two faults about releasing the lock have no place left; the file
is 57 tokens larger and has two more functions. This is the first before-and-after on real code
with every lens filled, on one type.

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

Branch `lesscode/shared-frame-wake-tests` of `drift-rdp`, worktree
`D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`, on `09b4ce8`, local, not pushed:

| Commit | What |
| --- | --- |
| `08c6f9b`, `e4d3b2c` | the six tests (R171, R172) |
| `f58b017` | the declaration of the step: base `e4d3b2c`, growth allowed 60 tokens |
| `18c9043` | the rework |
| `e02154e` | what the new value did not need, dropped |
| `204782f` | the accepted verdict and the ledger entry |

Outputs in `D:\BuildArtifacts\lesscode\lesscode-metrics\r173-rework`: `verdict-1-refused.json`,
`check-1-refused.log`, `check.log`, `work\base-run.json` and `work\step-run.json` (every planted
fault on both sides), `holders-after.json`, `scope-after.json`, `watch`. The verdict itself is on the
branch. Working copies and build output are removed.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | One commit, one file, no test; 542 tests pass as they are | one file, no test, 542 passed and 6 skipped; but two commits, the second after the refusal | in part |
| 2 | `step check` accepts, growth up to 60 tokens | refused at 72; accepted at 57 after the second commit; `step record` and `step guard` accept | at the second attempt |
| 3 | Writable shapes of `SharedFrame` by the program: 8 to 3 | 8 to 3 (with `wake` by hand, 16 to 6) | yes |
| 4 | The tests see 6 shapes, and 6 is all that can be written | 6 of 6 | yes |
| 5 | At most 3 faults unnoticed after; the two about the lock have no place | 2 unnoticed; no fault of that kind is planted any more | yes |
| 6 | Functions and calls in the file | 15 to 17 functions, 8 to 11 call edges | reported |
| 7 | Size | 4,495 to 4,552 tokens, 3,359 to 3,411 syntax nodes | reported |

### The refusal

```
step shared-frame-pending: refused
  the scope grows from 4495 to 4567 tokens; the declaration allows 60 more
```

The four things that went in the second commit: three derives on the new value that nothing used;
a default that one `replace` makes unnecessary; a load of the version where the frame's own
`version()` does it; a match arm that returns the value it has. The tests, the formatting check and
the project's lints were run again before the second check.

### The six lenses, before and after

| Lens | Measure | Before | After |
| --- | --- | --- | --- |
| Size | tokens of the file | 4,495 | 4,552 |
| Structure | functions; call edges | 15; 8 | 17; 11 |
| State | ways the type can be written, with `wake` | 16 | 6 |
| | of them reached by the code | 12 | 6 |
| | of them seen by the tests | 12 (8 before the tests of R171) | 6 |
| | holders the shipped program sets and never reads | 1 (`dirty`) | 0 |
| Local complexity (`kiro-cli` 1.1.0) | `take_damage`: cyclomatic; lines | 1; 6 | 3; 8, within the limits; nothing blocks |
| | statements and lines over the routines it pairs | | 6 statements fewer, 2 lines more |
| Behaviour kept | the crate's library tests, unchanged | 542 pass | 542 pass |
| Protection | planted faults in the functions of the step: noticed; unnoticed; do not build | 37; 4; 1 | 33; 2; 2 |

Fewer faults can be planted after, 37 against 42: the flag and the two releases are gone. In no
function do more faults go unnoticed than before. The two still unnoticed are the two I read in
R172 as changing nothing: the fill value of the grown buffer, and the row just below the picture.

### What the watching shows after

(`wake`, pending) with pending as nothing, rectangle, whole: all six combinations, in 34, 13, 11,
6, 2 and 2 tests. Eleven changes, each a function with the shape before and after. The two shapes
that were passed through between the write and the mark are gone as shapes of the type: what is
pending no longer needs a second flag to agree with it. The moment itself is still there for a
caller that writes through `begin_write` and marks later; it is now "pending, not yet rung".

### All copies ran

Three copies on each side, none dropped out.

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
