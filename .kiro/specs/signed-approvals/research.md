# Research and design decisions: signed approvals

Written 2026-11-25, about 13:58 (Europe/Berlin), with the design. Discovery: light (an extension
of `step protect result`). Prototype record: [R192](../../../docs/research/2026-11-25/03-signed-approvals-prototype.md).

## Summary

- `step protect result` decides "approved" in `WriteOff::approved()` (`native/src/step/protect_numbers.rs`)
  from three fields anyone can write. The receipt (`protect_receipt.rs`) says that the tool cannot
  know a person wrote an approval. Both change.
- The core starts `git` as an outside program with an allow-list of read-only subcommands
  (`revisions.rs`), and has no HTTP client on purpose (`native/Cargo.toml`: "No HTTP client"). The
  design starts `ssh-keygen` and `curl` the same way and adds no crate.
- Integration tests run the binary; the library is public (`lesscode_core::step::*`), so a test
  can call the result with a stand-in forge without the command line offering one.

## Research log

### How is an SSH-signed commit checked without the host's Git settings?
- Git stores the signature in the commit's `gpgsig` header (`gpgsig-sha256` in SHA-256
  repositories). The signed bytes are the commit object without that header. Namespace: `git`.
- `ssh-keygen -Y check-novalidate -n git -s <sig>` with the payload on stdin checks the signature
  alone and prints `Good "git" signature with <TYPE> key SHA256:<fp>`; exit 255 on failure.
  Seen identical in OpenSSH 9.2p1, 9.5p2 and 10.6p1 (R192, G1 to G3).
- `git verify-commit` and `%G?` depend on `gpg.ssh.allowedSignersFile` and the committer e-mail
  (R192 side finding), so they are not used.

### Where does GitHub list signing keys?
- `GET https://api.github.com/users/<account>/ssh_signing_keys` answers without a token, with
  `key` (type and base64, no comment) and `title`; 60 requests per hour per address. 404 for an
  unknown account. Paged, up to 100 per page.
- Adding a signing key needs the token scope `admin:ssh_signing_key`; the owner's `gh` session
  lacks it today. `gh ssh-key add <file> --type signing --title <t>`; `gh auth refresh -s` grants,
  `gh auth refresh -r` removes a scope.

### What does the account have today?
- Four signing keys (three Ed25519, one RSA), one of them titled "GitHub Actions Signing Key -
  observability-k8s". The local `~/.ssh/id_ed25519` is not among them, nor among the login keys.
  Hence the owner's choice: a key counts only when both the project's list and GitHub have it.

### Earlier approvals
- scitools-hook clone, `protect/protect-cycles`: `421c1ef` (by script, unsigned).
- drift-rdp clone, `protect/protect-shared-frame`: `0b5fe69`, `39e09d4` (by script, unsigned).
- The stage on `yuv.rs` (R188) approved write-offs too; its branch is listed in the task that
  re-approves (requirement 6.3).

## Architecture pattern evaluation

| Option | For | Against | Chosen |
| --- | --- | --- | --- |
| `ssh-keygen -Y check-novalidate` as an outside program | already on every platform the CI runs; no new crate; proven equal across versions | output parsed from text; depends on the program found on PATH | **yes** |
| A Rust crate for SSH signatures (`ssh-key` with Ed25519) | no outside program | new dependency tree in the core, more compile time (the owner judged build time before) | no |
| `git verify-commit` with `-c gpg.ssh.allowedSignersFile` | Git does the parsing | its verdict depends on the committer e-mail as principal and on host settings | no |
| `curl` as an outside program for the forge | on Windows 10+ and in the CI image; keeps "no HTTP client" | output and exit codes read by lesscode | **yes** |
| `gh api` for the forge | uses the owner's token | not installed everywhere; a token is not needed | no |
| A lesscode command that writes and commits the approval | one step for the owner | lesscode has never committed; an agent could run it too (it would fail only at the passphrase) | no |

## Design decisions

### Decision: the approval is the signed commit that last changed the write-off
- Context: requirement 1.1, 1.2, 1.6.
- Alternatives: (a) an approval file that only the owner writes; (b) empty signed commits with a
  trailer naming the faults; (c) the last change of the entry in `write-offs.json`.
- Selected: (c). The entry gains `"approved": true`; the owner's signed commit sets it. Any later
  change of the entry, signed by the agent's key or unsigned, takes the approval away.
- Trade-off: a forgotten `-S` on an older commit needs two commits to repair (set false, then true
  signed). On the newest commit, `git commit --amend -S --no-edit` repairs it.

### Decision: the signer list is Git's allowed-signers format at `.lesscode/allowed_signers`
- Context: requirement 2.1 to 2.3, 5.5.
- The principal `github.com/<account>` names the forge and the account. The owner may add their
  e-mail as a second principal on the same line; Git's own `git log --show-signature` then shows
  "Good" when the owner points `gpg.ssh.allowedSignersFile` at the file. lesscode reads only the
  `github.com/` principal.

### Decision: the forge is asked only when an approval has passed the local checks
- Context: requirement 2.3, 2.4, 7.4. A stage without approvals, or with only unsigned ones, is
  judged as today and needs neither the list nor the network. This is how the binary's tests stay
  offline.

## Risks and mitigations

- `ssh-keygen`'s wording changes: the fingerprint is read by pattern (`SHA256:` and base64), and a
  test compares it with lesscode's own fingerprint of the listed key.
- Rate limit of 60 requests per hour: one request per run; a 403 is "not judged", with GitHub's words.
- The owner's terminal prompt through Git on Windows (G9, not run): the guide offers
  `ssh-keygen -Y sign` with a manual commit only if the prompt fails at setup.
- Tools on the owner's machine replaced by an agent: outside any local check; the result can be run
  again on another machine with the same answer.

## References

- OpenSSH `ssh-keygen(1)`, section "-Y check-novalidate" and "ALLOWED SIGNERS".
- Git `gpg.format=ssh`, `user.signingKey`, `gpg.ssh.allowedSignersFile` (git-config(1)).
- GitHub REST "List SSH signing keys for a user".
