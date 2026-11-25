# Handover 92: signed approvals, group 3 done (the receipt and the eleven cases)

Written 2026-11-25, about 15:45 Europe/Berlin. Predecessor: [task_91_handover.md](task_91_handover.md).
Spec: [signed-approvals](specs/signed-approvals/tasks.md). Research entry:
[R194](../docs/research/2026-11-25/05-signed-approvals-receipt-cases.md).

## What was done

- **3.2 (`09f84b6`), the receipt and `protection.json`.**
  - A counted approval reads "Approved with the key "<title>" (<fingerprint>) of
    github.com/<account>, in <commit> on <date>." as before.
  - An approval that does not count now reads "Not counted: <the refusal's words>." beside it,
    or the old-way words. An unmarked write-off stays "Not approved.".
  - After the write-offs comes "Signer list: github.com/<account>, <n> of <m> listed key(s) on the
    forge.". Where no listed key signed anything, the forge is not asked and the line says so.
  - `protection.json` gains top-level `signers` (`account`, `listed`, `on_forge`) and `refusal`
    on each waiting fault whose write-off was refused. `approval` was already there. No field was
    removed, and the receipt id stays `step-protection-v1`.
  - The module comment no longer says the tool cannot know who approved.
  - Two new tests in `step_protect_result.rs` read all of this from the receipt and the file.
- **3.3 (`0b11aa1`), the eleven cases.** They are tests of `native/tests/step_protect_signed.rs`,
  each with its exit code and words, through `run_result_with` and the stand-in forge. Case 2 also
  applies the repair it names (`--amend -S`) and finds it counted. The stage helpers moved to
  `native/tests/step_support/protect.rs`, shared by both test files.

## Deviations and findings, for the owner

- **Test signing on Windows used the wrong `ssh-keygen`.** Git for Windows puts its own MSYS
  `usr\bin` first on the PATH of its children. So the tests' signed commits were made by
  `/usr/bin/ssh-keygen`, not by the Windows OpenSSH one the tool starts. Under parallel load it
  sometimes failed to start ("failed to write commit object", nothing else): 2 of 5 runs, once in
  a test older than this group. The test support now sets `gpg.ssh.program` to the `ssh-keygen` on
  the test's PATH; 4 of 4 runs and the full suite green after. The tool's code is unchanged by this.
  It matters for 4.1: when the owner signs on Windows, the same thing happens unless the guide sets
  `gpg.ssh.program`. That also bears on which program asks for the passphrase (5.2).
- The signer-list line has a second form ("the forge was not asked"), with `on_forge` null. The
  design named only the "n of m" form.

## Evidence

- Windows at `0b11aa1`: 550 of 550 tests (1 skipped), doctests, format, Clippy with
  `-D warnings`, budgets within (`protect_numbers.rs` 2,217 of 2,800 tokens, `protect_receipt.rs`
  1,598, `protect_result.rs` 1,535). Logs: `D:\BuildArtifacts\lesscode\signed-approvals\`
  `nextest-3.2-windows.log`, `nextest-3.3-windows.log`, `budgets-3.2.json`, `budgets-3.3.json`.
- Linux through Dagger at `0b11aa1` (15:33 to 15:41): every step 0, overall 0; core 549 of 549
  (1 skipped; one test runs on Windows only, as before), model 85 of 85, all eleven cases passed.
  `D:\BuildArtifacts\lesscode\native-evidence\linux-0b11aa1`; Dagger log
  `D:\BuildArtifacts\lesscode\signed-approvals\dagger-3.3.log`.

## Current state

- lesscode `main` at this handover's commit; `origin/main` at `9f6aab7`. Not pushed; push only on
  your word.
- Clones untouched (drift-rdp on `protect/protect-shared-frame`, scitools-hook on
  `protect/protect-cycles`). Their stages still report the earlier approvals as "approved the old
  way" (exit 1) until 5.3.
- Containers: only the shared `dagger-engine-v0.21.11` is running; this group started no other.
- Storage: D: 143.3 GiB free at the end (166.6 at the start). The lesscode caches barely moved:
  `D:\BuildArtifacts\lesscode-cargo-test` 15.49 GiB (reused), `D:\BuildArtifacts\lesscode` 5.70 GiB.
  Most of the drop is in Docker Desktop: the shared Dagger engine's volume is 46.06 GiB. That
  engine and its volume are shared, so I did not prune them. Both limits hold (lesscode caches
  21.2 of 40 GiB; D: above 50 GiB). No isolated build directory was made. Retained evidence grew by
  the Linux export `linux-0b11aa1` and the four logs and budgets files above (a few MiB).

## Next

Group 4: 4.1 the owner's guide `docs/approvals.md` (with the `gpg.ssh.program` point above), 4.2
the steps guide, the skill, the release notes and the stage spec. Then group 5: 5.1 validation,
5.2 your key (your act, guided), 5.3 the re-approval on the three stage branches. The open points
of handover 89 are unchanged.
