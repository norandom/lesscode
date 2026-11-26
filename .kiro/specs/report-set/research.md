# Research & Design Decisions: report-set

## Summary

- **Feature**: `report-set`
- **Discovery Scope**: Extension (light discovery), read in the main context without subagents, by
  the owner's rule.
- **Key Findings**:
  - A report page already holds its whole run as JSON, and the tool can already take it out again
    (`page_text::data_of`). A step of a set is therefore a run that exists; nothing has to be
    measured or run again.
  - The run's report type (`page_report::Report`) already has the six lenses with `measured` and
    `handed_in`, the faults per function on both sides, the functions and calls on both sides, and
    the shapes of a type. The set needs a head around several of these and nothing new inside them.
  - The page draws functions and calls as SVG, not on a canvas. SVG prints as shapes and text, which
    is what a PDF needs.

## Research Log

### What a written report holds
- **Context**: Requirement 2 asks for a set made from finished runs without copying numbers.
- **Sources Consulted**: `native/src/adapters/driver/page_text.rs`, `page_snapshot.rs`,
  `page_report.rs`; `native/src/step/reports.rs`.
- **Findings**:
  - `report(snapshot)` writes the page with the snapshot at the marker `/*DATA*/null`; `data_of`
    reads it back from the fixed text `const data = ` to the end of that line, in any version of the
    page.
  - `Snapshot` holds the tool's name and version, the command, start and end, outcome, exit code,
    seconds, the subject (repository, branch, commit, what it is compared with), tests, stages,
    faults and the optional `Report`.
  - `step reports` already reads pages this way and builds its list from them.
- **Implications**: the inputs of a set are report pages. The set stores each run as the page held
  it, cut down to what the set page shows.

### Leaving names out
- **Context**: Requirement 3 wants the names out of the set, not only out of a page.
- **Sources Consulted**: `native/src/adapters/driver/page_names.rs` (`leave_out`), `page_list.rs`.
- **Findings**: `leave_out(run)` collects the names of a run, withholds commands, written files,
  shapes and the example fault's source line, and rewrites names to numbers. It does not give the
  names back.
- **Implications**: the set uses the same function per run, before the run enters the set. It needs
  the names back once, to check the notes (3.5) and to keep one number per name across the steps of
  one project (3.3).

### How the page gets its data
- **Context**: Requirements 4 and 5: a set inside the page, and a set handed to it.
- **Sources Consulted**: `native/page/run.html`, `page_text.rs`.
- **Findings**: the page takes two values from markers, `events` and `data`. Its parts are functions
  in one script: `Tiles`, `Lenses`, `PerFunction`, `Bowl`, `Graph`, `Shapes`, `Result`, and `Run`
  around them. A page opened from disk cannot fetch a neighbouring file; it can read a file the
  person picks or drops.
- **Implications**: a third marker for a set, and one more part that shows a set by calling the
  existing parts once per step.

### Printing by a browser without a window
- **Context**: Requirement 7.
- **Sources Consulted**: the command line of Chromium-based browsers as used in the run page's
  prototype gate (Microsoft Edge headless, R174 to R176); `native/src/adapters/driver/process.rs`
  and `interrupt.rs` for running a program under a time limit; `verification/` for the browser the
  page checks use.
- **Findings**: Chrome, Chromium and Edge take `--headless`, `--print-to-pdf=<file>` and
  `--no-pdf-header-footer`, and print a `file:` address. Whether they wait for a page's script
  before printing is not something I have seen on this page. Firefox has no such option on its
  command line.
- **Implications**: the three Chromium-based browsers are the ones the command names. The waiting
  question is expectation G5 of the gate. Not searched on the web; the gate is the evidence.

### Skills consulted
- `cobesy` for the order of the set page (the answer first, then each step).
- `artifact-design` and `dataviz` guidance as used for the page of R199: text in text colours, a
  legend for two series, light and dark both designed. They inform the print layout's light scheme.

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| --- | --- | --- | --- | --- |
| A second page `set.html` | A new page with its own copy of the drawing parts | `run.html` untouched | Two copies of every part to keep alike | Rejected |
| Shared parts in a third file | Parts moved out of `run.html` into a file both pages take in | No duplication | Rewrites a finished page for no gain to its own users | Rejected |
| A set mode of `run.html` | One more marker and one more part in the existing page | Smallest change; the parts are reused as they are | The page grows; its existing checks must still pass | Selected |

## Design Decisions

### Decision: a step of a set is a run as a report page holds it
- **Context**: Requirements 1 and 2.
- **Alternatives Considered**:
  1. A new, smaller shape per step, filled from verdicts and plans.
  2. The run as the page holds it, cut down.
- **Selected Approach**: 2. The set's reader checks the head strictly and each step's run with the
  types the tool already has.
- **Rationale**: no second description of the same numbers; `step reports` already trusts this data.
- **Trade-offs**: a set inherits the snapshot's shape and its version.
- **Follow-up**: G1 of the gate.

### Decision: one input kind for a step the tool did not run
- **Context**: Point 1 of the requirements; four of R199's six reworks have no report page.
- **Alternatives Considered**:
  1. Accept verdict files, plan files and scope receipts as further inputs.
  2. One step file in which a person or an agent states the step, with handed-in lenses, shapes,
     faults per function, and optionally the two scope receipts of `deconvolute measure-scope`.
- **Selected Approach**: 2.
- **Rationale**: one more reader, not four. The scope receipt is taken because the drawing cannot
  be typed by hand.
- **Trade-offs**: a step that the tool once judged, and that has no page, enters as "not judged in
  this set". The step file may name the verdict as where its outcome is from.
- **Follow-up**: open decision D1 in the design.

### Decision: "when the set was made" is the end of its newest input
- **Context**: 2.7 asks for the same set from the same inputs; 4.8 asks the page to say when the set
  was made.
- **Selected Approach**: the set holds no reading of the clock. It holds, per input, the file's name
  and hash and the end of its run, and `as_of`, the newest of those ends.
- **Rationale**: both criteria hold.
- **Trade-offs**: a set made today from last week's runs is dated last week. That is what it shows.

### Decision: the PDF command drives a browser and adds no crate
- **Context**: the owner's third decision; Requirement 7.
- **Alternatives Considered**: a PDF library in Rust; a browser-automation crate; the browser's own
  command line.
- **Selected Approach**: the browser's command line, run through the driver's existing process
  module under a time limit.
- **Rationale**: no dependency, and the page is drawn once.
- **Trade-offs**: only Chromium-based browsers; depends on G5.

## Synthesis

- **Generalization**: "one step" and "several steps" are one case; a single report is a set of one
  (2.2). Written-in and handed-over sets are one case for the page: both arrive as the same value.
- **Build vs adopt**: adopted from the tool itself: the report types, `data_of`, `leave_out`, the
  process module, the page's parts, the Playwright checks. Built: the set's head, the step file's
  reader, the set part of the page, the print layout, the browser lookup.
- **Simplification**: no second page, no shared-parts file, no served set page, no clock in the set,
  no input kinds beyond pages and step files.

## Risks & Mitigations

- The browser prints before the drawings exist: G5; the page marks itself as drawn, and the command
  gives the browser a time budget. If it fails, the command is cut and the control in the page stays.
- `run.html` grows and a finished page changes: the run page's own browser checks run unchanged in
  the first task that touches it.
- A name survives in a place `leave_out` does not know: the set is searched for every collected
  name before it is written, and is refused on a hit.
- A large set makes a heavy page: G7; no limit is designed in until a real set needs one.

## References

- [brief](brief.md), [requirements](requirements.md), [prototype gate](prototype-gate.md)
- [run-dashboard design](../run-dashboard/design.md), "One file"
- [R178](../../../docs/research/2026-11-23/15-run-page-group-6.md),
  [R179](../../../docs/research/2026-11-23/16-run-page-group-7.md),
  [R199](../../../docs/research/2026-11-26/03-progress-report.md)
