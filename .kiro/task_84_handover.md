# Task handover 84: the run page, group 8 (closing); the spec's tasks are all ticked

Written 2026-11-24, 10:00 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_83_handover.md](task_83_handover.md).

## Done

Group 8, on `main`, local:

- **8.1** (`6e08e14`): a real run of `step plan --page --open` on the renderer of drift-rdp, watched
  by the owner. First attempt void (three copies, out of memory), second to its end on two copies:
  268 planted faults, 72 noticed, as in R164. Nine remarks of the owner, seven changes of the page.
  [R181](../docs/research/2026-11-24/02-run-page-a-real-run-watched.md).
- **8.2** (`1f1325b`): the page in `docs/steps.md`, `docs/state-refactoring.md` and the skill
  `untangle-state`; `step reports` documented; diffcore recommended, sentence approved by the owner.
- **8.3** (this commit): [validation.md](specs/run-dashboard/validation.md) says GO and names six
  things that stand against it; `docs/releases/2.2.0.md`; the opener with a stand-in in a Linux
  container. [R182](../docs/research/2026-11-24/03-run-page-closing.md).

## State

- Windows at the code of `6e08e14`: core suite 471 of 471, 1 skipped; lints, format, doctests, size
  budgets, `tools\bin\kiro-cli.exe code check` pass.
- Linux through Dagger at `1f1325b`: overall 0; core suite 470 of 470, 1 skipped.
- The pages in Chromium through Playwright: 67 of 67, none skipped.
- `origin/main` is at `bb10ad7`. Groups 3 to 9 and their records are local; the owner chose not to
  push. The humanizer pass on the new documentation is done.
- The spec is marked `implementation-complete`; every task is ticked.

## What the next session has to know

- **Nobody has watched the changed page follow a run.** The owner saw the page of R180 live and
  the changes in pictures. `…\real-run-8.1\plan-new-page.html` and `attempt-1\plan-new-page.html`
  hold the two real runs in the changed page; a script outside the tool made them.
- **The page now**: the steps of a run are a lane (`ol.lane`, `button.stop`) with the result as
  the last stop; the tree has one row per stage that plants faults (`button.grp`), shut until the
  stage begins; a function with faults waiting and none judged is "not begun" (`standing`,
  `rowAs`); what a command printed is behind "What the tool recorded" (`Raw`); at its end a run
  stands at the result unless a step is picked (`atResult`). `.mark` is now in the lane and the
  tree.
- **Proposed to the owner, not agreed** (R181, "Not done"): the tool says whether a build or a test
  failed in a copy; keeps the whole output of a failed first run; tries a stopped copy again;
  chooses the number of copies from free memory. `runner.rs` is at 2,799 of 2,800 tokens and
  `page_log.rs` at 2,790, so new code needs new modules.
- **Memory**: three copies of the renderer building at once failed with 8.4 of 29.6 GiB free; two
  worked. Do not run a build beside a fault run on this machine.
- **The brownfield test stage** is the owner's next direction:
  [brief](specs/brownfield-test-stage/brief.md). Decided: the stage is a spec, worked with Kiro,
  whose product is the tests; it lives in the project under work; it comes after this group. Open:
  what "protected enough" is, and who may write a fault off. No `spec.json` yet; the next step is
  `kiro-spec-init` and the requirements.
- `step pin` needs a step declaration; a developer who only has a plan has none. The brief's
  design has to say how the tests of the stage are admitted.

## Next

1. With the owner: push or not; the version of the next release; the four proposed changes of the
   tool.
2. The spec `brownfield-test-stage`, from its brief. Next research record R183; next handover
   `task_85_handover.md`.
3. Carried, not rechecked: the `kiro-cli` extension for state metrics in openidea-kiro and the
   stopping-rule test of R163; the branch `drift-rdp` unmerged; the hook test that fails under
   load. Slice B, the hardening against drift of the model: closed by the owner; only the drift of
   one model over time is unmeasured.

## Storage

D: had 120.2 GiB free at 09:57; 120.3 at the start of the session. The shared Cargo target
directory is 76.0 GiB (shared with other projects, lesscode's share not separated); no target
directory was added. Both limits hold as far as they can be told.

Removed: the three working copies of the real run (12.9 GiB at their largest), and the container,
network and two volumes of the Linux check (`docker compose down --volumes`; none left, checked).
The Dagger engine and the Podman machine are shared and stay.

Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`: `real-run-8.1` (0.9 MiB), `states-g8-1`
(11.6 MiB), `linux-open` (0.2 MiB), `budgets-g8.json`; and what the earlier handovers name. Linux
evidence: `…\native-evidence\linux-1f1325b` (0.5 MiB). Evidence grew by about 13 MiB in this group.

## Amended 2026-11-24, 10:45: the four changes of the tool are made

The owner: "push main, then make the four tool changes". `origin/main` was pushed at `3fe351b`.
The changes are in [R183](../docs/research/2026-11-24/04-turns-by-memory.md): new files
`first_run.rs` and `memory.rs` under `native/src/adapters/driver`; a copy says whether a build
ended or a test failed, keeps the whole output (`first-run-w<copy>-<attempt>.log`), tries once
more; every run of the project's commands waits for its turn by free memory. "Proposed, not
agreed" above no longer holds. "Do not run a build beside a fault run" no longer holds for the
tool's own copies; a build of yours beside a run is still not counted by the tool.
Windows 478 of 478; the pages 67 of 67. Next research record R184. Kept in addition under
`D:\BuildArtifacts\lesscode\run-dashboard\`: `first-run-renderer-1`, `first-run-renderer-2`,
`first-run-smoke`, `states-g10-1`, `budgets-g10.json`; their working copies are removed.

## Amended 2026-11-24, 11:05: the open decisions are made

The owner chose from choices: the next release is **2.2.0** (notes in
[docs/releases/2.2.0.md](../docs/releases/2.2.0.md); `native/Cargo.toml` is not bumped and nothing is
tagged); for the brownfield test stage a function is protected enough when **every fault is noticed
or written off**, and a fault is written off when **the agent proposes the reason and the owner
approves it**; the changed page is watched **with the first run of that stage**. All four are in
the [brief](specs/brownfield-test-stage/brief.md), which has no open question left. Under "Next",
point 1 is settled except for the tag.

## Amended 2026-11-24, 11:15: the spec of the brownfield test stage is begun

`origin/main` was pushed at `dd053ee`. The owner: "push it, then start the brownfield spec. since we
copy from kiro and re-use the tdd it should be relatively straight forward". Written:
[spec.json](specs/brownfield-test-stage/spec.json) and
[requirements.md](specs/brownfield-test-stage/requirements.md), seven requirements and five points
proposed for approval. **The requirements are not approved.** The next step is the owner's
approval, then the design with its prototype gate on the renderer of drift-rdp, which the owner
watches. How Kiro's test-first protocol is reused: the planted fault stands where the missing
implementation stands in Kiro, so "red" is the test failing on the code with the fault and "green"
is the test passing on the unchanged code.

## Amended 2026-11-24, 11:50: requirements approved, a provisional design and its gate written

The owner approved the requirements ("approved, go on to the design"). Written:
[design.md](specs/brownfield-test-stage/design.md) (provisional),
[research.md](specs/brownfield-test-stage/research.md) and
[prototype-gate.md](specs/brownfield-test-stage/prototype-gate.md) (not run). The design: the stage
is a step that adds tests and changes no code; `step plan`, `step pin` and `step check` with a
declaration's `focus` carry it (as in the step `escape-html`, R94/R95); to build are two small
commands (`step protect begin`, `step protect result`) and a skill. **One open contradiction
blocks approval**: Rust keeps unit tests inside the file of the code, and `step pin` carries only
files under `test_paths` into its copy. The gate tries two placements of the tests on
`display_path.rs` of the renderer, in an isolated clone, with the owner watching the first plan
and naming the functions. Next: the owner at the screen, then the prototype (R184).

## Amended 2026-11-24, 13:15: the prototype of the brownfield test stage has run; the design waits for approval

[R184](../docs/research/2026-11-24/05-brownfield-stage-prototype.md). In an isolated clone
(`D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`, branch `protect/display-path` at
`4da65d8`; the checkout of drift-rdp is untouched) four tests in a file of their own turned six of
nine unnoticed faults of five functions of `display_path.rs` into noticed ones; the owner accepted
the tests and wrote the other three off, in one review. The gate says `proceed`; its limit of ten
minutes per pin was passed once (17 min 50 s, the base's run) and the owner accepted that. The
design is revised in six points and requirements 2.5, 4.3, 4.4 and 7.3 are amended to one review.
**Next: the owner approves the design; then `kiro-spec-tasks`.**

For run-dashboard, chosen by the owner and not done: when a run ends, an end screen, and the tool
opens the saved report so that the tab that stays survives a reload ("once the process is down,
current report info is lost"). Done at `b52718a`: the line over the bar says when all faults were
run.

Not merged anywhere: the clone's branch. Whether its tests go into drift-rdp is the owner's.
Storage: the prototype's copies took 34.5 GiB and are removed; D: has 120.1 GiB free; kept
`D:\BuildArtifacts\lesscode\brownfield-test-stage\prototype-1` (16.1 MiB) and the clone. Next research
record R185; next handover `task_85_handover.md`.

## Amended 2026-11-24, 13:30: design approved, tasks written

`origin/main` was pushed at `9f6aab7`. The owner approved the design ("approved, push and write the
tasks"). [tasks.md](specs/brownfield-test-stage/tasks.md) is written and **not approved**: five
groups, twelve tasks. 1 `step protect begin`; 2 `step protect result`; 3 the skill and its
templates; 4 the guide; 5 closing, with the stage run by its commands on another renderer file and
on a project that is not Rust, both marked "can fail". Next: the owner's approval of the tasks,
then group 1 in the foreground with explicit task numbers.
