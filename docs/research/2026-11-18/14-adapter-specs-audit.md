# R148: The three adapter specs are not delivered as specified; the driver covers static analysis and fault runs, and nothing covers bounded comparison or runtime observation

- Report date/time and timezone: 2026-11-18, 13:15, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 13:06 to 13:12 (from the clock and the test output)
- Status: completed
- Spec: [typescript-deconvolution-adapter](../../../.kiro/specs/typescript-deconvolution-adapter/requirements.md), [java-deconvolution-unification](../../../.kiro/specs/java-deconvolution-unification/requirements.md), [rust-deconvolution-unification](../../../.kiro/specs/rust-deconvolution-unification/requirements.md); task 1.3 of the [corrective tasks](../../../.kiro/corrective-tasks.md)
- Hypothesis ID and predecessor: extends [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md) to the specs it left out; follows [R75](../2026-11-11/01-specimen-controls-rerun-through-driver.md), [R86](../2026-11-11/12-rust-candidate-built-and-measured.md), [R87](../2026-11-11/13-density-by-syntax-specimens-restated.md) and [R137](03-gate-result-literals.md)
- Type: retrospective audit with reruns; no model call

## Decision

The three specs said `completed` and `GO`. Each requirement was checked against the code that a
command reaches today. None of the three is delivered as specified. Their `spec.json` now carries a
finding per requirement, `phase: not-delivered` and the withdrawn disposition, like the two specs of
R134.

The three requirement texts are identical apart from the language, and so is the finding.

| Requirement | Finding | What exists today |
| --- | --- | --- |
| 1 State and effect coverage | **Partly, through the driver. 1.2 and 1.4 not delivered.** | `deconvolute analyze --lang <lang>` lists state-write sites, function boundaries and the constructs the pack declares as gaps, and checks write sites against a frame file. Nothing observes a running program, so no observed mutation is compared with a frame (1.2) and purity is not backed by runtime observations (1.4). |
| 2 Bounded semantic comparison | **Not delivered.** | No command executes a baseline and a candidate in the target runtime over a finite domain. The adapters no longer implement the comparison trait. The reported "100 Cartesian cases, 0 divergences" came from a function that returned `Verified` without running anything (R75, removed in `8e0e2b9`). |
| 3 Mutation sensitivity | **Partly, through the driver. 3.2 not met.** | `deconvolute run` plants the pack's operators and runs the project's own tests in copies, with the outcomes killed, survived, timeout, invalid and error. Of the twelve preregistered gate controls, five have a driver mutant and seven do not. R75's rerun left survivors in every language, so no oracle was qualified. The adapters' `generate` only says whether a mutant exists and never reports a kill. |
| 4 Runtime observation and isolation | **Not delivered as specified.** | `NativeSpecimenRunner` starts a process with a deadline and reads protocol events from its output. Only tests reach it. No state, stream or error event is recorded or replayed for these languages. The probes (`runner.mjs`, `observe.mjs`, `ProbeMain`) are in `native/adapters/` and part of no build. Copies, time limits and the hashes of pack, configuration, tests and sources exist in `deconvolute run` (4.3, 4.4 in part). |
| 5 Qualification and compatibility | **5.2 fails by the current measure. 5.1 replaced. 5.3 rested on gate literals.** | The line and statement counters were removed in `9c48ab8`. `deconvolute density` counts syntax, and all three candidates are `marginal` today. The gate scripts carry fixed results (R137). The remaining adapter tests pass (5.4). |

## Hypothesis and falsification

Hypothesis from task 1.3: after `964921c` deleted the text-matching analysers and operators, the three
adapter modules are thin wrappers and the specs' requirements are met only where the driver meets
them. It would be rejected by a command that reaches an adapter module, or by a command that runs
both versions of a specimen function in its runtime. Neither was found.

## Frozen inputs and method

- Repository at `fb95f9a`; binary `lesscode-core 2.0.0`, debug build of 18 November, 10:17.
- Java: `apache/commons-csv` at `2bd66bf181d44ffc92806f585d3955e915666f7f`, clean, `CSVPrinter.java`, against `native/adapters/java/candidate_CSVPrinter.java`.
- TypeScript: `OpenUltraCode` at `b1bf7ce49f273af8eaecebbaf6115ddf5d3d06d2`, clean, `src/degradation.ts`, against `native/adapters/typescript/candidate_degradation.ts`.
- Rust: `native/adapters/rust/baseline_lifecycle.rs` against `candidate_lifecycle.rs`.
- Reachability: a search of `native/src` for the three adapter types and `NativeSpecimenRunner`. Outside their own modules they appear only under `native/tests/`.
- Commands, per language: `lesscode-core deconvolute analyze --lang <lang> --source <file>` on both versions and `lesscode-core deconvolute density --lang <lang> --baseline <file> --candidate <file>`.
- Tests: `cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml -E 'binary(typescript_adapter) | binary(typescript_e2e) | binary(java_adapter) | binary(java_e2e) | binary(rust_adapter) | binary(rust_e2e) | binary(driver_contract) | binary(adapter_contract) | binary(specimen_runner)'`.
- Not rerun: `deconvolute run` on the specimens. R75's figures are quoted.

## Evidence and results

Static analysis by the driver, today:

| Specimen | Version | Sites | State-write sites | Function boundaries | Declared gaps |
| --- | --- | --- | --- | --- | --- |
| Java `CSVPrinter` | original | 204 | 65 | 23 | 0 |
| Java `CSVPrinter` | candidate | 203 | 61 | 23 | 0 |
| TypeScript `degradation` | original | 55 | 4 | 14 | 0 |
| TypeScript `degradation` | candidate | 36 | 0 | 13 | 0 |
| Rust lifecycle | original | 172 | 16 | 8 | 0 |
| Rust lifecycle | candidate | 155 | 14 | 7 | 0 |

Size by syntax, today, against the thresholds of 20% of tokens, 400 tokens and 10% of nodes:

| Specimen | Tokens | Nodes | Lines | Verdict | Validation report said |
| --- | --- | --- | --- | --- | --- |
| Java | 1420 → 1341 (−5.6%) | 1212 → 1159 (−4.4%) | 221 → 136 (−38.5%) | marginal | lines −38.46%, statements −27.68% |
| TypeScript | 1283 → 1042 (−18.8%) | 1064 → 842 (−20.9%) | 201 → 113 (−43.8%) | marginal | lines −43.78%, statements −40.38% |
| Rust | 1846 → 1716 (−7.0%) | 1460 → 1350 (−7.5%) | 249 → 149 (−40.2%) | marginal | lines −40.32%, statements −15.66% |

The line figures reproduce. By syntax the candidates are 6% to 19% smaller, as R87 found.

Fault runs through the driver, from R75 (11 November, not rerun): TypeScript candidate 5 killed and 0
survived, original 15 and 1; Java candidate 78 and 11, original 78 and 12; Rust original, six
lifecycle functions, 20 and 18.

Tests today: 32 run, 32 passed. They cover the wrappers' profile names, analysis of small files
through the driver, the `NotRun` outcome of `generate`, process runs of `java`, `node` and
`rustc`, and the lookup of the twelve gate controls (five found, seven not).

What the validation reports named and no longer exists: `adapters/<lang>/analysis.rs` and
`mutation.rs` (deleted in `964921c`), `metrics.rs` (deleted in `9c48ab8`), and the test files of the
same names. Each adapter is one file of 44 to 62 lines that passes requests to the driver.

Evidence: `D:\BuildArtifacts\lesscode\corrective-audit\task74\1.3\` (six analysis receipts, three
density receipts, `nextest.log`, `evidence-sha256.json`; 320,807 bytes).

## Limits and interpretation

- The gate scripts were not rerun. R137 lists their literal results and R75 what their controls did.
- "0 declared gaps" means the shipped pack lists no gap construct that occurs in these files. It does
  not mean every effect is resolved. The Rust lifecycle code starts processes, and the analysis
  reports no assumption about that.
- Requirement 4.4's binding to a runtime identity was not checked in `deconvolute run`.
- The full suite was not rerun for this task; only the nine binaries named above.

## Tool/spec consequence and next gate

- `spec.json` of the three specs: `phase: not-delivered`, the GO withdrawn, an `audit` block with one
  finding per requirement.
- Task 3.2 moves the candidates and probes out of `native/adapters/`. The test of the twelve controls
  reads the three candidates and has to move with them.
- Task 3.3 corrects `roadmap.md`, which still says "completed with GO" and "100% adverse sensitivity".
- The three wrapper modules and `NativeSpecimenRunner::run` have no command. Task 5.2's reachability
  test will list them; whether they stay is a decision for task 3.1's group.
- Bounded comparison for Java enters core through the port of R146 and R147. TypeScript and Rust have
  no prototype for it.

## COBESY check

Each row names the command or the search behind it. Figures taken from R75 are marked as not rerun.
"Not delivered" means no command does the thing today, not that it would fail if built.
