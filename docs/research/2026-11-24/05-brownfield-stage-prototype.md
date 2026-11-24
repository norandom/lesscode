# R184: the brownfield test stage by hand on display_path.rs: six of nine unnoticed faults noticed by four tests, no code changed

- Report date/time and timezone: 2026-11-24, 13:10 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, 11:47:14 (start of the plan) to 13:02 (end of the last pin); the owner's review at about 13:05
- Status: completed; the gate's decision is `proceed`, with one limit passed and accepted by the owner
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/design.md), its [prototype gate](../../../.kiro/specs/brownfield-test-stage/prototype-gate.md)
- Hypothesis ID and predecessor: [R181](02-run-page-a-real-run-watched.md) (where the owner asked for the stage); [R171](../2026-11-23/08-tests-for-four-uncovered-shapes.md), [R172](../2026-11-23/09-tests-from-the-remaining-faults.md) (tests from what a plan shows, on another file)
- Type: preregistered (the gate was written before the run and amended, dated, before any test)

## Decision

The stage can be built as the design says: a step that adds tests and changes no code, carried by
`step plan`, `step pin` and `step check`. On five functions of `display_path.rs` in the renderer of
drift-rdp, four tests turned six of nine unnoticed faults into noticed ones, and the owner wrote
the other three off. Rust needs one thing the design did not have: a test file of its own, joined
by one line that is committed before the plan.

This permits the design to go to the owner for approval. It does not show the two commands and the
skill, which do not exist; everything they would do was done by hand here.

## Expectations and what was measured

Written before the run in the gate; G7 was changed at 12:10, before any test, when the owner chose
one review at the end.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| G1 | `step pin` and `step check` judge with a declaration written by hand | seven pins and one check ended with 0 or 1; one call ended with 3 because I asked for a fault number that is not an unnoticed fault | yes |
| G2 | a test of these functions can be admitted in Rust | placement (a): `display_path/protect.rs`, joined by `#[cfg(test)] mod protect;` in the base. Placement (b) ruled out by reading: `mod renderer` is private and the test command runs `--lib` | yes, for (a) |
| G3 | at least half of the unnoticed faults built on Windows end noticed; every other has a proposal | 6 of 9 noticed; 3 proposals, approved by the owner | yes |
| G4 | the branch changes tests and the stage spec only | `git diff --name-only 72507e1..protect/display-path`: five files of the stage spec and `protect.rs`; the scope file has 3,095 tokens before and after | yes |
| G5 | two adverse controls are rejected | "the tests with the pin pass on the fault"; "the tests with the pin fail on the unchanged code" | yes |
| G6 | at most 3 hours; no pin over 10 minutes | 1 h 15 min; the first pin 17 min 50 s, the others 93, 90, 90, 91 and 143 seconds | **no**, for the first pin |
| G7 | one review at the end; nothing merged before it | the owner accepted the four tests and approved the three write-offs in one review | yes |
| G8 | not more tests than faults noticed | 4 tests, 6 faults | yes |

**G6.** The first `step pin` of a stage runs the faults of the chosen functions on the base. I had
not allowed for that in the limit. The gate said to stop the run when a limit was passed; I did
not, and saw it only when the pin had ended. By the gate as written the decision was `revise`. The
owner was given three ways to treat it and chose "Accept, and say it up front": the design now
runs the base once as a step of its own and says beforehand how long it takes.

## What was run

- Subject: drift-rdp at `09b4ce8`, cloned to `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`.
  The checkout at `D:\Source\drift-rdp` was only read.
- Tool: `lesscode-core` built from `b52718a` (release).
- Commits of the clone, branch `protect/display-path`: `72507e1` the joining line and an empty
  test file (the base; the owner approved the line); `7af8b2c` the stage spec; `7fe9557` four
  tests; `4336df3` the verdict; `4da65d8` the owner's review.

| Step | Result | Time |
| --- | --- | --- |
| `step plan --open` on `display_path.rs`, two copies | exit 0: 20 faults, 1 noticed, 18 unnoticed, 1 that did not build | 14 min 22 s |
| the four tests tried in an already built copy | 13 of 13 tests of the module pass | 25 s |
| `step pin`, faults 1, 2, 3, 8, 9, 10 | six admitted | 17 min 50 s, then 90 to 143 s each |
| `step pin`, fault 7 | exit 3: not an unnoticed fault (it did not build); the unnoticed ones were listed | 44 s |
| adverse: a test that asserts nothing | rejected | 5 min 35 s |
| adverse: a test that contradicts the code | rejected; each copy tried its first run a second time (R183) | 2 min 13 s |
| `step check --page` | accepted | 9 min 54 s |

Per function, noticed of the faults that count, from the verdict:

| Function | Before | Now | Left, written off |
| --- | --- | --- | --- |
| `current` | 0 of 2 | 2 of 2 | |
| `fall_back` | 0 of 1 | 1 of 1 | |
| `layer_backends` | 0 of 1 | 1 of 1 | |
| `decide` | 0 of 3 | 2 of 3 | `arg_literal:string->empty@481:47`: a fact only Linux uses |
| `runtime_fallback` | 0 of 2 | 0 of 2 | `drop_statement@458:4` and `@461:4`: needs the handle of the running app |

`collect`, 9 faults in code built on Linux only, was left out. Two expectations of mine before the
tests were wrong in the cautious direction: I told the owner that four of the nine looked testable
and that the target of a half might fail. Reading `choose` showed that `decide("native")` on
Windows can only answer "the GPU was not tried" when the GPU was not tried, so the two faults in
`decide` that skip the GPU are noticed by one test that asks the real GPU for an adapter.

## What was done by hand, and stands for what

| By hand | Stands for |
| --- | --- |
| the declaration `step.json`, with `focus` on the five functions and `max_token_growth` 0 | `step protect begin` |
| the stage spec: five requirements, four test tasks, one task of proposals | the skill and its templates |
| the count per function from the verdict, and the three write-offs against `unnoticed_after` | `step protect result` |
| the joining line and the empty test file | a phase of the skill, with the person's approval |
| `approved_by` and `approved_at` of the write-offs, filled in by me from the owner's answer in the session, and marked so | the owner filling the two fields |

None of these is a capability the prototype shows.

## What the owner said

After the plan, on being asked four things at once: "could you give me choces for open items here,
considering what works. I don't understand the problem. all we want is better tests to ensure our
lesscode transformations are not causing problems. we brownfield onbaord the codebase with a tdd
workflow". I had described the stage in terms of its gate. Given choices in plain words, the owner
chose all five functions, "try, then note the rest", and one review at the end.

On the run page, which the owner watched during the plan: "it needs an end state, and a save state
to load past results (report). but I am not sure yet how this should look. once the process is
down, current report info is lost." And of a page from the morning: "The Dashboard progress bar
did not finish and it seemed as if there were still ongoing tasks". The second is changed
(`b52718a`: the line over the bar says "Done. All N planted faults were run:", and the hatched
part of the bar has an entry). For the first the owner chose, from four directions: an end screen,
and the tool opens the saved report so that the tab that stays is the one that survives a reload.
That is work for run-dashboard and is not done.

Not answered: whether the owner could say after ten seconds what the run was doing, and whether
the browser opened by itself.

## Evidence

`D:\BuildArtifacts\lesscode\brownfield-test-stage\prototype-1` (16.1 MiB): `plan.json`, `plan.html`,
`plan.log`, `run.json`, `start.json`, `end.json`, `work\plan-run.json`, `pins\` (the receipts and
logs of every pin and `times.json`), `check.html`, `check.log`, and the stored outcomes. The stage
spec, the tests and the verdict are in the clone, branch `protect/display-path` at `4da65d8`; the
adverse controls are its branches `protect/adverse-useless` and `protect/adverse-contradicts`.

## Limits

- One file, five functions, nine faults, one language, one machine. The project has a Kiro setup;
  one without was not tried.
- Three of the nine faults could not be reached, and one of them leaves a real gap: nothing checks
  that `runtime_fallback` moves the sessions to the canvas.
- One test asks the real GPU and needs `ONE_ONE_RENDERER` unset. On a machine without an adapter it
  still passes, by its second allowed answer; that was not run.
- The tests were written by the agent that also judged which faults a test could reach. The owner
  accepted them without reading the file.
- Storage: the eight working copies took 34.5 GiB and D: went from 120.2 to 87.9 GiB free during
  the run. Both limits of AGENTS.md held, the 40 GiB of caches by a margin of 5.5. The copies are
  removed; D: has 120.1 GiB free.
- Reviews were inline, by me.

## Tool/spec consequence and next gate

The design is revised in six points (its section "What the prototype changed") and the
requirements 2.5, 4.3, 4.4 and 7.3 are amended to the one review. Next: the owner's approval of
the design, then tasks. For run-dashboard: the end screen and the saved report the owner chose.

## COBESY check

The outcome stands first. The one expectation that failed is named with what I did wrong. What was
done by hand is listed apart from what a command did. Two of my forecasts to the owner were wrong
and are recorded.
