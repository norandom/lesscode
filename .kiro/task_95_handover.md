# Handover: task group 1 of `report-set`, the dashboard as a module of its own

Written 2026-11-27, about 11:00 Europe/Berlin (from the clock). Start the next group (2, the set
page) from here. Spec: [tasks](specs/report-set/tasks.md), [design](specs/report-set/design.md).
Record: [R201](../docs/research/2026-11-27/02-dashboard-module.md).

## What was done

- **1.1, `2db0f8d`.** The ten `page_*.rs` files of `native/src/adapters/driver/` are
  `native/src/dashboard/{list,log,names,open,report,serve,snapshot,subject,text,watch}.rs` with a
  `mod.rs` that says what belongs there. Only module paths, imports and the three `include_str!`
  paths changed. Users changed their imports: eight files under `step`, `runner.rs` and
  `run_command.rs` in the driver, nine `run_page_*.rs` tests.
- **The stop point fired:** `lib.rs` needed `pub mod dashboard;` and came to 198 logical units
  against 197. The owner chose one more registration-only line (ceiling 198, recorded in the
  entry's allocation in `native/complexity-budgets.json`). `dashboard/mod.rs` is listed there,
  owner `report-set`.
- **1.2, `a6a8098`.** `native/tests/dashboard_border.rs` (three tests) keeps the border; the
  fragment scan in `driver_contract.rs` reads `src/dashboard` too. Notes with the new paths in
  `.kiro/steering/structure.md` and the head of the `run-dashboard` design.

## Evidence

`D:\BuildArtifacts\lesscode\dashboard-move\`:

- `before-nextest.log`, `after-nextest.log`, `after-1.2-nextest.log`: 552, 552 and 555 passed.
- `pages-before.sha256`, `pages-after.sha256`, made by `pages.ps1`: eight pages, the same bytes.
- `budgets-before.json`, `budgets-after.json`, `budgets-1.2.json`.
- `border-trials.log`: the three trials failing the border test.
- `dagger-1.1.log`: "All native checks passed" on `2db0f8d` in 707 s. `dagger-1.2.log`: "All native checks passed" on
  `a6a8098` in 411 s.
- `move.ps1`: the script of the move.

## How the browser checks were run

They need `LESSCODE_PAGE_CHECK_OUT` and `PLAYWRIGHT_BROWSERS_PATH=D:\BuildArtifacts\playwright`.
To run all 67 without skips, first create a directory, set `LESSCODE_CHECK_REPORTS` to it, and run
the tests `run_page_check` and `run_page_list` with it set; both write into it. Set for only one of
the two, a check fails instead of skipping.

## Current state

- `main` is ahead of `origin/main`; nothing is pushed. Push only on the owner's word.
- Linux check of `a6a8098` through Dagger: "All native checks passed" in 411 s (`dagger-1.2.log`).
- No container of this group is left; the shared Dagger engine was used and is not ours to prune.
- Docker Desktop is running (started by the agent this morning for the gate's Linux leg).

## Storage

- Start of the group: D: 127.7 GiB free. `D:\BuildArtifacts\lesscode-cargo-test` 15.5 GiB (reused,
  the established target directory of lesscode), `D:\BuildArtifacts\lesscode` 5.7 GiB, together
  21.2 GiB of the 40 GiB limit, counted as the earlier handovers count. Not counted there and also
  lesscode's: about 13.5 GiB of lesscode's artifacts inside the shared
  `D:\BuildArtifacts\cargo-target` and the inactive `cargo-target\model-seam-prototype*` (8.4 GiB,
  the model-seam gate of 2026-11-14). Counted with them the total is about 43 GiB, over the limit;
  nothing was cleaned, because the shared directory is not ours to clean wholesale and the
  model-seam builds are named in an earlier handover. **The owner may want to decide whether the
  model-seam prototype builds can go.**
- Where the 21.4 GiB of D: went during the group, as far as measured: `lesscode-cargo-test` grew 6.9 GiB (the builds, test builds and Clippy of the moved crate, in the reused directory). Docker Desktop's disk image `D:\Docker\disk\docker_data.vhdx` is 89.3 GiB; inside it the shared Dagger engine's volume is 21.6 GB after two check runs. The image was not measured at the start, so its share of the drop is not known; it does not shrink by itself. The engine and its volume are shared and were not pruned. D: stays above 50 GiB; lesscode's caches, counted as before, are 28.1 of 40 GiB.
- End of the group: D: 106,3 GiB free; `lesscode-cargo-test` 22,37 GiB, `D:\BuildArtifacts\lesscode` 5,74 GiB. No isolated build directory was made. The two copies of the
  binary in the evidence directory (19 MiB each) and the pages are removed at the end; the logs,
  hashes, budgets files and scripts are kept (under 1 MiB).

## Next

1. Group 2, task 2.1: the tool's own page shows a set written into it. Prototype to read:
   `D:\BuildArtifacts\lesscode\report-set\page\run.html` (its set mode) and `checks\set.spec.mjs`.
   The fixed set for the checks comes from `D:\BuildArtifacts\lesscode\report-set\out\set.json`;
   it holds no name of the Python project.
2. Still open with the owner: the look of the gate's two PDFs (remarks go into task 2.3), whether
   to push, and the storage question above.
3. Carry-over: release 2.2.0 waits for the owner's approval of the `release` environment; the agent
   does not approve it. Drift hardening stays closed by the owner (R104); only the drift of one
   model over time is unmeasured.
