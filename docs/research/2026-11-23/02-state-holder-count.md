# R165: a count of state holders and writable shapes, in Rust

- Report date/time and timezone: 2026-11-23, 09:27 Europe/Berlin (registration; the time of commit `e6c23db`. I first wrote 10:25 here without reading the clock; corrected at 09:30, from the clock)
- Execution interval: 2026-11-23, 09:28 (build start, from the build output) to before 09:30 (last run; the clock read 09:30 after it), Europe/Berlin
- Status: completed for the six registered rows; exploratory for the renderer
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: the hand counts in [the kiro-cli concept note](../../../.kiro/specs/state-consolidation/kiro-cli-state-metrics-concept.md)
- Type: preregistered for the three known examples; exploratory for the renderer of drift-rdp

## Decision

All six registered rows hold. The count of holders and of writable shapes can go into the state
view for the renderer. The lists of writers and deciders are not good enough yet: on the renderer 73
uses of a field were left out because more than one type has a field of that name.

A prototype, written while the fault run of [R164](01-renderer-baseline-faults.md) was under way.
It is a research program, not a released command.

## What is counted

For each struct and enum declared in the given files, outside test modules:

- **Holder**: a field that keeps part of the standing of the program. Kinds: flag (`bool`,
  `Option`), cell (lock, atomic, interior-mutable type), enumeration (a field whose type is an enum
  of the scope), compound (a struct of the scope that itself has more than one shape).
- **Shape**: the kind of a value, not its content. `bool` has two, `Option<T>` one more than `T`,
  an enum one per variant, a wrapper or a cell as many as what it holds, anything else one.
- **Writable shapes** of a type: the product over its fields. This is the count of states that can
  be written, not of states that are reached.
- Per holder, the functions that write it and the functions that test it in a condition. A use is
  tied to its type through `self.` inside an implementation of that type, or through the field's
  name when only one type of the scope has it; other uses are counted as left out.

Which syntax means what is in a rules file; the program names no language.

## Hypothesis and falsification

Written before the program was built or run. The numbers are the hand counts of 18 November.

| Input | Expectation | Rejected when |
| --- | --- | --- |
| `docs/examples/order-flags/before.rs` | `Order`: 3 flags, 8 writable shapes | any other number |
| `docs/examples/order-flags/after.rs` | `Order` is an enumeration with 5 shapes; no flags | any other number |
| `docs/examples/clipboard-lease/before.rs` | `NativeClipboardState`: 3 cells; `Lease`: 1 cell | any other number |
| `docs/examples/clipboard-lease/after.rs` | `NativeClipboardState`: 2 cells; `Lease`: 0 cells | any other number |
| `runner.rs` of this repository at `6da5952` | `Shared`: 3 cells | any other number |
| `runner.rs` at `6b580a5` | `Shared`: 1 cell | any other number |

Not predicted: the writable shapes of the clipboard and runner types, and everything about the
renderer of `drift-rdp`. The research finder reported 13 cells there, but it also counts locals and
statics, which this program does not look at.

If a row fails, the cause is named and the program or the hand count is corrected with a dated
note; a row is not dropped.

## Frozen inputs and method

- Program and rules: [main.rs](../../experiments/lesscode-metrics/state-holders/main.rs),
  [rust.holders.json](../../experiments/lesscode-metrics/state-holders/rust.holders.json); a binary
  of the experiment crate `docs/experiments/state-consolidation/e5-bound`, built on `lesscode-core`'s
  parser.
- Inputs as named in the table; `drift-rdp` at `09b4ce8`, the eight files of `src-tauri/src/renderer`.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r165-state-holders`.

## Evidence and results

The program built at the first attempt (the experiment crate's lock file moved `lesscode-core` from
2.0.0 to 2.1.0; built with `--offline`, since `--locked` refused that). Binary SHA-256 `F9333723…BC136D43`,
kept with all outputs in `D:\BuildArtifacts\lesscode\lesscode-metrics\r165-state-holders`.

| Input | Expected | Counted | Holds |
| --- | --- | --- | --- |
| order flags, before | `Order`: 3 flags, 8 shapes | 3 flags, 8 shapes | yes |
| order flags, after | `Order`: enumeration, 5 shapes, no flags | enumeration, 5 shapes, no flags | yes |
| clipboard lease, before | `NativeClipboardState` 3 cells; `Lease` 1 cell | 3 cells; 1 cell | yes |
| clipboard lease, after | `NativeClipboardState` 2 cells; `Lease` 0 cells | 2 cells; `Lease` has no holder | yes |
| `runner.rs` at `6da5952` | `Shared` 3 cells | 3 cells | yes |
| `runner.rs` at `6b580a5` | `Shared` 1 cell | 1 cell | yes |

Not predicted, and read off the same outputs:

- Writable shapes fall in two of the three examples and stay in one. Order: 8 to 5. Clipboard
  lease: `NativeClipboardState` 3 to 2, and the 2 of `Lease` are gone. Batch runner: `Shared` has 4
  before and 4 after; the step put three locks behind one and removed no shape.
- In `runner.rs` 11 uses of a field were left out as ambiguous, in both revisions.

The renderer of `drift-rdp` at `09b4ce8`, eight files, exploratory (`renderer.json`):

- 38 types declared outside test modules; 16 structs have holders; 35 holders, of them 16 flags and
  11 cells; the rest are fields of an enumeration or compound type.
- Largest counts of writable shapes: `Facts` 216 (three enumerations of three and three flags),
  `Renderer` 16, `Layer` 8, `SharedFrame` 8.
- 73 uses of a field were left out as ambiguous. For many holders the lists of writers and
  deciders are therefore empty or short; they are not to be read as "never written".

## Weaknesses found

- Tying a use to its type by the field's name fails where names repeat (`full`, `geometry`,
  `smooth` in the renderer). The lists of writers and deciders need the type of the receiver.
- A lock taken to read counts as a write (`lock` is in the list of writing methods).
- A field given once when the value is built and never changed is counted as a holder. `Facts` is
  such a case: it looks like a record of findings, not a machine. Without telling "set once" from
  "changed", 216 overstates it.
- Setting a field in a struct literal is not counted as a write.

## Limits and interpretation

- Locals, statics and globals are not looked at. A shared local between two closures, as in the
  reply side channel, is invisible to this count.
- A collection that is empty or not is counted as one shape, though code often decides on it.
- Writable is not reached. Which shapes the code can get into needs a solver or a walk over
  sequences of events.
- Uses are tied to a type by name, not by type inference.

## Tool/spec consequence and next gate

If the six rows hold: the count goes into the state view of the page for the renderer, marked as
writable shapes only. The next gate is the count of reached shapes.

## COBESY check

Expectations written before the build; what is not predicted is said; limits are listed.
