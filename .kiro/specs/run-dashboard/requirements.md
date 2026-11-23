# Requirements: the run page

Generated 2026-11-23, 14:56 (Europe/Berlin) from the [brief](brief.md). Not approved yet.
Evidence: [R164](../../../docs/research/2026-11-23/01-renderer-baseline-faults.md) (a run of 1 hour
43 minutes with nothing to look at; a copy that dropped out unseen),
[R173](../../../docs/research/2026-11-23/10-shared-frame-rework.md) (the six lenses before and after).

## Points proposed for approval with this document

1. The page is one file and needs no network. Fonts, style and script are inside it.
2. The page is opened only when the person asks for it, and never in a pipeline.
3. The page explains and shows. It never judges: exit codes and receipts stay the only judgement.
4. Names of the project's files and functions appear on the page. Source text does not, unless
   the person asks for it.
5. The gallery (Requirement 8) is part of this feature but may be delivered after the rest.

## Introduction

A developer who runs lesscode on code they care about cannot see what happens to it. A run of
planted faults is silent for a long time and ends in a receipt. Whether the checkout is touched,
what a planted fault is, and why a result without tests means nothing is written in the
documentation and nowhere in front of the person while the tool works.

This feature gives lesscode one page that is part of the binary. A command writes it, and opens it
if asked. During a run it shows the steps, how far each is, and in plain words what the step does
and does not do. After the run it is the report.

Terms: the *run page* is that page. A *run* is one invocation of a command that executes the
project's own commands (`step plan`, `step pin`, `step check`, `deconvolute run`). A *stage* is one
part of a run that a person would name, such as "copy the project" or "plant faults". A *copy* is
a working copy of the project that the tool makes outside the repository. A *planted fault* is one
small deliberate change the tool makes to a copy to see whether the tests notice. A *lens* is one of
the six measures of a rework named in the [lesscode-metrics brief](../lesscode-metrics/brief.md).
A *report* is the run page after its run has ended.

## Boundary Context

- **In scope**: the run page as part of the binary; writing and opening it; what it shows during a
  run; what it says about the safety of the person's code; its explanation of planted faults; its
  warnings where tests are missing; the report after a run; a list of several reports.
- **Out of scope**: what the commands measure or judge; new measurements; the state programs
  becoming released commands; a server that outlives the run; editing code from the page; accounts,
  sharing or uploading.
- **Adjacent expectations**: the commands' exit codes, receipts and terminal output stay as they
  are. A run without the page behaves exactly as today. The skill `untangle-state` may tell an agent
  to open the page for the person.

## Requirements

### Requirement 1: One page inside the binary
**Objective:** As a developer, I want the page to come with the tool, so that I need no second
install and no network to see a run.

#### Acceptance Criteria
1. The tool shall contain the run page and everything it needs to display, and shall write it as
   one file.
2. The run page shall display without any request to another machine.
3. The run page shall display in the current versions of the common desktop browsers when opened
   from disk.
4. The run page shall be readable in a light and in a dark colour scheme and at the width of a
   phone.
5. Where a run is started without asking for the page, the tool shall write no page and behave as
   it does today.

### Requirement 2: Asking for the page
**Objective:** As a developer, I want to decide whether a page is written and whether it opens, so
that the tool never surprises me or a pipeline.

#### Acceptance Criteria
1. Where the person asks for the page, the tool shall write it to the place they name, or beside
   the run's other outputs, and shall print where it is.
2. Where the person asks for the page to be opened, the tool shall open it in the default browser
   once, at the start of the run.
3. If the page cannot be opened, then the tool shall say so, print where the page is and go on
   with the run.
4. The tool shall never open a browser unless asked in that invocation.
5. The tool shall refuse to write the page into the repository's tracked files without being told
   the place explicitly.

### Requirement 3: The run, stage by stage
**Objective:** As a developer, I want to see what the tool is doing now and how far it is, so that
a long run is not a silent one.

#### Acceptance Criteria
1. While a run goes on, the run page shall show every stage of the run in order, and for each
   whether it has not begun, is going on, has ended, or failed.
2. While a stage that works through a known number of items goes on, the run page shall show how
   many are done, how many are left, and the time it expects to need for the rest.
3. While planted faults are being run, the run page shall show for each function how many faults
   were noticed and how many were not, as far as the run has come.
4. The run page shall show changes of the run within five seconds of their happening, without the
   person reloading it.
5. When a copy drops out of a run, the run page shall say so at once, with the reason the tool
   has, and shall say how many copies are still working.
6. When the run ends, the run page shall say so, with the result the command returned and the
   time the run took.
7. If the tool ends without finishing the run, then the run page shall not go on showing the run
   as going on: it shall say that the run stopped and when it last heard from it.
8. The run page shall take every time it shows from the run's own records.

### Requirement 4: What happens to your code
**Objective:** As a developer who cares about my codebase, I want to be told plainly what the tool
touches, so that I can let it run without watching my files.

#### Acceptance Criteria
1. The run page shall say, at the top and for the whole run, that the tool works in copies and
   shall name where the copies are.
2. The run page shall name the repository, the branch and the commit the run reads, and shall say
   whether the working tree had uncommitted changes when the run began.
3. The run page shall say which commands of the project the run executes, and that it executes
   them in the copies only.
4. Where a run writes a file into the repository, such as a verdict, the run page shall name that
   file and say that it is the only kind of file the run writes there.
5. Where the run judges a change, the run page shall name the branch the change is on, and shall
   warn if that branch is the repository's main branch.
6. When the run has ended, the run page shall say how much space the copies take and how to remove
   them.

### Requirement 5: What a planted fault is
**Objective:** As a developer who has not met fault injection, I want it explained where I see it
happen, so that I understand the numbers.

#### Acceptance Criteria
1. The run page shall explain a planted fault in a few sentences without assuming the term is
   known: a small deliberate change in a copy, the tests run, and the fault is noticed if a test
   fails.
2. The run page shall show at least one fault of the current run as an example: where it was
   planted, what was changed, and whether the tests noticed it.
3. The run page shall say that a planted fault never reaches the repository.
4. The run page shall say what a noticed fault shows and what it does not: that the tests are
   sharp at that place, and nothing about whether two versions of the code behave the same.
5. Where faults were planted in code the platform does not compile, the run page shall show them
   apart and say that they tell nothing.
6. Where a fault is not noticed, the run page shall name the three things that can mean: a test is
   missing, the fault changes nothing a caller can see, or it changes something the tests cannot
   observe.

### Requirement 6: Without tests, nothing is protected
**Objective:** As a developer, I want to be told before I change anything where my tests would not
notice a mistake, so that I do not take a quiet result for a safe one.

#### Acceptance Criteria
1. Where no test notices any planted fault in a function, the run page shall mark that function as
   unprotected and say that a change to it can break it without any test failing.
2. Where the project's tests do not pass before any fault is planted, the run page shall say that
   the run cannot judge anything, with the reason the tool has.
3. Where the run was given no test command, or the test command runs no test, the run page shall
   say that no fault can be noticed and that every result of the run is empty.
4. Where a run judges a change to an unprotected function, the run page shall say that the
   acceptance means only that nothing got worse, and that nothing was protected before.
5. The run page shall show the share of unprotected functions of the scope before any other result
   of the faults.
6. Where a function cannot be tested as it stands because it needs a device, a window or a
   network, and the tool knows what it reaches out to, the run page shall list that and say that
   tests cannot come first there.

### Requirement 7: The report
**Objective:** As a developer, I want the result on the same page when the run has ended, so that
I can read it, keep it and show it.

#### Acceptance Criteria
1. When a run has ended, the run page shall stay readable without the tool.
2. Where a run compares a state before and a state after, the report shall show the two side by
   side for each lens that was measured.
3. The report shall mark every lens that was not measured as not measured, and shall not leave it
   out.
4. The report shall show for each lens the direction that was expected and whether the measure
   moved that way.
5. Where the functions and calls of the scope were measured, the report shall draw them, before
   and after to the same scale.
6. Where the shapes of a type were counted, the report shall show which can be written, which the
   code reaches and which the tests see, and shall mark any of the three that was not measured.
7. Where a change was refused, the report shall show the refusal and its reasons as prominently as
   it would show an acceptance.
8. The report shall name the tool's version, the commands that produced each number and the
   commits they ran on.
9. The report shall show the project's file and function names and no source text, unless the
   person asked for source text to be included.

### Requirement 8: Several reports, side by side
**Objective:** As someone who wants to show what lesscode did, I want one page that lists several
reports, so that improvements can be compared.

#### Acceptance Criteria
1. Where the person names several reports, the tool shall write one page that lists them, with for
   each the project, the scope, the date and the lenses before and after.
2. The list shall link to each report.
3. The list shall show a report that was refused, or in which a lens got worse, in the same way as
   the others.
4. Where the person asks for it, the tool shall leave the names of files and functions of a named
   project out of the list and of its reports.
