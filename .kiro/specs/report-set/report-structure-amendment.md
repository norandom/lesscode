# Report set: structure and vocabulary of the report (proposal)

Status, 2026-11-27 about 13:00 Europe/Berlin: the structure is accepted by the owner at the fourth
look ("good structure"), on the rebuilt prototype of section 8. Evidence:
[R202](../../../docs/research/2026-11-27/03-report-structure-and-vocabulary.md). Requirements, design
and tasks are not amended yet; that is the next step and needs the owner's approval phase by phase.
The owner's last remark is a rule for the wording: a refactoring that does not simplify is natural;
"mixed" and "unchanged" are findings, not failures.

As first written: proposal, written 2026-11-27. Not approved. It answers the open point G8 of the prototype
gate ("How does it look on paper?"): the owner read `set-windows.pdf` and `base-windows.pdf` and
rejected them. The owner's words: the report is a data dump, it is not comprehensible, its language
is not suited for programmers, and it must be grounded in an ontology.

This file is a composition blueprint made with the COBESY skill (cognitive layer, composition
back-end). It gives the section skeleton and the vocabulary. It drafts no report prose. Requirements
4 and 6 and the design prescribe the present structure, so nothing here takes effect until the owner
approves it and the spec is amended (see "What this changes in the spec").

## 1. Diagnosis of the present PDF

| Finding | Where it shows |
|---|---|
| No conclusion. The reader is never told what the six reworks achieved. | The title counts reworks and projects. No sentence states a result. |
| The first sheet opens with bookkeeping. | Input file names and two hand-written notes with internal record numbers (R199) come before any result. |
| Everything has the same weight. | Each rework shows all six lenses, including the empty ones. "not measured" fills a large share of the first table. |
| The same numbers appear three times per rework. | Row of the side-by-side table, the tiles, the before-and-after table. |
| The heads name topics, not results. | "The steps side by side", "Before and after", "Functions and calls". |
| The words are paraphrases a programmer has to translate back. | "planted faults", "places that write stored state", "calls from one function to another", "how tangled single routines are". |
| One concept has several words, and several words have no definition. | noticed / detected; handed in / stated; routine / function; "lens", "moved that way", "judged", "shapes of the state". |
| The measures are free text per rework. | 22 distinct measure strings in six reworks, for example "cells holding the lease". Two reworks cannot be compared on them. |
| The most important caveat is buried. | "No test noticed any fault before the step in 9 functions; acceptance there means only that nothing got worse" is in a detail line. |

## 2. Reader and purpose

- Reader: a programmer who was sent the PDF. They did not run the tool and do not know its words.
  They know refactoring, test suites, mutation testing, call graphs, cyclomatic complexity.
- What they want to know, in this order: is the behaviour preserved, is the code simpler, how far
  can I trust each number.
- Action after reading: state for each refactoring whether it is acceptable and on what evidence,
  after the first sheet, and look up any term in the glossary without leaving the document.
- Assumption about this reader: a claim counts only when it is stated in terms they already use and
  can check against the code. Invented wording reads as vagueness.
- Tone: terse, technical, one term per concept. Standard terms are not cryptic labels: every term
  resolves to a glossary entry. Internal record numbers stay out of generated text.
- We are wrong if: a programmer who has never seen lesscode cannot name verdict, what got simpler
  and the evidence class of each refactoring from the first sheet alone.

## 3. Governing idea

One sentence, generated from the data of the set, and visible in the title, the summary and every
section head:

> N refactorings in P projects: behaviour preserved in B, simpler in S; K verdicts by the tool, R
> reported by the authors.

The report argues this sentence. It does not list everything that was counted. Evidence that does
not support or limit the sentence goes to the appendix.

The support is three questions, and every metric belongs to exactly one of them:

1. Simplicity: is the code simpler? (mutable state, function complexity, call graph)
2. Safety: is the behaviour preserved? (test results, mutation testing)
3. Cost: what did it cost? (size, as a guard that it did not grow, shown last)

The order is the owner's decision of 2026-11-27: what got simpler first, then the proof that
behaviour is preserved.

## 4. The vocabulary (the ontology)

Home: `.kiro/vocabulary.toml`, in the format of openidea-kiro (schema version 2, topics and terms
with `kind`, `definition`, `broader`, `alt_labels`). The present wording becomes `alt_labels`, so
old and new resolve to the same term.

The file is written (46 terms in 2 topics; `kiro-cli vocab validate` of kiro-cli 1.1.0 reports no
finding). It is the source; the two tables below are the first draft and differ from it in three
points: the dimension of functions and calls is "Call graph", not "Coupling"; the term
`overall-result` (improved, mixed, worse, unchanged, not measured) was added for the result of a
whole group; and the values of the enumerated terms are written the way they are printed ("not
judged", "as expected"). Rules:

- Every label the report prints is the `label` of a term. A check fails when the page prints a
  dimension, metric, verdict or evidence word that is not in the vocabulary.
- A step file names a metric by term id, not by free text. A metric outside the vocabulary is
  allowed only as an explicitly marked custom metric with its own definition in the file.
- The glossary in the PDF is generated from the vocabulary: only the terms this set uses.
- Each definition is checked against the code that computes the number before it is written.
  Entries marked "verify" below are my reading of the output and are not yet checked.

### Topic `refactoring-report`: the things a report is about

| Term id | Label | Kind | Present wording | Definition (draft) |
|---|---|---|---|---|
| `report-set` | Report set | entity | set | One file that holds several refactorings with their measurements. |
| `refactoring` | Refactoring | entity | step, rework | One change to a named scope that is meant to preserve behaviour, compared before and after. |
| `scope` | Scope | entity | files in scope | The files and functions a refactoring may change. |
| `revision` | Revision | enum: before, after | before, after | The state of the scope a number was taken from. |
| `expected-change` | Expected change | enum: decrease, no-change, no-increase | falls, stays, does not rise | The direction a metric must take for the refactoring to count as an improvement. |
| `metric-result` | Result | enum: as-expected, not-as-expected, not-measured | moved that way: yes / no | Whether the metric changed in the expected direction. |
| `evidence-source` | Evidence | enum: measured, reported | measured by, handed in, stated | measured: lesscode ran the command named beside it. reported: the author supplied the number and lesscode did not check it. |
| `verdict` | Verdict | enum: accepted, rejected, not-judged | outcome, refusal, "not judged in this set" | The decision of `step check`. not-judged: the check was not run, the figures are reported. |
| `rejection-reason` | Rejection reason | entity | refusal for ... | The check that failed. |
| `anonymized-project` | Anonymized project | bool | counts only, "project 1" | The project's names are removed; only numbers are shown. |

### Topic `code-metric`: dimensions and their metrics

Dimensions are the parents (`broader`) of their metrics. Group is the question of section 3.

| Group | Dimension (present word) | Metric id | Label | Present wording | Note |
|---|---|---|---|---|---|
| Safety | Behaviour preservation (Behaviour kept) | `test-result` | Test suite result | "your tests on the code, with no fault planted", "pass to pass" | pass or fail, per revision |
| Safety | Test strength (Protection) | `mutants-killed` | Mutants killed | planted faults noticed / detected | |
| Safety | | `mutants-survived` | Mutants survived | unnoticed planted faults | |
| Safety | | `mutants-not-covered` | Mutants not covered | "neither" | verify what "neither" is |
| Safety | | `untested-function` | Function with no killed mutant | unprotected function, "nothing was protected before" | |
| Simplicity | Mutable state (State) | `state-write-sites` | State write sites | places that write stored state | |
| Simplicity | | `representable-states` | Representable states | ways the state can be written | verify |
| Simplicity | | `reachable-states` | Reachable states | ways reached by the code | verify |
| Simplicity | | `tested-states` | States covered by tests | seen by the tests | verify |
| Simplicity | | `write-only-fields` | Write-only fields | holders set and never read | |
| Simplicity | | `lock-guarded-fields` | Lock-guarded fields | locked fields | |
| Simplicity | Function complexity (Local complexity) | `branch-count` | Branches per function | decisions of one routine, conditions | cyclomatic complexity minus one, per `docs/state-refactoring.md`; verify |
| Simplicity | | `nesting-depth` | Nesting depth | nesting | |
| Simplicity | | `parameter-count` | Parameter count | parameters | |
| Simplicity | | `function-length` | Function length | length | |
| Simplicity | Coupling (Structure) | `function-count` | Functions | functions | |
| Simplicity | | `call-edges` | Call-graph edges | calls from one function to another | static, resolved by name |
| Simplicity | | `cross-file-call-edges` | Cross-file call edges | calls across files | |
| Simplicity | | `coupled-file-pairs` | Coupled file pairs | pairs of files with a call between them | |
| Cost | Size | `token-count` | Tokens | tokens of the files in scope | |
| Cost | | `ast-node-count` | AST nodes | syntax nodes of the files in scope | |

No vocabulary term yet: "cells holding the lease" (a count made for one rework) and "shapes of the
state" (needs a name such as "state space of a type"; verify what it draws).

## 5. Section skeleton

Heads are generated from data and state a result. Bullets say what the section covers. No prose is
drafted here.

| # | Head (pattern) | Covers | Emphasis | Terms |
|---|---|---|---|---|
| 1 | Title: the governing sentence of section 3 | Name of the set, date, tool version in one line under it (requirement 6.6) | The numbers B, S, K, R | refactoring, verdict, evidence |
| 2 | Summary | Three lines: what was compared; the result counts; the limit of the evidence (how many numbers are reported and not measured, how many functions have no killed mutant) | The limit is stated here, not in a footnote | evidence, untested function |
| 3 | Results | One row per refactoring: scope and project, verdict, what got simpler (only metrics that changed, as before -> after), tests, mutants, evidence | Verdict first. No column per dimension, no "not measured" cells; unmeasured dimensions are one count per row | verdict, test result, mutants, metric labels |
| 4 | How to read this report | The model in at most eight lines or one small diagram: refactoring, scope, two revisions, metrics in dimensions, expected change, result, verdict, evidence | Placed before the detail, so the terms are known when they are used | the terms of topic `refactoring-report` |
| 5 | Author's notes | The hand-written notes, marked as such | After the results, not before | |
| 6.n | "n. Title (project): verdict" plus one generated sentence | One fixed block per refactoring, parts a to d below | Same order in every block | |
| 6.n a | Simplicity | Only the metrics that changed, grouped as state, complexity, coupling; before, after, difference, expected change, evidence | A metric that moved against expectation is as prominent as one that moved with it (requirement 4.6) | metric labels |
| 6.n b | Safety | Test suite before and after; mutants killed and survived before and after; functions with no killed mutant, with what an acceptance means there | The caveat of the untested functions | test result, mutants, untested function |
| 6.n c | Cost | Size before and after | Last and short | tokens, AST nodes |
| 6.n d | Evidence drawings | Call graph before and after, with a caption that states what changed; state space of a type where measured | Omitted with one line when not measured | call-graph edges |
| 6.n e | Not measured | One line naming the dimensions without data | One line, not table rows | |
| A | Appendix: all metrics | The full table per refactoring, including unmeasured rows and the command that measured each | Reference, not argument | |
| B | Appendix: mutation results per function | The per-function table; the body shows only functions whose result changed | | |
| C | Appendix: glossary | The vocabulary terms this set uses, with definitions | Generated from `.kiro/vocabulary.toml` | |
| D | Appendix: provenance | Tool version, date, input files and their kinds | Moved from the top of sheet 1 | |

Checks on the skeleton:

- Each number appears once in the body. The tiles go; the full table is appendix A.
- The three groups do not overlap and cover every metric of section 4.
- Sheet 1 holds sections 1 to 3. A set too large for that keeps 1 and 2 on sheet 1.
- Nothing is dropped from the document: requirement 6.1 (everything the page shows is on paper)
  holds through the appendices.

## 6. What this changes in the spec

Approved text that the proposal contradicts or extends:

- Requirement 4.3 (side-by-side table with each lens before and after) and 4.4 (each step shows what
  the step report shows today): replaced by sections 3 and 6.n above.
- A new requirement: the report's labels come from the vocabulary, a set names metrics by term, and
  the PDF carries a glossary.
- Design: the set page's own parts replace the reuse of `Tiles` and `Lenses` in set mode; the set's
  step file takes metric ids.
- Tasks 2.1 and 2.3 change; one task is added for the vocabulary and its check; task 4.2 (step file)
  and 8.1 (example) follow.
- The prototype gate's point G8 is recorded as not accepted, with a research entry.

## 7. Decisions of the owner, 2026-11-27

1. The ontology is `.kiro/vocabulary.toml` in openidea-kiro's format.
2. Scope of the renaming: the set page and its PDF in this spec. The run page, `step check`, the
   README and `docs/steps.md` keep the present words until a follow-up spec; requirement 8 (nothing
   else changes) stays as it is. The present words are `alt_labels` of the new terms.
3. One before-and-after change is called a "Refactoring". "step" and "rework" are alternative labels.
4. The groups are Simplicity, Safety, Cost, in that order.

These decisions settle the direction. The amended requirements, design and tasks are still to be
written and approved; a rebuilt prototype PDF is shown to the owner first (prototype-first rule).

## 8. The rebuilt prototype, 2026-11-27

Built in `D:\BuildArtifacts\lesscode\report-set` (the prototype of the gate, not part of lesscode):

- The research program compiles `.kiro/vocabulary.toml` into itself. For each refactoring it writes
  the rows in the terms of the vocabulary (`run.report.metrics`: dimension, metric or custom metric,
  detail, before, after, expected change, result, evidence, source) and puts the terms the set uses
  into the set (`vocabulary`). A step file names a metric by term id; a row of a report page is
  mapped through the alternative labels; a wording the vocabulary does not know stays a custom
  metric. A metric named under the wrong dimension is refused.
- The set page (`page\run.html`, set mode only; the run mode is untouched) follows section 5. Every
  label of a dimension, metric, verdict, result and kind of evidence is read from the vocabulary in
  the set, and so is the glossary.
- `look.ps1` makes the set, the page and the Windows PDF into `out-restructure\`. The earlier
  outputs in `out\` are as the gate left them.

What the new first sheet says of the six refactorings: simplicity improved in 4 and mixed in 2;
tests pass at both revisions in 5 of 6; 1 accepted, 5 not judged; 18 of 36 values are reported and
not checked; 9 of 27 functions with mutation results had no killed mutant before. The two "mixed"
results (functions and call-graph edges rose in one refactoring, one more call-graph edge in
another) stood in the earlier report only in the tiles and rows of each rework; its first sheet
did not say them. The PDF has 17 sheets, against 14 before: the appendices repeat the rows in full
and add the glossary.

### Second look of the owner, 2026-11-27

The owner rejected the first rebuilt PDF and named what was missing: the project name, the author,
the tool's name and version, horizontal bar charts of the metrics, and a summary that opens with
the key numbers. Added to the prototype:

- A head on the first sheet: projects, author, tool and version, date. The author is stated by hand
  in the notes file (`author`) and carried by the set; a set without one says "not stated". The
  tool is the one that measured (here `lesscode-core 2.1.0`); the research program that wrote the
  set is named beside it. Each refactoring names its project, verdict, evidence and revisions in the
  same form.
- The summary opens with six headline numbers as tiles: refactorings, simplicity improved, tests
  pass before and after, accepted by lesscode, values measured by lesscode, functions with no
  killed mutant.
- Horizontal bars, before and after, for every metric with two numbers: under the summary, summed
  over the refactorings that have the metric (custom metrics are in no sum), and in each
  refactoring for Simplicity, Safety and Cost. They replace the tables in the body; the tables are
  appendix A. Each metric is drawn to its own scale, because the metrics have different units, and
  the figure says so. One hue in two steps (before lighter, after darker), checked with the
  palette validator of the dataviz skill for the light and the dark scheme; a result is a coloured
  mark beside its words, not coloured text.
- The row per refactoring in "Results" no longer repeats the numbers: it gives the overall result
  of each group and the count of metrics behind it.

The PDF has 19 sheets.

### Third look of the owner, 2026-11-27

"Much better." Still missing up front: the revision (git hash), the branch and the like. And the
same report is wanted at three widths: for one run, for one project, and across projects. Added:

- Which code a report is about, directly under the head. A report of one run names repository,
  branch, commit before, commit after and uncommitted changes in its head. A report of several
  refactorings has a table with a row per refactoring: branch, commit before, commit after,
  uncommitted changes. A run of the tool already records all of it (`Subject` of the run's data); a
  step file may now state a `branch` beside its two commits. What is not recorded reads "not
  stated".
- An anonymized project withholds its repository and its branch as well as its names: a path or a
  branch name can carry the project's name. The research program removes both from the set, and
  the page says "withheld".
- Three widths from the same inputs and the same page: every input (across projects),
  `--only-project <name>` (one project), `--only-step <n>` (one run). The page reads the width from
  what the set holds. One run: the head carries the run's code, the tiles are the run's own
  (verdict, simplicity, test suite result, mutants killed, values measured, functions with no
  killed mutant), and there is no table of results and no sum. One project: the title and the
  tiles name the project; the column of projects is gone. A narrowed report numbers its
  refactorings from 1 and does not carry the notes about the whole set.
- Eight terms were added to the vocabulary for the head: project, run, repository, branch, commit,
  uncommitted changes, author, tool (54 terms; `kiro-cli vocab validate` reports no finding).

`look.ps1` writes the three reports into `out-restructure\`: `across-projects.pdf` (19 sheets),
`project-drift-rdp.pdf` (15), `run-display-path.pdf` (10).

To be settled in the spec: the report of one run is a set of one refactoring made from the run's
report page, so the run page itself stays as it is (decision 2). A report page holds neither the
project's name nor a title for the refactoring; today both come from the notes file.

Not done, and to be said when the prototype is judged:

- The gate's checks were not run on the new page: the browser checks of `checks\set.spec.mjs`, the
  search for names of the anonymized project, the Linux PDF. They expect the earlier wording.
- The receipts of the anonymized project were removed after the gate. For this look they were made
  again from the drawing in the gate's set, which holds numbers in place of names.
- The page does not print the tool's own sentence about an accepted refactoring
  (`outcome_detail`), because it is in the earlier words; the same facts are generated from the
  mutation results per function.
- Prototype shortcuts: the set keeps the earlier rows (`lenses`) beside the new ones; the order of
  groups and dimensions is fixed in the page; the row of three mutant counts is split by the
  research program.
- Five definitions are marked "verify" in the vocabulary: state write sites, lock-guarded fields,
  branches per function, function length, mutants not judged.
- "cells holding the lease", "conditions" and "tokens more than before" have no term and are shown
  as custom metrics.
