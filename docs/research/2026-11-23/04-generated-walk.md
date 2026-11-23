# R167: the walk over a type's methods, generated from its source

- Report date/time and timezone: 2026-11-23, 10:08 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, 10:11:07 to 10:14:44 Europe/Berlin (from the run's own record)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R166](03-reached-shapes-walk.md), where the walk was written by hand
- Type: preregistered

## Decision

All six rows hold, on the first run and with nothing generated touched by hand. The reached shapes
of a type can be measured without writing a walk for it. Two subjects, one of them made up: this is
a prototype that works twice, not a tool.

## What is built

A research program in Rust, `walk-gen`. From a source file, the name of a type and a rules file it
writes a Rust test module that:

- builds a value of the type, through an associated function that returns it (one named `new`
  first), or through `Default` when the type derives it;
- reads the holders of the type as a list of small numbers: a flag, an optional, a cell that is
  filled once, an enumeration by its variant, and the same through a lock or a wrapper into a
  struct of the same file;
- calls every method of the type that takes its receiver by reference, with values for the
  parameters taken from a small table in the rules file (numbers 0, 1 and 4; an empty and a filled
  slice; and so on). One call uses the middle value for every parameter; further calls change one
  parameter at a time;
- where a method returns another type of the file (a guard, say), also calls that type's methods on
  what was returned, in one expression;
- keeps methods that exist only in test builds apart.

The generated module runs every sequence of these calls up to a length on a fresh value, and
writes the shapes it saw, the shortest way to each, and for each holder the calls that changed it.

It names no project. Which syntax means what, and which values to try, is in the rules file.

## Hypothesis and falsification

Written before the generator exists. "Unedited" means the generated file is compiled and run as
written; if it has to be touched by hand, the row fails.

| # | Subject | Expectation | Rejected when |
| --- | --- | --- | --- |
| 1 | `Order` in `docs/examples/order-flags/before.rs` (made up; changed through `&mut self`, built by `Default`) | compiles and runs unedited; reaches 5 of 8 shapes of (`paid`, `shipped`, `cancelled`): none set; paid; paid and shipped; cancelled; paid and cancelled | otherwise |
| 2 | `Order` | `paid` is changed only by `pay`, `shipped` only by `ship`, `cancelled` only by `cancel` | otherwise |
| 3 | `SharedFrame` in `drift-rdp` at `09b4ce8` (real; shared behind `Arc`, built by `new`) | compiles and runs unedited; its own methods reach the 4 shapes R166 found for the whole calls, with the guard's methods the 6, and the test-only method adds none; twice as many with `wake` | otherwise |
| 4 | `SharedFrame` | the holders read are `wake`, `dirty`, and behind the lock `damage` and `full`; the counter `version` is not one | otherwise |
| 5 | `SharedFrame` | outside the test-only set no call takes `dirty` from set to not set | such a call is listed |
| 6 | both | no method that takes its receiver by reference is left uncalled | one is left out |

## Frozen inputs and method

- Generator and rules: `docs/experiments/lesscode-metrics/walk-gen/` (`main.rs`, `rust.walk.json`),
  a binary of the experiment crate `e5-bound`.
- `Order`: the file is copied into a throwaway crate as its library root.
- `SharedFrame`: `drift-rdp` is copied as in R166, and the generated module is placed beside
  `shared_frame.rs` in the copy, with one line added to declare it. The project is only read.
- Sequences up to length 4.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r167-generated-walk`.

## Evidence and results

Generator SHA-256 `B7B2096D…104BF1AF`, rules `48F22289…CDCCEA8B`. Outputs in
`D:\BuildArtifacts\lesscode\lesscode-metrics\r167-generated-walk`: `run.json`, and for each subject
the generator's report (`*-gen.json`), the generated module as it was compiled (`*-state_walk.rs`),
the test log and the result (`*-reached.json`). The build output of the copy, 4.3 GiB, was removed
at 10:15.

The generator built at the first attempt with one warning (a function I had written and not used;
removed before the run). Both generated modules compiled and ran as written.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | `Order`: runs unedited; 5 of 8 shapes: none set; paid; paid and shipped; cancelled; paid and cancelled | those 5; 256 sequences | yes |
| 2 | `Order`: each flag changed by one method | `paid` by `pay`, `shipped` by `ship`, `cancelled` by `cancel` | yes |
| 3 | `SharedFrame`: runs unedited; own methods 4, with the guard's methods 6, test-only adds none; twice as many with `wake` | 4, 6, 6; with `wake` 8, 12, 12; the same shapes as R166 | yes |
| 4 | Holders read: `wake`, `dirty`, `damage`, `full`; not `version` | `wake`, `inner.damage`, `inner.full`, `dirty` | yes |
| 5 | Outside the test-only set nothing clears `dirty` | read lower only after `publish` | yes |
| 6 | No by-reference method left uncalled | none left out: 4 of `Order`; 10 of `SharedFrame` and 4 of `WriteGuard` | yes |

`SharedFrame` was built by `SharedFrame::new(4, 4)`. 25 calls of its own methods, 20 of the guard's
methods on what `begin_write` returns, 1 test-only; 4,477,456 sequences of length 4 in the largest
set, 33 seconds for the three sets in a debug build. Nothing new after two calls in any set.

What the walk adds over R166, because it records it for every holder: the calls after which a
holder read differently, as measured.

| Holder | Changed by | Read lower after |
| --- | --- | --- |
| `wake` | `set_wake` | nothing |
| `damage` | `update_rect`, `update_full`, `take_damage`, and the guard's `update_rect` and `update_full` | `take_damage` |
| `full` | `update_full`, `take_damage`, the guard's `update_full` | `take_damage` |
| `dirty` | `mark_dirty`, `update_rect`, `update_full`; in test builds also `publish` | only `publish` |

This agrees with what the static count of R165 listed as changers, and settles two of its "none
found": it names who changes a holder from what the code did.

One difference from the hand-written walk: there the shortest way to "dirty with nothing pending"
was `update_rect` with an empty rectangle; here it is `mark_dirty` alone, which the generator counts
among the type's own methods. The generator cannot know which public methods are meant as parts.

## Limits and interpretation

- The shapes reached depend on the values tried. A shape that needs a particular argument the
  table does not hold is missed. What is reported is a lower bound on what is reached.
- One thread.
- A parameter whose type has no entry in the table makes its method uncallable; the generator
  lists such methods.
- Two subjects are few. One of them is made up.

## Tool/spec consequence and next gate

If the rows hold: the reached shapes of a type can be measured without writing a walk by hand, and
the state view of the page can be filled for the changing types of the renderer.

## COBESY check

Expectations and their rejection are written before the generator exists; the limit that matters
most, the dependence on the values tried, is stated first.
