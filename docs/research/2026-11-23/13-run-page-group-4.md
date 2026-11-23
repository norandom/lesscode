# R176: the run page, group 4: the page of the design in the binary, its six states, and a walk by keyboard

- Report date/time and timezone: 2026-11-23, 18:12 Europe/Berlin (from the clock)
- Execution interval: 2026-11-23, about 16:58 to 18:12 (the start is the time of the session's checkpoint before task 4.1, the end is the end of the Linux run)
- Status: completed on Windows and on Linux; one browser engine seen
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), tasks 4.1 to 4.4
- Hypothesis ID and predecessor: [R175](12-run-page-group-3.md)
- Type: implementation record with its deviations; nothing was preregistered beyond the tasks' own "done when"

## Decision

Group 4 is done. The page of the prototype and its view library are inside the binary, and the
stand-in of group 3 is gone. For thirteen fixed situations the page gives itself the state the
design names, at a width of 1,280 and of 400, with no failure of its code; a drawing that is made to
fail gives the state "cannot read" and not a blank page. Every control is reached and used by
keyboard. All of this was seen in one browser engine, the one of Chromium and Edge, on Windows.

Not seen: Firefox and Safari; a screen reader; the page following a real run in a real browser.

## What exists

| File | What |
| --- | --- |
| `native/page/run.html` | the page: style, code, and the places the tool fills |
| `native/page/htm-preact-standalone.umd.js` | the view library, htm 3.1.1 with preact, 13,296 bytes, unchanged; `htm-LICENSE.txt` and `preact-LICENSE.txt` beside it |
| `native/bundled.json` | names the library, its source address and its SHA-256 |
| `native/src/adapters/driver/page_text.rs` | takes both files in with `include_str!`, puts the library in its place once, fills one of two markers |
| `scripts/license-inventory.ps1` | reads `bundled.json`, checks the hash, writes a row and a section into `LICENSES.md` and `THIRD_PARTY_NOTICES.txt` |
| `native/tests/run_page_text.rs` | 5 tests: the page holds its style, code and library; names no address on another machine; is under 150 KiB without data; its markers are filled; the library's hash is the one in the notices |
| `native/page/states.spec.mjs` | the check in a browser: 29 cases |
| `verification/` | the pinned package for that check (`@playwright/test` 1.64.0, pnpm lock) and its configuration |

## The walk by keyboard (task 4.4)

Run by the check, in Chromium 156 without a window, on the page of a run that has ended. The lines
are the check's own record, `walk.txt`; "focus solid 2px" is the outline the browser draws on the
element that has the focus.

Tab from the top of the page, one press per line:

| # | Stop | State |
| --- | --- | --- |
| 1 | button "Follow the stage that is going on" | pressed |
| 2 | button "Show the functions of every file" | |
| 3 | button "Hide the functions of every file" | |
| 4 | button "Show noticed faults" | pressed |
| 5 | button "Show unnoticed faults" | pressed |
| 6 | button "Show faults that did not build" | pressed |
| 7 | button "Sort by unnoticed faults" | pressed |
| 8 | button "Copy the place of the copies" | |
| 9 | button "What a planted fault is" | |
| 10 | stage "Copy the project and run your tests on the unchanged code" | |
| 11 | stage "Plant faults 12 / 12" | |
| 12 | file "draw.rs 0 noticed · 4 unnoticed" | expanded |
| 13 | file "app.rs 5 noticed · 3 unnoticed" | expanded |
| 14 | stage "Check the copies and write the plan" | |
| 15 | tab "Now" | selected |
| 16 | the pane, named by its tab | |

Every stop had the outline and a name. After the sixteenth the focus leaves the page.

What worked:

- **Follow**: Space switches it off, Enter switches it on again.
- **A file of the tree**: Enter hides its functions, Space shows them; the other file keeps its own.
- **A stage of the tree**: Enter makes it the current one and switches Follow off.
- **The tabs**: they are one stop. The right arrow goes to "Your code" and shows it; End goes to
  "Result", Home to "Now"; the left arrow on the first tab goes round to the last. Tab leaves the
  tabs for the pane, Shift+Tab comes back to the tab that is shown.
- **The tool for the explanation**: Enter shows the tab "Planted faults".
- **Marks**: each is a sign with a name, not a colour alone: ✓ ended, ✗ unprotected, ! some faults
  were not noticed, ■ was going on when the tool stopped.
- **Reduced motion**: with the setting on, the moving bar of a stage that has nothing to count
  stands still.

What did not work, or was not seen:

- **Copy** answered "Could not copy. The place is in the tab Your code." A page opened from disk in
  a browser without a window is not allowed to write to the clipboard. Whether it copies on the
  page the tool serves was not seen. Before this task the button said nothing when it failed.
- **A function's row** is not a stop. A long name is cut, and the whole name shows on hover only.
  By keyboard the whole name can be read in the tab "Unprotected", and there only for unprotected
  functions.
- **The nine tools** are nine stops. The usual way for a toolbar is one stop and the arrows; I did
  not build that.
- **No screen reader was run.** What a screen reader is given was written down (`aria.txt`): a
  banner with the state, a toolbar "View", the navigation "Stages and files", a region with a list
  of tabs and a pane. What a person hears with it is not known.

What I changed in the page for this task: the tabs as one stop with the arrows; the pane can take
the focus and is named by its tab; marks are images with a name; the state "quiet" no longer
changes a status text once a second, which a screen reader would have read out each time; Copy says
whether it copied; the outline is drawn inside the tree, the tabs and the pane, where it was cut
off; column heads of the tables are marked as such.

## Deviations from the tasks, each for a stated reason

1. **The page is not the prototype's page unchanged** (4.1). What the design dropped is gone: the
   data file beside the page, the configuration value, a clock that could be set. The two markers
   of group 3 stay.
2. **The version of preact inside the library is not stated by its build** (4.1). htm 3.1.1 asks
   for `^10.2.0`. The licence text kept is that of preact 10.2.0, and the notice says so.
3. **The notice also enters the inventories of the model crate** (4.1), because that crate depends
   on the core.
4. **One address is in the page**: the name of the SVG format, inside the library. It is not
   fetched. The test allows exactly that one.
5. **The fixed situations are made by the check, not by the tool** (4.2). The tool cannot write
   damaged data or data of the wrong kind. The check fills the two markers the way
   `page_text.rs` does, in a few lines of its own.
6. **"Waiting" shows a time from the page's own clock** (4.2): how long the page has been open.
   Requirement 3.8 asks for the run's records; there is none yet in that state, and the design asks
   for the time.
7. **A stream that ends before any data leaves the state "waiting"**, with the sentence "The tool
   does not answer" (4.2). The design has six states and none for this.
8. **"Collapse all" keeps the files and hides their functions** (4.2). Before, it hid the files too.
9. **The second browser has the same engine as the first** (4.3): Edge 158 and Chromium 156. The
   task asks for a browser other than Edge and is met by its words. Requirement 1.3 names the
   common desktop browsers, and Firefox and Safari were not seen.
10. **The check brings a Node package into the repository** (after 4.3). The design says no package
    manager and no JavaScript build are added to the project. The owner pointed to Playwright as
    the tool for this and to its use in `QOpenBB`; the layout follows that one. The tool and its
    build do not use it.

## Faults of mine, found by the checks or by the owner

- **A hand-written client for the browser's debugging port, in PowerShell.** I wrote the check of
  4.2 that way, 250 lines, and extended it for 4.3 to set a width of 400 over the port. The owner
  asked whether this was not a job for Playwright. It was: the same 27 cases went from about 90
  seconds to about 10, with a real width of 400, and the script is deleted.
- **I proposed Firefox in a container through Podman.** The owner called it senseless, rightly: the
  task did not ask for a second engine, and a container is no place for a desktop browser.
- **Vivaldi 8.3 hung** when I tried it as the second browser. I stopped the run after ten minutes
  and did not find the reason. It ran with a profile of its own; it may have shown a window on the
  owner's desktop. No process of that run was left.
- The first Chromium run broke off on the largest page because the script asked a page that was
  still loading. That was the script's fault, not the page's.
- Data of the wrong kind made the page of 4.1 fail in drawing (a stage that was `null`). The check
  of 4.2 found it; lists of records now leave out what is not a record.
- A commit message with a path in it was refused by the shell's guard, and nothing ran. The message
  goes through a file now.

## Evidence

| Check | Result |
| --- | --- |
| Core suite on Windows, `cargo nextest`, at `3872d5a` | 456 of 456 (451 before the group) |
| Lints with warnings as errors; format check; doctests | clean, at `3f96ae7`; no Rust file changed after it |
| Size budgets, at `3f96ae7` | pass |
| `kiro-cli code check --changed` | passes at `3f96ae7` (two warnings on `live`, within the limits) and at `f103d1e` (one warning on the new files, not looked into) |
| The page in Chromium 156 through Playwright, at `3872d5a` | 29 of 29: 13 situations at 1,280 and at 400, the drawing made to fail, the walk, reduced motion |
| The same check against the page of `3f96ae7` | 12 of 27 fail, on six situations at both widths: the check can fail |
| The page in Edge 158 and in Chromium 156 with the PowerShell script, at `2f407b0` | 27 of 27 each; 400 set over the port |
| A report written by a real run of the toy project, opened in Edge without a window, at `3f96ae7` | state "ended", no failure, 44,548 bytes with its data |
| Linux through Dagger at `3872d5a` | overall 0; 455 of 455 |

Records: `D:\BuildArtifacts\lesscode\run-dashboard\states-playwright-20261123-180357` (the last
run, with `walk.txt`, `aria.txt`, `states.json` and a picture of every page);
`states-playwright-before-20261123-175916`; the PowerShell runs under `states-*`. Linux:
`D:\BuildArtifacts\lesscode\native-evidence\linux-3872d5a`.

The check ran on this Windows machine only. It needs Node, the installed package and a Chromium
from the Playwright store; none of that is in the Linux container.

## Limits

- One engine. Firefox and Safari were not seen.
- No screen reader, and nobody but me has used the page.
- The page was never seen following a real run in a real browser. The check opens pages from disk;
  the stream is covered by the unit tests of group 3 with a stand-in for the browser. `--open` has
  still not been started by me.
- The fixed situations are mine. They are shaped like what the tool writes, taken from one real
  report, and are not a run.
- The wording of every state is mine and has not been read by the owner.
- The review of each task was mine, inline; no second reader.

## Next

Group 5: what the page explains, from task 5.1 on.
