# R108: The six first cases through `lesscode-model call`: all answered in the stated structure. `check` finds five endpoints usable; on the Anthropic case it was not run

- Report date/time and timezone: 2026-11-14, 08:58, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-14, 08:51:57 to 08:53:56
- Status: completed; `check` on case 4 was run at 09:07 after the owner's approval (amendment below)
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), requirements 2.7 and 1.6, [task 5.1](../../../.kiro/specs/model-seam/tasks.md)
- Hypothesis ID and predecessor: the first cases of [R105](../2026-11-12/12-model-seam-gate-p1.md) and [R107](../2026-11-12/14-model-seam-gate-p4.md), now through the built binary instead of a prototype
- Type: live check of an implemented task; the pass condition is the task's (written 2026-11-12)

## Decision

Requirement 2.7 is met by the built binary: the same call completes on all six first cases. Requirement
1.6 is met: the local endpoint is used like any other, with nothing in the command or the code that is
specific to it.

Task 5.1 asks for `call` and `check` on each case. `check` was not run on the Anthropic case: the owner
allowed three calls to that endpoint, `call` used the third, and `check` needs one to three more. The task
stays open on that one cell until the owner allows the calls or accepts the case without them.

**Amendment, 2026-11-14, 09:09.** The owner approved the calls ("approved", 09:06). `check` on case 4 ran
at 09:07:33: exit code 0, one call, 6.6 s, 876 tokens in, 528 out, no wait, "this endpoint can be used".
All six cases now pass both commands and task 5.1 is closed. Calls to the Anthropic endpoint: four, the
three first allowed and one of the one to three approved today. The output is in
`live/case-4/check.txt`. The title and the text below stand as written at 08:58.

## Hypothesis and falsification

Each case completes the turn of R102 through `lesscode-model call` with exit code 0, a complete stop and a
reply in the structure the turn asks for, and `lesscode-model check` ends with exit code 0. A case fails
when the call fails, the reply is cut off or not in the structure, or the check rejects or fails; it is
then named with the endpoint's words. No output or record may show the key.

## Frozen inputs and method

- Binary: `lesscode-model` built from main at `064d7f8` plus this task's test file, debug profile,
  `genai` 0.7.4.
- `model/tests/live.rs`, two tests through the binary, run only when `LESSCODE_LIVE_ENV` names an endpoint
  file. `docs/experiments/model-seam/run-live.ps1` runs the cases one after the other with one test
  thread, so that no two calls go to one upstream side by side.
- The turn: `system.txt` and `user.txt` of the turn `t2` of R102 (1,933 and 820 bytes), the prompts
  `regenerate run` wrote for the frozen module `total_price`. Request: output limit 64,000, no
  temperature, and a structure (an object with `files`, `structure`, `change_log`) that is checked on the
  reply and not sent.
- `check`: the first task of the built-in set through the repair loop, at most three calls, judged by the
  core's parsing and static checks.
- Endpoint files outside the repository (`mistral.env`, `openai.env`, `model.env` in the home directory's
  `.lesscode`), and the repository's git-ignored `.env` for the local endpoint. Cases 1 and 6 name their
  model with `LESSCODE_MODEL_NAME`. The reasoning level is what each file states.
- The local endpoint is the container `mvibe-proxy-proxy-1`. It had been stopped for two days and was
  started for this run.

## Evidence and results

What the binary said is in `docs/experiments/model-seam/live/case-N/` (`call.txt`, `check.txt`,
`record.jsonl`).

`call`, the turn of R102, one call per case:

| Case | Exit | Seconds | Tokens in, out (reasoning tokens reported) | Stop | In the structure | Level sent |
| --- | --- | --- | --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | 0 | 2.3 | 645, 192 (35) | complete | yes | high |
| 2 Mistral Large, Mistral's API | 0 | 7.1 | 650, 496 (385) | complete | yes | high |
| 3 `gpt-6.1-sol`, OpenAI's API | 0 | 12.0 | 641, 432 (303) | complete | yes | medium |
| 4 `claude-opus-5-5`, Anthropic's API | 0 | 5.7 | 887, 517 (195) | complete | yes | none |
| 5 GLM 5.3, local endpoint | 0 | 1.4 | 645, 206 (24) | complete | yes | high |
| 6 Mistral Large, local endpoint | 0 | 26.2 | 650, 1,024 (870) | complete | yes | high |

`check`, the built-in task:

| Case | Exit | Calls | Seconds | Tokens in, out | Finding |
| --- | --- | --- | --- | --- | --- |
| 1 | 0 | 1 | 1.3 | 638, 241 | this endpoint can be used |
| 2 | 0 | 1 | 7.2 | 643, 880 | this endpoint can be used |
| 3 | 0 | 1 | 14.9 | 634, 625 | this endpoint can be used |
| 4 | not run | | | | no call left for this endpoint |
| 5 | 0 | 2 | 2.9 | 1,500, 413 | this endpoint can be used |
| 6 | 0 | 1 | 11.5 | 643, 897 | this endpoint can be used |

- No case failed, so there are no endpoint's words to report. No call waited; standard error was empty on
  every call.
- Cases 4 and 5 wrapped the reply in a code fence; the structure was found in both.
- Case 5's check took two calls: the first reply was rejected and the repair was accepted. The report does
  not say why the first reply was rejected, and the check keeps no record, so the reason is not known.
- Each `call` wrote one line to its record. No kept output and no record contains the key of its endpoint
  file (asserted by the test).
- Nothing was sent that was not stated: no temperature on any call, a reasoning level only where the file
  has one.
- The crate's suite without `LESSCODE_LIVE_ENV`: 79 tests pass (the two live tests end at once);
  `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- Calls to the Anthropic endpoint: three of the three allowed.

## Limits and interpretation

- One call and one check per case. This shows that the path works, not how often a model's reply is
  accepted; that is task 5.3.
- No rate limit was met; waiting on a real refusal is task 5.2.
- The times are single observations. Mistral Large took 7 s on Mistral's API and 26 s through the local
  endpoint on the same turn, with more than twice the reasoning tokens; one call each does not say which
  side the difference comes from.
- The debug build was used, not a released binary; the released form is task 8's.

## Tool/spec consequence and next gate

- Nothing in the design changes.
- Observation for the owner, not acted on: when a repair succeeds, `check` reports the number of calls
  and not the reason the first reply was rejected. Requirement 5 does not ask for it.
- Open with the owner: `check` on the Anthropic case (one to three calls), or the case accepted as it is.
- Next: task 5.2, one real refusal waited out on the local endpoint.

## COBESY check

The cell that was not run is in the title and in the decision. Every number is from a kept file. The
pass condition predates the run. What one call per case cannot show is stated.
