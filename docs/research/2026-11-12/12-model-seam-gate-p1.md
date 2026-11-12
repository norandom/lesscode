# R105: Gate P1 of the model seam. One library completes all six first cases; the alternative fails the way our own client did

- Report date/time and timezone: 2026-11-12, 18:36, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 18:28 to 18:35
- Status: complete for P1; P2 to P4 not run
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), [prototype gate](../../../.kiro/specs/model-seam/prototype-gate.md)
- Hypothesis ID and predecessor: assumption P1 of the gate; [R103](10-model-seam-library-prototype.md)
- Type: prototype gate, pass and fail conditions written before the run (commit `2eb3f2c`)

## Decision

P1 passes for `genai` 0.7.4. It stays the candidate library of the design. `llm` 1.3.8, the alternative
tried, is not usable for this purpose without rules of ours per vendor. A third library was looked at and
not run (`rig-core` 0.46.0: by its own documentation it holds contracts only and needs companion crates
for transport).

## Hypothesis and falsification

P1: a client library covers every first case without a rule of ours that names a model or vendor. Passes
when one library completes the turn on all six and gives, for a refused call, the status, the endpoint's
message and any stated wait. Fails when no library tried completes all six, or a refusal's details are not
reachable.

## Frozen inputs and method

- The turn of R102: the prompts `regenerate run` wrote for the ten-line Python function.
- The six first cases of requirement 2.7, run by `docs/experiments/model-seam/run-first-cases.ps1` with
  endpoint files outside the repository and the repository's `.env` for the local endpoint.
- Prototypes: `docs/experiments/model-seam/prototype/` (`genai`) and `prototype-llm/` (`llm`), each with its
  own lock file. Each reads the endpoint with the core's loader and sends an output limit of 64,000 tokens
  and the reasoning level the file names.

## Evidence and results

`genai` 0.7.4:

| Case | Result | Seconds | Tokens in, out |
| --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | completed, object as asked | 1 | 645, 172 |
| 2 Mistral Large, Mistral's API | completed, object as asked | 37 | 650, 1,519 |
| 3 `gpt-6.1-sol`, OpenAI's API | completed, object as asked | 16 | 641, 592 |
| 4 `claude-opus-5-5`, Anthropic's API | completed, object as asked | 7 | 887, 590 |
| 5 GLM 5.3, local endpoint | completed, object as asked | 3 | 645, 262 |
| 6 Mistral Large, local endpoint | completed, object as asked | 5 | 650, 391 |

- For a refused call the library's error carries the status, the response body and the response headers
  (`ResponseFailedStatus` in its `webc` error; seen with HTTP 429 from the local endpoint at 17:43).
- One addition of ours was needed, by request style and not by vendor: the Anthropic style's address is
  given without its version segment in the owner's endpoint file, so the prototype appends it when absent.
  The research tool has done the same since R70.
- Case 6 came back inside a code fence, case 1 this time without one. The caller must accept both.

`llm` 1.3.8:

| Case | Result |
| --- | --- |
| 1, 2, 5, 6 | the model answered; the library could not read the reply ("invalid type: sequence, expected a string"), because the text came as parts |
| 3 | refused by OpenAI's API: `max_tokens` not supported, use `max_completion_tokens` |
| 4 | not run, to keep the calls to that endpoint for the cases that count |

- Its backend named OpenAI speaks only the Responses API, which Mistral's API and the local endpoint do not
  serve (404, "Unsupported endpoint"). The five cases above were run through its backend named Mistral,
  the one that sends plain chat completions to a given address. Choosing a backend by who serves the
  endpoint is the kind of rule P1 excludes.
- Its errors are strings; a status or a stated wait is not reachable as data.
- It has retry with backoff built in, without telling the caller how long it waits.

Dependencies, crates in the prototype's lock file (the core alone: 115): `genai` 293, `llm` with two
backends and TLS 253, `rig-core` 295 before any transport crate.

Calls to the Anthropic endpoint: one of the three allowed.

## Limits and interpretation

- One turn per case. Long replies, time limits and rate limits are P3's subject.
- `llm` was configured from its source in half an hour; a setting that reads replies in parts may exist
  and was not found. The OpenAI refusal does not depend on that.
- The comparison is of these versions on this day.

## Tool/spec consequence and next gate

- Design: `genai` 0.7.4 pinned, as written. The seam normalises the Anthropic style's address.
- Next: P2 (build for the five release targets, with the TLS option to choose), P3 (waiting against a stub
  endpoint and one real refusal), P4 (the core through the process boundary; receipts).

## COBESY check

Pass and fail conditions were in the gate before the run. The alternative's failure is reported with what
was and was not tried to make it work. What was only read and not run is named.
