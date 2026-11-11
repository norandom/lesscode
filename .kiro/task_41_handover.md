# Task handover 41: task group 5 of gradual-refactoring-steps (ledger, record, guard, hook), two documents

Written 2026-11-11, 18:28 (Europe/Berlin, from the clock). Predecessor: `.kiro/task_40_handover.md`.

## What was done

| Item | Commit | Result |
| --- | --- | --- |
| 5.1, 5.2 ledger, `step record`, `step guard` | `df611f5` | chained append-only ledger; record of accepted verdict, override with reason, release; guard on committed content; ledger number held in `step check` (Requirement 7.5) |
| 5.3, 5.4 hook and protection tests | `284a1dc` | `step install-hook`, `step pre-push`; a test pushes through the hook to a bare repository and is stopped when the code changed after the verdict |
| Testability document (owner request, COBESY) | `9dc0b7a` | `docs/testability.md` |
| README figure for people who write code (owner request) | `91a3d37` | terminal output and code instead of flow-graph notation; generator `docs/figures/make_how_it_works.py` |
| Guide sections 5 and 6, design Amendment 4 | this commit | `docs/steps.md` |

385 tests pass; format, lints and budgets clean; `kiro-cli check` 0 errors, 0 warnings.

Owner's wording, 2026-11-11: "Pex and Moles" means testability in general with this approach, not those
tools. `docs/testability.md` has a section saying so.

## Next: task group 6 (real steps and delivery)

- 6.1 The `verdict()` change of R89 as the first step through the tooling, on this repository: spec and
  `step.json` under `.kiro/specs/<step>/`, branch, pins are on main already (so `focus` on `verdict` and
  the change as the only code commit), `step check` with a work directory under `D:\BuildArtifacts`,
  `step record`, `step guard`. The patch is `docs/experiments/gradual-steps/step1/change-refused.patch`.
  Test command at the repository root: `cargo nextest run --locked --manifest-path native/Cargo.toml`
  (the whole suite, or `--test recording` to keep it short; say which), `env CARGO_TARGET_DIR=target`,
  `run.exclusions ["target"]`, base as a commit hash. The budgets now count tokens, so the change is within
  budget (7,306 of 7,307). The owner's decision on merging it is still open.
- 6.2 One step in the Java and one in the TypeScript project of R75 (copies under
  `D:\BuildArtifacts\lesscode\annotation-driver\scopes`; never the upstream checkouts).
- 6.3 README section, skill that walks the phases, list of checks that could not be made mechanical.
- 6.4 `kiro-validate-impl`, research entry, issue text for openidea-kiro about the evidence format.

## Notes

- Git hooks on this machine: a subshell in the hook (`refs=$(cat)`) failed with a cygwin fork error;
  the hook is a single `exec` for that reason.
- The guard judges committed content: commit verdict, evidence and ledger before pushing.
- Declarations should name a commit or tag as base.
- Work directories under `D:\BuildArtifacts\lesscode` (t11, t12, t13, t33, t41, t53, steps, step1,
  budget-case, figure-check) can be deleted.
- Model spend recorded: about 32.9 of 40 USD. Slice B is closed. No container of this project exists.

## CI

Filled in once the run for this commit has finished.
