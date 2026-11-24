# R180: the run page, short first: the flow of the page and its descriptions

- Report date/time and timezone: 2026-11-24, 07:50 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, about 07:20 to 07:50 (the start is the owner's request and was not read from the clock; the first check in a browser ran at 07:35)
- Status: task 9.1 completed on Windows; on Linux the suite ran at the first of its two commits
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), task 9.1
- Hypothesis ID and predecessor: [R179](../2026-11-23/16-run-page-group-7.md)
- Type: implementation record with its deviations; the reasoning follows the skill `cobesy`

## Decision

The owner asked for the page to be short and simple at first, with a way to go deeper: developers
are heavily loaded, work with AI puts their understanding under stress, and a new tool that makes
their code different needs explainers and pictures.

The page now answers three questions in this order, before any table: what is this, what did it
find, what do I do. Everything it showed before is still there, behind them. Only the page was
changed (`native/page/run.html`, `native/page/list.html`); the data the tool writes is the same.

## The reasoning, in the order of the skill

**The behaviour wanted.** A developer who opens the page can say within a few seconds whether
their code was changed, what the run found, and what to do next, and does that next thing
(writes a test for an unprotected function, or fixes a refused step) without first learning
the tool's words.

**The assumption designed around.** "A tool that goes at my code and says it is not good enough is
judging me, and what tools say these days has to be checked by me anyway." Nobody was asked; this
is my reading of the owner's sentence.

**How the page is framed because of it.**

- The page says first what does not happen: "No file of your project is changed."
- The head is the developer's own question, not the name of a command: "Which of your functions do
  your tests protect?", "Does this change keep what your tests protect?"
- The page says what its result does not show. "An unnoticed fault can mean three things" is open
  by default; "A noticed fault shows less than it seems" is one press away.
- Words the page does not use about the person or the code: bad, poor, wrong, should have.
- We are wrong if: a developer who sees the page for the first time cannot say, after ten
  seconds, whether their code was changed and what the run found. Task 8.1, the owner watching a
  real run, is where that is seen.

**The order of what is said (the conclusion first).**

| Level | On the page |
| --- | --- |
| The question | the head, in the person's words; at the end the outcome stands beside it |
| Is my code safe | one bold sentence, then where the copies are and what is written |
| What it found | the outcome in its box; while it runs, one sentence on the stage |
| What I do | "What to do next" directly under the outcome; while it runs: nothing is needed |
| In short | one tile per lens in plain words; one square per function |
| The numbers | the rows of the measures, the faults per function, the drawings, as before |
| How it works | four steps of a planted fault, closed until opened; the tab "Planted faults" |

**Through whom it spreads.** Not worked out. A page has readers; the skill's part on who adopts
first belongs to task 8.1 and the documentation.

## What was changed

| Where | Before | Now |
| --- | --- | --- |
| Head | the command, the repository, the commit; "Ended" | the question the run answers, then the same; an ended run is named by its outcome, in its colour |
| What the run touches | began with where the copies are | begins with "No file of your project is changed." |
| The bar of faults | colours only, numbers in a name for screen readers | a line under it: how many were planted, noticed, unnoticed, still to run, each with what it means |
| The tree | marks explained only on hover | under it, the meaning of the marks it shows now, and of no other |
| A function with no fault noticed or unnoticed | the mark of a finished one | a mark of its own |
| A stage, while it runs | title, all sentences, the bar | one sentence first; "Nothing is needed from you"; the functions so far as squares; the four steps, closed |
| The result | outcome, compared, table, per function, drawings, facts, next | outcome, next, a tile per lens, the squares, then the rest as before |
| A lens | a row of seven columns | also a tile: its name, what it means in plain words, whether it moved as expected, the first of its numbers |
| Tab "Planted faults" | one paragraph of four sentences, then three headings with text | one sentence, the four steps as a picture, the example, three parts that open |
| The list of reports | the full table of every report | the tiles; the table of a report opens on request |

A tile says "unchanged" where a measure has the same value before and after, and "did not move as
expected" only where the value differs. The tool marks both as not moved; two functions before and
two after is not a warning.

## Evidence and results

| Check | Result |
| --- | --- |
| The pages in Chromium through Playwright, at `05955d4` | 66 of 66, none skipped; one test is new |
| Core suite on Windows, `cargo nextest`, at `30cb69d` | 471 of 471, 1 skipped; after the last change to the page the 59 tests of the page, 1 skipped |
| Format check; `tools\bin\kiro-cli.exe code check --changed` | pass |
| Linux through Dagger at `30cb69d` | overall 0; core suite 470 of 470, 1 skipped (`D:\BuildArtifacts\lesscode\native-evidence\linux-30cb69d`) |
| Linux at `05955d4` | not run; that commit changes two pages and one check of the browser, nothing the Linux run looks at but the text taken into the binary |

What the browser check now holds: the outcome is the name of the run at the top; the outcome, then
"What to do next", then "In short", stand before "What was compared" and the rows; six tiles with
their words; the squares and their sentence; only the marks shown are explained; the four steps
are closed while a run goes on and open with Enter; in the list the rows of a report are hidden
until its line is opened by keyboard. A refusal still stands where an acceptance stands.

Three texts of the check had to change, all of them the place of "What to do next". No sentence
that a requirement asks for was removed; the long paragraph on planted faults is now one sentence
and four steps that say the same.

I looked at these pictures: a run going on at 1,280 and at 400, an accepted and a refused check, an
ended plan, the tab "Planted faults", and the list.

Outputs: `D:\BuildArtifacts\lesscode\run-dashboard\states-g9-4` (10.6 MiB, a picture of every
situation), `reports-g9` (1.0 MiB, the reports and lists written with the new page).

## Deviations and my faults

1. **No one of the audience was asked.** The assumption, the three questions and every wording are
   mine. The disproof condition above is untested until task 8.1.
2. **The page compares two numbers.** "Unchanged" on a tile is the page finding that before and
   after are the same value. The design says the page computes nothing the tool knows. The tool
   could say it in the data; it does not yet.
3. **The plain words for the lenses are in the page**, not in the data, and exist for the six
   lenses the tool has. Another lens gets a tile without them.
4. **"No file of your project is changed" is said for every run**, also where the data says
   nothing about what is written. It holds for `step plan` and `step check` by how they are built.
5. **An outcome I did not change reads oddly**: a plan that finds two of three functions
   unprotected ends in a green box, "The plan is written". The colour follows the exit code.
6. **At 400 wide** the head, the line on the code and the line under the bar take about a third of
   the first screen.
7. **The first commit of the task (`30cb69d`) showed an unchanged measure as a warning.** I saw it
   in the picture of the list after the commit and changed it in a second commit (`05955d4`).
8. **Not seen**: Firefox, Safari, a screen reader, a real run. The review was inline, by me.

## Storage

D: had 120.3 GiB free at 07:45, after the Linux run (122.4 GiB the evening before). Three superseded outputs of the browser check were removed
(`states-g9-1` to `-3`, 28.7 MiB). Kept: `states-g9-4`, `reports-g9`, the Linux evidence, and what
handover 82 names.
