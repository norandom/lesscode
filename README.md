# Lesscode

[![DOI](https://zenodo.org/badge/1440672292.svg)](https://doi.org/10.5281/zenodo.23958363)
[![Release](https://img.shields.io/badge/version-1.1.0-blue.svg)](https://github.com/norandom/lesscode/releases/tag/v1.1.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

The goal is substantial code simplification while preserving features. In grown
codebases, custom implementations, boilerplate and supporting layers can often
be replaced by capabilities already provided by the project's framework,
libraries or language. Lesscode aims to help identify those opportunities and
check the resulting changes.

The work is to move responsibility into suitable existing capabilities: use a
framework shorthand, reuse a library operation, or express the same behavior
with clearer idiomatic code. Success means less code and structure to maintain
for the same features and public contracts. Moving custom code into another
module, hiding it behind a wrapper, or dropping a feature does not establish
that result. A replacement's dependencies, configuration and behavioral
differences also count when judging whether it simplifies the project.

## What you can do with it today

`lesscode-core` is one native binary. You give it source files you already
have, and it writes JSON receipts you can read in a review. No command changes
your sources, and none makes a network or model request.

| You want to know | Command | Works on |
| --- | --- | --- |
| Is this rewrite a substantial reduction or a cosmetic one? | `deconvolute density` | Rust, Java, TypeScript; Python by line count only |
| Where does this file decide behaviour and write state? | `deconvolute analyze` | Rust, Python, TypeScript, Java |
| Which small faults can be planted in this file? | `deconvolute mutate` | Rust, Python, TypeScript, Java |
| Which of those faults do my tests catch? | `deconvolute run` | the same four, with your own build and test commands |
| Is this language pack fit for use? | `deconvolute admit-pack` | any pack for a compiled-in grammar |
| Are my Rust files still within their agreed size ceilings? | `budgets` | Rust source trees |
| Can a push be stopped when those ceilings are broken? | `deconvolute install-hook` | Any Git repository with a budgets file |

`deconvolute run` is the one command that executes your project's commands. It
does so in copies of your source tree under a work directory you name, never in
the tree itself, and it refuses a work directory inside or around the source.
Everything else only reads. `analyze`, `mutate` and `run` share one engine,
described in [the agnostic driver](docs/driver.md); what it knows about a
language is a data file of at most 40 lines (`native/packs/`), not code.

Read [what is not checked](#what-is-not-checked) before you rely on a result.
Releases before 1.1.0 had different `analyze` and `mutate` commands, bound to
this project's experiment specimens; their limits are in
[R55](docs/research/2026-11-10/01-released-binary-adopter-dogfood.md).

## Install

Releases carry prebuilt binaries for five targets. The scripts below download
the archive for your machine from the latest release, verify its SHA-256
digest and copy the binary into a directory you own. They do not use
administrator rights.

Linux and macOS:

```sh
curl -fsSL https://github.com/norandom/lesscode/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/norandom/lesscode/releases/latest/download/install.ps1 | iex
```

Then check the installation:

```sh
lesscode-core --version
```

| Platform | Release archive | Default install directory |
| --- | --- | --- |
| Linux x86_64 | `lesscode-core-x86_64-unknown-linux-gnu.tar.gz` | `~/.local/bin` |
| Linux aarch64 | `lesscode-core-aarch64-unknown-linux-gnu.tar.gz` | `~/.local/bin` |
| macOS Apple Silicon | `lesscode-core-aarch64-apple-darwin.tar.gz` | `~/.local/bin` |
| macOS Intel | `lesscode-core-x86_64-apple-darwin.tar.gz` | `~/.local/bin` |
| Windows x86_64 | `lesscode-core-x86_64-pc-windows-msvc.zip` | `%LOCALAPPDATA%\Programs\lesscode-core\bin` |

Options, set as environment variables before running the script:

- `LESSCODE_VERSION=1.0.1` installs that release instead of the latest.
- `LESSCODE_INSTALL_DIR=/some/dir` changes the target directory.
- `LESSCODE_ADD_TO_PATH=1` (Windows only) appends the directory to your user
  `PATH`. Without it the script prints the directory and leaves `PATH` alone.
  On Linux and macOS the script never edits shell profiles.

The Linux binaries are built on Ubuntu 24.04 and need glibc 2.39 or newer. On
an older distribution, or on any other platform, build from source. You can
also download an archive and its `.sha256` file by hand from the
[releases page](https://github.com/norandom/lesscode/releases/latest); each
archive contains the binary, the license and the third-party notices.

The install scripts check the SHA-256 digest of the download. From release 1.0.2 on, archives are also signed with the project's Ed25519 key and
carry a GitHub provenance attestation; [release signing](docs/release-signing.md)
shows how to verify them. Release 1.0.1 itself is not signed.

### Build from source

Use Rust 1.98.1 and a C compiler. [native/README.md](native/README.md) lists
the per-platform prerequisites.

```sh
cargo +1.98.1 build --release --locked --manifest-path native/Cargo.toml
```

The binary is written to `native/target/release/lesscode-core`
(`lesscode-core.exe` on Windows), or under `CARGO_TARGET_DIR` if you set it.

## Try it on one change

Pick a rewrite you can already review by hand: the file before and the file
after. Keep the receipts outside your source tree.

### 1. Measure the reduction

```sh
lesscode-core deconvolute density --lang rust \
  --baseline before/lifecycle.rs --candidate after/lifecycle.rs \
  --out receipts/density.json
```

Run against the lifecycle pair kept in this repository
(`native/adapters/rust/baseline_lifecycle.rs` and `candidate_lifecycle.rs`),
the receipt reads:

```json
{
  "receipt": "deconvolution-density-v1",
  "baseline_lines": 248,
  "candidate_lines": 148,
  "line_reduction_pct": 40.32,
  "baseline_stmts": 83,
  "candidate_stmts": 70,
  "stmt_reduction_pct": 15.66,
  "passed": true,
  "verdict": "pass"
}
```

Read that receipt with care. The candidate in this pair is written more densely than a formatter allows.
With both files formatted the same way the same command reports 248 to 219 lines (11.7%) and 83 to 76
statements (8.4%), verdict `marginal`
([R86](docs/research/2026-11-11/12-rust-candidate-built-and-measured.md)). Format both files the same way
before you run `density`; it counts lines.

`pass` needs all four conditions: at least 20% and 50 fewer canonical lines,
and at least 10% and 10 fewer statements. Canonical lines exclude blank lines
and line comments. Anything less is reported as `marginal` with exit code 1. A
marginal verdict says the change is small; it does not say the change is wrong.

### 2. List the write sites

```sh
lesscode-core deconvolute analyze --lang typescript \
  --source src/session.ts --out receipts/analysis.json
```

The receipt lists every place the file decides behaviour (`sites`), each state
write as `file:line`, the function boundaries, and each construct the
language pack declares it cannot follow (for example `unsafe` code) under
`gaps`. With `--frame-spec FILE` you supply a JSON file whose
`allowed_write_sites` lists the sites you allow (the file also needs
`permitted_shell_locations` and `failure_invariants`, which may be empty
lists), and any other site is reported under `violations`. Exit code 0 means no gaps and no
violations; 1 means at least one.

### 3. See which faults your tests catch

Write a run configuration that names the files, your test command and, if
needed, setup and build commands (the fields are in
[the agnostic driver](docs/driver.md#run-configuration)):

```json
{
  "language": "python",
  "source_root": "../my-project",
  "files": ["pkg/ratelimit.py"],
  "test_paths": ["tests/"],
  "setup": [["uv", "sync", "--quiet"]],
  "test": ["uv", "run", "--no-sync", "python", "-m", "pytest", "tests/test_ratelimit.py", "-q", "-x"],
  "runner_evidence_patterns": [" passed", " failed", " error"],
  "invalid_exit_codes": [2, 3, 4, 5]
}
```

```sh
lesscode-core deconvolute run --config run.json --work /some/scratch/dir \
  --workers 6 --out receipts/run.json
```

The driver copies the source tree into the work directory once per worker,
checks that your tests pass there unmutated, plants one fault at a time, runs
your commands, restores the file and verifies the restoration. The receipt
classes every fault as `killed`, `survived`, `invalid`, `timeout` or `error`.
Exit code 0 means the run is complete, 2 that some faults ended in `error`, 3
that the run is void and nothing in it may be relied on. Stopping the run with
Ctrl-C kills your commands, restores the copy and ends the run as void; faults
already classed are kept and reused by the next run.

### 4. Hold Rust files to size ceilings

Write a budgets file that names every `.rs` file under the root you govern:

```json
{
  "files": [
    { "path": "lib.rs", "owner": "core team", "allocation": "public arithmetic API" }
  ]
}
```

```sh
lesscode-core budgets --config complexity-budgets.json --root src --out receipts/budgets.json
```

Paths are relative to `--root`. Each file defaults to 400 physical lines, 300
syntax units and 80 lines per function; set `max_physical_lines`,
`max_logical_units` or `max_fn_lines` per file to change that. A `.rs` file
that is not listed fails the check, as does a listed file that is missing or
over a ceiling. Exit code 0 means every file is within budget. This
repository's own file is [native/complexity-budgets.json](native/complexity-budgets.json).

### 5. Stop a push that breaks the ceilings

```sh
lesscode-core deconvolute install-hook
```

This writes `.git/hooks/pre-push`. On each push the hook looks for
`complexity-budgets.json` with a `src` directory at the repository root (or
`native/complexity-budgets.json` with `native/src`) and runs the budget check.
A violation blocks the push and names the files.

Know these three things before you install it:

- The hook calls `lesscode-core` by name. If the binary is not on `PATH`,
  every push fails with `lesscode-core: not found` until you fix `PATH` or
  delete `.git/hooks/pre-push`.
- `LESSCODE_SKIP_HOOK=1 git push` skips the check for one push. It only works
  when the binary can be found.
- In a repository without a budgets file the hook passes without checking
  anything.

`install-hook` refuses to replace an existing `pre-push` hook unless you pass
`--force`.

## What is not checked

- No command establishes that a rewrite behaves the same as the original.
  `deconvolute run` measures how sensitive your tests are to planted faults: a
  fault they do not catch is a place where a cut is not protected. A run in
  which every fault is caught is not an equivalence proof.
- The planted faults are these: a negated or altered condition, a
  removed failure signal, a dropped state write, two adjacent checks swapped,
  and a changed call argument. Other faults (an element dropped from a list, a
  removed chained call, what a process prints) are not planted.
- In typed languages a share of the planted faults does not compile. They are
  reported as `invalid` and say nothing about your tests; each still costs a
  build.
- Speed is your project's own build and test time per fault. Measured here:
  under a second per fault for a Python file, several seconds for Java, and
  five to twenty seconds for Rust, which rebuilds the crate each time.
- Copies must not share a build cache. With Cargo and a global
  `CARGO_TARGET_DIR`, set `"env": {"CARGO_TARGET_DIR": "target"}` in the run
  configuration; otherwise copies run each other's builds and the run is
  voided ([details](docs/driver.md#run-configuration)).
- Four languages have a pack. A fifth needs its grammar compiled into the
  binary and a pack that `admit-pack` accepts.
- `deconvolute density --lang python` counts lines only; its statement fields
  repeat the line count.
- The budget check reads the files in your working tree, not the commits being
  pushed, and it covers Rust only.
- The hook checks complexity budgets only, and from release 1.0.2
  it treats every branch the same. Version 1.0.1 prints a note on `refactor/*`
  and `deconvolute/*` branches but performs no additional analysis on them.
- Receipts written with `--out` may be placed anywhere, including inside your
  source tree, and an existing file is overwritten.

The research pipeline behind the published experiments (recorded behavior,
bounded finite-domain comparison, mutation scoring against real test suites,
model-driven regeneration) lives in the library and in the `prepare`, `compare`,
`admission`, `surface` and `regenerate` commands. Those need frozen manifests,
recorded evidence and per-specimen setup. Start from the
[working paired example](docs/reports/native-core-readiness.md) if you want to
use them; they are not a one-command workflow yet.

## Try it with one reviewer first

Use the first trial to find out whether the receipts help a review you already
do. Choose one module whose owner and regular reviewer are both willing, agree
beforehand what a `marginal` or failing receipt will mean, and compare the
receipt with the diff. The [adoption guide](docs/adoption.md) describes that
trial, when to stop, and what to settle before turning a check into a gate.

## Evidence

The project keeps a dated research log with the experiments, the failed
attempts and the revisions to its hypotheses.

- [Research log](docs/research/README.md): the latest decision first, then the
  record by date.
- [Multi-language synthesis (R53)](docs/research/2026-11-09/11-multi-language-deconvolution-synthesis.md):
  four specimens in Python, TypeScript, Java and Rust with 24% to 44% fewer
  lines as measured then; the Rust figure falls from 40% to 12% when both files
  are formatted the same way (R86). Its behavioural figures (100 compared cases, all adverse controls
  killed) are weaker than they read: see the next entry.
- [The specimen controls rerun through the driver (R75)](docs/research/2026-11-11/01-specimen-controls-rerun-through-driver.md):
  what the earlier "4 of 4 killed" figures were, and what the project's own
  tests catch when the driver plants faults in the same code.
- [Native driver on five real scopes (R73)](docs/research/2026-11-10/19-native-driver-batches-on-gate-scopes.md):
  `deconvolute run` gives 556 of 556 mutants the class the Python prototype gave.
- [Self-application (R54)](docs/research/2026-11-09/12-dogfood-self-application-synthesis.md):
  the native test suite on a clean clone and the pre-push hook tests.
- [Released binary on this repository (R55)](docs/research/2026-11-10/01-released-binary-adopter-dogfood.md):
  what an adopter's first session finds, including the gaps listed above.
- [Proxy routing trial](docs/experiments/proxy-routing-trial/README.md): a real
  replacement that removed 23 lines with 1,172 tests passing.

Those results describe specific specimens under finite checks. They are not a
claim that the tool will reduce your code by a similar amount, and they are not
equivalence proofs.

## Documentation

- [Documentation index](docs/README.md)
- [Native core: build, prerequisites, licenses](native/README.md)
- [Architecture and how to add a language](docs/multi-language-deconvolution.md)
- [CI and release workflows](docs/ci-validation.md)
- [Agent skill for deconvolution](.agents/skills/deconvolute/SKILL.md)

Native checks run on Linux x86_64 and aarch64, Windows x86_64, and macOS on
Intel and Apple Silicon. The earlier Python prototype (`lesscode-baseline`) was
retired at commit `fd0b5b9`; its guides remain under `docs/` and are marked as
archived.

## Where the project is going

1. A release that contains the agnostic driver.
2. Slice B, hardening against drift in model output: wherever a model writes
   something the tool then relies on (today: a language pack), measure how
   much its answers vary between runs and between models, and keep a
   mechanical barrier that admits only what passes. Measured so far on two
   models, with two prompt-optimisation engines; see the research log from
   R66 on.
3. Deterministic rewrites and synthesis: extend concolic extraction and bounded
   synthesis to more language semantics and idiomatic transformations.

## License

Lesscode is licensed under the [MIT License](LICENSE). Third-party dependencies
and fixtures retain their own licenses and notices; see the
[native dependency inventory](native/LICENSES.md).

## Citation

Releases are archived on Zenodo; the DOI badge above resolves to the latest
archived release. Citation metadata is in [CITATION.cff](CITATION.cff).
