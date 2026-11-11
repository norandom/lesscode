# R83: Annotation layer: Opus runs agree with each other on 0.69 to 0.92 of their sites; three of five Mistral runs give no annotations

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 09:10 to 09:14 local (`claude-opus-5-5`) and 09:10 to 09:38
  (`mistral-large-4`); times from the runs' output
- Status: completed
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R82](07-slice-b-annotation-baseline-preregistration.md)
- Type: confirmatory baseline, two models, five runs per scope

## Decision

The annotation layer drifts in two different ways, and they need different answers.

On `claude-opus-5-5` every run gives usable annotations, and two runs on the same file agree on between 0.69
and 0.92 of the sites they name (median per scope). The sites named in all five runs are between 44% and 85%
of the sites named in any. A run that relies on one annotation call therefore tests a selection that the
next call would not repeat exactly. That is drift in what is tried, not in soundness: an annotation the
validator rejects never becomes a mutant.

On `mistral-large-4` the first problem is not agreement but getting an answer: 15 of 25 calls used the whole
output limit of 32,000 tokens without a readable list. Where an answer came, it was as valid as Opus's and,
for the two scopes with more than one answer, as stable. The two models choose differently: an Opus run and
a Mistral run agree on 0.28 to 0.72 of their sites, less than Opus runs among themselves in every scope.

For the tool this argues for what the driver already does by default: mutate every eligible site and use
annotations only to select. Where annotations are used to save time, the stable part is the sites several
runs agree on.

Separate outcomes: which sites a model names. Whether they are the important ones, and what the projects'
tests do with them, was not measured here.

## Hypothesis and falsification

| Claim of R82 | Result | Verdict |
| --- | --- | --- |
| A1: Opus, at least 80% of annotations accepted in every run | 23 of 25 runs; two at 76% and 75% | refuted |
| A2: Opus, median agreement between runs at least 0.50 in every scope | 0.69 to 0.92 | holds |
| A3: agreement between an Opus run and a Mistral run is lower than among Opus runs in at least four of five scopes | lower in five of five | holds |
| A4: Mistral, at least 80% accepted in at least three of five scopes (median) | four of five, but three of those rest on a single run | holds, weakly |

## Frozen inputs and method

As R82: guide `guide-v2.md`, the five gate scopes, one call per scope and run, validator from commit
`eea0109`. Results: `docs/experiments/annotation-driver/results/slice-b/annot-opus/` and `annot-mistral/`
(`summary.json` in each; the Opus summary holds the comparison).

One deviation. `annotate.py` could not read a reply that arrives as a list of parts, as `mistral-large-4`
sends it. Seventeen calls of the Mistral arm were made and their responses saved, but nothing after that was
written, including the ledger line. The reader was corrected while the arm was running; the last eight calls
were read normally, and the seventeen were finished from their saved responses by
`slice-b/salvage_annotations.py`, without a new call. Each such run is marked in its `call.json`. Until that
step about one dollar of spend was not in any ledger.

## Evidence and results

`claude-opus-5-5`, five runs per scope:

| Scope | Annotations per run | Accepted, lowest to highest | Median agreement between runs | In every run / in any run | Eligible sites |
| --- | --- | --- | --- | --- | --- |
| Rust `update.rs` | 23 to 30 | 76% to 96% | 0.74 | 15 / 26 | 71 |
| Python `ratelimit.py` | 54 to 63 | 91% to 94% | 0.88 | 48 / 61 | 211 |
| TypeScript `state.ts` | 37 to 40 | 97% to 100% | 0.92 | 34 / 40 | 70 |
| TypeScript `verification.ts` | 16 to 26 | 75% to 89% | 0.69 | 11 / 25 | 64 |
| Java `Lexer.java` | 30 to 33 | 94% to 100% | 0.82 | 24 / 35 | 184 |

`mistral-large-4`, five calls per scope:

| Scope | Runs with annotations | Annotations per run | Accepted | Median agreement between runs | Agreement with Opus runs (median, range) |
| --- | --- | --- | --- | --- | --- |
| Rust `update.rs` | 4 of 5 | 29 to 45 | 76% to 84% | 0.75 | 0.67 (0.48 to 1.00) |
| Python `ratelimit.py` | 1 of 5 | 92 | 92% | one run | 0.60 (0.56 to 0.64) |
| TypeScript `state.ts` | 3 of 5 | 43 to 53 | 96% to 100% | 0.81 | 0.72 (0.64 to 0.82) |
| TypeScript `verification.ts` | 1 of 5 | 49 | 82% | one run | 0.45 (0.28 to 0.55) |
| Java `Lexer.java` | 1 of 5 | 98 | 84% | one run | 0.28 (0.26 to 0.28) |

- Rejections on both models are a kind that does not fit the named code and, less often, an anchor that is
  not in the file. No annotation was refused for anything else, and no call was refused by a model.
- Mistral names more sites: 82 accepted in `Lexer.java` against 29 to 33 for Opus, 85 against 51 to 58 in
  `ratelimit.py`. Most of the low agreement between the models is that difference in how much is selected.
- The 15 Mistral calls without annotations all ended at the output limit (`length`); none was a refusal.
- Cost at list prices: Opus 2.64 USD (25 calls), Mistral 1.56 USD (25 calls, 696,000 output tokens, nearly all
  reasoning). Recorded spend after this baseline: about 29.7 of 40 USD.

Compared with the two Opus runs per scope of the gate (R64), re-checked with the same tooling: validity 0.81
to 1.00 then, 0.75 to 1.00 now; agreement 0.55 to 0.94 then, medians of 0.69 to 0.92 now. Five runs look like
two did.

## Limits and interpretation

- Agreement is counted on accepted sites only, so a run with many rejected annotations can still agree well.
- For three scopes there is one Mistral run; nothing can be said about Mistral's run-to-run agreement there.
- The output limit differs between the arms (16,000 and 32,000 tokens), as it must for a model that counts
  its reasoning against it; even 32,000 was not enough in 15 of 25 calls. A higher limit was not tried.
- The guide was written against Opus.
- Whether sites named in every run are better targets than sites named once was not tested.

## Tool/spec consequence and next gate

- The default stays: without annotations the driver mutates every eligible site, which does not drift.
- If annotations are used, two cheap hardenings follow from the numbers and are the owner's to choose:
  take the sites that at least two of three runs name, or take the union of several runs. Neither needs an
  optimiser. Both would be preregistered before use.
- `annotate.py` reads list-shaped replies now. The salvage script stays as the record of how the seventeen
  runs were finished.
- Slice B after R66 to R83: drift is measured for pack authoring and for annotation, between runs and
  between models, and under two optimisers for pack authoring. Not measured: one model over time.

## COBESY check

Outcome first: one claim refuted, three hold, one of them on thin data and said so. Replay: guide, scopes,
caller, validator, result directories and the one deviation with its repair named. Disproof: thresholds of
R82 applied as written. The unledgered spend during the run is reported.
