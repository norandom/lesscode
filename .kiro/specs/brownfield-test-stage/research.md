# Research log: brownfield-test-stage

Written 2026-11-24, 11:30 Europe/Berlin, for the design phase. Discovery was done in the main
context, by reading the code and the receipts of earlier steps; no subagent, no web research.

## What already exists and carries the stage

Read in `native/src/step/` and in the kept receipts of earlier steps
(`docs/experiments/gradual-steps/steps/`).

| Need of the stage | What is there | Where seen |
| --- | --- | --- |
| A test is shown failing on the fault and passing on the unchanged code | `step pin`: a copy with the code of the base and the test files of the branch; the tests run with and without the fault; exit 0 admitted, 1 rejected with which of the two happened, 2 not judged | `pin.rs`; `escape-html/pin.txt` |
| Only the chosen functions' faults are run again | a declaration's `focus`: functions to look at "when the step does not change them (a step that only adds pins)" | `declaration.rs`; `escape-html/step.json` |
| Before and now per function, and the faults that remain | the verdict of `step check`: per function `before` and `after` (noticed, unnoticed, other) and `unnoticed_after`, each fault as `#N line L operator@L:C` | `escape-html/verdict-pins-only-accepted.json`: 0 of 15 noticed before, 15 of 15 after |
| A noticed fault must not become unnoticed | `step check` refuses when unnoticed faults rise in a function | `check_runs.rs` |
| The code is not changed | `max_token_growth: 0` in the declaration; the check "changes stay in scope" | `escape-html/step.json` |
| Tests apart from code | the check "pins apart from the change" | verdict of the same step |
| The run can be watched | `step check --page --open` shows the stages and, per function, before and after | run-dashboard, R178 |
| The place and kind of every unnoticed fault of a plan | the run receipt of a plan (`plan-run.json`): per fault the file, line, column, operator, text, class, seconds and a hash | `D:\BuildArtifacts\lesscode\run-dashboard\real-run-8.1\work\plan-run.json` |
| What a function reaches out to | the plan: `reaches_out.calls` and `reaches_out.state_writes` for unprotected functions | `plan.json` of the same run |

A step that only adds tests was taken to its end before: `escape-html` (Python, R94/R95), accepted
with 15 of 15 faults noticed where none was.

## What is missing

1. **From a plan to a declaration.** `step pin` and `step check` need a step declaration. Someone
   who has a plan has a run configuration. The two hold the same commands in different shapes.
2. **Writing a fault off.** Nothing records that a fault is no call for a test, by whom that was
   approved, and nothing counts such faults.
3. **"Protected enough" by exit code.** The verdict says accepted when nothing got worse. The
   stage needs: every fault noticed or written off.
4. **The stage spec.** Nothing describes what such a spec holds, and no skill walks an agent
   through the stage.

## Synthesis

- **Generalisation found.** The stage is a step that adds tests and changes no code. Nothing new
  runs a fault. This is the owner's reading ("we copy from kiro and re-use the tdd") and the code
  agrees with it.
- **Build or adopt.** Adopt `step plan`, `step pin`, `step check`, the declaration and the run
  page as they are. Build two small commands (from a plan to the start of a stage; from a verdict
  and the write-offs to "protected enough") and one skill with its templates.
- **Simplification.** No command writes the stage spec. An agent writes it from a file the first
  command leaves, in Kiro's form. The tool holds no model and no templates of prose.
- **Kiro's test-first protocol** asks for a failing run before the implementation and a passing
  run after. Here the implementation exists. The planted fault takes the place of the missing
  implementation: red is the test failing on the code with the fault, green is the test passing
  on the unchanged code. `step pin` runs both and answers by exit code, so its receipt is the
  evidence Kiro's protocol asks for under `RED_PHASE_OUTPUT` and `TESTS_RUN`.
- **What is copied from Kiro** is the form: the files of a spec (`requirements.md`, `tasks.md`,
  `spec.json`), numbered requirements with acceptance criteria, tasks with a boundary and a done
  condition, approval per phase. The text of Kiro's skills is not copied into lesscode's skill: the
  directory `.agents/skills/kiro-*` is not tracked in this repository and its licence is not
  recorded here.

## Risks found, in the order they could stop the design

1. **Rust keeps unit tests inside the file of the code.** `step pin` takes the test files of the
   branch by path (`test_paths`) and the code of the base. A test added to the `mod tests` of
   `display_path.rs` is, for the tool, a change of the code: it is not carried into the pin's
   copy, and it grows a scope file. The three renderer files looked at
   (`display_path.rs`, `shared_frame.rs`, `yuv.rs`) all have their tests inline. The only test path
   of the crate is `src-tauri/tests/`, whose tests cannot reach private functions. Python,
   TypeScript and Java keep tests in files of their own and do not have this. Three ways out are
   named in the design; which one works is for the prototype.
2. **Cost.** Each `step pin` builds and tests the project twice in a copy. On the renderer a run
   with one planted fault took about 40 seconds once a copy was built, and a first build about
   three minutes. Twenty faults with one pin each is workable; hundreds are not. One test may
   notice several faults, and `step check` with `focus` judges all of them in one run.
3. **Faults in conditions on the platform.** Of the first unnoticed faults of `display_path.rs`
   two negate `cfg!(target_os = "linux")` and `cfg!(windows)`. On one system such a fault cannot be
   noticed by any honest test. These are candidates for "cannot be tested as the code stands", and
   they show that a share of write-offs is to be expected in real code.
4. **Who approved.** The tool can read that a write-off names an approver and a date. It cannot
   know that a person wrote them. The design says so and puts the approval where a person sees it.
5. **Fault identity.** The verdict names a remaining fault as `#N line L operator@L:C`. The number
   depends on the run; file, operator, line and column do not, as long as the code is unchanged,
   which the stage requires.

## Not researched

- Whether a project without any Kiro setup takes the stage spec well. drift-rdp has one.
- A Java subject for the stage. Tests were pinned before in Python (`escape-html`, R94/R95) and in
  TypeScript (`shape-checks`, R93); no Java step added a test.
