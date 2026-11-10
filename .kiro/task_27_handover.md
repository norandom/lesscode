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

- Branch `main`. See `git log` for the commits after `f5e39a9`.
- The pre-push hook is not installed in this checkout (removed after the test,
  because `lesscode-core` is not on this workstation's `PATH`).
- Many experiment directories linked from `docs/README.md` exist only as
  untracked files in the owner's working tree; those links fail in a fresh
  clone. Not changed here.
- `pyproject.toml` still declares the retired `lesscode_baseline` package.

## 5. Next steps

1. Owner decision on the three non-working porcelain paths: implement, make
   them fail loudly, or remove them. Minimum fix either way: Python `analyze`
   must not report `passed: true`, and `mutate` must not exit 0 with no mutants.
2. Repeat the R55 session with the published archives on Linux and macOS via
   the installers, and on one external repository per language.
3. Decide whether to track or drop the untracked experiment records linked
   from the docs index.
4. Slice B remains as described in [task 26](task_26_handover.md), subject to
   the consequence in section 2.
