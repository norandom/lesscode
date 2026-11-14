# Task 46 handover: group 2 of the model seam, waiting with an estimate

Written 2026-11-14, 07:41 Europe/Berlin (from the clock). Spec `model-seam`, tasks 2.1 and 2.2.

## What was done

- 2.1 `wait.rs`: `call` and `call_with` (the same call with its time given). A refusal with status 429 or
  503 is waited out: the wait the endpoint states in a header, else in its message, else ten seconds; the
  wait made is the larger of that and twice the wait that just failed, never under one second; the limit
  is ten minutes of refused attempts and pauses, settable. Each wait is announced before it starts. One
  record line per call; a line that cannot be written fails the call.
- 2.2 `client.rs`, `wait.rs`: a request on its way (`begin`, `Attempt`) is awaited in steps. When it has
  run for twice the median of this process's answered requests, or for a minute when there are none, the
  user is told, and every 30 seconds after that. At the answer limit (sixty minutes, settable as
  `Limits.answer_seconds`) the request is given up with `CallError::NoAnswerInTime`. `send` was split:
  `begin` starts the request, `read` turns the library's reply into ours, `send` is one attempt awaited
  to its end.

## Evidence

- `cargo nextest run` in `model/`: 39 tests pass. `tests/waiting.rs` (9, the five cases of R106) and
  `tests/slow_answer.rs` (6) run against the local stub with injected time; one test of the latter waits
  a fifth of a second for real, to show that a request stays on its way between steps.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean.
- Budgets: `lesscode-core budgets --config model/complexity-budgets.json --root model/src` exits 0. The
  longest functions are `begin` in `client.rs` and `attempts` in `wait.rs`, 343 of 560 tokens each.
- Commits `0bd099e` (2.1) and `92d7f1e` (2.2). Neither is pushed; `origin/main` is at `a235588`.

## Deviations, recorded in the design as Amendments 3 and 4

- Amendment 3 (2.1): `call` lives in `wait.rs`; no wait under one second; the limit counts the time since
  the call began and a wait that would pass it is not made; an unwritable record fails the call.
- Amendment 4 (2.2): the seam ends a slow request itself by dropping it, and the library's web timeout is
  not set; `Time` has `answer_within`; the answer limit counts per request; the median is over answered
  requests only, without their waits.
- Open for the owner: the minimum wait of one second and the failing of a call whose record cannot be
  written (both Amendment 3) have not been commented on.

## Current state

- Nothing in `native/` changed. `Endpoint` there is not `Clone`; `begin` copies what the request needs.
- The slow-answer tests need a process each, as nextest gives them: the expected time depends on the
  answers the process has had. Under plain `cargo test` they can disturb each other.
- The build directory of the crate is `D:\BuildArtifacts\cargo-target\lesscode-model`.
- Not yet in the pipeline (task 8.1). Remote branch `model-seam-gate-p4` (`37effda`) is kept for task 7.1.
- Calls to the vendor endpoint: 2 of 3 used; the last is for task 5.1.

## Next steps

1. Group 3: `repair.rs` as designed, on top of `call`. A fresh call with the request, the rejected reply
   and the reasons; the request again when no reply arrived or it was cut off; at most two repairs unless
   the caller says otherwise; every attempt recorded; the loop writes no file. `tests/repair.rs` and a
   doc test that the loop cannot be called without a check.
2. Then group 4 (`call`, `check`, `measure` commands and `model/tasks`), group 5 live, group 6 research
   tool, group 7 core, group 8 CI, release and docs.
