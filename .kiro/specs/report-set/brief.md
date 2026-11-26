# Brief: report-set

Written 2026-11-26 from the owner's direction of that day, after reading the progress report
([R199](../../../docs/research/2026-11-26/03-progress-report.md)). A brief, not approved requirements.
Nothing here is built.

## Problem

The owner wants a report like the one of 26 November from the tool itself: several reworks across
projects, each read through the six lenses before and after, with the drawings. That report was put
together by hand. An agent read forty research entries, copied numbers into a page, and joined the
drawing data with a script written for the one occasion. Nothing of that can be repeated by a
command, and the numbers on the page are not tied to the runs they came from.

## Current State

- `step plan --page` and `step check --page` each write one HTML file with the run's data inside it.
  The data is not written as a file of its own, and the page cannot load one.
- `step check --measures <file>` takes a small JSON with rows of the state and local-complexity
  lenses and the shapes of a type's state. The page marks them as handed in
  (`native/src/step/check_handed.rs`).
- `step reports` copies finished HTML reports into a directory and writes one page that lists them,
  with `--without-names-of <project>` for a project that may be shown by counts only
  (`native/src/step/reports.rs`). It lists; it does not set the lenses of several steps side by side.
- The page, its list and its view library are inside the binary
  (`native/page/run.html`, `list.html`, `htm-preact-standalone.umd.js`).
- No export to PDF exists.

The gap: one file that holds a set of reworks, a page that draws it, and a PDF of that page.

## Desired Outcome

- One command writes a **report set**: a single self-contained JSON file that holds several steps,
  possibly of several projects. Per step: the project and what it may show, the verdict, the six
  lenses before and after with the command each number is from, planted faults per function on both
  sides, the data of the drawing of functions and calls, the shapes of a type's state where they were
  handed in, and notes.
- The same command writes an HTML page with that data inside it, so the page opens from the file
  system without a server, as today.
- The page can also load another report set that is dropped on it or picked.
- The page has a layout for print and a control that saves it as PDF. A command produces the same PDF
  unattended by driving a browser that is installed, and answers by exit code.
- A step of one is a set of one. The report of 26 November can be made again from its six reworks.

## Approach

Chosen by the owner on 2026-11-26, each from three options:

1. **One format, a set of reworks.** Not a format per step plus a second one that points at several.
2. **The command writes the JSON and the page; the page also opens a file.** Not a page that is
   always empty until it is handed a file, and not a page that can never load one.
3. **A print layout plus a command that drives an installed browser headless.** Not a PDF written by
   the binary itself, which would draw everything a second time in a PDF library.

Why: the page in the binary already draws the lenses, the faults, the functions and calls and the
shapes for one step. A set is that data several times over with a shared head. No new crate and no
server are needed, and every command still runs unattended.

The words of a report (the answer first, the comparison with earlier reports, the conditions under
which it is wrong) are not produced by the tool, which contains no model. An agent following a skill
writes them into the set as notes, and the page shows them marked as written by hand.

## Scope

- **In**:
  - the report set's format, versioned, with unknown fields refused;
  - a command that builds a set from the outputs of finished runs (verdicts, plans, handed-in
    measures) and from a notes file;
  - the page of a set in the binary: the lenses of all steps side by side, and per step what the
    report of `step check` shows today;
  - loading a set from a file in the page;
  - leaving a project's names out of the set itself, not only out of the page;
  - the print layout, the control in the page, and the command that writes the PDF;
  - documentation in `docs/steps.md` and one example set in the repository.
- **Out**:
  - measuring state or local complexity in the tool; they stay handed in;
  - writing the notes; that is an agent's or a person's work;
  - a chart of what the tests notice per file for a whole plan beyond what the plan's report shows;
  - a served page for a set; the live page stays for the length of a run only;
  - a PDF written without a browser.

## Boundary Candidates

- **The set's format and its reader**: one module that reads and checks a set, used by the command
  and tested without a page.
- **Building a set**: from a run's outputs to one entry of the set, including the removal of names.
- **The page of a set**: drawing from the set alone, whether it was baked in or loaded.
- **Print and PDF**: the print layout in the page, and the command that finds a browser and drives it.

## Out of Boundary

- The live run page, its listener and its six states (`run-dashboard`).
- The verdict, the ledger and the guard (`gradual-refactoring-steps`).
- The protect stage and its receipts (`brownfield-test-stage`, `signed-approvals`), beyond reading
  what they wrote.
- The research programs that count holders and shapes (`lesscode-metrics`, `state-consolidation`).

## Upstream / Downstream

- **Upstream**: `step plan`, `step check` and `step protect result` with the files they write; the
  handed-in measures of `step check --measures`; `deconvolute measure-scope` for the drawing of
  functions and calls.
- **Downstream**: the onboarding spec, which needs examples a developer can open; the examples
  folder asked for in the `lesscode-metrics` brief; a later progress report.

## Existing Spec Touchpoints

- **Extends**: none. `run-dashboard` is complete and stays closed; this spec reuses its page and
  its view library and adds a second page beside `run.html` and `list.html`, or a mode of one of
  them, which the design decides.
- **Adjacent**: `run-dashboard` requirement 7 (the report of a step) and requirement 8 (several
  reports listed). `step reports` and its `--without-names-of` must keep working as they do. Whether
  a set replaces the list of `step reports` later is not decided here.

## Constraints

- Rust only; no Python and no new package manager. The page's check runs through Playwright in
  `verification/`, as for `run-dashboard`.
- The page opens from the file system: no fetch of a neighbouring file, so the data is baked in or
  read from a file the person hands over.
- Commands run unattended and answer by exit code; nothing opens a browser window unless asked.
- A project that may be shown by counts only must have no name of its own in the set, the page or
  the PDF.
- The page stays within what the binary carries today; a new library needs the owner's word and an
  entry in the third-party notices.
- Windows through PowerShell; Linux through containers.

## Prototype gate, proposed

Per [prototype-first](../../steering/prototype-first.md), before requirements are approved for
design. On the six reworks of R199, by hand where no command exists yet:

1. One JSON file holds all six, with the Python project's names left out, and a reader refuses a
   file with an unknown field.
2. A page built from the binary's present page draws the lenses of the six side by side and the
   drawing of functions and calls for at least one, from the file alone, opened from the file system.
3. The same page loads a second set that is handed to it.
4. Chrome or Edge, started headless by a command, writes a PDF of that page on Windows and in the
   Linux container, in which every drawing is present and no table is cut. Not shown counts as
   failed.

What would stop the spec: the drawings missing from a headless PDF, or a set too large to bake into
a page that still opens quickly.

## Open questions for the owner

- Whether the PDF command may require Chrome or Edge, or must also work with Firefox.
- Whether a set records cost per step (run time, model spend), as the `lesscode-metrics` brief asks
  for the page.
- Whether the example set in the repository is the six reworks of R199.
