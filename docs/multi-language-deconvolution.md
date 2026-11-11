# Architecture, and how to add a language

This page describes `lesscode-core` as of release 1.2.0. The page it replaces, written on 2026-11-09 for
per-language adapters that no longer exist, is kept in the
[archive](archive/multi-language-deconvolution-2026-11-09.md) with its corrections.

## One binary, four parts

`lesscode-core` is a single Rust executable built from `native/`. It reads source files with tree-sitter
grammars that are compiled in, for Rust, Python, TypeScript and Java.

| Part | Code | Commands | What it does |
| --- | --- | --- | --- |
| Driver | `native/src/adapters/driver/` | `deconvolute analyze`, `mutate`, `run`, `admit-pack`, `merge-annotations` | Finds the places where a file decides behaviour, plants one small fault at a time, and runs the project's own tests against each fault in copies of the source tree. [The agnostic driver](driver.md) |
| Steps | `native/src/step/` | `step plan`, `pin`, `check`, `record`, `guard`, `install-hook` | Judges one small change on a branch: tests before and after, faults noticed per function before and after, size, scope. Keeps a ledger. [Refactoring in checked steps](steps.md) |
| Measures | `native/src/cli_deconvolute.rs`, `native/src/budgets.rs` | `deconvolute density`, `budgets`, `deconvolute install-hook` | Counts tokens and syntax nodes of a before and after pair; holds Rust files to size ceilings |
| Research pipeline | `compare.rs`, `admission.rs`, `surface.rs`, `regenerator.rs`, `recording.rs` and others in `native/src/` | `prepare`, `compare`, `admission`, `surface`, `regenerate` | The frozen-manifest comparison and the regeneration experiments. They need per-specimen setup and are not a one-command workflow. [Working example](reports/native-core-readiness.md) |

The steps part is built on the driver. `step plan`, `pin` and `check` call the same fault runs as
`deconvolute run`, limited to the functions a step touches.

## Where language knowledge lives

In the driver and the step commands, nowhere in code. A test scans both module trees for language and
project names and fails if it finds one. What the engine knows about a language is a pack:
`native/packs/rust.json`, `python.json`, `typescript.json` and `java.json`, each at most 40 lines. A pack
maps the grammar's node kinds to the site kinds of the driver (a condition, a failure signal, a state write,
two adjacent checks, a function boundary, a call's arguments) and gives the operator templates. The packs
are compiled into the binary and listed once, in `native/src/adapters/driver/registry.rs`.

Everything else comes from the project under test: its build command, its test command, its test files.
The engine reads exit codes and a few text fragments you name in the run configuration. It does not parse
test runner output.

Three structs remain under `native/src/adapters/{rust,java,typescript}/`. They implement the traits of
`native/src/adapter_traits.rs` for the research pipeline and are not used by the driver or the steps.

## What runs where

- `analyze`, `mutate`, `density`, `budgets` and `step guard` only read your repository.
  `step record` and `step check` write the verdict, the evidence file and the ledger at the paths your step
  declaration names; nothing else is written inside the repository.
- `deconvolute run` and `step plan`, `pin` and `check` execute your commands. They do it in copies under a
  work directory you name, and refuse a work directory that is the source root, lies inside it or
  contains it.
- The driver, the steps and the measures contact no network service and no model. One command does:
  `regenerate`, part of the research pipeline, sends a prompt to a model endpoint that you configure.

## Add a language

For the driver and the steps a new language needs no engine code.

1. Enable its tree-sitter grammar as a feature of `ast-grep-language` in `native/Cargo.toml`.
2. Write a pack of at most 40 lines under `native/packs/` and list it in
   `native/src/adapters/driver/registry.rs`.
3. Write a control file: annotations on real code of that language with the outcome each must get. Examples
   are in `native/tests/fixtures/driver/controls/`.
4. Run `lesscode-core deconvolute admit-pack --pack P --lang L --controls K --source-root S`. It checks
   size and shape, that every node kind exists in the grammar, the operator templates, and that the
   controls come out as recorded. A rejected pack is not used.
5. Run `deconvolute run` on a real project in that language and read the classes. A high share of
   `invalid` means the operators often produce code that does not build; change the pack, not the engine.

`deconvolute density` uses the same grammars and needs no further work. `budgets` covers Rust only.

How far a model can be trusted to write a pack was measured in Slice B: its answers vary between runs, and
`admit-pack` is the barrier that decides ([closing summary](reports/slice-b-drift-hardening.md)).

## Limits

- Functions are matched by name, and a call is a name before an opening parenthesis. Calls through values
  are not seen.
- The faults are the ones the packs can express; [the driver page](driver.md#what-is-not-there-yet) lists
  what is missing.
- Finite checks are not proofs. No part of the binary shows that two versions of a program behave the same.
