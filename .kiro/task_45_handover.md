# Task 45 handover: group 1 of the model seam, the crate and one call

Written 2026-11-13, 15:02 Europe/Berlin (from the clock). Spec `model-seam`, tasks 1.1 to 1.4.

## What was done

- New crate `model/` (`lesscode-model`, library only so far), with its own lock file and its own budget
  file `model/complexity-budgets.json` at the default ceilings.
- 1.1 `request.rs`, `client.rs`: one attempt, `send`, through the client library `genai` 0.7.4. Only stated
  settings go on the wire; text is read from a string and from parts, without the reasoning; stop reason,
  tokens and a cut-off reply are reported; the second request style gets its version segment when the
  address has none.
- 1.2 `structure.rs`: a reply says whether its text, with or without a code fence, has the structure
  asked for. Nothing about the structure is sent.
- 1.3 `error.rs`, `endpoint.rs`, `notice.rs`: one line per kind of failure; text from an endpoint has the
  key removed and is cut to 300 characters; an unusable endpoint file is named with what is wrong; the
  notice lines for the terminal.
- 1.4 `record.rs`: one JSON line per call to the path the caller names, nothing without a path, no key.

## Evidence

- `cargo nextest run` in `model/`: 21 tests pass, all against a local stub endpoint
  (`model/tests/stub_endpoint/`); no test calls a real endpoint.
- `cargo clippy --all-targets` and `cargo fmt --check` clean.
- Budgets: `lesscode-core budgets --config model/complexity-budgets.json --root model/src`: every file
  within the default ceilings. The longest function is `send` in `client.rs`, 516 of 560 tokens: the next
  change to it has to split it.
- Commits `b979c98` (1.1), `f41593c` (1.2), `1a44fcb` (1.3) and `97babac` (1.4).

## Deviations, recorded in the design as Amendment 2

- No response format is sent for a structure; the reply is checked locally.
- `CallError` has the kind `Unreadable`; `Refused` keeps the header's stated wait as written.
- `client.rs` offers one attempt; the call with waiting is group 2.

## Current state

- Nothing in `native/` changed. `tools/guide-optimizer` and the prototypes are untouched.
- The build directory of the crate is `D:\BuildArtifacts\cargo-target\lesscode-model` (set
  `CARGO_TARGET_DIR` to it; the global one is shared with the core).
- Not yet in the pipeline (task 8.1).

## Next steps

1. Group 2: `wait.rs`. Task 2.1 builds the call with waiting on `send`, from the rule shown in R106
   (`docs/experiments/model-seam/prototype-wait`): stated wait from the header, else from the message,
   else ten seconds; the wait made is the larger of that and twice the wait that just failed; limit ten
   minutes; time injected in tests. Task 2.2: "still running" notices and the answer limit of sixty
   minutes (the library's web configuration has a timeout).
2. Then group 3 (`repair.rs`), group 4 (`call`, `check`, `measure` commands).
