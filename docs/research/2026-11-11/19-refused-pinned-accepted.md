# R93: A real step is refused, pinned and accepted; on the way it shows that a reused working copy kept its old tests

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 19:42 to 20:28 local (times from command output)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/requirements.md),
  Requirements 3, 5, 9.2 and 11.2; all tasks of the spec were ticked before this entry, which is release
  preparation for 1.2.0 by owner decision of 2026-11-11
- Hypothesis ID and predecessor: [R92](18-three-real-steps.md), which named a pin and a refusal as not shown
  on a real step
- Type: one real step in a foreign project, with a prediction stated before the first check; one fault
  found in the driver and fixed

## Decision

1. The refusal and the pin work on a real step. In OpenUltraCode (TypeScript) the step `shape-checks`
   rewrites `isPersistedWorkflowState` to one rule per statement. `step check` refused it: unnoticed faults
   in that function rise from 1 to 7. `step record` and `step guard` then exit 1. Nine tests, committed by
   themselves, were admitted by `step pin` against the one unnoticed fault of the base. The step was then
   accepted, recorded and passed by the guard: 3 noticed / 1 unnoticed before, 11 / 1 after.
2. The driver had a fault that this flow hits every time, and it is fixed. Between the refusal and the
   acceptance stood two more refusals that were wrong. A working copy under the work directory was made
   once and used again as it was. The second check ran the mutants again, as it should after a test
   changed, but in a copy that still had the old tests. Releases 1.1.0 and 1.1.1 have the same fault in
   `deconvolute run`. Commit `aa9d7d7` makes a copy follow its source before it is used again.
3. Release 1.2.0 can be prepared. The paths R92 left open are shown, and the release would have shipped
   the fault without this step.

Separate outcomes: mechanism (refusal, pin, record, guard: shown on a real step); a defect in the tool
(found, reproduced by a test that fails without the fix, fixed); behaviour of the changed code (tests pass,
fault sensitivity higher than before; not an equivalence proof); size (25 tokens more, declared);
adoption (none: the step is on a local clone). No model was called.

## Hypothesis and falsification

Stated before the first check: the change will be refused on planted faults, because the project's tests
give the function one corrupt file that is not JSON at all and no file with a wrong shape, so every single
rule can be broken unseen once the rules stand apart. It would have been refuted by an acceptance. It was
refused, with seven named faults.

Second prediction, after the pins were admitted: the next check accepts. This was refuted twice, and the
reason was not the step. See "The fault in the driver".

## Frozen inputs and method

- Clone of the copy of OpenUltraCode at `37a4e55` under `D:\BuildArtifacts\lesscode\steps-foreign`, branch
  `step/shape-checks`: `c06a9c8` spec and declaration, `b8b8090` the change, `8b856bb` the pins, `8d06979`
  verdict, evidence and ledger. The project's `prepare` script had set `core.hooksPath` in the clone when
  the tests were run by hand; it was unset again so that the clone's own hooks did not run on these commits.
- Declaration: scope `src/state.ts`, test command `node --import tsx --test tests/state.test.ts`, focus on
  `isPersistedWorkflowState`, `max_token_growth` 30.
- The function was chosen from a plan: `step plan` on `src/state.ts` (9 min 42 s for three files), 17
  functions, `isPersistedWorkflowState` 3 noticed / 1 unnoticed.
- Checks 1 and 2 with a debug build of main before the fix. Checks 3 and 4 and a first replay were meant
  to use the fix and did not: the binary copied for them came from the negative test of the fix, with the
  refresh switched off (an `examples` build had not rebuilt the executable). Its behaviour in these runs is
  that of the binary before the fix. The final replay used a build of `aa9d7d7`, checked by its version
  output and by the record file it updates.
- Order of commits: the pins were written after the refusal, so they come after the change. `step pin`
  tries a pin against the code of the base in either case, and the check asks only that pins and change are
  in separate commits.

## Evidence and results

Files: `docs/experiments/gradual-steps/steps/shape-checks/`.

| Run | Binary | Work directory | Result | File |
| --- | --- | --- | --- | --- |
| Check 1, change without pins, 19:49 to 19:54 | before the fix | fresh | refused: unnoticed 1 to 7 in `isPersistedWorkflowState` | `check-1.txt`, `verdict-1-refused.json` |
| `step record`, `step guard` | before the fix | | exit 1, "the verdict is refused" | |
| `step pin --fault 4`, 29 s | before the fix | same, own copy | admitted; no unnoticed fault of the base remains | `pin.json` |
| Check 2, with pins | before the fix | same as check 1 | refused, same seven faults; this verdict is wrong | `check-2-stale-copy.txt`, `verdict-2-refused-stale-copy.json` |
| Check 3, with pins, 25 s | refresh off | same as check 1 | refused, same seven: the wrong classes of check 2, read from the store | `check-3-stored-outcomes-of-check-2.txt` |
| Check 4, with pins, 20:05 to 20:09 | refresh off | fresh | accepted: 3 / 1 before, 11 / 1 after; 1,505 to 1,530 tokens | `check-4-accepted.txt`, `verdict.json` |
| `step record`, `step guard` | refresh off | | exit 0, "accepted" | `ledger.jsonl` |
| Replay without the fix, second clone: check without pins, add pins, check again, 20:10 to 20:17 | refresh off | one, fresh at the start | refused, then refused again with the same seven: the fault reproduced | `replay-without-refresh-refused.txt` |
| Replay with the fix, same clone: check without pins 20:18 to 20:25, add pins, check again 20:25 to 20:28 | `aa9d7d7` | one, fresh at the start | refused, then accepted: 3 / 1 before, 11 / 1 after; the worker copy holds the pins | `replay-1-refused.txt`, `replay-2-accepted.txt` |

The seven faults of check 1: five times a `return false` turned into `return true`, one `&&` turned into
`||`, and one swap of two neighbouring checks. The pins are nine tests, each writing one state file that
parses and has one thing wrong (null, a list, another schema version, a time that is not a string, and the
same for the inner state), and expecting the file to be treated as corrupt. After the pins the one
unnoticed fault left is the swap of the second and third check, which can stand in either order.

### The fault in the driver

- What happened: check 1 made the worker copy `work/step-run/w1/tree` from the step commit without pins.
  Check 2 found that directory, opened it and ran. The store key holds the hash of the test files since
  task 2.1, so all mutants ran again, in a copy whose `tests/state.test.ts` did not contain the pins
  (checked: no match for the new test names in the copy, two in the fresh one).
- Why the tests of task 2.1 did not see it: `a_changed_test_file_makes_the_mutants_run_again` asserts that
  the mutants are not reused. It does not assert that a class changes.
- The fix (`aa9d7d7`, `native/src/adapters/driver/sync.rs`): a copy records the files it was given with
  their hashes. Before it is used again, changed and new files of the source are copied, files that are
  gone are removed, what the copy's own builds left is kept, and the setup commands run again if anything
  changed. A copy without such a record is compared file by file.
- The test: `a_changed_test_file_reaches_a_copy_that_is_used_again` adds an expectation to a test file
  between two runs on one work directory. With the fix the surviving mutant is killed in the second run;
  with the refresh disabled the test fails with "the new expectation did not reach the copy".
- Check 3 shows a second-order effect: the wrong classes of check 2 were stored under the right key, and
  any later run on that work directory reads them back, a fixed binary included. Only stores written by builds between task 2.1 and the fix can hold such
  entries, which means this machine. Released stores (1.1.x) have another key and are not read.
- Earlier results: R92's three steps and the gate runs of R91 used fresh work directories for base, step
  and pin. Earlier research runs that used a work directory twice were not re-examined here.

Also found and fixed (`9495223`): `step record` printed its refusal in debug form, and the top-level help
described the binary as it was at release 1.0.

## Limits and interpretation

- One step, chosen by the author of the tooling after reading a plan. The refusal was predicted, so this is
  a demonstration, not a surprise.
- The change makes the file larger. It is a readability step and says so in its spec; nothing here is
  evidence about reduction.
- The pins assert that a misshapen file is treated as corrupt. Whether that is what the project wants for
  every case (a newer schema version, for example) is the project's decision; `step pin` cannot know.
- The fix copies changed files into the copy and reruns setup. It does not clean build outputs in the copy;
  a build tool that does not notice a changed input would still be wrong. None of the four languages tried
  showed that.
- One more replay attempt started from a clone whose branch already held the pins. It was accepted twice
  and shows nothing; it is not in the table.
- Not shown on a real step, as in R92: an override, a release entry, a frame specification, an
  out-of-reach mark, and a push through the installed hook.

## Tool/spec consequence and next gate

- `docs/driver.md` says that copies follow their source from 1.2.0 and that 1.1.x needs a fresh work
  directory after a change to tests. The release notes say the same under "Fixed".
- `docs/steps.md` carries this step as the worked example of a refusal.
- For a later spec: a test for every "runs again" claim should assert a changed class, not only a missing
  reuse.

## COBESY check

Outcome first: the step's path and the defect, both in the title. Replay: commits, binaries, work
directories and files per run. Disproof: the first prediction could have failed; the second did fail, and
the entry says why. The wrong verdicts are kept and labelled, not removed, and so is the mistake with the
binary: three runs that were reported in the session as made with the fix were not.
