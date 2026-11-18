# R160: The clipboard lease of drift-rdp goes from three cells to one; four of five registered points hold

- Report date/time and timezone: 2026-11-18, 17:45, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 17:37 (preregistration) to 17:42 (commit on the branch), from the clock
- Status: completed; four of five registered points held, one missed as registered
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 2 (consolidate); brief only, no requirements yet
- Hypothesis ID and predecessor: a machine found in [R159](25-finder-transfer-drift-rdp.md), consolidated by hand
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md), commit `44ccb74`, before any code was written

## Decision

The owner's goal for the state work is simpler code, shown in before-and-after examples. This is
the first such example on code the tool was not written for:
[docs/examples/clipboard-lease](../../examples/clipboard-lease/README.md).

| Registered point | Result |
| --- | --- |
| 1. The crate builds, clippy `-D warnings` clean | held |
| 2. The lease's test passes with unchanged assertions | held; only the call of `guard` changed |
| 3. Library suite has the same pass count | held: 416 of 416 before and after |
| 4. Finder: three machines before, none after, one shared cell | **missed**: three before, one after. `owner` is still reported as a machine, by the condition in `guard` |
| 5. Cells 3 to 1, `if` conditions do not increase | held: 3 to 1, and 5 to 3 |

Point 4 was a wrong prediction about the finder, not a fault of the change. The finder's answer is
the better one: the machine is still there, in one cell. I had expected the finder to lose it
behind the plain fields.

The change was made by hand. No command of this project proposed it or checked it; the finder
only named the three cells.

## Hypothesis and falsification

As registered: the three cells `owner`, `claim_sequence` and `valid` are one machine and can be one
value behind one `Mutex`, with every transition a method of it, without changing what the existing
test observes. Falsified by a failing build, lint or test, by a changed assertion, or by more
conditions than before.

## Frozen inputs and method

- `drift-rdp` at `b6fbcf6`; `native_clipboard.rs` is unchanged since `7eb659e` (R159).
- Branch `lesscode/clipboard-lease-machine` in `D:\Source\drift-rdp`, commit `1a49d70`, local and
  not pushed. Made in a separate worktree, so the repository's working tree stayed on `master`.
- Finder binary `6af3a9d1…` and rules `3f156fdb…`, as in R159.
- Baseline test run on the untouched branch first, then the change, `cargo fmt`, clippy, tests.
- Conditions counted by text in the part of the file above the first command.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r160): `before/`, `after/`,
`before-nextest.log`, `after-clippy.log`, `after-nextest.log`, `finder-before.json`,
`finder-after.json`.

- `valid.store(false)` stood in three methods. After the change no method ends a lease by hand:
  taking the lease out of the owner is the end of it.
- `claim_sequence` was an atomic touched only under the owner's lock. It is a plain number now.
- A lease handed to a running operation was an `Arc` with its own flag. It is a copy of three
  strings now, and `guard` compares its id with the owner's.

## Limits and interpretation

- The test is single-threaded. `guard` now takes the owner's lock for an instant where it read an
  atomic. In the old code a running operation could see `valid` turn false without any lock; in the
  new code it waits for whoever holds the owner's lock, which is held only across assignments.
- Windows only. Linux and macOS builds of `drift-rdp` were not run.
- One machine, small, and chosen because it was easy. `SessionState`, the large one, is untouched.
- The source text of the lease part did not get shorter. The claim is fewer cells and fewer
  conditions, not less text.

## Tool/spec consequence and next gate

1. Consolidation by hand works on a found machine and gives an example that can be shown. What
   stage 2 would add as a tool is not shown: nothing here was proposed or checked by a command.
2. The finder's report after the change is a usable "after" picture: one cell, five units, one
   guard.
3. Next candidates for the example set: `fatal` and `queue` in this repository's runner; then
   state in plain enum fields, which needs a finder experiment first.

## COBESY check

The missed point is in the table in bold. The example page lists what it does not show. No count
of lines or tokens is used as a measure; the one remark on text length says it did not shrink.
