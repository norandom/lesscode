# R96: Preregistration. Why the second model writes fewer admissible packs: reasoning level, output limit or knowledge of the grammar

- Report date/time and timezone: 2026-11-12, 10:25, Europe/Berlin (UTC+1)
- Execution interval: not run at the time of writing
- Status: planned
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B (closed 2026-11-11). Reopened for this question by owner decision of 2026-11-12: the
  model-driven loops must work on a model that is affordable in pipelines and agents; Opus is not
- Hypothesis ID and predecessor: [R72](../2026-11-10/18-slice-b-baseline-mistral.md) (Mistral Large 4,
  5 of 20 first replies admitted), [R69](../2026-11-10/15-slice-b-baseline-rerun-stronger-admission.md)
  (Opus, 15 of 20)
- Type: preregistered; hypotheses and thresholds below were written before the first call

## Decision

None yet. The result decides what the DSPy-style loops have to change for a cheaper model, before a spec
lets a model propose simplifications.

## Hypothesis and falsification

The owner's point: R72 never set the reasoning level, and Mistral has to run at "high". R72 also ran with
an output limit of 16,000 tokens, which the model's reasoning used up in six of twenty runs.

| ID | Claim | Refuted when |
| --- | --- | --- |
| H1 | Replication. Mistral Large 4 at its default level and 16,000 tokens gives 3 to 7 admitted first replies of 20, as in R72 (5) | fewer than 3 or more than 7: the model, the endpoint or the judge differs from R72 |
| H2 | The empty replies were the output limit. At 64,000 tokens at most 1 of 20 replies is empty or cut off | 2 or more |
| H3 | Reasoning level. At "high" and 64,000 tokens at least 10 of 20 first replies are admitted | fewer than 10 |
| H4 | What remains is knowledge of the grammar. Of the replies rejected at "high", at least half fail on a node kind the grammar does not have | fewer than half |
| H5 | GLM 5.3 at "high" and 64,000 tokens: at least 10 of 20 first replies admitted | fewer than 10 |

## Frozen inputs and method

- Tool: `tools/guide-optimizer` (Rust) in its `--judge-only` mode, at commit `e8e3785` (reasoning
  level from the endpoint file or the environment). One fresh authoring call per trial, judged by the
  admission checks; no repair turn. R72 used the Python prototype of the same baseline; H1 tests whether
  the two agree.
- Guide: `docs/experiments/annotation-driver/pack-guide-v3.md`, sha256 `dc49e14bb0d6…`, unchanged.
- Trials: five per language, Rust, Python, TypeScript, Java: 20 per arm.
- Endpoint: the owner's endpoint from the repository's `.env` (`LESSCODE_MODEL_BASE_URL`); the model and
  the reasoning level are set per arm in the environment of the run.

| Arm | Model | Reasoning level | Output limit |
| --- | --- | --- | --- |
| A1 | `mistral-large-4` | not sent | 16,000 |
| A2 | `mistral-large-4` | not sent | 64,000 |
| A3 | `mistral-large-4` | high | 64,000 |
| A4 | `zai-glm-5-3` | high | 64,000 |

- Opus is not called. Its numbers come from R69.
- Spend: the owner set no limit for Mistral and GLM in this development work. Results are written under
  `docs/experiments/model-diagnosis/results/`, outside the tree whose ledgers count against the 40 USD
  stop for Opus. No list price for GLM is known here; tokens are reported, not a cost.
- Recorded per trial: admitted or not, feedback of the admission, stop reason, input and output tokens,
  the reply.

## Evidence and results

Not run at the time of writing.

Amendment, 2026-11-12, 13:07: run on the same day, 10:24 to 13:06. The result, and an amendment for calls
the endpoint refused, is in [R97](04-model-diagnosis-result.md). The text above is unchanged.

## Limits and interpretation

- Twenty trials per arm, one day, one endpoint that sits in front of the vendors' APIs.
- The judge makes no repair turn, so "admitted in the end" of R72 has no counterpart here.
- A2 and A3 differ in the reasoning level only; A1 and A2 in the output limit only. A4 differs from A3 in
  the model.
- The tool records output tokens, not reasoning tokens apart.

## Tool/spec consequence and next gate

Written after the run, in its own entry.

## COBESY check

Thresholds are stated before the first call; each arm changes one thing against its neighbour; what the
tool cannot record is named.
