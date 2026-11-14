# Task 50 handover: group 6 of the model seam, the guide tool on the seam

Written 2026-11-14, 11:02 Europe/Berlin (from the clock). Spec `model-seam`, task 6.

## What was done

- **6 The research tool on the seam** (`43fe8a8`). `tools/guide-optimizer` has no client of its own:
  `Live::chat` is one call of `lesscode_model::call` (default limits, notices to the terminal, no seam
  record), the `Model` trait speaks the seam's `CallRequest`, `Reply` and `CallError`, and
  `Authoring::one` runs the seam's `with_repairs` with the tool's `judge` as the check. Deleted: the
  tool's `Reply`, `Live::request`, `Live::read`, `text_of`, `retried`, `WAITS`, the use of `ureq_post`,
  `Authoring::attempt` and the tool's `repair_request`.
  [R111](../docs/research/2026-11-14/04-model-seam-guide-tool-on-the-seam.md).
- The implementation was written by an Opus subagent, on the owner's instruction of 2026-11-14 for this
  group. Decisions, review of the diff, the comparison with the state before and the live run were made
  in the main context.

## Evidence

- `cargo nextest run` in `tools/guide-optimizer`: 3 tests pass. `cargo clippy --all-targets -- -D warnings`
  and `cargo fmt --check` are clean. The crate has no doc tests.
- Dry runs before (`d7902e7`) and after: `summary.json`, `ledger.jsonl`, candidates and logs are
  identical for the optimisation and for a judgement with two repairs. `trials.jsonl` differs only in
  the stop word (`end_turn` became `complete`).
- Live: `docs/experiments/model-seam/run-guide-tool.ps1`, output in
  `docs/experiments/model-seam/guide-tool/`. Four trials on the local endpoint (GLM 5.3), 9 calls,
  137 s: first replies admitted 1 of 4, within two repairs 3 of 4 (Rust not admitted: two replies cut
  off at 4,000 output tokens). 23,701 input and 25,798 output tokens. The 2.29 USD in the ledger is the
  tool's fallback price, not a price paid.

## Current state

- Groups 1 to 6 are ticked. Nothing in `native/` or `model/` changed in this group.
- `43fe8a8` and this handover are on main and **not pushed**.
- Changed with the move, the seam's rules (R111, for the owner, not acted on): a failed call ends the
  trial; a refused reply is asked again within the repairs; the repair text shows the whole reply and
  says "answer"; the records use the seam's stop words. After a repair reply is cut off, the next call
  is the first request, without the reasons.
- The `mipro` engine of the tool was compiled and linted, not run.
- The local endpoint container `mvibe-proxy-proxy-1` was started by the owner and is left running.
- The build directory of the tool is `D:\BuildArtifacts\cargo-target\guide-optimizer`; the seam's is
  `D:\BuildArtifacts\cargo-target\lesscode-model`.
- The owner has not commented on design Amendments 3, 5, 6, 7 and 8. `model/` is not yet in the pipeline
  (task 8.1). Remote branch `model-seam-gate-p4` (`37effda`) is kept for task 7.1.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. Group 7, the core on the seam: 7.1 the launcher (read branch `model-seam-gate-p4` and R107 first),
   7.2 `regenerate` through the launcher and the hand-written request out of the core, 7.3 the receipt
   that states only what was sent (carried from the gate, can fail), 7.4 the check that the core stays
   free of models. This is the only group that changes `native/`.
2. Group 8: pipeline, release for the five targets, documentation, final validation.
