# R156: A finder returns the reply-checking machine from the four files, and the measures of 18 November do not see what its consolidation changed

- Report date/time and timezone: 2026-11-18, 15:40, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 15:24 (preregistration) to 15:31 (final replay), from the clock
- Status: completed; registered criterion not met to the letter (two extra lines), assessed `revise`
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 1 (find); brief only, no requirements yet
- Hypothesis ID and predecessor: the open gate of [R153](19-reply-machine-tool-findings.md) to [R155](21-reply-rule-in-the-loop.md)
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md) at 15:24; the sweep and revision 2 are exploratory and marked so

## Decision

A research binary, given the four files at `1eb106f` and a rules file of 20 lines, returns the machine
that people wrote down in R130 and R153:

| Part | Expected | Returned |
| --- | --- | --- |
| kept | `adapter.rs:191` | `adapter.rs:191`, local cell `stop`, owner `one` |
| written | `adapter.rs:195`, in closure `call` | `195` (`set`, in `call`); also `199` (`take` empties it) |
| consumed | `adapter.rs:199`, in closure `check` | `199` (`take`, in `check`) |
| guards on the consumed state | `adapter.rs:200`, `203` | `200`, `203` |
| who moves it | handed over at `adapter.rs:213` to `with_repairs`, called at `repair.rs:115` and `119` | the same three lines, with the parameter each closure arrives as |
| guards of the mover | `repair.rs:127`, `131`, `139` | `116`, `117`, `127`, `131`, `139` |

Five rows are exact. The last has two extra lines: the arms of `match call(&asked)`, where a failed
call ends the loop. They are real guards of the loop that the expected answer left out. By the
registered rule an extra line fails the check, so the result is `revise`, and the revision asked for
is of the expected answer, not of the finder. Nothing else in the four files is reported as a machine.
All three controls passed.

What consolidation changed, on the same four files, before (`1eb106f`) against head:

| Measure | Before | Head |
| --- | ---: | ---: |
| Finder: cells that more than one unit touches, with a condition on them | 1 | 0 |
| Finder: units that share that cell (the two closures) | 2 | 0 |
| Finder: conditions on state the reader cannot see being written | 2 | 0 |
| Finder: hand-overs of a writer and a reader to a third function | 1 | 0 |
| By hand: conditions on a reply's stop reason | 4 | 4 |
| By hand: units that hold such a condition | 2 | 1 |
| By hand: files that hold such a condition | 2 | 1 |
| `analyze`: branch conditions, all four files | 13 | 13 |
| `analyze`: state writes, all four files | 13 | 13 |
| `analyze`: declared gaps | 5 | 5 |
| `analyze`: call-argument sites | 203 | 198 |
| `measure-scope`: edges, of which wrong ones out of `with_repairs` | 33, 2 | 33, 2 |

The rule did not get smaller: four conditions before, four after. It is in one function of one file
instead of two units in two files, and no state travels between closures any more. The commands of
18 November report none of that. Their only movement is five call-argument sites and two calls. The
conditions that were removed and the ones that were added are both `match` arms, which the Rust pack
does not list, so the mutation step has no fault to plant on this rule before or after.

## Hypothesis and falsification

Registered at 15:24, before the finder was written: one machine with the six rows above and nothing
else; a machine in `check.rs` or `request.rs`, a missing or extra line, or a failed control rejects.
Registered for the effect: `analyze` and `measure-scope` change little, the finder's counts fall to
none, the number of conditions on the stop reason does not fall, the number of places does. Registered
for the sweep: the reply machine is the only local-cell machine at `1eb106f`, none at head, and field
cells behind `Mutex` appear as shared state, most without a condition.

## Frozen inputs and method

- Before: `1eb106f`, taken with `git archive`. Head: `72849d1`. `check.rs` differs between them in one
  line, `request.rs` in none.
- [Finder](../../experiments/state-consolidation/machine-find/main.rs) and
  [rules](../../experiments/state-consolidation/machine-find/rust.state.json) (20 lines, 1,258
  characters, SHA-256 `a9ff99e9…0be3e9`). The rules say which nodes are units of code (functions, and
  closures named by their binding), which constructors and types make a cell, which methods write or
  read one, which nodes hold a condition, and how a call names its callee and arguments. The finder
  names no language and none of `stop`, `call`, `check`, `with_repairs`.
- A cell is reported as a machine when one unit writes it, another reads it, and a read is the subject
  of a condition. For a local cell the finder then looks for a call that is handed a writer and a
  reader, follows it by name to the function, and lists where that function calls the two parameters
  and its own conditions.
- Controls: head (no machine); a copy with the cell, the closures and the mover renamed (the same
  lines under `halted`, `ask`, `verify`, `drive`); a copy where the read feeds no condition (listed as
  shared, not as a machine).
- `lesscode-core` of R152 (SHA-256 `d8d51d80…fe34ab2`) for `analyze` and `measure-scope`.
- Replay: `run.ps1 -Binary <find-machine.exe> -Out <new directory>` in the experiment directory. The
  final replay after formatting gave the same bytes as the run before it.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r156): `final/` (reports, the copies
and `summary.txt`), `effect/` (the two commands' outputs), `r1/` (revision 1 outputs), `sha256.txt`.

Sweep of the project's own Rust sources, 97 files at each revision, vendored crates left out. Each
finding was read in the source.

| Cell | Where | What it is | Finder |
| --- | --- | --- | --- |
| `stop` | `adapter.rs:191`, `1eb106f` only | the reply machine | all parts |
| `fatal` | `runner.rs:126` | a run becomes void: `one` sets it after a failed restore (187) or an interrupt (190), every worker's loop stops on it (261), the run's summary reads it (382) | 3 of 4 places; 382 is missed |
| `held_back` | `guide-optimizer/src/model.rs:151` | calls kept affordable: set in `hold_back` (182), read in `chat` (201), which refuses a call at 202 | found by revision 2 only |
| `state` | `guide-optimizer/src/model.rs:155` | money spent, reserved and calls made: three places in `chat` and `spent`, condition at 196 | found by revision 2 only |

- `fatal` was not written down in any earlier entry. It is a two-state machine in one file and three
  functions, of which the finder saw two. It is not a duplicate of the error `one` returns: an error stops the worker that got it,
  `fatal` stops all of them.
- Revision 1 listed `held_back` and `state` as shared without a condition. Both reads are bound to a
  name that a later condition tests. Revision 2 follows one such binding; it changed no line of the
  reply machine or of a control. This revision was made after seeing the sweep.
- Five more cells are touched by one unit only (`queue`, `results`, a local `shared`, the adapter's
  `log` and `rejected`).
- Calls of a parameter that a by-name edge points elsewhere: 4 in the sweep, among them
  `with_repairs` calling its parameter `check`, which `measure-scope` reports as edges to
  `Rules::check` and to `check.rs`. The finder lists them; nothing corrects the edges yet.
- Predictions: the reply machine is the only local-cell machine before and there is none at head, as
  said. "Most field cells without a condition" held in revision 1 (2 of 3) and not in revision 2 (0 of
  3). The effect prediction held on every line.

## Limits and interpretation

- One language and one repository. The author knew the reply machine while writing the rules. `fatal`
  and the two budget cells were not known, and they are the only evidence that the finder finds
  something it was not written for.
- Misses seen: a field read through another name than `self` (`shared.fatal`, line 382); a captured
  mutable variable is state too and is not a cell (`judged` in the adapter, still there at head: the
  check writes the verdict into it and `one` reads it after the loop); a read that reaches a condition
  through more than one binding.
- `lock` is counted as a write and a read, so for a `Mutex` the finder cannot say which unit only reads.
- The mover is followed by name. Two functions of the same name would both be listed.
- The table's "by hand" rows are counts by a reader, not by a tool.
- Whether a reader could consolidate from the report alone was not tested. The bounded comparison of
  stage 2 was not rerun; R154 and R155 stand.

## Tool/spec consequence and next gate

1. Owner's call: accept `repair.rs:116` and `117` as part of the expected answer. With that, the find
   step passes on its first real case.
2. The rules do not fit the Rust pack (40 lines and 2,480 of 2,500 characters) and need not: as a
   second file per language they are 20 lines. Where that file lives is a design question.
3. The Rust pack lists no `match` arm as a condition. On this rule that leaves the mutation step
   without a site before and after consolidation. This is the first thing to change if the effect of
   a consolidation is to be visible to `analyze` and guarded by `step guard`.
4. Proposed next experiment: the finder on a repository it was not written against, from the owner's
   list under `D:\Source`, with every finding read and counted as right, wrong or missed; and `fatal`
   as the second case for stage 2.

## COBESY check

The expected answer was fixed before the finder existed and the one deviation is shown in the first
table. The revision made after seeing data is named as one. Counts by hand are labelled. No line or
token count is used as a measure.
