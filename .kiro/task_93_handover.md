# Handover 93: signed approvals, group 4 done (the guides and the skill)

Written 2026-11-25, about 17:45 Europe/Berlin. Predecessor: [task_92_handover.md](task_92_handover.md).
Spec: [signed-approvals](specs/signed-approvals/tasks.md). Research entry:
[R195](../docs/research/2026-11-25/06-signed-approvals-guides.md).

## What was done

- **4.1 (`da881b6`), the owner's guide [docs/approvals.md](../docs/approvals.md).** It covers:
  - the key, with a passphrase and kept out of the agent, and the check that it asks each time;
  - `gh`: grant the scope, add the key, take the scope back, with the reason;
  - the signer-list line, in PowerShell and POSIX;
  - Git set up in the project only: `gpg.format`, `user.signingkey`, and `gpg.ssh.program` on
    Windows (the finding of handover 92), with no `commit.gpgsign`;
  - the approval: read `git diff HEAD -- <file>`, then the one `git commit -S … -- <file>`;
  - the two repairs, replacing the key, and what the check does not stop.

  A row in `docs/README.md` points to the guide. Every command except the `gh` ones was run once
  with throwaway keys, on a scratch clone of scitools-hook's `protect/protect-cycles`, with the
  test binary of `0b11aa1`. The result asked the real GitHub and refused the throwaway key, as it
  should (0 of 1). Both repairs worked. The clone and the keys are deleted.
- **4.2 (`7d87f23`): `docs/steps.md`, the skill `protect-before-reduce` with its
  `templates/tasks.md`, `docs/releases/2.2.0.md`, and a dated note at requirement 4.3 of
  `brownfield-test-stage`.**
  - The skill's role table says the agent does not sign, set up or change signing, or script an
    approval.
  - Phase 7 proposes without the mark. Phase 9 stops for the owner's signed commit. The agent may
    commit the signer list with the public key the owner gives, and may set marks in the working
    tree when asked.
  - The release notes say that earlier approvals stop counting until signed.

## Deviations and findings, for the owner

- **The passphrase prompt was not seen.** With the passphrase key, `ssh-keygen -Y sign` waited for
  input (stopped after 120 s). The same command with a key without one signed at once. So the
  signed commits of the trial used the second key. The prompt itself is G9, in 5.2.
- PowerShell 7 passes `-N '""'` as a passphrase of two quote characters. `-N ''` is empty. The
  guide asks interactively, so only test scripts are affected.
- `gh api -X DELETE user/ssh_signing_keys/<id>` (removing an old key) is from GitHub's REST
  interface and was not run.
- The brownfield-test-stage design still describes `approved_by`/`approved_at`. It was left as
  the record of what was approved; the requirement carries the note.
- I fixed a missing line break between two "Known limits" bullets in `docs/releases/2.2.0.md`
  along the way.

## Evidence

- Trial outputs are in R195's table. No suite was run in this group: no code changed. 5.1 runs
  the suites and the Dagger run on the group's commit.

## Current state

- lesscode `main` at this handover's commit; `origin/main` at `9f6aab7`. Not pushed; push only on
  your word.
- Clones untouched: drift-rdp on `protect/protect-shared-frame`, scitools-hook on
  `protect/protect-cycles`.
- Containers: none started by this group.
- Storage: D: 139.7 GiB free at the end (143.3 at the start). This group built nothing, and its
  scratch clone was about 4 MiB and is deleted, so the drop is not lesscode's. Both limits hold.
  Retained evidence grew only by R195.

## Next

Group 5. 5.1: the Windows suites, doctests, format, Clippy, budgets, the Dagger Linux run, and
`kiro-cli code check`. 5.2 is your act, with the guide: make the key, grant the scope, add the key,
take the scope back. Then I read the forge and `gh auth status`, and check one signed commit in a
scratch repository. 5.3: approve the three stage branches again.
