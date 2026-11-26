# R197: The owner's approval key is on GitHub and the signing scope is gone; the key has no passphrase, by the owner's decision

- Report date/time and timezone: 2026-11-26, about 11:30 Europe/Berlin
- Execution interval: about 11:10 to 11:25 Europe/Berlin (the owner's commands, then the agent's
  reads)
- Status: completed, with two checks of the task waived by the owner
- Spec: [signed-approvals](../../../.kiro/specs/signed-approvals/tasks.md), task 5.2
- Hypothesis ID and predecessor: [R196](../2026-11-25/07-signed-approvals-validation.md), the
  validation
- Type: exploratory (the owner's setup, following [docs/approvals.md](../../approvals.md)
  sections 1 and 2)

## Decision

Task 5.2 is closed. The forge lists the owner's key under "lesscode approvals", it is the same key
as the local public key, and the `gh` session no longer has the signing-key scope. The key has no
passphrase: the guide's own check showed it, and the owner decided to keep it so and to spend no
more time on this task. The passphrase prompt (gate row G9) and the signed test commit in a scratch
repository were therefore not run. The counted path against the real forge is first seen in 5.3,
on the real stage branches.

## Hypothesis and falsification

H: following sections 1 and 2 of the guide gives a signing key on the owner's account, a session
without the scope, and a key that asks for its passphrase on every signature. It would be
falsified by the key missing from the forge, a different key there, the scope still present, or a
signature made without a prompt.

## Frozen inputs and method

- Windows 11, PowerShell 7, `ssh-keygen` of Windows OpenSSH, `gh`, `curl.exe`.
- The owner ran the commands in their own terminal and pasted the output; the agent ran none of
  the `gh` steps and read only public data and the public key.

## Evidence and results

| Step | Seen |
| --- | --- |
| `ssh-keygen -t ed25519 -f ~/.ssh/lesscode_approvals -C "lesscode approvals"` | key made, `SHA256:2XmfsKgBelTz7qe5WadF6mBe7pXXWdWc0yl2vQ0Aohg` |
| `ssh-add -L` | "The agent has no identities." |
| `'check' \| ssh-keygen -Y sign -n git -f …` twice | both signed at once, without a prompt |
| `gh auth refresh -s admin:ssh_signing_key`, `gh ssh-key add … --type signing`, `gh auth refresh -r admin:ssh_signing_key` | each completed (owner's output) |
| `curl.exe -s https://api.github.com/users/norandom/ssh_signing_keys` | five keys; id 1368810 "lesscode approvals", created 2026-11-26 10:19 UTC |
| forge key against `~/.ssh/lesscode_approvals.pub` | equal |
| `gh auth status` | scopes `gist`, `read:org`, `repo`, `workflow`; no `admin:ssh_signing_key` |

The key is not in the agent, so the signature without a prompt means the key has no passphrase,
as the guide says. The owner was told the guide's repair (`ssh-keygen -p`) and declined it.

## Limits and interpretation

- Without a passphrase, any process that can read `~/.ssh/lesscode_approvals` can sign an
  approval that the check counts, an agent on this machine included. The check then cannot tell
  the owner's approval from an agent's. What keeps them apart is only the skill's rule that the
  agent does not sign. Requirement 5.1's passphrase is not met on this machine; the guide is
  unchanged and still asks for one.
- G9 (the prompt through Git on Windows) stays unseen, since no prompt exists.
- No signed commit was made in this task. 5.3's first signed approval is the first one judged
  against the real forge with a listed key.

## Tool/spec consequence and next gate

No code or guide change. Next is 5.3: on drift-rdp's `protect/protect-yuv` and
`protect/protect-shared-frame` and scitools-hook's `protect/protect-cycles`, the agent commits the
signer list with the owner's public key and sets the marks; the owner makes one signed commit per
branch.

## COBESY check

The result comes first, then the owner's decision and what it costs. The two checks not run are
named, with where the counted path is seen instead.
