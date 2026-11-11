# Native capability-compression core

This Rust 2021 package builds one library (`lesscode_core`) and one executable
(`lesscode-core`). To use the tool without building it, install a release
binary as described in the [project README](../README.md#install); this page
is for building from source and for the license obligations.

The executable has two groups of commands. The `deconvolute` group and
`budgets` work on plain source files and are described, with their measured
limits, in the project README. From release 1.1.0 the `deconvolute` commands `analyze`,
`mutate`, `run` and `admit-pack` are served by one engine driven by language
packs (`packs/`, compiled into the binary) for Rust, Python, TypeScript and
Java; see [the agnostic driver](../docs/driver.md). Of all commands only
`deconvolute run` executes a project's own build and test commands, and it does
so in copies of the source tree, never in the tree itself. The `prepare` and
`compare` commands freeze
reviewed capability manifests and compare supplied Python, TypeScript and TSX
source with source-bound external behavioral evidence. They write JSON and
Markdown, count complete replacements and leave specimens unchanged. They
never run project checks. The Python CLI that preceded this package was retired
at commit `fd0b5b9`.

Start with the [working paired example and scoped readiness](../docs/reports/native-core-readiness.md).
`prepare --manifest MANIFEST --baseline ROOT --out FRESH.json` freezes the trial;
`compare --frozen FROZEN --baseline ROOT --candidate ROOT --evidence EVIDENCE
--artifact-root ARTIFACTS --out FRESH` produces `result.json` and `report.md`.
Repeat `--artifact-root` only for distinct required roots. Output must be new and
outside both specimens. Exit codes: 0 prepared/accepted; 1 rejected/not-demonstrated;
2 incomplete; 3 invalid input/baseline-ineligible/I/O error.

Metrics use explicit per-language v1 definitions: physical source lines,
logical grammar statements (compound headers and children, no block containers),
aggregate decisions (no implicit function base), function definitions and resolved
internal call layers. Dynamic/cyclic relationships keep full depth unavailable
with located unresolved calls and optional lower bounds. These are not Radon LLOC
or Lizard CCN. The [design metric contract](../.kiro/specs/capability-compression-core/design.md#metrics-contract)
defines them. Byte/gzip size and runtime are not compression-benefit metrics.

The current correctness matrix covers five runners (Linux x86_64 and aarch64,
Windows x86_64, macOS Intel and Apple Silicon); see
[CI validation](../docs/ci-validation.md) for the latest run. The first hosted
validation passed 102 Windows, 103 Linux and 103 macOS tests with zero
skips, plus build/fmt/clippy/doctest/CLI checks in Actions run 41333477507;
the readiness report retains exact source and artifact identities. Finite checks and consistency hashes
do not establish universal equivalence or trusted execution. See the
[coverage map](../.kiro/specs/capability-compression-core/requirement-coverage.md)
and [readiness gate](../.kiro/specs/capability-compression-core/readiness.md).

## Reproduce the build

Install rustup and Rust **1.98.1**. `rust-toolchain.toml` selects this version
when running inside `native/`; from the repository root select it explicitly:

```text
cargo +1.98.1 test --locked --manifest-path native/Cargo.toml
cargo +1.98.1 build --locked --manifest-path native/Cargo.toml
cargo +1.98.1 run --locked --manifest-path native/Cargo.toml -- --help
```

For parallel process-isolated execution use cargo-nextest; retain a separate
doctest run. See [parallel test commands](../docs/testing.md) for pinned setup and
the Python reference suite's pytest-xdist command.

For development correctness checks, also run:

```text
cargo +1.98.1 fmt --manifest-path native/Cargo.toml --all -- --check
cargo +1.98.1 clippy --locked --manifest-path native/Cargo.toml --all-targets -- -D warnings
```

The [native correctness workflow](../.github/workflows/native-core.yml) runs these
commands on Windows, Linux (x86_64 and aarch64) and macOS (Intel and Apple
Silicon) with Rust 1.98.1 and locked dependencies. The
[release workflow](../.github/workflows/release.yml) builds the published
binaries on the same runners.
`cargo test` includes the metric/decision unit fixtures, parser probes, paired
workflow fixtures and CLI exit-code/source-preservation checks. Platform-gated
path fixtures exercise Windows reparse points and Unix symlinks/case collisions.
No hardware access, model downloads or performance suite is required.

Each matrix job retains raw command logs, exit codes, runner/source identities
and its outcome as a `native-core-<runner>-<attempt>` artifact for 14 days, including
failed checks. Final validation must link the actual run and retain its evidence
before artifacts expire. This workflow configuration is not a platform-pass
claim: absent, skipped, cancelled or failed runs remain missing passing evidence.

Direct dependencies use exact versions and `Cargo.lock` freezes transitive
versions/checksums. Keep `--locked`; do not update dependencies as part of a
replay. First builds need network access to the Rust distribution and crates.io
(or populated toolchain/crate caches). No Python/Node process, commercial
analyzer or hosted account is needed by native analysis.

### Compilation prerequisites

- Windows x64: the `x86_64-pc-windows-msvc` toolchain, Visual Studio Build Tools
  with Desktop development with C++ (MSVC compiler/linker), and a Windows SDK.
  The grammar build scripts use the `cc` crate to discover the compiler.
- Linux x64: the `x86_64-unknown-linux-gnu` toolchain, a C compiler, linker,
  libc development headers and standard build utilities (Debian/Ubuntu
  `build-essential`). Generated grammar C sources ship in the crates; generating
  grammars with Node or tree-sitter CLI is unnecessary.
- Linux aarch64: the `aarch64-unknown-linux-gnu` toolchain with the same
  compiler and header packages. CI uses the `ubuntu-24.04-arm` hosted runner.
- macOS x64: the `x86_64-apple-darwin` toolchain and Xcode Command Line Tools
  (Apple Clang and SDK). CI uses the `macos-15-intel` hosted runner.
- macOS Apple Silicon: the `aarch64-apple-darwin` toolchain and Xcode Command
  Line Tools. CI uses the `macos-15` hosted runner.
- rustfmt/clippy are pinned toolchain components for development; neither is
  a native executable runtime requirement. Target-project behavioral checks
  still require their own recorded runtimes.

When using WSL against this checkout, use a separate Linux target directory:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=/tmp/lesscode-native-target
cargo +1.98.1 test --locked --manifest-path native/Cargo.toml
```

## Parser selection and license obligations

Four grammar features are enabled on `ast-grep-language`: `tree-sitter-python`,
`tree-sitter-typescript`, and, for the driver, `tree-sitter-rust` and
`tree-sitter-java`; its broad `builtin-parser` default is disabled. TypeScript
and TSX are separate parsers supplied by the same grammar crate. The lock selects
ast-grep core/language **0.51.0**, tree-sitter **0.27.1**, Python grammar **0.25.0**,
TypeScript/TSX grammar **0.23.2**, Rust grammar **0.24.2** and Java grammar
**0.23.5**. The library exposes their existing parser
types directly. `tests/parsers.rs` checks source retention, language-specific
nodes, JSX recognition distinct from TypeScript, and syntax-error detection for
the Python, TypeScript and TSX grammars. These are parsing checks, with no metric
names or counts. The Rust and Java grammars are exercised by the driver's tests
(`tests/driver_*.rs`).

[LICENSES.md](LICENSES.md) inventories every dependency record from locked Cargo
metadata, including optional and platform-specific records. Preserve the
applicable copyright and permission notices with source/binary distributions;
[THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt) contains the packaged notices
and license texts. MIT/Apache alternatives permit selecting one license;
Apache-only, Unicode-3.0 and Zlib conditions also remain applicable where used.
Keep notices unchanged, preserve attribution, and include relevant license texts
when redistributing. Lesscode project code is licensed under the [MIT License](../LICENSE).

Three crates omit their upstream MIT license text from their published archive.
The missing texts are retained under `licenses/`, fetched from the exact source
commits recorded in the crates' `.cargo_vcs_info.json`:

- ast-grep core/language: `48b54b3120b70fa9e8cbb40a03bf3bfebe6bf50a`,
  [upstream LICENSE](https://github.com/ast-grep/ast-grep/blob/48b54b3120b70fa9e8cbb40a03bf3bfebe6bf50a/LICENSE).
- tree-sitter-typescript: `f975a621f4e7f532fe322e13c4f79495e0a7b2e7`,
  [upstream LICENSE](https://github.com/tree-sitter/tree-sitter-typescript/blob/f975a621f4e7f532fe322e13c4f79495e0a7b2e7/LICENSE).

## Task 1.1 validation record (2026-11-01)

Windows x64/MSVC and Debian WSL2 x64/GCC 14.2.0 both compiled with rustc/cargo
1.98.1. On both platforms `cargo test --locked --manifest-path native/Cargo.toml`
passed all four parser tests; `cargo build --locked --manifest-path
native/Cargo.toml` and CLI `--help` also succeeded. Linux artifacts were placed
outside the checkout under `/tmp/lesscode-native-task11-target`. WSL initially
lacked Rust; user-local rustup installed the pinned minimal toolchain. No macOS
run or portable CI integration is claimed for this foundation milestone.

Tests preceded the library parser exports. Initial Windows test compilation
failed with E0432, unresolved imports `lesscode_core::{AstGrep, Python, Tsx,
TypeScript}`. Adding the real substrate exports made all four tests pass.
The tests parse in-memory fixtures and do not read or write source specimens.
