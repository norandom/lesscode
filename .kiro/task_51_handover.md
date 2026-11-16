# Task 51 handover: group 7 of the model seam, the core on the seam

Written 2026-11-16, 15:56 Europe/Berlin (from the clock). Spec `model-seam`, tasks 7.1 to 7.4.

## What was done

- **7.1 The launcher** (`bb1bc90`). `native/src/seam.rs` finds `lesscode-model` (`LESSCODE_MODEL_BIN`,
  beside the running program, the search path), writes one request, reads one answer, passes standard
  error through. Scripted executable: `native/examples/scripted_model.rs`.
- **7.2 `regenerate` through the launcher** (`2ea4e26`). `HttpTransport`, `ureq_post`, `openai_call`, the
  hand-written `call` and the `ureq` dependency are gone. One code fence is removed before parsing; a
  failed turn prints its reason.
- **7.3 The receipt** (`e6203b3`). New turns: `"mode": "seam-1"` and only the settings reported as sent.
  Earlier receipts keep their form. The task could fail and did not.
- **7.4 Separation checks** (`bfab244`). Three steps in `tools/check_native.sh`: `separation-suites`,
  `separation-lock`, `separation-model-source`. The Dagger container mounts `model/`.
- Beside the tasks: `3bd6496` (lock files of `model/` and `tools/guide-optimizer` without `ureq`; the
  success line says `complete`) and `51f7f3c` (the stored receipt of the 7.3 test is a fixture under
  `native/tests/fixtures/model-seam/`).
- [R112](../docs/research/2026-11-16/01-model-seam-core-on-the-seam.md), design Amendment 10.
- Tasks 7.1 to 7.3 and 7.4 were each written by an Opus subagent, which the owner allowed for this group
  on 2026-11-16. Review, reruns, the two extra commits and the last pipeline run in the main context.

## Evidence

- `native` at `51f7f3c`: 400 of 400 tests, clippy, format and budgets clean; `native/Cargo.lock` has no
  `ureq`. `model`: 79 of 79. `tools/guide-optimizer`: 3 of 3. All `--locked`, toolchain 1.98.1.
- `dagger call check` on `51f7f3c`: "All native checks passed", 7 min 39 s.
- Each separation check failed on a planted violation and passed without it, directly and in Dagger.
  Logs: `D:\BuildArtifacts\lesscode\task-7.4\`.
- `regenerator.rs` 20,788 to 19,699 tokens (ceiling lowered to that); `seam.rs` 1,667.

## Current state

- Groups 1 to 7 are ticked. origin/main is `f7e07b8`; the group's commits and this handover are on main
  and **not pushed**.
- No live call was made in this group. macOS and the hosted jobs have not run.
- Two Dagger runs were killed with exit 137 in the main test step while the container host had about
  200 MB free of 8.3 GB; after `podman restart dagger-engine-v0.21.11` the run passed. The engine
  container is left running. `mvibe-proxy-proxy-1` is the owner's; it shows as exited two days ago and
  was not touched.
- For the owner, not acted on:
  - `--effort` and `--thinking-display` of `regenerate run` are accepted, not sent and not recorded;
    `--temperature` is sent only when given.
  - The refine loop's schedule still steps an effort and records effort and display in its attempt
    notes, although neither is sent (`regenerate refine` is not implemented).
  - A failed call's reason appears twice on the terminal.
  - From R111, still open: whether a refusal ends a trial of the guide tool; a reply cut off after a
    repair is asked again without the reasons.
- The owner has not commented on design Amendments 3, 5, 6, 7, 8 and 10.
- Build directories: `D:\BuildArtifacts\cargo-target\lesscode-core`, `...\lesscode-model`,
  `...\guide-optimizer`.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. Push, on the owner's word.
2. Group 8: 8.1 build and test `model/` in the pipeline (it can take over the build that
   `separation-model-source` does), 8.2 release `lesscode-model` for the five targets, 8.3
   documentation (humanizer pass before pushing README or guide text), 8.4 final validation, with one
   live `regenerate run` through the built `lesscode-model`.
