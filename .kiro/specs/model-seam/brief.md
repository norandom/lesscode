# Brief: model-seam

Discovery of 2026-11-12, path C (one new spec). Not approved; requirements are the next step.
Evidence: [R102](../../../docs/research/2026-11-12/09-product-path-two-models.md) (the present model path fails
on three vendors), [R103](../../../docs/research/2026-11-12/10-model-seam-library-prototype.md) (prototype on an
existing library), [R99](../../../docs/research/2026-11-12/06-repair-gate-result.md) (propose, judge, repair).

## Problem

An adopter who wants a model to do work behind lesscode can use only the model the tool was built against.
The owner's statements of 2026-11-12:

- Language packs are published once with the tool. They are the knowledge base and are not regenerated per
  repository or with a cheaper model.
- What is needed is that adopters can run GLM or another model behind the tool.
- The core and the driver are model agnostic. The fence around a model's output is needed because inference
  is not deterministic.
- Use an existing model library; do not write another client by hand.
- Preferred solution: horizontal compatibility with an estimated wait time.
- The internal endpoint on localhost is one of the endpoints that must work.

## Current State

- `lesscode-core regenerate` builds its request by hand in `native/src/regenerator.rs`. Six live turns on
  2026-11-12 (R102) ended in four kinds of failure before any answer was used: a rate limit not waited out
  (GLM 5.3, local endpoint), a temperature always sent (refused by Mistral's API without `top_p` 1 and by
  OpenAI's `gpt-6.1-sol` outright), an output limit parameter with one fixed name (refused by OpenAI's API),
  and a reply delivered as parts that is not read (Mistral Large and GLM 5.3 on Mistral's API).
- The endpoint file accepts the styles `openai` and `anthropic`; only the first is sent.
- A failed turn prints nothing. The reason is in the receipt only.
- The research tool `tools/guide-optimizer` has a second hand-written client with its own fixes (reply
  parts, a wait on HTTP 429, repair turns). Two clients, neither complete.
- The endpoint file (`LESSCODE_MODEL_API_STYLE`, `_BASE_URL`, `_NAME`, `_API_KEY`, optional
  `_REASONING_EFFORT`; model and level can be set per run in the environment) works and stays.
- Prototype (R103): about 80 lines on the `genai` crate complete the same turn on GLM 5.3 and Mistral Large
  (Mistral's API) and on `gpt-6.1-sol` (OpenAI's API), and on Mistral Large through the local endpoint. GLM
  5.3 through the local endpoint was refused with HTTP 429 while another run of ours held the limit; the
  library reported it and did not wait.

## Desired Outcome

- An adopter writes one endpoint file and the model path works, for every endpoint on the same footing:
  a vendor's API, another vendor's API, a local endpoint. That is the horizontal compatibility: no endpoint
  is the reference and the others the exception.
- When a call has to wait (a rate limit, a slow endpoint, a queue), the tool waits and says how long it
  expects to wait, taken from what the endpoint states and from what it has observed, and says so on the
  terminal. It does not fail silently and does not hang without a word. Reading of "estimated wait time",
  to be confirmed in the requirements.
- A failed call states its reason where the user sees it.
- An adopter can check a configured model once, with a small fixed task through the same fence, and gets
  a plain answer: usable, or what the endpoint refused.
- The core and the driver make no model call and carry none of the client's dependencies.
- What a model returns is used only after the existing checks. A rejected answer can be repaired in a fresh
  call with the checks' reasons, at most twice (the loop of R99).

## Approach

Chosen: a separate crate beside the core holds every model call and is built on an existing client
library. `regenerate` and the research tool call it; the core keeps the fence (admission, controls,
receipts). Reasons: the library removed all four failures in the prototype without vendor-specific code of
ours; its cost is the dependency set (293 crates in the prototype's lock file against the core's 115, an
async runtime, a TLS stack with a C build), which a separate crate keeps out of the core.

Considered and not chosen:

- Repair the hand-written client in place. No new dependency, but every vendor's next change is ours to
  follow, and the owner ruled it out.
- Put the library into the core. One binary, but the core stops being free of network code for models and
  more than doubles its dependencies.

Open for the design, with a gate each:

- Which library. Only `genai` 0.7.4 was built and run. At least one alternative is compared on the same
  turn (`rig-core`, `llm`, `aisdk` and `multi-llm` were looked up, not tried).
- The seam crate builds on the five release platforms.
- How the wait is estimated and how a rate limit is recognised across endpoints (a `Retry-After` header, a
  message such as "retry after 1 seconds", or neither).

## Scope

- **In**: the client crate; the endpoint file as the only configuration; waiting with an estimate; reasons
  on the terminal; the check command; moving `regenerate` and the research tool onto the crate; the repair
  loop as a reusable part; first cases: GLM 5.3 and Mistral Large on Mistral's API, `gpt-6.1-sol` on
  OpenAI's API, one Anthropic model, and the local endpoint with at least two of its models.
- **Out**: any change to what the fence checks; language packs and their guide; choosing or ranking models;
  prices and spend accounting beyond recording tokens; streaming output to the user; tool calling by the
  model.

## Boundary Candidates

- Endpoint description: the file, its keys, what an adopter must and may state.
- Call: one request and one reply, the same for every endpoint, with tokens and stop reason.
- Waiting: recognising "not now", estimating, telling the user, giving up after a stated limit.
- Check command: the fixed task, the fence it goes through, the wording of its result.
- Repair loop: propose, judge, repair with reasons; the judge is supplied by the caller.
- Migration of the two existing callers, with the receipts' content kept.

## Out of Boundary

- The step flow (`step plan`, `pin`, `check`, `record`, `guard`) and the driver: unchanged, no model call.
- What a model is asked to do with a codebase (proposing simplifications): the next spec, which uses
  this one.
- The onboarding skill (`/lesscode start`): its levels 3 and up will use the check command; it is its own
  spec.

## Upstream / Downstream

- **Upstream**: `native/src/regenerator.rs` (`load_endpoint`, receipts, `regenerate`), the driver's
  admission and controls as the fence, `tools/guide-optimizer`.
- **Downstream**: a spec in which a model proposes a simplification where branch conditions and state
  concentrate and the step flow judges it; the onboarding spec.

## Existing Spec Touchpoints

- **Extends**: none. `constraint-guided-compression` (Slice B) supplies the evidence and stays closed.
- **Adjacent**: `deconvolution-loop` and `refinement-evidence-integrity` (the regenerator's receipts and
  their frozen request shape: a change of the request changes what receipts hash, which the design must
  state), `gradual-refactoring-steps` (the step flow, untouched), `agnostic-annotation-driver` (the driver,
  untouched).

## Constraints

- Rust only; no Python tooling. Endpoints, models and keys come from endpoint files, never from code.
- Keys stay outside the repository and out of receipts, logs and error messages.
- Five release platforms; pinned dependency versions; complexity budgets hold for files in the core.
- Prototype first (`.kiro/steering/prototype-first.md`): the library choice, the build on five platforms
  and the wait estimate each need real-code evidence before tasks are generated. R103 is the first part.
- Runs against one upstream are not started side by side; a rate limit met in a test must be the
  endpoint's, not ours.
- Foreground work, one commit per task, entries under `docs/research/` for every run.
