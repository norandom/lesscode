# Requirements: the report set

Generated 2026-11-26, 18:20 (Europe/Berlin) from the [brief](brief.md) and the owner's three
decisions of that day. Approved by the owner on 2026-11-26, with the six points below.
Evidence: [R199](../../../docs/research/2026-11-26/03-progress-report.md) (a report of six reworks in
three projects, put together by hand), [R178](../../../docs/research/2026-11-23/15-run-page-group-6.md)
(the report of one step on the page, with handed-in measures),
[R179](../../../docs/research/2026-11-23/16-run-page-group-7.md) (several reports listed, names left
out of the page).

## Project Description (Input)

**Who has the problem.** The owner, and after the owner a developer or maintainer who wants to see
what several reworks did to a codebase: through the six lenses, before and after, with the drawings.

**Current situation.** The progress report of 26 November was put together by hand. An agent copied
numbers from research entries into a page and joined the drawing data with a script written for that
one occasion. In the tool, `step plan --page` and `step check --page` each write one HTML file with
the run's data inside it; the data is not written as a file of its own and the page cannot load one.
`step check --measures <file>` takes handed-in rows for the state and local-complexity lenses.
`step reports` lists finished reports on one page and can leave a project's names out of the page; it
does not set the lenses of several steps side by side. No export to PDF exists.

**What should change.** One command writes a single self-contained file that holds several steps of
several projects, and a page with that data inside it. The page can also load another such file. The
page can be saved as PDF, by a control in the page and unattended by a command.

## Points proposed for approval with this document

1. **A step may enter a set from handed-in numbers alone.** Four of the six reworks of R199 have no
   verdict of `step check` (the two Python subsystems, the clipboard lease, the batch progress as
   first recorded). Without this point the report of 26 November cannot be made again. Such a step is
   marked as not judged by the tool, on the page and in the PDF (Requirement 2).
2. **Cost is run time only.** A set holds how long a step's run took where the run recorded it. It
   holds no model spend, because these runs call no model. This answers the brief's second open
   question with the smaller option.
3. **The PDF command names the browsers it can drive, and Firefox need not be among them.** Which
   browsers is for the design. The requirement is that the command says which it looks for and
   refuses plainly when none is there (Requirement 7).
4. **The example set in the repository is the six reworks of R199**, with the Python project shown
   by counts only.
5. **`step reports` stays as it is.** A set does not replace the list. Whether it should later is
   not decided here.
6. **Notes are plain text.** The page shows them as text and interprets no markup in them.

## Introduction

Lesscode judges one step at a time and writes one report per run. A person who wants to show what a
series of reworks did, across files and across projects, has no single thing to open or to send. The
report of 26 November showed that such a report is wanted and what belongs in it: for every rework
the six lenses before and after, the planted faults the tests notice, the drawing of functions and
calls, the shapes of a value's state, and a few sentences that say what it means.

This feature gives lesscode a file that holds all of that for a set of reworks, a page that draws it,
and a PDF of that page.

Terms. A *step* is one rework with a state before and a state after, as in `step check`. A *lens* is
one of the six measures of a rework named in the
[lesscode-metrics brief](../lesscode-metrics/brief.md). A *report set*, or *set*, is one file that
holds one or more steps with their lenses, faults, drawings and notes. The *set page* is the page
that shows a set. A number is *handed in* when it was measured outside the tool and given to it in a
file. A *note* is text a person or an agent wrote. A project is *shown by counts only* when none of
its file, function or type names may appear.

## Boundary Context

- **In scope**: the report set as a file; the command that makes a set from the outputs of finished
  runs, handed-in numbers and notes; leaving a project's names out of a set; the set page with a set
  inside it; loading another set in the page; the page's layout for print and its control for saving
  as PDF; a command that writes the PDF unattended; documentation and one example set.
- **Out of scope**: what the commands measure or judge; measuring state or local complexity in the
  tool; writing notes; any model call; a server for the set page; a PDF made without a browser;
  editing a set in the page; uploading or sharing.
- **Adjacent expectations**: `step plan`, `step check`, `step protect result` and `step reports`
  behave as today, with the same output, exit codes and receipts. The set reads what those commands
  wrote and runs nothing of the project. The live run page and its states are not changed. The
  skill that guides an agent through a rework may tell it to write notes and build a set.

## Requirements

### Requirement 1: One file that holds a set of reworks
**Objective:** As someone who wants to show what several reworks did, I want all of it in one file,
so that I can keep it, send it and have it drawn again later.

#### Acceptance Criteria
1. The report set shall be one file that holds one or more steps and needs no other file to be
   shown.
2. The report set shall hold for each step the project, the scope, the commits before and after,
   the outcome, and for each of the six lenses the measures before and after with the direction
   that was expected.
3. The report set shall hold for each number where it is from: the command that produced it and the
   version of the tool, or that it was handed in and by which command it was counted.
4. Where a lens of a step was not measured, the report set shall say so for that lens and shall not
   leave the lens out.
5. Where planted faults were run for a step, the report set shall hold per function how many were
   noticed, not noticed and written off, before and after.
6. Where the functions and calls of a step's scope were measured, the report set shall hold what is
   needed to draw them before and after.
7. Where the shapes of a type were handed in for a step, the report set shall hold which can be
   written, which the code reaches and which the tests see.
8. Where a run recorded how long it took, the report set shall hold that time for the step.
9. The report set shall name its format's version, the version of the tool that wrote it and the
   inputs it was made from.
10. If a file that is given as a report set has a newer format version, a missing required part or
    a part the format does not know, then the tool shall refuse it, name the part and change
    nothing.

### Requirement 2: Making a set from finished runs
**Objective:** As a developer, I want one command to put the results of my runs into a set, so that
no number is copied by hand.

#### Acceptance Criteria
1. When the person names the outputs of one or more finished runs, the tool shall write one report
   set that holds one step for each, in the order they were named.
2. When the person names only one finished run, the tool shall write a set of one step.
3. Where the person hands in numbers for a step that the tool did not measure, the tool shall put
   them into that step and mark them as handed in.
4. Where a step is given by handed-in numbers alone, the tool shall take it into the set and mark
   the step as not judged by the tool.
5. Where the person names a file of notes, the tool shall put each note into the set at the place
   the file names: the set as a whole, or one step.
6. The tool shall run none of the project's commands and change no file of any project while it
   makes a set.
7. When the same inputs are given again, the tool shall write a set with the same content.
8. If an input cannot be read, belongs to a run that did not end, or contradicts another input
   about the same step, then the tool shall write no set, name the input and the reason, and end
   with an exit code that is not 0.
9. If the place for the set already holds a file, then the tool shall refuse to overwrite it unless
   the person asked for that.
10. The tool shall take a step that was refused, or in which a lens got worse, into the set in the
    same way as any other.

### Requirement 3: A project shown by counts only
**Objective:** As the owner of a project whose code may not be shown, I want its names out of
everything that leaves my machine, so that I can still show what a rework did to it.

#### Acceptance Criteria
1. Where the person names a project to be shown by counts only, the tool shall write no file,
   function or type name of that project into the report set, and shall put a number in its place.
2. Where a project is shown by counts only, the report set shall hold no source text, no command
   and no path of that project.
3. The report set shall keep the counts, the lenses, the faults per function and the drawing of
   functions and calls for such a project, with the same number standing for the same name
   throughout the set.
4. The report set shall say for each project whether it is shown by counts only.
5. If a note contains a name that was left out of the set, then the tool shall refuse to make the
   set and shall name the note and the word.
   *Amended 2026-11-27 by the owner, after the prototype gate
   ([R200](../../../docs/research/2026-11-27/01-report-set-prototype-gate.md)):* where the name is
   also an everyday word that the page uses in its own text, the tool shall make the set and shall
   warn, naming the note and the word. As first written, the criterion refused the note "The first
   run of the tests took longer." because the project has a function named `run`.
6. While a set says that a project is shown by counts only, the set page and its PDF shall show for
   that project only what the set holds.

### Requirement 4: The page of a set
**Objective:** As someone who reads a report, I want one page that shows the whole set, so that I
see first what changed across the reworks and then each rework by itself.

#### Acceptance Criteria
1. When the tool writes a report set, it shall also write the set page as one file with that set
   inside it, where the person asked for the page.
2. The set page shall display from disk without the tool and without any request to another
   machine.
3. The set page shall show first the steps of the set side by side, with for each step the project,
   the outcome and each lens before and after.
4. The set page shall show for each step what the report of that step shows today: the lenses with
   the expected direction and whether each moved that way, the planted faults per function on both
   sides, the drawing of functions and calls before and after to one scale, and the shapes of a
   type's state.
5. The set page shall mark every lens that was not measured as not measured, every number that was
   handed in as handed in, and every step that the tool did not judge as not judged.
6. The set page shall show a step that was refused, or in which a lens got worse, as prominently as
   one that was accepted.
7. The set page shall show notes as text, marked as written by hand, and shall interpret nothing in
   a note or in any other text of a set as markup or as a program.
8. The set page shall name the tool's version, when the set was made and the inputs it was made
   from.
9. The set page shall be readable in a light and in a dark colour scheme and at the width of a
   phone, and every control on it shall be reachable and usable by keyboard.
10. Where the person did not ask for the page, the tool shall write the set and no page.

### Requirement 5: Loading another set in the page
**Objective:** As someone who was sent a report set, I want to open it in a page I already have, so
that I need not run the tool to read it.

#### Acceptance Criteria
1. When the person picks a report set in the set page or drops one on it, the set page shall show
   that set in place of the one it showed.
2. The set page shall say at all times which set it shows: the one it was written with, or the name
   of the file that was loaded.
3. If a file that is loaded is not a report set the page can show, then the set page shall keep
   showing the set it showed, and shall say what is wrong with the file in words that name the part.
4. The set page shall send no part of a loaded set to another machine.
5. When a loaded set says that a project is shown by counts only, the set page shall show it as it
   would show the same set written into it.
6. When the page is opened again from disk, the set page shall show the set it was written with.

### Requirement 6: Print and saving as PDF from the page
**Objective:** As someone who has to hand a report to people who will not open a page, I want a PDF
of it, so that the report can be attached, filed and printed.

#### Acceptance Criteria
1. When the set page is printed, it shall lay itself out for paper: every lens, table, drawing and
   note that the page shows is on the paper, and no control is.
2. When the set page is printed, it shall print what is folded shut on the screen as if it were
   open.
3. When the set page is printed, it shall not cut a drawing or a row of a table across two sheets.
4. When the set page is printed, it shall print in the light colour scheme, whatever the screen
   shows.
5. The set page shall have one control that starts saving the page as PDF through the browser.
6. The printed page shall name on its first sheet the set, when it was made and the version of the
   tool that wrote it.
7. Where a project is shown by counts only, the printed page shall show no more of it than the
   screen does.

### Requirement 7: The PDF by command
**Objective:** As a developer, I want the PDF written by a command, so that a pipeline can produce
the report without a person at a browser.

#### Acceptance Criteria
1. When the person asks for the PDF of a report set, the tool shall write one PDF file at the place
   named and end with exit code 0.
2. The PDF shall hold what the printed set page holds, with every drawing in it.
3. The tool shall open no window, ask nothing and need no network while it writes the PDF.
4. The tool shall install nothing. It shall use a browser that is already on the machine, and its
   documentation shall name the browsers it can use and how to point it at one.
5. If no browser it can use is found, then the tool shall write no PDF, say which browsers it
   looked for and where, and end with an exit code that is not 0 and differs from that of a refused
   set.
6. If the browser does not finish within the time the tool allows, or ends without a PDF, then the
   tool shall write no PDF, say so, leave no browser running, and end with an exit code that is not
   0.
7. If the place for the PDF already holds a file, then the tool shall refuse to overwrite it unless
   the person asked for that.
8. The tool shall write the PDF on Windows and on Linux.

### Requirement 8: Unattended, and nothing else changes
**Objective:** As someone who runs lesscode in a pipeline, I want the new commands to behave like
the others, so that I can script them and trust that the old ones are as they were.

#### Acceptance Criteria
1. The commands of this feature shall take everything they need as arguments and files, shall ask
   nothing, and shall answer by exit code.
2. The commands of this feature shall print where they wrote each file.
3. Where a run is started without asking for a set, the tool shall write no set and behave as it
   does today.
4. The reports of `step plan` and `step check` and the list of `step reports` shall be the same
   with this feature as without it.
5. The commands of this feature shall write only at the places the person named.

### Requirement 9: Documentation and an example
**Objective:** As a developer who meets this for the first time, I want to see a set and how to
make one, so that I can make my own.

#### Acceptance Criteria
1. The documentation of the step commands shall describe how to make a set, how to leave a
   project's names out, how to write notes, how to load a set in the page and how to get the PDF,
   with one command for each.
2. The documentation shall describe the format of a report set well enough that a set can be
   written by another program.
3. The repository shall hold one example set and the page made from it.
4. The tool's own checks shall make the page and the PDF from the example set, so that the example
   cannot fall behind the format.
