# Design: the run page

Provisional. Written 2026-11-23 after the owner approved the [requirements](requirements.md)
("approved, go on to the design and its prototype"). It becomes final only after the
[prototype gate](prototype-gate.md) has been run and the owner has decided the points in
"Decisions for the owner".

Two wishes of the owner shape it beyond the requirements:

> would be happy about a interface that uses a mature javascript approach and buttons typical to
> idea ide

> not like an AI slop UI which breaks easily. we need to think a little about exceptions, waiting
> time and flows

So the design starts from what can go wrong and how long things take, and gives the page its own
small, written-down state machine. The look comes after that.

## Overview

```
 lesscode-core (Rust)                          browser, page opened from disk
 ┌───────────────────────────┐                ┌──────────────────────────────┐
 │ a run (step plan, ...)    │                │ run.html                     │
 │   │ events                │                │  ├ page code (embedded)      │
 │   ▼                       │  writes        │  ├ view library (embedded)   │
 │ RunLog ── snapshot ───────┼──────────────► │  └ loads run-data.js again   │
 │   │  every change, and    │  run-data.js   │     every second             │
 │   │  at least every 5 s   │                │                              │
 │   ▼ at the end            │  rewrites      │                              │
 │ run.html with the data in ┼──────────────► │ the same page is the report  │
 └───────────────────────────┘                └──────────────────────────────┘
```

In this first form nothing listens on the machine. The page never talks to the tool; it only reads
what the tool wrote. Exit codes and receipts are produced as today, by the same code as today.

### Subscribing instead of reading again (owner's question, 15:10)

> is it possible to have a rust worker and the UI subscribes to the tools?

Yes, in two sizes.

- **The tool serves its run while it lives.** It opens a listener on `127.0.0.1` with the standard
  library, serves the page there, and sends each new snapshot down one long answer (an event
  stream; the browser's `EventSource` subscribes to it). No dependency is needed. The page learns
  at once that the tool is gone, because the stream ends. What it costs: a listening port for the
  length of the run, which other programs on the machine and pages in the same browser can try to
  reach, so the address needs a secret in it and the listener must refuse requests that do not
  carry it; and the live view ends with the tool, so the report still has to be a file.
- **A worker that outlives runs.** One long-lived Rust process that runs report to and pages
  subscribe to: one page for several runs, a history, perhaps starting a run from the page. This is
  outside the approved requirements ("a server that outlives the run" is out of scope) and would
  need them amended.

The snapshot is the same in every case; only how it travels differs. The prototype tries both the
file and the event stream of the first size, so that the choice is made on what was seen.

## Parts

| Part | Where | What it does |
| --- | --- | --- |
| Page template | one HTML file in the crate, taken into the binary with `include_str!` | style, page code, the view library; a marked place for data |
| `RunLog` | a module of the driver | takes the events a run already has (a stage begins, a fault is classed, a copy stops) and keeps the snapshot |
| Snapshot | one JSON value, versioned (`schema: 1`) | everything the page shows; the same value during the run and at its end |
| Writer | part of `RunLog` | writes the snapshot as `run-data.js` beside the page, by writing a new file and putting it in place of the old; at the end writes `run.html` again with the snapshot inside |
| Opener | a few lines | starts the default browser on the page, only with `--open` |
| Gallery writer | a subcommand | reads several reports' snapshots and writes one list page |

The run's own code changes in one way: where it prints progress to the terminal or adds to a
receipt, it also tells `RunLog`. A run without `--page` has no `RunLog`.

## The snapshot

One value. Every field the page needs is in it; the page computes nothing that the tool knows.

```
schema, tool {name, version}, command, started, heartbeat, ended, outcome, exit_code
subject {repository, branch, commit, dirty, on_main_branch}
copies {directory, planned, working, dropped [{copy, at, reason}], bytes}
commands [[...]]            the project's commands the run executes
writes_in_repository [...]  files the run writes there, if any
tests {command_given, passed_unchanged, tests_run, reason}
stages [{id, title, status, started, ended, done, total, what, typical_seconds}]
faults {total, done, noticed, unnoticed, not_built, not_compiled_here,
        example {file, line, what_was_changed, outcome},
        functions [{file, function, noticed, unnoticed, not_built, left, unprotected, reaches_out}]}
report {lenses [{lens, measure, before, after, expected, moved, measured}],
        graph {before, after}, shapes [...], refusal {reasons}}        only when there is one
```

`heartbeat` is written at least every five seconds while the tool lives, also during a build that
has nothing to count. That one field is what lets the page tell "working" from "gone".

## The page's states

The page is always in exactly one of these. Each has its own screen; none is an error message
pasted over another screen.

| State | When | What the person sees |
| --- | --- | --- |
| waiting for data | the page is open and no snapshot has been read yet | "Waiting for lesscode to write its first data", how long it has waited, and where it looks |
| running | a snapshot was read, the run has not ended, the heartbeat is younger than 15 seconds | the run, stage by stage |
| quiet | the heartbeat is 15 to 60 seconds old | the run as last seen, and "No word from the tool for N seconds" |
| stopped | the heartbeat is older than 60 seconds and the run has not ended | "The tool stopped without finishing. Last heard at ..."; the run as last seen; nothing shown as going on |
| ended | the snapshot says the run ended | the outcome, then the report |
| cannot read | the data is there and is not a snapshot this page understands (another `schema`, or damaged) | what was found, the page's and the data's versions, and "write the page again with this version of the tool" |

A read that fails once, because the tool was just putting a new file in place, changes nothing:
the page keeps what it has and tries again. Only the heartbeat's age moves the state.

## Stages and waiting

Each command has a fixed list of stages; `step plan` has four: copy the project, run the unchanged
tests in each copy, plant faults, close. A stage is `not begun`, `going on`, `ended`, `failed` or
`left out`.

Waiting is where a run loses people, so each kind of wait is shown for what it is:

| Kind of stage | What is shown | What is not shown |
| --- | --- | --- |
| it counts items (planted faults) | a bar with done and left; time for the rest, from the items done so far | a time for the rest before five items are done |
| it has nothing to count (the first build) | time gone so far, and one sentence on why it takes long ("the project is built from nothing in each copy") | a bar that pretends to know |
| it ended | how long it took | |

Times for the rest are given as a range once at least five items are done, and always with the
word "about".

## What can go wrong, and what the page does

| What happens | Where the tool knows it | What the page shows |
| --- | --- | --- |
| The unchanged tests fail in one copy | worker stops before any fault | a notice at the top, with the copy, the reason and how many copies still work; the run goes on |
| The unchanged tests fail in every copy | run is void | state "ended", outcome "could not judge", the reason, and "nothing below was measured" |
| No test command, or it runs no test | before the run | the same, before any stage begins |
| A planted fault does not build | fault classed as not built | counted apart; never as noticed or unnoticed |
| A planted fault runs over its time limit | fault classed as timed out | counted apart, with the limit |
| A fault lies in code the platform does not compile | known from the build | shown apart with the sentence that it tells nothing |
| The tool is ended from outside | no more heartbeats | state "stopped" |
| The disk is full, or the data file cannot be put in place | writer fails | the tool goes on and says so on the terminal; the page turns "quiet", then "stopped" if it lasts |
| The browser cannot be started | opener fails | the tool prints where the page is and goes on |
| The page is opened without its data file | page finds nothing | state "waiting for data", with the place it looks |
| An older page meets newer data | `schema` differs | state "cannot read" |
| A function has a very long name, or there are 2,000 functions | always possible | names wrap or are cut with the full name on hover; the list is drawn in parts |
| The page is left open overnight after the run | run ended | state "ended"; it stops looking for new data |

The page's own code has one rule for all of this: it draws from the snapshot and from nothing
else, every field may be missing, and a missing field is drawn as "not measured" or "not known".
If drawing fails all the same, the page shows the state "cannot read" with the failure's text, not
a blank screen.

## What the person is led through

1. **On opening**: what this run is, on which commit and branch, and the three assurances of
   Requirement 4 (copies, where, what is written into the repository). This stays reachable in a
   tab for the whole run.
2. **While it runs**: the stage going on is selected and followed, unless the person has chosen
   something else. Following can be switched back on with one button.
3. **The first planted fault** that is classed becomes the example of Requirement 5, shown with
   the explanation.
4. **As soon as a function turns out unprotected**, the tab "Unprotected" shows a count.
5. **At the end**: the outcome first, then what to do next for that outcome: refused, with the
   reasons and "fix the step, not the limit"; unprotected functions, with "tests come first"; and
   for code that needs a device, what it reaches out to.

## The interface

Laid out like a tool window of a desktop IDE, because that is where developers read runs of tests
every day:

- a **toolbar** of small square icon buttons with a tooltip each: follow the current stage, expand
  all, collapse all, show noticed, show unnoticed, show not built, sort, copy the place of the
  copies, show the explanations;
- under it **one bar** for the whole run, in parts: noticed, unnoticed, not built, left;
- on the left a **tree**: the stages, and under "plant faults" the files and their functions, each
  with a status mark and its counts;
- on the right a **pane with tabs**: Now, Your code, Planted faults, Unprotected, Result;
- at the bottom a **status line**: when the tool was last heard, copies working, time gone, time
  expected.

The icons are drawn for this page. No name, logo or typeface of an IDE vendor is used. Text is set
in the fonts the system has, so that nothing has to be fetched or embedded for it.

Every control works with the keyboard, shows where the focus is, and has a name for a screen
reader. Colour never carries a meaning alone: each status has a mark and a word as well.

## The view library

The page is drawn from one value into a tree of elements, again whenever the value changes. Doing
that by hand is where pages of this kind break. A small, long-established library does it instead.

| Choice | Size in the binary | Licence | Needs a build step for the page | Note |
| --- | --- | --- | --- | --- |
| Preact with htm, one file | 13 KiB | MIT and Apache-2.0 | no | the model of React, ten years in use; recommended |
| Lit | about 16 KiB compressed, more as text | BSD-3-Clause | no | web components |
| React with ReactDOM | about 140 KiB | MIT | usually | the same model, ten times the size |
| none | 0 | | no | the page's own code grows by what the library would do, and is tested by nobody else |

The library is one file kept in the repository with its licence and its hash, taken into the page
at build time, and named in the third-party notices. No package manager and no JavaScript build
are added to the project.

## Requirements traceability

| Requirement | Met by |
| --- | --- |
| 1 One page inside the binary | page template, `include_str!`; fonts of the system; one file when the run has ended (see decision 1) |
| 2 Asking for the page | options `--page <place>` and `--open`; opener; refusal for tracked places |
| 3 The run, stage by stage | `RunLog`, snapshot, the states "running", "quiet", "stopped", "ended"; heartbeat |
| 4 What happens to your code | `subject`, `copies`, `commands`, `writes_in_repository`; tab "Your code" |
| 5 What a planted fault is | `faults.example`; tab "Planted faults" |
| 6 Without tests, nothing is protected | `tests`, `functions[].unprotected`, `reaches_out`; tab "Unprotected"; outcome "could not judge" |
| 7 The report | `report`; tab "Result"; the page with the data inside |
| 8 Several reports | gallery writer |

## Decisions for the owner

1. **Two files while the run goes on.** Requirement 1.1 says the tool writes the page as one file.
   During a run there are two, the page and its data beside it; when the run has ended the page is
   written again with the data inside and is one file. I propose to read 1.1 as "one file when the
   run has ended".
2. **The view library**: Preact with htm, kept as one file in the repository. This adds a
   third-party file in a language other than Rust. It is not tooling and is not run when the tool
   is built.
3. **Thresholds**: quiet after 15 seconds without a heartbeat, stopped after 60; a heartbeat at
   least every 5 seconds.
4. **Which commands get the page first**: `step plan` and `step check`. `deconvolute run` and
   `step pin` later.
5. **How the snapshot travels during a run**: read again from a file, or subscribed to from the
   tool over an event stream on `127.0.0.1`, or both with the file as the fallback. To be decided
   after the prototype. A worker that outlives runs is a separate decision and changes the
   requirements.

## What the prototype has to show before this is final

See the [prototype gate](prototype-gate.md): that a page opened from disk follows a data file that
changes; that putting a new data file in place does not disturb a page that reads it; that the
page with the data inside needs nothing else; that the states above are what the page shows for
their inputs; and that the page does not break on an empty run, a large one and a damaged one.
