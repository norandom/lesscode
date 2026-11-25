# Requirements: approvals an agent cannot make

Generated 2026-11-25, 14:05 (Europe/Berlin), from the project description below and the owner's
four answers of 13:58. Approved by the owner on 2026-11-25 ("approved, go on to the design").
Related spec: [brownfield-test-stage](../brownfield-test-stage/requirements.md),
whose requirement 4.3 this spec makes stricter.

## Project description

**Who has the problem.** The owner of the code under a brownfield test stage, who approves
write-offs, and lesscode's `step protect result`, which counts them.

**The situation today.** A write-off counts as approved when its entry in `write-offs.json` has a
reason, `approved_by` and `approved_at`, committed on the branch. Anyone with shell access can write
these fields, an agent included. In R188 to R190 (the stages on drift-rdp's `yuv.rs` and
`shared_frame.rs` and on scitools-hook's `cycles.py`) the owner approved by running a PowerShell
script the agent had prepared. The owner asked why approvals come by a script, and on 2026-11-25
chose "Signed commits" as the kind of approval an agent cannot make (handover 89, open point 1).
The receipt prints who approved and when, so that a false approval can be seen. Nothing stops one
from being written.

**What should change.** `step protect result` counts an approval only when it comes in a commit
signed with the owner's SSH key. The public part of that key is added to the owner's GitHub
account as a signing key with `gh` (owner, 2026-11-25: "should be via the ssh key, which pub key
part we add to github via gh"). An agent with shell access but without the owner's key can still
propose a write-off. It cannot make one count.

## The owner's answers of 2026-11-25, 13:58

Asked before these requirements were written, each chosen as recommended:

1. **Which keys count: the project's list and GitHub, both.** A key counts only when it is in a
   signer list committed in the project *and* among the signing keys of the owner's GitHub account.
   The account already has four signing keys. One of them, "GitHub Actions Signing Key -
   observability-k8s", has its private half in a CI job. Neither of the other machines' keys nor
   the CI key counts unless the owner adds it to the project's list.
2. **Earlier approvals: approve again by signing.** The approvals made by script stop counting. The
   owner approves them again in one signed commit per clone. The protection branches are not merged,
   so nothing that has already left the clones depends on them.
3. **A signing key of its own.** A new key with a passphrase, never held in the SSH agent, so that
   each signature needs the owner to type it. `~/.ssh/id_ed25519` stays as it is.
4. **Write-offs only.** The approvals of a Kiro spec's phases in `spec.json` stay as they are.

## Words

- *Signed approval*: a write-off whose approval came in a commit signed with an allowed key.
- *Allowed key*: a public key that is both in the project's signer list and among the signing keys
  of the GitHub account the list names.
- *Signer list*: the file in the project under work that names the owner's GitHub account and the
  keys that may approve there.
- *Forge*: GitHub, where the owner's account lists its signing keys for anyone to read.

## Boundary context

- **In scope**: how `step protect result` decides that a write-off is approved, what its result and
  receipt say about each approval, the owner's setup of the key, the re-approval of the earlier
  approvals, the tests, the guides and the skill `protect-before-reduce`.
- **Out of scope**: the approvals of Kiro spec phases in `spec.json`; the release-signing key
  (a zipsign key in the GitHub `release` environment, [release-signing.md](../../../docs/release-signing.md));
  signing of other commits; branch protection on the forge; the merge of protection branches.
- **Adjacent expectations**: the forge lists a user's signing keys publicly and lets only the
  account, with a token that has the signing-key scope, add one. Git stores a signature in the
  commit it signs. The stage's other rules (requirement 4 of `brownfield-test-stage`, except 4.3's
  "who approved it and when", which this spec replaces) stay as they are.

## Requirements

### Requirement 1: An approval counts only when it is signed
**Objective:** As the owner of the code, I want only my signed approvals to count, so that an
agent with shell access can propose a write-off but cannot approve it.

#### Acceptance Criteria
1. When `step protect result` reads an approved write-off, it shall count the write-off as approved
   only when the commit on the branch that last changed that write-off carries a good signature
   made with an allowed key.
2. If a commit without such a signature changes a write-off after its signed approval, then
   `step protect result` shall no longer count that write-off as approved.
3. If an approval does not count, then `step protect result` shall count the fault as waiting and
   shall say which of these holds: the commit is not signed, the signature does not verify, the
   key is not in the signer list, or the key is not among the account's signing keys on the forge.
4. `step protect result` shall take the approver and the date of an approval from the signed
   commit: the key and the commit's date. Fields written in the file shall not decide either.
5. The stage shall go on accepting a proposal to write off from anyone, signed or not, and shall
   count its fault as waiting until a signed approval exists.
6. The stage shall count one signed commit as the approval of every write-off that commit changed.
7. `step protect result` shall keep its exit codes: 0 when every chosen function is protected
   enough, 1 when something waits, 2 when it cannot judge.

### Requirement 2: Which keys count
**Objective:** As the owner of the code, I want a key to count only when my project and my GitHub
account both name it, so that neither an edited file nor a key held by a CI job can approve alone.

#### Acceptance Criteria
1. `step protect result` shall count a key only when it is in the signer list committed on the
   branch and among the signing keys of the GitHub account that list names.
2. If a key is in the signer list but not among the account's signing keys on the forge, then
   `step protect result` shall not count approvals signed with it, and shall name the key by its
   fingerprint.
3. If the project has no signer list, or the list names no account, then `step protect result`
   shall not judge (exit 2) and shall say what is missing.
4. If the forge cannot be reached or does not answer, then `step protect result` shall not judge
   (exit 2) and shall say so. It shall never count an approval without the forge's answer.
5. When the owner removes a key from the account on the forge, `step protect result` shall stop
   counting the approvals signed with it from its next run on.
6. `step protect result` shall take the account's keys from the forge only. A stand-in for the
   forge shall exist in the test suite alone and shall not be reachable from the command line.

### Requirement 3: What the result and the receipt show
**Objective:** As the owner of the code, I want every approval shown with the key and commit it
came from, so that I see at once an approval I did not make.

#### Acceptance Criteria
1. The receipt shall show for each counted approval the account, the key's title on the forge, the
   key's fingerprint, the commit and its date.
2. The receipt shall show the account the signer list names and the number of keys that count, out
   of those listed.
3. Where an approval does not count, the receipt shall show it with its reason from 1.3, beside
   the approvals that count.
4. `step protect result` shall write in its result file the same facts as the receipt for every
   approval, counted or not.

### Requirement 4: The owner's act of approving
**Objective:** As the owner of the code, I want to approve the reviewed write-offs with one signed
commit, so that approving stays one act of mine and is not a script an agent prepared.

#### Acceptance Criteria
1. The guide shall show the owner one command that commits the approval of the reviewed write-offs
   with a signature, asking for the key's passphrase once.
2. If the owner's approval commit is not signed, then `step protect result` shall say so and shall
   show the command that makes a signed approval commit.
3. The skill `protect-before-reduce` shall tell the agent to propose write-offs and to leave the
   approval commit to the owner. The agent shall not sign, change the signing setup, or prepare a
   script that approves.
4. The guide shall tell the owner that a signed commit approves exactly what it contains, and that
   they read the change before they sign it.

### Requirement 5: The owner's key and its setup
**Objective:** As the owner, I want a signing key that needs my passphrase and is known to GitHub,
so that signing needs me at the keyboard and verifying needs nothing from my machine.

#### Acceptance Criteria
1. The guide shall show how to make a signing key of its own with a passphrase, and how to keep it
   out of the SSH agent.
2. The guide shall show how to check that the key asks for its passphrase on each signature.
3. The guide shall show how to add the public key to the GitHub account as a signing key with
   `gh`, including the grant of the signing-key scope to the `gh` session first.
4. The guide shall show how to take the signing-key scope from the `gh` session again after the
   key is added, and shall say why: an agent that uses the same session could otherwise add a key
   of its own.
5. The guide shall show how to add the public key to the project's signer list and how to set up
   Git to sign approval commits in that project only.
6. The guide shall show how to replace the key: add the new one in both places, sign with it, and
   remove the old one. It shall say that approvals signed with the old key stop counting when the
   old key is removed from the forge (2.5).

### Requirement 6: Approvals made before this spec
**Objective:** As the owner of the code, I want the approvals made by script to stop counting and
to be made again by signing, so that no stage stands on an approval an agent could have written.

#### Acceptance Criteria
1. When this change is in place, `step protect result` shall no longer count an approval that
   came in an unsigned commit, whenever it was made.
2. The re-approval in the clones of drift-rdp and scitools-hook shall be one signed commit per
   clone, made by the owner, after which `step protect result` exits as it did before in each.
3. The research record of this spec shall list the earlier approvals by clone and commit, and the
   commit that approved each again.

### Requirement 7: The same verdict everywhere, and tests without the owner's key
**Objective:** As a maintainer, I want the signature check to be tested on Windows and Linux with
keys made for the tests, so that CI proves it without the owner's key or the network.

#### Acceptance Criteria
1. `step protect result` shall reach the same verdict for the same commits and the same keys on
   Windows and on Linux.
2. The verdict shall not depend on the host's Git signing settings, the SSH agent, or the user's
   own allowed-signers files.
3. The tests shall make their own keys and commits, and shall cover each case of 1.3, a change
   after a signed approval (1.2), a key removed from the forge (2.5), and an unreachable forge
   (2.4).
4. The tests shall run in the repository's existing CI (the Dagger run) without network access to
   the forge and without the owner's key.

### Requirement 8: Guides
**Objective:** As a user of lesscode, I want the guides to say how approvals work now, so that I
set up the key once and approve without a script.

#### Acceptance Criteria
1. The owner's setup (requirement 5) shall be written in a guide of its own, linked from the
   guide of the steps.
2. The guide of the steps and phase 9 of the skill `protect-before-reduce` shall say that an
   approval is a signed commit by the owner, and shall drop the fields `approved_by` and
   `approved_at` as the way to approve.
3. The release notes of the next release shall say that approvals made before it stop counting
   until they are signed (6.1).
