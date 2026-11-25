# R190: the brownfield test stage in a Python project, on scitools-hook (task 5.2)

- Report date/time and timezone: 2026-11-25, written before the plan at about 10:00 Europe/Berlin.
- Execution interval: 2026-11-25, 10:01:25 to 11:54:09 Europe/Berlin. The sections "Decision",
  "Limits", "Tool/spec consequence" and "COBESY check" were written at about 12:00, after the run.
- Status: completed
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 5.2 (can fail)
- Hypothesis ID and predecessor: [R189](../2026-11-24/10-protect-stage-shared-frame.md) (the stage on
  Rust code, with the fix of task 6.5)
- Type: preregistered (the expectations below were written before the plan)

## Decision

Nothing of the stage had to be done differently for Python. The two commands, the stage spec,
`step pin`, `step check` and the skill held as written. No joining line was needed, because the
tests live in files of their own. `begin` without `--function` chose the one function with
missed faults. One test was written and admitted. The one fault left changes nothing, and the
owner wrote it off. `step protect result` ended with exit 0.

What did take work was the project's own test command. The suite is written for POSIX, so it
runs in a Linux container. Six of eight expectations hold. E3 fails, because `begin`'s time
figure leaves out the container's start and copy. E7 holds for the stage, with that one
qualification about the test command.

This permits closing task 5.2 and going on to the validation of 5.3.

## Hypothesis and falsification

Task 5.2 asks whether anything of the stage has to be done differently for a language that is
not Rust. The owner chose the Python project scitools-hook, and of four files offered with their
fault counts, `src/scitools_hook/analysis/structure/cycles.py` (162 lines, 22 faults by
`deconvolute mutate`). Its tests live in files of their own under `tests/`, so no joining line is
needed.

Found before this entry, while setting the project up:

- The project's suite is written for POSIX. On Windows, even without the tests that need a
  licensed Understand (`-m "not contract"`), 441 tests fail: symlinks need a privilege
  (`WinError 1314`), and FIFOs and permission bits behave otherwise.
- So the tests run in a Linux container on Docker Desktop. The image holds Python 3.12.12, uv
  0.13.5, git, and the locked dependencies in `/venv`. It runs as a user, because root ignores
  the permission bits the suite checks, and links `/usr/bin/python3`, because the suite's
  stand-in interpreters search only `/bin:/usr/bin`. In it, all 5,330 tests pass, in 2 min 8 s of
  pytest.
- The test command of the run configuration is a PowerShell line that starts the container on
  the copy lesscode made (mounted read-only, then copied inside), with `-n 4` per copy and two
  copies. The exit code comes through: 0 when all pass, 5 when no test ran.

| # | Expectation | Rejected when |
| --- | --- | --- |
| E1 | the plan exits 0 and is complete, with every one of the 22 faults classed | another exit code, an incomplete plan, or fewer faults |
| E2 | `step protect begin` without `--function` exits 0, chooses every function of the plan with an unnoticed fault, and writes `step.json` and `stage.json` only; it needs no joining line | another exit code, a refusal about test paths, or another file in `git status` |
| E3 | the base run and the check take at most what `begin` says to allow | longer |
| E4 | each test written is admitted by `step pin` before its task is marked done, and the branch changes test files and the stage spec only | a task done without an admitted pin, or a code file in the branch's diff |
| E5 | `step check` exits 0 | 1 or 2 |
| E6 | `step protect result` exits 0 after the owner's committed approvals; a fault the plan miscounted is handled as in 6.5 | another exit code |
| E7 | nothing in the stage's commands, files or skill had to be done differently for Python, except what the project's own test command needs (here a container) | a step of the skill that does not hold for Python as written |
| E8 | the agent never writes `approved_by` or `approved_at` | a commit of the agent holds a value in either |

## Frozen inputs and method

- Subject: scitools-hook at `73a736e`, cloned to `D:\BuildArtifacts\scitools-hook\lesscode-stage`;
  `D:\Source\scitools-hook` is only read.
- Tool: `lesscode-core` release build of `7f0324a`, frozen at
  `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6\bin-6.5\lesscode-core.exe`, SHA-256
  `F6F83C73B7CC54A63EB1622E8CEA586EC6CF3FA144648F1277D2B11FDEB9EB3F`.
- Image: `lesscode-stage/scitools-hook-tests:2026-11-25`, built from
  `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-5.2\image\` (Dockerfile and
  `run-tests.sh`) on Docker Desktop 29.9.3, 8 CPUs, 8 GiB.
- Run configuration: `...\stage-5.2\run-cycles.json`.
- Work: `...\stage-5.2\work\plan` and `...\stage-5.2\work\stage`.
- Storage before: D: 208 GiB free.

## Evidence and results

Logs and receipts: `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-5.2\` (`plan.*`,
`plan-1-void\`, `begin.log`, `pins\`, `check.*`, `result-1.log`, `result-2.log`,
`protection-1.json`).

- **First plan, void, by my configuration**: 10:01:25 to 10:10:55, exit 3. Both copies' first runs
  failed twice: 3 failed and 18 errors. I had excluded `.dagger` from the copies, and 21 tests read
  the project's Dagger module. The CLI's tests need a git working tree, and lesscode's copies
  leave `.git` out. Fixed in the run configuration, where `.dagger` is no longer excluded, and in
  `run-tests.sh`, which makes the copied tree a git repository with one commit. In a copy the suite
  then passed: 5,330 tests, 129 s.
- **Plan**: 10:13:49 to 10:53:24, 39 min 35 s, exit 0. All 22 faults classed: 20 noticed, 2
  unnoticed, both in `_finding` (**E1 holds**). `arg_swap@138:8` swaps the keyword arguments
  `kind=` and `rule=` of `Finding(...)`, which is equivalent. `arg_swap@145:25` swaps `members`
  and `closing` in the call of `_message`, which is visible. The existing test only asks that
  `"a.py, b.py, c.py"` appears in the message, and it still does after "closed by".
- **`step protect begin` without `--function`**: exit 0 in 122 ms. Chosen `_finding`, reported
  none, and only the stage spec's directory written. There was no refusal about test paths and no
  joining line (**E2 holds**). It announced 4 faults to run again, about 8 min 18 s, "allow about
  11 min 37 s", and 41.1 MiB per copy.
- Stage spec committed in the clone, `fda78d5`.
- **Base run**: 10:54:46 to 11:09:09, 14 min 23 s, exit 3 as the skill allows. It listed #3 and
  #4 unnoticed. That is 1.73 times the figure and over the allowance.
- The two tests, tried first in the base run's copy: 128 tests of `tests/analysis/structure/`
  passed. Committed alone: `5bdbb3a`.
- **Pin #4**: admitted, 11:09:57 to 11:29:30, 19 min 33 s; the first pin with a test, in copies of
  its own. Receipt and the proposal for #3 committed in `5543663`. One test written and admitted
  before its task was ticked; `git diff --name-only 73a736e..HEAD` names the stage spec and
  `tests/analysis/structure/test_cycles_protect.py` only (**E4 holds**).
- The pins' copies were removed before the check, as the skill now says.
- **`step check`**, `--page` outside the repository: 11:30:27 to 11:52:55, 22 min 28 s, **exit 0,
  accepted** (**E5 holds**). That is 2.7 times the figure. Verdict committed: `a0a45dd`.
  **E3 is rejected** for both long runs.
- **`step protect result` before the review**: exit 1, `_finding` 2, 3, 0, 1, and #3 waiting:
  "its write-off is not approved".
- The owner approved at 11:54:03 by running the approval script themselves (`421c1ef`).
- **`step protect result` after**: 11:54:09, **exit 0**, "Protected enough". `_finding` 2, 3, 1,
  0, the same as the verdict (**E6 holds**). Committed in the clone: `d4c9a6a`.
- **E8 holds**: the only commit with a value in `approved_by` is the owner's.
- **E7, what differed for Python.** The stage itself did not differ. What the project needed:
  - a test command that runs the suite in Linux;
  - an image with a user, because root ignores permission bits;
  - a `/usr/bin/python3` link;
  - a git repository in the copied tree.

  None of it is in lesscode or the skill, and all of it is in the run configuration and the
  image. **E7 holds**, with that qualification.
- Storage: copies removed, 0.04 GiB of evidence, D: 203.0 GiB free. No container is left.

## Limits and interpretation

- One file, one function with missed faults, 4 faults run again. A file whose tests already notice
  20 of 22 faults gives the stage little to do. The run shows that the stage works in Python, not
  how much it adds there.
- The faults ran against the suite without the licensed tests (`-m "not contract"`). A fault that
  only a licensed test would notice counts as unnoticed here.
- The time figure of `begin` comes from the plan's seconds per fault, and the base run and the
  check ran 1.73 and 2.7 times as long. Part of it is the container's start, the copy and the
  install (about 50 s per run). Part of it is two copies at once on 8 CPUs, each with `-n 4`. The
  plan ran with the same setting, so contention alone does not explain the gap.
- The Python pack plants `arg_swap` on keyword arguments, which can never change a call. It cost
  one review here; on a file of keyword-heavy calls it would cost many.

## Tool/spec consequence and next gate

- Task 5.2 is done.
- For the owner to decide later:
  - the Python pack should not swap keyword arguments, as the Rust pack no longer empties labels;
  - `begin`'s time figure should count what one run costs besides its tests. The base run of the
    plan measures that already.
- The image and its script stay with the evidence. They are the test setup of this project, not
  part of lesscode.
- Next: task 5.3, the validation. The suites run on Windows and through Dagger on Linux, now on
  Docker Desktop, where the old images are gone.

## COBESY check

The outcome stands first, separated from the work the project's test command needed. The void
first plan is recorded as my configuration error, with its logs. The failed expectation is named
with its cause.

## Amendment, 2026-11-25, about 13:00 Europe/Berlin

The owner accepted both points left to decide (handover 88, points 2 and 3), and they became task
group 7 of the spec. The reading above, that the container's start and copy made `begin`'s figure
low, is revised by [R191](02-begin-time-figure.md). The container starts in every run of the tests,
so the tests' seconds already hold that. The figure counted one run of the tests where a run makes
three, and it gave the check the base run's figure, where the check runs both the base and the step.
The text above is kept as it was written.
