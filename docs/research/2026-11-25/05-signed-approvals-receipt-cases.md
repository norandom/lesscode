# R194: The receipt shows every approval with its key or its refusal; the eleven cases hold on Windows and Linux

- Report date/time and timezone: 2026-11-25, about 15:45 Europe/Berlin
- Execution interval: 15:08 to 15:41 Europe/Berlin (commits `09f84b6` 15:17, `0b11aa1` 15:32;
  the Linux run 15:33 to 15:41)
- Status: completed
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/tasks.md), tasks 3.2 and 3.3
- Hypothesis ID and predecessor: [R193](04-signed-approvals-judge.md), the judge
- Type: confirmatory (the eleven cases of the design's testing strategy, written before the run)

## Decision

Task group 3 of the spec is done. The receipt of `step protect result` names each counted
approval by key title, fingerprint, account, commit and date. It shows each approval that does not
count beside them, with its reason, commit, key and repair, and one line for the signer list.
`protection.json` carries the same facts. The eleven cases pass on both platforms. Groups 4
(guides) and 5 (validation, the owner's key, the re-approval) can go on.

## Hypothesis and falsification

H: the judge of R193, wired into the result, gives each of the eleven cases of the design the
exit code and words the design names, on Windows and on Linux, with no network.
It would be falsified by any case giving another exit code, by words missing from the receipt,
or by a case passing on one platform only. Not falsified: 11 of 11 on both.

One finding outside the hypothesis, about the tests and not the tool: on Windows, Git signed the
tests' commits with the MSYS `ssh-keygen` of Git for Windows (`/usr/bin/ssh-keygen`, first on the
PATH Git gives its children), not with the Windows OpenSSH `ssh-keygen` the tool starts. Under
parallel load that program now and then failed to start ("failed to write commit object" with no
other words): 2 of 5 runs of the three protect suites at 16 threads, once in a test older than
this group. This session's Git Bash showed the same failure to start (`0xC0000142`) at the
same time. The test support now sets `gpg.ssh.program` to the `ssh-keygen` on the test process's
PATH. After that, 4 of 4 runs and the full suite were green. Four runs do not prove the flake is
gone. They show it is much rarer. The tool's code is unchanged by this.

## Frozen inputs and method

- lesscode `main` from `6caae83` (handover 91) to `0b11aa1`; Rust 1.98.1, cargo-nextest 0.9.143,
  OpenSSH for Windows (`C:\Windows\System32\OpenSSH`), Git 2.56.0.windows.3; Linux through Dagger
  CLI 0.21.11 on Docker Desktop.
- Keys made per test without a passphrase. In every test the forge is the stand-in of
  `native/tests/step_support/signing.rs`. No test reached the network.
- The cases: `native/tests/step_protect_signed.rs`, through `run_result_with` on the toy stage of
  `helper` (case 8 on the stage of `three`). The receipt and file tests of 3.2:
  `native/tests/step_protect_result.rs`.

## Evidence and results

| Case (design, testing strategy) | Exit | Words read |
| --- | --- | --- |
| 1 signed by a listed key on the forge | 0 | approval line, "1 of 1 listed key(s) on the forge" |
| 2 unsigned on the head | 1, then 0 | "is not signed: sign it with `git commit --amend -S --no-edit`"; the amend counts |
| 3 grafted signature | 1 | "does not verify" |
| 4 key not listed | 1 | "not in the signer list", the forge not asked |
| 5 listed, not on the forge | 1 | "not among the account's signing keys on the forge", by fingerprint |
| 6 removed between runs | 0, then 1 | `refusal.kind` = `not_on_forge`, `signers.on_forge` = 0 |
| 7 unsigned change after a signed approval | 1 | "is not signed" |
| 8 one signed commit, two entries | 0 | "written off 2, waiting 0" |
| 9 forge unreachable | 2 | "GitHub cannot be reached", nothing written |
| 10 no list; `cert-authority` | 2; 3 | "no signer list at .lesscode/allowed_signers"; the option named, nothing written |
| 11 old fields only, even signed | 1 | "approved the old way", the forge not asked |

| Point | Windows | Linux (Dagger, `0b11aa1`) |
| --- | --- | --- |
| after 3.2 (`09f84b6`) | 539 of 539, 1 skipped; doctests, format, Clippy, budgets | not run |
| after 3.3 (`0b11aa1`) | 550 of 550, 1 skipped; doctests, format, Clippy, budgets | every step 0, core 549 of 549 (1 skipped), model 85 of 85 |

- Windows logs: `D:\BuildArtifacts\lesscode\signed-approvals\nextest-3.2-windows.log`,
  `nextest-3.3-windows.log`; budgets `budgets-3.2.json`, `budgets-3.3.json` beside them
  (`protect_numbers.rs` 2,217 of 2,800 tokens, `protect_receipt.rs` 1,598, `protect_result.rs`
  1,535; no function over 560).
- Linux: `D:\BuildArtifacts\lesscode\native-evidence\linux-0b11aa1`; the Dagger log
  `D:\BuildArtifacts\lesscode\signed-approvals\dagger-3.3.log`.

## Limits and interpretation

- As in R193, the counted path is tested only through the library entry with the stand-in. A real
  signed approval against GitHub comes with the owner's key (5.2) and the re-approval (5.3).
- The words of a refusal shorten the commit to 12 characters in the receipt. `protection.json`
  carries the full hash.
- Where every good signature is by an unlisted key, the forge is not asked. The signer-list line
  then says so, and `on_forge` is null rather than a number. The design gave only the "n of m"
  form, so this second form is my addition.

## Tool/spec consequence and next gate

- Recorded in tasks.md: the "Not counted: …" form of the receipt line, the second form of the
  signer-list line, the shared test helpers in `tests/step_support/protect.rs`, and the
  `gpg.ssh.program` finding.
- For the guide (4.1): a person signing on Windows with Git for Windows uses its MSYS
  `ssh-keygen` unless `gpg.ssh.program` says otherwise. The prototype (R192) saw the owner's
  passphrase prompt as not yet run; which `ssh-keygen` asks for it bears on that prompt.
- Next: group 4 (the guides, the skill, the release notes), then group 5.

## COBESY check

The outcome comes first. Every number has a log or a commit. The test-support finding is kept
apart from the hypothesis and labelled as being about the tests. Nothing here claims the real
forge or the owner's passphrase prompt has been seen.
