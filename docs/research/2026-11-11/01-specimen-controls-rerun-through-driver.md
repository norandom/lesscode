# R75: The specimen gates' adverse controls, looked up and rerun through the driver

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: preregistered 2026-11-11 before the runs; results are added below when the runs end
- Status: preregistered, runs pending
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md), tasks 7.1 and
  7.2 (Requirements 7.3, 7.4, 7.5, 9.4)
- Hypothesis ID and predecessor: restates the adverse-control figures of
  [R49](../2026-11-09/07-native-closure-prototype-gate.md),
  [R50](../2026-11-09/08-typescript-adapter-prototype-gate.md),
  [R51](../2026-11-09/09-java-adapter-prototype-gate.md),
  [R52](../2026-11-09/10-rust-adapter-prototype-gate.md) and
  [R53](../2026-11-09/11-multi-language-deconvolution-synthesis.md); follows
  [R73](../2026-11-10/19-native-driver-batches-on-gate-scopes.md)
- Type: retrospective reading of the earlier gates' scripts, and a confirmatory rerun

## What the earlier figures were (read from the gate scripts, 2026-11-11)

The three language gates each reported "4 of 4 adverse controls killed". Reading the scripts that wrote those
receipts shows three different things behind the same words:

| Gate | Script | What "killed" was |
| --- | --- | --- |
| R52 Rust | `.kiro/specs/rust-deconvolution-unification/run_gate.py`, `evaluate_adverse_controls` | Four entries written as `"killed": True` in the script, each with a witness sentence written by hand. No mutant was made and nothing was run. |
| R50 TypeScript | `.kiro/specs/typescript-deconvolution-adapter/run_gate.py`, `verify_adverse_controls_sensitivity` | Three text replacements in the candidate, each run through a probe on one hand-picked input and compared with the baseline's output: measured. The fourth (`TS_ADV_3_Buffer_Drop`) is `stream_killed = True` in the script. |
| R51 Java | `.kiro/specs/java-deconvolution-unification/run_gate.py`, `evaluate_adverse_controls` | Four text replacements in the candidate, each compiled and run through a probe and compared with the baseline's output: measured. A mutant that did not compile or whose probe raised an exception also counted as killed. |

In none of the three was a mutant run against the project's own tests. Two further points from the same
scripts:

- R52's "100 Cartesian cases, 0 divergences" compared pairs of Python functions written inside the gate
  script, not the Rust baseline and candidate. The candidate file `native/adapters/rust/candidate_lifecycle.rs`
  is not part of any build, and the gate's "314 tests passed" ran the unchanged repository.
- The Java and TypeScript gates ran the project's tests against the candidate by copying it over the file in
  the owner's upstream checkout and copying the original back afterwards.

The operators that were then built into the binary (`adapters/{rust,java,typescript}/mutation.rs`, deleted on
2026-11-11 in commit `964921c`) reported a control as killed when the mutated text differed from the original.

R49's four controls are of another kind: they exercise the Python model's monitors on constructed inputs
(a missing audit record, a shortened stream, a `None` input, an order of two checks in a four-line sample) and
do not mutate a specimen. They are not rerun here.

## Hypothesis and falsification

Fixed before the runs:

- P1. With each candidate in the place of its original, the project's own tests pass unmutated, so the run is
  not void. (Rust: the candidate is not buildable in place, so the Rust run uses the original code in
  `native/src/recording.rs`, from which the baseline specimen was copied.)
- P2. Every gate control that has a driver mutant at its place (five of twelve, pinned in
  `native/tests/driver_contract.rs`) is killed by the project's own tests. Refuted by any such mutant that
  survives. A control without a driver mutant is recorded as not reproducible (Requirement 7.5) and is not
  counted as killed or survived.
- P3 (descriptive, no threshold): the share of killed mutants over killed plus survived for the whole file,
  for candidate and original side by side where both can run.

## Frozen inputs and method

- Binary: `lesscode-core` built from commit `964921c`, shipped packs as embedded.
- Source trees: isolated copies under `D:\BuildArtifacts\lesscode\specimen-rerun` (TypeScript and Java
  candidates in copies of the gate clones at the gate commits; an export of `native/` at `964921c`) and the
  gate clones under `D:\BuildArtifacts\lesscode\annotation-driver\scopes` for the originals. No upstream
  checkout is read or written.
- Configurations: `docs/experiments/annotation-driver/results/specimens/*-config.json`. Tests: Node's test
  runner on `tests/degradation.test.ts` after `tsc`; `CSVPrinterTest` through Maven in a container; `cargo
  nextest run --test recording`.
- The mapping of the six control kinds onto driver operators is `adapters/driver/contract.rs`
  (`operators_of`); `TruncatedStream` has no operator.

## Evidence and results

Pending.
