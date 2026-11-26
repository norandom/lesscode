# Handover 94: drift-rdp's display path in one value, accepted by `step check` (R198)

Written 2026-11-26, about 15:10 Europe/Berlin. Predecessor: [task_93_handover.md](task_93_handover.md).
Research entries: [R197](../docs/research/2026-11-26/01-signed-approvals-owner-key.md) (signed
approvals 5.2) and [R198](../docs/research/2026-11-26/02-display-path-rework.md) (the rework).

## What was done

- **Signed approvals, closed by the owner.** 5.2 is done (`1ad2672`, R197). The owner's key is on
  GitHub and the `gh` signing scope is gone. The key has no passphrase, by the owner's decision.
  5.3 (the re-approvals) was stopped by the owner ("skip this whole thing"), noted in `1e68cdb`.
  The spec stays at 5.2.
- **Renderer rework in drift-rdp, R198 (registered `6927606`, completed `944904e`).** The owner
  chose this topic, the display path, and "tests only": no write-offs, and the three glue faults
  are reported. In drift-rdp's worktree `D:\BuildArtifacts\drift-rdp\lesscode-renderer-rework`,
  branch `lesscode/renderer-rework` (local, not pushed):
  - `FrameStores` went into `DisplayPathState`. The value in force is `(Reason, Option<Stores>)`:
    the layer with its stores, or the canvas. `register` refuses after a fallback, under the same
    lock. `connect` reads the path once. `frame_stores.rs` is deleted.
  - Commits: base `949127d`, declaration `86457e6`, change `2cf0fd8`, test `cea0924`, cuts
    `d391f3f`, `02142be` and `ba2963b`, verdict and ledger `4eeacaa`.
  - All eight expectations hold. Places holding the path 5 to 4, managed renderer values 3 to 2,
    shapes 4 to 3. Tokens 16769 to 16761, accepted with growth 0. No function has more unnoticed
    faults.
- **lesscode fix `a20ecd5`.** `step check` could not judge a step that deletes a scope file: the
  fault run of each side was given every scope file (exit 3, "cannot read"). Now each side runs
  faults only in the files it has. New test `a_step_that_deletes_a_scope_file_is_judged`. The
  budget ceiling is kept (`judge_by_runs` stays at 560 tokens or fewer). An amendment in R198,
  written before any verdict, records the stop and the fix.

## Deviations and findings, for the owner

- `step check` needed five attempts. Three were refused for size, and the cuts made the code
  smaller in ways that read less clearly: a tuple in place of a two-case enum, and merged imports.
  The fourth attempt stopped on the tool defect above.
- In lesscode's whole suite, `begin_gives_the_base_run_and_the_check_their_own_figures` failed in
  2 of 6 runs with the fix. It passed alone three times, and the one run without the fix passed.
  It is not explained and not attributed. It is worth a look: it compares a rounded time estimate.
- drift-rdp: one copy fails its first run of the unchanged tests on a fresh copy and passes the
  second. This happened in the plan and in the accepted check.
- I wrongly reported once that check 3 had passed the size gate, and corrected it.

## Current state

- Nothing of lesscode's is running.
- Kept: `D:\BuildArtifacts\lesscode\renderer-rework\` (33 MiB of evidence):
  - the binaries `bin\lesscode-core.exe` (`1e68cdb`) and `bin\lesscode-core-a20ecd5.exe`;
  - the plan, check and attempt logs, receipts and pages;
  - the fault outcomes;
  - the `before\` and `after\` measurements.
- Removed: the plan and check copies, about 27 GiB, and the scratch token copies.
- D: has 109.5 GiB free.
- The drift-rdp worktree stays (it holds the branch).

## Next steps, all on the owner's word

- Merge `lesscode/renderer-rework`, and the protection branches it carries, into drift-rdp; push.
- lesscode: tag 2.2.0 and push. These were open from before. `native\target` (7.5 GiB, inactive
  since 2026-11-08) was removed on the owner's word, 2026-11-26 about 15:20, with `cargo clean
  --manifest-path native/Cargo.toml --target-dir native/target`. D: then had 116.1 GiB free.
  Docs and skills that name `native/target/debug/lesscode-core.exe` as a default path now find no
  binary there. Builds on this machine go to `D:\BuildArtifacts\cargo-target`. CI and Dagger use
  their own container caches.
- Drift hardening (Slice B), meaning the measurement of one model's drift over time, stays closed
  by the owner (R104).
