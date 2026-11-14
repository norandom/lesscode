# Task 49 handover: group 5 of the model seam, the live runs

Written 2026-11-14, 10:02 Europe/Berlin (from the clock). Spec `model-seam`, tasks 5.1, 5.2, 5.3.

## What was done

- **5.1 The six first cases** (`4fe4dd8`, `283d510`). `model/tests/live.rs`: two tests through the
  binary, run only when `LESSCODE_LIVE_ENV` names an endpoint file. `docs/experiments/model-seam/run-live.ps1`
  runs the cases one after the other. `call` with the turn of R102 and `check` pass on all six cases.
  [R108](../docs/research/2026-11-14/01-model-seam-first-cases-live.md).
- **5.2 A real refusal waited out** (`4fdcf57`). `docs/experiments/model-seam/run-refusal.ps1`: 16 large
  calls side by side met the local endpoint's rate limit; ten were refused after about 130 s, announced
  a wait of about 1 s, waited and were answered. The live condition of gate P3 is closed.
  [R109](../docs/research/2026-11-14/02-model-seam-real-refusal-waited-out.md).
- **5.3 The indicator on real models** (`9ea1f61`, closed by the owner's decision).
  `docs/experiments/model-seam/run-measure.ps1`. Built-in set, the core's parsing and static checks,
  20 trials: first replies accepted 19 (GLM 5.3, Mistral's API), 17 (Mistral Large), 20 (`gpt-6.1-sol`),
  19 (GLM 5.3, local endpoint) of 20; all 20 of 20 within two repairs; "good enough direct" for all.
  The set does not tell the models apart.
  [R110](../docs/research/2026-11-14/03-model-seam-indicator-on-real-models.md).
- **`check` names the reasons of a repaired answer** (owner's decision): `model/src/check.rs`,
  `rejected_lines`, used for a repaired and for a rejected outcome.

## Owner decisions of 2026-11-14 (design Amendment 9)

- Further calls to the Anthropic endpoint were approved for the check of case 4; one was used (four
  calls to that endpoint in all). None is planned.
- The indicator is a floor ("good enough for small steps"), not a ranking. The built-in set is kept;
  cost is reported and does not enter the verdict.
- The wait limit stays as designed: the time an endpoint holds a refused attempt counts against it.
- `check` prints the reasons of a repaired answer. The "still waiting" notices after a few seconds on a
  fast model are left as they are.

## Evidence

- Kept output: `docs/experiments/model-seam/live/`, `refusal/`, `measure/`.
- `cargo nextest run` in `model/`: 79 tests pass without `LESSCODE_LIVE_ENV` (the two live tests end at
  once). `cargo test --doc` passes. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`
  are clean. The budgets check exits 0 for `model/src`.
- The check's new lines are shown against the stub (`tests/check_command.rs`, the repaired case), not
  live.
- Review: inline, by the owner's rule against subagents.

## Current state

- Groups 1 to 5 are ticked. Nothing in `native/` changed.
- The owner has not commented on design Amendments 3, 5, 6, 7 and 8.
- `measure` reports no counts per task and no reasons of rejected replies, and keeps no record of its
  calls (a limit named in R110; not asked for).
- The local endpoint is the container `mvibe-proxy-proxy-1` of another project. It was started for this
  group and stopped again at its end; it is not removed.
- `model/` is not yet in the pipeline (task 8.1). Remote branch `model-seam-gate-p4` (`37effda`) is kept
  for task 7.1.
- The build directory of the crate is `D:\BuildArtifacts\cargo-target\lesscode-model`.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. Group 6: `tools/guide-optimizer` on the seam (`Live` calls the seam; its own request building, reply
   reading and wait are deleted; its repair turns use `repair.rs`). Read tasks.md group 6 first; the local
   endpoint has to be started for live runs.
2. Then group 7 core (the launcher, the receipt that states only what was sent, task 7.3), group 8 CI,
   release and docs.
