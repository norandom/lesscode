# Validation report: run-dashboard

Written 2026-11-24, about 10:00 Europe/Berlin, for task 8.3. Validated inline by the agent that
implemented the spec; no independent reviewer. Code at `6e08e14`; `1f1325b` after it changed
documentation only and is the commit the Linux run names.

## Validation Report

- DECISION: **GO**, with what stands against it named under "What stands against it".
- MECHANICAL_RESULTS:
  - Tests, Windows: PASS. `cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml`,
    exit 0, 471 of 471, 1 skipped (the replay of a recorded run).
  - Tests, Linux: PASS. `dagger call evidence --source . --revision 1f1325b…`, overall 0, each of
    its 22 steps 0; core suite 470 of 470, 1 skipped; model suite 85 of 85
    (`D:\BuildArtifacts\lesscode\native-evidence\linux-1f1325b`).
  - The pages in a browser: PASS. Playwright with Chromium, 67 of 67, none skipped, at 1,280 and
    at 400 wide (`D:\BuildArtifacts\lesscode\run-dashboard\states-g8-1`).
  - Lints with warnings as errors, format, doctests: PASS on Windows and in the Linux run.
  - Size budgets: PASS (`budgets-g8.json`). `runner.rs` and `page_log.rs` stay within ten tokens of
    their limits and were not touched.
  - `tools\bin\kiro-cli.exe code check --changed 850999a`: PASS, 0 errors.
  - TBD/TODO grep over the files of `native/` that the spec changed: CLEAN.
  - Secrets grep over the same files: CLEAN. One match is the listener's secret, made anew for
    every run (`page_serve.rs`).
  - Smoke boot: PASS. A real run of `step plan --page --open` on the renderer of drift-rdp, 268
    planted faults in 1 h 35 min, exit 0, the report written
    ([R181](../../../docs/research/2026-11-24/02-run-page-a-real-run-watched.md)). On Linux, in a
    container: `step plan --open` on a toy project with a stand-in for `xdg-open`, eleven of
    eleven points (`D:\BuildArtifacts\lesscode\run-dashboard\linux-open\result.txt`; script in
    `docs/experiments/run-dashboard/linux-open/`).
- INTEGRATION:
  - Cross-task contracts: hold. The snapshot (`page_snapshot.rs`) is the one thing the runner's
    listener, `step plan`, `step check`, the report writer and the page share; the list of reports
    reads the same snapshot out of a report.
  - Shared state consistency: holds. One `RunLog` per run; a run without `--page` has none.
  - Boundary audit: no spillover found. The page judges nothing: since task 9.1 it compares two
    numbers to say "unchanged", and since task 8.1 it decides from the counts whether a function
    has begun; both are named in their records as things the tool could say in the data.
- COVERAGE:
  - Requirements mapped: 8 of 8 sections have tasks that are ticked.
  - Coverage gaps:
    - 1.3 (common desktop browsers): shown for Chromium and Edge. Firefox and Safari were not run.
    - 3.5 and 6.2 ("with the reason the tool has"): met by the letter. The reason is the last 400
      characters of a command's output and does not say whether a build or a test failed. On the
      real run this was the first thing the owner could not read.
    - 7.6 (shapes of a type): shown from numbers handed in with `--measures`; no run measures them.
- DESIGN:
  - Architecture drift: two, both on the page and both from the owner's watching on 2026-11-24.
    The stages are a lane above the tree and no longer rows of it; the tree has one row per stage
    that plants faults. The design's table of the page's parts does not name the lane.
  - Dependency direction: no violation. The page modules sit under `adapters/driver`, the commands
    under `step`, and no file under `adapters/driver` names `crate::step` (searched).
  - File Structure Plan vs actual: not compared. The design has a table of components and no plan
    of files; the files are named in the records of the groups (R174 to R181).
- OWNERSHIP: LOCAL
- UPSTREAM_SPEC: N/A
- BLOCKED_TASKS: none. All tasks of groups 1 to 9 are ticked.
- REMEDIATION: none required for GO.

## What stands against it

None of these is a failed check. They are what the checks did not cover.

1. **Nobody has watched the changed page follow a run.** The owner watched the page of R180, said
   nine things, and saw the changes in pictures. The lane, the shut files and the new notices have
   run only on fixed situations and on the data of the two real runs put into the page afterwards.
2. **One reader, who knows the tool.** Whether a developer new to it can say after ten seconds
   whether their code was changed and what the run found was not answered.
3. **The reason of a copy that stops** is unreadable as the tool gives it. Four changes of the
   tool are proposed in R181 and not agreed.
4. **Not run:** Firefox, Safari, a screen reader, macOS (`open`), a real browser started by
   `--open` on Linux.
5. **The reviews of every task were inline**, by the agent that wrote the code.
6. **The design document** does not yet show the lane.

## Evidence kept

| Place | Size | What |
| --- | --- | --- |
| `D:\BuildArtifacts\lesscode\run-dashboard\real-run-8.1` | 0.9 MiB | the two attempts of the real run and their reports |
| `…\run-dashboard\states-g8-1` | 11.6 MiB | the browser check: pictures, walk, record |
| `…\run-dashboard\linux-open` | under 1 MiB | the opener and listener on Linux |
| `…\run-dashboard\budgets-g8.json` | | the size budgets |
| `D:\BuildArtifacts\lesscode\native-evidence\linux-1f1325b` | 0.5 MiB | the Linux run through Dagger |

## Amended 2026-11-24, 10:45

Point 3 of "What stands against it" is answered in the tool
([R183](../../../docs/research/2026-11-24/04-turns-by-memory.md)): a copy that stops says whether its
commands ended before any test ran or a test did not pass, keeps its whole output in a file, tries
once more, and the copies take turns by free memory. Windows 478 of 478, the pages 67 of 67. The
gap named under requirements 3.5 and 6.2 is closed where the configuration has
`runner_evidence_patterns`. The other five points stand.

## Amended 2026-11-24, 11:05

The owner decided that the changed page is watched with the first run of the brownfield test stage
(point 1), and that the next release is 2.2.0 ([notes](../../../docs/releases/2.2.0.md)).
