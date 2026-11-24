# R182: the run page closed: documentation, the opener on Linux, and the validation

- Report date/time and timezone: 2026-11-24, 09:57 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, about 08:45 to 10:00 (the documentation was written while the run of R181 went on; the Linux runs started after 09:45)
- Status: tasks 8.2 and 8.3 completed; the spec's tasks are all ticked
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 8.2 and 8.3; [validation report](../../../.kiro/specs/run-dashboard/validation.md)
- Hypothesis ID and predecessor: [R181](02-run-page-a-real-run-watched.md)
- Type: implementation record

## Decision

The validation report says GO and names six things that stand against calling the page finished.
The first is that nobody has watched the changed page follow a run.

## What was done

- **Documentation (8.2).** `docs/steps.md`: the options `--page`, `--open` and `--page-source`
  with the page's words, `step reports` with what `--without-names-of` leaves out and keeps, and a
  recommendation of diffcore, read first; the owner approved its sentence ("diffcore sentence is
  ok"). `docs/state-refactoring.md`: "The same on a page". The skill `untangle-state`: when to add
  `--open` and when `--page`. The owner read them ("docs are fine"). Humanizer pass: one list lost
  its bold headers.
- **The opener and the listener on Linux (8.3).** In a container of `rust:1.98.1-bookworm`,
  `step plan --open` on a toy project with a script named `xdg-open` first on the path. The script
  writes down the address it is given and reads there what a browser would. Eleven of eleven:
  the opener is started once, on `http://127.0.0.1:<port>/<32 hex digits>/`; the page is served
  with 200; the stream carries a snapshot; the address without the secret answers 404; the report
  is written; and in an image without `xdg-open` the run ends with 0 and says "The browser could
  not be started (xdg-open: No such file or directory (os error 2)). The run goes on; open the
  address above to see it." Script and compose file:
  [linux-open](../../experiments/run-dashboard/linux-open/check.sh). Output:
  `D:BuildArtifactslesscodeun-dashboardlinux-open`.
- **Suites (8.3).** Windows 471 of 471, 1 skipped. Linux through Dagger at `1f1325b`: overall 0,
  core suite 470 of 470, 1 skipped (`D:BuildArtifactslesscode
ative-evidencelinux-1f1325b`).
  Playwright 67 of 67. Lints, format, doctests, size budgets, `kiro-cli code check` pass.
- **Release notes (8.3).** [docs/releases/2.2.0.md](../../releases/2.2.0.md), with the known limits.
  The version and the tag are the owner's.

## Limits

- The stand-in is not a browser. That `xdg-open` starts a real browser on a Linux desktop was not
  run. The stand-in is started by the tool and not waited for; the check sleeps six seconds for it.
- macOS was not run.
- The validation was inline, by the agent that wrote the code.

## Next gate

The brief [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/brief.md), which the
owner placed after this group. The four changes of the tool proposed in R181 wait for the owner.
