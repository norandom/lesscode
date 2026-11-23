# R168: the generated walk over all 16 structs with holders in the renderer of drift-rdp

- Report date/time and timezone: 2026-11-23, 10:18 Europe/Berlin (registration, from the clock)
- Execution interval: 2026-11-23, 10:23:46 to 10:29:54 Europe/Berlin (from the run's own record); a first attempt from 10:20 to 10:23 is void
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md), the state lens (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R167](04-generated-walk.md), where the generator worked on two types
- Type: preregistered

## Decision

The four judged rows hold, and the result is as poor as expected: 3 of 16 types are walked, and only
one walk, `SharedFrame`, says anything. The other two run to the end without calling the method that
changes the type's state. As it stands the generator does not carry the state view for a subsystem.

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

Generator revision 2, SHA-256 `563A1827…926CBDC2`; rules `28FD6FF8…`. Outputs in
`D:\BuildArtifacts\lesscode\lesscode-metrics\r168-renderer-walks-2`: `run.json` with one row per type,
`logs` (generator reports and errors, build logs, run logs), `modules` (what was generated), `reached`
(results). The unchanged copy built in 185 seconds. Build output removed after the run.

**A void first attempt.** My start script kept the generator's report in a variable `$gen`, which
PowerShell does not tell from the script's parameter `$Gen`, the path of the generator. After the
first generated type the path was overwritten and the script stopped, five types in. Nothing is
drawn from that attempt; its logs are in `…\r168-renderer-walks`. The variable was renamed and the
whole run repeated in a fresh copy. Generator, rules and expectations were not touched.

| Type | Outcome | Why, or what it gave |
| --- | --- | --- |
| `Composition` | not generated | no way found to build a value |
| `Damage` | not generated | no way found to build a value |
| `DisplayPathInfo` | not generated | no way found to build a value |
| `Facts` | not generated | no way found to build a value |
| `FrameStores` | does not compile | the rules guess `Default::default()` for a parameter of type `&Arc<SharedFrame>` |
| `Layer` | not generated | no way found to build a value |
| `LayerTarget` | not generated | no way found to build a value |
| `Params` | not generated | no value for `&View`, which its constructor takes |
| `RenderShared` | walked | 5 calls of 3 methods, 625 sequences, 1 shape; `set_view`, the one method that changes it, was not called: no value for `View` |
| `Renderer` | not generated | no value for `&LayerTarget`, which its constructor takes |
| `SharedFrame` | walked | 46 calls, 4,477,456 sequences, 8, 12 and 12 shapes with `wake`; 4, 6 and 6 without |
| `SharedFrameInner` | not generated | no way found to build a value |
| `State` | does not compile | the type is not in scope where the module is declared (a nested, platform-only module) |
| `View` | walked, no calls | built by `Default`; it has no methods |
| `Wake` | walked | 1 call (`notify`), no holder read; `wait` was not called: no value for `Duration` |
| `YuvFrame` | not generated | no way found to build a value |

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | 2 to 5 of 16 walked | 3 | yes |
| 2 | at least 6 not generated | 10 | yes |
| 3 | at least 2 do not compile, for a guessed default or a nested or platform-only module | 2, one for each reason | yes |
| 4 | `SharedFrame` as in R167 | 12 shapes with `wake`, 6 without | yes |
| 5 | `Wake`: not judged | generated and walked; the method that waits was left uncalled, so nothing stopped | reported |

Row 1 holds by its letter and overstates the result. Of the three walked types only `SharedFrame`
had its state changed by the calls. Counted by "the walk changed at least one holder": 1 of 16.

No walk had to be stopped, and no call wrote outside the copy that I can see: `drift-rdp` shows
only untracked files under `.kiro` that are not mine.

## Why the others fail, grouped

| Reason | Types | What would be needed |
| --- | --- | --- |
| A record that is built by writing its fields, with no function that returns it | `Damage`, `Facts`, `DisplayPathInfo`, `YuvFrame`, `SharedFrameInner`, `LayerTarget`, `Layer`, `Composition` (8) | building a value field by field; the module sits inside the file and may do that |
| A constructor or method takes another type of the subsystem | `Params`, `Renderer`, `RenderShared` (`set_view`), `FrameStores` (4) | values for types of the scope, built the same way, and passing them by reference |
| A type from the standard library without an entry | `Wake` (`Duration`) (1) | more entries in the table |
| The type sits in a nested or platform-only module | `State` (1) | declaring the module where the type is; on Windows this one cannot be compiled at all |

A limit of the idea, not of the generator: most of the eight records have no methods of their own.
Their state is changed by free functions and by other types. `Facts`, the type with the most
writable shapes, is filled by a function `collect`. A walk over a type's own methods has nothing to
call there; the unit to walk would be the functions that take the type.

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
