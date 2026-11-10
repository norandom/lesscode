# R55: The 1.0.1 binary used as an adopter would use it: budgets, density and three analyzers work; Python analysis, general mutation and the refactor-branch check do not

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, one session on the owner's Windows workstation; individual command timestamps were not retained
- Status: completed
- Spec: [.kiro/specs/dogfood-self-application/](../../../.kiro/specs/dogfood-self-application/validation-report.md) (no new requirement or task; this entry tests that spec's delivered commands from the outside)
- Hypothesis ID and predecessor: [R54](../2026-11-09/12-dogfood-self-application-synthesis.md)
- Type: exploratory

## Decision

The released command surface is usable for three things on a repository that
is not a prepared specimen: the Rust size budgets, the density comparison for
Rust, Java and TypeScript, and the write-site analysis for the same three
languages. The pre-push hook enforces the budgets when the binary is on `PATH`.

Three advertised capabilities are not available to an adopter and must not be
described as working: Python analysis, mutation of arbitrary code, and
state-boundary validation of `refactor/*` branches in the hook. The README and
guides were corrected in the same change. No code was changed; whether to
implement, narrow or remove those commands is an open owner decision.

Separate outcomes:

- Setup: a release build from the tagged source runs with no other files.
- Extraction (write sites): works for Rust, Java, TypeScript; absent for Python.
- Behavioral evidence: none is produced by these commands. Unchanged from R54.
- Compression measurement: works; Python is line-based only.
- Adoption: not assessed. One operator, one repository.

## Hypothesis and falsification

R54 established that the test suite passes on a clean clone and that the hook
unit tests pass. It did not run the installed binary against code other than
the test fixtures. Hypothesis: a person who installs the binary and follows the
README can run each documented command on this repository's own sources and
get a meaningful receipt.

Rejected if any documented command fails as written, returns a passing receipt
without having examined the input, or produces no output on ordinary source
files. Targets were stated before execution; no thresholds were tuned.

## Frozen inputs and method

- Source: `norandom/lesscode` at `f5e39a9` (version 1.0.1), working tree with
  untracked owner files only; no tracked modifications during the runs.
- Binary: `cargo +1.98.1 build --release --locked --manifest-path native/Cargo.toml`,
  host `x86_64-pc-windows-msvc`, SHA-256
  `6b653c62acff3e640c03bf6bad01370c797ae7314dedebc4e301dbe488922997`.
  This is a local build of the release source, not the published archive; the
  archive did not exist yet.
- Operator: one agent session (Claude Code), commands issued through
  PowerShell 7 on Windows 11. No model calls by the tool under test.
- Receipts were written to a session scratch directory outside the repository
  and are not retained; the counts below are transcribed from them. Every
  command is reproducible from the listed revision.
- Not run: `prepare`, `compare`, `admission`, `surface`, `regenerate`; Linux and
  macOS; the published installer scripts (they need a published release).

## Evidence and results

### Commands as documented in the README at `f5e39a9`

| README command | Result |
| --- | --- |
| `deconvolute analyze --lang rust --file F --out R` | Exit 3: `unexpected argument '--file'`. The flag is `--source`. |
| `deconvolute mutate --lang rust --file F --out-dir D` | Same flag error. |
| `budgets --config .kiro/complexity-budgets.json --root .` | Exit 3: `--out` is required. With `--out`: exit 3, ``unknown field `schema_version` ``. That file is the spec-level budget record, not a checker config. |
| `deconvolute density --baseline B --candidate C` | Works as written. |
| `deconvolute install-hook` | Works as written. |

### Budgets

- `budgets --config native/complexity-budgets.json --root native/src --out R`:
  exit 0, 38 files, 0 unlisted, all within budget or disposed.
- Same config with `--root native` or `--root .`: exit 1 with 81 and 320
  unlisted files and no message on the terminal. The root must be the
  directory the config paths are relative to; the only signal is the receipt.
- A minimal config with one file (`path`, `owner`, `allocation`) in a scratch
  repository: exit 0. Adding a second, unlisted `.rs` file: failure.
- The metric is `rust-syntax-units-v1`. Only `.rs` files are measured.

### Analyze

| Language | File | Write sites | Gaps | Exit |
| --- | --- | --- | --- | --- |
| Rust | `native/src/budgets.rs` | 19 | 0 | 0 |
| TypeScript | `native/adapters/typescript/candidate_degradation.ts` | 10 | 0 | 0 |
| Java | `native/adapters/java/ProbeMain.java` | 31 | 0 | 0 |
| Python | `native/shim/recording_shim.py` | 0 | 0 | 0 |

The Python row is not a measurement. `run_analyze` in
`native/src/cli_deconvolute.rs` returns two empty lists for Python without
reading the syntax tree, and the receipt then states `"passed": true`.

With `--frame-spec` allowing one of the 19 Rust sites, the other 18 were
reported as violations and the exit code was 1. The frame spec must contain all
three fields; a file with only `allowed_write_sites` is rejected.

### Mutate

- Rust: all 38 files in `native/src` were tried. 36 produced 0 mutants and 2
  produced 4. `native/adapters/rust/candidate_lifecycle.rs` produced 4.
- Python: the first 40 tracked `.py` files were tried. 0 produced mutants;
  `generate_mutants` has no Python branch.
- TypeScript and Java: 4 mutants each on the adapter specimens, each differing
  from the source in one line.
- Cause: the operators in `native/src/adapters/*/mutation.rs` replace fixed
  substrings taken from the experiment specimens (for example
  `arm_exit != 0`, `"PYTHONPATH"`, `return 124;`). They are not syntax-driven.
- With zero mutants the receipt says `"passed": false` and the process exits 0.

### Density

- Rust lifecycle pair: 248 to 148 canonical lines (40.32%), 83 to 70
  statements (15.66%), verdict `pass`, exit 0. This reproduces R52.
- Identical files: verdict `marginal`, exit 1.
- Python: the statement count is the canonical line count (`count_metrics`
  falls back to `count_canonical_lines` for both values).

### Pre-push hook, installed in this repository

| Case | Result |
| --- | --- |
| `git push --dry-run`, binary not on `PATH` | Push fails: `exec: lesscode-core: not found`. `LESSCODE_SKIP_HOOK` cannot help because the binary never starts. |
| Binary on `PATH`, clean tree | Budgets verified, push proceeds. |
| `LESSCODE_SKIP_HOOK=1` | Skipped, push proceeds. |
| Uncommitted, unlisted `native/src/zz_dogfood_probe.rs` present | Push blocked, file named. The check reads the working tree, not the pushed commits. |
| Scratch repository with no budgets file, `refs/heads/refactor/x` | "Detected major refactoring branch", then passes. Nothing is checked. |
| `install-hook` when a hook exists | Exit 3 unless `--force`. |

`check_refactor_cleanliness` in `native/src/cli_hook.rs` runs `git status` and
prints a note when tracked files are modified. It returns success in every
case. The statement in R54 and in the earlier README that such branches
"require clean state boundaries and non-empty ASTs" is not implemented.

The probe file was removed after the run. The hook was removed from this
repository at the end of the session because the binary is not on the
workstation's `PATH`.

## Limits and interpretation

- One platform, one operator, one repository. The Java and TypeScript
  analyzers were only run on this repository's adapter specimens; their
  behaviour on general code is untested here.
- Write-site counts were not compared with a manual inventory, so analyzer
  precision and recall are unknown. This entry shows the analyzers produce
  located output, not that the output is complete.
- The published archives and installer scripts were not exercised; that
  requires the release to exist.
- The specimen results in R49 to R53 are not affected. Those ran through the
  library and test harnesses with specimen-specific operators, which is what
  the operators were written for. What does not transfer is the reading that
  the porcelain commands offer the same checks to arbitrary code.

## Tool/spec consequence and next gate

- Justified today: recommending `budgets`, `density` and `analyze` (Rust, Java,
  TypeScript) and the budget hook to an adopter, with the stated limits.
- Revised: R54's tool consequence. The hook enforces budgets only.
- Open, owner decision: for Python analysis, general mutation and the
  refactor-branch check, either implement them behind the existing commands,
  make the commands fail loudly instead of passing, or drop them from the
  porcelain. Until then `analyze --lang python` should not return
  `"passed": true`, and `mutate` should not exit 0 with zero mutants.
- Next gate: repeat this session with the published archive on Linux and macOS
  through the installer scripts, and run the analyzers on one external
  repository per language.

## COBESY check

Outcome first: three commands work, three do not, stated before the detail.
Replay: revision, build command, binary digest and every command are given.
Disproof: the hypothesis was rejected on all three stated conditions.
Negative results are the main content. Facts are in the tables; interpretation
is confined to the decision and limits sections. No claim is made about
adoption or about analyzer accuracy.
