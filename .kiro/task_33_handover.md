# Task handover 33: driver task group 7 and task 8.3; Slice B node kinds and MIPROv2

Written 2026-11-11, about 08:35 (Europe/Berlin). Predecessor: `.kiro/task_32_handover.md`.

## Owner decisions at the start of the session (2026-11-11)

- Do all of driver task group 7, then 8.3. No release, no version bump.
- None of the four Rust points of R71 goes into the shipped Rust pack.
- Try the grammar's node kinds as input for pack authors, on both models.
- Raise the stop for model spend to 40 USD and run the MIPROv2 comparison on Opus.

## Driver: what was done

| Task | Commit | What |
| --- | --- | --- |
| 7.1 | `545b1b2` | `adapters/driver/contract.rs`: the adapter traits answered by the driver; six control kinds mapped onto operators; `MutantOutcome::NotRun` |
| 7.3 | `964921c` | six adapter files and their tests deleted (1,665 lines removed in all); scan test for the removed fragments |
| 7.2 | `6904b7a`, `ec3a543` | R75: preregistration, reruns, amendments to R49 to R53 |
| fix | `0d0da64` | a void closing run names the copy and quotes the failing output |
| 7.4 | `d4da9a8` | README, native README, adoption guide, architecture page, deconvolute skill |
| 8.3 | this commit | validation report, R80, design Amendment 3 |

## Evidence

- 382 tests pass (410 before the deletion: 29 removed with the deleted code, 1 added); lints and budgets clean.
- R75 receipts and configurations: `docs/experiments/annotation-driver/results/specimens/`, including the void
  Rust run and the two Java attempts that could not reach the container engine.
- Validation: `.kiro/specs/agnostic-annotation-driver/validation-report.md`.

## What went wrong and what it showed

- The first Rust rerun was void. Cause: `CARGO_TARGET_DIR` is set globally on this machine, three copies
  shared it, and Cargo shares compiled units between checkouts of one package. 16 of 133 classes were wrong.
  The closing run caught it. A hand experiment in a copy then gave this repository's own tests a stale
  library (two failures in `tests/recording.rs`) until `cargo clean -p lesscode-core`.
- The Podman machine was not running. I did not start it. Java ran natively (JDK 21, Maven 3.9.9, offline).
- My sense of elapsed time was off by more than a factor of two during the session; all times in the
  entries are taken from files and command output.

## Slice B: what was done

| Entry | What | Result |
| --- | --- | --- |
| R76 | preregistration: node kinds as author input, two arms per model | four claims |
| R78 | result | Mistral: invalid kinds 10 to 1 of 20, admission 5 to 6 of 20; Opus 15 and 15. Not adopted |
| R77 | preregistration: MIPROv2-style engine on Opus | three claims |
| R79 | result | chosen guide 0.50 on the scoring set; Rust 0 of 5 in the final judgement (GEPA: 5 of 5). Not adopted |

Tooling: `native/examples/grammar_kinds.rs`; `--kinds-dir` in `author_baseline.py`;
`tools/guide-optimizer/` with `--engine mipro` and the copied `dsrust-tpe` (`3f68560`).
Spend today: 4.46 USD (node kinds) and 5.82 USD (MIPROv2); recorded total about 25.5 of 40 USD.

## Current state

- main at the commit of this handover; CI state is in the validation report's last section.
- No containers of this project exist; the Podman machine is not running.
- Work directories under `D:\BuildArtifacts\lesscode\specimen-rerun` (about three Rust build trees) can be
  deleted by the owner; I did not delete them.

## Next steps (all for the owner)

1. A release that contains the driver, or a statement that it waits.
2. A signal handler to close Requirement 5.7.
3. The two functions that report success without running anything (`PureCoreVerifier::compare` on the
   adapters, `NativeMutantGenerator::generate`): remove, implement, or mark.
4. Slice B: preregister an annotation-layer baseline, or close the pack-authoring question.
5. Smaller: operators for changed arguments and literals in calls; a CI job for the guide optimiser; rotate
   the Mistral key.
