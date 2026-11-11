# R75: The specimen gates' adverse controls, looked up and rerun through the driver

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: preregistered 2026-11-11 before the runs (commit `6904b7a`); runs 2026-11-11, 07:31 to
  08:17 local, times from the receipt files
- Status: completed
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

## Decision (added after the runs)

The figure "20 of 20 adverse controls killed" in R53, and the "4 of 4" of R50 to R52, should no longer be
cited as evidence that the tests of those specimens detect faults. Of the twelve hand-made controls of the
three language gates, five have a counterpart among the driver's mutants. Run against the projects' own
tests, four of those five are killed and one survives. That one, a dropped `lock.unlock()` in the Java
specimen, was counted as killed in R51 because the hand-made mutant did not compile.

What replaces the old figures is ordinary mutation testing of the same code with the projects' own tests:

| Code | Mutants | Killed | Survived | Invalid | Timeout | Killed of killed plus survived |
| --- | --- | --- | --- | --- | --- | --- |
| TypeScript candidate `degradation.ts` | 6 | 5 | 0 | 1 | 0 | 5 of 5 |
| TypeScript original `degradation.ts` | 17 | 15 | 1 | 1 | 0 | 15 of 16, 94% |
| Java candidate `CSVPrinter.java` | 93 | 78 | 11 | 2 | 2 | 78 of 89, 88% |
| Java original `CSVPrinter.java` | 94 | 78 | 12 | 2 | 2 | 78 of 90, 87% |
| Rust original `recording.rs`, whole file | 133 | 75 | 58 | 0 | 0 | 75 of 133, 56% |
| Rust original, the six lifecycle functions | 38 | 20 | 18 | 0 | 0 | 20 of 38, 53% |

Separate outcomes: these are measurements of test sensitivity. They say nothing new about whether the
candidates behave like their originals, and the compression figures of R50 to R52 are untouched.

## Evidence and results

### The predictions

| Prediction | Result |
| --- | --- |
| P1: the unmutated tests pass with each candidate in place | Holds for TypeScript (3 tests) and Java (168 tests run, 7 skipped). Rust: run on the original code, as stated. |
| P2: every control with a driver mutant is killed by the project's tests | Refuted: 4 of 5. |
| P3: candidate and original side by side | See the table above. Java: 88% and 87%. TypeScript: the candidate has 6 mutants against 17, because it is written as expressions the pack has few operators for. |

### The twelve controls

| Control | What the gate changed | Old record | Driver mutant at that place | Class with the project's tests |
| --- | --- | --- | --- | --- |
| RUST_ADV_1 | removed the `taskkill` call | `"killed": True` in the script | none: the call is the value of a `let` | not reproducible |
| RUST_ADV_2 | inverted `arm_exit != 0` | `"killed": True` in the script | negation | killed |
| RUST_ADV_3 | disabled the check `level2_ops > 0 …` | `"killed": True` in the script | negation | killed |
| RUST_ADV_4 | dropped the `PYTHONPATH` entry | `"killed": True` in the script | none: an element of a list | not reproducible |
| JAVA_ADV_1 | removed `newRecord = false` | killed by the probe | dropped statement | killed |
| JAVA_ADV_2 | removed `lock.unlock()` | "killed at compile time": the replacement left a syntax error | dropped statement | **survived** |
| JAVA_ADV_3 | replaced the argument `newRecord` by `true` | killed by the probe | none: a changed argument | not reproducible |
| JAVA_ADV_4 | removed the flush | killed by the probe | dropped statement | killed |
| TS_ADV_1 | `"warning"` to `"notice"` in a call | killed by the probe | none: a literal inside a call | not reproducible |
| TS_ADV_2 | removed two chained `.replace(…)` calls | killed by the probe | none | not reproducible |
| TS_ADV_3 | "stream buffer drop" | `stream_killed = True` in the script | none: not a change of source | not reproducible |
| TS_ADV_4 | appended `.reverse()` | killed by the probe | none | not reproducible |

The Rust classes are those of the same conditions in `native/src/recording.rs` (lines 859 and 867), since the
candidate file is not buildable in place. Seven controls are recorded as lost under Requirement 7.5; no
specimen-specific rule was reintroduced for them. Under the six kinds of the old interface this is the
mapping of `adapters/driver/contract.rs`; `TruncatedStream` has no operator.

### What survives

- Java: of the 11 survivors in the candidate, 7 are a dropped `lock.unlock()`; the original has 7 of 12. The
  test class runs on one thread and a lock that is never released does not show. The others are a dropped
  `newRecord = true`, two swaps of adjacent checks and one negation.
- Rust: 18 of the 38 mutants in the six functions survive. By function, killed of all: `verdict` 10 of 11,
  `build_groups` 9 of 9, `call` 1 of 5, `probe` 0 of 3, `run_groups` 0 of 6, `arm_env` 0 of 4. The last three
  are not called by `tests/recording.rs`, the test file this run used. Two tests in that file need a recorded
  environment under `.runs/` and skip themselves in a clean export; whether they would reach these functions
  was not checked.
- TypeScript: the one survivor in the original is a boundary swap (`>` to `>=`).

### A void run, and what caused it

The first Rust run (three copies) was voided by the driver: after the mutants, the unmutated tests failed in
one copy (`rust-baseline-shared-target-void-receipt.json`). The cause was not in the code under test. This
machine sets `CARGO_TARGET_DIR` globally, the copies inherited it, and Cargo then shares compiled units
between checkouts of the same package. A copy could run its tests against a library built from another
copy's mutant. Rerun with `CARGO_TARGET_DIR=target`, so that each copy builds inside itself, the run is
complete and its closing runs pass (`rust-baseline-receipt.json`). Between the two runs 16 of 133 mutants
have a different class; for example the negation of `arm_exit == 0`, which a test demonstrably kills, was a
survivor in the void run.

Three consequences were drawn at once: the void reason now names the copy and quotes the failing output
(commit `0d0da64`, with a test); `docs/driver.md` says that each copy must build into its own directory; and
the same shared directory was found to have given this repository's own test run a stale library once this
morning (two failures in `tests/recording.rs`, gone after the package's build output was cleared). The Rust
batch of R73 is not affected: its configuration set its own build directory and ran one copy.

### Java ran natively, not in a container

The Podman machine was not running, and I do not start or stop it. The first two Java attempts therefore
ended as void runs with the message that the container engine could not be reached
(`java-candidate-receipt.json`, `java-baseline-receipt.json`; nothing was mutated). The runs reported here
use the JDK 21 and Maven 3.9.9 installed on this machine, offline, started without a shell
(`java-*-native-config.json`). Maven's test plugin starts its test process through `cmd.exe` on Windows; that
is Maven's doing and cannot be configured away here.

## Limits and interpretation

- The Rust candidate was never built, neither by the gate of R52 nor here. Its "finite-domain parity" in R52
  compared Python functions written for the gate. Whether `candidate_lifecycle.rs` behaves like the original
  is not known.
- The Java and TypeScript candidates pass their projects' tests; that was already so in R50 and R51 and is
  the only behavioural statement about them that stands.
- A high share of killed mutants in the TypeScript candidate rests on 5 mutants.
- One run per configuration, on a machine that was running other batches at the same time. Wall times are
  not comparable with R73.
- The Python specimen and its "8 of 8" in R53 were not re-examined.

## Tool/spec consequence and next gate

- Tasks 7.1 and 7.2 are done. The amendments to R49 to R53 point here and leave their text as it was.
- Two other places in the library report success without running anything and are outside the six deleted
  files: `PureCoreVerifier::compare` on the three language adapters returns `Verified` with the number of
  cases a domain would have, and `NativeMutantGenerator::generate` (`mutant_generator.rs`) reports every
  control as killed ("default mock execution"). Neither is reachable from the command line. Whether to remove
  them is the owner's decision; they are named in the amended architecture page so that nobody relies on
  them.
- For the driver: operators for a changed argument and a changed literal inside a call would have covered
  three of the seven lost controls. That is a pack and engine extension to be decided separately.

## COBESY check

Outcome first: the old figures are withdrawn as evidence of test sensitivity, with the reason per gate, and
replaced by measured classes. Replay: binary commit, configurations and receipts in
`docs/experiments/annotation-driver/results/specimens/`, including the void and the failed attempts.
Disproof: P2 was refuted and is reported as refuted. What was not examined (the Rust candidate's behaviour,
the Python figures) is stated.
