# R193: The judge of signed approvals is in `step protect result`; the task order left the result's tests failing between two tasks

- Report date/time and timezone: 2026-11-25, about 15:05 Europe/Berlin
- Execution interval: 14:33 to 15:03 Europe/Berlin (commits `6fe9029` 14:39, `f57cdd3` 14:48,
  `bf24fb8` 14:56; the Linux run 14:56 to 15:03)
- Status: completed
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/tasks.md), tasks 2.1, 2.2 and 3.1
- Hypothesis ID and predecessor: [R192](03-signed-approvals-prototype.md), the prototype gate
- Type: exploratory (implementation record with one deviation from the task plan)

## Decision

`step protect result` now counts a write-off only when it is marked `"approved": true` and the
commit that last changed it carries a good SSH signature by a key that is both in the branch's
signer list and among the account's signing keys on GitHub. Approvals made through the fields
`approved_by`/`approved_at` wait as "approved the old way". Groups 3 to 5 of the spec can go on;
3.1 is done already.

## Hypothesis and falsification

The plan assumed that each task group could end with the existing suites passing. Falsified for
task group 2 as written: from task 2.1 on, five tests of `step_protect_result.rs` approve through
the old fields. The tests expected exit 0. After 2.1 they got exit 1, because those approvals no
longer count (requirement 6.1). The command judged no signed approval before 3.1 wired in the
judge. Nothing in 2.1 or 2.2 could make them pass without weakening them. The owner chose to carry
3.1 into the group ("Carry 3.1 in") rather than close the group with them failing.

## Frozen inputs and method

- lesscode `main` from `e657337` (handover 90) to `bf24fb8`; Rust 1.98.1, cargo-nextest 0.9.143,
  OpenSSH for Windows 9.5, Git for Windows; Linux through Dagger CLI 0.21.11 on Docker Desktop
  29.9.3.
- Keys made per test without a passphrase; the forge in every test is the stand-in of
  `native/tests/step_support/signing.rs`. No test reached the network.

## Evidence and results

| Point | Windows | Linux (Dagger, `bf24fb8`) |
| --- | --- | --- |
| after 2.1 (`6fe9029`) | numbers 7 of 7; the 5 result tests fail with exit 1 for 0 | not run (known failures) |
| after 2.2 (`f57cdd3`) | 531 of 536, the same 5 failing; judge 7 of 7 | not run (known failures) |
| after 3.1 (`bf24fb8`) | 537 of 537, 1 skipped; doctests, format, Clippy, budgets | every step 0, core 536 of 536, model 85 of 85 |

- Windows log: `D:\BuildArtifacts\lesscode\signed-approvals\nextest-group2-windows.log` (the
  run at `bf24fb8`'s content; the run after 2.2 was overwritten by it, its summary is the one quoted
  in the commit message of `f57cdd3`).
- Linux: `D:\BuildArtifacts\lesscode\native-evidence\linux-bf24fb8`.
- Budgets: `D:\BuildArtifacts\lesscode\signed-approvals\budgets-group2.json`. The judge was first
  one function of 821 tokens against the ceiling of 560 per function. It was split into
  `signatures` (387) and `against_forge`, with `judge` itself at 379. `approvals.rs` at 2,289 of
  2,800, `protect_result.rs` at 1,511.

## Limits and interpretation

- The counted path is tested only through the library entry with the stand-in forge. The
  command's own path to GitHub (`curl`) is tested in group 1 on recorded answers. No run has yet
  judged a real signed approval against the real forge. That comes with the owner's key (5.2) and
  the re-approval (5.3).
- The judge reads the history of `write-offs.json` once for all marked entries
  (`last_changes`), and checks each approving commit's signature once. Timing was not measured:
  the toy stages have a handful of commits.

## Tool/spec consequence and next gate

- Two additions to the design, recorded in tasks.md: `Judged` carries `on_forge` for the receipt's
  signer-list line (3.2), and `last_change` of 1.5 is the one-entry case of `last_changes`.
- The task-graph review of the plan missed that 2.1 breaks the result's binary tests until 3.1.
  For a later plan: a task that changes what counts should come with the task that wires it in.
- Next: group 3 (3.2 the receipt and `protection.json`, 3.3 the eleven cases), then groups 4 and 5.

## COBESY check

The outcome comes first. Every number above has a log or a commit. The deviation and the owner's
choice are stated as such. Nothing here claims that the real forge or the owner's passphrase
prompt has been seen.
