# R78: The node kind list removes invalid node kinds but does not raise admission on Mistral Large 4

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 07:43 to 07:58 local (`mistral-large-4`, both arms side by side) and 08:00 to
  08:04 (`claude-opus-5-5`, both arms side by side); times from the result files
- Status: completed
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R76](02-slice-b-node-kinds-preregistration.md)
- Type: confirmatory comparison, two arms on each of two models

## Decision

Giving the pack author the grammar's node kinds does what it was meant to do and does not solve the problem.
On `mistral-large-4`, first replies that name a node kind the grammar does not have fall from 10 of 20 to 1 of
20. First replies that are admitted stay where they were: 5 of 20 without the list, 6 of 20 with it. With the
list, more replies end at the output limit without a pack, and those that arrive fail the control cases
instead. On `claude-opus-5-5` the list changes nothing.

The list is therefore not adopted as part of the task's input. It is cheap and harmless for a model that
knows the grammars, and for one that does not it moves the failure from "wrong name" to "wrong behaviour"
or "no answer in the space given".

Separate outcomes: pack authoring only. No project's tests were run against these packs.

## Hypothesis and falsification

| Claim of R76 | Result | Verdict |
| --- | --- | --- |
| K1: Mistral with the list, at least 12 of 20 first replies admitted | 6 of 20 | refuted |
| K2: Mistral, at least 6 more than the control arm | 6 against 5, a difference of 1 | refuted |
| K3: Mistral with the list, at most 2 of 20 first replies fail `kinds_valid` | 1 of 20 (control arm: 10 of 20) | holds |
| K4: Opus with the list, no fewer first replies admitted than the control arm minus 1 | 15 against 15 | holds |

## Frozen inputs and method

As R76: runner `author_baseline.py` with `--kinds-dir`, guide `pack-guide-v3.md`, the shipped packs of `main`
as examples, admission binary from commit `964921c`, five runs per language and arm, at most two repairs,
output limit 16,000 tokens for Mistral and 4,000 for Opus. Results:
`docs/experiments/annotation-driver/results/slice-b/kinds-{mistral,opus}-{control,treatment}/`.

## Evidence and results

First replies admitted, of 5 per language:

| Model | Arm | Java | Python | Rust | TypeScript | Total |
| --- | --- | --- | --- | --- | --- | --- |
| `mistral-large-4` | without the list | 1 | 1 | 0 | 3 | 5 of 20 |
| `mistral-large-4` | with the list | 2 | 1 | 0 | 3 | 6 of 20 |
| `claude-opus-5-5` | without the list | 5 | 5 | 0 | 5 | 15 of 20 |
| `claude-opus-5-5` | with the list | 5 | 5 | 0 | 5 | 15 of 20 |

What happened to the first reply on `mistral-large-4`:

| | Without the list | With the list |
| --- | --- | --- |
| Admitted | 5 | 6 |
| Named a node kind the grammar does not have | 10 | 1 |
| Ended at the output limit | 4 | 9 |
| Empty first reply, where the run ended | 4 | 8 |
| Runs that ended with an empty reply, at any attempt | 5 | 11 |

Admitted in the end, after at most two repairs:

| Model | Arm | Java | Python | Rust | TypeScript | Total | Equal to the shipped pack's mutants |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `mistral-large-4` | without the list | 4 | 5 | 2 | 4 | 15 of 20 | 7 |
| `mistral-large-4` | with the list | 3 | 1 | 2 | 3 | 9 of 20 | 3 |
| `claude-opus-5-5` | without the list | 5 | 5 | 5 | 5 | 20 of 20 | 15 |
| `claude-opus-5-5` | with the list | 5 | 5 | 5 | 5 | 20 of 20 | 14 |

- With the list, Mistral's remaining first-reply failures are control cases and missing operators (Java,
  TypeScript, Rust) and replies cut off at the limit (Python: four of five runs gave no pack).
- No call was refused on either model.
- Cost at list prices: Mistral 0.74 and 0.70 USD (32 and 28 calls), Opus 1.41 and 1.61 USD (25 calls each).
  Recorded spend after these runs: about 19.7 of 40 USD.

Two observations outside the claims:

- Mistral's control arm ends with 15 of 20 admitted; R72 ended with 13 of 20. Rust went from none to two.
  The admission feedback now names the failing control and the operators expected and found (changed after
  R69); the examples also changed. The two cannot be separated from these runs.
- On Opus, Rust is still never admitted on the first reply and is now admitted after one repair in ten of
  ten runs across both arms. In R69 repair worked twice in five. The same two changes apply.

## Limits and interpretation

- Five runs per language and arm; one morning. A difference of one or two runs is within what repetition
  alone produces (R68, R69).
- The list was appended to the request as plain text. Whether a shorter list, or the kinds grouped by what
  they are, would cost the model less reasoning was not tried.
- The output limit of 16,000 tokens binds for this model: 9 of 20 first replies with the list ended there.
  A higher limit might turn some of them into packs; it would also raise the cost per call, which is almost
  all reasoning. That is a separate question with its own preregistration.
- The fall in final admission with the list (15 to 9 on Mistral) is mostly runs that never produced a pack
  and so had nothing to repair.

## Tool/spec consequence and next gate

- The pack-authoring request stays as it is. The lister (`native/examples/grammar_kinds.rs`) and the option
  stay as research tooling.
- For a model that does not know the grammars, neither a better instruction (R74) nor the list of names
  (this entry) gives admissible first replies. What does work for it is the repair turn with precise
  feedback: 15 of 20 in the end. The barrier held in every arm: nothing inadmissible was admitted.
- Owner decision open: whether pack authoring by such a model is worth pursuing further (a higher output
  limit, a grouped list), or whether the finding "use a model that knows the grammar, and keep the barrier"
  is enough for Slice B.

## COBESY check

Outcome first: two claims refuted, two hold, with the numbers. Replay: runner, option, lists, binary, limits
and result directories named. Disproof: thresholds applied as written in R76; the one claim about the
targeted failure holds and is not presented as success of the idea. Observations outside the claims are
marked as such, with the reason they cannot be attributed.
