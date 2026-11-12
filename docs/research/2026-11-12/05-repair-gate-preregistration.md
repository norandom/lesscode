# R98: Preregistration. Packs admitted in the end with repair turns, at "high" and 64,000 tokens, with and without the node kind list

- Report date/time and timezone: 2026-11-12, 13:31, Europe/Berlin (UTC+1)
  (Corrected 2026-11-12 from the commit time; first written as 14:25, an estimate. A first start at 13:31
  failed on the run script's arguments before any call was made.)
- Execution interval: not run at the time of writing
- Status: planned
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B, reopened for cost by owner decision of 2026-11-12
- Hypothesis ID and predecessor: [R97](04-model-diagnosis-result.md) with its amendment;
  [R78](../2026-11-11/04-slice-b-node-kinds-result.md) (list tested at 16,000 tokens, default level; repair
  gives 15 of 20 in the end); [R72](../2026-11-10/18-slice-b-baseline-mistral.md)
- Type: preregistered; hypotheses and thresholds below were written before the first call

## Decision

None yet. The result says whether an affordable model with repair turns reaches the number of admitted
packs that matters for a pipeline, what that costs against Opus, and whether the node kind list earns a
place in the request once the output limit no longer binds (the question R78 left open).

## Hypothesis and falsification

The measure is packs admitted in the end, not first replies. Opus for comparison (R78): 20 of 20 in the
end, 25 calls, 1.41 USD, which is 0.07 USD per admitted pack.

| ID | Claim | Refuted when |
| --- | --- | --- |
| Q1 | Mistral Large 4 without the list ends with at least 16 of 20 admitted (R78: 15 at the default level and 16,000 tokens) | fewer than 16 |
| Q2 | Without the limit binding, runs no longer end without a pack: at most 1 of 20 for Mistral without the list (R78: 5) | 2 or more |
| Q3 | Replication of R78's K3: with the list, at most 2 of 20 first replies of Mistral name a node kind the grammar lacks | 3 or more |
| Q4 | The list does not change the end result for Mistral: the two arms differ by at most 2 admitted packs | 3 or more, in either direction |
| Q5 | GLM 5.3 without the list ends with at least 16 of 20 admitted | fewer than 16 |
| Q6 | Mistral without the list costs at most 0.07 USD per admitted pack at the list price of R72 (0.70 and 2.10 USD per million tokens), the level of Opus | more than 0.07 USD |

Q4 is stated as "no difference" because that is what R78 found. If the list helps by 3 or more, R78's
decision not to adopt it is overturned for this setting.

## Frozen inputs and method

- Tool: `tools/guide-optimizer` at commit `9a55007`, `--judge-only` with `--repairs 2` and, in the list
  arms, `--node-kinds`. The tool waits out a rate limit (`a913de7`).
- A repair here is a fresh call: the first request, the rejected pack and the admission's reasons. Where no
  pack arrived, the request is sent again. R72 and R78 repaired inside one conversation with the Python
  runner; the two are not the same procedure, and Q1 compares across that difference.
- The list: the named, visible node kinds of the grammar the tool ships, appended to the request as one
  paragraph. Same code for every language.
- Guide: `docs/experiments/annotation-driver/pack-guide-v3.md`, sha256 `dc49e14bb0d6…`, unchanged.
- Trials: five per language, Rust, Python, TypeScript, Java: 20 per arm. Reasoning level "high", output
  limit 64,000 tokens, in every arm.
- Endpoint: the owner's endpoint from the repository's `.env`; model and level set per arm in the
  environment of the run. The two Mistral arms run one after the other; the two GLM arms run beside them.

| Arm | Model | List |
| --- | --- | --- |
| B1 | `mistral-large-4` | no |
| B2 | `mistral-large-4` | yes |
| B3 | `zai-glm-5-3` | no |
| B4 | `zai-glm-5-3` | yes |

- Opus is not called.
- Recorded per trial: every attempt with admission, stop reason, tokens and feedback; the final reply.
- Results: `docs/experiments/model-diagnosis/results/b1…b4`.

## Evidence and results

Not run.

## Limits and interpretation

- Twenty trials per arm; R68 and R69 put run-to-run variation at one or two trials.
- No list price for GLM is known here; tokens are reported, not a cost. The cost of Q6 is computed from
  recorded tokens at a list price, not from an invoice of the endpoint.
- The Opus figures are from another day, another runner and repair inside a conversation.
- Admission is the barrier, not a measure of a pack's quality. How many admitted packs give the shipped
  pack's mutants is recorded and reported, with no threshold.
- GLM gets no claim for the list; its two arms are reported.

## Tool/spec consequence and next gate

Written after the run, in its own entry.

## COBESY check

Thresholds are stated before the first call. The earlier results that argue against the list are named,
and the claim about it is written so that either outcome is informative. What differs from the runs it is
compared with is listed.
