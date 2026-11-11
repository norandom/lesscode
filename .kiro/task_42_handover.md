# Task handover 42: task group 6 of gradual-refactoring-steps (real steps, documents, final validation)

Written 2026-11-11, 19:18 (Europe/Berlin, from the clock). Predecessor: `.kiro/task_41_handover.md`.

## What was done

| Item | Commit | Result |
| --- | --- | --- |
| 6.1 `verdict()` change as the first real step | `48d2d4d` on main; branch `step/one-exit` (`08f4a05`, `e64b7b3`, `0d54533`) | accepted, recorded, guard exit 0; `verdict` 11 / 0 to 12 / 0 noticed / unnoticed; 7,307 to 7,306 tokens |
| 6.2 one step in Java and one in TypeScript | `633f608` | commons-csv `comment-start` and OpenUltraCode `slug-separator`, both accepted, recorded, guard exit 0 |
| 6.3 README section, skill, what no command checks | `198f369` | `README.md` (row, section 6, limits), `docs/steps.md` (examples, "What no command checks"), `.agents/skills/refactor-in-steps/SKILL.md` |
| 6.4 validation, research entry, issue text | this commit | [R92](../docs/research/2026-11-11/18-three-real-steps.md); `.kiro/specs/gradual-refactoring-steps/openidea-kiro-issue.md` for the owner to file |

All tasks of `gradual-refactoring-steps` are ticked.

## Validation report (kiro-validate-impl, inline; no subagent, owner rule of 2026-11-07)

- DECISION: GO
- MECHANICAL_RESULTS (main, 2026-11-11 19:16 to 19:17, log `D:\BuildArtifacts\lesscode\validate-g6.log`):
  - Tests: PASS, `cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --no-fail-fast`,
    385 of 385, exit 0; doctests exit 0; format check exit 0; clippy with `-D warnings` exit 0
  - TBD/TODO grep over `native/src/step`, the step tests, `docs/steps.md`, the skill: clean
  - Secrets grep over `native/src/step`: clean
  - Smoke: `lesscode-core step --help` and the three real steps (check, record, guard, each exit 0)
  - `kiro-cli check --feature gradual-refactoring-steps`: 0 errors, 0 warnings, 26 info
- INTEGRATION: the three real steps ran declaration, copies, fault runs, verdict, evidence, ledger and guard
  in one chain in three languages. Not exercised on a real step: `step pin`, `step plan` (last used in the
  gate), a refusal, an override, a release entry, the hook on a real push of a step branch. Those rest on
  the integration tests of groups 3 to 5.
- COVERAGE: 61 criteria; 60 are named by a task. 11.1 (a step by hand before production tasks) is not named
  by a task because it was met before tasks existed (R89).
- DESIGN: no code changed in this group; file structure as at handover 41.
- OWNERSHIP: LOCAL. BLOCKED_TASKS: none.
- This is an inline review by the implementer, not an independent one.

## Current state

- main holds everything above. `step/one-exit` is a local branch, not pushed and not merged.
- The foreign steps are on clones under `D:\BuildArtifacts\lesscode\steps-foreign` (`commons-csv`,
  `OpenUltraCode`); their outputs and patches are copied to `docs/experiments/gradual-steps/steps/`.
- The spec's `phase` in `spec.json` is still `tasks-generated`; `ready_for_implementation` is false. Not
  changed here.

## Next steps (owner's decisions)

1. Merge `step/one-exit` into main, or not. If it is merged, the ledger `.kiro/steps/ledger.jsonl` and the
   spec `.kiro/specs/one-exit/` come with it.
2. File the issue text at openidea-kiro, or change it first.
3. A release that contains the step commands (the README says "build from source" until then).
4. A real step that needs a pin and one that is refused, in a foreign project; R92 names this as not shown.

## Notes

- `docs/experiments/gradual-steps/step1/change-refused.patch` does not apply with `git apply` on main
  (context offset); the change made by hand has the blob hash the patch names.
- Step specs carry `requirements.md` and `spec.json` only; `kiro-cli check` reports the missing design and
  tasks as info, not as findings.
- Work directories that can be deleted: `D:\BuildArtifacts\lesscode\step-one-exit`, `steps-foreign\work-ts`,
  `steps-foreign\work-java`, and those listed in handover 41.
- No model was called in this group. Model spend unchanged: about 32.9 of 40 USD. Slice B is closed. No
  container of this project was started.
