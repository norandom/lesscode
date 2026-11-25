# Handover 91: signed approvals, group 2 done (the judge), with 3.1 carried in

Written 2026-11-25, about 15:08 Europe/Berlin. Predecessor: [task_90_handover.md](task_90_handover.md).
Spec: [signed-approvals](specs/signed-approvals/tasks.md). Research entry:
[R193](../docs/research/2026-11-25/04-signed-approvals-judge.md).

## What was done

- **2.1 (`6fe9029`)** A write-off carries the mark `"approved": true`. The judge fills in the
  counted approval (`Signed`: account, key title, fingerprint, commit, committer date) or the
  refusal (not signed, does not verify, not in the signer list, not on the forge; with commit, key
  and repair). Neither is ever read from the file. `approved_by`/`approved_at` still parse and
  decide nothing. A fault waits with the refusal's words, with "approved the old way, which no
  longer counts: approve it in a signed commit", or as before. The receipt line of a counted
  approval names the key, fingerprint, account, commit and date.
- **2.2 (`f57cdd3`)** `approvals::judge`: last change per marked entry (history read once),
  signature checked once per commit, signer list read once when a good signature needs it, forge
  asked once when a listed key needs it. Unsigned approval repair: `git commit --amend -S
  --no-edit` on the branch head, else approve again in a signed commit. Missing list, unreachable
  forge, or `ssh-keygen` that cannot start: not judged. Bad list: bad input. 7 library tests in
  `native/tests/step_protect_judge.rs`.
- **3.1 (`bf24fb8`)** `step protect result` runs the judge between reading the stage and
  counting. `run_result_with(args, forge, out)` is the library entry. The command passes GitHub
  and has no forge option; a test checks `--help`. The result's approving tests now commit the mark
  and a signer list in a signed commit and run through the library entry with the stand-in forge.
  The binary tests stop before the forge.

## Deviations and findings, for the owner

- **3.1 was carried into this group, by your choice.** The plan's order left five tests of
  `step_protect_result.rs` failing from 2.1 until 3.1: they approved through the old fields, which
  no longer count. The commits of 2.1 and 2.2 say so; only `bf24fb8` is green on both platforms.
- `judge` was split into three functions to stay within the 560-token ceiling per function (it was
  821 at first). Nothing was exempted.
- Two small additions to the design, noted in tasks.md: `Judged.on_forge` (for the receipt line of
  3.2) and `last_changes` (several entries, one history read).

## Evidence

- Windows at `bf24fb8`: 537 of 537 tests (1 skipped), doctests, format, Clippy with `-D warnings`,
  budgets within (`approvals.rs` 2,289 of 2,800 tokens; `protect_result.rs` 1,511). Log:
  `D:\BuildArtifacts\lesscode\signed-approvals\nextest-group2-windows.log`; budgets receipt
  `D:\BuildArtifacts\lesscode\signed-approvals\budgets-group2.json`.
- Linux through Dagger at `bf24fb8` (14:56 to 15:03): every step 0, overall 0; core 536 of 536
  (one test more runs on Windows only, as before), model 85 of 85.
  `D:\BuildArtifacts\lesscode\native-evidence\linux-bf24fb8`.

## Current state

- lesscode `main` at this handover's commit; `origin/main` at `9f6aab7`. Not pushed; push only on
  your word.
- The receipt shows a counted approval, but not yet the signer-list line, the refusals beside the
  approvals, or `approval`/`refusal`/`signers` in `protection.json` as the design has them. That
  is 3.2. Counted write-offs already carry `approval` in `protection.json`, because `WriteOff` is
  serialized there.
- Clones untouched (drift-rdp on `protect/protect-shared-frame`, scitools-hook on
  `protect/protect-cycles`). Run today, their stages would report the earlier approvals as "approved
  the old way" (exit 1) until 5.3.
- Containers: only the shared `dagger-engine-v0.21.11` is running; this group started no other
  container. Docker images of handover 90 kept as before.
- Storage: D: 166.6 GiB free at the end (176.0 at the start). Caches:
  `D:\BuildArtifacts\lesscode-cargo-test` 15.47 GiB (reused, shared by this work), within the
  40 GiB limit together with the rest. No isolated build directory was made.

## Next

Group 3 without 3.1: 3.2 the receipt and `protection.json` (signer-list line, refusals beside
approvals, the `signers` field, module comment), 3.3 the eleven cases through the library entry.
Then group 4 (guides: `docs/approvals.md`, steps guide, skill, release notes), group 5 (validation,
your key, the re-approval on the three stage branches). Open points of handover 89 are unchanged.
