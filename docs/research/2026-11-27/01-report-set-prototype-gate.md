# R200: A set of six reworks is one checked file, one page shows it, and a headless browser prints it whole on Windows and Linux; the design is wrong at three points

- Report date/time and timezone: 2026-11-27, about 09:35 Europe/Berlin (from the clock)
- Execution interval: 2026-11-27, about 09:00 to 09:30 Europe/Berlin; the last full run of the start
  script went from 09:27:55 to 09:28:51
- Status: completed
- Spec: [report-set](../../../.kiro/specs/report-set/design.md), its
  [prototype gate](../../../.kiro/specs/report-set/prototype-gate.md), registered 2026-11-26 before
  anything was built. Requirements approved; design provisional
- Hypothesis ID and predecessor: G1 to G8 of the gate; predecessor
  [R199](../2026-11-26/03-progress-report.md)
- Type: preregistered

## Decision

The gate is assessed as **proceed**. Seven expectations were checked by a program or a browser and
hold; the eighth, how the PDF looks on paper, is the owner's to judge. The design may be put to the
owner once it is corrected at the three points below, which the prototype showed to be wrong as
written. No production task exists yet, and none is permitted before the design is approved.

What each kind of evidence shows, kept apart:

- **Making a set** from the tool's report page and from step files: shown, by a research program
  that shares no code with the tool.
- **The page** showing a set from inside and a set handed to it: shown in a browser, on a copy of
  the tool's page.
- **The PDF by command**: shown with Edge on Windows and Chromium in a Linux container.
- **Names kept out** of a set, its page and its PDF: shown for this one set, with a control.
- **The tool itself**: nothing is shown. No line of `native/src` was run or changed.

## Hypothesis and falsification

The eight expectations and their rejection conditions are in the gate file, unchanged since
2026-11-26. Three assumptions were at risk: a headless browser prints the drawings; a report page
holds everything a set needs; one page can show a set from inside and one handed to it when opened
from the file system.

## Frozen inputs and method

- lesscode at `7533374`. The page: a copy of `native/page/run.html` (sha256 `651f1d41…` before the
  change, `12b548fa…` with the set mode). The repository's page is unchanged.
- Step 6, the report page of R198: `D:\BuildArtifacts\lesscode\renderer-rework\check.html`, sha256
  `c1d1717b…`, written by lesscode-core 2.1.0.
- Steps 1 to 5: five step files written by hand in the design's format, with the numbers of R199's
  table. The drawings of steps 1 and 2 come from the owner's private page "Two Bowls" of 2026-11-23,
  which holds the names; the drawing and the shapes of step 5 come from the fixture
  `native/page/shared-frame.json`.
- The research program: Rust, one file, `serde_json` and `sha2`, outside the repository. The start
  script: PowerShell. No Python.
- The page checks: Playwright 1.64.0 of `verification/`, Chromium, on Windows.
- Windows: Microsoft Edge 158.0.4370.33, started by the research program.
- Linux: Chromium 156.0.8078.108 in a container (`debian:bookworm-slim`, with `poppler-utils`), run
  through Docker Compose with no network. `pdftotext` of that container read the PDFs of both
  systems, because Windows has no such tool installed.
- Storage before the run: 133 GiB free on D:, about 6 GiB of lesscode build caches.

Everything is at `D:\BuildArtifacts\lesscode\report-set\`: `gate.ps1` runs it, `out\gate.json` holds
what was seen.

## Evidence and results

| # | Expectation | Result |
| --- | --- | --- |
| G1 | The report page alone gives a step its six lenses, faults per function on both sides, the drawing, the commits and the outcome | **Holds.** All six lenses, four measured and two marked not measured; 27 functions, 21 of them with faults on both sides; 7 and 6 files, 83 and 84 calls; both commits; "Accepted". Not in the page: a name for the project and a title for the step (point 1 below) |
| G2 | One JSON file; three changed copies refused, each naming the part | **Holds.** 76,935 bytes. Refused with exit code 3: an unknown part (`colour`), a newer format (`schema`), no steps (`steps`); also a step whose run is not a run (`steps[2].run`) and an unknown part inside a step (`steps[4].colour`). The same inputs gave the same bytes twice |
| G3 | No file or function name of the two Python subsystems in the set, the page or the PDF | **Holds, but not by the check as registered** (point 2 below). 180 names were searched as whole words. None stands in a place that comes from the project's data. Control: the same set made without taking names out is caught with 177 names |
| G4 | The page shows the six steps and the renderer's drawing; a second set replaces the first; a file that is not a set leaves the first and shows a message; no address | **Holds.** Six rows, six sections, eight drawings; the renderer's has 70 dots with 83 calls and 70 with 84. A picked file and a dropped file replace the set. Four wrong files (other kind, not JSON, newer format, no steps) each leave it and name the part. The page asked for nothing but its own file. The one address-like text in it is the name of the SVG format inside the bundled library, which the existing page has too |
| G5 | A headless browser writes the PDF within 60 seconds, with every drawing as shapes with text, no control, the folded part open | **Holds on both systems.** Windows: 1.5 s, 14 pages. Linux: 1.3 s, 16 pages. All eight captions are in both texts. The 41 file names along the rims, which are text inside the drawings, are all there. No control's text is there. The folded table is there (26 and 25 of its 27 function names found unbroken). I looked at the rendered pages: the rims, dots and lines are drawn |
| G6 | No browser process is left, also after a stop at the time limit | **Holds on both systems.** A page that never finishes loading was stopped at 5 s: no PDF, exit code 2, no process, no profile directory left |
| G7 | At most 400 KiB; first screen within 2 seconds | **Holds.** 163 KiB. Drawn 116 to 121 ms after opening, five openings |
| G8 | How it looks on paper | Not judged by me. The PDFs: `out\set-windows.pdf` and `out\set-linux.pdf` |

On G5's open question: the extracted text of a PDF does keep the labels of an SVG drawing, so the
fallback of comparing pictures was not needed. On Linux three captions break onto a second line
beside their neighbour, and the text tool puts the last word behind the neighbour's caption; those
three were matched up to their last word.

### Three points where the design is wrong as written

1. **A report page does not say which project it is about, nor what the step is called.** It holds
   the path of a working copy (`…/lesscode-renderer-rework`) and a branch. The design gives every
   step a project and a title and does not say where they come from for a page. The prototype
   refuses a page without them and takes them from a `labels` part it added to the notes file.
2. **The rule that keeps names out cannot be a plain search.** Five of the 180 names are everyday
   words that stand in the page's own text; one of them is also the name of a part of every step
   in the set. The design's last check, "refuse when a name still stands in the written set",
   would refuse every set of this project. Its rule for notes has the same fault: the note "The
   first run of the tests took longer." was refused. The prototype searches only the texts inside
   the steps of that project, which works for the set. For the page and the PDF the check here
   compared each with the same page made without the project's steps, and then read the
   occurrences that remained: all are sentences of the page ("handed in, not measured by this
   run") or names of the other project. That comparison is a check for a gate, not a rule a tool
   can apply to a note.
3. **The page's part for one run's result cannot be reused for a step that was stated.** It draws
   functions and calls only when the run names the commit it compared with, and it words the
   outcome by exit code. Four of the five stated steps name no commit, and none has an exit code.
   The prototype's set section
   calls the smaller parts itself (tiles, the table of lenses, faults per function, the drawing,
   the shapes) and writes the outcome line its own way.

### Smaller findings

- A step file can carry a drawing only through receipts of `measure-scope`. For the two Python
  steps there were none; the research program made receipts from the Two Bowls data.
- The sentence "handed in, not measured by this run" is wrong inside a set, which is not a run.
- The names of the input files stand in the head of the set and of the page. A file named after a
  subsystem would give a name away; nothing checks that.
- R199's table calls one measure "across files" (12 to 9, 10 to 10, 25 to 22). The tool's name for
  that number is "pairs of files with a call between them". Calls across files, as the drawings
  count them, are 34 to 28, 80 to 65 and 41 to 41. An amendment to R199 says so.
- The fixture for `SharedFrame` has 12 calls after the rework; R199's table says 11. The set shows
  the fixture's 12. Which is right was not looked into.
- On paper: every step starts a page, which leaves short pages, and the mark of a fold is printed.

## Limits and interpretation

Done by hand or by a stand-in, so not shown for the tool:

- Five of the six steps were written by hand. Only step 6 went from a file the tool wrote to the
  set without a person in between.
- Names were taken out by the research program's own code. The tool's `page_names::leave_out` was
  not run, so nothing shows that its numbers agree across two steps of one project.
- The set was read by the research program's reader, written by hand over plain JSON, not by
  types that refuse unknown parts.
- On Windows the browser was started and stopped by the research program, which ends it with
  `taskkill` for the whole tree. On Linux a shell did it with `timeout`, which stops the process
  group. The design's `print` in Rust was not run on Linux.
- Only Edge on Windows. Chrome is not installed here.
- The type's name and the texts of its shapes are not taken out with the names.
- One set of six steps. Fifty steps, a path with a blank in it, Firefox and macOS were not tried.

What transfers: the format, the page's set mode and its print rules, and the two command lines for
the browser. What is specific: the numbers of steps 1 to 5 are copies from R199.

## Tool/spec consequence and next gate

Justified: one JSON file per set, the set mode as a third mode of the run page, the PDF by an
installed browser started headless with the address ending in `#print`. Revise in the design before
approval: where a page's project name and title come from; the rule for names in notes and the last
check on a written set; the set section built from the page's parts. Next: the corrected design to
the owner, then tasks. The first task after approval is the one the gate could not show: the tool's
own page, changed in place, still passing the checks of the run page.

Kept at `D:\BuildArtifacts\lesscode\report-set\` (11 MiB): the research program's source, the page,
the checks, the start script, the step files and `out\` with both PDFs. Removed: the container
image, the build output, and every input that holds names of the Python project (the copy of Two
Bowls, the list of names, four receipts), so `gate.ps1` needs them made again before it runs.

## COBESY check

- Outcome first: the title and the decision say what holds and what does not.
- Replay: `gate.ps1` and `out\gate.json`; versions and hashes above.
- Disproof: the rejection conditions were registered a day before the run. G3's check was changed
  during the run, and the entry says how and why.
- Negative results: the three points and the smaller findings.
- Facts and interpretation: "proceed" is my assessment; the table is what was seen.
