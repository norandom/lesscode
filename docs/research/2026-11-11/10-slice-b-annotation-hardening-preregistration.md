# R84: Preregistration: do rules over three annotation runs select more stably than one run?

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: not yet run; written before any call of this experiment
- Status: preregistered
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: the two hardenings named in
  [R83](09-slice-b-annotation-baseline.md)
- Type: preregistration of a confirmatory experiment on one model

## Decision

Owner decision, 2026-11-11: harden annotations. R83 found that two Opus runs on one file agree on 0.69 to
0.92 of the sites they name. Two rules that need no optimiser were named there: keep the sites that at least
two of three runs name (majority), or keep every site any of three runs names (union). This entry fixes how
they are tested, on fresh runs.

## Hypothesis and falsification

Six fresh annotation runs per scope on `claude-opus-5-5`. Every split of the six into two groups of three
(ten splits) gives two selections per rule. Agreement between the two selections is the number of sites in
both divided by the number in either. For comparison: agreement between single runs (fifteen pairs).
Medians per scope.

| Claim | Statement | Refuted when |
| --- | --- | --- |
| H1 | Majority selections agree with each other more than single runs do, in at least four of five scopes | fewer than four |
| H2 | The median agreement of majority selections is at least 0.80 in at least four of five scopes | fewer than four |
| H3 | Union selections agree with each other more than single runs do, in at least four of five scopes | fewer than four |

Descriptive: the number of sites each rule keeps, against a single run and against the eligible sites. Cost
per file is three calls instead of one for either rule.

What the result will not show: whether the sites a rule keeps are the ones that matter, or what the
projects' tests do with them; whether the rules help a model that often gives no answer.

## Frozen inputs and method

- As R82 and R83: guide `guide-v2.md`, the five gate scopes, one call per run, output limit 16,000 tokens,
  caller `annotate.py` (with the reply reader corrected in R83), validator `annotation_sites` from commit
  `eea0109`. The validator's packs are those of that commit; the later removal of the argument swap in two
  packs does not concern the four kinds an annotation can name.
- Analysis: `slice-b/annotate_baseline.py ensemble`, written before the runs.
- The five runs per scope of R83 are not used for the claims; they were seen before the rules were chosen
  for testing.
- Budget: expected about 3.2 USD. Recorded before: about 29.7 of 40 USD.
- A refusal or a run without annotations is an outcome; a scope with fewer than six usable runs is reported
  and its claims are not evaluated.

## Evidence and results

None yet. Results go into their own entry.

## COBESY check

Outcome first: nothing measured; three claims with thresholds. Replay: inputs, analysis and budget named.
Disproof: each claim can fail. The earlier runs are kept out of the claims and that is stated.
