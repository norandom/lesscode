# Task handover 79: the run page, group 4

Written 2026-11-23, 18:12 Europe/Berlin (from the clock). Start file for the whole project:
[handover.md](handover.md). Spec: [run-dashboard](specs/run-dashboard/tasks.md). Before this:
[task_78_handover.md](task_78_handover.md).

## Done

Group 4, tasks 4.1 to 4.4: the page of the design and its view library inside the binary; the six
states and a drawing that cannot leave a blank page; a width of 400 and a second browser; the
keyboard. Commits `3f96ae7`, `63c7589`, `2f407b0`, `f103d1e`, `3872d5a` on main. Details, the walk
by keyboard, ten deviations and my faults: [R176](../docs/research/2026-11-23/13-run-page-group-4.md).

## State

- Windows at `3872d5a`: core suite 456 of 456. Lints, format, doctests and budgets passed at
  `3f96ae7`; no Rust file changed after it. `kiro-cli` passes.
- Linux through Dagger at `3872d5a`: overall 0, 455 of 455.
- The page in Chromium 156 through Playwright at `3872d5a`: 29 of 29.
- `origin/main` is at `bb10ad7`. Groups 3 and 4 and their records are local.

## Owner directions of this group

- The check of the page in a browser is a job for Playwright, laid out as in `D:\Source\QOpenBB`.
  No hand-written browser clients.
- No Firefox and no browser in a container through Podman. Do not propose a second engine again
  unless the owner asks.

## What the next group has to know

- The page is `native/page/run.html`; the tool fills `/*LIB*/` once and then `/*EVENTS*/null` or
  `/*DATA*/null`. The tests in `run_page_options.rs` and `run_page_text.rs` read the page as text.
- The check in a browser, from the repository root:
  `$env:LESSCODE_PAGE_CHECK_OUT = 'D:\BuildArtifacts\lesscode\run-dashboard\<new name>'`, then
  `node verification/node_modules/@playwright/test/cli.js test --config verification/playwright.config.mjs`.
  It needs `pnpm install` in `verification/` once (done on this machine) and
  `PLAYWRIGHT_BROWSERS_PATH`. It takes about ten seconds. It is not one of the unit tests and does
  not run on Linux in the container.
- The fixed situations are in `native/page/states.spec.mjs`. A change of the page's wording can
  break a case there; group 5 changes wording.
- When the library changes: its hash goes into `native/bundled.json`, then
  `scripts/license-inventory.ps1 -Crate native` and `-Crate model`. A unit test fails otherwise.
- Guards as before: nothing with the name of a shipped language in `native/src/adapters/driver` or
  `native/src/step`; no `.replacen(` under `native/src/adapters`; `runner.rs` is at 2,799 of 2,800
  tokens.
- Still not seen: `--open` starting a real browser; the page following a real run; Copy on the
  served page; Firefox, Safari, a screen reader.
- `page_text.rs` is mapped by no spec's file plan (a note of `kiro-cli`, no error). One warning of
  `kiro-cli` on the new files of the check was not looked into.

## Next

Group 5, tasks 5.1 to 5.4: Your code; what a planted fault is, with one from this run; faults in
code the platform does not compile (can fail); without tests, nothing is protected.

## Storage

D: had 120.4 GiB free at 18:12, 123.9 GiB at the start. The Linux run took about 3 GiB in the
shared Podman storage, which is not this project's to clean. The shared Cargo target directory
measured 75.3 GiB at 17:00; it is shared with other projects, the share of lesscode was not
separated, and only incremental builds were made. No target directory was added.

Kept: 15 MiB of records of the check under `D:\BuildArtifacts\lesscode\run-dashboard\states-*`,
0.4 MiB of Linux evidence under `…\native-evidence\linux-3872d5a`, and `verification/node_modules`
in the repository (18 MiB, links into the pnpm store, ignored by Git), which the check needs.
Removed: two superseded outputs of the check and the browser profiles of two runs that broke off.
