# R186: `step protect result` is built; on the toy repository it gives the four numbers, the exit code and the receipt

- Report date/time and timezone: 2026-11-24, 14:45 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, about 14:05 to 14:41 (the commits of tasks 2.1, 2.2 and 2.3 at 14:11, 14:18 and 14:41); the run on Linux after that, see below
- Status: completed for group 2 of the tasks; nothing was run on a real project
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), tasks 2.1 to 2.3; requirements 3.6, 4.1 to 4.7, 5.1, 5.3 to 5.8, 6.4, 6.5, 7.4, 7.5
- Hypothesis ID and predecessor: [R185](06-step-protect-begin.md) (`step protect begin`)
- Type: implementation record, with one open point for the owner and three rules the design did not have

## Decision

Group 3 (the skill `protect-before-reduce`) can begin. The second command of the stage exists and
is shown on the toy repository only. It says whether the chosen functions are protected enough,
from the verdict of `step check`, `stage.json` and `write-offs.json`, and answers by exit code. It
runs nothing, opens nothing and asks nothing. Whether it holds on the renderer of drift-rdp is task
5.1 and is not shown here.

One point is open for the owner (finding 1). It does not block group 3; the skill has to say
whichever way it is decided.

## What was built

| Commit | Task | What |
| --- | --- | --- |
| `7ec7669` | 2.1 | the command takes the verdict only when it is the one for the committed content, by the hashes `step guard` compares; else exit 2 and nothing written; also exit 2 when the branch changed a scope file |
| `42de998` | 2.2 | per chosen function: noticed before, noticed now, written off, waiting; exit 0 when nothing waits, 1 otherwise with every waiting fault named |
| `de723dc` | 2.3 | every write-off with reason, approver and date; what a function reaches out to, for a write-off that cannot be tested as the code stands; that the stage advises; the next step on exit 0; `protection.json` |

```sh
lesscode-core step protect result --declaration <specs-dir>/<name>/step.json [--repo .] [--out <file>]
```

| Exit | When |
| --- | --- |
| 0 | nothing waits in any chosen function |
| 1 | a fault waits, or the verdict refused because unnoticed faults rose |
| 2 | no verdict; a verdict for other content than the branch has; a scope file the branch changed; a verdict that did not judge, or refused for a reason other than faults; a verdict without numbers for a chosen function |
| 3 | bad input: the declaration is not committed on its branch; no `stage.json`; a write-off kind that is not one of the two; a write-off for a fault the stage does not have |

`protection.json` (`step-protection-v1`) is written on exit 0 and on exit 1, beside the declaration
or where `--out` names. Nothing is written on exit 2 or 3.

## Evidence

- Tests through the binary, each on a real plan of the toy repository:
  `native/tests/step_protect_result.rs`, seven tests. They show the three cases of task 2.1 (no
  verdict, a verdict for other content, a branch that changed a code file), each with its sentence
  and with the repository unchanged; the six cases of task 2.2; and for task 2.3 a stage with one
  fault of each standing, its output line by line and its receipt against a written expectation.
- Tests on the library: `native/tests/step_protect_numbers.rs`, four tests, for what the toy cannot
  produce (finding 2).
- Two cases of the binary tests use a verdict edited by hand, because the toy cannot produce them:
  a verdict that refused because unnoticed faults rose (exit 1), and one that refused for another
  reason (exit 2). The edit keeps the hashes, so the verdict still counts as the one for the
  content.
- Windows at `de723dc`: core suite 498 of 498, 1 skipped; format check, lints with warnings denied,
  doctests, size budgets (`D:\BuildArtifacts\lesscode\brownfield-stage\budgets-2.3.json`),
  `kiro-cli code check --changed HEAD` with no error and three warnings.
- Linux through Dagger at `de723dc` (`dagger call evidence`, 341 s): overall 0, every step 0; core
  suite 497 of 497, 1 skipped, with the seven tests of `step_protect_result` and the four of
  `step_protect_numbers` among them. Evidence:
  `D:\BuildArtifacts\lesscode\brownfield-stage\linux-g2`.

## Findings

1. **Open for the owner: which copy of a file counts.** The verdict, `stage.json` and
   `write-offs.json` are the files that are committed with the stage spec. The command reads the
   copy in the working tree where there is one, else the committed one, as `step record` does for
   a verdict. In a pipeline the two are the same. Under a person an approval that is written and
   not committed already counts, and the receipt is written from it. The other way is committed
   files only: stricter, and the verdict and every change of a write-off have to be committed
   before a result can be asked for. Built: working tree first.
2. **Three rules the design did not have**, each from the owner's decision that a fault is matched
   by file, line and operator:
   - Two faults of a function with the same line and operator wait together or are written off
     together. A write-off for one of them does not count: the verdict does not say which of the
     two is unnoticed. The output names both and says that they cannot be told apart.
   - A fault the verdict calls unnoticed that the stage did not begin with waits and cannot be
     written off. It is a fault a test noticed when the plan was made.
   - A write-off for a fault the stage does not have is bad input (exit 3), so that a mistyped id
     is seen and not passed over.
3. **`protect_result.rs` alone would have gone over the size budget.** The numbers are in
   `protect_numbers.rs`, the output and the receipt in `protect_receipt.rs`. The comparison of the
   hashes is split out of `stale` in `step/guard.rs` as `moved` and reused; the guard's messages
   and results are as before, and its tests pass unchanged. The file plan of the design is amended.

## Deviations from the tasks, each small

- `step/guard.rs` is changed, outside the boundary of task 2.1 (`protect_result.rs`). The task
  graph review had noted that 2.1 reuses how the guard matches a verdict.
- Two source files beyond the one the tasks name, and the tests of the result in a file of their
  own, with the toy plan both test files use in `native/tests/step_support/planned.rs`.
- `protection.json` carries `result` and `unnoticed_rose` beside what the design lists.
- The receipt is written on exit 1 too. The design says it is what a later reader finds; a stage
  that is not protected enough is also a result.

## Limits

- The tool cannot know that a person wrote an approval. It prints every write-off with its
  approver and date.
- The date of an approval is taken as text: it is not checked to be a date.
- A fault whose label in the verdict cannot be read as `#N line L <operator>` waits, and is named
  by the label.
- Where `--out` is given, the receipt is not also written beside the declaration.
- Shown on Python in the toy repository only. The path of a Rust project (tests in a file of their
  own) is task 5.1.

## Next gate

Group 3, the skill. It has to say how the stage runs in a pipeline, without `--open`, and how
finding 1 is decided.

**Amendment, 2026-11-24, about 15:00.** Finding 1 is decided by the owner: committed files only.
The command no longer reads the working tree (`d8d1d1d`). What is written above about "working
tree first" describes the command as it was at `de723dc`. See [R187](08-protect-skill.md).

## COBESY check

The outcome and its limit come first. A reader can replay with
`cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml -E 'binary(step_protect_result) | binary(step_protect_numbers)'`.
What would reject the claim: an exit 2 that writes a file, a fault that waits and is not named, or
an unapproved write-off that counts; each is a test. Nothing is claimed about a real project.
