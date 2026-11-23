# R169: a walk over the functions of a subsystem, with the shapes of its types read along the way

- Report date/time and timezone: 2026-11-23, 10:33 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R168](05-generated-walk-over-renderer.md): a walk over one type's own methods says something for 1 of 16 types
- Type: preregistered

## Decision

Open.

## Why the unit changes

R168 showed that half the types of the renderer are records without methods of their own; their
state is changed by free functions and by other types. And where a function needs a value of
another type of the subsystem, the walk over one type had none to give. The owner's decision: walk
the subsystem's functions.

## What is built

A research program in Rust, `scope-walk`. From the files of a subsystem and a rules file it writes
Rust test modules, one inside each file (so that it reaches private functions and fields) and one
that runs the walk. Together they:

- keep a **world**: at most one value of each struct of the subsystem;
- **build** values with the code's own functions that return a type, with `Default` where the type
  derives it, and, for a record nothing returns, by writing its fields once with fixed values. A
  value made up in that last way is marked as such: its starting shape was chosen by the walk, not
  reached by the code;
- **call** every function and method it can supply: numbers, flags, slices and strings from a small
  table, and values of the subsystem's own types from the world, by reference, by changeable
  reference or by value. A returned value of a subsystem type goes into the world. Where a method
  returns a guard that cannot be kept, the guard's methods are called on it at once;
- **read** after every call the holders of every value in the world;
- run every sequence of calls up to a length from a world in which everything buildable is built,
  and write, per type, the shapes seen and the shortest way to each, and per holder the calls that
  changed it.

Left out, and listed by the program with the reason: functions that are generic, asynchronous,
unsafe, compiled only on some platform or only in tests, or inside a module written inside a file;
and functions with a parameter it has no value for.

Each call is tried once first, by itself, with its name written to a log before it runs. A call
that panics is left out of the walk and listed. If a call does not return, the start script ends
the test after a time limit, takes the name from the log, and runs again without it; that call is
listed as not returning.

## Hypothesis and falsification

Written before the program exists. I expect the generated code not to compile at first.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | The generated modules compile and run without a hand edit after at most 5 revisions of the generator, each a general change listed in the record | a hand edit is needed, a change is made for one named type or function, or more than 5 |
| 2 | Of the 16 structs with holders, between 2 and 4 have a holder changed by the walk (R168: 1) | fewer than 2 or more than 4 |
| 3 | At least 8 of the 16 have a value in the world | fewer than 8 |
| 4 | Between 25% and 50% of the functions the program reads outside tests are called | outside that range |
| 5 | `SharedFrame` reaches the 12 shapes of R167 again, 6 without `wake`, and no others | otherwise |
| 6 | Calls that panic, and calls that do not return: not judged | reported with their names |

Why row 2 is low: the drawing code needs a graphics device or a window, which the walk has no
value for; and several types exist only on another platform.

## Risks, named before the run

The walk calls real functions with made-up arguments. A function may wait, start a thread, open a
window or ask the graphics driver for something. It runs in a copy and under a time limit. What a
call does outside the process is not contained by anything the walk does; if I see such an effect
I report it. Functions that need a device or a window handle cannot be supplied and are not called.

## Frozen inputs and method

- `drift-rdp` at `09b4ce8`, copied without `.git`, `node_modules` and `target`; the seven files of
  `src-tauri/src/renderer` that hold code.
- Generator and rules: `docs/experiments/lesscode-metrics/scope-walk/`.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r169-scope-walk`.
- One thread, Windows.

## Evidence and results

Not run yet.

## Limits and interpretation

- What is reached depends on the values tried and on what could be built; it is a lower bound.
- The world holds one value of each type. What two values of one type do to each other is not seen.
- A shape of a made-up record says nothing about what the code reaches, unless a call changed it.
- One thread.

## Tool/spec consequence and next gate

The count in row 2 and the list of what was not called say whether a generated walk can fill the
state view of the page for a subsystem, or whether reached shapes have to come from somewhere
else: from a solver over a model of the functions, or from watching the project's own tests run.

## COBESY check

The unit was changed for a stated reason; expectations and their rejection precede the program; the
expected count is low and said to be; risks of running generated calls are named.
