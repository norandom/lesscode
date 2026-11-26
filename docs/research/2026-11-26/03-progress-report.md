# R199: A progress report for 18 to 26 November, with the drawings as one page; file-level maps of the code show no difference worth reporting

- Report date/time and timezone: 2026-11-26, about 17:30 Europe/Berlin
- Execution interval: about 16:10 to 17:30 Europe/Berlin
- Status: completed
- Spec: none. The owner's follow-up, a report of this kind made by the tool, has no spec yet (see the
  last section)
- Hypothesis ID and predecessor: [R198](02-display-path-rework.md), the last result the report covers;
  the earlier reports are [state-boundary](../../reports/state-boundary-deconvolution-report.md) and
  [driver feasibility](../../reports/agnostic-driver-feasibility.md)
- Type: retrospective (a synthesis of R158 to R198; nothing was built, tested or planted for it)

## Decision

The report is [docs/reports/progress-2026-11-26.md](../../reports/progress-2026-11-26.md), with its
drawings in [one page](../../reports/figures/progress-2026-11-26.html). It makes no new measurement.
Every number in it is copied from an entry it names.

The owner asked for two things after reading it (2026-11-26, about 17:25): the entry in the log, and
that a report of this kind be made by the tool itself, with the drawings' data in one self-contained
JSON file that the page inside the binary loads, and an export to PDF. The second is a feature and
needs a spec. It is not started here.

## Hypothesis and falsification

None registered. Two things were tried and are recorded as tried:

1. Before-and-after maps of drift-rdp made with Floorplan (`haydenschmitty/floorplan` at `94f81b4`),
   on the owner's suggestion. Its scripts were run directly, without installing the plugin, on two
   exported copies of `src-tauri` at `949127d` and `ba2963b`. Result: 84 code files and 185
   file-to-file imports before, 83 and 184 after; one part and its one import gone. The owner's
   reading: these plans do not allow a real difference to be reported. Dropped, and the copies
   removed. A map of files and imports cannot show a rework whose gain is in the state of a value.
2. Understand Anything (`Egonex-AI/Understand-Anything`), also the owner's suggestion, was read and
   not run. It needs a plugin install and a run of several agents over the whole code, twice, and its
   graph is written by a model, where the report's drawing is made from calls the tool measured.

## Frozen inputs and method

- lesscode at `eaeb996`.
- The research index and the entries R134, R138, R139, R159 to R162, R164, R165 to R173, R177, R184
  to R190, R192 to R198, read on 2026-11-26.
- Drawing of functions and calls, two Python subsystems: the data of the page "Two Bowls" of
  2026-11-23, with every file and function name replaced by a number, because that project is shown
  by counts only.
- Drawing of functions and calls, renderer of drift-rdp: `deconvolute measure-scope` output of R198,
  `D:\BuildArtifacts\lesscode\renderer-rework\before\scope.json` and `after\scope-final.json`.
- The script that joined these into
  [progress-2026-11-26-drawings.json](../../reports/figures/progress-2026-11-26-drawings.json), the
  page's template and two screenshots: `D:\BuildArtifacts\lesscode\progress-report\`. The script is
  JavaScript, written for this one report; it is kept as evidence and is not part of the tool.

## Evidence and results

- The page was rendered once through Playwright (the one in `verification/`), at 1,200 wide in the
  light theme and 400 wide in the dark theme: no horizontal overflow, no script error.
- The two series colours of its charts pass the colour-blindness check of the chart guidance in both
  themes.
- The page was also published as a private page of the owner
  (`https://claude.ai/artifact/P9mEU65RdF22e9QbK8WoeZ`).

## Limits and interpretation

- A report can only be as right as its entries. Two figures were chosen among versions: 204 faults in
  code built on Windows is R164's count, and R177's recount (66 of 268 not built there, not 64) is
  given beside it, not folded in.
- The row for `shared_frame.rs` in the stage's table is R189's, on the stage's own base (8 missed).
  R171 and R172 raised the same file from 34 to 38 noticed of 42 on another local branch, and R173's
  rework starts from that branch. The report does not join the two.
- The drawing shows a call found by name. It does not show that the call happens at run time.
- The part of the report on who takes this up next is a proposal.

## Tool/spec consequence and next gate

The owner wants this kind of report from the tool. What exists: `step check --page` draws functions
and calls before and after and the shapes of a type's state, the state lens only from a file handed in
with `--measures` (R178); `step reports` lists several reports on one page (R179). What does not
exist: one file that holds the lenses of several steps across projects, a page that loads it, and an
export to PDF. Next gate: discovery and requirements for that, with a prototype on the six reworks of
this report as its data.

## COBESY check

- Outcome first: the report's first section is its answer, and the comparison with the earlier
  reports names what was claimed and did not hold.
- Replay: every number names its entry; the drawing data is committed.
- Disproof: three "we are wrong if" lines close the report.
- Negative results: the dropped maps are recorded here with their counts.
