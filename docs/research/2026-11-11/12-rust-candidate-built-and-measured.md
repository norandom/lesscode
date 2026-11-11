# R86: The Rust specimen candidate, built for the first time: it works, and most of its 40% is formatting

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 10:42 to 11:00 local (driver batch 10:45 to 10:55; times from command output)
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md) (Requirement 7.4),
  restating [R52](../2026-11-09/10-rust-adapter-prototype-gate.md)
- Hypothesis ID and predecessor: closes the open point of
  [R75](01-specimen-controls-rerun-through-driver.md): "the Rust candidate was never built"
- Type: retrospective check of an earlier result

## Decision

Two findings, one for and one against the earlier record.

For: the candidate is real code. Put in the place of the six original functions in `native/src/recording.rs`,
it compiles without a change, the lints pass, and all 375 tests of the crate pass. Under the driver its tests
are about as sensitive as the original's.

Against: its headline figure does not hold. R52 reported 248 canonical lines reduced to 148, a reduction of
40.3%, verdict "pass". The candidate is written more densely than the project's formatter allows, and the
original is formatted. With both files formatted the same way the figures are 248 to 219 lines, a reduction
of 11.7%, and 83 to 76 statements, 8.4%: verdict "marginal" under the tool's own thresholds. About three
quarters of the reported reduction was layout.

This also shows a weakness of `deconvolute density`: it counts lines, and its statement count moved from 70
to 76 when only the layout changed. Both inputs must be formatted the same way before the receipt means
anything.

Separate outcomes: compression (restated downwards), behaviour under the crate's tests (shown for the first
time), test sensitivity (measured). Equivalence is not shown: three of the six functions are not reached by
the tests that ran.

## Hypothesis and falsification

No claim was fixed beforehand; the owner asked for the candidate to be built and tested, and "if it does not
compile, that is the finding". It compiles. The check of the reduction under a common formatter was added
when the format check reported ten differences in the candidate's region.

## Frozen inputs and method

- An export of `native/` at commit `e59f265` under `D:\BuildArtifacts\lesscode\rust-candidate`. In
  `src/recording.rs`, everything from the section "subprocess arms" to the end of the file (the six functions
  and one helper) was replaced by `native/adapters/rust/candidate_lifecycle.rs` from its error type to its
  end. Nothing else was edited. The copy builds into its own directory.
- Tests: `cargo nextest run` for the whole crate; then `deconvolute run` on `src/recording.rs` with
  `--test recording`, three copies, the binary and configuration of the original's run in R75
  (`results/specimens/rust-candidate-config.json`, `rust-candidate-receipt.json`).
- Reduction: `deconvolute density` on the two specimen files as committed, and on copies of both formatted
  with `rustfmt` 1.98.1.

## Evidence and results

| Check | Result |
| --- | --- |
| Build of every target with the candidate in place | succeeds, no warning |
| `clippy -D warnings` | clean |
| Format check | 10 differences, all in the candidate's region |
| Budgets check | passes |
| Whole test suite | 375 of 375 pass |

Driver, whole file with the candidate in place, against the original (R75):

| | Mutants | Killed | Survived | Invalid |
| --- | --- | --- | --- | --- |
| Original `recording.rs` | 133 | 75 | 58 | 0 |
| With the candidate | 131 | 73 | 57 | 1 |
| Original, the six functions | 38 | 20 | 18 | 0 |
| Candidate, the six functions | 36 | 18 | 17 | 1 |

By function, killed of all, candidate (original): `verdict` 10 of 12 (10 of 11), `build_groups` 7 of 8 (9 of
9), `call` 1 of 6 (1 of 5), `probe` 0 of 3 (0 of 3), `run_groups` 0 of 5 (0 of 6), `arm_env` 0 of 2 (0 of 4).
The run was complete and its closing runs passed.

Reduction of the pair:

| Files | Canonical lines | Statements | Verdict |
| --- | --- | --- | --- |
| As committed (R52) | 248 to 148, 40.3% | 83 to 70, 15.7% | pass |
| Both formatted with `rustfmt` | 248 to 219, 11.7% | 83 to 76, 8.4% | marginal |

## Limits and interpretation

- "All tests pass" covers `verdict`, `build_groups` and part of `call`. For `probe`, `run_groups` and
  `arm_env` no test that ran can tell the candidate from the original, or from a broken version of either.
- The Java and TypeScript candidates were not re-measured under a common formatter. The Java candidate has
  four lines with more than one statement and nine lines over 100 characters; whether its reduction shrinks
  in the same way is not known.
- The candidate was not adopted into the crate. This entry only says what it is.

## Tool/spec consequence and next gate

- The README no longer presents the pair's receipt as an example of a substantial reduction without saying
  what formatting does to it, and `docs/driver.md` is unaffected. R52 and R53 get a second dated amendment.
- For the owner: `deconvolute density` should either format its inputs, refuse unformatted ones, or count
  syntax instead of lines. Until then its verdict depends on layout. This also touches the compression
  figures of the other specimens in R53 (24% to 44%), which were measured the same way.

## COBESY check

Outcome first: the candidate works under the tests there are, and its reduction is restated from 40% to 12%.
Replay: export commit, the splice, commands and receipts named. Disproof: the build could have failed and did
not; the reduction was checked against a common formatter and did not survive it. What the tests cannot see
is stated.
