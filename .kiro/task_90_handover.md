# Handover 90: signed approvals, group 1 done (signatures, signer list, forge, history)

Written 2026-11-25, about 14:25 Europe/Berlin. Predecessor: [task_89_handover.md](task_89_handover.md).
Spec: [signed-approvals](specs/signed-approvals/tasks.md); requirements, design and tasks approved
by the owner today. Prototype gate: proceed, [R192](../docs/research/2026-11-25/03-signed-approvals-prototype.md).

## What was done

- **The spec**, from the owner's choice "Signed commits" (handover 89, open point 1) and "should be
  via the ssh key, which pub key part we add to github via gh". Asked before the requirements, the
  owner chose: a key counts only when it is both in a signer list committed in the project and
  among the GitHub account's signing keys (the account already has four, one of them held by a CI
  job); approvals made by script stop counting and are signed again; a signing key of its own with
  a passphrase; write-offs only. At the design's approval: the signer list and the network only
  when an approval is to be judged (2.3 amended), and re-approval one signed commit per stage branch
  to be merged, three in all (6.2 amended).
- **1.1 (`b6c0686`)** test support: keys per test, signed commits with `-c` for one command,
  a signature grafted onto other content, the stand-in forge.
- **1.2 (`0ea39e8`)** `step/sshsig.rs`: a commit split into payload and signature, checked with
  `ssh-keygen -Y check-novalidate -n git`, fingerprints computed in the core.
- **1.3 (`81ad577`)** `step/signers.rs`: `.lesscode/allowed_signers`, one `github.com/<account>`.
- **1.4 (`5e0ad5c`)** `step/forge.rs`: the `Forge` seam, GitHub through `curl`; the stand-in
  implements it and counts how often it is asked.
- **1.5 (`0ada887`)** `revisions.rs` reads first-parent history, commit objects, files at a revision;
  `step/approvals.rs` finds the commit that last changed one write-off.

## Deviations and findings, for the owner

- **A guard test was changed (design C9).** `only_the_revisions_module_starts_git` allowed no file
  in `src/step` but `revisions.rs` to start a program. The design missed it; the full suite found
  it. It now names one program per module (`revisions.rs` git, `sshsig.rs` ssh-keygen, `forge.rs`
  curl) and still fails for any other file or program, including git from the two new modules.
- The commits of 1.2 to 1.4 were put together from the finished work through Git's index, one per
  task; only the last state (`0ada887`) was built and tested.
- The times first written into the spec and R192 were guessed and later than the clock; they were
  corrected from the clock and the commit times before the tasks (`bac9725`).

## Evidence

- Windows at `0ada887`: 528 of 528 tests (511 before, 17 new), doctests, format, Clippy with
  `-D warnings`, budgets within (the four new files at 333 to 1,587 tokens; `revisions.rs` at 2,409
  of 2,800). Log: `D:\BuildArtifacts\lesscode\signed-approvals\nextest-group1-windows.log`.
- Linux through Dagger at `0ada887` (14:21 to 14:27): every step 0, overall 0; the core's suite 527 of 527 (510 at `8b41287`, the 17 new ones all pass on Linux; one more runs on Windows only), model 85 of 85 as before
  (`D:\BuildArtifacts\lesscode\native-evidence\linux-0ada887`).

## Current state

- lesscode `main` at this handover's commit; `origin/main` at `9f6aab7`. Not pushed; push only on
  the owner's word.
- Nothing in the judging path uses the new modules yet: `step protect result` still decides by
  `approved_by`/`approved_at`. That changes in group 2 and 3.
- Clones untouched since handover 89 (drift-rdp on `protect/protect-shared-frame`, scitools-hook on
  `protect/protect-cycles`).
- Docker image `buildpack-deps:bookworm-scm` (515 MB) kept for the prototype's repeat; image
  `lesscode-stage/scitools-hook-tests:2026-11-25` still kept. No container of this group is left.
- Storage: D: 181.7 GiB free at the start of the group. Caches: `D:\BuildArtifacts\lesscode-cargo-test`
  (shared by this work), `native\target` 7.47 GiB inactive inside the source repository (cleanup on
  the owner's word, handover 89 item 4).

## Next

Group 2: 2.1 the mark `"approved": true` and the waiting texts (including "approved the old way"),
2.2 the judge (last change, signature, list once, forge once). Then group 3 (the command, the
receipt, the eleven cases), group 4 (guides), group 5 (validation, the owner's key, re-approval).
Other open points of handover 89 are unchanged: Rust labels through a helper, merging the
protection tests, the 2.2.0 tag, `native\target`.
