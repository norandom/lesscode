# R157: On a repository it was not written for, the finder lists every cell and finds none of its three machines

- Report date/time and timezone: 2026-11-18, 16:05, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 15:52 (preregistration) to 16:04 (census written), from the clock
- Status: completed; rejected as a tool to port in its present form
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 1 (find); brief only, no requirements yet
- Hypothesis ID and predecessor: next gate of [R156](22-find-step-reply-machine.md)
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md) at 15:52, commit `03e2de4`, before the run

## Decision

The finder of R156, unchanged, ran over all 236 Rust files of `mmhmem` at `452fb7c`. Every file parsed.
It lists the declaration of all eight cells of the production code. It reaches the accesses of one of
them. None of the three machines among the eight is reported, and the one machine it does report is
not one.

| Production cell | What it is, read in the source | Finder | Why |
| --- | --- | --- | --- |
| `state`, `download_line.rs:132` | **machine**: `finish` and `draw` share it, the redraw is throttled on it | no access | reached as `shared.state`, not `self.state` |
| `stop`, `release_local.rs:37` | **machine**: `drop` sets it, the server thread's loop ends on it | one unit | the thread reads a clone under another name |
| `max_scroll`, `ui/app.rs:87` | **machine**: set while drawing, `scroll_down` is guarded by it | one unit | the write is in another file of the same `impl` |
| `step`, `check_session.rs:47` | progress marker, two units, no condition on it | one unit | the cell is handed to a function and written under the parameter's name |
| `buf`, `check_session.rs:232` | shared between a task and `text`, no condition | no access | a clone, and a tuple field |
| `mismatch`, `release.rs:410` | shared between `check` and `mismatch`, no condition on its value | **machine** | the condition at 436 tests whether the lock was taken |
| `out`, `download_line.rs:133` | one unit | no access | `shared.out` |
| `conn`, `storage/db.rs:37` | one unit, a resource | no access | a clone |

In test code the picture is the opposite: the six cells reported as shared between units are all as
reported. They are fields of test doubles, read and written through `self` in one file, which is the
shape the rules were written on.

Find works on the case it was built on and on code of that shape. It does not transfer. The rule
registered for this was "more than one in three reported machines wrong": one of one is wrong. The
finder is not ported as it is.

## Hypothesis and falsification

Registered before the run: the files parse; the finder lists fewer than half of the census
declarations; of the cells it reports as machines at least two in three are right; at least one
reported machine misses an access.

| Prediction | Result |
| --- | --- |
| The files parse | held: 236 of 236 |
| Fewer than half of the declarations are listed | **wrong**: 8 of 8 are listed. What is missing is the accesses: complete for 1 of 8 |
| At least two in three reported machines are right | **wrong**: 0 of 1 |
| A reported machine misses an access | wrong: the one reported has both of its accesses |
| The misses are statics, clones, parameters and cells in `Arc` | partly: clones 3, another receiver 2, a parameter 1, another file 1; no mutable static exists |

## Frozen inputs and method

- `D:\Source\mmhmem` at `452fb7c`, working tree clean for Rust files, taken with `git archive`. The
  repository was only read.
- Finder binary of R156 (SHA-256 `1919526a…0cd647`) and rules `a9ff99e9…0be3e9`, unchanged.
- Census without the finder: a search over `src` for the cell words of the rules, for other `Atomic`
  types, `OnceLock`, `LazyLock`, `static`, `lazy_static` and `thread_local`; each hit read. A cell
  counts when it is mutable after creation and not under `cfg(test)`. Then every access to each of the
  eight was looked up by name and read.
- Each finder report of a cell touched by more than one unit was read against the source.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r157): `finder.json` (SHA-256
`0f70926e…e8afdd`), `census.md` (the table with every access line), `files.txt`, the source copy.

- Finder totals: 50 listings, 1 machine, 6 shared without a condition, 43 in one unit; 34 listings
  have no access at all.
- Of the 50 listings, 9 are the eight production cells (`stop` twice, as field and as local), 40 are
  test code and one is no cell: `db` at `storage/db.rs:110` is a struct literal that contains one.
- Five `LazyLock` statics are set once and not counted as state.
- Verdict right for 2 of 8 production cells (`out`, `conn`), in both without an access found.
- 13 calls of a parameter that a by-name edge would point at a function of the same name.

## Limits and interpretation

- One repository, one reader. The census is by hand and by name; a cell reached only through a name
  that was not searched would be missing from it too.
- "Machine" follows the finder's own definition: written in one unit, read in another, a condition on
  what was read. `step` is a progress marker whose reads only label a failure; by that definition it is
  shared state, though a person would call it a machine.
- The four causes are about how a cell is named at the place of access. None is about the cell kinds or
  the method names, which fitted. `let … else` is not in the rules as a condition; `download_line.rs:203`
  decides on the shared state that way.
- Whether the same causes dominate in a third repository is not known. `fatal` at `runner.rs:382` in
  this repository is the first cause again.

## Tool/spec consequence and next gate

1. The reply machine was found because its closures use the cell's own name inside the function that
   declares it. Real code mostly reaches a cell under another name. The find step needs to follow a
   cell through its names: a clone or reference bound to a new name, a parameter it is passed as, a
   field reached through any receiver, a field of a type whose `impl` spans files.
2. A condition counts only when it is on the value. A test of the lock's own result does not.
3. Proposed next experiment, to be registered before it runs: revision 3 of the finder with name
   following, scored against this census (three machines found, `mismatch` not a machine, eight cells
   with complete accesses) and against R156's rows and controls, which must not change. A third
   repository is read only after that passes.
4. The `match`-arm prototype for the Rust pack and `fatal` as a second consolidation case stay queued.

## COBESY check

The table is first and says the tool failed where it was expected to transfer. Three of five
predictions were wrong and are shown as wrong. Every row has a line to check. No count of lines or
tokens is used.
