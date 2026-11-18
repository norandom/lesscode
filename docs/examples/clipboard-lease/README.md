# Example: three cells that were one state machine

Source: `src-tauri/src/commands/native_clipboard.rs` in `drift-rdp`, a remote desktop client.
Before is commit `b6fbcf6`; after is `1a49d70` on the local branch
`lesscode/clipboard-lease-machine`. Research record: [R160](../../research/2026-11-18/26-clipboard-lease-consolidated.md).

## What the code does

One session at a time owns the computer's clipboard. A session first reserves a ticket, then
claims ownership with it, and gets a lease. A clipboard read or write that is already running
must stop as soon as its lease is no longer the owner.

## Before

[before.rs](before.rs). The answer to "who owns the clipboard" is spread over three places:

| Cell | Kind | Changed in |
| --- | --- | --- |
| `owner` | `Mutex<Option<Arc<Lease>>>` | `reserve`, `claim`, `release` |
| `claim_sequence` | `AtomicU64` | `reserve`; tested in `claim` |
| `valid`, one per lease | `AtomicBool` | `reserve`, `claim`, `release`; tested in `guard` |

Each method that takes a lease out of `owner` must also remember to set that lease's `valid` to
false. Three methods do so, each with its own `if`.

## After

[after.rs](after.rs). One value, `Owner`, holds the ticket count and the current lease, behind
one `Mutex`. Every change of owner is a method of `Owner`. `valid` is gone: a lease is valid
exactly while it is the owner, so `guard` asks the owner.

## What changed, counted

| Measure | Before | After |
| --- | --- | --- |
| Cells that hold the machine's state | 3 | 1 |
| Places that must end a lease by hand (`valid.store(false)`) | 3 | 0 |
| `if` conditions in the lease part | 5 | 3 |
| State machines the finder reports in the file | 3 | 1 |
| Library tests of the crate | 416 of 416 | 416 of 416 |

The finder's reports are [finder-before.json](finder-before.json) and
[finder-after.json](finder-after.json). It is a research program, not a released command.

## Complexity before and after

Measured with `kiro-cli` 1.1.0 ([R162](../../research/2026-11-18/28-complexity-before-after-kiro-cli.md)).

| Measure | Before | After |
| --- | --- | --- |
| Decisions in the file (cyclomatic sum minus routines) | 14 | 12 |
| Cyclomatic, `reserve`, `claim`, `release` | 2, 3, 2 | 1, 1, 1, with 1, 2, 2 in the methods of `Owner` |
| Cyclomatic, largest routine | 8 | 8 |

## What this does not show

- The existing test for the lease passes with the same assertions; only the way `guard` is
  called changed. That test runs on one thread.
- `guard` now takes the owner's lock for an instant where it read an atomic before. No test
  here measures that under load.
- Built and tested on Windows only.
- The state machine is not gone. It is in one type where it was in three cells.
