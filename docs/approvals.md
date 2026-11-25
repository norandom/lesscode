# Approving write-offs with a signed commit

A write-off is a fault of the protection stage that no test is to notice, with a reason a person
can check. An agent may propose one. Only the owner of the code approves it, and from release 2.2.0
an approval counts only when it comes in a commit signed with the owner's SSH key.
`step protect result` checks three things:

- the commit that last changed the write-off carries a good signature;
- the key is in the project's signer list, `.lesscode/allowed_signers`, as committed on the branch;
- the key is among the signing keys of the GitHub account that list names.

An agent with a shell can still write a proposal, set the mark and commit it. Without your
passphrase it cannot make the signature that counts. This guide sets up the key once, then shows
the approval itself. The commands are for PowerShell on Windows; where Linux or macOS differ, it
says so.

## 1. A signing key of its own

Make a key for approvals only, with a passphrase. Leave your login key (`~/.ssh/id_ed25519`) as it
is.

```powershell
ssh-keygen -t ed25519 -f $HOME/.ssh/lesscode_approvals -C "lesscode approvals"
```

It asks for the passphrase twice. Pick one you type yourself, and keep it out of any file an agent
could read.

Keep the key out of the SSH agent. Do not `ssh-add` it, and do not use it to log in anywhere, so
that no `AddKeysToAgent` setting adds it for you. The Windows agent keeps keys across restarts, so
look:

```powershell
ssh-add -L
```

Its public key must not be in the list ("The agent has no identities." is fine too). If it is
there, remove it with `ssh-add -d $HOME/.ssh/lesscode_approvals`.

Then check that it asks on every signature. Sign a word twice:

```powershell
'check' | ssh-keygen -Y sign -n git -f $HOME/.ssh/lesscode_approvals
```

Each run must ask for the passphrase before it prints a signature. Throw the signature away. If it
signs without asking, the key is in the agent or has no passphrase; give it one with
`ssh-keygen -p -f $HOME/.ssh/lesscode_approvals`.

## 2. The key on GitHub

The forge needs the public key as a signing key of your account. `gh` can add one only with the
signing-key scope, which a `gh` session does not have by default. Grant it, add the key, and take
the scope back:

```powershell
gh auth refresh -h github.com -s admin:ssh_signing_key
gh ssh-key add $HOME/.ssh/lesscode_approvals.pub --type signing --title "lesscode approvals"
gh auth refresh -h github.com -r admin:ssh_signing_key
```

Take the scope back every time. An agent working on your machine can use the same `gh` session.
While the scope is there, it could add a key of its own to your account, and that key would then
pass the forge's half of the check.

Check both. The forge lists signing keys to anyone, without a token:

```powershell
curl.exe -s https://api.github.com/users/<account>/ssh_signing_keys
gh auth status
```

The first shows the key under the title "lesscode approvals". In the second, the token scopes no
longer name `admin:ssh_signing_key`. (On Linux and macOS, write `curl` for `curl.exe`.)

## 3. The signer list in the project

The forge also lists keys you did not make for approvals, such as a key another machine signs with
or one a CI job holds. The signer list says which of them may approve in this project. Add your key
to it at the root of the project, on the branch of the stage:

```powershell
New-Item -ItemType Directory -Force .lesscode | Out-Null
Add-Content .lesscode/allowed_signers "github.com/<account> namespaces=`"git`" $(Get-Content $HOME/.ssh/lesscode_approvals.pub)"
git add .lesscode/allowed_signers
git commit -m "The signer list for approvals"
```

In a POSIX shell, the second line is:

```sh
printf 'github.com/<account> namespaces="git" %s\n' "$(cat ~/.ssh/lesscode_approvals.pub)" >> .lesscode/allowed_signers
```

The file has Git's allowed-signers form, one key per line. Every line names the same account as
`github.com/<account>`. Other principals on a line, such as your commit e-mail, are allowed and
ignored. `namespaces="git"` is the only option lesscode accepts. Lines starting with `#` are
comments.

This commit need not be signed. Anyone can edit the list, and that is why the forge is asked too: a
key counts only where both name it.

## 4. Git signs in this project only

In the project's clone:

```powershell
git config --local gpg.format ssh
git config --local user.signingkey "$HOME/.ssh/lesscode_approvals"
git config --local gpg.ssh.program C:/Windows/System32/OpenSSH/ssh-keygen.exe
```

The third line is for Windows only. Git for Windows puts its own `ssh-keygen` first on the path
of the programs it starts, and without this line it would sign with that one. In lesscode's own
tests that one sometimes failed to start under load. The line makes Git use the `ssh-keygen` of Windows, the
same one lesscode checks with. On Linux and macOS leave it out.

Do not set `commit.gpgsign`. Then only the commits you sign with `-S` are signed and ask for the
passphrase, and the agent's own commits go on as before. Read what Git will use:

```powershell
git config --show-origin --get-regexp '^(gpg\.|user\.signingkey|commit\.gpgsign)'
```

It should list the three lines from `.git/config` and nothing else.

## 5. Approving

The agent proposes a write-off in `write-offs.json` with its reason and without the mark. You
review the stage, then set `"approved": true` on each entry you approve, in your editor.

A signed commit approves every write-off whose entry it changes, with the reason as it stands in
that commit. Read the change first, in your own terminal, not one an agent drives or reads:

```powershell
git diff HEAD -- .kiro/specs/<name>/write-offs.json
```

Then the one command that approves:

```powershell
git commit -S -m "Approve the write-offs of <name>" -- .kiro/specs/<name>/write-offs.json
```

It asks for the passphrase once and commits that file alone, whatever else is staged. Run the
result:

```powershell
lesscode-core step protect result --declaration .kiro/specs/<name>/step.json
```

Each approval reads "Approved with the key "lesscode approvals" (<fingerprint>) of
github.com/<account>, in <commit> on <date>." The line "Signer list: github.com/<account>, 1 of 1
listed key(s) on the forge." follows. Commit `protection.json` with the stage, as
[the steps guide](steps.md#end-step-protect-result) says; that commit need not be signed.

A later unsigned change of an approved entry, a new reason for example, takes the approval back.
The entry waits until you approve it again.

The result asks GitHub only when a signed approval by a listed key needs it, once per run and
without a token. If GitHub cannot be reached, the result is "not judged" (exit 2), never a pass.

Git may print "gpg.ssh.allowedSignersFile needs to be configured", or show the commit as `N` in
`git log --format=%G?`. That is Git's own view and depends on its settings. lesscode does not ask
Git.

## 6. A forgotten signature

If you committed the approval without `-S`, the result says the approval "is not signed" and names
the repair. When that commit is still the head of the branch:

```powershell
git commit --amend -S --no-edit
```

When later commits follow it, or the branch is pushed already, set `"approved"` to `false` in one
commit and back to `true` in a signed one:

```powershell
git commit -m "Take the approval back" -- .kiro/specs/<name>/write-offs.json
git commit -S -m "Approve the write-offs of <name>" -- .kiro/specs/<name>/write-offs.json
```

Edit the file before each of the two commits. The second must change the entry, because only a
commit that changes it can approve it.

## 7. Replacing the key

1. Make the new key as in section 1, under another name, for example
   `$HOME/.ssh/lesscode_approvals_2027`.
2. Add it on GitHub as in section 2, scope granted and taken back.
3. Add its line to `.lesscode/allowed_signers` and commit. The list may hold both keys for a while.
4. Point Git at it: `git config --local user.signingkey "$HOME/.ssh/lesscode_approvals_2027"`.
5. Approve again, with the new key, the write-offs you want to keep counting. Use the two commits
   of section 6, the second signed.
6. Remove the old key from GitHub. Take its `id` from the list of section 2, then:

   ```powershell
   gh auth refresh -h github.com -s admin:ssh_signing_key
   gh api -X DELETE user/ssh_signing_keys/<id>
   gh auth refresh -h github.com -r admin:ssh_signing_key
   ```

7. Remove its line from `.lesscode/allowed_signers` and commit.

From the next run after step 6, approvals signed with the old key stop counting. The result shows
them waiting, with the old key's fingerprint and "not among the account's signing keys on the
forge". If the old key was lost or may have been seen by someone else, do step 6 first, at once,
and approve again afterwards.

## What the check does not stop

The check stops an agent that has a shell, the repository and your `gh` session, but not your
passphrase. It does not stop:

- a key held unlocked in an SSH agent, or kept without a passphrase;
- a passphrase typed where an agent can read it, such as a terminal the agent drives;
- a `gh` session that still has the signing-key scope;
- a key in the signer list whose private half sits somewhere else, such as a CI job;
- an agent that replaces `git`, `ssh-keygen`, `curl` or lesscode on your machine, or changes
  `gpg.ssh.program` in `.git/config` to a program of its own. Read the configuration with the
  command of section 4 before you sign.

Even then, anyone can run `step protect result` again on another machine and get the same answer
for the same commit. The signatures are in Git, the keys are on GitHub, and
the receipt names every key by its title and fingerprint, so an approval you did not make can be
seen.
