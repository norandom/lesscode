# Task handover 40: task group 4 of gradual-refactoring-steps (pin, check, verdict, evidence)

Written 2026-11-11, 17:51 (Europe/Berlin, from the clock). Predecessor: `.kiro/task_39_handover.md` (the
time written there, "about 18:30", was wrong; it was about 17:25).

## What was done

| Task | Commit | Result |
| --- | --- | --- |
| 4.1 `step pin` | `08259ef` | admitted / rejected with which of the two / not judged; lists remaining unnoticed faults |
| 4.2 to 4.4 `step check`, verdict, evidence | `e985f6e` | `step-verdict-v1` for accepted, refused and not judged; `step-evidence-provisional-v0` |
| Guide and design Amendment 3 | this commit | `docs/steps.md` sections 2 to 4 |

378 tests pass; format, lints and budgets clean; `kiro-cli check` 0 errors, 0 warnings.

New files under `native/src/step/`: `trees.rs` (trees compared, copies), `faults.rs` (driver runs read
back), `pin.rs`, `context.rs` (committed declaration, revisions, hashes), `verdict.rs`, `check.rs`
(checks without a run), `check_runs.rs` (tests, faults, comparison, evidence). Tests:
`tests/step_pin.rs`, `tests/step_check.rs` on a small git repository (`tests/step_support/mod.rs`) whose
test file is a list of expectations for the toy check.

## Next: task group 5 (guard and ledger)

- 5.1 `step/ledger.rs` and `step record`: append-only JSON lines with the hash of the line before; last
  accepted unnoticed number per function; `--release`; `--override --reason`. Then wire the ledger number
  into `check_runs.rs` `compare` (Requirement 7.5, deferred from 4.3).
- 5.2 `step guard`: recompute `Context::hashes_of` for the pushed revision and compare with the verdict
  (`revision` may be an ancestor once verdict and ledger are committed; the hashes must be equal).
- 5.3 `step install-hook` beside the budgets hook; pipeline section in `docs/steps.md`.
- 5.4 protection tests.

## Notes

- I committed 4.1 once with a failing budget check (a function over 560 tokens) because the commit was
  chained after the checks without a gate. Fixed before any push by amending the unpushed commit. Commits
  are now gated on the checks' exit codes.
- Example binaries are not rebuilt by `cargo nextest run --test X`; after changing `examples/toy_check.rs`
  run `cargo build --examples`.
- The driver reports a command that cannot start as "a command could not be started", no longer as failing
  tests.
- Work directories under `D:\BuildArtifacts\lesscode` (t11, t12, t13, t33, t41, steps, step1, budget-case,
  figure-check) can be deleted.
- Model spend recorded: about 32.9 of 40 USD. Slice B is closed. No container of this project exists.

## CI

Filled in once the run for this commit has finished.
