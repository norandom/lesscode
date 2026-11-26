# Requirements Document

## Project Description (Input)

From the discovery brief ([brief.md](brief.md), 2026-11-26), with the owner's three decisions of that day.

**Who has the problem.** The owner, and after the owner a developer or maintainer who wants to see what
several reworks did to a codebase: through the six lenses (size, structure, state, decisions, behaviour
kept, protection), before and after, with the drawings.

**Current situation.** The progress report of 26 November
([R199](../../../docs/research/2026-11-26/03-progress-report.md)) was put together by hand. An agent
copied numbers from research entries into a page and joined the drawing data with a script written for
that one occasion. In the tool, `step plan --page` and `step check --page` each write one HTML file with
the run's data inside it; the data is not written as a file of its own and the page cannot load one.
`step check --measures <file>` takes handed-in rows for the state and local-complexity lenses.
`step reports` lists finished HTML reports on one page and can leave a project's names out of the page;
it does not set the lenses of several steps side by side. No export to PDF exists.

**What should change.**

1. One command writes a report set: a single self-contained JSON file that holds several steps, possibly
   of several projects. A single step is a set of one.
2. The same command writes an HTML page with that data inside it, drawn by the page inside the binary,
   so that it opens from the file system without a server. The page can also load another report set
   that is dropped on it or picked.
3. The page has a layout for print and a control that saves it as PDF. A command produces the same PDF
   unattended by driving an installed browser headless, and answers by exit code.
4. A project that may be shown by counts only has no name of its own in the set, the page or the PDF.
5. The words of a report are not produced by the tool, which contains no model. An agent following a
   skill, or a person, writes them into the set as notes, and the page shows them as written by hand.

Out of this spec: measuring state or local complexity in the tool; writing the notes; a served page for
a set; a PDF written without a browser. Scope, boundaries, constraints, the proposed prototype gate and
three open questions for the owner are in the brief.

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
