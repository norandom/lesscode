# Prototype gate: the report set

Registered 2026-11-26, about 18:45 Europe/Berlin, before the prototype exists. Steering:
[prototype-first](../../steering/prototype-first.md). The gate decides whether the provisional
[design](design.md) may be put to the owner for approval and go on to tasks.

Status: **not-run**.

This feature adds no compression mechanism, so the steering's reduction targets do not apply. The
gate is kept because three assumptions of the design can fail on real inputs, and each would change
the design.

## The assumptions at risk

1. A browser started headless prints the page with every drawing in it. The drawings are made by
   the page's script after it loads; a browser that prints before the script has run gives a PDF
   with empty figures.
2. The reports the tool already writes hold everything a set needs, so that a set can be made from
   them without running anything again.
3. One page can show a set that was written into it and a set that is handed to it, opened from the
   file system, where a page may not read a neighbouring file.

## What is built for the gate

- A set written by hand in the format of the design, from the six reworks of
  [R199](../../../docs/research/2026-11-26/03-progress-report.md): the data inside the report page of
  R198 (`D:\BuildArtifacts\lesscode\renderer-rework\check.html`), and five steps from handed-in
  numbers. The two Python subsystems carry numbers in place of names.
- A research program in Rust that reads the set, refuses what the design says it refuses, and writes
  the page with the set inside. It lives outside `native/src` and is not part of the tool.
- A copy of `native/page/run.html` with the set mode, the control that loads a file, and the print
  layout. The page in the repository is not changed for the gate.
- A start script in PowerShell that runs the browser headless and the checks. No Python.

## Expectations

| # | Question | Expectation | Rejected when |
| --- | --- | --- | --- |
| G1 | Does a report page hold what a step of a set needs? | From the data inside R198's page alone the step has all six lenses, the faults per function on both sides, the drawing of functions and calls on both sides, the commits and the outcome | any of these has to be taken from another file |
| G2 | Is the set one file that the reader checks? | The set of six steps is one JSON file. The research program reads it, and refuses three changed copies: one with an unknown field, one with a newer format version, one without a required part, each with a message that names the part | a changed copy is read, or a refusal does not name the part |
| G3 | Are the names out? | In the set, in the page made from it and in the PDF, no file or function name of the two Python subsystems occurs; checked by searching all three for every name of the original data | one occurrence |
| G4 | Does one page show a set from inside and a set handed to it? | Opened from the file system, the page shows the six steps side by side and the drawing for the drift-rdp renderer. A second set, picked through the page's control, replaces the first; a file that is not a set leaves the first in place and shows a message. The page names no address on another machine | another result in any of the four, or an address |
| G5 | Does a headless browser print it whole? | Microsoft Edge or Chrome on Windows, and Chromium in a Linux container, each started by a command without a window, write a PDF in at most 60 seconds. Every drawing of the page is in the PDF as shapes with text, found by extracting the PDF's text for the function count of each drawing; no control of the page is in it; the folded parts are in it | no PDF, a drawing missing, a control present, a folded part missing, or over 60 seconds |
| G6 | Does the browser stay behind? | After each PDF, and after a run that is stopped at its time limit, no process of the browser started by the command is left | one is left |
| G7 | Is it small enough? | The page with the set of six inside is at most 400 KiB and shows its first screen within 2 seconds of opening on this machine | larger or slower |
| G8 | How does it look on paper? | Not judged by me. The PDF is shown to the owner | |

For G5 I do not know whether the extracted text of a PDF keeps the labels of an SVG drawing. If it
does not, the drawings are checked by rendering the PDF's pages to pictures and comparing the
figures' areas with a blank figure, and the record says so.

## What the gate does not show

- That the page of the tool itself, changed in place, still passes the checks of the run page. That
  is a task after the gate, with the existing browser checks as its guard.
- That a set of fifty steps is readable.
- Firefox.

## Stop conditions

- G5 fails on both systems: the PDF by command is not feasible as designed. The owner decides
  between the control in the page alone and another way to make the PDF.
- G1 fails: the set cannot be made from report pages, and the design's input side is revised.

## Budget

One working session. No new crate in `native`. Disposable output under
`D:\BuildArtifacts\lesscode\report-set\`, removed at the end except the evidence named in the
research entry.
