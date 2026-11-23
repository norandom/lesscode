# R172: tests written from the planted faults the tests of `shared_frame.rs` still miss

- Report date/time and timezone: 2026-11-23, 11:46 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, tests written and run 11:56 to 11:57; planted faults 11:57:54 to 12:14:57 Europe/Berlin (from the commits and the run's output)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R171](08-tests-for-four-uncovered-shapes.md) (tests from uncovered shapes), [R164](01-renderer-baseline-faults.md) (the faults)
- Type: preregistered

Registered while the fault run of R171 is still under way. It assumes what R171 expects and has
not yet shown: that the four shape tests notice the dropped `mark_dirty` in `update_rect`. If R171
shows otherwise, that fault joins the list here and this record is amended before any test is
written.

## Decision

All five rows hold. Two tests written from the faults notice three of the seven; the tests of
`shared_frame.rs` now notice 38 of 42 planted faults, where they noticed 34. The four left are, by
my reading, two that change nothing and two that change only when the lock is released. No run
shows that reading; a run can only show that these tests do not notice them.

## The seven faults, read one by one

Numbers are those of the run of R164. Lines are those of `shared_frame.rs` at `09b4ce8`.

| Fault | Where | What is planted | What it would change |
| --- | --- | --- | --- |
| 198 | line 93 | `region.x0 < region.x1` becomes `<=` | a rectangle without width, with rows inside the picture, would count as damage |
| 199 | line 93 | `&&` becomes `\|\|` | a rectangle without width or without height would count as damage |
| 202 | line 100 | `dy >= fh` becomes `dy > fh` | one more turn of the loop for the row just below the picture; the check on the buffer two lines later refuses that row anyway |
| 207 | line 127 | the check for too little data changes place with the statement after it | with too little data and another size, the picture would be resized before the call gives up |
| 217 | line 133 | the buffer is grown with 1 in place of 0 | nothing: every byte of the grown buffer is overwritten in the next statement |
| 228 | line 199 | the lock is not released before `mark_dirty` in `update_full` | the render thread is rung while the lock is still held |
| 232 | line 210 | the same in `update_rect` | the same |

## Hypothesis and falsification

Written before any test is written.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | Two tests notice three faults: one test for rectangles without width or height (198 and 199), one for a short full update at another size (207) | fewer than three of these are noticed |
| 2 | 202 and 217 stay unnoticed, because they change nothing a caller can see: no test can notice them | one is noticed |
| 3 | 228 and 232 stay unnoticed by these tests: what they change is when the lock is released, which nothing in the type's interface shows on one thread | one is noticed |
| 4 | The new tests pass on the code as it is; the suite has 542 passing tests | a test of mine fails on unchanged code, or another count |
| 5 | No fault that was noticed before goes unnoticed | one does |

So I expect three of seven. The other four are of two kinds, and telling them apart is the result:
two that are the same program written differently, and two that are a real difference no test of
this type can see without a second thread and a way to look at the lock.

## Frozen inputs and method

- The branch `lesscode/shared-frame-wake-tests` of `drift-rdp` in its worktree, after the commit of
  R171. One more commit, tests only. Not pushed.
- Faults: `lesscode-core` 2.1.0 `step plan` with the configuration of R171.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r172-fault-tests`.

## Evidence and results

The tests: second commit on the branch `lesscode/shared-frame-wake-tests` of `drift-rdp`, `e4d3b2c`,
tests only, `cargo fmt --check` clean on the file. Local, not pushed. Tool: `lesscode-core` built
from this repository at `a60f426` (the sources of the tool are those of `4b213d4`), SHA-256
`FDD0C660…B36FDA74`. Outputs in `D:\BuildArtifacts\lesscode\lesscode-metrics\r172-fault-tests`:
`plan.json`, `plan.log`, `work\plan-run.json`. The three working copies and the worktree's build
output are removed.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | Three faults noticed: 198, 199, 207 | those three | yes |
| 2 | 202 and 217 stay unnoticed | both unnoticed | yes |
| 3 | 228 and 232 stay unnoticed | both unnoticed | yes |
| 4 | The new tests pass on the code as it is; 542 passing tests | 542 passed, 6 skipped | yes |
| 5 | No fault noticed before goes unnoticed | 35 noticed before, 38 now, of 42 | yes |

The file over the three steps, 42 planted faults each time:

| Tests | Noticed | Unnoticed | Shapes of `SharedFrame` the tests see, of 12 reached |
| --- | --- | --- | --- |
| as found (R164, R170) | 34 | 8 | 8 |
| with four tests from uncovered shapes (R171) | 35 | 7 | 12 |
| with two tests from the faults (this) | 38 | 4 | not read again |

### The four that are left

- 202 and 217: I read them as the same program written differently. For 202 the row just below
  the picture is refused by the check on the buffer two lines on; for 217 every byte of the grown
  buffer is overwritten in the next statement.
- 228 and 232: without the explicit release, the lock is held until the function returns, so the
  render thread is rung while the writer still holds it. Results are the same; the order of two
  events between two threads is not. Nothing in the type's interface lets a test on one thread see
  that. The rule "release, then ring" is in the code as two `drop(guard)` lines and is checked by
  nothing.

### The copies

All three copies ran this time, 14 faults each, and closed with the unchanged tests passing; the
run took 17 minutes. So this run says nothing about why copies dropped out in R164 and R171. That
stays unexplained.

## Limits and interpretation

- Tests by hand, by the author of the hypothesis, who has read the faults.
- "No test can notice it" is my reading of four faults, argued in the table above. It is not shown
  by a run; a run can only show that my tests do not.

## Tool/spec consequence and next gate

If the rows hold, a planted fault that stays unnoticed has three possible readings, and a tool that
drives tests from faults has to keep them apart: a test is missing; the fault changes nothing; the
fault changes something the interface does not show. Only the first is a call for a test. The
third is a note for the rework: the release of the lock before the ring is a rule of this type that
nothing checks.

## COBESY check

Each fault is read and its expected fate written down before a test exists; the dependence on an
unfinished run is stated at the top.

## Amendment, 2026-11-23, 11:55 Europe/Berlin: before any test is written

- R171 is in: the shape tests notice the dropped `mark_dirty`. The list of seven stands.
- The tool for this run is not release 2.1.0 but a build of `lesscode-core` from this repository at
  `9cc86fe`. Reason: in R171 two of three copies dropped out at the start because the unchanged
  tests did not pass in them, and the release keeps no output of that. The later build names the
  step that failed and the end of its output. The planted faults and their classes are produced by
  the same code in both.
