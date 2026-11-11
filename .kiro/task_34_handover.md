# Task handover 34: three driver changes, annotation baseline, report updates, CI for the optimiser, release 1.1.0 prepared

Written 2026-11-11, about 10:05 (Europe/Berlin). Predecessor: `.kiro/task_33_handover.md`.

## Owner decisions at the start of this round (2026-11-11)

- Prepare release 1.1.0; the owner tags.
- Signal handler; remove the functions that report success without running anything; operators for
  arguments and literals in calls.
- Update the COBESY report before and after these changes, with data visualisation of runs and efficiency.
- Slice B: annotation-layer baseline.
- A CI job for the guide optimiser.

## What was done

| Item | Commit | Evidence |
| --- | --- | --- |
| Report, first update with four figures | `5d96060` | `docs/reports/agnostic-driver-feasibility.md` section 8; `docs/reports/figures/` |
| Three false-success functions removed | `8e0e2b9` | R81; 10 tests removed with them |
| Stop handler | `56abd58` | `adapters/driver/interrupt.rs`, `tests/driver_interrupt.rs`; CI green on five platforms |
| Call-argument operators | `eea0109` | design Amendment 4; `tests/driver_contract.rs`; R81 |
| CI for the guide optimiser | `f49d93c` | `tools/check_native.sh`, `.dagger/main.go`; CI green |
| R81: measurement on five scopes | `7821e57` | `results/native-args/` |
| R82, R83: annotation baseline | `fbec74b`, `92067e0` | `results/slice-b/annot-opus`, `annot-mistral` |
| Report, second update with two more figures | `ce00613` | section 9 |
| Release 1.1.0 prepared | this commit | `docs/releases/1.1.0.md`; version bumped; README describes the release |

375 tests pass; lints and budgets clean; a locked release build gives `lesscode-core 1.1.0`, 13.28 MB.

## What went wrong

- `annotate.py` could not read Mistral's reply format. Seventeen calls were paid for and their responses
  saved, but no ledger line was written until I finished them from the saved responses. I had fixed the same
  problem in the baseline runner yesterday and did not check this caller before using it with that model.
- The five measurement batches for the call-argument operators ran at the same time as two model runs. Wall
  times are unusable and one Rust mutant ran into its time limit.
- The first dry run of the optimiser in the CI script made no call at all and still exited 0, because the
  recorded spend was above the tool's default stop. A dry run now ignores spend and fails when the scripted
  improvement is not adopted.
- nextest reported one "leaky" test once in a full run; six repetitions of the run and interrupt tests did
  not reproduce it. Cause unknown.

## Current state

- main carries the release preparation; CI state for it: see the end of this file.
- Recorded model spend: about 29.7 of 40 USD.
- No container of this project exists.
- Work directories under `D:\BuildArtifacts\lesscode` (`specimen-rerun`, `args-run`, `annot-try`, dry-run
  folders) were not deleted; the owner did not choose that item.

## Open, for the owner

1. Tag `v1.1.0` and approve the release environment (steps in `docs/releases/1.1.0.md`).
2. `swap_first_two` in the Java and Rust packs: off, or keep (R81: 2 of 17 build).
3. Annotations: a hardening (sites named by two of three runs, or the union), drift over time, or close
   Slice B with the finding of R83.
4. The Mistral key is unrotated.
5. The Rust specimen candidate has never been built.

## Addition, 2026-11-11, about 10:20

Owner answers: tag the release (yes); switch the argument swap off for Java and Rust (yes, done in the packs,
tests and documents); the Mistral key is no longer an open item. The Slice B answer was "yes" to a list of
three alternatives and is asked again.

## CI

Run 42991301300 for commit `3316cbc`: success on all five jobs (Linux x86_64 through Dagger, Linux aarch64,
Windows, macOS on Intel and on Apple silicon), finished 2026-11-11, 10:06 local. That commit contains the
version bump and every code change of this round.

The runs for `ce00613` and `3455604` failed, on every platform, at the format check: the example added with
R82 (`native/examples/annotation_sites.rs`) was committed unformatted, and I pushed three commits on top
without running the format check again. `3316cbc` formats it. Tag from `3316cbc` or later, not from
`3455604`.
