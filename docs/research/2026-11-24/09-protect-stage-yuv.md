# R188: the brownfield test stage with its two commands on yuv.rs (task 5.1)

- Report date/time and timezone: 2026-11-24, 16:20 Europe/Berlin (from the clock), written before the run
- Execution interval: not run yet
- Status: running
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 5.1 (can fail)
- Hypothesis ID and predecessor: [R184](05-brownfield-stage-prototype.md) (the stage by hand on
  `display_path.rs`), [R187](08-protect-skill.md) (the skill)
- Type: preregistered (the expectations below were written before the plan was started)

## Decision

Not reached yet.

## Hypothesis and falsification

The skill `protect-before-reduce`, with `step protect begin` and `step protect result` in place of
what R184 did by hand, carries a stage on a second file of the renderer from plan to result. The
owner named the file: `src-tauri/src/renderer/yuv.rs`. The owner names the functions after the
plan and does the one review.

| # | Expectation | Rejected when |
| --- | --- | --- |
| E1 | `step protect begin` exits 0 for the functions the owner names, and writes `step.json` and `stage.json` into the stage spec's directory and nothing else | another exit code for a valid choice, or `git status` shows another file |
| E2 | the base run (the first `step pin`) takes about what `step protect begin` announced | the time on the clock is under half or over one and a half times the announced time |
| E3 | the disk figure of `step protect begin` holds: the copies of the stage take at most the number of copies times the figure, plus a quarter | the work directory of the stage measures more at its largest |
| E4 | every task's test is admitted by `step pin` before the task is marked done, and the branch changes test files and the stage spec only | a task done without an admitted pin, or `git diff --name-only <base>..protect/<name>` names a code file |
| E5 | `step check` exits 0 on the branch with the tests | exit 1 or 2 |
| E6 | `step protect result` exits 1 while a proposal waits, and 0 after the owner's committed approvals (or 0 at once where nothing is proposed) | another exit code at either point, or a result that counts an approval that is not committed |
| E7 | the four numbers per function in `protection.json` agree with the verdict, counted by hand from `unnoticed_before` and `unnoticed_after` | one number differs |
| E8 | at least half of the unnoticed faults of the chosen functions that build on Windows end noticed; every other has a proposal (as G3 of R184) | fewer than half, or a waiting fault without a proposal |
| E9 | the stage takes at most 3 hours from the plan to the result, without the owner's waiting time; no pin after the base run takes over 10 minutes | more |
| E10 | the agent never writes `approved_by` or `approved_at` | the agent's commit holds a value in either |

What the plan of the task suggested (task 5.1, from an earlier plan): `convert` 8 noticed and 6
unnoticed, `new` 4 and 5, `uniform_bytes` 4 and 1. `new` and `convert` take a `wgpu` device; a
test of them needs a GPU adapter, as one test of R184 did.

## Frozen inputs and method

- Subject: drift-rdp at `09b4ce8`, the clone `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`
  (made for R184; `D:\Source\drift-rdp` is only read). Branch `stage-base/yuv`: `4fff14f` the joining
  line `#[cfg(test)] mod protect;` at the end of `yuv.rs` and an empty
  `src-tauri/src/renderer/yuv/protect.rs`, approved by the owner on 2026-11-24 before the plan.
- Tool: `lesscode-core` release build of `45d8039` (the code of `22630db`), frozen at
  `D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv\bin\lesscode-core.exe`, SHA-256
  `93FC6ED7D045FA4E8728C9A56BFFCBB47CBA17263261215EDEE1DB4828CD969B`.
- Run configuration: `D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv\run.json`, the one of R184
  with `files` set to `yuv.rs` and the test paths `src-tauri/tests/` and
  `src-tauri/src/renderer/yuv/`; test command `cargo nextest run --locked --manifest-path
  src-tauri/Cargo.toml --lib --no-fail-fast`, two copies, `CARGO_TARGET_DIR=target` in each copy.
- Work: `...\yuv\work\plan` for the plan, `...\yuv\work\stage` for everything after it.
- The owner watches the plan on the page (`--open`), by their choice.
- Storage before: D: 119.6 GiB free.

## Evidence and results

Not run yet.

## Limits and interpretation

Written after the run.

## Tool/spec consequence and next gate

Written after the run.

## COBESY check

Written after the run.
