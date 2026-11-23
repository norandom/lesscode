# Task handover 80: the run page, group 5

Written 2026-11-23, 19:05 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_79_handover.md](task_79_handover.md).

## Done

Group 5, tasks 5.1, 5.3 and 5.4: what the run touches, at the top of the page and in the tab;
faults in code that is not built on this system, counted apart; the notices where tests are missing
or do not pass. Task 5.2 (what a planted fault is, with one of the run) is written and committed
but **not ticked: the owner has to read the wording** of the tab "Planted faults". Commits
`8093f1c` (5.1), `94ea387` (5.2), `0c46ae9` (5.4), `30f2959` (5.3) on main. Details, six deviations
and my faults: [R177](../docs/research/2026-11-23/14-run-page-group-5.md).

## State

- Windows at `30f2959`: core suite 460 of 460, 1 skipped (the replay of a recorded run, which needs
  files outside the repository). Lints, format, doctests, budgets and `kiro-cli` pass.
- Linux through Dagger at `30f2959`: overall 0; core suite 459 of 459, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-30f2959`)
- The page in Chromium through Playwright at `30f2959`: 41 of 41.
- `origin/main` is at `bb10ad7`. Groups 3, 4 and 5 and their records are local.

## Owner decisions of this group

- The admission limit of a pack is 50 lines and 3,200 characters (was 40 and 2,500), so that the
  Rust pack can name the conditions under which code is not built on a system.

## What the next group has to know

- `native/packs/rust.json` has `not_built_markers`. `sites.rs` has `not_built_spans` and `within`;
  `plan_page.rs` has `class_at`, which turns a fault under such a condition into
  `Class::NotCompiledHere` for the page's log. **The run and the plan do not know this**: they
  still plant those faults and count them as unnoticed. The page and the plan differ on a function
  that holds only such faults. Leaving them out of a run was not decided.
- R164 said 64 such faults on the renderer; it is 66. R164 has an amendment.
- `step plan` now sets what it writes into the repository (at the beginning) and the space of the
  copies (at the end): `written_inside` and `space` in `page_subject.rs`, called from
  `plan_show.rs`. `step check` (task 6.1) has to do the same.
- The warning on the main branch shows whenever the run reads the main branch. Requirement 4.5
  wants it where a change is judged; the snapshot has no field for that yet. Task 6.1 is the place.
- The notice "Your test command ran no test" is in the page; no run sets `tests_run` to 0 today.
- The fixed situations in `native/page/states.spec.mjs` now also say what a tab shows and hides
  (`tabs`), in what order texts come (`order`), and how many marks of a kind there are (`signs`).
- The check in a browser, from the repository root, as in handover 79:
  `$env:LESSCODE_PAGE_CHECK_OUT = 'D:\BuildArtifacts\lesscode\run-dashboard\<new name>'`, then
  `node verification/node_modules/@playwright/test/cli.js test --config verification/playwright.config.mjs`.
- `kiro-cli` is at `D:\BuildArtifacts\cargo-target\release\kiro-cli.exe`; it wants a revision:
  `kiro-cli code check --changed <commit before the group>`.
- Guards as before: nothing with the name of a shipped language in `native/src/adapters/driver` or
  `native/src/step`; no `.replacen(` under `native/src/adapters`; `runner.rs` is at 2,799 of 2,800
  tokens and was not touched.
- Still not seen: `--open` starting a real browser; the page following a real run; Copy on the
  served page; Firefox, Safari, a screen reader; a real run with the new pack.

## Next

1. The owner reads the wording of the tab "Planted faults"; then task 5.2 and group 5 are ticked.
2. Group 6, tasks 6.1 to 6.3: `step check` tells the log; the report; the two drawings.

## Storage

D: had 120.1 GiB free at 18:56, 120.3 GiB at the start. No target directory was added; only
incremental builds in the shared Cargo target directory (75.3 GiB at 17:00, shared with other
projects, the share of lesscode not separated). The Linux run uses the shared Podman storage, which
is not this project's to clean.

Kept under `D:\BuildArtifacts\lesscode\run-dashboard\`: `states-g5-final` (4.7 MiB, the last check
with a picture of every page and tab), `r164-renderer-report.html` and `.png` (0.2 MiB),
`r164-source` (0.1 MiB, the renderer sources at `09b4ce8` for the replay), `budgets-g5.json`.
Linux evidence under `…\native-evidence\linux-30f2959`. Removed: five superseded outputs of the
check (`states-g5-red`, `-1` to `-4`, about 20 MiB).
