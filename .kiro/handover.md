# Handover: the stage on shared_frame.rs waits for a fix of the tool and the owner's approvals

Reset 2026-11-24, about 22:10 Europe/Berlin (from the clock). This file is the start for the next
session. The version before this one is in Git (`4aa5dee`, the reset of 18:18 about `yuv.rs`);
older ones are [handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). The group handovers are
[task_84](task_84_handover.md) to [task_87](task_87_handover.md); the next is
`task_88_handover.md`, written when the open tasks of the stage spec are finished.

Nothing is running. All working copies are removed.

**Update 2026-11-27, about 09:50: the owner approved the design; the tasks of `report-set` are
generated and wait for the owner's approval. Start here.**

- [tasks.md](specs/report-set/tasks.md): 8 groups, 21 sub-tasks and one closing task, every
  requirement mapped. The sanity review was inline, verdict PASS. Not approved; no implementation
  before that.
- Group 1 is the page (the tool's own page changed in place, which the gate did not show), then
  the set as a file, the inputs, counts only, `step set`, the PDF by command, the example and the
  documentation, and last the gate's expectations run on the tool itself.
- Still open with the owner: the look of the two PDFs (G8; remarks go into task 1.3), and whether
  to push. `main` is ahead of `origin/main` by the commits of today.
- **Next:** the owner approves the tasks, then `kiro-impl report-set 1.1` in the foreground. The
  group's handover will be `task_95_handover.md`.

**Update 2026-11-27, later that morning: the owner decided the three corrections, `design.md` is
corrected and waits for the owner's approval.**

- Decided by the owner, all as recommended (D4 to D6 in the design): project name and title of a
  report page come from `labels` in the notes file; names are searched only in the project's own
  steps and in the notes, refused when distinctive and warned when an everyday word of the page
  (requirement 3.5 carries a dated amendment); the set section composes the page's smaller parts
  and `Result` stays.
- **Next:** the owner approves the design (and looks at the two PDFs for G8), then
  `kiro-spec-tasks report-set`. Tasks are not generated before that. Nothing is pushed.
- The three points and the "Next" line in the update below are overtaken by this.

**Update 2026-11-27, about 09:40: the prototype gate of `report-set` is run and says proceed; the
design needs three corrections before the owner sees it.**

- **The gate, R200** ([entry](../docs/research/2026-11-27/01-report-set-prototype-gate.md),
  [gate file](specs/report-set/prototype-gate.md)). G1 to G7 hold on the six reworks of R199: one
  checked JSON file, one page that shows it and takes another, a PDF by Edge on Windows and by
  Chromium in a Linux container in under 2 seconds with every drawing, no process left. G8, how the
  PDF looks, is the owner's: `D:\BuildArtifacts\lesscode\report-set\out\set-windows.pdf` and
  `set-linux.pdf`. Nothing of `native/src` was run or changed.
- **Wrong in the design, not corrected in `design.md` yet** (its head says so):
  1. A report page holds no project name and no title for its step. The prototype took them from a
     `labels` part in the notes file.
  2. The rule that keeps names out cannot be a plain search: five of the Python project's 180 names
     are everyday words, one of them the name of a part of every step. The note "The first run of
     the tests took longer." was refused.
  3. The page's `Result` cannot draw a stated step (no commit, no exit code). The set section calls
     the smaller parts itself.
- **Next, in this order:** the owner's word on the three corrections (a proposal for each is in the
  session's last message and in R200), correct `design.md`, the owner's approval of the design,
  then `kiro-spec-tasks report-set`. The first task after approval: the tool's own page, changed in
  place, still passes the checks of the run page.
- **R199 amended:** its "across files" is the tool's "pairs of files with a call between them".
- **Kept outside the repository:** `D:\BuildArtifacts\lesscode\report-set\` (11 MiB: the research
  program's source, the page, the checks, `gate.ps1`, the step files, `out\` with both PDFs and
  `gate.json`). Removed: the container image, the build output, and the inputs that hold names of
  the Python project, so `gate.ps1` needs the Two Bowls page read again before it can rerun.
- **Storage:** D: 133 GiB free before, 128.7 GiB after; lesscode caches about 6 GiB.
- **Left running:** Docker Desktop. It was off; the agent started it for the Linux leg. No
  container, network or image of this gate is left.
- **Unchanged:** release 2.2.0 waits for the owner's approval of the `release` environment; the
  agent does not approve it. Drift hardening stays closed by the owner (R104); only the drift of
  one model over time is unmeasured. This was not a task group; the next group handover is still
  `task_95_handover.md`.

**Update 2026-11-26, about 19:10: a progress report, and the spec `report-set` up to a provisional
design.**

- **Progress report, R199 (`ceef3f3`).** [docs/reports/progress-2026-11-26.md](../docs/reports/progress-2026-11-26.md)
  with [its page of drawings](../docs/reports/figures/progress-2026-11-26.html), a synthesis of R158
  to R198 with no new measurement. Also a private page of the owner,
  `https://claude.ai/artifact/P9mEU65RdF22e9QbK8WoeZ`. Before-and-after maps of files and imports
  (Floorplan) were tried on the owner's suggestion and dropped by the owner: 84 files and 185
  imports to 83 and 184 show nothing of a rework whose gain is in the state. Understand Anything
  was read and not run. Both are in
  [the entry](../docs/research/2026-11-26/03-progress-report.md).
- **Spec `report-set`** ([brief](specs/report-set/brief.md),
  [requirements](specs/report-set/requirements.md), [design](specs/report-set/design.md),
  [research](specs/report-set/research.md), [prototype gate](specs/report-set/prototype-gate.md)).
  The owner wants such a report from the tool: one self-contained JSON file that holds a set of
  reworks across projects, drawn by the page inside the binary, with an export to PDF.
  - Requirements: approved by the owner, with six points (among them: a step may enter a set from
    handed-in numbers alone; cost is run time only; notes are plain text).
  - Design: generated, **provisional, not approved**. A step of a set is a run as a report page
    already holds it; inputs are report pages, step files and a notes file; the set page is a third
    mode of `native/page/run.html`; the PDF is printed by an installed Chrome, Chromium or Edge
    started headless; no new crate.
  - Decided by the owner at about 19:05: a judged rework without a report page enters through a
    step file, marked "not judged in this set"; the PDF on Linux is shown once in a Linux container
    with a Chromium, and the page's own checks stay Playwright on the host; the commands are
    `step set` and `step set-pdf`.
  - **Prototype gate: registered, not run.** Eight expectations, G1 to G8, on the six reworks of
    R199. The one most likely to fail is G5: a headless browser that prints before the page's
    script has drawn. If it fails on both systems the PDF command is cut and the owner decides.
- **Next, in this order:** run the gate under `D:\BuildArtifacts\lesscode\report-set\` (a research
  program in Rust outside `native/src`, a copy of the page, a PowerShell start script; no Python),
  record it as R200, put the design to the owner, then `kiro-spec-tasks report-set`. No production
  task before the gate says `proceed`.
- **Kept outside the repository:** `D:\BuildArtifacts\lesscode\progress-report\` (the one-off script
  that joined R199's drawing data, the page's template, two screenshots; under 1 MiB). The
  Floorplan copies are removed.
- **Unchanged from the update below:** release 2.2.0 waits for the owner's approval of the
  `release` environment; the agent does not approve it. Docker Desktop is running. Drift hardening,
  the second work package of the driver research, stays closed by the owner (R104); only the drift
  of one model over time is unmeasured.
- This was not a task group, so no `task_N_handover.md` was written; the next is still
  `task_95_handover.md`.

**Update 2026-11-26, about 16:40: the release and the two merges are done, on the owner's word.**

- **lesscode 2.2.0.** Pushed `main` (`9f6aab7..5583488`) and the annotated tag `v2.2.0` on
  `5583488`. The preparation, `f4bbd89`, followed `964209d` for 2.1.0: the version in the crate,
  the three lock files, `CITATION.cff`, `.zenodo.json` (with R1 to R198) and the README badge.
  The notes now say the release is out and list two fixes and one limit:
  - `step check` and a deleted scope file (`a20ecd5`);
  - the page's sign of life (`84a3a46`);
  - no approval has been judged yet with the owner's own key, which has no passphrase.

  The release workflow (run `45847414249`) builds, and then its publish job waits for the owner to
  approve the `release` environment on GitHub. The agent does not approve it. Zenodo archives the
  release once it is published.
- **lesscode CI had been red on GitHub since 2026-11-24; it is green on `84a3a46`, on all six
  jobs.**
  - Two run-page tests made their places under `CARGO_TARGET_TMPDIR`. On GitHub that directory is
    inside the checkout, because the target directory is `native/target` there. `0a5c49b` moves
    them to the system's temporary directory.
  - On macOS Intel, the sign-of-life test saw 4 beats where 5 were asked for. That was a defect in
    the tool: the heart added up the naps it asked for instead of reading the clock, and a nap
    lasts about twice as long there, so with the real 5 s a sign of life came about every 11 s.
    `84a3a46` fixes it.
  - `b585d57` had first widened the test's interval. That was a wrong diagnosis: the gap grew with
    the interval. `84a3a46` returns the test to its first form.
- **drift-rdp:** `protect/protect-yuv` merged onto `fork/master` (`8b254d3`, no conflicts) and
  pushed to `fork` (`2cc0c3d..8b254d3`).
  - 560 library tests pass, the GPU test for NV12 included.
  - fmt is clean. Clippy shows one warning more under `--all-targets`: `let_and_return` in the
    approved test `yuv/protect.rs:84`. CI's clippy does not lint tests, and the pinned test was
    left as approved.
  - CI on `8b254d3` passed, all jobs. The macOS job on `2cc0c3d` had been cancelled after waiting
    15 minutes for a runner, with no step run.
- **scitools-hook:** `protect/protect-cycles` merged onto `main` (`a72f1cc`) and pushed to
  `origin` (`norandom/scitools-hook`, `73a736e..a72f1cc`). In the stage's test image: the stage's
  command passed 5332 tests with 2 skipped. The CI gate's command (pytest with branch coverage
  of at least 85%) passed 5332 with 257 skipped, at 97.83% coverage. No workflow ran on the push.
- **Left running:** Docker Desktop. The agent started it for the scitools-hook tests; it was off
  before. No container of lesscode's is left.
- **Next:** the facdrone visualization (decision 4 below). Decision 3 stands: `signed-approvals`
  is closed at 5.2.

**Update 2026-11-26, about 15:45: drift-rdp merged and pushed; the owner's decisions for the
next session.**

- **Done.** `lesscode/renderer-rework` was merged onto `fork/master` (`14dc5a1`, 7 commits newer
  than the branch's base). The merge is `424973b`, with one conflict in `lib.rs`: `RenderShared`
  is built from the display path state, under the `cfg` that now includes macOS. It is followed by
  `2cc0c3d`, which only applies rustfmt to the protection tests of `shared_frame.rs`, so CI's
  `fmt` passes. On the merge: 559 library tests pass, 7 skipped; Clippy shows the 9 warnings that
  were there before; `fmt` is clean. `D:\Source\drift-rdp` master was fast-forwarded and pushed to
  `fork` (`norandom/1-1`, `14dc5a1..2cc0c3d`). Nothing was pushed to `origin` (CREVIOS, the
  upstream project, which local master is 230 commits ahead of). CI was queued on `2cc0c3d` at
  14:33 UTC; its result is not in this file. The merge brought the protection branches
  `protect/display-path` and `protect/protect-shared-frame` with it. The macOS paths of the fork
  were read by hand, not built: they call `attach` with the same `Arc<Wake>`.
- **The owner's decisions, 2026-11-26 about 15:40, in this order:**
  1. lesscode: push `main` (80 commits ahead of `origin/main`) and tag `v2.2.0`. The tag starts
     the release workflow and the Zenodo deposit. Package signing (zipsign) still waits for the
     owner's key, so check what the workflow does without it before tagging.
  2. Merge and push the two protection branches left: drift-rdp `protect/protect-yuv` (onto
     `fork/master`, push to `fork` only) and scitools-hook `protect/protect-cycles` (find its
     owner's remote first). Use today's procedure: merge, then tests, fmt and Clippy, then push.
  3. `signed-approvals` is closed at 5.2 by the owner. The earlier approvals stay unsigned. The
     next stage that needs an approval uses a signed one from the start.
  4. Next topic: a visualization of facdrone, the Python calibration subject. It is read only, and
     its counts go to lesscode, nothing to facdrone.

**Update 2026-11-26, about 15:10: start at [task_94_handover.md](task_94_handover.md).** The
owner closed signed approvals at 5.2 (R197); 5.3 was stopped. drift-rdp's display path is held in one
value, and `step check` accepted it with growth 0 (R198). The step is recorded on the local branch
`lesscode/renderer-rework`, not merged or pushed. lesscode `a20ecd5` fixed `step check` for a
step that deletes a scope file. Nothing is running. `native\target` was removed on the owner's word
(7.5 GiB). Merge, tag and push wait for the owner.

**Update 2026-11-25, about 19:30, end of the day: task 5.1 of `signed-approvals` is done; 5.2
waits for the owner.** Start tomorrow at [task_93_handover.md](task_93_handover.md) for group 5,
then [R196](../docs/research/2026-11-25/07-signed-approvals-validation.md) for 5.1. No group
handover yet: group 5 ends with 5.3, and `task_94_handover.md` is written then.

- **5.1, validation of the tool.** At `1dc83bf`: Windows 551 of 551 (1 skipped), doctests,
  format, Clippy, budgets; Linux through Dagger all 23 steps 0, core 550 of 550, model 85 of 85;
  `kiro-cli code check --changed 09c3076` passes with its 2 pre-existing errors (`step_guard.rs`).
  At the group-4 commit `8dfaf9d` the suites passed but `kiro-cli` refused two routines the spec
  had added (`against_forge` with 6 parameters, a result test of 69 lines); `1dc83bf` cuts them
  down without a change of behaviour. Record: `f59ef16`.
- **Next, 5.2, the owner's act:** follow [docs/approvals.md](../docs/approvals.md), make the key
  with a passphrase, grant `gh` the signing-key scope, add the public key as a signing key, take
  the scope back. The agent runs none of the `gh` steps. Note whether Git asks for the passphrase
  when signing (gate row G9, never seen so far). Then the agent checks the forge's list,
  `gh auth status` and one signed commit in a scratch repository with lesscode's check. After it,
  5.3: the re-approval on drift-rdp `protect/protect-yuv` and `protect/protect-shared-frame` and
  scitools-hook `protect/protect-cycles`.
- **State.** lesscode `main` at this update's commit; `origin/main` at `9f6aab7`, not pushed
  (push only on the owner's word). Clones untouched: drift-rdp
  (`D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`) on `protect/protect-shared-frame`,
  scitools-hook (`D:\BuildArtifacts\scitools-hook\lesscode-stage`) on `protect/protect-cycles`.
  Nothing running of lesscode's; the shared Dagger engine container was left running.
- **Storage.** D: 111.9 GiB free (139.7 at the start of 5.1). Most of the drop is Docker
  Desktop's disk `D:\Docker\disk\docker_data.vhdx` (89.3 GiB), where the shared Dagger engine's
  cache volume holds 60.6 GB; shared, not pruned. lesscode's own caches: `lesscode-cargo-test`
  15.5 GiB, `native\target` 7.5 GiB (inactive since 2026-11-07, removal waits for the owner's
  word, handover 89 item 4). Both limits hold. New evidence is small: `signed-approvals\` logs and
  `native-evidence\linux-8dfaf9d`, `linux-1dc83bf`.
- **Still open from handover 89:** merging the protection tests into drift-rdp and scitools-hook,
  the tag of 2.2.0, the push. Drift hardening (Slice B) stays closed by the owner (R104); only
  the drift of one model over time is unmeasured.

**Update 2026-11-25, about 17:45: task group 4 of `signed-approvals` is done.** Start at
[task_93_handover.md](task_93_handover.md). Next is group 5 (validation, the owner's key, the
re-approval).

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
