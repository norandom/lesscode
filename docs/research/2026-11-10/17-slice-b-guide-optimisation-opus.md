# R71: Guide optimisation with the GEPA engine on Opus: Rust admission goes from 0 of 5 to 5 of 5 and Java is stable again; the Rust packs still differ from the hand pack, so the guide is not adopted

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, 21:23 to 21:43 local for the optimisation and its first final trials; 21:48 to 21:49 for the trials that completed the final judgement
  (Corrected 2026-11-10 from file and commit times; first written as "2026-11-10, about 21:23 to 22:20 local for the optimisation and its first final trials;   about 22:45 for the ten trials that completed the final judgement", which was an estimate.)
- Status: completed for `claude-opus-5-5`
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R70](16-slice-b-guide-optimisation-preregistration.md) with Amendments 1 to 3
- Type: confirmatory (one optimisation run with a fixed acceptance rule)

## Decision

The optimised guide is not adopted: the acceptance rule of R70 needs Rust packs that give the shipped pack's
mutants, and none of five did. What the run does show is that reflective optimisation of the guide, with
admission as the barrier, removes the instability found in R68 and R69 for this model: every first reply was
admitted in the final judgement, for all four languages, and Java, Python and TypeScript gave exactly the
shipped pack's mutants in every trial. The remaining Rust difference is the same in every trial except for
the spelling of the empty statement; it is a disagreement with the hand-written pack, not run-to-run drift.
Whether the hand pack or the model is right on those points is the owner's decision (see below).

Separate outcomes: stability of pack authoring under one guide, on one endpoint, on one evening. No project's
tests were run against these packs. No compression was measured.

## Hypothesis and falsification

Final judgement of the selected guide, five fresh trials per language (20 calls):

| Claim (R70) | Rust | Java | Python | TypeScript | Result |
| --- | --- | --- | --- | --- | --- |
| H1: at least 4 of 5 first replies admitted (Rust, Java) | 5 of 5 | 5 of 5 | | | holds |
| H2: at least 4 of 5 admitted with the shipped pack's mutants (Rust, Java) | 0 of 5 | 5 of 5 | | | refuted for Rust |
| H3: no regression on the languages kept apart, both measures | | | 5 of 5, 5 of 5 | 5 of 5, 5 of 5 | holds |

For comparison, guide version 3 in R69: Rust 0 of 5 admitted; Java 5 of 5 admitted, 2 of 5 with the shipped
pack's mutants; Python and TypeScript 5 of 5 on both.

## Frozen inputs and method

As in R70 Amendment 3: tool `tools/guide-optimizer/` at commit `7912e1c`, copied GEPA engine, model
`claude-opus-5-5`, starting guide `pack-guide-v3.md`, 60 metric calls, minibatch 3, seed 0. Results:
`docs/experiments/annotation-driver/results/slice-b/optimise1-opus/` and `optimise1-opus-final-topup/`.

Deviation: the run's budget stop (6 USD) was reached during the final judgement, after 11 of its 20 trials.
The nine trials that were not made were counted by the tool as failures and it printed "not passed"; that was
a defect of the tool. The final judgement was completed with the missing fresh trials of the same guide in a second
invocation, so that each language has exactly five. The tool
now keeps the final judgement's budget apart from the optimisation's and reports an unfinished judgement as
incomplete.

## Evidence and results

The engine ran 13 iterations and 62 metric calls and kept two candidates: the starting guide and one new
guide (8,139 characters against 6,049). Eight further proposals were discarded by the barrier, all for
length (9,186 to 10,640 characters against a limit of 9,073); none was discarded for naming a node kind or a
check.

Frontier, on the trials of the optimisation phase:

| Guide | Rust admitted | Rust distinct behaviours | Java admitted and equal to shipped | Output tokens per call, Rust / Java |
| --- | --- | --- | --- | --- |
| version 3 (start) | 0 of 3 | none admitted | 4 of 4 | 2,537 / 1,763 |
| candidate 1 | 23 of 23 | 2 | 23 of 23 | 2,583 / 1,716 |

What the new guide adds, in general terms: how to choose the empty statement in expression-oriented
languages, that every form of test marker must be listed, how failure signals that are ordinary calls are
recognised, and that every name must be checked against the grammar.

The Rust packs written under it, compared with the shipped pack (identical in all five final trials except
where noted):

- the empty statement is `()` in three trials and `{}` in two, where the shipped pack has `;`; this alone
  changes the text of 39 mutants;
- the `?` operator is rewritten by removing it, where the shipped pack replaces it by an unwrap;
- aborting macros get a drop operator, which the shipped pack does not have;
- a `match` guard is a condition; the list of aborting macros differs slightly.

Cost: 85 calls and 5.95 USD for the optimisation run (72 authoring calls with 290,127 input and 156,921
output tokens; 13 reflections with 70,543 and 68,666), and 0.54 USD for the ten completing trials. The
estimate printed before the run was too low: an authoring call under a longer guide cost about 0.06 USD and
a reflection about 0.13 USD. Recorded spend for the Anthropic endpoint is 12.38 of 20 USD at list prices; the Mistral runs come on top.

## Limits and interpretation

- "Equal to the shipped pack" punishes spelling. `()`, `{}` and `;` are three ways to write a statement that
  does nothing; the comparison by hash of the mutated text counts 39 differences for it. A comparison of
  behaviour would need the mutants to be built and tested, which is the batch runner's job.
- Removing `?` instead of unwrapping usually does not compile, so those mutants would be invalid. The control
  added in R69 checks that the operator exists, not what it writes.
- One optimisation run, one seed. The eight discarded proposals show the length limit was the binding
  constraint; a looser limit might have found a better guide or a bloated one.
- The languages kept apart were at the ceiling before, so they show absence of regression only.
- The observations the engine learned from came from Rust and Java, the same languages it was scored on.

## Tool/spec consequence and next gate

- The candidate guide stays in the results directory; `pack-guide-v3.md` remains the guide in use.
- Fixed in the tool: budget for the final judgement is held back; unfinished judgements are reported as
  incomplete; replies in several parts (a model that returns its reasoning separately) are read; a
  judge-only mode completes or repeats a judgement.
- For the owner: decide the four Rust points above on their merits (as the Python pairs were decided). With
  those decided, either the shipped Rust pack or the Rust controls change, and the same guide can be judged
  again for a few cents.
- Next: the same protocol on `mistral-large-4`.

## COBESY check

Outcome first: not adopted, by the rule fixed beforehand, with what improved stated separately from what was
required. Replay: tool, commit, seed, results and both result directories named. Disproof: thresholds
applied as preregistered; the tool's own wrong verdict on an unfinished judgement is reported and fixed. The
weakness of the yardstick is named instead of being used to rescue the result.
