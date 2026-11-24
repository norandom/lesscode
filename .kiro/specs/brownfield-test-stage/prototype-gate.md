# Prototype gate: brownfield-test-stage

Written 2026-11-24, about 11:35 Europe/Berlin, before any run. **Decision: `proceed`**, since 13:10, with one limit
passed and accepted by the owner (see Result).
Rule: [prototype-first](../../steering/prototype-first.md). Design: [design.md](design.md),
provisional until this gate says `proceed`.

## What is being tested

The design says the stage is a step that adds tests and changes no code, and that the commands
which exist carry it. The gate tries that on real code, by hand, before the two new commands and
the skill are built. It does not need them: a declaration written by hand stands for
`step protect begin`, and a count by hand from the verdict stands for `step protect result`.

This spec compresses nothing. In place of a reduction target the gate has a protection target: how
many unnoticed faults of the chosen functions end noticed or written off.

## Frozen inputs

- Subject: drift-rdp at `09b4ce8`, in an isolated clone under
  `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`. The checkout at `D:\Source\drift-rdp` is
  only read. The stage spec is written into the clone's `.kiro/specs`, on a branch of the clone.
- Scope of the plan: `src-tauri/src/renderer/display_path.rs` (20 planted faults in the run of
  2026-11-24, 803 seconds of fault runs). The owner names the chosen functions while watching the
  plan. Proposed: the unprotected functions of that file that are built on Windows (`decide`,
  `current`, `runtime_fallback`, `fall_back`, `layer_backends`).
- Tests: the crate's library tests, as in
  [the run configuration of R181](../../../docs/experiments/run-dashboard/real-run/config.json).
- Tool: `lesscode-core` built from the commit of `main` at the start of the run, recorded with its
  hash. Commands: `step plan --open`, `step pin`, `step check --page`.
- Who writes: the agent of the session, following the phases of the design's skill table from this
  document, since the skill does not exist yet.

## Expectations, with what would reject each

| # | Expectation | Rejected when |
| --- | --- | --- |
| G1 | With a declaration written by hand (`focus` on the chosen functions, `max_token_growth` 0), `step pin` and `step check` end with a judgement on this crate. | either ends with 2 or 3 for a reason that is the declaration's or the tool's |
| G2 | A test of a private function can be admitted by `step pin` in Rust, by one of two placements: (a) a test file of its own, joined by one line that is part of the base; (b) a test under the crate's test path. | neither placement gets a test admitted |
| G3 | Of the unnoticed faults of the chosen functions that are built on Windows, at least half end noticed. Every other one has a proposal to write it off with one of the two kinds. None is left with neither. | fewer than half noticed, or a fault left with neither |
| G4 | The branch changes test files and the stage spec and nothing else; under (a), the one joining line lies before the base. | `git diff --name-only <base>..<branch>` names a file of the code |
| G5 | Two adverse controls are rejected by `step pin`: a test that asserts nothing ("the tests with the pin pass on the fault"), and a test that contradicts the code ("fail on the unchanged code"). | either is admitted |
| G6 | The whole prototype takes at most three hours from the start of the plan, and no single `step pin` more than ten minutes. | either limit is passed; the run is stopped and recorded |
| G7 | The owner reads and approves the stage spec (requirements and tasks) before a test is written, and each write-off before it counts. | a test is written, or a write-off counted, before the approval |
| G8 | The number of protection tests is not larger than the number of faults they notice. | more tests than faults noticed |

`proceed` needs G1 to G5 and G7. G6 and G8 are recorded and argued; passing a limit of G6 stops the
run and makes the decision `revise`.

## What the owner is asked while watching

The first plan runs with `--open` and the owner watches. This is also the look at the changed run
page that the validation of run-dashboard names as missing. Asked, and written down in the owner's
words:

1. After ten seconds: was your code changed, and what is the run doing?
2. Did the browser open by itself?
3. What is unclear on the page (the lane, the files that stay shut, the notices)?
4. Which functions do you choose?

The answers to 1 to 3 belong to run-dashboard and do not decide this gate.

## Evidence to keep

The plan and its run receipt; the page; the declaration; every pin receipt, the admitted and the
rejected; the verdict; `write-offs.json` with the proposals and the approvals; the diff of the
branch; the times; the count by hand of noticed, written off and waiting per function. In the
research log as R184, with what was done by hand named apart from what a command did.

## Manual interventions, named beforehand

- The declaration is written by hand.
- "Protected enough" is counted by hand from the verdict and the write-offs.
- The stage spec is written without a template.
- Under placement (a), the joining line is written by hand and committed before the plan.

None of these is a capability the prototype shows. They are what the production tasks build.

## Stop conditions

- G1 rejected: `stop`. The design's base, that the existing commands carry the stage, is wrong.
- G2 rejected: `revise`. The stage needs the tool to tell test code inside a source file from the
  code, which is outside this spec's boundary, or the stage excludes Rust until that exists.
- G3 rejected: `revise`. Record which faults resisted and why; do not lower the half.
- Storage: the gate of AGENTS.md holds (50 GiB free on D:, no new target directory beyond the
  clone's working copies, which are removed at the end).
- The copies take turns by free memory since R183; no build is started beside the run.

## Amended 2026-11-24, 12:10, before any test was written

The owner, asked for choices after the plan ("I don't understand the problem. all we want is better
tests to ensure our lesscode transformations are not causing problems. we brownfield onbaord the
codebase with a tdd workflow"), chose:

- all five proposed functions;
- for faults a plain test cannot reach: try, then note the rest for one decision;
- **one review at the end**, of the tests and of the faults no test reaches, in place of an
  approval before the tests and one per write-off.

That changes G7. It now reads: nothing is merged and no write-off counts before the owner's one
review at the end. The requirements 2.5, 4.3 and 7.3 name three stops; they have to be amended to
the one the owner chose before the design is approved.

Two more things were settled before the plan. Placement (b) of G2 was ruled out by reading: the
module `renderer` is private in `lib.rs` and the test command runs library tests only, so a test
under `src-tauri/tests` cannot reach these functions and would not be run. The owner approved the
joining line of placement (a); it is commit `72507e1` of the clone and the base of the stage.

## Result

Run 2026-11-24, 11:47 to 13:02. Record: [R184](../../../docs/research/2026-11-24/05-brownfield-stage-prototype.md).

| # | Outcome |
| --- | --- |
| G1 | **holds.** Seven calls of `step pin` and one of `step check` ended with a judgement. |
| G2 | **holds for placement (a).** Tests in `display_path/protect.rs`, joined by one line in the base, were admitted. Placement (b) was ruled out by reading. |
| G3 | **holds.** 6 of 9 unnoticed faults built on Windows are noticed; the other 3 have proposals, and the owner approved them. |
| G4 | **holds.** The branch changes the stage spec and `protect.rs`; the scope file has 3,095 tokens before and after. |
| G5 | **holds.** "the tests with the pin pass on the fault" for the test that asserts nothing; "the tests with the pin fail on the unchanged code" for the one that contradicts the code. |
| G6 | **one limit passed.** The whole took 1 h 15 min of 3 h. The first `step pin` took 17 min 50 s against a limit of ten minutes: it runs the base's faults once. The others took 90 to 143 seconds. I did not stop the run at that point, as the gate asked. |
| G7 (as amended) | **holds.** One review at the end; nothing is merged. |
| G8 | **holds.** Four tests, six faults. |

By the gate as written the decision was `revise` on G6. The owner was given three ways to treat it
and chose: "Accept, and say it up front". The prototype counts as passed; the design is changed so
that the stage runs the base once, on its own, and says before it starts how long that takes. No
target was lowered without the owner: the limit of ten minutes stays as it was written, and it was
passed once.

Not shown by this gate: another language; a project without a Kiro setup; the two commands and the
skill, which do not exist; more than one file.
