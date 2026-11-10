# R72: Slice B baseline on Mistral Large 4: one first reply in four is admitted; the barrier holds and repair helps for two languages

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, 21:43 to 21:57 local (869 seconds wall time, four conversations at a time); a first attempt at 21:39 to 21:42 crashed and is kept
  (Corrected 2026-11-10 from file and commit times; first written as "2026-11-10, about 23:00 to 23:15 local (869 seconds wall time, four conversations at a time);   a first attempt a few minutes earlier crashed and is kept", which was an estimate.)
- Status: completed for `mistral-large-4`
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: second model arm of [R66](12-slice-b-pack-authoring-drift-preregistration.md);
  compare [R69](15-slice-b-baseline-rerun-stronger-admission.md) for `claude-opus-5-5` under the same controls
- Type: confirmatory baseline

## Decision

With the same guide and the same checks, the second model writes far fewer admissible packs than the first:
5 of 20 first replies are admitted against 15 of 20. Changing the model is therefore the largest source of
"drift" measured so far, larger than run-to-run variation on one model. The admission barrier did what it
is for: nothing inadmissible got through, and no admitted pack was wrong in a way the controls cover.

Separate outcomes: pack authoring on one endpoint, one evening. No project's tests were run against these
packs.

## Hypothesis and falsification

Thresholds of R66, unchanged:

| Claim | Rust | Python | TypeScript | Java |
| --- | --- | --- | --- | --- |
| H1: at least 4 of 5 first replies admitted | 0 of 5, refuted | 1 of 5, refuted | 4 of 5 | 0 of 5, refuted |
| H2: at least 4 of 5 admitted packs give the shipped pack's mutants | none admitted | 1 of 3, refuted | 0 of 5, refuted | 5 of 5 |
| H3: rejected packs admitted after at most two repairs | 0 of 5, refuted | 2 of 4, refuted | 1 of 1 | 5 of 5 |

For comparison, `claude-opus-5-5` in R69: first replies admitted 0, 5, 5 and 5 of 5 for Rust, Python,
TypeScript and Java.

## Frozen inputs and method

R66 with Amendments 1 to 3: guide `pack-guide-v3.md`, the shipped packs as examples, admission with every
control set, at most two repairs, five runs per language. Endpoint: Mistral's chat completions API, model
`mistral-large-4` (requested and returned), price 0.70 and 2.10 USD per million tokens from Mistral's API
pricing page. Results: `docs/experiments/annotation-driver/results/slice-b/mistral/`.

Two deviations, both forced by how this model answers:

- It returns its reasoning as a separate part of the reply and counts it against the output limit. With the
  limit of 4,000 tokens used for the first model, every reply consisted of reasoning only and was cut off.
  The first attempt crashed on the multi-part reply (`results/slice-b/mistral-attempt1-crashed/`, 11 calls).
  The runner now reads the text parts, and this arm ran with an output limit of 16,000 tokens.
- The admission binary was the one used in R69, so the repair feedback for a missing operator was still the
  uninformative one that R69 describes.

## Evidence and results

| Language | First reply admitted | Admitted in the end | Repair replies | Same mutants as shipped | Outcomes |
| --- | --- | --- | --- | --- | --- |
| Java | 0 of 5 | 5 | 5 | 5 of 5 | admitted 5 |
| Python | 1 of 5 | 3 | 2 | 1 of 3 | admitted 3, empty reply 2 |
| TypeScript | 4 of 5 | 5 | 1 | 0 of 5 | admitted 5 |
| Rust | 0 of 5 | 0 | 5 | none | empty reply 4, rejected after repairs 1 |

- The commonest failure is a node kind that does not exist in the grammar (`text_block` and
  `switch_statement` for Java, `comparison_expression` for Python, `method_call_expression` and `match_guard`
  for Rust): the model writes plausible names without knowing the grammar. The `kinds_valid` check names
  them, and for Java one repair turn fixed every run.
- "Empty reply" means the reasoning used the whole output limit of 16,000 tokens and no pack was written.
  This happened in six of twenty runs, four of them for Rust.
- The five admitted TypeScript packs are five different texts, and none gives the shipped pack's mutants.
- Tokens: 81,530 input and 317,824 output in 33 calls, almost all of the output being reasoning. Cost 0.73
  USD; the crashed attempt cost about 0.10 USD.

## Limits and interpretation

- The output limit differs between the two model arms (4,000 and 16,000 tokens). Without the change this arm
  would have had no result at all; with it the arms differ in one more parameter than the model.
- The guide was written and revised against the first model. It names no node kinds on purpose; a model
  that does not know the grammar gets no help from it.
- Five runs per language; one evening.
- What the TypeScript packs do differently from the shipped pack was not analysed.

## Tool/spec consequence and next gate

- The optimisation of R70 is running on this model with the same parameters and an output limit of 16,000
  tokens for authoring and 24,000 for reflection; its result goes into its own entry.
- For a model without knowledge of the grammar, the cheapest help is likely not a better instruction but the
  list of the grammar's node kinds, which the driver can produce. That is a design idea for the owner, not
  something done here.

## COBESY check

Outcome first: the numbers per language against the preregistered thresholds, and the comparison with the
first model. Replay: endpoint, model, guide, limits and results named, including the crashed attempt.
Disproof: thresholds applied as written. Both deviations from the protocol are stated with their reason.
