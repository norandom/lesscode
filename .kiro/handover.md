# Handover: the brownfield test stage on `yuv.rs` waits for the owner's one review

Reset 2026-11-24, 18:18 Europe/Berlin (from the clock). This file is the start for the next session.
The version before this one is in Git (`9f6aab7`, the reset of 23 November about `SharedFrame` and
R173); older ones are [handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). The group handovers since then
are [task_84](task_84_handover.md) to [task_87](task_87_handover.md); the next is
`task_88_handover.md`, written when task group 5 of the stage is finished.

Nothing is running. All working copies are removed.

**Update 2026-11-24, 18:25:** task 5.1 is done. The owner approved the eleven write-offs
(`6602eb1` in the clone). `step protect result` exited 0, and its result is committed in the clone
as `2557598`. R188 is completed. Steps 1 and 2 of "What the next session does" below are done.
The owner decided:

- the Rust pack plants no empty string in a `label` field;
- the full run where tests get written is on another drift-rdp file that I propose with plan
  numbers.

Still open: the owner's question whether every fault the tests miss must block. My proposal of
three cases is in R188.

## Start here

1. Read [the stage spec's tasks](specs/brownfield-test-stage/tasks.md). Groups 1 to 4 are done;
   task 5.1 is open and waits for the owner, 5.2 and 5.3 follow.
2. Read [R188](../docs/research/2026-11-24/09-protect-stage-yuv.md), the record of task 5.1. Its
   expectations E1 to E10 were written before the run; the evidence is filled in up to the review.
3. Read the skill [protect-before-reduce](../.agents/skills/protect-before-reduce/SKILL.md) and the
   guide's section [Protect what you will change](../docs/steps.md#protect-what-you-will-change-step-protect).

## Where task 5.1 stands

The stage ran with `step protect begin` and `step protect result` on
`src-tauri/src/renderer/yuv.rs` of `drift-rdp`. It ran in the clone
`D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage` (drift-rdp at `09b4ce8`), and
`D:\Source\drift-rdp` was only read.

- Branch `stage-base/yuv`, `4fff14f`: the joining line `#[cfg(test)] mod protect;` at the end of
  `yuv.rs` and an empty `src-tauri/src/renderer/yuv/protect.rs`. The owner approved it before the plan.
- Branch `protect/protect-yuv`:
  - `49b422a`: stage spec;
  - `dc8bc1a`: the one test, `an_nv12_frame_converts_on_the_gpu_as_the_reference_does`;
  - `0bb194a`: pin receipt `pins/pin-5.json` and eleven write-off proposals;
  - `4d21edc`: where the log is kept;
  - `9508f87`: the accepted verdict.
- Not committed: `.kiro/specs/protect-yuv/protection.json`, written by `step protect result`,
  exit 1.
- The owner chose the functions `convert`, `new` and `uniform_bytes`. They are all the functions
  of the file with unnoticed faults.

Numbers, each from a run and seen:

- **Plan.** 30 faults: 18 noticed, 12 unnoticed.
- **`step protect begin`.** Exit 0 in 537 ms. It announced 28 faults, about 12 min 11 s per run,
  and 4.3 GiB per copy.
- **Base run** (the first `step pin`). 17 min 6 s, 1.40 times the announced time.
- **The pin of the test** (fault #5, `drop_statement@114:8`, the NV12 "interleaved" flag in
  `uniform_bytes`). Admitted in 11 min 21 s.
- **Check.** Accepted, 16 min 8 s.
- **`step protect result`.** Exit 1, 11 waiting. Per function, as noticed before, noticed now,
  written off, waiting:

  | Function | Noticed before | Noticed now | Written off | Waiting |
  | --- | --- | --- | --- | --- |
  | `convert` | 8 | 8 | 0 | 6 |
  | `new` | 4 | 4 | 0 | 5 |
  | `uniform_bytes` | 4 | 5 | 0 | 0 |

  Counted by hand from the verdict, the same.
- **The eleven waiting faults** each blank a wgpu debug label (`"yuv-convert"`, `"yuv-y"`, and so
  on). Each has a proposal of kind `nothing_a_caller_sees`.

How the expectations stand:

- **Hold:** E1, E2, E3, E4, E5, E7, E10, and the first half of E6.
- **Fail:**
  - E8: 1 of 12 noticed, the rest are labels.
  - E9: the first pin with a test built its own copies after the base run and took over 10 min.

### What the next session does

1. **Wait for the owner's review.** It was presented in the conversation of 24 November in the
   skill's order. The agent **never** writes `approved_by` or `approved_at`, not even from an
   answer in the conversation. The owner edits
   `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage\.kiro\specs\protect-yuv\write-offs.json`
   and commits on `protect/protect-yuv`, or refuses entries, or asks for tests. A script for the
   owner to run themselves is at
   `D:\BuildArtifacts\Temp\claude\1388d6fe5280a805\dadbd800-63ff-441e-ab6c-256d188c9d63\scratchpad\approve-write-offs.ps1`.
   It may be gone after a clear. If so, the owner edits the file by hand. Check
   `git -C <clone> log --oneline -3` for the owner's commit.
2. **After the owner's commit:** run `step protect result` again with the frozen binary (below);
   expect exit 0, the second half of E6. Then:
   - commit `protection.json` and the result section of the stage's `tasks.md` in the clone;
   - update its `spec.json`;
   - finish R188: execution interval, decision, the owner's words, limits, consequences, COBESY;
     status completed;
   - update the research index;
   - tick 5.1 in the stage tasks.
3. **The owner's scope decision of 24 November.** In the owner's words: "we do the fail injection
   brownfielding because we want a better reach for lesscode", and "the idea of lesscode isn't per
   function scoping". The owner chose "whole scope by default". By default the stage protects
   every function of the planned scope that has an unnoticed fault; naming functions narrows it.
   To do:
   - a dated design amendment;
   - `--function` optional in `step protect begin` (now `required = true` in `BeginArgs`,
     `native/src/step/protect_begin.rs`);
   - reword the skill, `docs/steps.md` and `docs/releases/2.2.0.md`.
4. **Findings of 5.1 for the skill, the guide and `begin`:**
   - announce that the first pin with a test builds its own copies after the base run, and that
     the base run's copies and the pins' copies exist side by side (17.2 GiB at once here);
   - the time estimate runs low, 1.3 to 1.4 times;
   - a project may ignore `*.log` (drift-rdp does), so the pin log cannot always be committed;
   - a file copied into a built copy keeps its old time, and Cargo rebuilds nothing; check the
     number of tests;
   - `begin` prints mixed path separators (`work/plan\w1`).
5. **For the owner to decide:** 11 of 12 unnoticed faults in `yuv.rs` are debug labels, which is
   noise for lesscode's reach. Should the Rust pack stop planting `arg_literal:string->empty` on
   wgpu `label:` fields?
6. Then 5.2: the stage in a project that is not Rust, a project the owner names. Then 5.3, the
   validation and `task_88_handover.md`.

## Frozen inputs and evidence of 5.1

- Binary: `D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv\bin\lesscode-core.exe`, release of
  `45d8039`, SHA-256 `93FC6ED7D045FA4E8728C9A56BFFCBB47CBA17263261215EDEE1DB4828CD969B`.
- Run configuration `…\yuv\run.json`. Logs and receipts: `plan.*`, `begin.log`, `check.*`,
  `result-1.log`, and `pins\` (base log, `pin-5.log`, times, disk samples). Under `…\yuv\work\` only
  outcomes and JSON are left.

## State of the repositories

- lesscode: `main` at `b254d8f` plus this handover. `origin/main` is at `9f6aab7`, and 24 commits
  are local. Push only on the owner's word. Release 2.2.0 is written up in
  `docs/releases/2.2.0.md` and is not tagged.
- `drift-rdp`: `master` untouched at `09b4ce8`. Clones and worktrees of mine:
  - `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`: branches `protect/display-path`
    (R184, merged nowhere), `stage-base/yuv` and `protect/protect-yuv`;
  - `D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`: branch
    `lesscode/shared-frame-wake-tests`;
  - the local branch `lesscode/clipboard-lease-machine` (`1a49d70`).

  None of them is merged.
- `openidea-kiro`: untouched.

## Carried, open

- Keep or revert the change in `.dagger/main.go` (`38d337e`). It gives the core's files the time of
  the run, so that every Linux run builds the core. The owner has not seen it. Do not edit `native/`
  while a Dagger run is going, and do not stop one half-way.
- The end screen of the run page.
- The state-metrics extension in openidea-kiro and the stopping-rule test of R163.
- The hook test that fails under load,
  `the_installed_hook_stops_a_push_without_a_matching_verdict`.
- Offered and not asked for: the "Two Bowls" page
  (https://claude.ai/artifact/6ggrijcAhxUDyiEk3gw8SY) redrawn from the real numbers of `yuv.rs`.
- From the reset of 23 November, still open:
  - `lesscode-metrics` as a command;
  - the cut for `gpu.rs`, which needs the owner's run of the application;
  - a made-up example over several files;
  - Python and facdrone;
  - `ats-agent`;
  - the checks of release 2.1.0 after publication;
  - the main README's introduction;
  - the `measures.rs` parenthesized-callee fix;
  - Java lowering;
  - `match` arms as branch conditions in the Rust pack.
- Drift hardening (Slice B): closed by the owner; only the drift of one model over time is
  unmeasured.

## Rules to keep

- Times from the clock. Say things in words, not record numbers.
- Never report a check as done unless it ran and its output was seen.
- Builds under `D:\BuildArtifacts`. Before a heavy job, D: needs at least 50 GiB free.
- PowerShell on Windows, never WSL; Rust tests with nextest.
- One commit per task. Implementation stays in the foreground.

## Storage

D: had 119.5 GiB free at 18:18 with nothing running. One built copy of `drift-rdp` takes about
4.3 GiB, and a stage holds up to four at once. Retained, small: the evidence under
`D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv\` and `D:\BuildArtifacts\lesscode\brownfield-stage`.

Next research record: R189. Next task handover: `task_88_handover.md`.
