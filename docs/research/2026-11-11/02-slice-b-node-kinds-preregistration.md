# R76: Preregistration: does giving the pack author the grammar's node kinds raise admission?

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: not yet run; written before any call
- Status: preregistered
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: follows the idea noted in
  [R72](../2026-11-10/18-slice-b-baseline-mistral.md) and supported by
  [R74](../2026-11-10/20-slice-b-guide-optimisation-mistral.md); protocol of
  [R66](../2026-11-10/12-slice-b-pack-authoring-drift-preregistration.md) with Amendments 1 to 3
- Type: preregistration of a confirmatory comparison (two arms, two models)

## Decision

Owner decision, 2026-11-11: try giving the pack author the grammar's node kind list, on both models, and
raise the stop for model spend from 20 to 40 USD. This entry fixes the comparison before it is run.

## Hypothesis and falsification

R72 found that a model that does not know the grammars writes node kinds that do not exist, and R74 found
that rewording the instruction does not help. The driver can list a grammar's node kinds. The claim is that
this list, added to the request, removes most of those failures.

Each model is run in two arms on the same day, with the same binary, guide, examples and controls: a control
arm without the list, and a treatment arm with it. Five runs per language and arm.

| Claim | Statement | Refuted when |
| --- | --- | --- |
| K1 | On `mistral-large-4`, the treatment arm has at least 12 of 20 first replies admitted | fewer than 12 |
| K2 | On `mistral-large-4`, the treatment arm has at least 6 more first replies admitted than the control arm | a difference below 6 |
| K3 | On `mistral-large-4`, at most 2 of 20 first replies in the treatment arm fail the check `kinds_valid` | 3 or more |
| K4 | On `claude-opus-5-5`, the treatment arm has no fewer first replies admitted than the control arm minus 1 | a drop of 2 or more |

Descriptive, without a threshold: replies that end at the output limit without a pack; admitted packs that
give the shipped pack's mutants; admission after repair. Repair outcomes are not compared with R69 or R72,
because the admission feedback has been improved since those runs.

What the result will not show: whether a pack written with the list is better at finding faults; whether the
effect holds for a grammar the model has never seen; anything about a third model.

## Frozen inputs and method

- Runner: `docs/experiments/annotation-driver/slice-b/author_baseline.py` with the new option `--kinds-dir`.
  In the treatment arm the first request's user message ends with one paragraph and the list:
  "These are the named node kinds of the LANGUAGE grammar that the pack will be checked against, one per
  line. Every node kind you name in the pack must be one of them."
- Lists: `docs/experiments/annotation-driver/slice-b/grammar-kinds/{rust,python,typescript,java}.txt`, written
  by `native/examples/grammar_kinds.rs` from the grammars compiled into the binary (named, visible kinds: 163,
  123, 177 and 142). Every kind used by the shipped packs is in its list.
- Guide: `pack-guide-v3.md`, unchanged (SHA-256 `dc49e14b...`). Examples: the shipped packs as they are on
  `main` today. They differ from those of R69 and R72 by the `gaps` key, which is why each model gets its own
  control arm instead of being compared with the earlier runs.
- Admission binary: `lesscode-core` built from commit `964921c`. Control sets as in R66 Amendment 3.
- Output limits: 4,000 tokens for `claude-opus-5-5` and 16,000 for `mistral-large-4`, as in R69 and R72. At
  most two repairs.
- Endpoints and list prices as in R66 Amendment 3 and R72.
- Budget: expected about 1.5 USD on `mistral-large-4` and about 3.5 USD on `claude-opus-5-5` for both arms.
  Recorded spend before this run: about 15.2 USD; stop at 40 USD.
- A refusal by a model is an outcome and is not reworded.

## Evidence and results

None yet. Results go into their own entry.

## COBESY check

Outcome first: nothing measured; the thresholds are fixed. Replay: runner, option, lists, guide, binary and
limits named. Disproof: four claims with numbers that the runs can miss. The known difference from the
earlier runs (examples with the `gaps` key, better repair feedback) is handled by same-day control arms and
is stated.
