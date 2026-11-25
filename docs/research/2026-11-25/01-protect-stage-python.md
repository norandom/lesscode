# R190: the brownfield test stage in a Python project, on scitools-hook (task 5.2)

- Report date/time and timezone: 2026-11-25, written before the plan at about 10:00 Europe/Berlin.
- Execution interval: not run yet
- Status: running
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 5.2 (can fail)
- Hypothesis ID and predecessor: [R189](../2026-11-24/10-protect-stage-shared-frame.md) (the stage on
  Rust code, with the fix of task 6.5)
- Type: preregistered (the expectations below were written before the plan)

## Decision

Not reached yet.

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

Run in progress.

## Limits and interpretation

Written after the run.

## Tool/spec consequence and next gate

Written after the run.

## COBESY check

Written after the run.
