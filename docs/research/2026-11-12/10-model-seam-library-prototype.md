# R103: Prototype. The same turn through an existing model client library completes on three models from three vendors

- Report date/time and timezone: 2026-11-12, 17:33, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 17:29 to 17:33
- Status: complete as a prototype; no hypothesis was preregistered
- Spec: none yet; prototype evidence for the model seam, as [prototype-first](../../../.kiro/steering/prototype-first.md)
  asks before a design
- Hypothesis ID and predecessor: [R102](09-product-path-two-models.md) (the hand-written request fails on
  GLM, Mistral and OpenAI endpoints for four reasons). Owner instruction, 2026-11-12: use an existing
  model library, there are many
- Type: bounded real-code prototype, one library, one turn per model

## Decision

Do not repair the hand-written request. The model seam is built on an existing client library. The one
tried here, `genai` 0.7.4, completes the turn on all three models without a line about any vendor in our
code. Its cost is the dependency set, so the seam belongs in its own crate beside the core, not in it:
the core and the driver stay free of model calls and of these dependencies.

## Hypothesis and falsification

None stated. The question was whether a library removes the four failures of R102 without vendor-specific
code of ours. A single failure on one of the three models would have been a no.

## Frozen inputs and method

- Prototype: `docs/experiments/model-seam/prototype/` (Rust, about 80 lines, own lock file, not a member of
  any workspace). It reads the endpoint file with the core's `load_endpoint`, maps the style to the
  library's adapter, and sends the system and user prompt that `regenerate run` wrote for the task of R102.
- Sent: an output limit of 64,000 tokens, and the reasoning level when the endpoint file names one. No
  temperature, no response format.
- Library: `genai` 0.7.4 (MIT or Apache-2.0), chosen from a look at what crates.io offers for several
  vendors behind one interface (`genai`, `rig-core`, `llm`, `aisdk`, `multi-llm`). Only this one was built
  and run; the others were not tried.

## Evidence and results

| Model, endpoint, level | Result | Seconds | Tokens in, out | Reply |
| --- | --- | --- | --- | --- |
| GLM 5.3, Mistral's API, high | completed | 1 | 645, 156 | the JSON object asked for, inside a code fence |
| Mistral Large, Mistral's API, high | completed | 7 | 650, 726 | the JSON object asked for |
| `gpt-6.1-sol`, OpenAI's API, medium | completed | 16 | 641, 657 | the JSON object asked for |

- Each reply has the three keys the turn asks for (`files`, `structure`, `change_log`), and each rewrites
  the function with the three identical branches merged.
- The four failures of R102 do not occur: the library names the output limit as each vendor wants it,
  sends no temperature unless asked, and reads a reply's text from parts. A rate limit was not met in
  these turns, so waiting one out is not shown.
- The three rewrites differ: one keeps the loop, one returns a sum, one keeps the lookup of each item's
  kind. None was checked against tests here. That is the fence's work, and the reason it stays.
- Cost: the core's lock file lists 115 crates; with the library and an async runtime the prototype's lists
  293, among them a TLS stack with a C build (`aws-lc-sys`).

## Limits and interpretation

- One turn per model on a ten-line task. Long replies, streaming, time limits and rate limits are not
  exercised.
- The Anthropic style is mapped in the prototype and was not run.
- One library. That it works does not say it is the best choice; the dependency count and the build on
  five platforms have to be weighed in the design, against at least one alternative.
- GLM's reply came in a code fence; the caller has to accept that or ask for a response format.

## Tool/spec consequence and next gate

For the spec of the model seam:

1. A separate crate holds every model call: `regenerate`'s request, the research tool's client and the
   check command for adopters. The core keeps the fence (admission, controls, receipts) and no network code
   for models.
2. The endpoint file stays the only configuration: style, address, model, key, optional reasoning level.
3. Design gate: build the seam crate on the five release platforms before committing to the library.
4. First cases: GLM 5.3 and Mistral Large on Mistral's API, `gpt-6.1-sol` on OpenAI's API, one Anthropic
   model.

## COBESY check

Outcome first, with the one condition that would have made it a no. What was run and what was only looked
up is separated. The cost is stated beside the benefit.
