# R168: the generated walk over all 16 structs with holders in the renderer of drift-rdp

- Report date/time and timezone: 2026-11-23, 10:18 Europe/Berlin (registration, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R167](04-generated-walk.md), where the generator worked on two types
- Type: preregistered

## Decision

Open.

## Question

R167 showed the generator on one made-up type and one real type that I had read closely and whose
walk I had written by hand first. How many of the types of a real subsystem can it walk when
nothing is fitted to them?

The subjects are the 16 structs that [R165](02-state-holder-count.md) lists with holders in
`src-tauri/src/renderer` of `drift-rdp` at `09b4ce8`: `Composition`, `Damage`, `DisplayPathInfo`,
`Facts`, `FrameStores`, `Layer`, `LayerTarget`, `Params`, `RenderShared`, `Renderer`, `SharedFrame`,
`SharedFrameInner`, `State`, `View`, `Wake`, `YuvFrame`.

## What changes in the generator (revision 2), and why

Only what running many types in one copy needs. No rule and no value is added for any type.

- The module is named after the type, so that several fit beside one file.
- The result goes to a file named after the type in a directory.
- The length of the sequences is shortened for a type with many calls, so that the largest set
  stays under 5 million sequences (`SharedFrame` keeps length 4).

## Outcomes, each type gets exactly one

1. **Not generated**: the generator finds no way to build a value, or has no value for a
   constructor's parameter.
2. **Does not compile**: the module is generated and the project's test build fails with it. The
   module is taken out again.
3. **Does not finish**: it compiles, and the walk fails or is stopped after 180 seconds.
4. **Walked, no calls**: it runs to the end and has no method to call.
5. **Walked**: it runs to the end with at least one call.

Nothing generated is edited. A type is not helped along.

## Hypothesis and falsification

Written from what I have seen of these files while working on R165 to R167; I have read
`shared_frame.rs` closely and the others only in parts.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | Between 2 and 5 of the 16 are walked (outcome 5) | fewer than 2 or more than 5 |
| 2 | At least 6 are not generated (outcome 1): most are records without a constructor the generator can use | fewer than 6 |
| 3 | At least 2 do not compile (outcome 2), because the rules guess `Default::default()` for an `Arc` parameter and the type has no default, or because the type sits in a nested or platform-only module | fewer than 2 |
| 4 | `SharedFrame` is walked and gives what R167 gave: 12 shapes with `wake`, 6 without | otherwise |
| 5 | `Wake` is generated; if it is walked or stops on a call that waits, I do not know | not judged; reported |

Why so low: the generator builds a value only through a function that returns the type or through
`Default`, has values for numbers, flags, byte slices and strings only, and reads one file.

## Risks of running generated calls, named before the run

The walk calls real methods with made-up arguments. A method can wait for something that never
comes; hence the limit of 180 seconds per type, after which the test process is ended. The
renderer's methods that draw need a device or a window the generator has no value for, so I expect
them to be left uncalled, not called. The run happens in a copy; if a call writes outside the copy,
that is a finding and is reported.

## Frozen inputs and method

- Generator revision 2 and rules in `docs/experiments/lesscode-metrics/walk-gen/`; start script
  `run-renderer.ps1` there.
- `drift-rdp` at `09b4ce8`, copied without `.git`, `node_modules` and `target`.
- Per type: generate; declare the module in the copy; build the tests (`cargo test --lib --no-run`);
  on failure take the module out and keep the error. Then run each remaining walk by itself.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r168-renderer-walks`.

## Evidence and results

Not run yet.

## Limits and interpretation

- An outcome other than "walked" says what the generator cannot do, not that the type has no
  state machine.
- The reached shapes are a lower bound; they depend on the values tried.
- One thread, one platform (Windows). Types that exist only on Linux cannot be compiled here.

## Tool/spec consequence and next gate

The share that is walked says how far a generated walk carries the state view of the page for a
real subsystem, and the reasons for the rest say what to build next: values for more parameter
types, constructors through other types, or reading across files.

## COBESY check

The expected result is poor and is written down as such; outcomes are defined before the run; the
risk of running generated calls is named.
