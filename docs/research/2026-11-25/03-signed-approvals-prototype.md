# R192: An approval signed with an SSH key can be checked the same way on Windows and Linux, against the project's list and GitHub

- Report date/time and timezone: 2026-11-25, about 14:10 Europe/Berlin.
- Execution interval: 2026-11-25, 13:50 to 14:00 Europe/Berlin (from the commit times of the
  prototype's repositories and the shell).
- Status: completed.
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/requirements.md), requirements
  approved; this is the prototype gate of its design ([prototype-gate.md](../../../.kiro/specs/signed-approvals/prototype-gate.md)).
- Hypothesis ID and predecessor: the owner's choice "Signed commits" on handover 88's open point 1,
  recorded in [R189](../2026-11-24/10-protect-stage-shared-frame.md) (approvals by script) and
  [handover 89](../../../.kiro/task_89_handover.md).
- Type: exploratory. The cases and their expected outcomes were planned before the runs (the
  owner was told what would be checked), but the gate file and this record were written after
  them. They are not preregistered.

## Decision

The design may rest on three things the prototype showed. First, `ssh-keygen -Y check-novalidate`
checks a commit's signature without the host's Git settings, the same way in three OpenSSH
versions (9.2 on Linux, 9.5 and 10.6 on Windows). Second, GitHub lists an account's signing keys to
anyone, without a token. Third, "the commit that last changed this write-off" can be found from
the file's history. On a copy of a real stage (scitools-hook, `cycles.py`) the whole check gave
the expected verdict in all five cases. The only thing it cannot show is the owner's passphrase
prompt in a real terminal. That waits for the owner's setup (task of requirement 5).

## Hypothesis and falsification

The check can be done by lesscode with the tools already on a developer's machine and in the CI
image (Git, OpenSSH's `ssh-keygen`, `curl`), with no new Rust dependency, and it reaches the same
verdict on Windows and Linux. Rejected if any of these holds: a signature good on one platform
fails on another; a tampered commit or a wrong namespace verifies; the check needs `gpg.ssh.*`
settings of the host; GitHub needs a token to list signing keys; the real stage's history cannot
name the commit that changed one write-off.

## Frozen inputs and method

- lesscode `380eb5f` (requirements). Scripts and logs under
  [docs/experiments/signed-approvals/prototype](../../experiments/signed-approvals/prototype/).
- Windows 11, Git 2.56.0.windows.3; OpenSSH 10.6p1 (Git Bash's `/usr/bin/ssh-keygen`) and 9.5p2
  (`C:\WINDOWS\System32\OpenSSH`, the one on PowerShell's PATH); `curl.exe` of Windows.
- Linux: `buildpack-deps:bookworm-scm` (`sha256:c1074fbc…`, the base of the CI image
  `rust:1.98.1-bookworm`), OpenSSH 9.2p1, Git 2.39.5, run with Docker Compose without network.
- Keys: throwaway Ed25519 keys made per run (`owner-test`, `agent-test`, one with a passphrase).
- `proto.sh`: three commits (unsigned; signed by "owner"; signed by "agent"), split each into the
  signed payload (the commit object without its `gpgsig` header) and the signature, check with
  `ssh-keygen -Y check-novalidate -n git`. Adverse controls: a tampered payload, the wrong
  namespace, signing with a passphrase key without a terminal.
- `chain.ps1`: on a copy of scitools-hook's `protect/protect-cycles` (`b885391`), find the newest
  commit whose entry for `arg_swap@138:8` differs from its first parent's, check its signature, and
  match its key against a signer list and the forge's keys.

## Evidence and results

| Case | Expected | Seen (Windows 10.6 / Linux 9.2; 9.5 where named) |
| --- | --- | --- |
| P1 owner-signed commit | good, owner's fingerprint | good, fingerprint equals `ssh-keygen -l` of the key |
| P2 agent-signed commit | good, agent's fingerprint (the key decides, not the signature) | the same |
| P3 unsigned commit | no signature | 0 bytes of signature |
| A1 one byte of the payload changed | fails | "incorrect signature", exit 255 |
| A2 namespace `file` in place of `git` | fails | "namespace does not match", exit 255 |
| C2 passphrase key, no terminal | cannot sign | "incorrect passphrase supplied", commit not written |
| 9.5 (System32) on P1, P2 | as 10.6 | the same lines |
| Forge, `norandom` | 4 signing keys without a token | HTTP 200, 4 keys, limit 60 per hour |
| Forge unreachable / unknown account | error | curl exit 6 / exit 22 (404) |
| R1 real stage, key not on GitHub | not counted, "not on the forge" | as expected, exit 1 |
| R2 the same, stand-in forge lists it | counted, with title and commit | as expected, exit 0 |
| R3 signer list names another key | not counted, "not in the signer list" | as expected, exit 1 |
| R4 original clone, `421c1ef` by script | not counted, "not signed" | as expected, exit 1 |
| R5 unsigned change after the signed approval | not counted, "not signed" | as expected, exit 1 |

Raw output: [windows-gitbash.log](../../experiments/signed-approvals/prototype/windows-gitbash.log),
[linux-bookworm.log](../../experiments/signed-approvals/prototype/linux-bookworm.log),
[real-stage-chain.log](../../experiments/signed-approvals/prototype/real-stage-chain.log), the
forge's answer with the keys shortened in
[forge-norandom-2026-11-25.json](../../experiments/signed-approvals/prototype/forge-norandom-2026-11-25.json).

Side finding: Git's own view (`git log --format=%G?`) said `N` for the signed commit on the
Windows copy, because no `gpg.ssh.allowedSignersFile` was set. In the Linux run with a file it said
`G` for the owner and `U` for the agent. Git's answer depends on host settings and on the committer
e-mail. That is why lesscode should not ask Git (requirement 7.2).

## Limits and interpretation

- The passphrase prompt in the owner's terminal, through Git on Windows, was not seen. A missing
  terminal made signing fail (C2), which is what keeps an agent out, but that a person *can* sign
  interactively is not shown yet. The owner's setup will show it.
- The prototype checked Ed25519 keys only. The account also has an RSA key; `ssh-keygen` handles RSA
  too, but no RSA case was run.
- An agent that replaces `ssh-keygen`, `git`, `curl` or lesscode itself on the owner's machine is
  outside what any check run there can stop. The check is worth what it is because anyone can run it
  again elsewhere: the signatures are in Git and the keys are on GitHub.
- The scripts are disposable PowerShell and shell scripts (no Python). One verification call on
  Windows went through `cmd /c` for a stdin redirect, against the owner's rule of PowerShell only;
  `chain.ps1` does the same with `Start-Process` and is the one the record relies on.
- The stand-in forge was a file. The command itself will take keys from GitHub only (requirement 2.6).

## Tool/spec consequence and next gate

The design uses `ssh-keygen -Y check-novalidate` and `curl` as external programs, in the way the
core already runs `git`. No HTTP client or signature crate joins the core. The gate's decision is
**proceed**. The design goes to the owner for approval.

## COBESY check

Outcome first; every case is replayable from the scripts; disproof conditions stated, though
written down after the runs; the one thing not shown (the interactive prompt) is named; no claim beyond the three OpenSSH
versions and Ed25519.
