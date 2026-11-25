# Implementation Plan: approvals an agent cannot make

Generated 2026-11-25, about 14:03 (Europe/Berlin). **Approved by the owner on 2026-11-25** ("approved, start with 1.1").
Basis: [requirements](requirements.md) (approved 2026-11-25, 2.3 and 6.2 amended at the design's
approval), [design](design.md) (approved by the owner on 2026-11-25: "approved, both proposals as
written, write the tasks"), [prototype gate](prototype-gate.md) (`proceed`,
[R192](../../../docs/research/2026-11-25/03-signed-approvals-prototype.md)).
Task-graph sanity review: inline (no subagent, by the owner's rule), verdict PASS, with the notes
at the end.

Rules for every task group (owner rules of 2026-11-07, 2026-11-10 and 2026-11-17):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format
  check and lints before every push. Rust only, no Python. Commands on Windows through PowerShell.
- No new crate. `ssh-keygen` and `curl` are started as outside programs with fixed arguments, the
  way `git` is. lesscode never reads a private key, never signs and never commits outside tests.
- Every new file gets its entry in `native/complexity-budgets.json` at the default ceilings. A file
  over budget is split, not exempted.
- `step protect begin`, `step check`, `step pin` and every other command keep their behaviour; each
  group ends with the existing suites passing on Windows and through Dagger on Linux.
- Tests make their own keys without a passphrase, sign with `-c gpg.format=ssh`, and never touch the
  host's Git settings, the SSH agent or the network. The stand-in forge lives in `native/tests` only.
- Each group ends with a dated research entry where a result or a deviation exists, and
  `.kiro/task_N_handover.md`. Storage: measure D: before a group.
- What is reported about a task is what ran and was seen. Times come from the clock.

## 1. Reading signatures, signer lists and the forge

- [x] 1.1 Test support for signed approvals
  - *Done 2026-11-25 (`native/tests/step_support/signing.rs`, `native/tests/step_protect_signed.rs`,
    2 tests pass on Windows). Two deviations: the test checks the commits with Git's own
    `verify-commit` against an allowed-signers file set with `-c` (it accepts the signed commit and
    rejects the unsigned one, the grafted one and the other key), not with `check-novalidate`, which
    the unit tests of 1.2 use; and the stand-in forge is the answer as data (`StandIn`), because the
    forge's trait comes with 1.4, which makes `StandIn` implement it.*
  - A test can make an SSH key pair without a passphrase, commit with a signature from it, commit
    without one, and graft a signature from another commit into a commit object.
  - A stand-in forge answers with a given list of keys, or with "unreachable", for the library
    function of the result.
  - Done when a small test makes a signed and an unsigned commit in a temporary repository and
    `ssh-keygen -Y check-novalidate` accepts the first; the helpers set no global Git setting.
  - _Requirements: 7.2, 7.3, 7.4_

- [x] 1.2 (P) Splitting a commit and checking its signature
  - *Done 2026-11-25 (`0ea39e8`; 5 unit tests). Deviation: the guard test `only_the_revisions_module_starts_git` allowed no file but `revisions.rs` to start a program, which the design missed. It now names the one program each module may start (`revisions.rs` git, `sshsig.rs` ssh-keygen, `forge.rs` curl) and still fails for any other file or program (design, contradiction C9).*
  - Split a raw commit object into the signed payload and the signature, for `gpgsig` and
    `gpgsig-sha256`, searching the header block only; a non-SSH signature counts as not signed
    with an SSH key.
  - Check a signature with `ssh-keygen -Y check-novalidate -n git`, payload on stdin, no terminal,
    no agent; read the fingerprint from its success line. "Cannot start ssh-keygen" is its own
    outcome.
  - Compute a key's `SHA256:` fingerprint in the core from its base64 blob.
  - Done when unit tests show: a good signature gives the key's fingerprint; a changed payload and
    the namespace `file` fail; the core's fingerprint equals `ssh-keygen -l` for a test key; a
    message line that starts with a space is not taken for a header.
  - _Boundary: sshsig_
  - _Requirements: 1.1, 1.3, 7.1, 7.2_

- [x] 1.3 (P) The signer list
  - *Done 2026-11-25 (`81ad577`; 4 unit tests).*
  - Read `.lesscode/allowed_signers` as committed on the branch, in Git's allowed-signers form: one
    `github.com/<account>` principal per line, the same account on every line, other principals
    allowed and ignored; `namespaces` must include `git`, any other option is bad input.
  - Done when unit tests show the account and the listed fingerprints for a good list, "not judged"
    words for an absent list, a list without account and two accounts, and bad input naming the
    line for `cert-authority` and for an unreadable line.
  - _Boundary: signers_
  - _Requirements: 2.1, 2.3, 5.5_

- [x] 1.4 (P) The forge
  - *Done 2026-11-25 (`5e0ad5c`; 3 unit tests, 1 test of the stand-in). Deviation: the recorded answer of R192 is inlined in the test, not read from `docs/experiments`, because the Dagger run copies only `native/` and a few files. The stand-in now implements `Forge` and counts how often it is asked.*
  - A forge answers the signing keys of an account, with each key's title. GitHub is asked through
    `curl` with the fixed GET of the design, page by page until a page has fewer than 100 keys.
  - The account name is checked before `curl` starts; curl's exit 6, 7 and 28 are "unreachable",
    HTTP 404 is "no such account", other HTTP errors are "refused" with GitHub's words; "cannot start
    curl" is its own outcome.
  - Done when unit tests on recorded outputs and exit codes give each of these outcomes without
    network, and a test reads the real answer shape of R192 (`forge-norandom-2026-11-25.json`).
  - _Boundary: forge_
  - _Requirements: 2.4, 2.5, 2.6_

- [x] 1.5 The commit that last changed a write-off
  - *Done 2026-11-25 (3 integration tests: a signature made by Git split and checked, the last change per entry including a reformatting that is no change, and a merge). History is read along the first-parent line (`git log --first-parent`), so a merge that brought a change is itself the last change. Group 1 on Windows: 528 of 528 tests, doctests, format, Clippy, budgets.*
  - Read the history of `write-offs.json` on the branch, newest first, with parents, and the raw
    commit object, through the existing read-only Git commands.
  - For one entry (file and fault): the newest commit whose entry differs from its first parent's;
    a root commit compares with "no entry"; a merge that changed the entry is itself the commit.
  - Done when tests on temporary repositories name the right commit for: a proposal then a
    signed approval; a later change of the reason; one commit that changes two entries; a change of
    another entry only (the first keeps its commit).
  - _Requirements: 1.1, 1.2, 1.6_

## 2. Judging approvals

- [x] 2.1 The mark and the judged state of a write-off
  - *Done 2026-11-25 (7 of 7 numbers tests pass, 1 new). Finding: from this task on, the 5 tests
    of `step_protect_result.rs` that approve through `approved_by`/`approved_at` exit 1 where they
    expected 0, because those approvals now wait "the old way" (6.1) and the command judges no
    signed approval before 3.1. The task order leaves them failing until 3.1 moves them to signed
    commits.*
  - A write-off carries the mark `"approved": true`; the judged approval (key title, fingerprint,
    account, commit, date) or refusal (one of the four kinds, commit, key, repair) is filled in by
    the judge, never read from the file. `approved_by` and `approved_at` are still read, so older
    files parse, and decide nothing.
  - A fault waits with the refusal's words when its write-off was refused, with "approved the old
    way, which no longer counts: approve it in a signed commit" when only the old fields are set, and
    as today when nothing is marked.
  - Done when the tests of the numbers pass with judged write-offs in place of the three fields,
    and a new test shows each of the three waiting texts.
  - _Requirements: 1.3, 1.4, 1.5, 6.1_

- [x] 2.2 The judge
  - *Done 2026-11-25 (`native/tests/step_protect_judge.rs`, 7 library tests on a step branch of
    the toy repository with the stand-in forge). Beside the design: `Judged` also carries
    `on_forge` (how many listed keys the forge has), which the receipt's signer-list line of 3.2
    needs; `last_changes` reads the history once for all marked entries, and `last_change` (1.5)
    is its one-entry case. `judge` was split into `signatures` and `against_forge` to stay within
    the per-function budget of 560 tokens (821 at first).*
  - For every marked write-off: its last change (1.5), the signature (1.2), the signer list read
    once when a good signature needs it (1.3), the forge asked once when a listed key needs it
    (1.4). The date is the commit's committer date.
  - The repair for an unsigned approval is `git commit --amend -S --no-edit` when the commit is the
    branch's head, otherwise setting the mark to false in one commit and back in a signed one.
  - Failure of the list, the forge or the programs is "not judged"; a bad list is bad input.
  - Done when library tests with the stand-in forge give: counted with title, fingerprint and
    commit; each of the four refusals; the forge not asked at all when no marked write-off has a
    good signature by a listed key; asked once for several.
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 4.2_

## 3. The command, the receipt and the result file

- [x] 3.1 `step protect result` with the judge
  - *Done 2026-11-25, carried into task group 2 by the owner's choice ("Carry 3.1 in"), so that
    the group ends with the suites passing. `run_result_with(args, forge, out)` is the library
    entry; `run_result` passes `GitHub` and stdout. The approving tests of `step_protect_result.rs`
    commit `"approved": true` and a signer list in a commit signed with a test key and run through
    the library entry with the stand-in; the binary keeps the unsigned approval (with the amend
    repair), the old way, the uncommitted approval and bad input. New test: `--help` has exactly
    `--declaration`, `--repo`, `--out`, `--help` and names neither forge nor GitHub. The judge now
    reads no history when no write-off is marked.*
  - The result runs the judge between reading the stage and counting; a library entry takes the
    forge, and the command always passes GitHub. "Not judged" is exit 2 and bad input exit 3, and
    neither writes anything, as today. The command gains no option for the forge.
  - The binary's existing tests are moved to signed commits where they approve; every binary test
    stops before the forge (unsigned, unmarked, no list), and the counted path runs through the
    library entry with the stand-in.
  - Done when the binary's tests pass offline, `--help` of `step protect result` names no forge, and
    a stage with no marked write-off gives the same result and receipt as before this spec.
  - _Requirements: 1.5, 1.7, 2.3, 2.4, 2.6, 7.4_

- [x] 3.2 The receipt and `protection.json`
  - *Done 2026-11-25. A write-off whose approval does not count reads "Not counted: <the
    refusal's words>." (or the old-way words) beside "Approved with the key …"; an unmarked one
    stays "Not approved.". The line "Signer list: github.com/<account>, <n> of <m> listed key(s)
    on the forge." follows the write-offs; where no listed key signed an approval the forge is not
    asked and the line says so (`on_forge` is then null). `protection.json` gains top-level
    `signers` and `refusal` on each waiting fault whose write-off was refused (`approval` was
    already there). Tests in `step_protect_result.rs`: a stranger's signed change refused beside
    the owner's counted approval, read from the receipt and from `protection.json`; a list whose
    keys signed nothing.*
  - The receipt shows each counted approval with the key's title, fingerprint, account, commit and
    date; one line with the account and how many of the listed keys are on the forge; each refused
    approval with its reason, commit, key and repair, beside the counted ones. The module comment
    no longer says the tool cannot know who approved.
  - `protection.json` carries the same facts (`approval`, `refusal`, `signers`), fields added and
    none removed; the receipt id stays.
  - Done when tests read each of these from the printed receipt and from `protection.json`.
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 3.3 The cases through the library entry
  - The eleven cases of the design's testing strategy on the toy stage, each with its exit code
    and words: counted; unsigned with the amend repair; signature that does not verify; key not
    listed; key not on the forge; key removed from the forge between two runs; unsigned change
    after a signed approval; one signed commit approving two entries; forge unreachable; marked
    entry without a list (and a list with `cert-authority`); only the old fields.
  - Done when all eleven pass on Windows and in the Dagger Linux run.
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 2.2, 2.3, 2.4, 2.5, 6.1, 7.1, 7.3_

## 4. The guides and the skill

- [ ] 4.1 (P) The owner's guide `docs/approvals.md`
  - A signing key of its own with a passphrase, kept out of the SSH agent, and how to check that it
    asks on every signature; `gh auth refresh -h github.com -s admin:ssh_signing_key`, `gh ssh-key
    add --type signing`, and `gh auth refresh -r admin:ssh_signing_key` with the reason; the line in
    `.lesscode/allowed_signers`; Git set to sign in the project only; the one command that commits
    an approval; reading the change before signing; the repair of a forgotten signature; replacing
    the key; what the check does not stop.
  - Done when every command in the guide has been run once on this machine with a throwaway key
    (the `gh` steps excepted, which are the owner's in 5.2), and the humanizer pass is done.
  - _Boundary: docs/approvals.md_
  - _Requirements: 4.1, 4.4, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 8.1_

- [ ] 4.2 (P) The steps guide, the skill, the release notes, the stage spec
  - `docs/steps.md` (protect result: approvals by signed commit, link to the owner's guide), the
    skill `protect-before-reduce` (role table; phase 7 proposes with `"approved"` absent or false;
    phase 9 stops for the owner's signed commit; the agent never signs, sets up signing or writes
    an approval script), `docs/releases/2.2.0.md` (earlier approvals stop counting until signed),
    and a dated note at requirement 4.3 of `brownfield-test-stage` pointing here.
  - Done when the four files say this, with no remaining mention of `approved_by`/`approved_at` as
    the way to approve, and the humanizer pass is done.
  - _Boundary: docs/steps.md, skill protect-before-reduce, docs/releases/2.2.0.md, brownfield-test-stage requirements_
  - _Requirements: 4.3, 6.1, 8.2, 8.3_

## 5. Validation and the owner's re-approval

- [ ] 5.1 Validation of the tool
  - Windows: the full nextest suite, doctests, format, Clippy, budgets. Linux: the Dagger run on the
    task's commit. `kiro-cli code check`.
  - Done when all pass, with the evidence directory named in the group's handover.
  - _Requirements: 7.1, 7.4_

- [ ] 5.2 The owner's key (owner's act, guided) — can fail
  - The owner follows `docs/approvals.md`: makes the key, grants the scope, adds the public key to
    GitHub as a signing key, takes the scope back. The agent runs none of the `gh` steps and never
    sees the passphrase. The agent then reads the forge's list and checks that the new key is on
    it and that the session no longer has the scope (`gh auth status`).
  - This is gate row G9: the passphrase prompt through Git on Windows. Its failure is recorded and
    changes the guide (signing with `ssh-keygen -Y sign` directly), not the check.
  - Done when the forge lists the key under its title, the scope is gone, and one signed test commit
    in a scratch repository verifies with lesscode's check.
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 5.3 Re-approval on the three stage branches — can fail
  - On drift-rdp's `protect/protect-yuv` and `protect/protect-shared-frame` and on scitools-hook's
    `protect/protect-cycles`: first, without the signer list, `step protect result` shows the
    earlier approvals waiting as "approved the old way". Then the agent commits
    `.lesscode/allowed_signers` with the owner's public key, and sets the mark on the approved
    write-offs in the working tree. The owner reads the change and makes one signed commit per
    branch.
  - Done when `step protect result` on each branch exits as before this spec (0), with the
    receipt naming the owner's key and commit; and a research entry lists, per branch, the earlier
    approval commits and the commit that approved them again.
  - _Depends: 3.1, 3.2, 4.1, 5.2_
  - _Requirements: 6.2, 6.3_

## Task-graph sanity review (inline)

Verdict **PASS**, reviewed against requirements.md, design.md and the task-generation rules.

- Coverage: every requirement 1.1 to 8.3 appears in a task; every component of the design's file
  plan (sshsig, signers, forge, approvals, the changes to revisions, protect_numbers,
  protect_result, protect_receipt, the tests, the guides) is the work of a task.
- Order: 1.1 comes first because 1.2 to 2.2 test with its helpers; 1.2 to 1.4 touch separate new
  files and run in parallel; 1.5 extends `revisions.rs` and is kept sequential. 2.1 changes the
  write-off before 2.2 fills it in. 4.1 and 4.2 can run beside group 3, since they describe the
  design's behaviour; their commands are checked against the built tool in 5.1.
- Note 1: 5.2 and 5.3 need the owner at the keyboard. 5.3 also changes files in two other
  repositories' clones, on their stage branches only, as the stages before did.
- Note 2: 5.3's first run (approvals waiting "the old way") is the evidence for 6.1 on real stages;
  it needs no signer list and no network, by C1.
