# R89: The first tracked step: the pins are accepted, the change is refused by a guard this repository already had

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 13:20 to 13:55 local (fault run 13:32 to 13:49; times from command output)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/requirements.md),
  Requirement 11.1 (requirements generated, not yet approved)
- Hypothesis ID and predecessor: the first real case named in
  [R88](14-gradual-steps-prototype-gate.md)
- Type: one real step, taken by hand on a copy of this repository

## Decision

The owner asked for the first step to be taken from a copy of the repository, with a thorough check of what
it breaks, and a guard. The step had two parts, in two commits.

1. **Pins: accepted.** One test that fixes the order of the reasons `verdict()` returns. No code under test
   changes. With it every planted fault in `verdict()` is noticed (10 of 11 before, 11 of 11 after). It is on
   the branch `step/verdict-one-exit`, pushed for review, not merged.
2. **The change: refused.** The simpler `verdict()` gives identical results on 480 input combinations, passes
   format check and lints, and breaks nothing in behaviour. It is refused because the file grows from 969 to
   972 lines and `native/src/recording.rs` has a frozen ceiling of 969. That guard is this repository's own
   complexity budget; three tests fail on it and the pre-push hook would stop the push. The change also
   removes one token in all. It is kept as a patch and on a local branch, not pushed.

So the first tracked step ends as the method says it should: the protection is taken, the change is not.
What made the difference was a hard boundary that existed before any skill or reviewer looked at it.

Separate outcomes: one step on one function. No model was called.

## Hypothesis and falsification

Stated to the owner beforehand: pins first, then the change, would be accepted (R88 had shown the pins turn
the fault verdict around). The fault verdict was not what refused the change; the budget did. The expectation
"the change will be accepted" is refuted.

## Frozen inputs and method

- A clone of this repository at `ad75c0f` under `D:\BuildArtifacts\lesscode\step1`, branch
  `step/verdict-one-exit`. Commit `e5dff41`: the pin. Commit `3698710`: the change (now on the local branch
  `step/verdict-one-exit-change`).
- Checks on each commit: format, lints, whole test suite, budgets. For the change in addition: a temporary
  differential test that holds the function as it was and compares it with the new one on ten exit codes,
  two freshness values, six difference counts and four evidence paths. For the pins: `deconvolute run` on
  `src/recording.rs` against the run of the base from R88, compared with `step_verdict.py`.
- Files: `docs/experiments/gradual-steps/step1/` (configuration, run receipt, verdict, the refused patch) and
  `docs/experiments/gradual-steps/ledger.jsonl` (two entries, the form a ledger could take).

## Evidence and results

| Check | Pins (`e5dff41`) | Change (`3698710`) |
| --- | --- | --- |
| Format, lints | clean | clean |
| Whole test suite | 359 of 359 | 356 of 359; the three failures are the budget checks |
| Budgets | pass | **fail: `recording.rs` 972 lines, ceiling 969** |
| Old against new on 480 inputs | not applicable | identical |
| Faults in `verdict()`: noticed / unnoticed | 10 / 1 to 11 / 0 | not run, refused before |
| Faults in the file: noticed / unnoticed | 90 / 101 to 91 / 100 | not run |
| Size of the file in tokens | unchanged | 7,307 to 7,306 |

Why the change is longer in lines although it is simpler: the project's formatter writes each of the three
guarded pushes on three lines and the choice of the exit code on several. The version in the specimen file
was shorter only because it was not formatted (R86).

## Limits and interpretation

- The refusal comes from a ceiling in lines, the unit this project has just stopped trusting for size. The
  ceiling still does its job as a ratchet here: it stopped a change that reshapes code without making it
  smaller. Whether budgets should count tokens is the owner's question, not answered here.
- The differential test is strong evidence for this function because it is pure and its inputs are few. It
  is the kind of check a step should carry whenever the changed unit allows it.
- `probe`, `run_groups` and `arm_env` remain without any noticed fault; this step did not touch them.

## Tool/spec consequence and next gate

- For the owner: merge `step/verdict-one-exit` (the pin) or not. The change stays out unless the ceiling is
  raised on purpose, which the step does not justify.
- For the requirements: a differential check as an optional part of a step, where a unit is pure; budgets as
  one of the guards a step verdict calls; a ledger entry also for a refused step.
- The requirements wait for the owner's approval.

## COBESY check

Outcome first: one part accepted, one refused, with the refusing check named. Replay: commits, branch, patch,
receipts and ledger named. Disproof: the expectation that the change would be accepted failed. The irony of
a line ceiling doing the refusing is stated, not smoothed over.
