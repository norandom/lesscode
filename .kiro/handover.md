# Handover: the stage on shared_frame.rs waits for a fix of the tool and the owner's approvals

Reset 2026-11-24, about 22:10 Europe/Berlin (from the clock). This file is the start for the next
session. The version before this one is in Git (`4aa5dee`, the reset of 18:18 about `yuv.rs`);
older ones are [handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). The group handovers are
[task_84](task_84_handover.md) to [task_87](task_87_handover.md); the next is
`task_88_handover.md`, written when the open tasks of the stage spec are finished.

Nothing is running. All working copies are removed.

**Update 2026-11-25, about 15:08: task group 2 of `signed-approvals` is done, with 3.1 carried
in.** Start at [task_91_handover.md](task_91_handover.md). Next is group 3 (3.2 and 3.3).

**Update 2026-11-25, about 14:25: the spec `signed-approvals` is written and approved, and its task
group 1 is done.** Start at [task_90_handover.md](task_90_handover.md). Next is group 2 (judging
approvals).

**Update 2026-11-25, about 13:55: task group 7 of `brownfield-test-stage` is done**, from the
owner's decisions on handover 88. The validation addition says GO. Start at
[task_89_handover.md](task_89_handover.md): it lists the owner's decisions of the afternoon (signed
approvals chosen, a spec still to write) and the state of the repositories.

**Update 2026-11-25, about 12:35: the spec `brownfield-test-stage` is finished.** Every task is
ticked, and the validation says GO
([validation.md](specs/brownfield-test-stage/validation.md)). The group handover is
[task_88_handover.md](task_88_handover.md): start there. It lists the open decisions of the owner
and the state of the repositories. The rest of this file is the state of the morning, kept for
its history.

**Update 2026-11-25, about 09:10:** task group 6 is done. Task 6.5 (`7f0324a`) is the fix the owner
chose: `step check` names the base run's unnoticed faults, and `step protect result` takes such a
fault into the stage. With it come the two fixes of the skill. Then the second check of
`shared_frame.rs` was accepted. The owner approved all four write-offs (`0b5fe69`, `39e09d4`), and
`step protect result` exited 0 (clone `cd9e2cd`). R189 is completed. The owner switched from
Podman to Docker Desktop: the old images are gone, and D: has 208 GiB free. Next: task 5.2, the
stage in a project that is not Rust, which the owner names. Then 5.3 and `task_88_handover.md`.
Items 1 to 3 of "What the next session does" below are done.

## Start here

1. Read [the stage spec's tasks](specs/brownfield-test-stage/tasks.md). Groups 1 to 4, task 5.1
   and tasks 6.1 to 6.3 are done. Task 6.4 is open. Tasks 5.2 and 5.3 follow it.
2. Read [R189](../docs/research/2026-11-24/10-protect-stage-shared-frame.md), the record of 6.4,
   up to "The owner's decisions at the review". Its expectations E1 to E11 were written before
   the run.
3. The amendment the owner approved today is at the top of
   [requirements.md](specs/brownfield-test-stage/requirements.md) ("Amended 2026-11-24, 18:35")
   and [design.md](specs/brownfield-test-stage/design.md) ("Amended 2026-11-24, 18:40").

## What was done on 24 November after 18:18

- **Task 5.1, the stage on `yuv.rs` (R188), is done.** The owner approved eleven label
  write-offs, and `step protect result` exited 0.
- **The owner's decisions on the stage.**
  - It protects the whole planned scope by default.
  - A missed fault holds the stage back only where a caller sees it and it lies in a chosen
    function; other functions are reported.
  - The Rust pack plants no empty string in a `label` field.
  - The amendment and task group 6 were approved.
- **The three code and docs tasks:**
  - 6.1 (`1ff09c4`): `--function` is optional, and functions not chosen are reported.
  - 6.2 (`96625d6`): the pack's `skip_strings_in`. A new plan of `yuv.rs` has 23 faults, not 30.
    The 4 labels left are passed through a helper, and the owner accepts writing them off.
  - 6.3 (`33a6273`, `c16ad21`): the skill, the guide and the 2.2.0 notes; `begin` says to allow
    1.4 times its time figure, and twice the plan's copies on disk. The owner read it.
- **Task 6.4 (R189), the run where tests get written,** on `src-tauri/src/renderer/shared_frame.rs`
  of drift-rdp, the file the owner chose.
  - `begin` without `--function` chose `update_rect` and `update_full`.
  - Three tests were written. Each was admitted by `step pin` and the check accepted the branch.
    Four faults the tests missed are now noticed.
  - The result is exit 1 with four faults waiting:
    - three proposals to write off, not approved yet;
    - **#6** (`swap:>=->>@100:18`), which the plan wrongly counted noticed. Planted by hand, all
      536 tests pass; the likeliest cause is the flaky GPU test of drift-rdp. It is not in
      `stage.json`, so it cannot be written off, and as the tool stands the stage cannot end
      with 0.
  - Expectations: E2, E3, E5, E7, E9 and E11 hold. E1 and E4 are rejected; E6 is rejected only
    for #6. E8's second half and E10 wait for the end of the run.

## What the next session does

1. **The fix the owner chose (decision of about 22:05):** `step protect result` treats a fault the
   base run of the check found unnoticed as one of the stage's, so that a test or a write-off can
   take it by its id. Write it as a dated amendment of the design and a task (6.5). Use the same
   way of working as 6.1: test first, the full suite, `fmt`, Clippy, budgets, doctests. Look at
   `native/src/step/protect_numbers.rs`: `write_offs` refuses a fault not in `stage.chosen`, and
   `stand` calls it `NOT_OF_THE_STAGE`. The verdict's `unnoticed_before` is not per function today;
   check what the verdict holds before designing.
2. **The two skill gaps, with the same change:**
   - phase 8 of `protect-before-reduce` gives `--page` a place outside the repository (a bare
     `--page` is refused there with exit 3);
   - the skill removes the pins' copies before the check, and `begin`'s disk figure says that
     otherwise three pairs stand at once (25.92 GiB here).

   The guide `docs/steps.md` says the same.
3. **Finish the stage.**
   - Done: the owner approved the three proposals by running the approval script themselves,
     commit `0b5fe69` in the clone (2026-11-25, 08:37).
   - Propose #6 as `nothing_a_caller_sees`: the bounds check below skips the row. The agent never
     writes `approved_by` or `approved_at`.
   - Run `step protect result` with the new build; no new check is needed. Then commit
     `protection.json` and the result in the stage's `tasks.md`, finish R189, update the index
     and tick 6.4.
4. Then 5.2 (a project that is not Rust, named by the owner), 5.3 (validation, Dagger on Linux),
   and `task_88_handover.md`.

## Frozen inputs and evidence

- Binaries:
  - `D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv\bin\lesscode-core.exe` for R188;
  - `...\stage-6\bin\` (6.2's plan of `yuv.rs`);
  - `...\stage-6\bin-6.4\` (release of `33a6273`, SHA-256 `52037674…F8B6`) for R189.
  - The fix needs a new build; R189 records which binary gave which result.
- Evidence of R189: `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6\`:
  - `sf-plan.*`, `sf-begin.log`;
  - `sf-pins\` (base run, pins, disk samples);
  - `sf-check.*`, `sf-result-1.log`, `sf-protection-1.json`;
  - the plan on `master`: `plan-shared-frame-master.*`.

  0.04 GiB with the binaries.
- D: 119.7 GiB free.

## State of the repositories

- lesscode: `main` with today's commits up to this handover. `origin/main` is at `9f6aab7`. Push
  only on the owner's word. Release 2.2.0 is written up and not tagged.
- drift-rdp clone `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`, clean, on
  `protect/protect-shared-frame` (`23f1ce9`). Its branches:
  - `stage-base/shared-frame` (`a8f3442`, the joining line the owner approved);
  - `protect/protect-yuv` (finished);
  - `stage-base/yuv`;
  - `protect/display-path`.

  None is merged. `D:\Source\drift-rdp` was only read.
- `openidea-kiro`: untouched.

## Carried, open

- Keep or revert the change in `.dagger/main.go` (`38d337e`): it gives the core's files the time of
  the run, so that every Linux run builds the core. The owner has not seen it.
- drift-rdp's GPU test `renderer::yuv::tests::the_gpu_conversion_matches_the_reference` aborts
  (`0xc0000409`) when two copies run at once. It caused a first-run retry in two plans and, most
  likely, the false kill of #6. That is a matter for drift-rdp, not lesscode, but it bears on any
  plan of the renderer.
