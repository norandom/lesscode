# R158: Following a cell through its names, the finder matches the hand census of mmhmem on every line

- Report date/time and timezone: 2026-11-18, 16:15, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 16:07 (preregistration) to 16:15 (final replay), from the clock
- Status: completed; passed as registered. A fit to a known census, not a transfer test
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 1 (find); brief only, no requirements yet
- Hypothesis ID and predecessor: the revision proposed in [R157](23-finder-on-mmhmem.md)
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md), commit `885e54e`, before the revision was written

## Decision

The first scored run of revision 3 met every registered line. Nothing was adjusted after it.

| Production cell of `mmhmem` | Census | Revision 2 (R157) | Revision 3 |
| --- | --- | --- | --- |
| `state`, `download_line.rs:132` | machine | no access | machine; 200, 214; conditions 203, 220 |
| `stop`, `release_local.rs:37` | machine | one unit | machine; 59, 83; condition 59 |
| `max_scroll`, `ui/app.rs:87` | machine | one unit | machine; `app.rs:361`, `app_draw.rs:92`; condition 361 |
| `step`, `check_session.rs:47` | shared, no condition | one unit | shared; 63, 127, 129, 132 |
| `buf`, `check_session.rs:232` | shared, no condition | no access | shared; 238, 252 |
| `mismatch`, `release.rs:410` | shared, no condition | machine | shared; 436, 443 |
| `out`, `download_line.rs:133` | one unit | no access | one unit; 233 |
| `conn`, `storage/db.rs:37` | one unit | no access | one unit; 129 |

For all eight the access lines are exactly the census lines. The struct literal `db` is no longer
listed. The six test-double cells keep their verdict and lines. No other cell of the repository is
reported as a machine.

On this repository the reply machine, the renamed copy and the unguarded copy give the same lines as
in R156, and the four files at head have no machine. In the sweep `fatal` now has all four of its
places, with line 382.

This shows that the four causes named in R157 were the causes. It does not show that the finder
transfers: the revision was written with the census open. The next run is a third repository with
this revision frozen.

## Hypothesis and falsification

Registered: the three machines found, `mismatch`, `step` and `buf` shared without a condition, `out`
and `conn` in one unit, access lines equal to the census, `db` gone, the test doubles unchanged, the
R156 rows and controls unchanged, `fatal` with 382. Prediction: the three machines are found and at
most one of the eight has a missing or extra line. Result: none has.

## Frozen inputs and method

- `mmhmem` at `452fb7c`, the copy of R157; census of R157, SHA-256 `4429c034…344a55dc`, pinned in the
  run directory before the revision was written.
- [Finder](../../experiments/state-consolidation/machine-find/main.rs) revision 3 and
  [rules](../../experiments/state-consolidation/machine-find/rust.state.json), now 30 lines and 2,008
  characters (SHA-256 `3f156fdb…fd6b30ae`).
- What the revision does. A cell is reached also under a name bound to a clone or reference of it;
  under the parameter it is passed as, in the function called (one of the same file, otherwise only
  when the name is unique); as a field through any receiver in its own file and through `self` inside
  an `impl` of its type in any file. A local cell put into a struct literal or a tuple struct is that
  field's cell. A test of the handle that `lock` returns is no condition on the value. `let … else` is
  a condition, and names bound inside a pattern count. A struct literal that contains a cell is no cell.
- The first run's binary and the final one after formatting gave identical outputs on both
  repositories.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r158): `run1/mmhmem.json` (the scored
run), `final/` (replay with the formatted binary, and the R156 script's outputs), `census-r157.md`,
`sha256.txt`.

- `mmhmem`: 43 listings (50 before), 3 machines, 9 shared without a condition, 31 in one unit; 12
  listings without an access (34 before), all in test code.
- This repository, sweep of 97 files at head, read in the source:

| Cell in `runner.rs` | Revision 2 | Revision 3 | Read |
| --- | --- | --- | --- |
| `fatal`, 126 | machine, 3 places | machine, 4 places (382 added) | right |
| `queue`, 123 | one unit | machine: 264, 273 in `worker`, 379 in `run_batch`; condition 264 | right: the worker's loop ends when the queue is empty. Two more lines, 268 and 272, are listed as conditions on a bound name and are wrong |
| `results`, 124 | one unit | shared, no condition: 269, 384 | right |
| local `shared`, 358 | listed | not listed | right: a struct literal |

- The two wrong lines: `job` is bound from the queue at 264, and the `match` at 267 tests what
  `self.one(…, &job, …)` returns. The rule counts any later condition that mentions the bound name.

## Limits and interpretation

- A fit. Seven rule changes were made against eight known cells; that they agree says little about a
  ninth.
- The bound-name rule over-counts, as seen on `queue`. It did not change a verdict here.
- A function is resolved by name. With several of the same name in other files the hand-over is
  dropped without a note in the report.
- A field reached through another receiver than `self` is matched by its name within the file. Two
  types in one file with a cell field of the same name would be mixed.
- Still not seen: captured mutable variables (`judged` in the guide adapter), state that is not in a
  cell type at all, more than one binding between a read and its condition.
- `lock` still counts as a write and a read.

## Tool/spec consequence and next gate

1. Third repository under `D:\Source`, revision 3 frozen, census by hand first and sealed before the
   finder runs. That is the transfer test R157 was meant to be.
2. This repository now has two machines in one struct that were never written down: `fatal` and
   `queue` in the driver's runner. They are the natural second case for stage 2.
3. Queued as before: the `match`-arm prototype for the Rust pack.

## COBESY check

The table shows three columns so that the change is visible against the failure it answers. The word
"fit" is in the status line. The two wrong lines found on our own code are reported. No count of lines
or tokens is used as a measure of success.
