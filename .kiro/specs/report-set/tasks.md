# Implementation Plan: the report set

Generated 2026-11-27 (Europe/Berlin). **Approved by the owner on 2026-11-27** ("proceed").
Basis: [requirements](requirements.md) (approved 2026-11-26; criterion 3.5 amended by the owner on
2026-11-27), [design](design.md) (approved by the owner on 2026-11-27, with the decisions D1 to D6,
and amended the same day by D7 to D9: the dashboard code is a module of its own),
[prototype gate](prototype-gate.md) (run on 2026-11-27, assessed as proceed,
[R200](../../../docs/research/2026-11-27/01-report-set-prototype-gate.md)).
Task-graph sanity review: inline (no subagent, by the owner's rule), verdict PASS, done again after
group 1 was added, with three notes: the fixed set of task 2.1 exists before the reader of task 3.1
and must be read by it unchanged; the example of task 8.1 cannot hold the names of the Python
project, so its inputs for that project are committed with numbers already in place; whether
`lib.rs` can take the one line of task 1.1 under its size ceiling is not known before that task.

Rules for every task group (owner rules of 2026-11-07, 2026-11-10, 2026-11-17 and 2026-11-23):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format
  check and lints before every push. No Python. The page and its library are files that the Rust
  build takes in as text; the page's checks are Playwright in `verification/`, on the host.
- A run that does not ask for a set must behave exactly as before. Every group ends with the
  existing suites and the existing browser checks of the run page and of the list passing
  unchanged, on Windows and through Dagger on Linux.
- Before a group starts, D: free space and lesscode's build caches are measured against the
  storage limits. Linux-only runs use containers; nothing runs in a WSL distribution.
- Each group ends with the budgets check passing for every new file at the default ceilings, a
  dated research entry where a result or a deviation exists, and `.kiro/task_N_handover.md`,
  starting with `task_95_handover.md`. A file over budget is split, not exempted.
- No name of a project that is shown by counts only enters the repository: not in a fixture, not in
  the example, not in a research entry.
- Tasks marked "can fail" carry something the gate did not show. A failure is recorded and goes
  back to the design before the group is closed.
- What is reported about a task is what ran and was seen. Times come from the clock.

The prototype of the gate is at `D:\BuildArtifacts\lesscode\report-set\`. It is evidence and a
source to read, not code to copy unread: its reader, its removal of names and its browser start are
stand-ins for parts the tool has to own.

Where the dashboard code lives (D7 to D9): everything that shows a run or a set is in the module
`dashboard`, one directory of its own in the crate. Core measures, plants faults, runs and judges,
and holds no page logic. The command files stay under `step`: `step` decides, the dashboard shows.
Every task from group 2 on writes its dashboard code in that directory and nowhere else.

## Tasks

- [x] 1. The dashboard as a module of its own
- [x] 1.1 The ten files of the run page move into the dashboard directory (can fail)
  - Done 2026-11-27, `2db0f8d`. The "can fail" point fired as foreseen: `lib.rs` came to 198
    logical units against 197 (tokens fit, 4,112 of 4,127). The owner chose one more
    registration-only line for it (ceiling 197 to 198, recorded in its budget entry). Evidence in
    `D:\BuildArtifacts\lesscode\dashboard-move\`: before and after, 552 tests pass; the 67 browser
    checks pass with their fixture reports written (none skipped); the 8 pages `step reports`
    writes from three real reports are the same bytes from the binaries before and after (the
    list's moment masked); budgets 101 within, 7 disposed, 0 exceeded; Dagger on Linux, "All
    native checks passed" in 707 s.
  - A pure move: the files that build the snapshot and its log, the report types, the page's text,
    the list, the removal of names, the subject, the listener, the opener and the watch leave the
    driver's directory and become the dashboard module, without their prefix. Their content
    changes only in the paths they import and in the paths to the page's own files.
  - Every file that uses them changes its import and nothing else: the command files under `step`,
    the runner and the command runner in the driver, and the tests, which keep their names.
  - The budgets file lists the moved files under their new paths with the same ceilings.
  - Can fail: the crate's top file is at its size ceiling and needs one more line. If the line
    does not fit, the task stops and the owner decides between one more line for that file and
    another place for the directory. No ceiling is raised without that decision.
  - Done when no file of the run page is left in the driver's directory, the whole Rust suite and
    the browser checks of the run page and of the list pass unchanged on Windows and through
    Dagger on Linux, the budgets check passes, and a report page written before the move and one
    written after it from the same fixture run are the same bytes.
  - _Boundary: dashboard, adapters::driver, step_
  - _Requirements: 8.3, 8.4_
- [x] 1.2 The border between core and dashboard, kept by a test
  - Done 2026-11-27: `native/tests/dashboard_border.rs`, three tests; the fragment scan of
    `driver_contract.rs` reads `src/dashboard` too; notes in `structure.md` and the `run-dashboard`
    design. Trials, each undone: an import of the report types in `compare.rs` failed the first
    test naming `compare.rs`; an import of `step` in `dashboard/list.rs` failed the second; an
    empty `adapters/driver/page_trial.rs` failed the third. 555 tests pass, fmt, Clippy and
    budgets pass. The commands under `step` may name the dashboard; that is the design, not a
    crossing.
  - A test reads the source tree and fails when a file of core other than the runner and the
    command runner names the dashboard, when a file of the dashboard names `step`, or when a file
    with the old prefix appears in the driver's directory again.
  - The existing scan of the adapters for forbidden fragments covers the dashboard directory too,
    so that the move loosens no rule the moved files were under.
  - The module's head says in a few sentences what belongs in it and what does not. The structure
    steering and the `run-dashboard` spec each get a dated note with the new paths; earlier
    records are not rewritten.
  - Done when the test passes on the moved tree, and fails in a trial where one core file is made
    to import the dashboard's report types and where one dashboard file is made to import `step`;
    both trials are undone.
  - _Boundary: dashboard_
  - _Requirements: 8.4_

- [ ] 2. The set page
- [ ] 2.1 The page shows a set that is written into it (can fail)
  - The tool's own page gets a third place where data can stand, beside the run's stream and the
    run itself. With a set there, the page shows the set; with none, it shows what it shows today.
  - The set is shown in this order: which set, as of when, the tool's version and the inputs; the
    notes of the set; the steps side by side with each lens before and after and the outcome; then
    one section per step with its outcome, its notes, the lenses with the expected direction and
    whether each moved that way, the planted faults per function, the drawing of functions and
    calls to one scale, and the shapes of a type's state.
  - The section of a step is built from the page's existing smaller parts. The part for one run's
    result is not used and not changed. A stated step without a commit shows its drawing.
  - A lens that was not measured, a number that was handed in and a step that was not judged each
    carry their mark; inside a set no text says "this run". A refused step stands in the same
    block as an accepted one. A project shown by counts only is called by its place in the set.
  - Every text of a set goes onto the page as text; a note with markup in it shows the characters.
  - A fixed set of six steps for the checks is taken from the gate's evidence. It holds no name of
    the Python project.
  - Can fail: the gate changed a copy of the page, not the page. Done when the new browser checks
    show six rows, six sections and eight drawings from the fixed set with no request leaving the
    page, and the existing checks of the run page and of the list pass unchanged.
  - _Boundary: set mode of run.html_
  - _Requirements: 3.6, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 8.3, 8.4_
- [ ] 2.2 Loading another set in the page
  - One control picks a file, and a file can be dropped on the page. A set that passes the page's
    check replaces the set that is shown, and the page names the file it came from.
  - The page's check follows the reader's rules for the head: the kind, a format version the page
    can read, the required parts, and for each step its number, project, title and run.
  - A file that fails leaves the shown set in place and shows a message that names the part.
  - Nothing is kept between openings, and nothing is sent anywhere.
  - Done when the browser checks show: a picked set and a dropped set each replace the first; four
    wrong files (another kind, not JSON, a newer format, no steps) each leave it and name the part;
    a loaded set with a counts-only project looks as it does when written in; reloading shows the
    first set; no request left the page.
  - _Boundary: set mode of run.html_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_
- [ ] 2.3 The layout for paper and the control that saves as PDF
  - Printed, the page uses the light scheme, hides every control, opens every folded part, keeps a
    drawing and a table row whole, and names on its first sheet the set, when it was made and the
    tool's version.
  - The same state is reached three ways: the page's control, the browser's own print, and an
    address that ends in the print mark. The page marks itself as drawn once every step is drawn.
  - The owner's remarks on the gate's PDFs, if any are given before this task, are taken up here.
  - Done when the browser checks show, with print emulated: no control visible, no folded part
    shut, light colours under a dark screen; and the control starts the browser's print.
  - _Boundary: set mode of run.html_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
- [ ] 2.4 Both colour schemes, the width of a phone, and the keyboard
  - The set page is checked light and dark, at 1,280 and at 400 wide, without the page scrolling
    sideways; every control is reached and used by keyboard alone.
  - Done when those browser checks pass and one picture per scheme and width is in the record.
  - _Requirements: 4.9_

- [ ] 3. The set as a file
- [ ] 3.1 (P) What a set is, and the one reader that checks it
  - The set's own types and its format version 1: the head with the tool, the inputs with their
    hashes, the projects with whether each is shown by counts only, the notes, and the steps; each
    step with its project, title, whether it was judged, where a stated outcome is from, its notes
    and its run, cut down to what a set keeps.
  - The reader refuses with the part named: a newer or missing format version, a missing part of
    the head, a part the format does not know, a step whose run is not a run, a step without one of
    the six lenses, a step that names no project of the set, two steps with one number.
  - Done when tests show each refusal with its part, a set that is read and written again is the
    same set, and the fixed set of task 2.1 is read unchanged. If the reader refuses that fixed
    set, the fixed set and the page are corrected in this task.
  - _Boundary: dashboard::set_
  - _Depends: 2.1_
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10_
- [ ] 3.2 Building a set from inputs that are already read
  - Steps stand in the order the inputs were named and are numbered from 1; projects are numbered
    in the order they first appear; a set of one step is a set.
  - Every step has all six lenses, each marked measured or not. A step that was refused, or in
    which a lens got worse, enters like any other.
  - The set reads no clock: "as of" is the end of the newest input's run. Maps are written in a
    fixed order.
  - Done when tests show the order, the six lenses on a step that states two, a refused step in
    the set, and the same bytes from the same inputs twice.
  - _Boundary: dashboard::set_
  - _Requirements: 1.4, 2.1, 2.2, 2.7, 2.10_

- [ ] 4. The inputs of a set
- [ ] 4.1 A report page as an input, with its label
  - A file is a report page when the tool can take a run out of it, in any version of the page.
    The run enters cut down: no stages, no copies, no commands of the project, no example fault.
  - A page does not say which project it is about nor what the step is called. Both come from a
    label in the notes file, found by the page's file name.
  - Refused, each with the file named: a page whose run did not end; a page without a label; a
    label for a file that is not an input; two labels for one file; a label for a step file.
  - Done when tests on fixture pages show a step with the label's project and title and with the
    run's lenses, faults, drawing, commits and outcome, and each refusal with its message.
  - _Boundary: set_inputs_
  - _Requirements: 1.2, 2.1, 2.8_
- [ ] 4.2 A step file: a rework stated by hand
  - A step file states project, title, commits, outcome and where the outcome is from, rows for any
    of the six lenses, shapes of a type, and faults per function. Its rows have the form the
    existing file of handed-in measures has; that file's format does not change.
  - Everything from a step file is marked handed in, and the step is marked not judged in this set.
  - Where the file names two receipts of the scope measure, the drawing and the structure lens are
    taken from them and are not marked handed in. A function a receipt counts and does not name is
    still drawn.
  - A part the format does not know is refused, with the file and the part named.
  - Done when tests show a stated step with handed-in marks, one with a drawing from receipts, and
    the refusals.
  - _Boundary: set_inputs_
  - _Requirements: 1.3, 1.6, 1.7, 2.3, 2.4_
- [ ] 4.3 The notes file, and inputs that contradict each other
  - A note is about the set or about one step, by its number, and may say who wrote it. It is
    placed there as plain text.
  - Refused: a note about a step that is not there; an input that cannot be read or is neither a
    page nor a step file nor a notes file; two inputs with the same project and commits and
    different outcomes, with both files and what differs named.
  - Done when tests show notes at the set and at a step, and each refusal with its message.
  - _Boundary: set_inputs_
  - _Requirements: 2.5, 2.8_

- [ ] 5. A project shown by counts only
- [ ] 5.1 The removal of names gives back what it took (can fail)
  - The existing removal of names from one run returns the names it replaced with their numbers.
    The list of reports ignores the result and writes what it wrote before.
  - Can fail: the gate did not run this part of the tool. If its numbers cannot be made to agree
    across two runs of one project, that goes back to the design.
  - Done when a test shows the returned names for a fixture run, and the tests and browser checks
    of the list of reports pass unchanged.
  - _Boundary: dashboard::names_
  - _Requirements: 3.1, 3.2, 8.4_
- [ ] 5.2 Names out of every step of a project, one number per name
  - For a project named as counts only, every file, function and type name is taken out of each of
    its steps; the same name gets the same number in every step of the set. No source text, command
    or path of the project stays.
  - Counts, lenses, faults per function and the drawing are kept. The set says for each project
    whether it is shown by counts only, and such a project has no name in the set.
  - Done when a test on two steps of one fixture project shows one number per name across both,
    the same counts and calls as before, and no path or command of the project in the set.
  - _Boundary: dashboard::set_names_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
- [ ] 5.3 The last search for names: refused when distinctive, warned when an everyday word
  - Before a set is written, the names taken out of a project are searched as whole words in the
    texts of that project's own steps, in the notes of the set, and in the file names of the
    inputs. The names of the format's parts and the steps of other projects are not searched.
  - A name is an everyday word when it stands as a whole word in the page's own text. Such a name
    is a warning with the place and the word, and the set is written. Any other name is a refusal
    that names the place and the word.
  - Done when tests with a fixture project that has a function named `run` show: a note with a
    distinctive name refused with the word; the note "The first run of the tests took longer."
    warned and the set written; a name inside a longer word not found; a distinctive name left in
    a text of the project's step refused.
  - _Boundary: dashboard::set_names_
  - _Requirements: 3.5_

- [ ] 6. The command that makes a set
- [ ] 6.1 The page written with a set inside it
  - The tool writes the page with a set at the place of task 2.1, so that a set cannot end the
    script it stands in. The same set gives the same page.
  - Done when a test shows that a set with markup and a script's end tag in a note is written and
    read back from the page unchanged, and that pages of a run are written as before.
  - _Boundary: dashboard::text_
  - _Depends: 2.1, 3.1_
  - _Requirements: 4.1, 8.4_
- [ ] 6.2 `step set`: check everything, then write
  - The command takes the inputs in order, the place for the set, and optionally a notes file, the
    wish for a page with or without a place, the projects to show by counts only, and the wish to
    overwrite. It asks nothing.
  - Every input is read and checked before anything is written. The set is written first, then the
    page where it was asked for; each written file is printed. Without the wish for a page, no page
    is written.
  - Exit code 0 when written; 3 for an invalid input, a refusal, a file in the way without the wish
    to overwrite, or a failure to write. A refusal names the file and the part and changes nothing.
  - The command runs none of the project's commands and writes only at the places named.
  - Done when tests through the binary show: a set of three steps from two pages and a step file; a
    set of one; an unfinished run, two contradicting inputs and a file in the way each end with 3
    and write nothing; no file of the fixture project differs afterwards; no page without the wish.
  - _Boundary: step set_
  - _Depends: 3.2, 4.3_
  - _Requirements: 2.1, 2.2, 2.6, 2.8, 2.9, 4.1, 4.10, 8.1, 8.2, 8.5_
- [ ] 6.3 Counts only, through the command
  - Integration of the command with the removal of names and the last search.
  - Done when tests through the binary show: with a project named as counts only, none of its
    names is in the set or in the page, and counts and calls are as before; a note with a
    distinctive name ends with 3 and names the word; a note with an everyday word ends with 0,
    writes the set and prints the warning on standard error.
  - _Depends: 5.3, 6.2_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 7. The PDF by command
- [ ] 7.1 (P) Finding a browser that is already there
  - The order: the path the person gave; the environment variable; the names of Chrome, Chromium
    and Edge on the search path; on Windows their usual places. Nothing is downloaded or installed.
  - When none is found, the answer lists what was looked for and where.
  - Done when tests show the order with stand-in files and the list when nothing is there.
  - _Boundary: dashboard::pdf_
  - _Requirements: 7.4, 7.5_
- [ ] 7.2 Driving the browser under a time limit (can fail)
  - The browser is started without a window, with a profile directory of its own, on the address
    of the page with the print mark, and asked to print to a file beside the place of the PDF.
  - At the time limit the browser and whatever it started are stopped. The profile directory is
    removed. The PDF is moved into place only when it is whole.
  - Can fail: the gate stopped the browser's processes through a system command on Windows and
    through a shell on Linux. The tool has to do both itself, with what it already has for running
    a program under a time limit.
  - Done when tests with stand-in programs show, on Windows and in a Linux container: a PDF that
    appears is moved into place; a program that writes nothing leaves no file; one that never ends,
    with a child of its own, is stopped at the limit and neither process is left; no profile
    directory is left in any case.
  - _Boundary: dashboard::pdf_
  - _Requirements: 7.3, 7.6_
- [ ] 7.3 `step set-pdf`
  - The command takes a set, the place for the PDF, and optionally a browser, a time limit and the
    wish to overwrite. It reads and checks the set, writes the page into a directory of its own,
    prints it, and removes the directory.
  - Exit code 0 with the PDF written and its place printed; 3 for a set that is refused or a file
    in the way; 2 when no browser was found or no PDF was made in time, with no file written.
  - Done when tests through the binary with a stand-in browser show each of these exit codes with
    its message, and that nothing is written except at the place named.
  - _Boundary: step set-pdf_
  - _Depends: 6.1, 7.2_
  - _Requirements: 7.1, 7.5, 7.6, 7.7, 8.1, 8.2, 8.5_
- [ ] 7.4 A real browser, on Windows and on Linux
  - With a browser that is found, the PDF of a set is made and its text holds the count of
    functions of every drawing and the names along the rims, no control's text, and the folded
    table. Where no browser is found the check is skipped and says why.
  - On Windows with the installed Edge. On Linux once, in a container with a Chromium and without
    a network, through Docker Compose; the container's definition comes from the gate's evidence
    and is removed after the run.
  - Done when both runs are recorded in a research entry with versions, times, page counts and
    what the text of each PDF holds, and no container, network or image of this task is left.
  - _Depends: 7.3_
  - _Requirements: 7.2, 7.3, 7.8_

- [ ] 8. The example and the documentation
- [ ] 8.1 The example: the six reworks of the progress report, kept true by the checks
  - The repository holds the inputs, the set and the page made from them: one report page of the
    tool, five step files, a notes file with the labels. The Python project is shown by counts
    only; its inputs are committed with numbers already in place of names, and none of its names
    is in any committed file.
  - Open from R200, settled here with their sources: which of the two Python subsystems is 7.5%
    and which 9.8% smaller, and whether the `SharedFrame` rework has 11 or 12 calls afterwards.
  - Done when a check builds the set again from the committed inputs and it equals the committed
    set, and the check of task 7.4 makes the example's PDF.
  - _Depends: 6.3, 7.4_
  - _Requirements: 9.3, 9.4_
- [ ] 8.2 How to make a set, and the three formats
  - The documentation of the step commands gets a section with one command each for: making a set,
    leaving a project's names out, writing notes and labels, loading a set in the page, and getting
    the PDF. It names the browsers the PDF command can use and how to point it at one.
  - The formats of the set, the step file and the notes file are described well enough that
    another program can write them, with the rule for names and its limit said plainly.
  - The texts get the pass for plain wording before they are pushed.
  - Done when every command in the section was run as written against the example and gave the
    result the text says.
  - _Depends: 8.1_
  - _Requirements: 7.4, 9.1, 9.2_

- [ ] 9. The gate's expectations, on the tool itself
  - The start script of the gate is run again with the tool's commands in place of the research
    program, on the example: one file that the reader checks; the changed copies refused; no name
    of the Python project in the set, the page or either PDF, searched with the names read afresh
    from the owner's private page and removed again afterwards; the page from inside and with a
    set handed to it; the PDF on both systems with every drawing; no process left; the page's size
    and the time to its first screen.
  - The printed example is read by a person for drawings or table rows cut across two sheets; no
    check measures a page break.
  - All existing suites and browser checks pass unchanged on Windows and through Dagger on Linux; a
    run that asks for no set writes none.
  - Done when a research entry sets each expectation of the gate beside what the tool showed, and
    names what differs from R200.
  - _Depends: 8.1_
  - _Requirements: 1.1, 3.6, 4.2, 5.4, 6.3, 6.7, 7.2, 7.8, 8.3, 8.4, 9.4_
