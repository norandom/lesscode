# Native foundation

This separate Rust 2021 package builds one library (`lesscode_core`) and one
executable (`lesscode-core`). It leaves the existing Python CLI and historical
evidence unchanged. This milestone provides real Python, TypeScript and TSX
parser probes and CLI help/version only. Prepare/compare, source inventory,
metric definitions and evidence decisions belong to subsequent tasks; parser
success does not establish metric correctness or behavioral equivalence.

## Reproduce the build

Install rustup and Rust **1.98.1**. `rust-toolchain.toml` selects this version
when running inside `native/`; from the repository root select it explicitly:

```text
cargo +1.98.1 test --locked --manifest-path native/Cargo.toml
cargo +1.98.1 build --locked --manifest-path native/Cargo.toml
cargo +1.98.1 run --locked --manifest-path native/Cargo.toml -- --help
```

For development correctness checks, also run:

```text
cargo +1.98.1 fmt --manifest-path native/Cargo.toml --all -- --check
cargo +1.98.1 clippy --locked --manifest-path native/Cargo.toml --all-targets -- -D warnings
```

The [native correctness workflow](../.github/workflows/native-core.yml) runs these
commands on Windows, Linux and macOS with Rust 1.98.1 and locked dependencies.
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
- macOS x64: the `x86_64-apple-darwin` toolchain and Xcode Command Line Tools
  (Apple Clang and SDK). CI uses the `macos-15-intel` hosted runner.
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

Only `tree-sitter-python` and `tree-sitter-typescript` features are enabled on
`ast-grep-language`; its broad `builtin-parser` default is disabled. TypeScript
and TSX are separate parsers supplied by the same grammar crate. The lock selects
ast-grep core/language **0.51.0**, tree-sitter **0.27.1**, Python grammar **0.25.0**
and TypeScript/TSX grammar **0.23.2**. The library exposes their existing parser
types directly. `tests/parsers.rs` checks source retention, language-specific
nodes, JSX recognition distinct from TypeScript, and syntax-error detection for
all three grammars. These are parsing checks, with no metric names or counts.

[LICENSES.md](LICENSES.md) inventories every dependency record from locked Cargo
metadata, including optional and platform-specific records. Preserve the
applicable copyright and permission notices with source/binary distributions;
[THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt) contains the packaged notices
and license texts. MIT/Apache alternatives permit selecting one license;
Apache-only, Unicode-3.0 and Zlib conditions also remain applicable where used.
Keep notices unchanged, preserve attribution, and include relevant license texts
when redistributing. This unpublished package adds no project license grant.

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
