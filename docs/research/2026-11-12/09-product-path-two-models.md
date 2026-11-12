# R102: The product's model path with two other models: three live turns, three different failures before any answer is used

- Report date/time and timezone: 2026-11-12, 17:20, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 17:04 to 17:20
- Status: complete as a probe; no hypothesis was preregistered
- Spec: none yet. Input for the next spec (model seam), by owner statement of 2026-11-12: packs are
  published once with the tool and are not regenerated; what is needed is that an adopter can run GLM or
  another model as the model behind the tool, with a model-agnostic core and driver and a fence around the
  model's output because inference is not deterministic
- Hypothesis ID and predecessor: gaps read in `native/src/regenerator.rs` after [R100](07-admitted-packs-gate-effect.md)
- Type: exploratory probe of existing code, three turns, written up the same hour

## Decision

`lesscode-core regenerate` works with the model it was built against and with no other model tried here.
The request it sends and the reply it accepts are fixed to one vendor's habits. Before a spec lets a model
propose changes for adopters, the model path needs its own requirements: one client, a check an adopter
can run, and the fence unchanged behind it.

## Hypothesis and falsification

None stated before the run. The four gaps had been read in the code and told to the owner before the
first turn: only the OpenAI-style request is sent; the reasoning level of the endpoint file is not sent;
the reply text must be a plain string; a rate limit is not waited out. The turns were run to see which of
them, or what else, stops a real call.

## Frozen inputs and method

- Binary: `lesscode-core` built from `37f3e1b` (version 1.2.0 with today's endpoint changes).
- Task: one turn of `regenerate run` on a ten-line Python function with three identical branches
  (`docs/experiments/model-seam/mod.py`, `request.json`), empty public surface, default parameters
  (64,000 output tokens, effort "high").
- Endpoints: the owner's local endpoint with `zai-glm-5-3` (repository `.env`), and Mistral's own API with
  `mistral-large-latest` (an endpoint file outside the repository).
- Script: `docs/experiments/model-seam/regen-probe.ps1`. Receipts beside it.

## Evidence and results

| Turn | Model, endpoint | Result | Reason in the receipt |
| --- | --- | --- | --- |
| t2 | GLM 5.3, local endpoint | failed after 652 s | HTTP 429, "Upstream rate limit persisted; retry after 1 seconds" |
| m1 | Mistral Large, Mistral's API | failed after 2 s | HTTP 400, "top_p must be 1 when using greedy sampling" |
| m2 | the same, `--temperature 0.7` | failed after 12 s | `KeyError: 'message'` |

- t2: the call is not repeated after a rate limit; the research tool waits (`a913de7`). The rate limit was
  partly of our own making: the guide run of R101 held four GLM calls at the time. A one-line request to
  the same model took 36 seconds at that moment.
- m1: the request always carries `temperature`, 0 when none is given. This endpoint refuses 0 for this model.
- m2: the model answered. The reply's text is a list of parts (reasoning and text), and the code takes
  only a plain string, so the answer was thrown away. The research tool reads both forms.
- In all three turns the command printed nothing; the exit code was 3 and the reason is only in the
  receipt. The reasons for t2 and m1 are readable because a failed call now carries the endpoint's message
  (`05eaec5`, this morning it would have been "HTTP 429" and "HTTP 400").
- Not reached: whether either model accepts the JSON schema response format, and what a returned
  candidate looks like. No turn got that far.

## Limits and interpretation

- Three turns, one task, one afternoon. A probe, not a measurement of either model.
- t2 was disturbed by a concurrent run of ours and must be repeated alone.
- The Anthropic style was not tried; by the code it is accepted in the endpoint file and not sent.

## Tool/spec consequence and next gate

Requirements to write, from this evidence:

1. One model client in the core, used by `regenerate` and by the research tooling: both request styles;
   optional parameters (temperature, reasoning level, response format) sent only when configured or known
   to be accepted; a reply's text read from a string or from parts; a rate limit waited out; the
   endpoint's message in every failure.
2. A failed turn says why on the terminal, not only in the receipt.
3. A check command for an adopter: after configuring an endpoint, one small fixed task through the same
   fence, with a plain result: this model can be used, or what it refused.
4. First cases to pass: GLM 5.3 on the owner's endpoint and Mistral Large on Mistral's API.
5. Unchanged: the core and the driver make no model call; whatever a model returns is used only after the
   checks.

## Amendment: three more turns, a third vendor, and the temperature error explained (2026-11-12, 17:28)

The owner asked about the Mistral error, pointed out that Mistral's API also offers GLM 5.3, and gave a
temporary key for OpenAI's API.

| Turn | Model, endpoint | Result | Reason in the receipt |
| --- | --- | --- | --- |
| g1 | GLM 5.3, Mistral's API | failed after 17 s | `KeyError: 'message'` |
| g2 | the same, `--temperature 0.7` | failed after 6 s | `KeyError: 'message'` |
| o1 | `gpt-5.4-mini`, OpenAI's API | failed after 2 s | HTTP 400, "'max_tokens' is not supported with this model. Use 'max_completion_tokens' instead." |

- The temperature error of turn m1, asked directly: on Mistral's API `mistral-large-latest` refuses
  `temperature: 0` alone, accepts it together with `top_p: 1`, and accepts a request without a temperature.
  So it is a rule of that API for that model (temperature 0 is "greedy sampling" and needs `top_p` 1), and
  `regenerate` trips it because it always sends a temperature and never a `top_p`. GLM 5.3 on the same API
  accepts temperature 0.
- GLM 5.3 on Mistral's API answers a one-line request in 1 second, against 36 seconds on the local
  endpoint at the time. The model answered in both turns; the answer was thrown away for its shape.
- OpenAI's API refuses the request for a parameter name. Which OpenAI model an adopter would use was not
  the question; a small one was taken for the probe.
- The owner then named the OpenAI model to use: `gpt-6.1-sol` at reasoning level "medium". Turn o2 with it
  fails as o1 did, on `max_tokens`. Asked directly, the model answers a request with
  `max_completion_tokens` and `reasoning_effort: "medium"`, and refuses `temperature: 0` ("Only the default
  (1) value is supported"). So on this model two of the parameters `regenerate` always sends are refused.
- `mistral-large-latest` and `mistral-large-4` are the same model: Mistral's model record names it
  "Mistral Large 4.0" with both as aliases. The doubt stated in R98's first amendment is settled.

Four kinds of failure in six turns on three vendors: a rate limit not waited out, a sampling parameter
sent unasked, a parameter name one vendor no longer takes, and a reply shape not read. Requirement 1 above
gains one item: the name of the output limit parameter depends on the endpoint.

## COBESY check

Outcome first: three failures, each with the endpoint's or the code's own words. What was expected
beforehand is named as read from the code, not as a tested claim. What the probe could not reach is listed.
