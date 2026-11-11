# Task handover 36: budgets in tokens, design for gradual refactoring in checked steps

Written 2026-11-11, 14:50 (Europe/Berlin). Predecessor: `.kiro/task_35_handover.md`.

## What was done

| Item | Commit | Evidence |
| --- | --- | --- |
| `budgets` counts tokens instead of lines | `d1cca0f` | [R90](../docs/research/2026-11-11/16-budgets-count-tokens.md); 361 tests |
| Ceilings of this repository migrated | `d1cca0f` | `native/complexity-budgets.json`, 52 entries; nothing looser, one file and two functions frozen |
| 15 criteria with more than one "shall" split | `564e87a` | `kiro-cli check`: 0 errors, 0 warnings |
| Design of `gradual-refactoring-steps` | `564e87a` | `.kiro/specs/gradual-refactoring-steps/design.md`, **not approved** |

## Waiting for the owner

1. **Approval of the design.** Its four open points:
   - callers found by name: good enough, or run only the changed functions;
   - the override stays the one way past the guard, recorded in the ledger;
   - a differential check for pure units is not in the requirements: add it or leave it out;
   - the change refused in R89 is within budget under token ceilings (7,306 of 7,307): take it as the first
     step through the finished tooling, or apply it now by hand.
2. **Release.** The token budgets change the released `budgets` command (new defaults, receipt schema
   `rust-syntax-units-v2`; old budgets files keep working). No release was prepared.

## Amendment, 2026-11-11, about 15:10: owner decisions

The owner approved the design: callers by name; no differential check for now; the change refused in R89
is kept as the first step through the finished tooling. No release was asked for. `tasks.md` is generated
(`5780220`), six groups, **not approved**. `kiro-cli check`: 0 errors, 0 warnings. Waiting for the owner:
approval of the tasks, then `kiro-impl gradual-refactoring-steps 1.1` and following in the foreground.

## Next, after approval

- `kiro-spec-tasks gradual-refactoring-steps`. The design's prototype-gate section names the first three
  tasks, each able to fail: the function filter gives the same classes as a whole-file run; copies from
  `git archive` build and test like a working tree; single-fault pin admission through the product path.
- Then the driver change of Requirement 9 (store key includes the test files) and the `step` commands.

## Things found and not yet fixed

- `probe`, `run_groups` and `arm_env` in `native/src/recording.rs` have no noticed fault.
- `.kiro/complexity-budgets.json` and the design of deconvolution-native-core still describe the line
  metric; left as records.
- `kiro-cli check` on `agnostic-annotation-driver` gives 22 warnings (for example a task boundary that names
  no design component). Not looked at.
- The work trees under `D:\BuildArtifacts\lesscode\steps` and `\step1` (about 20 GB) can be deleted.

## Current state

- main at `564e87a` plus this handover. Model spend recorded: about 32.9 of 40 USD. Slice B is closed.
- No container of this project exists.

## CI

- `564e87a` (run 43030208246): success on all five jobs. It contains the code of `d1cca0f`.
- `d1cca0f` (run 43029699235): four jobs success; `macos-15-intel` failed in the step that uploads the
  evidence artifact ("Failed to CreateArtifact: Unable to make request: ENOTFOUND"), a network error of the
  runner, after the tests. Not rerun.
