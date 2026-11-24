# R189: the stage over the whole scope of shared_frame.rs, where tests get written (task 6.4)

- Report date/time and timezone: 2026-11-24, written before the plan of the stage at about 19:45
  Europe/Berlin (from the clock).
- Execution interval: not run yet
- Status: running
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 6.4 (can fail)
- Hypothesis ID and predecessor: [R188](09-protect-stage-yuv.md) (the stage on `yuv.rs`: one test,
  eleven labels written off), [R184](05-brownfield-stage-prototype.md) (by hand)
- Type: preregistered (the expectations below were written before the plan on the stage's base)

## Decision

Not reached yet.

## Hypothesis and falsification

The owner has not yet seen a run in which the stage gets several tests written: "I still have not
seen one full run, given that we somehow don't generate the tests". In R188 one fault was there
for a test to catch. This run takes a file with several faults a caller can see, after the
amendment of 2026-11-24: the whole scope by default (`step protect begin` without `--function`),
reported functions, no label faults in Rust. The owner chose `src-tauri/src/renderer/shared_frame.rs`
of drift-rdp.

A plan of the file on drift-rdp's `master` (`09b4ce8`), made before this entry to choose the
file, planted 42 faults: 34 noticed, 8 unnoticed, all in the functions the plan calls
`update_rect` (5) and `update_full` (3). Read in the code before any test, four of the eight
change something a caller sees:

- `swap:<-><=@93:21` and `swap:&&->||@93:33`: an empty rectangle adds damage;
- `swap_next` at line 127: data that is too short still resizes the frame;
- `drop_statement` at line 211: `SharedFrame::update_rect` no longer marks the frame dirty.

The other four look equivalent or invisible in one thread: `>=`→`>` at line 100 (the bounds
check below skips the row), `resize(expected, 1)` at line 133 (every byte is overwritten), and
`drop(guard)` removed at lines 199 and 210 (the lock is released a moment later).

| # | Expectation | Rejected when |
| --- | --- | --- |
| E1 | the plan on the stage's base gives the same 42 faults, 34 noticed and 8 unnoticed, as the plan on `master`: the joining line adds no fault | another count or another unnoticed id |
| E2 | `step protect begin` without `--function` exits 0, chooses `update_rect` and `update_full` and no other, reports nothing, and writes `step.json` and `stage.json` only | another exit code, another choice, a reported function, or another file in `git status` |
| E3 | the base run takes at most what `begin` says to allow (1.4 times its figure) | longer |
| E4 | at most twice the plan's copies, times what `begin` measured for one, plus a quarter, stand in the stage's work directory at once | more at its largest |
| E5 | at least three tests are written, each admitted by `step pin` before its task is marked done, and the branch changes test files and the stage spec only | fewer, a task done without an admitted pin, or a code file in `git diff --name-only <base>..protect/<name>` |
| E6 | the four faults above end noticed; each of the other four has a proposal to write off | one of the four still unnoticed, or a waiting fault without a proposal |
| E7 | `step check` exits 0 | 1 or 2 |
| E8 | `step protect result` exits 1 while a proposal waits, and 0 after the owner's committed approvals | another exit code at either point |
| E9 | the four numbers per function in `protection.json` agree with the verdict, counted by hand | one differs |
| E10 | the agent never writes `approved_by` or `approved_at` | a commit of the agent holds a value in either |
| E11 | the stage takes at most 3 hours from the plan to the result without the owner's waiting; no pin after the first pin with a test takes over 10 minutes | more |

Known before the run: the plan names functions by name only, so `WriteGuard::update_rect` and
`SharedFrame::update_rect` are one row, as are the two `update_full`. And drift-rdp's GPU test
`renderer::yuv::tests::the_gpu_conversion_matches_the_reference` aborted once in each of the last
two plans (`0xc0000409`) while two copies ran at once; the copy's second attempt passed both times.

## Frozen inputs and method

- Subject: drift-rdp at `09b4ce8`, in the clone `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`
  (`D:\Source\drift-rdp` is only read). Branch `stage-base/shared-frame`, `a8f3442`: the line
  `#[cfg(test)] mod protect;` at the end of `shared_frame.rs` and a `shared_frame/protect.rs` with
  one doc line, approved by the owner on 2026-11-24 before the plan.
- Tool: `lesscode-core` release build of `33a6273`, frozen at
  `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6\bin-6.4\lesscode-core.exe`, SHA-256
  `52037674595DA0D18009EEE77840EE2AF63D3E3E224F36FBFCFE078A1D19F8B6`.
- Run configuration: `...\stage-6\run-shared-frame.json`, the one of R188 with `files` set to
  `shared_frame.rs` and the test paths `src-tauri/tests/` and `src-tauri/src/renderer/shared_frame/`;
  test command `cargo nextest run --locked --manifest-path src-tauri/Cargo.toml --lib
  --no-fail-fast`, two copies, `CARGO_TARGET_DIR=target` in each copy.
- Work: `...\stage-6\work\shared-frame\plan` for the plan, `...\stage-6\work\shared-frame\stage`
  for everything after it.
- The skill [protect-before-reduce](../../../.agents/skills/protect-before-reduce/SKILL.md) as of
  `33a6273`, walked phase by phase; the agent writes the stage spec and the tests.
- Storage before: D: 119.7 GiB free.

## Evidence and results

Run in progress; the numbers below are read from the outputs as they came. Logs and receipts:
`D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6\` (`sf-plan.*`, `sf-begin.log`,
`sf-pins\`, `sf-check.*`, `sf-result-1.log`).

- **Plan on the base** (`a8f3442`): 19:45:08 to 20:04:52 (19 min 44 s), exit 0. 42 faults, 35
  noticed, 7 unnoticed. **E1 is rejected as written**: `master` gave 34 and 8. The difference is
  `swap:>=->>@100:18`, noticed here (exit 100) and unnoticed on `master`. Planted by hand in the
  plan's built copy, all 536 tests passed, so the plan's kill was false. The outcomes keep no test
  log, so which test failed is not known. The likeliest is the GPU test that aborted with
  `0xc0000409` in the first runs of both earlier plans. The other 41 faults have the same class in
  both plans.
- **`step protect begin` without `--function`**, 20:05: exit 0 in 366 ms. Chosen `update_rect`
  (noticed 9, unnoticed 4) and `update_full` (14, 3); reported none; `git status` shows the stage
  spec's directory only (**E2 holds**). It announced 30 faults to run again, about 12 min 6 s,
  "allow about 16 min 57 s", 4.3 GiB per copy, and "allow for 4 copies at once, about 17.2 GiB".
- Stage spec committed in the clone, `51e4b94`: two requirements, three test tasks and one task of
  proposals.
- **Base run** (`step pin --fault 1`): 20:07:01 to 20:23:02, 16 min 1 s, 1.32 times the figure and
  inside the allowance (**E3 holds**). Exit 3 as the skill allows: fault 1 is noticed. The list
  of unnoticed faults has **eight**: the seven of `stage.json` and #6, line 100. So the base run
  agrees with the hand check, and `stage.json` does not have #6.
- The three tests, tried first in the base run's built copy (the file's time set to now): 19 tests
  of `shared_frame` passed, and the suite counted 545 where it had 542.
- **Pins**, each test committed alone first (`ffe26d5`, `c9eeae3`, `c1a1372`):
  - #2 (`swap:<-><=@93:21`): admitted, 9 min 57 s, the first pin with a test, building its own
    copies;
  - #3 (`swap:&&->||@93:33`): admitted, 94 s;
  - #11 (`swap_next@127:8`): admitted, 5 min 35 s;
  - #30 (`drop_statement@211:8`): admitted, 5 min 26 s.

  No later pin took over 10 minutes. Three tests were written and each was admitted before its
  task was ticked. `git diff --name-only a8f3442..protect/protect-shared-frame` names the stage
  spec and `shared_frame/protect.rs` only (**E5 holds**). Receipts and three proposals are in
  `e7adbaf`.
- **`step check`.** The first try refused at once (exit 3): the skill's phase 8 gives a bare
  `--page`, which would put the page inside the repository. With `--page` outside it: 20:48:03 to
  21:04:52, 16 min 49 s, 1.39 times the figure and inside the allowance, **exit 0, accepted**
  (**E7 holds**). Verdict committed: `23f1ce9`.
- **Disk: E4 is rejected.** At most 25.92 GiB in the stage's work directory, during the check,
  against the bound of 21.5 GiB. Three pairs of copies stood at once: the base run's, the pins'
  and the check's. Neither `begin` nor the skill says to remove the pins' copies before the check.
- **`step protect result` before the review**: exit 1 in 935 ms, "Not protected enough: 4
  fault(s) wait" (**first half of E8 holds**). Per function, as noticed before, noticed now,
  written off, waiting: `update_rect` 8, 11, 0, 2; `update_full` 14, 15, 0, 2. Counted by hand
  from `verdict.json`, the same (**E9 holds**). Three faults wait for approval. The fourth is #6:
  "it is not among the unnoticed faults the stage began with".
- **E6.** The four faults read as visible end noticed (#2, #3, #11, #30), and the three equivalent
  faults of `stage.json` have proposals. #6 waits without one, and cannot get one:
  `write-offs.json` refuses a fault the stage does not have (bad input, exit 3). So E6 holds for
  the faults of `stage.json` and is rejected for #6. As the tool stands, this stage cannot end
  with exit 0.
- **E10 holds so far**: every proposal has `approved_by` and `approved_at` `null`.
- **E11, time.** From the plan to the result, 19:45:08 to 21:05, 1 h 20 min, inside 3 hours. No
  pin after the first one with a test took over 10 minutes. **E11 holds.**
- Storage: the copies are removed; 0.02 GiB is left in the stage's work directory, and D: has
  119.7 GiB free.

## Limits and interpretation

Written after the run.

## Tool/spec consequence and next gate

Written after the run.

## COBESY check

Written after the run.
