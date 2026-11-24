# Handover 86: brownfield test stage, group 2 done (`step protect result`)

Written 2026-11-24, 14:50 Europe/Berlin. Predecessor: [task_85_handover.md](task_85_handover.md).
Spec: [brownfield-test-stage](specs/brownfield-test-stage/tasks.md). Record:
[R186](../docs/research/2026-11-24/07-step-protect-result.md).

## What was done

- **2.1** (`7ec7669`): `lesscode-core step protect result --declaration <specs-dir>/<name>/step.json
  [--repo .]`. Takes the verdict only when it is the one for the committed content, by the hashes
  `step guard` compares. Exit 2, a sentence and nothing written for: no verdict; a verdict for
  other content; a scope file the branch changed; a verdict that did not judge.
- **2.2** (`42de998`): per chosen function noticed before, noticed now, written off, waiting.
  Exit 0 when nothing waits; 1 when a fault waits (each named) or the verdict refused because
  unnoticed faults rose; 2 also for a verdict refused for another reason; 3 for a wrong kind of a
  write-off or a write-off for a fault the stage does not have.
- **2.3** (`de723dc`): the output names every write-off with reason, approver and date, repeats
  what a function reaches out to for the kind `not_testable_as_it_stands`, says that the stage
  advises, and names the next step on exit 0. `--out`. `protection.json` (`step-protection-v1`)
  on exit 0 and 1.

## Files

`native/src/step/`: `protect_result.rs` (arguments, the verdict it may use, exit code),
`protect_numbers.rs` (`write-offs.json`, the four numbers, matching a fault by line and operator),
`protect_receipt.rs` (the output, `protection.json`), `protect.rs` (`result` added), `guard.rs`
(`moved` split out of `stale`; behaviour unchanged). Budgets entries for the three new files; the
largest is `protect_numbers.rs` at 1503 of 2800 tokens. Tests: `native/tests/step_protect_result.rs`
(seven, on the toy repository), `native/tests/step_protect_numbers.rs` (four, on the library),
`native/tests/step_support/planned.rs` (the toy plan, shared with `step_protect.rs`). The file plan
of the design is amended.

## State

- Windows at `de723dc`: core suite 498 of 498, 1 skipped; lints, format, doctests, size budgets,
  `tools\bin\kiro-cli.exe code check --changed HEAD` with 0 errors and 3 warnings (`of_write_off`
  in `protect_receipt.rs` and `Repo::cli` in the test support have long statements; `toy` is one
  of seven similar routines in the tests).
- Linux through Dagger at `de723dc`: overall 0; core suite 497 of 497, 1 skipped.
- Nothing was run on a real project. That is task 5.1.
- `origin/main` is at `9f6aab7`. Local and not pushed: everything after it, including the three
  task commits of this group and the commit of this handover. Push only on the owner's word.

## Open for the owner

The command reads the verdict, `stage.json` and `write-offs.json` from the working tree where
there is a copy, else the committed one. Under a person an approval that is written and not
committed already counts. The other way is committed files only. Built: working tree first. The
skill of group 3 has to say whichever way is decided. Recorded in the design and in R186.

## Rules the design did not have (all in R186 and the design's amendment)

- Two faults of a function with the same line and operator wait or are written off together.
- A fault that is unnoticed now and was not among the stage's faults waits and cannot be written
  off.
- A write-off for a fault the stage does not have is exit 3.
- `protection.json` is also written on exit 1, and carries `result` and `unnoticed_rose`.

## Storage

D: has 119.9 GiB free. No isolated target directory was made; builds used the shared
`D:\BuildArtifacts\cargo-target`. Kept: `D:\BuildArtifacts\lesscode\brownfield-stage` (six budget
reports, `linux-g1` and `linux-g2`; 1.27 MiB, up from 0.4 MiB). The tests' repositories under the
temp directory remove themselves.

## Next

Group 3, tasks 3.1 and 3.2 (the skill `protect-before-reduce` and the two templates), in the
foreground with explicit task numbers, one commit per task. Read first: the design's section on
the skill, the record of the prototype (R184), and this handover's open point. The skill says how
the stage runs in a pipeline, without `--open`. Next research record R187; next handover
`task_87_handover.md`.

Carried, unchanged from handover 85: the end screen of the run page; the clone's branch
`protect/display-path` is merged nowhere; release 2.2.0 is not tagged; the state-metrics extension
in openidea-kiro and the stopping-rule test of R163; the hook test that fails under load. Drift
hardening (Slice B): closed by the owner, only the drift of one model over time is unmeasured.
