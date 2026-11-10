# Task handover 32: agnostic driver, task groups 4 to 6, and Slice B runs of the evening

Written 2026-11-10, about 22:10 (Europe/Berlin), during the unattended session planned in
`.kiro/overnight-plan-2026-11-10.md`. Predecessor: `.kiro/task_31_handover.md`.

## Driver: what was done

- Group 4 (`641f460`): `deconvolute analyze` and `deconvolute mutate` run through the driver for Rust,
  Python, TypeScript and Java. Receipts `deconvolution-analysis-v2` and `deconvolution-mutation-v2`;
  `mutate --limit N --seed S`; a source that does not parse is refused (exit 3) before anything is written.
- Group 5 (`bbfa3da`): `adapters/driver/workspace.rs`: copies without version-control metadata, refusal of a
  work directory in or around the source root, restore journal with verification and recovery.
- Group 6 (`50f80b6`): `config.rs`, `process.rs`, `runner.rs`, `store.rs`, `run_command.rs`;
  `deconvolute run --config C --work W [--workers N] [--annotations A] [--no-reuse] --out R`; exit 0 complete,
  2 incomplete, 3 void or bad input.

## Evidence

- 404 tests pass with cargo-nextest (389 before); `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, doctests and the budgets check are clean locally.
- `native/tests/cli_deconvolute.rs`: analysis and mutants for four languages, gaps and frame violations,
  parse failure, seeded selection, the source file left unchanged.
- `native/tests/driver_run.rs` with the example `native/examples/toy_check.rs` as the project's test
  command: killed and survived, timeout, invalid, error (no runner evidence, resource pattern), void runs,
  three copies giving the classes of one, reuse and `--no-reuse`, annotated runs, refusal of an unsafe work
  directory, recovery of an interrupted copy.
- CI for `50f80b6`: see "Current state".

## Tests whose expectations changed on purpose (group 4)

In `native/tests/cli_deconvolute.rs` the seven tests that pinned the old adapters' output were replaced:

- Receipt names are `...-v2`; the analysis receipt has `sites`, `function_boundaries` and `pack_sha256`.
- Python: `analyze` and `mutate` return results (exit 0 or 1) instead of the refusal with exit 2.
- Mutant files are named `mutant_NNNN_<operator>.<extension of the source>`; their number depends on the
  file, not on four fixed specimen kinds.
- Gaps come from the pack's `gaps` key, not from text patterns.

The per-language adapter tests (`rust_analysis.rs`, `java_mutation.rs` and the like) still test the old
adapter structs, which are still in the tree and no longer reached from the command line. Group 7 removes
both.

## Deviations (design Amendment 2)

Own process supervision instead of reuse; `gaps` as a pack key; `run_command.rs` and `analysis.rs` as extra
files; no Python adapter struct; parse-failure position is the start of the unparsed region; no signal
handler, so Requirement 5.7 holds for limits and normal exits only. The driver is about 2,900 production
lines in eighteen files, every file within budget.

## Slice B: what was done this evening

| Entry | What | Result |
| --- | --- | --- |
| R68 | Baseline, Opus, guide version 3 | 20 of 20 admitted; Rust packs drift unnoticed |
| R69 | Rerun with hand-written Rust controls and two Python pairs | Rust 0 of 5 on the first reply; Java 3 of 5 lack `throw` |
| R70 | Preregistration of the guide optimisation, three amendments | GEPA engine copied from `dsrust`; barrier, frontier, two models |
| R71 | Optimisation on Opus | every first reply admitted for all four languages; Rust still differs from the hand pack in four fixed points; not adopted |
| R72 | Baseline, Mistral Large 4 | 5 of 20 first replies admitted; invalid node kinds; Java repaired in one turn |

Running at the time of writing: the optimisation on Mistral
(`results/slice-b/optimise1-mistral`, stop at 2.50 USD).

Spend at list prices: about 13.2 USD recorded before the Mistral optimisation; project stop 20 USD. The
figure of 8 USD for the night, which I proposed in the plan, is exceeded by the Mistral optimisation; I went
on because the owner asked for as much as possible and the project stop is not at risk.

## Current state

- main at `1d1b339`; CI green through `419da6f`; the run for `50f80b6` (groups 4 to 6) was in progress when
  this was written. A red run there is the first thing to fix.
- `tools/guide-optimizer/` is not built by CI.
- No containers from this project are running.

## Next steps

1. Read the CI result for `50f80b6` and `1d1b339`; fix anything red (likely places: the process-group kill
   on Linux and macOS, the example binary's path under nextest, temporary directories).
2. Record the Mistral optimisation as its own research entry beside R71.
3. Driver group 7 (delete the six adapter files, rerun the specimen experiments, documentation) and group 8
   (full batches on the gate scopes, five platforms) need the owner: group 7 deletes code and changes
   published figures, group 8 needs containers for Java.
4. Owner decisions listed in `.kiro/overnight-plan-2026-11-10.md`, plus: the four Rust points of R71
   (empty statement, `?` rewrite, aborting macros, `match` guards).
