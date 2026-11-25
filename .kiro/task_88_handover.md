# Handover 88: brownfield test stage, groups 5 and 6 done, validation GO

Written 2026-11-25, about 12:35 Europe/Berlin. Predecessor: [task_87_handover.md](task_87_handover.md).
Spec: [brownfield-test-stage](specs/brownfield-test-stage/tasks.md), every task ticked.
Validation: [validation.md](specs/brownfield-test-stage/validation.md), **GO**. Records:
[R188](../docs/research/2026-11-24/09-protect-stage-yuv.md),
[R189](../docs/research/2026-11-24/10-protect-stage-shared-frame.md),
[R190](../docs/research/2026-11-25/01-protect-stage-python.md).

## What was done

- **5.1, the stage on `yuv.rs` of drift-rdp (R188).** One test was written. Eleven of twelve gaps
  were wgpu debug labels, which the owner wrote off, and `step protect result` exited 0.
- **The owner's amendment of 2026-11-24** (requirements 1.1, 1.2, 5.10 and 7.1, design "18:40",
  group 6). The stage protects the whole scope by default. A missed fault holds it back only in a
  chosen function, and functions not named are reported.
  - 6.1 (`1ff09c4`): `--function` is optional, functions not chosen are reported, and paths are
    printed with `/`.
  - 6.2 (`96625d6`): the pack's `skip_strings_in`. The Rust pack empties no string in a `label`
    field. This crosses into `agnostic-annotation-driver`'s pack.
  - 6.3 (`33a6273`): the skill, the guide and the 2.2.0 notes. `begin` tells you to allow 1.4
    times its time figure, and twice the plan's copies on disk.
- **6.4, R189, the run where tests get written**, on `shared_frame.rs` of drift-rdp. Three tests
  were written and four faults written off. On the way it found a flaky test of drift-rdp that
  made the plan count an equivalent fault as noticed.
- **6.5 (`7f0324a`), the owner's fix**, amendment of 2026-11-25. `step check` names the base
  run's unnoticed faults (`unnoticed_before`), and `step protect result` takes such a fault into
  the stage. With it, the skill gives `--page` a place outside the repository and removes the
  pins' copies before the check. The verdict field crosses into `step check`.
- **5.2, R190, the stage in Python**, on scitools-hook's `cycles.py`, which the owner chose.
  Nothing of the stage differed. The project's suite needed a Linux container (POSIX
  behaviour), with the image and script in `...\stage-5.2\image\`. One test was written and one
  fault written off, and the result exited 0.
- **5.3, validation, GO**:
  - Windows: 508 of 508; lints, format, doctests and budgets pass.
  - Linux: Dagger on Docker Desktop, 23 steps all 0, core 507 of 507 and model 85 of 85.
  - `kiro-cli code check` passes; its 4 errors are pre-existing.

## Decisions of the owner in this group, in their words where given

- "whole scope by default"; on what blocks, the three cases. A visible fault in a chosen function
  blocks, a fault no caller sees is not planted, and a visible fault outside is reported.
- the Rust pack skips `label:` fields; the 4 labels passed through a helper are accepted as
  write-offs.
- 6.4 on `shared_frame.rs`, its joining line approved; 5.2 on scitools-hook's `cycles.py`.
- "Verdict names base misses", over trusting the accepted check.
- the stage specs' requirements and tasks of `protect-yuv` and `protect-shared-frame` count as
  approved by the one review. `protect-cycles` was not asked and stays unapproved.
- the owner switched from Podman to Docker Desktop on 2026-11-25; the old images are gone.

## Open, for the owner to decide

1. **Approvals an agent cannot make.** The owner asked why approvals come by a PowerShell script.
   Offered: editing by hand, a command the owner runs, a review in the forge, signed commits.
   Nothing is decided.
2. **The Python pack swaps keyword arguments**, which is always equivalent. Proposed: skip a swap
   whose first argument is a keyword argument.
3. **`begin`'s time figure** leaves out what one run costs besides its tests, such as a
   container's start. It ran 1.7 to 2.7 times low in R190. Proposed: count the plan's base run.
4. **Rust labels passed through a helper** are still planted. Accepted for now.
5. **`.dagger/main.go` (`38d337e`)**, which gives the core's files the time of the run. The owner
   has not seen it; it was in effect in today's Linux run.
6. `protect-cycles`' spec phases: mark approved or not.

## Repositories and storage

- lesscode `main`: this handover's commit; `origin/main` at `9f6aab7`. Push only on the owner's
  word. Release 2.2.0 is written up and not tagged.
- drift-rdp clone `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`, clean. Branches:
  - `protect/protect-shared-frame` (`f3469cc`), on `stage-base/shared-frame`;
  - `protect/protect-yuv`, on `stage-base/yuv`;
  - `protect/display-path`.

  None is merged. `D:\Source\drift-rdp` was only read.
- scitools-hook clone `D:\BuildArtifacts\scitools-hook\lesscode-stage`, clean, on
  `protect/protect-cycles` (`d4c9a6a`). Not merged. `D:\Source\scitools-hook` was only read.
- Retained evidence: `D:\BuildArtifacts\lesscode\brownfield-test-stage\`, 0.13 GiB in all
  (`yuv`, `stage-6`, `stage-5.2`, `validation-5.3`, frozen binaries), and
  `D:\BuildArtifacts\lesscode\native-evidence\linux-7f9c889`.
- Docker image `lesscode-stage/scitools-hook-tests:2026-11-25` (664 MB), kept for a repeat of
  R190; `docker image rm` removes it.
- No working copy and no container is left. D: 187 GiB free; the 40 GiB cache limit and the 50 GiB
  floor of AGENTS.md hold.

## Next

The spec is finished. What comes next is the owner's choice: one of the open points above, merging
the protection tests in the two projects, the tag of 2.2.0, or another spec.
