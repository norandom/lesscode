# Handover: the loop of state, faults, tests and rework has run once on real code; a subsystem is still to do

Reset 2026-11-23, 12:57 Europe/Berlin (from the clock). This file is the start for the next session.
The version before this one is in Git (`a064849`, reset of 18 November, with the notes of 22
November added up to `fc34c4e`); older ones are
[handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md).

Amended 13:07: the check named under "Running" has ended and its results are recorded. Nothing
is running.

## Start here

1. Read [the brief](specs/lesscode-metrics/brief.md). It holds what the owner wants and the six
   lenses a rework is measured by.
2. Read the research index, entries R164 to R173, all under
   [docs/research/2026-11-23/](../docs/research/2026-11-23/). Each has its expectations written
   before the run, and what failed.
3. Look at the page "Two Bowls", https://claude.ai/artifact/6ggrijcAhxUDyiEk3gw8SY (private to the
   owner). Its source is in a session scratchpad and not in the repository.

## Running (ended 13:02; kept for the record)

**Outcome:** accepted at the second attempt, growth 57 tokens; recorded and guarded on the branch
(commit `204782f`). Watched again: the tests see all 6 ways the type can now be written. 2 of 37
planted faults go unnoticed where 4 of 42 did. All of it is in
[R173](../docs/research/2026-11-23/10-shared-frame-rework.md). The working copies and build output
are removed; D: had 125 GiB free at 13:07. The steps listed below are done.

`lesscode-core step check` on the rework of `SharedFrame` in `drift-rdp`, second attempt, started
12:29. Working directory `D:\BuildArtifacts\lesscode\lesscode-metrics\r173-rework\work`, log
`…\r173-rework\check.log`, verdict written to
`D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests\.kiro\specs\shared-frame-pending\verdict.json`.
The first attempt was refused in one second: the file grew by 72 tokens where 60 were allowed
(kept as `…\r173-rework\verdict-1-refused.json`). The limit was not raised; a second commit dropped
what the new code did not need.

When it has ended:

- read the verdict; if refused, read why and fix the step, never the limit;
- then, not at the same time, watch the tests again with `scope-watch` (row 4 of R173) and read the
  planted faults of the file from the check's run (row 5);
- write the results into [R173](../docs/research/2026-11-23/10-shared-frame-rework.md), update the
  research index, commit;
- remove the working copies under `…\r173-rework\work` (three or more copies of about 4.3 GiB) and
  the worktree's `target`.

## What the owner wants (22 and 23 November)

- lesscode is to be a capable harness by itself, in Rust throughout. Long runs go through the Rust
  commands only; where one is incomplete, name the gap to the owner before any substitute. A
  PowerShell loop that starts the Rust binaries and collects their output is fine.
- Examples are reworks of whole subsystems, measured before and after, shown side by side with a
  graph and the measures, "a little artsy". Single files do not help. The reworked code need not be
  published.
- Faults, tests and reduction go hand in hand: plant faults, let the unnoticed ones drive the
  writing of tests, then reduce.
- `kiro-cli` of `openidea-kiro` stays as it is and may be used for the measures of routines and
  types. The state measurement is built here, as a command `lesscode-metrics`. Not built yet.
- `drift-rdp` may be edited: lesscode or the agent edits the checked-out code, the developer makes
  the branch. `facdrone` is read only and nothing of its source goes into lesscode. Code from other
  repositories under `D:\Source` may be copied into examples.
- Reporting matters most. Say things in words, not in record numbers. Never report a check as done
  unless it ran and its output was seen. Take times from the clock.

## What was built on 23 November, all as research programs in Rust

All are binaries of the experiment crate `docs/experiments/state-consolidation/e5-bound`; sources
under `docs/experiments/lesscode-metrics/`. None is a released command.

| Program | What it does | Where it stands |
| --- | --- | --- |
| `state-holders` | per struct: the fields that hold state, which are changed after the value is built, the shapes it can be written in, who changes and who decides | revision 3; reproduces six hand counts; one registered row failed: uses of fields through closure parameters, loops and patterns are not followed |
| `walk-gen` | writes a test that calls one type's methods in every order and notes the shapes | works on two types; over the 16 structs of a subsystem it says something for one |
| `scope-walk` | the same over all functions of a subsystem, with a world of values | compiles and runs unedited; changes the state of 1 type in 16, because the state is changed from outside or sits behind a device |
| `scope-watch` | adds one statement, in test builds only, to each function that takes a struct by reference, and reads the holders while the project's own tests run | all rows held at the first revision; the tests give the same result with it |

## What was measured on `drift-rdp` (renderer, at `09b4ce8`)

- Planted faults, released `step plan`: 268 faults in 1 hour 43 minutes; 64 lie in code compiled
  only on Linux and say nothing on Windows; of the 204 compiled here the tests notice 72 (35%), from
  81% in the frame bookkeeping to 9% in the drawing code.
- State: 35 holders in 16 structs, 18 changed after construction.
- `SharedFrame`: 16 shapes can be written (with `wake`), the code reaches 12, the tests saw 8.

## The first turn of the loop, on `SharedFrame`

On the local branch `lesscode/shared-frame-wake-tests` of `drift-rdp`, in the worktree
`D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`. Five commits on `09b4ce8`, none pushed.
The owner's checkout is untouched on `master`.

| Commit | What | Measured |
| --- | --- | --- |
| `08c6f9b` | four tests for the reached shapes no test saw | the tests see 12 of 12; faults noticed in the file 34 to 35 of 42 |
| `e4d3b2c` | two tests written from the planted faults | 35 to 38 of 42 |
| `f58b017` | the declaration of the step, growth allowed 60 tokens | |
| `18c9043` | the rework: one value for what is pending, `dirty` gone, the lock released by the shape of the code | refused for size, 72 tokens |
| `e02154e` | what the new value did not need, dropped | accepted, 57 tokens; 33 noticed, 2 unnoticed of 37 |
| `204782f` | the accepted verdict and the ledger entry | `step guard` accepts the branch |

The four faults still unnoticed before the rework are, by my reading and not shown by a run, two
that change nothing and two that keep the lock while the render thread is rung.

Counted on the reworked file: writable shapes of `SharedFrame` 8 to 3 by the program (16 to 6 with
`wake` by hand); functions 15 to 17, call edges 8 to 11; the first rework commit adds 57 lines and
removes 35. No gain in size or decisions was expected or found. The gain is in the state.

## What I got wrong on 23 November, each corrected in its record

- Twice I miscounted the faults inside a test module, and told the owner the tool plants faults in
  test code. It does not. It plants them in code compiled only on another platform.
- Times in two records were estimates, not clock readings.
- A walk kept the first sequence that reached a shape, not the shortest.
- Two start scripts had faults of their own: a variable `$gen` beside a parameter `$Gen`; a count
  from a field that does not exist.
- The four examples of 18 November were measured by a count that cannot move when behaviour is kept.

## Open findings about the tool

- `step plan` and `step check` plant faults in code the platform does not compile and count them
  as unnoticed.
- When the unchanged tests fail in a copy at the start, the run goes on with the other copies and
  ends with "not incomplete"; that a copy dropped out is only in the list of workers. It happened
  in one of three copies, then in two of three, then in none. By hand the same tests passed each
  time. Not explained; a test that fails while three copies build at once is a guess.
- `step check` counts the tests that sit inside a scope file as scope; a step must take the commit
  after the tests as its base.
- `state-holders` reads named fields only: a state kept as a pair inside a lock in a struct without
  field names is missed (`DisplayPathState`). It counts a cell that is filled once as one shape.
- A fault costs about 47 seconds on `drift-rdp`, most of it the rebuild.

## Next

1. R173 is finished.
2. Decide with the owner what follows. My proposal: the same loop on the renderer's next type
   whose state the tests reach, `RenderShared` with `View`, or the drawing code in `gpu.rs`, where
   the tests notice 9% of the faults and tests have to come first.
3. The page: the bowls show the two facdrone reworks, the grid shows `SharedFrame`. It needs the
   renderer's own bowls, the faults-and-tests view (data now exists), and to be written by a command
   instead of by hand. The owner asked whether Rust can embed a web page as Go can: `include_str!`.
4. `lesscode-metrics` as a command: the brief names what it prints. Nothing of the research
   programs is in `native/` yet; that needs a spec and the prototype gate.
5. **The run page** (spec [run-dashboard](specs/run-dashboard/tasks.md), 23 November): a page inside
   the binary that shows a run while it happens and is its report afterwards. Requirements and
   design approved by the owner; the prototype gate ran five times and decided for an event stream
   on `127.0.0.1` over rereading a file; tasks are written and wait for the owner's approval.
   Nothing of it is in `native/` yet. The prototype is under `docs/experiments/run-dashboard/`.
   The owner wants `https://github.com/jamesaphoenix/diff-core` recommended in the documentation
   later; I have not read that project.
6. **How lesscode is adopted** (owner, 23 November): through an agent that follows a skill. The
   skill is `.agents/skills/untangle-state`; it holds the loop and, in general terms, the cut for
   code no test can reach. `gpu.rs` of `drift-rdp` is such code; the cut there needs the owner's
   run of the application and was not begun.
7. Not done from 22 November: a made-up spaghetti example over several files; Python and facdrone;
   `ats-agent` (`.cjs`, not checked whether a pack reads it).

## Open, for the owner

- Whether the branch `lesscode/shared-frame-wake-tests` is wanted in `drift-rdp`, in whole or the
  tests alone. Whether `dirty` may go: the shipped program set it and never read it.
- The branch `lesscode/clipboard-lease-machine` (`1a49d70`) in `drift-rdp`, from 18 November, is
  still local and not merged.
- A caller of `SharedFrame` that writes through `begin_write` can call `mark_dirty` while it holds
  the guard. Closing that changes the interface.
- Release 2.1.0: signatures, checksums, attestation and Zenodo were not checked by me after
  publication. A note on the release pages of 1.1.0 to 2.0.0 that points Linux users to 2.1.0.
- The main README's introduction; a humanizer pass before any push of README or guides.
- The test of a rule that says when a state machine is untangled far enough is registered and not
  run. The owner found it unclear; it can be dropped.
- From earlier and still open: the `measures.rs` parenthesized-callee fix; removal of the prototype
  worktree `corrective-audit\foreground-20261118\prototype`; the history form of `bound-check`;
  Java lowering; `match` arms as branch conditions in the Rust pack.
- The hardening against model drift is closed by the owner; only the drift of one model over time
  is unmeasured.

## State of the repositories

- lesscode: `origin/main` at `93ff83d`, in sync when this was written; this handover is the next
  commit. Release 2.1.0 is the published version; what is on main after it is for the next release
  notes (see the handover of 18 November in Git).
- `drift-rdp`: `master` at `09b4ce8`, 230 ahead of its origin, untouched by me. Worktrees of mine:
  `D:\BuildArtifacts\drift-rdp\lesscode-shared-frame-tests`. A copy of its ignored `dist` folder is
  in that worktree so that it builds.
- `openidea-kiro`: untouched.

## Storage

D: had 95.9 GiB free at 12:57, with the check's copies building; 125 GiB at 12:15 with nothing
running. The limit is 50 GiB free. Each copy of `drift-rdp` that builds takes about 4.3 GiB; a
fault run makes three. I removed them after every run so far. Retained, small:
`D:\BuildArtifacts\lesscode\lesscode-metrics\r164-…` to `r173-…` (receipts, reports, generated
modules, frozen binaries under `…-bin`), and the sources of one copy of `drift-rdp` without build
output under `…\r169-scope-walk\try1\tree`, reused by the watching script.

Next research record: R183. Task handovers: [task_77_handover.md](task_77_handover.md) (the run page, groups 1 and 2), [task_78_handover.md](task_78_handover.md) (group 3), [task_79_handover.md](task_79_handover.md) (group 4), [task_80_handover.md](task_80_handover.md) (group 5), [task_81_handover.md](task_81_handover.md) (group 6), [task_82_handover.md](task_82_handover.md) (group 7), [task_83_handover.md](task_83_handover.md) (group 9, short first), [task_84_handover.md](task_84_handover.md) (group 8, closing; then the brief `brownfield-test-stage`); the next is `task_85_handover.md`.
