# Task handover 37: task group 1 of gradual-refactoring-steps (the three assumptions)

Written 2026-11-11, about 16:15 (Europe/Berlin). Predecessor: `.kiro/task_36_handover.md`.

## What was done

Owner approvals on 2026-11-11: design (callers by name; no differential check; the R89 change becomes the
first step through the tooling), then the task list.

| Task | Commit | Result |
| --- | --- | --- |
| 1.1 function filter | `1b6e91a` | `functions` in the run configuration; 11 of 11 classes equal to the whole-file run; 2.1 min instead of about 17 |
| 1.2 copies from a commit | `5543cea` | `native/src/step/revisions.rs`; same test results as working trees on three projects when the whole repository is copied |
| 1.3 pin against one fault | commit after `5543cea` | `deconvolute run --only N`; admitted, rejected as useless, rejected as failing |
| 1.4 gate result | this commit | [R91](../docs/research/2026-11-11/17-gradual-steps-three-assumptions.md): proceed; design Amendment 1 |

366 tests pass; format, lints and budgets clean. `kiro-cli check`: 0 errors, 0 warnings.

## Design changes (Amendment 1)

- Copies hold the whole repository; the declaration has no `root`; commands run at the repository root.
- Verdict and guard hash committed content (blobs), not files on disk.
- No `git archive`; trees are read with `ls-tree` and `cat-file --batch`.

## Next: task group 2 (driver: stored outcomes depend on the tests)

- 2.1 Hash over the files under `test_paths` joins the configuration identity
  (`native/src/adapters/driver/config.rs` `identity()`, used in `run_command.rs` for `Store::key`). Test:
  a changed test file makes affected mutants run again without `--no-reuse`.
- 2.2 `docs/driver.md` hazard note and an amendment in the design of agnostic-annotation-driver.
- Then group 3: declaration, functions (changed, callers), `step plan`.

## Notes for the next groups

- `native/src/lib.rs` is at its unit ceiling (196 of 196); a new top-level module needs room. The budgets
  receipt became `budgets/report.rs` for that reason. New step code goes under `native/src/step/`.
- rustfmt reorders `mod` lines; do not rely on their position.
- `native/examples/copy_revision.rs` exists for evidence runs until the step commands replace it.
- A fresh copy builds from nothing (1.3 to 2.1 min here); the step commands should reuse a copy's build.
- Work directories `D:\BuildArtifacts\lesscode\t11`, `t12`, `t13`, `steps`, `step1`, `budget-case` can be
  deleted.

## Current state

- Model spend recorded: about 32.9 of 40 USD. Slice B is closed. No container of this project exists.

## CI

Run 43040111177 for commit `c51bd57`: success on all five jobs, 2026-11-11.
