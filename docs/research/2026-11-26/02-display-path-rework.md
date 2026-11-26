# R198: The display path of drift-rdp's renderer held in one value, so that a fallback and a new session cannot cross

- Report date/time and timezone: 2026-11-26, 12:05 Europe/Berlin (registration, from the clock)
- Execution interval: plan 11:32:30 to 11:57:06; change and checks 12:05 to 14:48:39; results
  written 2026-11-26 about 15:00
- Status: completed; all eight expectations hold
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

Kept. The path in force is one value. Either it is the layer with its sessions' stores, or it is the canvas with its reason. A fallback drops the stores under the same lock that `register` takes. `connect` no longer reads the path a second time to cover the race between the two. `step check` accepted the step at the fifth attempt, with growth allowed 0 tokens: three attempts were refused for size, and one stopped on a lesscode defect, fixed in `a20ecd5`. The step is recorded on drift-rdp's branch. It is not merged or pushed.

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

## Amendment, 2026-11-26 about 13:45, before any verdict: the tool could not judge a deleted file

The fourth `step check` (12:22:43 to 13:20:02) passed the size gate and finished the fault run on
the base, then stopped with exit 3 and no verdict: `cannot read …\check\step\src-tauri/src/renderer/frame_stores.rs`.
The rework deletes that file, and `step check` gave every scope file to the fault run of each side,
whether that side has the file or not. That is a defect of lesscode, not of the step: a step that
removes a file of its scope could never be judged.

The fix, in lesscode, does two things. The fault run of a side is given only the scope files that
side has. The functions to run are given only for those files. Nothing in the comparison changes:
a function of the deleted file has no counterpart after the step, is listed among the functions that
could not be matched, and counts towards its file's totals as before. A new test,
`a_step_that_deletes_a_scope_file_is_judged` (`native/tests/step_check.rs`), failed before the fix
with the same message and exit 3, and passes after it.

Of six runs of lesscode's whole suite with the fix, four passed and two failed one test,
`begin_gives_the_base_run_and_the_check_their_own_figures`. That test compares a rounded time
estimate. The same test passed alone three times, and the one run without the fix passed. The test
works from a plan's run, where every scope file exists, so the fix gives that run the same
configuration as before. The failure is reported here, not explained, and not attributed to the
fix.

The declaration, its base and its growth allowance are unchanged. The check is run again with
`lesscode-core` rebuilt with the fix. The binary is named in the results. The earlier binary is kept.

## Evidence and results

Commits on `lesscode/renderer-rework`, after the base `949127d`: the declaration `86457e6`; the change `2cf0fd8`; the new test `cea0924`; three cuts to meet the size allowance, `d391f3f`, `02142be` and `ba2963b`; and the verdict with the ledger, `4eeacaa`. Local runs at each commit: 544 library tests pass, 6 skipped. Clippy shows the 9 warnings that were there before.

| Attempt | Binary | Ran | Outcome |
| --- | --- | --- | --- |
| 1 | `1e68cdb` | 12:12:16 to 12:12:32, on `cea0924` | refused, size: 16769 to 16891 tokens |
| 2 | `1e68cdb` | 12:15:29 to 12:15:45, on `d391f3f` | refused, size: to 16786 |
| 3 | `1e68cdb` | 12:20:01 to 12:20:18, on `02142be` | refused, size: to 16770. rustfmt's trailing comma, which I counted only after formatting |
| 4 | `1e68cdb` | 12:22:43 to 13:20:02, on `ba2963b` | not judged, exit 3: the deleted file (amendment above) |
| 5 | `a20ecd5` (sha256 `4A969D52…EB12`) | 13:45:40 to 14:48:39, on `ba2963b` | accepted, exit 0, every check passes |

In the fifth run, copy 2 failed its first run of the unchanged tests on the base and passed the second, as in the plan. Verdict: `.kiro/specs/display-path-in-force/verdict.json` on the branch, and the work directory `D:\BuildArtifacts\lesscode\renderer-rework\` (`check.log`, `check.times`, `check.html`, and the logs of the earlier attempts).

| # | Result | Holds |
| --- | --- | --- |
| 1 | Tests pass. Test changes, from `git diff 949127d ba2963b`: the two tests of `frame_stores.rs` sit in `display_path.rs` unchanged, except the line that builds the value and the pruning test's read of the stores, now `stores.in_force.lock().unwrap().1.as_ref().unwrap().len()`; in gpu.rs's test, only the line that builds the value. The new test is in its own commit | yes |
| 2 | Accepted at attempt 5. Tokens 16769 to 16761, syntax nodes 13111 to 13094 | yes, after four attempts that were not accepted |
| 3 | Places holding the path: `DisplayPathState`, `SharedFrame::wake`, the session actor's `shared_frame` and the window's transparency, 4. Managed values for the renderer: `Arc<DisplayPathState>` and `RenderShared`, 2 | yes |
| 4 | `in_force: Mutex<(Reason, Option<Stores>)>`: the canvas (`None`), the layer with no stores, the layer with stores, 3. The canvas with registered stores cannot be written | yes |
| 5 | `connect` reads `display_path.path()` once. The second reading is now the answer of `display_path.register(…)` | yes |
| 6 | No function has more unnoticed faults after. See the table below | yes |
| 7 | Step scope (with `mod.rs`, which declares no function): 70 functions, 84 call edges, 13 module hops, against 70, 83 and 18. The whole renderer: 7 files to 6, 70 functions, 91 to 92 edges, 25 to 22 hops | reported |
| 8 | `git diff --quiet 949127d ba2963b` over `rdp/session.rs`, `src/`, `Cargo.toml`, `package.json`: no change | yes |

Planted faults in the functions that the step changes or names, noticed / unnoticed:

| Function | Before | After |
| --- | --- | --- |
| `register` | 3 / 0 in `frame_stores.rs` | 4 / 0 in `display_path.rs` |
| `get` | 1 / 0 in `frame_stores.rs` | 2 / 0 in `display_path.rs` |
| `wake` | 0 / 0 | 0 / 0 |
| `info` | 0 / 0 | 1 / 0 |
| `fall_back` | 1 / 0 | 1 / 0 |
| `connect` | 0 / 8 | 0 / 7 |
| `runtime_fallback` | 0 / 2 | 0 / 2 |
| `lib.rs` `run` | 0 / 38 | 0 / 37 |
| gpu.rs, `shared_frame.rs`, others | unchanged | unchanged |

The check could not match `register`, `get` and `wake` by name across the two files, so it judged those six rows by their files' totals, which do not rise. In `connect`, the unnoticed fault that dropped `frame_stores.register(…)` is gone. The call is now a condition, which a fault cannot drop as a statement. That is one fault site fewer, not a new test. In `lib.rs`, one unnoticed site went with the line that managed `FrameStores`.

## Limits and interpretation

- The three faults in Tauri glue are still without a test. `runtime_fallback`'s two are unchanged. In `connect`, the site has changed form, but no test reaches it.
- The rework was tailored to the allowance of 0 tokens. The first draft held the path in force as an enum with two named cases. The cuts replaced it with the tuple `(Reason, Option<Stores>)`. The tuple is smaller but names its cases less clearly, and the gate brought that trade-off about. Shortened imports and `new` returning the `Arc` were cut for the same reason.
- The measuring tool changed during the experiment (amendment above). The fix only lets a side skip the files it does not have. Attempts 1 to 3 judged size alone, and their figures equal what the fixed binary measures.
- Size: 8 tokens fewer. That figure is not the gain. The gain is the state: one value where two had to agree, and a race left with no place to happen.
- Module hops in the step scope fall from 18 to 13, because `frame_stores` is gone. Call edges rise from 83 to 84, because the render thread now reaches the stores through `DisplayPathState`.
- One copy of drift-rdp's tests fails its first run on a fresh copy and passes the second, in the plan and in the fifth check. That was not investigated here.

## COBESY check

- Culture: the owner chose the topic, the scope and "tests only" before any code. The decision to leave the glue untested is the owner's, and it is recorded with the time.
- Message: what changes for a reader of drift-rdp is one type to read for "which path draws". The token count is secondary.
- Adoption: the step is accepted and recorded locally. Merging, tagging and pushing wait for the owner's word.
