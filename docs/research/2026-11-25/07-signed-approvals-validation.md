# R196: The tool passes on Windows and Linux; kiro-cli refused two routines of the spec until they were cut down

- Report date/time and timezone: 2026-11-25, about 18:30 Europe/Berlin
- Execution interval: about 17:52 to 18:21 Europe/Berlin (baseline runs on `8dfaf9d` from about
  17:52, the fix `1dc83bf` committed 18:07, Dagger on it 18:14 to 18:21)
- Status: completed
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/tasks.md), task 5.1
- Hypothesis ID and predecessor: [R195](06-signed-approvals-guides.md), the guides
- Type: exploratory (the validation the task names, no preregistered expectation beyond "all pass")

## Decision

Task 5.1 is done at `1dc83bf`. Windows: nextest 551 of 551 (1 skipped), doctests, format, Clippy,
budgets. Linux through Dagger: all 23 steps 0, core 550 of 550 (1 skipped), model 85 of 85.
`kiro-cli code check --changed 09c3076` passes. It did not pass at the group's commit `8dfaf9d`:
two routines the spec added were over the default limits, and `1dc83bf` cuts them down without a
change of behaviour. Tasks 5.2 (the owner's key) and 5.3 (the re-approval) can go on.

## Hypothesis and falsification

H: the tool as the spec left it after group 4 passes the repository's checks on both platforms and
`kiro-cli`'s code check against the commit before the spec (`09c3076`, handover 89). Falsified by
any check with a nonzero exit.

## Frozen inputs and method

- lesscode `main` at `8dfaf9d` (handover 93), then `1dc83bf`. Rust 1.98.1, cargo-nextest 0.9.143,
  four test threads, `CARGO_TARGET_DIR=D:\BuildArtifacts\cargo-target`. The commands of
  [testing.md](../../testing.md) and `tools/check_native.sh`; budgets with
  `native/complexity-budgets.json` over `native/src`.
- Linux: Dagger CLI 0.21.11 on Docker Desktop 29.9.3, `dagger call evidence` then `dagger call
  check`, as [ci-validation.md](../../ci-validation.md) gives them.
- `kiro-cli` 1.1.0 (`D:\BuildArtifacts\lesscode\kiro-cli-1.1.0\x\kiro-cli.exe`), `code check
  --changed 09c3076`, with the repository's `.kiro/code-rules.toml`. The before side is the commit
  before `380eb5f`, the spec's first.
- The Windows run and the first Dagger run on `8dfaf9d` went side by side; the Dagger run took
  1,054 s instead of about 350 s. The second Dagger run had the machine to itself (388 s).

## Evidence and results

| Check | `8dfaf9d` | `1dc83bf` |
| --- | --- | --- |
| Windows nextest | 550 of 550, 1 skipped | 551 of 551, 1 skipped |
| doctests, format, Clippy, budgets | 0, 0, 0, 0 | 0, 0, 0, 0 |
| `kiro-cli code check` | **fail**: 4 errors, 2 of them pre-existing | pass: the 2 pre-existing errors |
| Linux, Dagger | 23 steps 0; core 549 of 549 (1 skipped), model 85 of 85 | 23 steps 0; core 550 of 550 (1 skipped), model 85 of 85 |

The two errors that were new:

- `native/src/step/approvals.rs` `against_forge`: 6 parameters, limit 5. It took the repository
  and the head as two parameters, both read from the `Context` its caller `judge` has. It now takes
  the context.
- `native/tests/step_protect_result.rs` `an_unnoticed_fault_waits_until_its_write_off_is_approved`:
  69 code lines, limit 60; it had 44 before the spec. Groups 2 and 3 added the unsigned, the
  old-way and the signed approval to the one story. It is now two tests, split where the old-way
  approval begins: the first keeps the steps up to the unsigned approval; the second,
  `only_a_signed_approval_writes_off_an_unnoticed_fault`, takes its own stage through the old way,
  the signed approval and the bad kind. Same assertions, one more stage (about 25 s more).

The two pre-existing errors are `step_guard.rs` `a_branch_stands_only_with_a_verdict_for_exactly_its_content`
(72 code lines, 59 statements), unchanged since before the spec.

- Windows logs at `1dc83bf`: `D:\BuildArtifacts\lesscode\signed-approvals\*-5.1-windows.log`,
  exit codes in `exit-codes-5.1-windows.txt`, budgets `budgets-5.1.json`; at `8dfaf9d` the same
  names under `signed-approvals\baseline-8dfaf9d\`, with the failing `kiro-code-check-5.1.log`.
- Linux: `D:\BuildArtifacts\lesscode\native-evidence\linux-1dc83bf` and `linux-8dfaf9d`; Dagger
  logs `signed-approvals\dagger-5.1.log` and `baseline-8dfaf9d\dagger-5.1.log`.
- Budgets: `step/approvals.rs` 2,303 of 2,800 tokens; its longest function is `signatures`
  (45 lines, 387 tokens).

## Limits and interpretation

- Groups 1 to 3 ran the suites after each group but not `kiro-cli`; the plan left that to 5.1. The
  limits were broken when the code was written, and found a group later. Running `kiro-cli` with
  each group's suites would have found them then.
- These checks run the judge only against the stand-in forge. A real signed approval against the
  real forge is task 5.2 and 5.3.
- Storage: D: fell from 139.7 to 111.9 GiB during the runs. Most of it is Docker Desktop's disk
  `D:\Docker\disk\docker_data.vhdx` (89.3 GiB, written at the end of the second run), where the
  shared Dagger engine's cache volume holds 60.6 GB. The engine is shared with other work and was
  not pruned. Both limits hold (at least 50 GiB free).

## Tool/spec consequence and next gate

No change of the design. Next: 5.2, the owner's act with [docs/approvals.md](../../approvals.md).

## COBESY check

The result comes first; the failing check at the group's commit is reported with its two causes
and the fix; the baseline evidence is kept beside the passing run.
