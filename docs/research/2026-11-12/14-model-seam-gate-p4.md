# R107: Gate P4 of the model seam. `regenerate run` completes on all six first cases through a second executable; the receipt still names settings that were not sent

- Report date/time and timezone: 2026-11-12, 20:10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 19:40 to 20:09
- Status: complete; the gate as a whole is assessed here
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), [prototype gate](../../../.kiro/specs/model-seam/prototype-gate.md)
- Hypothesis ID and predecessor: assumption P4 of the gate; [R105](12-model-seam-gate-p1.md), [R106](13-model-seam-gate-p2-p3.md)
- Type: prototype gate, pass and fail conditions written before the run (commit `2eb3f2c`)

## Decision

P4 passes for the process boundary and for earlier receipts. One part of its condition is not met: the
new receipt records what was sent, and beside it still carries the fields of the hand-written request
(effort "high", temperature 0), which were not sent. That is a consequence of where the prototype sits and
is closed by removing the hand-written request, which the design already plans.

Gate as a whole: **proceed**, with two named items carried into implementation: waiting out a real refusal
(P3, live) and the receipt stating only what was sent (P4). Neither changes the design.

## Hypothesis and falsification

P4: the core can reach the seam through a process boundary and keep its receipts. Passes when the turn
completes on the six first cases, the receipt states only what was sent, and the replay test and the
admission of earlier receipts still pass. Fails when a receipt written before this feature no longer
verifies, or the turn fails on a case that P1 passed.

## Frozen inputs and method

- Executable: `docs/experiments/model-seam/prototype-wait`, command `call ENV_FILE`. One JSON request on
  standard input (`protocol`, `system`, `user`, `max_output_tokens`, optional `temperature`), one JSON
  reply on standard output (text, stop, tokens, seconds, waits, and what was sent), notices and the reason
  of a failure on standard error. Main, commit `fea6344`.
- Core: branch `model-seam-gate-p4`, commit `37effda`. A new file `native/src/seam.rs` (713 tokens) is a
  transport for the regenerator: with `LESSCODE_MODEL_BIN` set it hands the request to that executable,
  names the endpoint by its file, removes one code fence around the reply, and returns the reply in the
  form the existing code reads. Without the variable nothing changes. The regenerator's two call sites
  name this transport; `regenerator.rs` grew from 20,788 to 20,804 tokens (ceiling 20,827).
- The turn of R102, by `docs/experiments/model-seam/run-p4.ps1`.

## Evidence and results

`lesscode-core regenerate run` through the executable, receipts and candidates in
`docs/experiments/model-seam/p4/`:

| Case | Exit | Seconds | Tokens in, out | Candidate written |
| --- | --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | 0 | 2 | 645, 142 | yes |
| 2 Mistral Large, Mistral's API | 0 | 18 | 650, 1,016 | yes |
| 3 `gpt-6.1-sol`, OpenAI's API | 0 | 13 | 641, 504 | yes |
| 4 `claude-opus-5-5`, Anthropic's API | 0 | 6 | 887, 502 | yes |
| 5 GLM 5.3, local endpoint | 0 | 2 | 645, 168 | yes |
| 6 Mistral Large, local endpoint | 0 | 12 | 650, 295 | yes |

- These are the turns that failed in R102 for four different reasons. The same command now parses each
  reply and writes the candidate file.
- The core's suite on the branch: 390 tests pass, among them the recorded three-turn replay and the
  admission of emitted receipts. The complexity budgets hold.
- Each receipt's turn note has what the executable reported as sent, for example host `api.openai.com`,
  model `gpt-6.1-sol`, output limit 64,000, reasoning level "medium", no temperature.
- Not met: the same note still has `"mode": "openai-json-http"`, `"effort": "high"` and
  `"temperature": 0.0` from the code that built the unused request, and the inputs fact still lists the
  effort. A reader of the receipt sees two statements and only one is true.
- Calls to the Anthropic endpoint: two of the three allowed.

## Limits and interpretation

- One turn per case, no rate limit met, no failure of the executable provoked in this run except in the
  launcher's own handling (an unreadable reply and a missing endpoint file are code paths without a live
  case here).
- The prototype passes no temperature at all; the final call passes one when the user gives one.
- The launcher finds the executable by a variable only. Looking beside the core's own executable and on
  the search path is designed, not built.

## Tool/spec consequence and next gate

- The process protocol of the design works as written, with one addition found here: the reply's text is
  returned as the model wrote it, and the core removes a code fence before parsing.
- Implementation order confirmed: the hand-written request leaves the core in the step that puts the
  launcher in, so that the receipt can state one thing.
- The design is revised to its demonstrated form and goes to the owner for approval. Tasks follow the
  approval.

## COBESY check

The unmet part of the condition is in the title. The assessment says what is carried forward and why
that is bounded. Counts and commits are given for every claim.
