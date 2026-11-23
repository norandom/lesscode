# R170: the holders of a subsystem, read while the project's own tests run

- Report date/time and timezone: 2026-11-23, 11:08 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R169](06-walk-over-a-subsystems-functions.md): a walk cannot reach state that is changed from outside the subsystem
- Type: preregistered

## Decision

Open.

## Why

A generated walk supplies its own callers, and in the renderer of `drift-rdp` it could change the
state of one type in sixteen. The project's own tests are callers that already exist and that
build what a walk cannot. The owner's direction: watch the tests, read the holders while they run.

## What is built

A research program in Rust, `scope-watch`. In a copy of the project it

- adds one statement, compiled in test builds only, at the start of each function that takes a
  struct of the subsystem by reference, as its receiver or as a parameter. The statement notes
  what the holders of that value read when the function is entered and again when it is left;
- writes a module inside each file that knows how to read each type's holders, and one module that
  collects what was noted;
- reads a holder behind a lock only if the lock is free at that moment. A lock that is held gives
  "not read", it is never waited for; so the tests cannot be made to hang by the reading.

The copy's tests then run as the project runs them. Each new combination of type, function, shape
before and shape after is written once per test process, with the name of the test. A second
command of the program collects these into one report: per type the shapes seen, in how many tests,
and per function the changes it made.

Nothing is called that the tests do not call. Only structs with at least one holder are watched.
Left out: functions that are generic, asynchronous, unsafe, compiled under a condition, inside a
module written in a file, or that implement a trait; and values taken whole instead of by
reference.

## Hypothesis and falsification

Written before the program exists.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | The copy with the added statements compiles without a hand edit after at most 3 general revisions of the program | a hand edit, a change for a named type or function, or more than 3 |
| 2 | The tests give the same result with the statements as without: 536 pass, 6 are skipped | another count, or a test that does not end |
| 3 | Of the 16 structs with holders, 5 to 9 are seen in at least one test | fewer than 5 or more than 9 |
| 4 | `SharedFrame`: the tests see 4 to 6 of the 8 shapes of `dirty`, `damage` and `full`, all among the six the walks reached; the two with `full` set and no `damage` are not seen | a shape outside the six, or fewer than 4 |
| 5 | `Facts`: the tests see at least 4 different shapes (the walk had one, made up) | fewer than 4 |
| 6 | Readings that found a lock held | not judged; counted |

## What the result will and will not mean

A shape seen in a test is a shape the code was in while a test ran. For a type the code itself
builds and changes, such as `SharedFrame`, that is a reached shape. For a record that a test
writes by hand and passes in, such as `Facts`, it is the shape the test chose; it shows what the
tests exercise, not what the program gets into. The report keeps the two apart by naming the
function at which a shape was seen.

A shape not seen is not shown to be unreachable. It is only not covered by these tests.

## Risks

The added statements run inside the project's code while its tests run. They read through a
pointer kept from the start of the function to its end, which is sound only because values taken
by reference outlive the function; values taken whole are left out for that reason. They add a
little time and a shared set behind a lock. If a test's result changes with them, row 2 fails and
the readings are not trusted.

## Frozen inputs and method

- `drift-rdp` at `09b4ce8`, copied without `.git`, `node_modules` and `target`; the seven files of
  `src-tauri/src/renderer` that hold code are the scope. Functions elsewhere in the crate are not
  changed; their tests still run and reach the watched functions through calls.
- Program and rules: `docs/experiments/lesscode-metrics/scope-watch/`.
- Tests: `cargo nextest run --locked --manifest-path src-tauri/Cargo.toml --lib --no-fail-fast`, the
  command of R164. Reference without the statements: 536 passed, 6 skipped (R164, by hand, same
  commit).
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r170-scope-watch`.
- Windows.

## Evidence and results

Not run yet.

## Limits and interpretation

- The readings of one value are taken one holder at a time; with another thread at work two
  holders may be read at different moments.
- Types that exist only on another platform, and code the tests do not reach, are not seen.
- Only the renderer is watched; a type's shape is not read inside functions of other parts of the
  crate.

## Tool/spec consequence and next gate

If the rows hold, the state view of the page can show, per type, three sets: the shapes that can
be written (R165), the shapes the tests cover (this), and, where a walk is possible, the shapes
reached (R166 to R169). The difference between the first and the second is where tests are
missing, which is what the owner's flow of faults, tests and reduction starts from.

## COBESY check

Expectations and their rejection precede the program; what a seen shape means for a hand-written
record is said before the result; the risk to the tests themselves is a registered row.
