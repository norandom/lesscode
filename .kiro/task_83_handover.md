# Task handover 83: the run page, group 9 (short first)

Written 2026-11-24, 07:50 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_82_handover.md](task_82_handover.md).

## Done

Group 9, task 9.1, on the owner's request of 2026-11-24 and before group 8: the page is short
first. It answers what this is, what it found and what to do, before any table; the numbers and
the explanations are behind that. Reasoned with the skill `cobesy`. Only `native/page/run.html`
and `native/page/list.html` changed, with their two checks; the data the tool writes is the same.
Commits `30cb69d` and `05955d4` on main, and one more for this record. Details and eight
deviations: [R180](../docs/research/2026-11-24/01-run-page-short-first.md).

## State

- The pages in Chromium through Playwright at `05955d4`: 66 of 66, none skipped.
- Windows at `30cb69d`: core suite 471 of 471, 1 skipped; at `05955d4` the 59 tests of the page.
- Linux through Dagger at `30cb69d`: overall 0; core suite 470 of 470, 1 skipped
  (`D:\BuildArtifacts\lesscode\native-evidence\linux-30cb69d`). Not run at `05955d4`.
- `origin/main` is at `bb10ad7`. Groups 3 to 7 and 9 and their records are local.
- Group 8 is open: 8.1, 8.2, 8.3.

## What the next group has to know

- **The owner has not seen the new page.** Nobody of the audience was asked. The record names
  what would show the design wrong: someone who sees the page for the first time cannot say after
  ten seconds whether their code was changed and what the run found. Task 8.1 is where to look.
- **The order of a result** is now: the outcome, "What to do next", "In short" (a tile per lens),
  the functions as squares, then what was there before. The browser check holds that order.
- **New parts of the page**: `ASKS` (the question of a command; a new command needs a line there
  or has none), `GLOSS` (a lens in plain words; the same list is in `list.html`), `Tiles`,
  `Strip`, `How`, the line under the bar, the marks under the tree. The Result has no part that
  opens, so the walk by keyboard meets the same stops as before.
- **Classes of the page that the checks count**: `.mark` (the tree only; the marks under the tree
  are `.sign`, the squares are `.strip i`), `.said` (one, in the toolbar; the word on a tile is
  `.verd`).
- **A tile says "unchanged"** where before and after are the same value. That is the page
  comparing two numbers; the tool could say it in the data.
- For 8.2: the guide should use the page's words (the two questions, "planted fault", "noticed",
  "unnoticed", "unprotected") and its four steps, so that page and guide say the same.
- Everything in handover 82 under "What the next group has to know" still holds.

## Next

1. Group 8: 8.1 a real run, watched by the owner; 8.2 the documentation; 8.3 validation and
   release notes. Next research record R181; next handover `task_84_handover.md`.
2. Carried, not rechecked: the `kiro-cli` extension for state metrics in openidea-kiro and the
   stopping-rule test of R163; the branch `drift-rdp` unmerged; the hook test that fails under
   load; the humanizer pass before README or guides are pushed. Slice B, the hardening against
   drift of the model: closed by the owner; only the drift of one model over time is unmeasured.

## Storage

D: had 120.3 GiB free at 07:45, after the Linux run; 122.4 GiB the evening before. No target
directory was added. Both limits hold as far as they can be told: more than 50 GiB free, no new
lesscode cache; the shared Cargo target directory was 76.0 GiB the evening before and was not
measured again.

Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`: `states-g9-4` (10.6 MiB, a picture of
every situation with the new page), `reports-g9` (1.0 MiB); and what handovers 81 and 82 name.
Linux evidence: `…\native-evidence\linux-30cb69d`. Removed: three superseded outputs of the
browser check (`states-g9-1` to `-3`, 28.7 MiB).
