# R166: which shapes of `SharedFrame` the code reaches, by a walk over its own methods

- Report date/time and timezone: 2026-11-23, 09:54 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R165](02-state-holder-count.md), which counts the shapes that can be written
- Type: preregistered

## Decision

Open.

## What is done

`SharedFrame` in the renderer of `drift-rdp` keeps four holders that change: the flag `dirty`, and
behind a lock the optional `damage` and the flag `full`; and `wake`, a cell that is filled once.
R165 counts 8 writable shapes without `wake`, 16 with it.

A walk runs the type's real methods, in every order up to a fixed length, on a fresh value each
time, and writes down the shape after every call. It is a Rust test placed inside a copy of the
project, as a child module of the file, so that it can read the private fields. The project itself
is not changed. Nothing is modelled: what is reached is what the compiled code did.

Three sets of calls, each a superset of the one before:

- **A, the whole calls**: `update_rect` with a rectangle inside the picture, `update_rect` with an
  empty one, `update_full` with enough data, `update_full` with too little, `take_damage`,
  `take_full`, `set_wake`.
- **B, with the parts**: also a write through `begin_write` that is not followed by `mark_dirty`
  (a rectangle, and a full update), and `mark_dirty` alone. All of these are public.
- **C, with the test-only call**: also `publish`, which exists only in test builds and is the only
  code that clears `dirty`.

## Hypothesis and falsification

Written after reading `shared_frame.rs` and before the walk exists. Shapes are (`dirty`, `damage`,
`full`); `wake` doubles each count.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | A reaches 4 of 8: nothing pending and not dirty; dirty with damage; dirty with damage and full; dirty with nothing pending | another number or another set |
| 2 | B reaches 6 of 8: every shape except the two with `full` set and no `damage` | another number or another set |
| 3 | C reaches the same 6 as B | another number or another set |
| 4 | In A and B no call takes `dirty` from set to not set | such a step is seen |
| 5 | With `wake`: A 8 of 16, B and C 12 of 16 | another number |
| 6 | The sets stop growing before the longest sequences: the last length adds no shape | the last length adds one |

Why 2: `update_full` sets `full` and `damage` together, and `take_damage` clears both or neither.
Why 4: outside test builds nothing clears `dirty`.

## Frozen inputs and method

- Subject: `drift-rdp` at `09b4ce8`, copied without `.git`, `node_modules` and `target` to
  `D:\BuildArtifacts\lesscode\lesscode-metrics\r166-reached-walk\tree`. One line is added to the
  copy of `shared_frame.rs` (`#[cfg(test)] mod state_walk;`) and one file beside it.
- Walk: [state_walk.rs](../../experiments/lesscode-metrics/reached-walk/state_walk.rs); start
  script [run.ps1](../../experiments/lesscode-metrics/reached-walk/run.ps1).
- Sequences up to length 5 for A and B and C; a 4 by 4 picture.
- One thread. What two threads can do between each other's calls is not walked.

## Evidence and results

Not run yet.

## Limits and interpretation

- A shape not reached in sequences up to length 5 on one thread is not shown to be unreachable.
  Expectation 6 says whether the sets had stopped growing.
- The walk is written by hand for this type. A tool would have to write it from the type's
  methods; that is not attempted here.
- Reached is not meant: a shape the code gets into can still be one nobody intended.

## Tool/spec consequence and next gate

If the rows hold: the grid on the page is lit for `SharedFrame`. The next question is whether a
walk can be generated for a type instead of written.

## COBESY check

Expectations and their rejection are written before the walk exists; the test-only call is kept
apart; limits are listed.
