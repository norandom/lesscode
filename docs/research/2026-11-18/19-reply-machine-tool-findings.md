# R153: On the reply-checking machine today's commands find the loop's guards and miss the state

- Report date/time and timezone: 2026-11-18, 14:40, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 14:33 to 14:38 (from the clock)
- Status: completed
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 1 (find); brief only, no requirements yet
- Hypothesis ID and predecessor: repeats the reply-checking row of [R130](../2026-11-17/17-hidden-state-machine-fitness.md) with the commands of [R151](17-pack-driven-call-counts.md)
- Type: exploratory; five commands on current source, no model call, no code change

## Decision

The reply-checking machine is live code in this repository, small, and its parts were written down by
reviewers in R130 before this run. Of its six parts, `analyze` and `measure-scope` report two. They
miss where the state is kept, where it is written and where it is consumed. Three causes are named
below. The case is a suitable first real target for finding and then consolidating a machine.

| Part of the machine | Where | Reported today |
| --- | --- | --- |
| State is kept: `let stop = Cell::new(None)` | `adapter.rs:191` | **no**; neither a site nor a gap |
| State is written: `stop.set(Some(reply.stop.clone()))` inside the `call` closure | `adapter.rs:195` | **no**; only as call arguments |
| State is consumed: `match stop.take()` inside the `check` closure | `adapter.rs:199` | **no** |
| Guards on the consumed state: the `CutOff` and `Refused` arms | `adapter.rs:200-203` | **no** |
| Guards of the loop: passed, repairs left or refused, whole reply | `repair.rs:127, 131, 139` | **yes**, three branch conditions |
| Who moves it: `one` hands both closures to `with_repairs`, which calls them in order | `adapter.rs:213`, `repair.rs:115, 119` | **partly**: the edge `one` to `with_repairs` is there; the calls of the two closures are not |

## Hypothesis and falsification

R130 said the released analyzer "missed both operations on the stop slot and emitted no link between
the callbacks". The question was whether the call edges added today change that. They would have, if
the scope report linked `with_repairs` to the closures or any command listed line 191, 195 or 199 as
state.

## Frozen inputs and method

- Source `1eb106f`; binary of R152 (SHA-256 `d8d51d80…fe34ab2`). Input hashes are in the run directory.
- Files: `tools/guide-optimizer/src/adapter.rs`, `model/src/repair.rs`, `model/src/check.rs`,
  `model/src/request.rs`. `check.rs` is R130's negative example: it shows outcomes and decides nothing.
- `deconvolute analyze --lang rust --source F` per file; `deconvolute measure-scope --lang rust --root .`
  over the four. The expected parts are R130's, read against the source at the lines above.

## Evidence and results

- `analyze` on `adapter.rs`: the function `one` is a boundary (lines 176 to 265) and has one state
  write, `judged = Some(verdict)` at 210. Lines 191 to 204 appear only as call arguments. The two gaps
  reported are the `Mutex` fields at 121 and 122.
- `analyze` on `repair.rs`: the three conditions of `with_repairs` and the two writes (`attempts.push`,
  `asked = …`) are listed.
- `measure-scope`: 21 function names, 33 edges, 7 between files. `one` to `with_repairs` is a real
  call. Two edges out of `with_repairs` are wrong: to `Rules::check` in `adapter.rs` (line 78) and to
  `check` in `check.rs`. `with_repairs` calls its parameter named `check`; the functions of that name
  are other things.

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r153)

## Limits and interpretation

Three causes, each visible in the output:

1. **A closure is not a function for the driver.** The Rust pack names `function_item` only. The two
   closures that hold the write and the read have no boundary and no name, and a call of a parameter
   is matched by name to unrelated functions.
2. **A `match` arm is not a condition.** The pack lists `if` and `while`. The guards on the consumed
   state are match arms.
3. **A method call on a shared cell is not a write.** `stop.set(…)` is an argument of `inspect`, and
   `stop.take()` is the subject of a `match`; neither is a statement. The gap rule knows `RefCell<`,
   `Mutex<` and `RwLock<` as written types, and `Cell::new(None)` has no written type.

One case, one language. Whether these three causes are the common ones is not known. The table checks
whether a part is reported; it does not test whether a reader could rebuild the machine from the report.

## Tool/spec consequence and next gate

The R140 port should not start from the prepared job machine alone. Proposed next experiment, on this
case: (a) a find step that returns the six parts with file and line, given only the four files; (b) the
machine and a one-place version as two arms of a history comparison, each run on the real
`with_repairs` with scripted replies; (c) the project's own tests on the one-place version. The
one-place version is likely a check that receives the reply's stop reason directly, which removes the
cell. That changes a signature of `lesscode-model` and is the owner's decision.

## COBESY check

The table is first and each row has a line number to check. The expected answer was written a day
earlier by others. Two wrong edges are reported as wrong. No claim is made beyond this case.
