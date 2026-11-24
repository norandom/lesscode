# R188: the brownfield test stage with its two commands on yuv.rs (task 5.1)

- Report date/time and timezone: 2026-11-24, written before the run and committed at 16:08:01
  Europe/Berlin (`e41cabf`); the plan started at 16:08:07. (Corrected the same day: the first
  version said 16:20, a time I had not read from the clock.)
- Execution interval: 2026-11-24, 16:08:07 to 18:23:27 Europe/Berlin; the owner's review waited
  from 17:17:13 to 18:23:13. The sections from "Decision" down to the end were written at about
  18:25, after the run.
- Status: completed
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 5.1 (can fail)
- Hypothesis ID and predecessor: [R184](05-brownfield-stage-prototype.md) (the stage by hand on
  `display_path.rs`), [R187](08-protect-skill.md) (the skill)
- Type: preregistered (the expectations below were written before the plan was started)

## Decision

The two commands and the skill carry a stage on a second file from plan to result with no step
done by hand: `step protect begin`, a base run, one pinned test, `step check`, the owner's one
review and `step protect result`, which ended with exit 0, "Protected enough". Eight of the ten
expectations hold. Two fail, and both are about the stage's yield and length, not about whether it
works:

- E8: one of the twelve faults the tests missed ends noticed. The other eleven blank wgpu debug
  labels that no caller can see, so no test could notice them, and they had to be written off.
- E9: the first pin with a test took 11 min 21 s, over the limit of 10 minutes, because it built
  its own copies after the base run.

This permits closing task 5.1. It also shows that the Rust fault pack plants faults that only cost
the owner's time: here 11 reviews for one real gap. The owner decided what follows (below).

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

The numbers below are read from the outputs as they came, during the run.

- Plan: 16:08:07 to 16:25:16 (17 min 9 s), exit 0, two copies, the owner watching on the page.
  30 faults planted in `yuv.rs`, 18 noticed, 12 unnoticed: `convert` 8 and 6, `new` 4 and 5,
  `uniform_bytes` 4 and 1, `reference_rgba` 2 and 0; `chroma_size`, `coefficients`, `sample` no
  fault planted. The same numbers as the earlier plan in task 5.1.
- The owner chose `convert`, `new`, `uniform_bytes` at about 16:27.
- `step protect begin` at 16:28:02: exit 0 in 537 ms; `git status` shows `step.json` and
  `stage.json` only (E1 holds). It announced 28 faults to run again, about 12 min 11 s for each of
  the base run and the check, and 4.3 GiB for one copy (measured on `work\plan\w1`; the two copies
  measured 4,407.4 MiB each when they were removed).
- **11 of the 12 unnoticed faults are debug labels.** Each `arg_literal:string->empty` blanks the
  label of a wgpu object (`"yuv-convert"`, `"yuv-y"`, `"yuv-u"`, `"yuv-v"`, `"yuv-uv"`), which only
  graphics debuggers and wgpu's validation messages show. The twelfth, `drop_statement@114:8` in
  `uniform_bytes`, drops the flag that tells the shader an NV12 frame's chroma is interleaved; the
  GPU test of the file converts planar frames only. So the plan ranked `convert` and `new` as weakly
  protected where their pixels are well protected. Read before any test was written.
- Stage spec committed in the clone as `49b422a` (16:29:59): one test task, one task of eleven
  proposals.
- Base run (`step pin --fault 1`): 16:30:05 to 16:47:11, **17 min 6 s against 12 min 11 s
  announced, a factor of 1.40** (E2 holds, inside 1.5; the estimate is low). Exit 3, as the skill
  allows: fault 1 did not build, and the error lists the twelve unnoticed faults with their
  numbers; the NV12 flag is #5, the labels #6, 7, 10, 11, 14 (`new`) and #16 to 19, 22, 23
  (`convert`). The numbers have gaps where faults did not build or were noticed.
- Disk during the base run, sampled every 30 s: at most 8.66 GiB in `work\stage`, against two
  copies of 4.3 GiB plus a quarter, 10.75 GiB (E3 holds). D: went from 119.2 to 111.5 GiB free.
- The test, tried once in the base run's built copy `w1` before it was committed: the first try
  built nothing (0.78 s) and ran the four old tests only, because the copied file kept the draft's
  older time, so Cargo saw nothing new. With the file's time set to now it built (22 s) and all
  five tests of `yuv` passed. The same defect as the stale Linux runs of R187, met by hand: a
  check of the number of tests caught it. Only the `yuv` tests were run there; `step pin` runs
  the whole suite.
- Test committed alone in the clone: `dc8bc1a` (16:48:23).
- `step pin --fault 5`: 16:48:28 to 16:59:49, exit 0, **admitted** ("the tests with the pin pass
  on the unchanged code and fail on the fault"); the eleven labels still unnoticed. **11 min
  21 s, over the 10 minutes of E9.** The pin built its own two copies (`work\stage\pin-run`) from
  nothing, a full debug build of the app, beside the base run's copies. In R184 the first pin had a
  test already and built both sets in one run of 17 min 50 s; here the base run was its own step,
  as the skill now says, and the first pin with a test paid for its copies after it. The sum is
  about the same (28 min 27 s against 17 min 50 s plus the later pins), but `step protect begin`
  and the skill announce only the base run as long.
- Four copies at once after the pin: 17.2 GiB (`base-run` and `pin-run`, two each, 8.61 GiB per
  pair). That is within four times the figure plus a quarter, but neither `step protect begin` nor
  the skill says that the base run's copies and the pins' copies exist side by side. The pin's
  copies were removed after the one pin (D: back to 112 GiB free).
- The project ignores `*.log` (`.gitignore:3`), so the skill's "keep `pins/pin-<n>.log`" with the
  stage spec does not hold here without forcing it in. The log is kept with the evidence outside
  the repository, and `tasks.md` says so (`4d21edc`).
- Write-off proposals committed with the pin receipt: `0bb194a`. All eleven of kind
  `nothing_a_caller_sees`, `approved_by` and `approved_at` `null`.
- `step check`: 17:00:55 to 17:17:03, **exit 0, accepted** (E5 holds). 16 min 8 s against 12 min
  11 s announced, a factor of 1.32. At most 17.23 GiB in `work\stage` (the base run's two copies
  and the check's two). Verdict committed: `9508f87`.
- `step protect result` before the review: **exit 1**, "Not protected enough: 11 fault(s) wait",
  each label "its write-off is not approved" (first half of E6 holds). Per function, noticed
  before, noticed now, written off, waiting: `convert` 8, 8, 0, 6; `new` 4, 4, 0, 5;
  `uniform_bytes` 4, 5, 0, 0. Counted by hand from `verdict.json` (`before`, `after`,
  `unnoticed_after`): the same (E7 holds).
- `git diff --name-only 4fff14f..protect/protect-yuv`: eight files of the stage spec and
  `src-tauri/src/renderer/yuv/protect.rs`, no code file (E4 holds). No commit of the agent has a
  value in `approved_by` or `approved_at` (E10 holds so far).
- E8 fails: 1 of the 12 unnoticed faults ends noticed, against at least half. The other eleven
  are the debug labels above.
- The owner's review (after the clear of 18:18): the owner said "write-offs reviewed and
  committed, continue with the result step", but the clone still stood at `9508f87` with every
  `approved_by` `null`; I said so and ran nothing. The owner then ran the approval script
  themselves (the first try failed because the `!` prefix runs Git Bash, which drops backslashes;
  with forward slashes it ran). Commit `6602eb1` at 18:23:13, author the owner: all eleven
  approved, "Marius Ciepluch", 2026-11-24.
- `step protect result` after the review (frozen binary, hash checked again): 18:23:27, **exit 0
  in 851 ms**, "Protected enough: nothing waits in any chosen function" (second half of E6 holds).
  Per function: `convert` 8, 8, 6, 0; `new` 4, 4, 5, 0; `uniform_bytes` 4, 5, 0, 0. Log
  `...\yuv\result-2.log`. `protection.json`, the result in the stage's `tasks.md` and its
  `spec.json` committed in the clone as `2557598`. In `spec.json` the requirements and tasks phases
  are left unapproved: the owner's review named the write-offs, not the spec phases.
- E10 holds: `0bb194a` wrote `null` in both fields; the only commit with values is `6602eb1`, made
  by the owner's own run of the script.
- E9, measured: from the plan to the verdict, 16:08:07 to 17:17:13, 1 h 9 min, and the result
  under a second, inside 3 hours; the pin of 11 min 21 s fails the second half.
- Storage after: D: 119.5 GiB free; the stage's evidence under `...\yuv` is 0.03 GiB. No working
  copy is left.

### The owner's decision on the stage's scope (2026-11-24, during the run)

Asked "which functions are you going to make smaller", the owner answered with all three, then
said: "generally we do the fail injection brownfielding because we want a better reach for
lesscode" and "the idea of lesscode isn't per function scoping". Offered three ways, the owner
chose: the stage protects the whole planned scope by default, every function of it with an
unnoticed fault; naming functions is an option to narrow it. This run is not affected: the three
functions are all of `yuv.rs` with unnoticed faults. The design is amended after this task;
`step protect begin` gets `--function` as an option, and the skill and the guide are reworded.

### The owner's decisions after the run (2026-11-24, about 18:20)

- Asked how to close the task: the eleven approved first, so that the run closes as recorded; the
  fault pack is changed after it.
- The Rust fault pack plants no `arg_literal:string->empty` in a struct field named `label` (the
  descriptor convention of wgpu and egui). Other string literals stay.
- The owner has not yet seen a full run where the stage gets tests written, and named that as the
  reason: "I still have not seen one full run, given that we somehow don't generate the tests". On
  `yuv.rs` the stage wrote one test, because one fault was there for a test to catch. That run
  happens on another drift-rdp file with several faults a caller can see, after `--function` is
  optional and the label faults are gone. The agent proposes the file with the plan's numbers
  before it starts.
- The owner asked: "the question is whether a fault injection that is successful must always
  prevent us", meaning whether every fault the tests miss must block. I proposed three cases for
  the design amendment, and the owner has not decided yet:
  - a fault a caller sees, in code the reduction changes, blocks;
  - a fault no caller can see is not planted, or is set aside without a review;
  - a fault a caller sees, outside the code the reduction changes, is reported and does not block.

## Limits and interpretation

- One file, three functions, 28 faults run again, one language, one machine. A project without
  Kiro was not tried here.
- The stage's yield on this file says more about the fault pack than about the tests. The GPU
  test of the file already noticed every fault in the pixels. What was left was eleven labels and
  one flag for NV12.
- The one test asks the real GPU; on a machine without an adapter it passes by its second
  allowed answer, which was not run.
- The agent wrote the test and also judged that no test could reach the labels. The owner
  approved all eleven in one step, by the script, without editing an entry.
- The time estimate of `step protect begin` was low twice (1.40 and 1.32 times), and it does not
  announce the first pin's own build or the copies that stand side by side.

## Tool/spec consequence and next gate

Task 5.1 is done. Before 5.2:

1. A dated design amendment: the whole planned scope by default, `--function` optional
   (`native/src/step/protect_begin.rs`, `BeginArgs`); the owner's question about what blocks, as
   the three cases above, once the owner decides.
2. The Rust pack plants no empty string in a `label` field.
3. The skill, `docs/steps.md` and `docs/releases/2.2.0.md` reworded, with the findings of this
   run:
   - the first pin with a test builds its own copies, and they stand beside the base run's;
   - the estimate runs 1.3 to 1.4 times low;
   - a project may ignore `*.log`;
   - a copied file keeps its old time and Cargo rebuilds nothing, so count the tests;
   - `begin` prints mixed path separators.
4. A full run where the stage gets tests written, on another drift-rdp file the owner approves.

Then 5.2 (a project that is not Rust, named by the owner) and 5.3.

## COBESY check

The outcome stands first, with the two failed expectations named and what caused each. The
approvals the owner first reported were not yet in the clone. That is recorded as found, with what
I did: nothing until the commit was there. The owner's question about what blocks is recorded as open,
with my proposal marked as mine.
