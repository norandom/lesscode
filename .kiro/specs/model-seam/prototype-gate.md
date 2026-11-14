# Prototype gate: model-seam

Written 2026-11-12 with the provisional design. Status: **proceed** (assessment at the end), run on the
same day. The design stays
provisional and design approval stays pending until this gate is assessed as `proceed`
(`.kiro/steering/prototype-first.md`). Results go to dated entries under `docs/research/` and are linked
here.

This feature removes no code by a compression mechanism, so the gate has no reduction target. What it has
to show is that the assumptions of the design hold on real endpoints and real build machines.

## Shown so far

- One turn through an existing client library completes on three models of two vendors and through the
  local endpoint, and returns the object the turn asks for
  ([R103](../../../docs/research/2026-11-12/10-model-seam-library-prototype.md), and the run of 17:43
  recorded in the brief).
- A fresh repair call with the check's reasons works for the affordable model
  ([R99](../../../docs/research/2026-11-12/06-repair-gate-result.md)).

## Assumptions to challenge

| ID | Assumption | Frozen input | Passes when | Fails when |
| --- | --- | --- | --- | --- |
| P1 | A client library covers every first case without a rule of ours that names a model or vendor | The turn of R102 (`docs/experiments/model-seam/request.json`, the prompts written by `regenerate run`); the six first cases of requirement 2.7 | One library completes the turn on all six and gives, for a refused call, the status, the endpoint's message and any stated wait | No library tried completes all six, or a refusal's details are not reachable |
| P2 | The seam builds for the five release targets with pinned versions and without system packages beyond the hosted runners' images | A skeleton crate with the chosen library and TLS option; the release workflow's matrix | `cargo build --release --locked` succeeds on all five | Any target fails for both TLS options |
| P3 | A wait can be recognised and estimated without a rule per endpoint | A stub endpoint that refuses with a stated wait in a header, with a stated wait in its message, and with neither, then answers; one refusal observed on the local endpoint | In each case the seam prints an estimate before waiting, the call then succeeds, and a wait beyond the limit ends with the report of requirement 3.6 | An estimate is missing or wrong by more than a factor of two in a stated case, or the call does not recover |
| P4 | The core can reach the seam through a process boundary and keep its receipts | `lesscode-core regenerate run` on the turn of R102; the recorded three-turn replay test of the core | The turn completes on the six first cases; the receipt states only what was sent; the replay test and the admission of earlier receipts still pass | A receipt written before this feature no longer verifies, or the turn fails on a case that P1 passed |

## At least one alternative

P1 is run for `genai` and for one other library from the list in the brief. The comparison records, per
library: first cases completed, crates added to the lock file, whether a refusal's status, message and
headers are reachable, licence, and the date of the latest release.

## Adverse controls

- An endpoint file with a wrong key: the failure names the refusal and shows no part of the key.
- An endpoint that never answers: the call ends at its time limit with "did not answer in time".
- A reply cut off at a small output limit: reported as cut off, not as an answer.
- A reply that fails the check three times: the check command ends with the check's reason and a non-zero
  exit code.

## Budget

- Calls: at most 60 across all cases; the Anthropic case at most 3 calls, inside the existing stop of
  40 USD for that endpoint (about 32.9 USD recorded).
- No two runs against one upstream side by side.
- Time box: one working day for P1 to P4.

## Stop conditions

- P1 fails for both libraries: `revise`. The owner decides between a third library and a hand-written
  client that sends only stated settings.
- P2 fails for both TLS options on a target: `revise`, with the failing target named.
- P3 or P4 fails: `revise` the affected component; the rest of the design stands.

## Assessment

- P1, 2026-11-12: **passes** for `genai` 0.7.4 on all six first cases; a refusal's status, body and
  headers are reachable. The alternative `llm` 1.3.8 completes none of the five OpenAI-style cases.
  [R105](../../../docs/research/2026-11-12/12-model-seam-gate-p1.md).
- P2, 2026-11-12: **passes**. Five targets, each with the library's `rustls` option and with the
  platform's TLS: ten builds, all succeeded (workflow run 43275325942 on the branch `model-seam-gate-p2`).
  [R106](../../../docs/research/2026-11-12/13-model-seam-gate-p2-p3.md).
- P3, 2026-11-12: **passes on the stub** (five scripted cases). Its live condition is **not met**: no
  refusal could be provoked on the local endpoint with 38 calls at once. Kept open; to be closed when a
  long run meets a rate limit. R106.
- P4, 2026-11-12: **passes for the process boundary and earlier receipts**: `regenerate run` completes on
  all six first cases through a second executable; 390 tests pass on the prototype branch
  (`model-seam-gate-p4`, `37effda`). **Not met:** the new receipt also carries settings of the unused
  hand-written request. [R107](../../../docs/research/2026-11-12/14-model-seam-gate-p4.md).

**Gate as a whole, 2026-11-12: `proceed`.** Carried into implementation as named items, each with a task
and a check: (1) one real refusal waited out with its announced estimate; (2) a new receipt states only
what was sent, shown after the hand-written request has left the core.

**Amendment, 2026-11-14.** Carried item (1) is closed: on the local endpoint ten calls through
`lesscode-model call` were refused for the rate limit, announced a wait of about one second, waited and
were answered. [R109](../../../docs/research/2026-11-14/02-model-seam-real-refusal-waited-out.md). Item (2)
remains with task 7.3.
