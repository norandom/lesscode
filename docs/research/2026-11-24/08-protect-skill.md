# R187: the skill `protect-before-reduce` and its two templates are written; the result reads committed files only; nothing was walked with them yet

- Report date/time and timezone: 2026-11-24, 15:40 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, 14:55 to 15:38 (commits `d8d1d1d` 14:58, `a87df10` 15:01, `50cfa39` 15:16, `8d4b68b` 15:17, `22630db` 15:18, `38d337e` 15:38)
- Status: completed for group 3 of the tasks; the skill is text and no agent has followed it
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), tasks 3.1 and 3.2; requirements 2.2 to 2.7, 3.1 to 3.7, 3.9, 4.1, 7.1 to 7.3
- Hypothesis ID and predecessor: [R186](07-step-protect-result.md) (`step protect result`); read against [R184](05-brownfield-stage-prototype.md) (the stage by hand)
- Type: implementation record, with two decisions of the owner built into the commands and one defect found in the Linux verification

## Decision

Group 4 (the path in the guide) and task 5.1 (the stage with its commands on another file of the
renderer) can begin. The walk an agent follows exists as a skill with two templates, and the two
commands behave as the owner decided today. What this record does not show: that an agent
following the skill arrives where the prototype arrived by hand. That is task 5.1, and it can fail.

## What the owner decided, and what was built from it

| Owner's words | Built | Commit |
| --- | --- | --- |
| "committed only" (the open point of R186) | `step protect result` reads the verdict, `stage.json` and `write-offs.json` as they are committed on the branch and no other copy. Without a committed verdict: exit 2. A write-off whose approval is written and not committed waits. | `d8d1d1d` |
| "print the disk figure" | `step protect begin` measures the plan's first copy (the directory `w1` beside the run receipt) and prints its size with its path. Where the copies are gone it says that the size is not known. Nothing of it goes into `stage.json`. | `50cfa39` |

The second closes a gap between the design and the commands: point 5 of "What the prototype
changed" had the sentence since 13:15, and groups 1 and 2 did not build it. I found it while
writing the skill, which would otherwise have promised an output the command did not have.

## What was written

| Commit | Task | What |
| --- | --- | --- |
| `a87df10` | 3.1 | `.agents/skills/protect-before-reduce/SKILL.md`: who does what; the eleven phases with their gates; how the stage runs in a pipeline; what no command checks |
| `8d4b68b` | 3.2 | `templates/requirements.md` and `templates/tasks.md` |
| `22630db` | | two tests split, after `kiro-cli code check` refused two routines of 61 lines against a limit of 60 |

The skill uses the form of Kiro's spec files (their names, numbered requirements, tasks with a
boundary) and none of the text of Kiro's skills.

## The skill read against the prototype

Done by me, inline. Every step R184 took by hand has its line:

| By hand in R184 | In the skill |
| --- | --- |
| the declaration `step.json`, with `focus` and `max_token_growth` 0 | phase 3, `step protect begin` |
| the joining line and the empty test file, approved and committed before the plan | "Before you start" |
| the stage spec | phase 4 and the two templates |
| the tests tried once with the project's own command before a pin | phase 6, point 1 |
| the first pin ran the base and took 17 min 50 s, unannounced | phase 5: a step of its own, announced |
| a fault number that was not an unnoticed fault (exit 3) | phase 5: numbers come from the list `step pin` prints and have gaps |
| eight copies, 34.5 GiB | "Before you start"; copies removed after phase 3 and after phase 8 |
| the count per function and the three write-offs | phase 10, `step protect result` |
| `approved_by` filled in by the agent from the owner's answer | forbidden in phase 7; the owner writes and commits it in phase 9 |
| the owner asked for plain choices after four questions at once | phase 2 |
| one test that asks the real GPU | "What no command checks": say what a test depends on |

The stage spec of the prototype (`protect-display-path` in the clone, branch
`protect/display-path`) read against the templates: its heading lines, the paragraph on planted
faults, what was left alone, the five requirements with their faults, the two forms of a proposal
inside a requirement, the four test tasks, the task of proposals and the result section each have
their place. The templates ask for three things the prototype's spec did not have: the plan's file
at the top, the number of a fault beside its id, and the pin receipts named per task.

## Three points where the skill says more than the design

The owner has read the skill with these named.

1. **The agent never fills the two approval fields**, also not from the owner's answer in the
   conversation. With "committed only" the owner writes them and commits the file.
2. **Pin receipts go into the stage spec** (`pins/pin-<n>.json`) and are committed with it. In the
   prototype they lay outside the repository.
3. **The base is run by `step pin --fault 1` before any test exists.** The answer is then exit 1
   ("the tests with the pin pass on the fault") or exit 3 with the list of unnoticed faults; either
   prints the numbers. The design's gate for phase 5 was "exit 0 or 1".

## Evidence

- "Committed only": the tests of the result commit the verdict and each write-off. One assertion
  is new: an approved write-off that is written and not committed leaves the fault waiting (exit
  1), and the same file committed gives exit 0. A stage whose verdict is not committed is exit 2
  with "there is no committed verdict".
- The disk figure: one test reads the sentence with the copy's path; a second removes the copy and
  reads that the size is not known.
- Windows at `22630db`: core suite 500 of 500, 1 skipped; format check, lints with warnings denied,
  doctests; the size budgets are tested inside the suite and pass; `kiro-cli code check --changed
  296a7e6` with no error and nine warnings.
- Linux through Dagger, the code of `22630db` with the module of `38d337e` (`dagger call evidence`,
  352 s): overall 0, every step 0; core suite 499 of 499, 1 skipped, with the two new tests among
  them. Evidence: `D:\BuildArtifacts\lesscode\brownfield-stage\linux-g3c`. Two runs before it
  (`linux-g3`, `linux-g3b`) also say overall 0 and are not evidence: see finding 1.

## Findings

1. **The Linux verification could test the binaries of an earlier run, and say overall 0.** Two
   runs at `22630db` finished the build of the tests in 0.10 s and ran 497 tests; the two tests
   added today were not in the log. The Dagger module keeps the target directory between runs.
   The files of the source directory arrive in the container without the time of their change, and
   cargo decides by that time whether to build. Changed (`38d337e`): the module gives the core's
   own files the time of the run, so the core is built in every run and its dependencies stay
   built. After it: `Compiling lesscode-core`, 499 of 499. **What this means for earlier records**:
   a Linux result through Dagger is only as good as its log. The run of group 2 (`linux-g2`) shows
   `Compiling lesscode-core` and its seven tests of the result; I did not check the logs of earlier
   groups and specs. When a run built and when it did not, I do not know: group 2 changed files and
   was built, today's changes were not.
2. **`protect_begin.rs` was at its size budget.** The first way of carrying the copy's path through
   it failed the budget test. Where the receipt of a plan's run lies (`run_of`) moved to
   `protect_says.rs`, which also holds the measuring; `protect_begin.rs` got smaller.
3. **The first pin of a stage is two runs**, by reading `pin.rs`: every fault of the chosen
   functions on the base, then the unnoticed ones again in a copy with the branch's test files. The
   prototype's 17 min 50 s was both; how it divides was not measured. `step protect begin`
   announces the time of the first of the two. The skill says "the long run" and does not say two;
   task 5.1 compares the announced time with the clock.
4. **One test of the suite failed in a full run and passed alone**
   (`the_installed_hook_stops_a_push_without_a_matching_verdict`, in a run beside a second build).
   It is the hook test that fails under load, carried since handover 85.

## Amendment, 2026-11-24, 15:46: the earlier Linux logs were checked, and finding 1 has another cause

The owner asked for the check. All 43 kept evidence directories under `D:\BuildArtifacts\lesscode`
that hold a `test.log` and an `overall.exit-code` were read for `Compiling lesscode-core`, the
seconds of the test build and the number of tests.

- **Every earlier Linux run built the core**: 27 runs from 10 to 24 November (Dagger on this
  machine and the pipeline's Linux runners), each with `Compiling lesscode-core` and a test build
  of 13 to 56 seconds. The number of tests rises with the commits, 351 to 497. No earlier Linux
  result is stale.
- The only runs that built nothing are the two of today, `linux-g3` and `linux-g3b`, and two local
  Windows runs of 16 November (`task-8.1\local-evidence`, `task-8.5\local-evidence-184059`), which
  ran in the developer's own target directory right after a build there and are as they should be.

So the cause written in finding 1 is wrong: files do not arrive without the time of their change,
or 27 runs would not have built. What fits the times is a run of mine. I started a Linux run
from the tree of `8d4b68b` (committed 15:17:17), then edited two test files while it ran
(committed 15:18:38 as `22630db`), and stopped it later. The engine went on and built the older tests after the
newer files were saved. The kept target directory then held binaries newer than the files they
lacked, and the two next runs took them as up to date. I did not replay this; it is the reading
that fits the clock and the 497 tests those runs had.

The change `38d337e` stays useful against exactly that, a run overlapping an edit or a stopped
run, and costs one build of the core per run (about 20 seconds here). Whether to keep it is the
owner's. The comment in `.dagger/main.go` now names this cause.

## Limits

- Why cargo built in the run of group 2 and not today was not found out; the change does not
  depend on the answer.
- No agent has followed the skill. What it says is checked by a person reading it and by task 5.1.
- The reading against R184 and against the prototype's stage spec was mine, and I wrote both sides.
- The size of a copy is what the directory holds when `step protect begin` runs. Where a project
  builds outside the copy (a shared build directory), the figure leaves that out.
- The templates are in English; the skill tells the agent to write in the language of the
  project's specs. No other language was tried.
- A project without Kiro's directories was not tried (as in R184).

## Next gate

Group 4: the section of the guide, read by the owner. Then task 5.1 on a file of the renderer the
owner names, in an isolated clone, with the whole skill.

## COBESY check

What exists and what is not shown come first. The two decisions are given in the owner's words
beside what was built. What would reject the claims about the commands is a test each. What would
reject the skill is task 5.1, and that is said.
