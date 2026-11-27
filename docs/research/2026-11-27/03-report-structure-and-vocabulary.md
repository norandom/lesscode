# R202: The owner rejected the look of the report set; rebuilt on a vocabulary, with the result first, it was accepted at the fourth look

- Report date/time and timezone: 2026-11-27, 13:00 Europe/Berlin (from the clock)
- Execution interval: 2026-11-27, about 12:00 to 13:00 Europe/Berlin
- Status: completed
- Spec: [report-set](../../../.kiro/specs/report-set/report-structure-amendment.md), point G8 of the
  [prototype gate](../../../.kiro/specs/report-set/prototype-gate.md); requirements 4 and 6 are
  affected and not yet amended
- Hypothesis ID and predecessor: G8 of the gate ("How does it look on paper?"), left to the owner in
  [R200](01-report-set-prototype-gate.md)
- Type: exploratory (G8 had no expectation written down; the owner's judgment is the measure)

## Decision

The structure of the report is settled by the owner: "good structure", after three rejected or
incomplete versions. The requirements, the design and the tasks of the spec may now be amended to
it and put to the owner for approval. Nothing of the tool is changed by this entry.

What is settled is how a report reads. That the gate's other points (G3 names out, G4 the page's
checks, G5 the PDF on Linux) still hold for the new page is not shown: those checks were not run
again.

## Hypothesis and falsification

R200 made a PDF that held everything (G5) and left its look to the owner. The owner read it and
rejected it: a data dump, not comprehensible, not in a programmer's language, not grounded in an
ontology. The proposal was to rebuild the page so that it states its result first, in terms that
are defined once in a vocabulary, and to let the owner judge again. Rejected when the owner says no.

## Frozen inputs and method

- The prototype of the gate, `D:\BuildArtifacts\lesscode\report-set` (outside the repository, not
  part of the tool): the research program `proto\src\main.rs` (sha256 `092f589feeebb383…`), the copy
  of the run page `page\run.html` (`053e21469a672aa5…`, set mode only; the run mode is untouched),
  the start script `look.ps1` (`4a782cd431a3fb00…`).
- The vocabulary `.kiro/vocabulary.toml` (`f3c56a9dcaf39774…`), in the format of openidea-kiro,
  schema version 2: 54 terms in 2 topics. `kiro-cli vocab validate` of kiro-cli 1.1.0 reports no
  finding. The research program compiles the file into itself.
- The six reworks of R200 as inputs: five step files, now naming metrics by term id, and the report
  page of R198. The inputs that held names of the Python project were removed after R200; its four
  receipts were made again from the drawing in R200's set, which holds numbers in place of names.
- The section skeleton was made with the COBESY skill (composition back-end); the colours of the
  bars were checked with the palette validator of the dataviz skill (`--ordinal`, light and dark).
- The owner chose from a menu: the ontology is `vocabulary.toml` in openidea-kiro's format; only
  the set page and its PDF take the new terms in this spec; one before-and-after change is a
  "Refactoring"; the groups are Simplicity, Safety, Cost, in that order.

## Evidence and results

The four looks of the owner, in order:

| Look | What was shown | The owner's words |
| --- | --- | --- |
| 1 | R200's PDFs, 14 sheets: side-by-side table of six lenses, tiles, tables | "isn't comprehensible at all. it's a data dump"; the language is not suited for programmers; must be grounded in an ontology |
| 2 | Title sentence, summary in three lines, results table, a block per refactoring, four appendices; 17 sheets | "no": project name, author, tool name and version, horizontal bar charts with the metrics and a summary opener with the key numbers are missing |
| 3 | Head with projects, author, tool and date; six tiles; bars before and after; 19 sheets | "much better"; missing up front: revision (git hash), branches; the report is wanted per run, per project and across projects |
| 4 | Branch, commits and uncommitted changes up front; the same page at three widths | "ok, not all projects can get simplified. that is natural. good structure." |

What the accepted reports are, in `out-restructure\`:

| Report | Set | PDF | Sheets |
| --- | --- | --- | --- |
| across projects, 6 refactorings in 3 projects | `across-projects.json` (`e82e5cc7c29ae0ed…`) | `across-projects.pdf` (`6a0a23442d7307d7…`) | 19 |
| one project, 3 refactorings | `project-drift-rdp.json` (`3ea73a22e63be09c…`) | `project-drift-rdp.pdf` (`35a73bf0d8d4d98e…`) | 15 |
| one run | `run-display-path.json` (`0f625f272f0df4d4…`) | `run-display-path.pdf` (`d67477f3f92da58d…`) | 10 |

After the fourth look one thing was changed, on the owner's remark that not simplifying is natural: the
mark beside "mixed" is neutral, no longer the colour of a warning. The page and the three PDFs were
written again with it; the hashes above are of those. The sets did not change.

Each of the three pages was opened in a headless Chromium (Playwright of `verification/`) in the
print layout and drew without a script error.

What the rebuilt first sheet says of the six refactorings, from the same numbers as R200: simplicity
improved in 4 and mixed in 2; tests pass at both revisions in 5 of 6; 1 accepted by the tool, 5 not
judged; 18 of 36 values are reported and not checked; in 9 of 27 functions with mutation results no
mutant was killed before the refactoring. R200's first sheet said none of this.

The full account of the diagnosis, the skeleton, the decisions and the three rebuilds is the
[proposal](../../../.kiro/specs/report-set/report-structure-amendment.md).

## Limits and interpretation

- One reader judged, the owner. No programmer who does not know lesscode has read the report; the
  proposal's own condition of failure (such a reader cannot name verdict, what got simpler and the
  evidence of each refactoring from the first sheet) is untested.
- The gate's checks were not run on the new page: the browser checks of `checks\set.spec.mjs`, the
  search for names of the anonymized project in set, page and PDF, the PDF on Linux. They expect the
  earlier wording. G3, G4 and G5 are therefore shown for R200's page, not for this one.
- Five definitions in the vocabulary are written from the tool's output and not checked against the
  code that computes the number: state write sites, lock-guarded fields, branches per function,
  function length, mutants not judged. They are marked "verify" in the file.
- Three measures of the hand-written step files have no term and are shown as custom metrics.
- The sums under the summary add a metric over the refactorings that have it. They are sums of
  different scopes, in two cases of different projects; they show direction, not a quantity of one
  code base.
- The PDF grew from 14 to 19 sheets. The body is shorter; the appendices repeat every row and add
  the glossary.
- The author of the reports is stated by hand in the notes file. The tool that wrote the sets is the
  research program, not lesscode.

## Tool/spec consequence and next gate

- The spec's requirements 4.3 and 4.4 (side-by-side table of lenses; each step shown as the step
  report shows it today) are contradicted and are to be replaced. New requirements are needed for:
  the vocabulary and the glossary; metrics named by term in a set; the head (projects, author, tool
  and version, date) and the code a report is about (branch, both commits, uncommitted changes); the
  headline numbers and the bars; the three widths; an anonymized project withholding its repository
  and its branch.
- Decision 2 of the owner keeps the run page, `step check`, the README and the documents in the
  earlier words; they follow in a separate spec. The report of one run is a set of one refactoring
  made from the run's report page.
- Open for the design: where a report of one run takes the project's name and the refactoring's
  title from; today both come from the notes file.
- The owner's last remark is a rule for the wording: a refactoring that does not simplify is a
  finding, not a failure. "mixed" and "unchanged" are stated as plainly as "improved".
- Next gate: the owner's approval of the amended requirements, then design, then tasks. The gate's
  checks are brought to the new page as part of task group 2.

## COBESY check

Outcome first: yes, the owner's acceptance and what it permits. Replay: `look.ps1` makes the three
reports again from the inputs kept in the prototype's directory. Disproof: the owner's "no", given
twice and recorded. Negative results: the first rebuilt version was rejected and is in the table.
Facts and interpretation are apart; what was not checked again is listed.
