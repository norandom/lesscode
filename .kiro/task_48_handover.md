# Task 48 handover: group 4 of the model seam, the commands of `lesscode-model`

Written 2026-11-14, 08:15 Europe/Berlin (from the clock). Spec `model-seam`, tasks 4.1, 4.2, 4.3.

## What was done

- **4.1 `call`** (`6aa2e77`). `model/src/protocol.rs`: the request is read with its protocol number (1; a
  missing or another number is refused), the answer is one JSON object, the kinds of failure have names,
  exit codes 0, 3 and 2. `model/src/main.rs`: the command line, with a small reader for `--name VALUE`
  arguments (no `clap`). `--endpoint-env`, `--record`, `--max-wait`, `--answer-limit`.
- **4.2 `check`** (`b11b051`). `model/src/check.rs`: one small fixed task through `with_repairs`, at most
  three calls; a report with model, host, calls, seconds, tokens and waits, then the finding. Exit codes
  0 usable, 3 the endpoint failed, 4 rejected by the check, 2 usage.
- **4.3 `measure`** (`4264cd5`). `model/src/measure.rs`: `measure` (trials over a task set through the
  repair loop), `Counts`, `verdict` with the thresholds 3/4 and 9/10 as constants, `Measured` (the report
  as lines and as JSON). `model/src/taskset.rs`: a task set on disk, the built-in set, `CoreCheck` (the
  core's `parse_result`, `prepare_candidate`, `write_candidate`, `evaluate_tier1_static` on a copy per
  reply in a temporary directory). `model/tasks/`: four small modules, embedded in the binary.
  `--trials`, `--quick`, `--task-set`, `--out`.
- The check command's task is the first of the built-in set since 4.3; `check.rs` holds no task itself.
- `model/tests/binary/mod.rs` runs the binary for the tests of all three commands.

## Evidence

- `cargo nextest run` in `model/`: 77 tests pass. New: `tests/call_command.rs` (5), `tests/check_command.rs`
  (6), `tests/measure.rs` (18). Through the binary against the stub: an answered call, a failed call with
  its reason on standard error and exit code 3, an unreadable request with exit code 2; check usable,
  repaired, refused, rejected after three requests, a wait reported; measure with exit codes 0, 4, 3
  and 2, twenty trials by default, the quick outcomes, a supplied set that is only read, `--out`. No file
  appears in the directory the commands are run in.
- The built-in set against the core's checks: an answer to each of the four tasks passes; careless
  answers are rejected by `surface` (task 2), `static_targets` (tasks 3 and 4) and `structure` (task 4).
- The verdict rule at 15 and 14 of 20 first replies, 18 and 17 of 20 passed, the quick outcomes, five and
  four of twenty not run.
- `cargo test --doc`: passes. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean.
- Budgets exit 0: `main.rs` 89 units, `taskset.rs` 81, `measure.rs` 74, `check.rs` 30, `protocol.rs` 22;
  the longest function is `measure_command` with 53 lines.
- Not pushed; `origin/main` is at `a235588`. Review: inline, by the owner's rule against subagents.

## Deviations, recorded in the design as Amendments 6, 7 and 8

- `--endpoint-env` may be left out on all three commands (the default endpoint file).
- A request that cannot be read is answered on standard output too (`kind: request`). A configuration
  that cannot be used ends with exit code 2, as the design's table of errors says.
- The check removes one code fence before the core's parsing, and writes the reply's files with the
  core's `write_candidate` before the static checks.
- "The model that answered" is the model named in the request. `Reply` does not hold the name an
  endpoint returns.
- `measure` stops starting trials once more than a fifth have failed for a reason of the endpoint; the
  rest count as not run. The cost per accepted answer includes the answers that were not accepted.
- `--quick` with `--trials` is refused. `measure` has no `--max-wait` and prints nothing between trials.
- Still open for the owner: Amendment 3 (a minimum wait of one second; a record that cannot be written
  fails the call), and Amendments 5 to 8 have had no comment.

## Current state

- Nothing in `native/` changed. No live call was made: every result above is against the stub. Whether
  the built-in set tells real models apart is task 5.3 and can fail.
- Calls to the vendor endpoint stay at 2 of 3 used; the last is for task 5.1.
- No research entry for this group: there is no measured result, and the deviations are in the
  amendments.
- The build directory of the crate is `D:\BuildArtifacts\cargo-target\lesscode-model`.
- Not yet in the pipeline (task 8.1). Remote branch `model-seam-gate-p4` (`37effda`) is kept for task 7.1.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. Group 5, live: 5.1 the six first cases through `call` and `check` (the last vendor call is used
   here); 5.2 one real refusal waited out; 5.3 the indicator on real models, and whether the built-in set
   tells them apart. Endpoint files outside the repository; no two runs against one upstream.
2. Then group 6 research tool, group 7 core, group 8 CI, release and docs.
