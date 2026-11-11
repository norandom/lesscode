# Task handover 39: task group 3 of gradual-refactoring-steps (declaration, functions, step plan)

Written 2026-11-11, about 18:30 (Europe/Berlin). Predecessor: `.kiro/task_38_handover.md`.

## What was done

| Task | Commit | Result |
| --- | --- | --- |
| 3.1 declaration | `9e1fc28` | `native/src/step/declaration.rs`: read, validate, run configuration, revisions |
| 3.2 functions | `0448cc0` | `native/src/step/functions.rs`: changed by token sequence, callers by name, faults per function; name scan covers `src/step` |
| 3.3 `step plan` | `5ac36ab` and this commit | `lesscode-core step plan`; command group in `step/mod.rs`; `docs/steps.md` |

374 tests pass; format, lints and budgets clean. `kiro-cli check`: 0 errors, 0 warnings. Design Amendment 2
records what was built differently.

Real plan on `native/src/recording.rs` (tests of `tests/recording.rs` only): 191 faults, 91 noticed, 100
not, 13.8 minutes with three copies; a second run from the store is immediate. Receipt:
`docs/experiments/gradual-steps/gate2/plan-recording.json`. The totals equal the whole-file run of R89.

## Next: task group 4 (pins and the step check)

- 4.1 `step pin`: composite copy (base code, test paths of the step revision) and `run --only N`; the
  three outcomes were shown by hand in R91. Needs: copy base with `revisions::copy_tree`, overlay the test
  paths from the step revision, a base fault run to number the faults.
- 4.2 `step check` checks 1 to 4 (declaration and revisions, files outside the scope, pins and change in
  one commit, tests on base and step). `revisions.rs` still needs `log` with names of changed files.
- 4.3 fault runs limited to changed functions and callers, comparison, size, frame.
- 4.4 verdict and evidence; hashes of committed blobs (design Amendment 1).

## Notes

- `main.rs` ceilings were lowered to its measured size after its path helpers moved to `cli_paths.rs`;
  `lib.rs` is at its unit ceiling. New code goes under `native/src/step/`.
- `run_run` is called as a library function by `step plan` and writes its receipt to
  `<work>/plan-run.json`.
- The Python pack counts a call that is a statement of its own as a state write.
- Work directories under `D:\BuildArtifacts\lesscode` (t11, t12, t13, t33, steps, step1, budget-case,
  figure-check) can be deleted; t12\whole is the copy used for the plan.
- Model spend recorded: about 32.9 of 40 USD. Slice B is closed. No container of this project exists.

## CI

Run 43054185408 for commit `72a95e6`: success on all five jobs, 2026-11-11.
