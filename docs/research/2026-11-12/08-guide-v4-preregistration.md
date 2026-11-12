# R101: Preregistration. A guide that describes the call-argument kind, judged by findings kept

- Report date/time and timezone: 2026-11-12, 16:59, Europe/Berlin (UTC+1)
- Execution interval: not run at the time of writing
- Status: planned
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B, reopened for cost by owner decision of 2026-11-12
- Hypothesis ID and predecessor: [R100](07-admitted-packs-gate-effect.md) (admitted packs lose findings
  almost only in the call-argument kind, which guide version 3 does not describe);
  [R99](06-repair-gate-result.md) arm B4
- Type: preregistered; claims and thresholds below were written before the first call

## Decision

None yet. Owner position, 2026-11-12: the core and the driver are model agnostic, and the fence around a
model's output is needed because inference is not deterministic. This run tests one instance of that
arrangement: a gap found by the fence's measure is closed in the instruction, and the fence measures again.

Context for what a pack is: one data file per language, shipped with the tool for Java, Python, Rust and
TypeScript. A repository does not need its own pack. A model writes a pack only for a language the tool
does not ship, so this is the route to a fifth language, not a step in every project.

## Hypothesis and falsification

A finding is a mutant of the shipped pack that the tests of the gate scope did not notice (R100).

| ID | Claim | Refuted when |
| --- | --- | --- |
| G1 | At least 4 of 5 Python packs keep at least 15 of the 17 call-argument findings (version 3: 0 or 1 kept in 10 of 10 packs) | fewer than 4 packs |
| G2 | At least 4 of 5 TypeScript packs lack no finding (version 3: 3 of 10) | fewer than 4 packs |
| G3 | The longer guide does not cost admission: at least 18 of 20 admitted in the end (version 3: 20) | fewer than 18 |
| G4 | Java and Rust stay as they were: no pack lacks more than 2 findings | any pack lacks 3 or more |

## Frozen inputs and method

- Guide: `docs/experiments/annotation-driver/pack-guide-v4.md`. Version 3 with five added table rows and
  nothing else changed: `kinds.call_argument.nodes`, and `literal_swaps`, `empty_string` and
  `swap_first_two` under `operators.call_argument`. Written by hand, in words, without a node kind name of
  any grammar. 6,747 characters against 6,049.
- Model: `zai-glm-5-3`, reasoning level "high", output limit 64,000 tokens, endpoint from the repository's
  `.env`.
- Tool: `tools/guide-optimizer` at `9bd5cc9`: `--judge-only` with `--repairs 2 --node-kinds`, five trials
  per language; then `--gate-effect` on the trials, with the receipts of R81.
- Comparison: arm B4 of R99, same settings with guide version 3, as joined in R100.
- Results: `docs/experiments/model-diagnosis/results/c1-glm-guide-v4/`.

## Evidence and results

Not run.

## Limits and interpretation

- One arm of twenty trials; the comparison arm ran earlier the same day.
- The example packs in the request contain the call-argument entries in both arms; only the guide differs.
- Extra mutants are counted, not classed (R100).
- The guide was revised by hand for a gap the measure showed. The guide optimiser's rules for a candidate
  guide (purpose, sections, length, no grammar names) are met; its search was not used.

## Tool/spec consequence and next gate

Written after the run, in its own entry.

## COBESY check

Claims and thresholds are stated before the first call, each with the figure from the earlier version
beside it. The one change between the arms is named.
