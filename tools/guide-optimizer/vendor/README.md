# Copied code

`dsrust-gepa/` and `pyrng/` are unmodified copies of the `src/` directories of two crates from
<https://github.com/getlatentic/dsrust>, as published on crates.io at version `0.1.0-alpha.3`
(repository tag `v0.1.0-alpha.3`, commit `dc98722febe9829bdfb033e88949895a05868c4a`), copied on 2026-11-10
with the owner's permission to copy instead of depending on the crates.

- `dsrust-gepa`: a Rust reproduction of GEPA's reflective prompt-evolution engine (Pareto-front candidate
  selection and the evolution loop, behind an adapter trait). Author: Tosin Amuda.
- `pyrng`: reproductions of Python's Mersenne-Twister generators, which the engine draws from.

`dsrust-tpe/` was added on 2026-11-11 for the MIPROv2 comparison: an unmodified copy of the `src/` directory
of the crate `dsrust-tpe` (a reproduction of optuna's TPE sampler over categorical choices, same author and
licence), taken from the repository at the same commit. Each file was checked against the repository's
object hash at that commit.

MIPROv2 itself is not a separate crate in that repository; it is part of the main `dsrust` crate and built
on its program model. `src/mipro.rs` of this tool therefore follows that code (`optimize/mipro/search.rs`,
`grounded.rs`, `signatures.rs`) for the parts this task has a use for, and quotes its proposer wording and
its tips; it is written for this tool and is not a copy of a file.

Licence: MIT OR Apache-2.0. The published crates carry no licence file; `LICENSE-MIT` and `LICENSE-APACHE`
here are the files of the repository at that commit. The `Cargo.toml` files are written for this package
(path dependencies, pinned versions); the sources are byte-identical to the published crates. Do not format
or edit them; to update, copy a newer release and record its commit here.

This code is used only by the research tool `guide-optimizer`. It is not compiled into `lesscode-core`.