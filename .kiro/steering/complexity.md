# Complexity caps and task planning (Lesscode)

## Solution-first accounting amendment (2026-11-07, updated after complexity audit)
[U1-U7](../specs/deconvolution-native-core/prototype-gate.md) proposes `<= 600`
added/modified maintained canonical lines of shared + adapter-wiring code across `<= 2`
attempts, subject to frozen-plan approval.
- **Three-stratum whole-replacement accounting (`U5`)**: Empirical measurement of the
  14 E3/E4 baseline apparatus files under `ruff==0.17.3 format --line-length 100` shows
  `2,222` as-written lines expand to `3,963` canonical lines (`e1b_oracle.py`: `406 -> 659`;
  `e4_oracle.py`: `534 -> 1,258`), of which `1,498` canonical lines in the two oracles are
  specimen-specific `op_*` methods/fixtures (`854`) and transcribed `SEEDS`/generators
  (`644`). Whole-replacement accounting therefore reports all three strata
  ($B_{\text{app}}, B_{\text{ops}}, B_{\text{seeds}}$ and $C_{\text{app}}, C_{\text{ops}}, C_{\text{seeds}}$)
  and requires both $C_{\text{app}} \le 0.80 \cdot B_{\text{app}}$ with
  $B_{\text{app}} - C_{\text{app}} \ge 200$ canonical lines AND
  $C_{\text{total}} \le B_{\text{total}} - 200$ canonical lines with $C_{\text{seeds}} = B_{\text{seeds}}$.
- **Brownfield entrypoint & verbatim-migration rules (`U7`)**: Five of the six existing
  files in `native/src/` (`lib.rs` 618, `model.rs` 511, `metrics.rs` 1,530, `scope.rs` 1,598,
  `compare.rs` 1,667; `main.rs` is 225) predate the 400-line per-file cap. For `U7`, all
  new/modified shared modules and adapter wiring are charged in full; if `lib.rs` or
  `main.rs` is touched solely for module/subcommand registration ($\le 20$ diff lines,
  zero logic), only the diff lines are charged; verbatim-migrated `op_*`/`SEEDS` blocks
  verified by AST/diff parity count in $C_{\text{ops}} / C_{\text{seeds}}$ (`U5`) while
  any new/modified wrapper lines around them charge to `U7`.
- Production checker (`syn` + `proc-macro2`, already in `native/Cargo.lock`) follows
  unlock/phase approvals and precedes capped feature work; explicit inventory/existing
  tools suffice for authorized disposable prototypes.

Owner request, 2026-11-07. Convention adapted from `../QOpenBB/.kiro/steering/complexity.md`
(its application-specific architecture limits do not transfer; the mechanism does).
Purpose: the deconvolution-native-core suite must stay a small, self-contained tool —
no vast codebase. Policy text alone is not enforcement: an executable checker over
`.kiro/complexity-budgets.json` is a foundation task that must be accepted before any
capped feature work starts.

## Per-file / per-function caps (maintained production Rust under `native/`)

| Metric | Cap |
| --- | --- |
| Physical lines per maintained production file | 400 |
| Logical syntax units per maintained Rust file | 300 |
| Physical lines per function | 80 |

Physical lines include comments and blank lines; normalize LF/CRLF/CR; do not count a
terminal newline as an extra line. The logical metric is `rust-syntax-units-v1`:
Rust AST items/statements counted by a `syn`-based parser (`syn` with `visit` +
`proc-macro2` `span-locations`, already present in `native/Cargo.lock`), defined and
version-pinned by the checker foundation task. Nested function bodies are measured
separately for the function cap but remain in file totals. This measures code size, not
cyclomatic complexity or behavioral correctness. A modified maintained file is checked
as a whole, not only its changed lines.

## Spec-level budgets (deconvolution-native-core, rebalanced 2026-11-07)

| Budget | Cap | Source |
| --- | --- | --- |
| Hand-written Rust lines, first release (Python + Rust adapters) | 3,500 | arbitrated fusion plan 2026-11-07, calibrated against `rustfmt` verbosity |
| Hand-written Rust lines, all three languages | 4,600 | stated up front so the TS/JS extension is a scoped decision |
| Top-level CLI subcommands | 12 | gate coverage moves into verbs/flags |
| Per-component hand-written ceilings (sum = 4,600) | agnostic core 750; driver shims 600 total; mutation-engine readers 450; receipts 250; host guard 250; tree hash/edit scope 250; surface+measures AST code 850; query differential 400; reports (residue/missing-tests/turn/review/budgets) 350; CLI/config/descriptors 450 | rebalanced 2026-11-07 after auditing `native/src/` (6,149 lines) and prototype components: `reports` raised 100->350, `surface_measures_code` 500->850 (AST walkers replace infeasible YAML rules), `cli_config` 350->450; offset by `driver_shims` 900->600, `mutation_engine_readers` 600->450, `agnostic_core` 900->750, `host_guard` 300->250 (`windows-sys`), `tree_hash` 300->250; `pgwire` proxy deferred out of core |

A cap breach means **defer the work** (next release, next language, or a spike), never
pad, compress statements, or split to pass a counter. Research-code line budgets for
experiments remain reported per loop req 11.6; tests, fixtures, docs and generated
files are outside production caps but never hide hand-maintained logic (record
generator + exclusion reason in `complexity-budgets.json`).

## Scope, brownfield baseline and exclusions

- Applies to new or modified maintained production code in `native/` (and any owned
  tooling under `tools/quality/`). Exclusions: `native/target/**`, tests, fixtures,
  `docs/**`, `.kiro/**`, lockfiles, generated/vendor files — all listed with reasons in
  `.kiro/complexity-budgets.json`.
- **Pre-existing `capability-compression-core` files in `native/src/`**: `compare.rs`
  (1,667 lines), `scope.rs` (1,598), `metrics.rs` (1,530), `lib.rs` (618) and `model.rs`
  (511) exceed the 400-line cap at baseline (`main.rs` is 225 lines). All new
  `deconvolution-native-core` code must live in new module files under `<= 400` physical
  lines. Before production task execution modifies `lib.rs` or any other pre-existing
  oversized file, either extract a cohesive submodule so the touched file is `<= 400`
  lines or record a narrow registration-only baseline exception in
  `complexity-budgets.json`.

## Required in tasks

1. Every executable production-code task names its ownership scope and carries an
   observable completion bullet requiring the applicable caps to pass, with the exact
   verified check command. Non-code tasks state why caps do not apply.
2. Foundation task first: the scope inventory (`.kiro/complexity-budgets.json`) and
   the executable combined checker, runnable locally and in CI independently of any
   container builds. Missing scope input must fail, not silently skip.
3. Per-task size evidence plus a final cross-task audit of shared modules; update
   `.kiro/breadth.toml` and the inventory when adding files or responsibilities.
4. Workers receive their task's caps, allowed files and check command together with
   the functional acceptance criteria.
