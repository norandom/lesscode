# Task handover 38: task group 2 of gradual-refactoring-steps, README figure

Written 2026-11-11, about 17:10 (Europe/Berlin). Predecessor: `.kiro/task_37_handover.md`.

## What was done

| Item | Commit | Result |
| --- | --- | --- |
| 2.1 stored outcomes depend on the test files | `fe2fb10` | hash over the files under `test_paths` joins the store key; receipt field `test_files_sha256`; a changed or added test file reruns the mutants |
| 2.2 documents | `7225c7c` | `docs/driver.md`; Amendment 7 in the design of agnostic-annotation-driver |
| README figure redrawn (owner request) | `71f3009` | monochrome figure with three captioned parts; the control-flow graphs are drawn from `verdict()` and the candidate change |

367 tests pass; format, lints and budgets clean.

## Next: task group 3 (declaration, functions, `step plan`)

- 3.1 `native/src/step/declaration.rs` and the `step` command group with exit codes. Per design
  Amendment 1 the declaration has no `root`; paths are relative to the repository root.
- 3.2 `native/src/step/functions.rs`: changed functions by token sequence, callers in scope by callee
  name, faults per function. `adapters/driver/functions.rs` already gives functions with names and
  positions. Extend the name scan to `native/src/step/`.
- 3.3 `step plan`.

## Notes

- Two limits of the store key remain and are documented: no `test_paths` declared, and changes to other
  source files that are neither mutated nor tests.
- `native/src/lib.rs` is at its unit ceiling; new code goes under `native/src/step/`.
- Work directories under `D:\BuildArtifacts\lesscode` (t11, t12, t13, steps, step1, budget-case,
  figure-check) can be deleted.
- Model spend recorded: about 32.9 of 40 USD. Slice B is closed. No container of this project exists.

## CI

71f3009 completed success 43046139374
