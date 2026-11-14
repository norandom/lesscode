# Task 47 handover: group 3 of the model seam, the repair loop

Written 2026-11-14, 07:53 Europe/Berlin (from the clock). Spec `model-seam`, task 3.

## What was done

- `model/src/repair.rs`: `with_repairs(call, request, check, repairs)`. A first call, and while the
  caller's check rejects the reply, at most `repairs` more (`REPAIRS` is 2). A rejected reply that is
  complete and has text is followed by a fresh call that shows the request, the rejected reply and the
  reasons (`repair_request`); an empty reply or one that did not stop as complete is followed by the
  request as the caller stated it. The outcome is `Accepted`, `Rejected` (the last reply, every attempt)
  or `Failed` (a call failed; the attempts before it). Each attempt keeps its verdict, reasons, stop and
  tokens. The loop makes its calls through the caller's `call` and touches no file.
- `lib.rs` has the module; `complexity-budgets.json` has its entry.

## Evidence

- `cargo nextest run` in `model/`: 48 tests pass, 9 of them in `tests/repair.rs` with scripted replies:
  accepted on the first, second and third call; rejected after two repairs with the reasons of every
  attempt; a cut-off and an empty reply asked again unchanged; the number of repairs stated by the
  caller; attempts with stop and tokens; a failed call as `Failed`; over all eight courses of three
  replies, `Accepted` only after a verdict that passed.
- `cargo test --doc` in `model/`: 2 pass. The loop with a check compiles; the same without one does not.
  nextest does not run doc tests, so this is a second command until the pipeline has it (task 8.1).
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean. Budgets exit 0; `repair.rs`
  has 24 logical units and a longest function of 44 lines.
- Commit `67cc1dc` (task 3). Not pushed; `origin/main` is at `a235588`.
- Review: inline, by the owner's rule against subagents.

## Deviations, recorded in the design as Amendment 5

- `Outcome::Rejected` holds `last: Reply`, not `Option<Reply>`: a rejection without a reply cannot occur.
- Every reply goes to the check, a cut-off one too. "No reply arrived" is read as a reply with empty text.
- A request sent again as it was counts as one of the repairs, so the calls are at most `repairs + 1`.
- A failed call ends the loop at once; the loop does not retry it.
- `repair::Attempt` is not re-exported at the crate root, where `Attempt` is the request on its way.
- Still open for the owner from Amendment 3: the minimum wait of one second, and the failing of a call
  whose record cannot be written.

## Current state

- Nothing in `native/` changed. No live call was made; calls to the vendor endpoint stay at 2 of 3 used,
  the last is for task 5.1.
- No research entry for this group: there is no measured result, and the deviations are in Amendment 5.
- The build directory of the crate is `D:\BuildArtifacts\cargo-target\lesscode-model`.
- Not yet in the pipeline (task 8.1). Remote branch `model-seam-gate-p4` (`37effda`) is kept for task 7.1.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. Group 4: the commands of `lesscode-model`. 4.1 `call` (the process protocol), 4.2 `check` (one small
   fixed task through `with_repairs`, at most three calls, no file in the working directory), 4.3
   `measure` with `model/tasks` (trials through `with_repairs`, the three verdicts).
2. Then group 5 live, group 6 research tool, group 7 core, group 8 CI, release and docs.
