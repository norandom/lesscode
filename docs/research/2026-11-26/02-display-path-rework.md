# R198: The display path of drift-rdp's renderer held in one value, so that a fallback and a new session cannot cross

- Report date/time and timezone: 2026-11-26, 12:05 Europe/Berlin (registration, from the clock)
- Execution interval: plan 11:32:30 to 11:57:06; the rest after this registration
- Status: registered
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md): faults, tests and
  reduction hand in hand (a brief, not approved requirements)
- Hypothesis ID and predecessor: [R173](../2026-11-23/10-shared-frame-rework.md), the first rework
  of the same renderer (one type, one file); the stages of
  [R184](../2026-11-24/05-brownfield-stage-prototype.md) and
  [R189](../2026-11-24/10-protect-stage-shared-frame.md) protected `display_path.rs` and
  `shared_frame.rs`
- Type: preregistered

## Decision

Registered before the code changes. Filled in after the check.

## What the code holds today

Which path draws the remote picture, the GPU layer or the web canvas, is held in five places of
five files:

| Place | File | What it says |
| --- | --- | --- |
| `DisplayPathState(Mutex<(Path, Reason)>)` | `renderer/display_path.rs` | the path in force and why |
| `FrameStores { stores, wake }` | `renderer/frame_stores.rs` | the sessions drawn on the layer |
| `SharedFrame::wake` (`OnceLock`) | `renderer/shared_frame.rs` | that a store was registered |
| each session actor's `shared_frame: Option<…>` | `rdp/session.rs` | that this session draws on the layer |
| the window's transparency | `lib.rs` | the path at startup |

The first two can disagree. A store can be registered when the path is already the canvas, and a
fallback leaves the registered stores in place. `connect` (`commands/session.rs`) therefore reads
the path twice: once to decide whether a session gets a store, and once after registering it,
because "the layer may have failed while this session was being created, after the fallback told
the sessions it knew about". The second reading exists only to cover that race. The app manages
three values for the renderer: `DisplayPathState`, `FrameStores`, `RenderShared`.

## The rework, decided before it is written

1. `FrameStores` goes into `DisplayPathState`. What is in force becomes one value with two cases:
   the layer, with the stores drawn on it, or the canvas, with its reason. "The canvas with
   registered stores" has no case.
2. `register` takes the same lock as `fall_back` and answers whether the store was taken. After a
   fallback it refuses. `fall_back` drops the stores.
3. `connect` sends a session to the canvas when `register` refuses, in place of reading the path a
   second time. Its `frame_stores` parameter goes.
4. `lib.rs` manages `Arc<DisplayPathState>` and no `FrameStores`. `RenderShared` holds the state in
   place of the stores. `frame_stores.rs` is removed, and `renderer/mod.rs` loses its line.

Not changed: `rdp/session.rs` (the session actor's `shared_frame` and `use_canvas`), the
frontend, `Cargo.toml`, `SharedFrame`'s wake handle, the window's transparency.

The two tests of `frame_stores.rs` move into the tests of `display_path.rs`. Only the lines that
build the value change, and the pruning test reads the stores through the new value. The gpu test
that builds `FrameStores` changes the same way. One new test, in a commit after the change, shows
that `register` refuses after a fallback.

## Before the rework: what the tests notice

`step plan` over `display_path.rs`, `frame_stores.rs` and `commands/session.rs`, on `d1a16e7`
(exit 0, 24.6 min, 2 copies; one copy failed its first run of the unchanged tests and passed the
second):

- `fall_back` 1 noticed of 1, `register` 3 of 3, `get` 1 of 1; `new`, `info`, `path`, `wake`,
  `get_display_path`: no fault planted. The functions the rework changes are protected already.
- Three faults the rework touches go unnoticed. All three are in Tauri glue that no test can reach
  as the project builds (no Tauri `test` feature):
  - `#41` `drop_statement@222:8`, `frame_stores.register(&session_id, store)` in `connect`;
  - `#5` `drop_statement@458:4`, `state.fall_back(…)` in `runtime_fallback`;
  - `#6` `drop_statement@461:4`, the task that moves the sessions to the canvas in
    `runtime_fallback`.

  The owner chose to rework with these reported, not protected (2026-11-26, about 12:00).
- Outside the rework, the plan also found `connect`'s other 7 faults (the connection store and
  config), 9 in `collect` (code for Linux, not built here), 1 in `decide` and 14 in the other
  commands of `commands/session.rs`. The rework leaves them as they are.

## Hypothesis and falsification

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | The crate's library tests pass after the change. Test changes are only those named above: the lines that build the value, and the pruning test's way of reading the stores | a test fails, or another test or assertion has to change |
| 2 | The released flow accepts the step: `step check`, base the commit before the change, scope growth allowed 0 tokens, set now | refused or not judged |
| 3 | Places that hold the path: 5 to 4. Values the app manages for the renderer: 3 to 2 | other numbers |
| 4 | Shapes of the value in force, by hand from the types: 4 to 3 (path × stores empty or not, against layer empty, layer with stores, canvas). The one that goes, the canvas with registered stores, is the one the second reading in `connect` covers | another count, or the shape that goes is not that one |
| 5 | `connect` reads the path once, not twice | still twice |
| 6 | Planted faults: in no function of the step more unnoticed after than before (the check's own rule) | refused for faults |
| 7 | `measure-scope` over the six files of the step: functions, call edges, module hops (70, 83, 18 before) | not judged; reported, with fewer module hops expected because `frame_stores` goes |
| 8 | `rdp/session.rs`, the frontend and `Cargo.toml` are not touched | any of them changes |

What I do not expect: fewer lines in `connect` beyond the parameter and one condition, or fewer
decisions. The gain registered here is in the state: one value where there were two that had to
agree, and a race that has no place left, not size.

## Frozen inputs and method

- drift-rdp worktree `D:\BuildArtifacts\drift-rdp\lesscode-renderer-rework`, branch
  `lesscode/renderer-rework`: master `09b4ce8`, merges of `protect/display-path` (`3de500d`) and
  `protect/protect-shared-frame` (`0578e5b`), the joining lines for the plan (`d1a16e7`) and their
  revert (`949127d`). Local, not pushed.
- `lesscode-core` built at lesscode `1e68cdb`, frozen at
  `D:\BuildArtifacts\lesscode\renderer-rework\bin\lesscode-core.exe` (sha256 `BDE0AF8C…1488`).
- Run configuration, plan and the measurements before:
  `D:\BuildArtifacts\lesscode\renderer-rework\` (`run.json`, `plan.json`, `plan.log`, `before\`).
- Declaration: `.kiro/specs/display-path-in-force/step.json` on the branch.

## Evidence and results

Pending.

## Limits and interpretation

Pending. Known before the run: the three glue faults stay without a test on both sides, so the
check cannot show that `connect` and `runtime_fallback` still call what they call. The rework keeps
that glue to one call each.

## COBESY check

Pending.
