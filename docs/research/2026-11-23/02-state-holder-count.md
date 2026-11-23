# R165: a count of state holders and writable shapes, in Rust

- Report date/time and timezone: 2026-11-23, 10:25 Europe/Berlin (registration)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: the hand counts in [the kiro-cli concept note](../../../.kiro/specs/state-consolidation/kiro-cli-state-metrics-concept.md)
- Type: preregistered for the three known examples; exploratory for the renderer of drift-rdp

## Decision

Open. A prototype, written while the fault run of [R164](01-renderer-baseline-faults.md) is under
way. It is a research program, not a released command.

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

Not run yet.

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
