# Task handover 35: density by syntax, releases 1.1.0 and 1.1.1, gradual refactoring in checked steps

Written 2026-11-11, about 14:20 (Europe/Berlin). Predecessor: `.kiro/task_34_handover.md`.

## What was done since handover 34

| Item | Commit or place | Evidence |
| --- | --- | --- |
| `deconvolute merge-annotations` | `c791516` | design Amendment 5 of agnostic-annotation-driver |
| Slice B closed with a summary | `docs/reports/slice-b-drift-hardening.md` | R66 to R85 |
| Rust specimen candidate built for the first time | R86 | compiles, tests pass; 40% lines is 12% when formatted |
| `deconvolute density` counts syntax | `cc9144e` | design Amendment 6; receipt v2 |
| Specimens re-measured | R87 | 6% to 19% fewer tokens, none substantial; amendments to R50 to R53 |
| README intro picture for developers | `docs/figures/how-it-works.svg` | three steps, link to openidea-kiro |
| Line and keyword counters removed | `9c48ab8` | 358 tests then |
| Releases 1.1.0 and 1.1.1 | tags on `54c2806` and `9c48ab8` | published and verified, `docs/releases/` |
| Gradual refactoring: brief, gate, requirements | `.kiro/specs/gradual-refactoring-steps/` | R88; requirements approved 2026-11-11 |
| First tracked step | R89 | pins merged (`4004bf6`); change refused by the complexity budget |
| kiro-cli installer | `tools/install_kiro_cli.ps1`, `.sh` | 1.1.0 into `tools/bin` (ignored) |

359 tests pass on main after the merge; format and lints clean.

## Owner decisions still to be carried out

1. **Design for `gradual-refactoring-steps`** (requirements approved). To cover: the `step` commands (plan,
   pin, check, record), the guard (pre-push hook and pipeline job, verdict bound to revisions and hashes of
   code and tests), ledger and ratchet, evidence keyed by spec criteria (format to be proposed to
   openidea-kiro, not fixed alone), running only the functions a step touches, the outcome store keyed by
   the test files (Requirement 9), an optional differential check for pure units.
2. **Complexity budgets should count tokens instead of lines** (owner: "yes, I would agree"). Not started.
   Needs: token and node measurement in `budgets` (the driver's `measure` with the Rust pack), a migration of
   the 51 entries in `native/complexity-budgets.json` (frozen files get their current token count as
   ceiling), the pre-push hook and its tests, README section on budgets. The refused change of R89
   (`docs/experiments/gradual-steps/step1/change-refused.patch`) is the test case: one token fewer, three
   lines more.
3. **Split the criteria with more than one "shall"**: `tools/bin/kiro-cli.exe check --feature
   gradual-refactoring-steps` reports 0 errors and 15 warnings of that kind. Do it when the design is
   written, without changing meaning, and note it in the spec.

## Things found and not yet fixed

- Stored outcomes of `deconvolute run` do not depend on the tests (documented in `docs/driver.md`; fix is
  Requirement 9 of the new spec).
- `probe`, `run_groups` and `arm_env` in `native/src/recording.rs` have no noticed fault; they need
  isolation before any step can touch them.
- nextest flagged two older CLI tests as leaky once, under three suites running side by side; not reproduced.
- The work trees under `D:\BuildArtifacts\lesscode\steps` and `\step1` (about 20 GB) can be deleted.

## Current state

- main at the commit of this handover; CI state below.
- Model spend recorded: about 32.9 of 40 USD. Slice B is closed.
- No container of this project exists.

## CI

Filled in once the run for this commit has finished.
