# R95: A second Python step with a frame specification; a wrong prediction; the test suite fills the temp folder

- Report date/time and timezone: 2026-11-12, 09:30, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 08:50 to 09:25 local (times from file and commit timestamps)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/requirements.md); open
  work from handover 43, by owner decisions of 2026-11-12 (second real step; guard in the pipeline; fix the
  hook message; retire the adopter check; branch names `lesscode-<step>-<run>`)
- Hypothesis ID and predecessor: [R94](01-first-python-step.md)
- Type: one real step in a foreign project with a prediction stated before the first check; two
  demonstrations on throwaway changes, labelled; three changes to this repository

## Decision

1. Step `checkpoint-clear` in x_likes_scraper is accepted, recorded and passed by the guard, with a frame
   specification declared. `Checkpoint.clear` calls `unlink(missing_ok=True)` instead of asking first:
   4 noticed / 3 unnoticed before, 4 / 0 after, four lines removed and two added in the method.
2. The prediction was wrong. I expected a refusal, because no test clears a directory without files and
   `missing_ok=True` is new. The check accepted the change before any pin (4 / 3 to 2 / 2). The Python
   pack plants no fault in a keyword argument, so `missing_ok=True` could be turned to `False` unseen by
   the tool. The test for that case was written because a person thought of it.
3. A frame refusal and a release entry work, each shown on a throwaway change that was removed again. The
   frame names write sites by line number: one added line moved every site below it, and the refusal
   listed five sites where one write was new.
4. An out-of-reach mark was not needed and is still not shown on a real step. The two faults left after
   the change were in a `print`, and a test with `capsys` pins them.
5. In this repository: the hook no longer says "passed" before a refusal (`dc78afa`); the guard is a job
   of the pipeline (`12957e4`, not yet run on the host); the adopter check is retired (same commit).
6. The test suite leaves its directories in the temp folder: 26,285 of them on the owner's machine since
   2026-11-09, the `lc-regen-e4-replay-*` ones alone 48.5 GiB. The regenerator tests are fixed
   (`83f29d6`); the 21 other test files that use the temp folder are not, and were not looked at one by
   one.

Separate outcomes: mechanism (frame, release entry: shown; out of reach: not shown); behaviour of the
changed code (tests pass, two new tests fix the print and the case without files; not a proof); size (two
lines fewer); adoption (none; both branches are on a local clone). No model was called.

## Hypothesis and falsification

- Check 1, change without pins: refused on planted faults. Refuted by an acceptance. It was accepted.
- Pin against fault #6 (the `print` dropped): admitted. It was.
- Check 2, with pins: accepted. It was.

## Frozen inputs and method

- Project: `x_likes_scraper`, base `3a214ff`, clone as in R94; branch `lesscode-checkpoint-clear-1`, head
  `f60610b`. The branch of R94 was renamed to `lesscode-escape-html-1` (head `39040e2`); the declaration
  names the branch, so that step was checked and recorded again and is accepted.
- Binary: `lesscode-core 1.2.0` as in R94 (without the hook fix; the hook was not used here).
- Declaration: [step.json](../../experiments/gradual-steps/steps/checkpoint-clear/step.json), focus
  `clear`, test file `tests/test_checkpoint.py`, `max_token_growth` 0, `frame_spec` with the 22 write
  sites `deconvolute analyze` lists for the changed file.
- A fresh work directory per run, deleted after the run.

## Evidence and results

Files in `docs/experiments/gradual-steps/steps/checkpoint-clear/`.

| Run | Result | File |
| --- | --- | --- |
| Check 1, no pins | accepted; `clear` 4 / 3 before, 2 / 2 after; frame passes | `verdict-1-accepted-before-pins.json` |
| `step pin --fault 6` | admitted; one base fault remains (`swap_next`, line 110) | `pin.txt`, `pins.patch` |
| Check 2, with pins | accepted; 4 / 3 before, 4 / 0 after | `verdict.json`, `evidence.json`, `change.patch` |
| `step record`, `step guard` | recorded as accepted; guard exit 0 | `ledger.jsonl` |
| Throwaway: `self.cleared = True` added | refused in 5 s, "writes state outside the declared frame", five lines named | `verdict-throwaway-frame-refused.json` |
| Throwaway: `step record --release 1.1.1` | "1 steps since the release before: checkpoint-clear" | `ledger-with-throwaway-release.jsonl` |

In this repository: 388 tests pass with the hook fix; format, lints and budgets clean. The test
`the_installed_hook_stops_a_push_without_a_matching_verdict` now asserts that a refused push does not
print "verification passed". After the fix to `tests/regenerator.rs` a run of its 50 tests leaves the
count of `lc-regen-*` directories unchanged (4,815 before and after).

Both steps together change x_likes_scraper by 11 lines removed and 4 added in two source files of about
11,000 lines of Python outside the tests, and add 34 lines of tests.

## Limits and interpretation

- Two steps do not measure what lesscode can remove from this project. Both reductions were found by a
  person reading the plan and the code. The tool judged them.
- "Accepted" twice meant less than it says: in R94 a change of output passed, here a new argument was
  outside what the pack can break. In both cases the missing piece was a test nobody had asked for.
- In a Python plan the column "writes" counts assignments to local names as well; `get_progress` has
  four and writes no state.
- The frame by line number refuses correctly and reports badly.
- The pipeline job is written and not run. Whether it holds is known after the next push.

## Tool/spec consequence and next gate

For the onboarding spec, as findings, not decisions:

- What "accepted" promises has to be said in plain words at each level, or the check has to refuse where
  nothing is protected (owner decision of 2026-11-12: refuse; to be designed in the new spec).
- The Python pack should plant faults in keyword arguments.
- The frame should name sites in a way that survives an inserted line.
- Work and test directories have to be removed by the tool and by the tests. A cap on disk use per run is
  a requirement for levels 1 to 3.
- Branch names: `lesscode-<step>-<run number>`.

Next: the remaining test files that leave directories behind; the owner's approval for deleting old run
output; the first hosted run of the guard job; then `$kiro-discovery` for the onboarding spec.

## COBESY check

Outcome first, including the wrong prediction; runs can be repeated from the declarations; throwaway
demonstrations are labelled and their changes are gone; the unverified pipeline job is named as such;
counts come from commands, not estimates; no claim about equivalence, effectiveness in general or
adoption.
