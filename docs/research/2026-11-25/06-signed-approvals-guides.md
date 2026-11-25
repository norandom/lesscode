# R195: The owner's guide holds when its commands are run; the real forge refuses a key it does not list

- Report date/time and timezone: 2026-11-25, about 17:40 Europe/Berlin
- Execution interval: about 16:00 to 17:35 Europe/Berlin (commits `da881b6` 16:10, `7d87f23` 17:35)
- Status: completed
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/tasks.md), tasks 4.1 and 4.2
- Hypothesis ID and predecessor: [R194](05-signed-approvals-receipt-cases.md), the receipt and the
  eleven cases
- Type: exploratory (a walk of the guide's commands, no preregistered expectation beyond the
  task's "every command run once")

## Decision

Task group 4 of the spec is done. [docs/approvals.md](../../approvals.md) covers the owner's key,
GitHub, the signer list, Git's signing settings, the approval, the repairs, replacing the key and
what the check does not stop. Its commands were run once with throwaway keys, except the `gh`
steps, which are the owner's in task 5.2. The steps guide, the skill `protect-before-reduce`, the
2.2.0 notes and requirement 4.3 of `brownfield-test-stage` now say that an approval is the owner's
signed commit. Group 5 (validation, the owner's key, the re-approval) can go on.

## Hypothesis and falsification

H: the commands of the guide, run as written on this machine, take a stage from "approved the old
way" to a signed approval that the built tool judges against the real forge, and the two repairs
it names work. It would be falsified by a command that fails as written, by a result other than
the guide says, or by a repair that does not make the signature found.

## Frozen inputs and method

- The test binary of `0b11aa1` (`D:\BuildArtifacts\lesscode-cargo-test\debug\lesscode-core.exe`;
  the guide commits changed no code).
- A clone, in the session's scratch directory, of scitools-hook's `protect/protect-cycles` at
  `b885391`, the stage of R190. The real clone was not touched.
- Windows 11, PowerShell 7, `C:\WINDOWS\System32\OpenSSH\ssh-keygen.exe` (9.5), Git for Windows,
  `curl.exe`. Two throwaway Ed25519 keys: one with a passphrase, one without. Both were deleted
  with the clone at the end.
- The signer list named the account `norandom`, so the result asked the real GitHub, without a
  token.

## Evidence and results

| Step of the guide | Seen |
| --- | --- |
| result before any mark | exit 1, "approved the old way, which no longer counts: approve it in a signed commit" |
| `ssh-add -L` | "The agent has no identities." |
| `'check' \| ssh-keygen -Y sign -n git -f <key with passphrase>` | waited for the passphrase; stopped after 120 s, and again after 15 s from a background job |
| the same with the key without passphrase | signed at once |
| `New-Item`, `Add-Content` of the signer line, its commit; the POSIX `printf` line in Git Bash | the line as the guide shows it |
| the three `git config --local` lines and `--show-origin` | three lines from `.git/config`, no `commit.gpgsign` |
| `"approved": true`, `git diff HEAD -- …`, `git commit -S … -- …` | a commit with a `gpgsig` header; Git printed "gpg.ssh.allowedSignersFile needs to be configured" and `%G?` gave `N` |
| result after the signed commit | exit 1, "signed with SHA256:LcveRNuk…, which is not among the account's signing keys on the forge"; "Signer list: github.com/norandom, 0 of 1 listed key(s) on the forge." |
| an unsigned approval at the head | "is not signed: sign it with `git commit --amend -S --no-edit`"; after it, signed and refused by the forge as above |
| an unsigned approval with a later commit | "set `approved` to false in one commit and back to true in a signed one"; after the two commits, signed and refused by the forge |
| `ssh-add -d` on a key not in the agent | "agent refused operation" (nothing to remove) |
| `ssh-keygen -p` | the passphrase changed |
| `curl.exe … /users/norandom/ssh_signing_keys` | four keys with `id` and `title` |
| `gh auth status` | scopes `gist`, `read:org`, `repo`, `workflow`, no signing-key scope |

This is the first time the built tool asked the real forge. Before this, the result had seen the
stand-in only (R193, R194). The refusal is the expected one: a throwaway key is on no account.

Side finding: in PowerShell 7, `ssh-keygen -N '""'` sets a passphrase of two quote characters, not
an empty one. `-N ''` gives an empty one. The guide asks for the passphrase interactively, so this
bears only on scripts that make test keys.

## Limits and interpretation

- The passphrase prompt was not seen. `ssh-keygen -Y sign` waited for input, and the same command
  without a passphrase did not wait, so the wait is the prompt. Whether a person can type the
  passphrase there, through Git on Windows with `gpg.ssh.program` set, is gate row G9. It is
  checked in 5.2.
- The counted path against the real forge was not seen. It needs a key on the account, which is
  the owner's act in 5.2.
- None of the `gh` steps were run: granting and taking back the scope, adding the key, and
  deleting a key through `gh api -X DELETE user/ssh_signing_keys/<id>`. The delete path is taken
  from GitHub's REST interface and is unverified here.
- The commands were run on Windows only. The POSIX variants differ only in the `printf` line, which
  ran in Git Bash, and in leaving out `gpg.ssh.program`.

## Tool/spec consequence and next gate

No code change. The guide sets `gpg.ssh.program` on Windows, after the finding of R194. Next is
group 5. In 5.1 the full suites and the Dagger run go on the group's commit. In 5.2 the owner
makes the key and adds it on GitHub, which settles G9 and the counted path. In 5.3 the three stage
branches are approved again.

## COBESY check

The result comes first. The steps are replayable from the guide, and the two things not shown
(the prompt and the counted path against the real forge) are named, each with the task that
shows it. No claim beyond Windows and Ed25519.
