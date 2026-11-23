# R169: a walk over the functions of a subsystem, with the shapes of its types read along the way

- Report date/time and timezone: 2026-11-23, 10:33 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, three runs: 10:40:20 to 10:45:29, 10:47:40 to 10:52:11, 10:53:19 to 10:57:49 Europe/Berlin (from each run's own record)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R168](05-generated-walk-over-renderer.md): a walk over one type's own methods says something for 1 of 16 types
- Type: preregistered

## Decision

The walk works as a program and does not answer the question it was built for. It compiled
unedited at the first attempt, called 53 of 74 functions without a panic or a call that did not
return, and found `SharedFrame`'s 12 shapes again. But the count that matters stays where it was:
one of the 16 structs has its state changed by the walk. Changing the unit from a type's methods
to the subsystem's functions did not help, because this subsystem's state is changed from outside
it or sits behind a device. Reached shapes for a subsystem have to come from somewhere else.

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

Three runs, each as generated, with no hand edit. Outputs in
`D:\BuildArtifacts\lesscode\lesscode-metrics\r169-scope-walk\try1`, `try2`, `try3`: `run.json`,
`gen.json` (what the generator read, kept, called and left out), `modules` (what was generated),
`build.log`, `run-1.out`, `reached\scope.json`, `reached\probe.log`. Generators kept in
`…\r169-bin\rev1` to `rev3`. One copy of `drift-rdp` served all three runs, its renderer sources put
back before each; its build output is removed. The rows are judged on the third run.

### The revisions, all general

| Revision | SHA-256 | Change | Why |
| --- | --- | --- | --- |
| 1 | `2D676978…` | as registered, with three fixes made after reading the generator's own report on a scratch copy and before anything was compiled: a lifetime is no type argument; a method of an enumeration is called on its plain variants; pairs of numbers get values | the report listed a guard's methods and an enumeration's methods as uncallable |
| 2 | `FC564E27…` | shorter sequences first; a new value in the world is told from a change of the value it replaces; the time of each call is recorded; time limit 240 seconds | run 1 used its 120 seconds on 15,950 of 1.6 million sequences in counting order, so only sequences beginning with the first two calls ran; and `SharedFrame::new` was listed as changing every holder |
| 3 | `E93DBCC0…` | a value is built with the largest values of the table | run 2 built a 1 by 1 picture, in which the rectangle the walk tries lies outside; R167 had this rule and I had left it out |

None names a type or a function of the subject.

### The rows, on run 3

74 functions read outside tests; 53 called, in 118 distinct calls. Every sequence of one and of two
calls ran, and part of those of three: 30,313 sequences in 240 seconds. No call panicked, alone or
in a sequence, and none failed to return.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | Compiles and runs unedited within 5 general revisions | unedited at revision 1; three revisions in all | yes |
| 2 | 2 to 4 of the 16 have a holder changed by the walk | 1: `SharedFrame` | **no** |
| 3 | At least 8 of the 16 have a value in the world | 9 | yes |
| 4 | 25% to 50% of the functions are called | 72% | **no**, higher |
| 5 | `SharedFrame`: the 12 shapes of R167, 6 without `wake` | 12, the same 6 without `wake` | yes |
| 6 | Panics and calls that do not return | none of either | reported |

### The 16 structs

| Type | In the world | How it got there | State changed by a call |
| --- | --- | --- | --- |
| `SharedFrame` | yes | its own function | yes, all four holders |
| `RenderShared` | yes | its own function | no |
| `View` | yes | handed out by `RenderShared` | no |
| `FrameStores` | yes | `Default` | no holder is read |
| `Wake` | yes | a function | no holder is read |
| `DisplayPathInfo` | yes | handed out by another type | no; only new values appear (2 shapes) |
| `Damage` | yes | handed out by `SharedFrame` | no; only new values appear (2 shapes) |
| `Facts` | yes | made up by writing its fields | no |
| `SharedFrameInner` | yes | made up by writing its fields | no |
| `Params`, `Renderer` | no | their constructors need a value the world never holds | |
| `LayerTarget`, `Composition`, `YuvFrame` | no | nothing builds them without a window or a device | |
| `Layer`, `State` | no | compiled only on another platform | |

### Why the count stays at one

- `RenderShared::set_view` is called now, but the only `View` in the world is the one `RenderShared`
  handed out, so setting it changes nothing. The functions that put another view in take a handle of
  the application, which the walk has no value for.
- `Facts` and `SharedFrameInner` are read by the functions that take them and changed by none. On
  Windows nothing fills `Facts` after it is built; the function that does is compiled on Linux only.
- `DisplayPathInfo` and `Damage` are snapshots. They are made, never changed.
- Five types need a window, a device or another platform.

So the state of this subsystem is either changed from outside it, by callers the walk cannot reach,
or sits behind a device. That is a property of the subject and the walk states it; it is not
something more generator would fix.

### What the walk found beyond R167

`FrameStores::register` fills `wake` on the frame it is given. A walk over `SharedFrame`'s own
methods could not see a change made from another type; this one names it.

### A risk that came true, mildly

The slowest calls in run 2: `decide` up to 206 milliseconds, `try_gpu` 142 milliseconds, against
well under a millisecond for almost everything else. Both ask the graphics driver what it can do,
and the walk called them some hundreds of times. I saw no effect beyond the queries. The sources of
`drift-rdp` are unchanged after the runs. A walk of this kind does run code that talks to the system.

### Not read at all

`DisplayPathState` keeps its state as a pair inside a lock, in a struct without named fields. The
program reads named fields only, so it lists no holder for it, though `fall_back` changes it.

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
