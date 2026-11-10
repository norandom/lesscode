# Task Group Handover: v1.0.1 Release Workflow, Adopter Documentation and Released-Binary Dogfood (R55)

Written 2026-11-10, Europe/Berlin. Start the next task group from this file.

## 1. What was done

1. **Version 1.0.1.** `native/Cargo.toml`, `native/Cargo.lock`, `CITATION.cff`,
   `.zenodo.json` and the README badge bumped (commit `f5e39a9`). `v1.0.0`
   stays where it was pushed (`c9bfdb4`, a commit with a failed CI run and no
   GitHub release); it was not moved.
2. **Release workflow.** `.github/workflows/release.yml` builds
   `lesscode-core` for five targets on a `vX.Y.Z` tag, attaches archives,
   `.sha256` files and the install scripts to a draft release, and publishes
   only if all five builds succeed. A manual run builds without publishing.
   Procedure: [docs/ci-validation.md](../docs/ci-validation.md#release-binaries).
3. **Installers.** `scripts/install.sh` (Linux, macOS) and `scripts/install.ps1`
   (Windows) install the latest release, verify SHA-256, need no admin rights.
4. **Adopter documentation (COBESY composition).** README rewritten around
   install, one first trial and measured limits. Docs index, adoption guide,
   native README, CI guide, agent skill updated. Python-prototype guides carry
   an archive notice. Plan recorded as a dated amendment in
   [docs/maintainers/documentation-design.md](../docs/maintainers/documentation-design.md).
5. **Dogfood of the released command surface.** Recorded as
   [R55](../docs/research/2026-11-10/01-released-binary-adopter-dogfood.md);
   R54 and the research index carry dated amendments.

6. **Release published.** Tag `v1.0.1` on `eda13bb`; release workflow run
   42773584139 built and published all five archives. The Windows installer
   was run against the published release and installed 1.0.1.
7. **CI speed-up.** `tools/install_nextest.sh` installs the prebuilt, pinned
   and digest-checked cargo-nextest 0.9.143 on the Actions runners and in the
   Dagger container (commit `eda13bb`). Native jobs dropped to 83 to 274 s.
   No cache was added, by owner decision; sccache with a self-hosted backend
   was discussed as an option for Gitea runners only.
8. **Release signing, ported from mmhmem** (commit `6886c8e`). zipsign 0.2.2,
   archives signed in place, `SHA256SUMS` plus `SHA256SUMS.sig`, provenance
   attestation, publish job in the approval-gated `release` environment.
   Scripts: `scripts/sign-release-assets.ps1`, `scripts/publish-release.ps1`,
   `scripts/setup-release-signing.ps1`; key list
   `scripts/release-public-keys.txt` (empty). Tested locally with a throwaway
   key on the 1.0.1 archives: sign, verify, wrong-context rejection, and
   extraction all behaved; both refusal paths (no key, unlisted key) fire.
   The publish job itself has not run yet; its first run is the next tag.

## 2. Findings that change earlier claims

- `deconvolute analyze --lang python` is a stub: empty receipt, `"passed": true`.
- `deconvolute mutate` replaces specimen-specific substrings. 36 of 38 Rust
  files in `native/src` and 40 of 40 Python files tried gave zero mutants; the
  command exits 0 anyway. No Python generator exists.
- `deconvolute density --lang python` reports lines as statements.
- The pre-push hook enforces budgets only; `refactor/*` handling checks nothing.
- With the hook installed and the binary off `PATH`, every push fails.
- The pre-1.0.1 README used `--file` (actual flag `--source`), omitted the
  required `--out` for `budgets`, and pointed at `.kiro/complexity-budgets.json`,
  which the checker rejects.

**Consequence for Slice B (task 26 blueprint).** `tools/mipro/metric.py` was
planned to score candidates with `lesscode-core deconvolute analyze`. For
Python specimens that gate is vacuous today. Decide this before building the
reward function.

## 3. Evidence pointers

- CI on the bump commit: Actions run 42769611277, five jobs green.
- R55 lists the revision, build command, binary digest and every command.
- Receipts from the session were written to a scratch directory and not kept.

## 4. Current state

- Branch `main` at `6886c8e` (plus the handover commit), CI green on all five jobs (run 42774694344). Build-only release run 42774694964 green.
- `v1.0.1` published, unsigned. The signing key does not exist yet; the owner must run `scripts\setup-release-signing.ps1` and commit the key list before the next tag.
- The pre-push hook is not installed in this checkout (removed after the test,
  because `lesscode-core` is not on this workstation's `PATH`).
- Many experiment directories linked from `docs/README.md` exist only as
  untracked files in the owner's working tree; those links fail in a fresh
  clone. Not changed here.
- `pyproject.toml` still declares the retired `lesscode_baseline` package.

## 5. Next steps

0. Owner: set up the signing key (section 1, item 8), then cut 1.0.2 as the first signed release and check that the `publish` job, the attestation step and both installers work against it.
1. Owner decision on the three non-working porcelain paths: implement, make
   them fail loudly, or remove them. Minimum fix either way: Python `analyze`
   must not report `passed: true`, and `mutate` must not exit 0 with no mutants.
2. Repeat the R55 session with the published archives on Linux and macOS via
   the installers, and on one external repository per language.
3. Decide whether to track or drop the untracked experiment records linked
   from the docs index.
4. Slice B remains as described in [task 26](task_26_handover.md), subject to
   the consequence in section 2.
