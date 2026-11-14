# R109: A real refusal on the local endpoint is waited out: ten calls through `lesscode-model call` announce a wait of about one second, wait and are answered. The small call sent beside them was answered before the limit was met

- Report date/time and timezone: 2026-11-14, 09:18, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-14, 09:09:16 to 09:14:05
- Status: completed
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), requirements 3.1 to 3.4, [task 5.2](../../../.kiro/specs/model-seam/tasks.md); the live condition of P3 in the [prototype gate](../../../.kiro/specs/model-seam/prototype-gate.md)
- Hypothesis ID and predecessor: the open live part of P3, [R106](../2026-11-12/13-model-seam-gate-p2-p3.md)
- Type: live check of a carried gate condition; the pass condition is the task's (written 2026-11-12)

## Decision

The item the gate carried forward is closed: a real endpoint refused calls for its rate limit, the seam
announced each wait before making it, waited, asked again and was answered. This happened on ten calls
in one round. Task 5.2 is ticked.

One point differs from the task's wording. The task names "one further call" beside the load. That call,
the small turn of R102, was answered in 1.3 s at the start of the round, before the limit was met. The
ten calls that waited are calls of the load. They went through the same command, so the behaviour asked
for is shown; the owner may read the wording more strictly.

**Amendment, 2026-11-14, 10:00.** The owner decided to keep the wait limit as designed: the time an
endpoint holds a refused attempt counts against it (design Amendment 9).

## Hypothesis and falsification

With enough large calls side by side, the local endpoint refuses some for its rate limit. A refused call
through `lesscode-model call` then prints the endpoint's host and an estimate before it waits, makes the
wait, and is answered; the wait is in the reply and in the record. Rejected when a refused call fails
instead, when a wait is made without a notice, or when no refusal can be provoked in one hour.

## Frozen inputs and method

- Binary: `lesscode-model`, debug build of main at `283d510`. Limits at their defaults: ten minutes of
  waiting, sixty minutes for an answer.
- Endpoint: the repository's `.env`, the local endpoint at `127.0.0.1:8765` (container
  `mvibe-proxy-proxy-1`), model `zai-glm-5-3`, reasoning level high. Nothing else of ours ran against it.
- `docs/experiments/model-seam/run-refusal.ps1`: rounds of large calls started side by side, each through
  `call` with its own record file. A large call is a document of 1,500 generated entries (about 24,100
  input tokens), each starting differently, with a request for about 800 words and an output limit of
  1,500. Beside them, one call with the turn of R102. Round 1 has 8 large calls, round 2 has 16; further
  steps were prepared and not needed. The script stops when a call was answered after a wait, after one
  hour, or at 15 million input tokens (a cap added for this run; the task bounds only the time).

## Evidence and results

Records and notices of every call are in `docs/experiments/model-seam/refusal/` (`call-ROUND-N.jsonl`,
`call-ROUND-N.notices.txt`; `N` 0 is the small call).

| Round | Started | Calls | Answered | Refused and waited out | Failed | Input tokens | Seconds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 09:09:17 | 8 large, 1 small | 9 | 0 | 0 | 193,601 | 72 |
| 2 | 09:10:29 | 16 large, 1 small | 17 | 10 | 0 | 387,205 | 216 |

- Round 1: every call was answered on its first attempt, the slowest after 71 s.
- Round 2: the first five large calls were answered on their first attempt (9.5 s to 120 s) and one more
  after 130 s. The other ten were refused after about 130 s, waited, asked again and were answered,
  between 138 s and 215 s after they were started.
- The notice of each of the ten, before its wait, for example call 14: "127.0.0.1:8765 asks to come back
  later (attempt 1); waiting about 1 s, 130 s waited so far". Each record has one wait of 1 s, marked as
  stated by the endpoint. Each of the ten needed one wait and two attempts.
- The estimate against what followed: the wait announced was about 1 s and the wait made was 1 s. The
  second attempt was accepted every time and then took about 8 s to 85 s to answer.
- Calls that ran longer than a minute also printed "still waiting for the answer" at 60 s, 90 s and
  120 s, as designed for a slow answer.
- The small call: 17.5 s in round 1 and 1.3 s in round 2, no wait.
- In all: 26 calls, 26 answered, 580,806 input tokens. No output or record contains the key.

## Limits and interpretation

- The endpoint's own words of the refusal were not kept: a call that is answered in the end records its
  waits and not the message of the attempt that was refused. The stated wait of one second fits the
  message seen in R102 ("Upstream rate limit persisted; retry after 1 seconds"); that it is the same
  message is an inference.
- The endpoint held each refused call for about 130 s before it refused. The seam counts that time as
  waited ("130 s waited so far"), as the design says for refused attempts. A call refused five times in
  this way would pass the default limit of ten minutes without having paused for more than a few seconds.
  That did not happen here; it is a property of this endpoint to keep in mind for long runs.
- One wait per call. The doubling of a wait after a second refusal was not met live; it is shown on the
  stub only.
- The limit was met at 16 large calls side by side and not at 8. Where between them it lies, and whether
  it counts calls or tokens, was not measured.

## Tool/spec consequence and next gate

- P3's live condition is met; the prototype gate has one carried item left (the receipt, task 7.3).
- Nothing in the design changes.
- For the owner, not acted on: whether the time an endpoint holds a call before refusing it should count
  against the wait limit as it does now.
- For task 5.3: `measure` runs its trials one after the other, far below the load that met the limit.
- Next: task 5.3, the indicator on real models.

## COBESY check

The difference to the task's wording is in the title and the decision. Counts and times are from the
kept records. What was not captured (the refusal's message) is said.
