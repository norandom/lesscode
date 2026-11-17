# R134: Audit of the port and reduction claims of 8 and 9 November: the reductions and the Z3 results reproduce, the verification claims and the port do not

- Report date/time and timezone: 2026-11-17, 21:45, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-17, 21:05 to 21:40 (from the clock and the test output)
- Status: completed for the two specs named; other specs not audited
- Spec: [native-deconvolution-closure](../../../.kiro/specs/native-deconvolution-closure/requirements.md), [state-boundary-deconvolution](../../../.kiro/specs/state-boundary-deconvolution/requirements.md)
- Hypothesis ID and predecessor: re-examines [R42](../2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md), [R45](../2026-11-09/03-e6-generalization-trial-qualification.md) and the port of 2026-11-09; follows [R75](../2026-11-11/01-specimen-controls-rerun-through-driver.md) and [R87](../2026-11-11/13-density-by-syntax-specimens-restated.md)
- Type: retrospective audit with reruns; no model call

## Decision

Asked by the owner: is the project's record of reductions and solver use real? Answer per claim, each
checked by reading the code that produced it and, where possible, by running it again.

| Claim | Finding | How checked |
| --- | --- | --- |
| Z3 was used and gave the recorded verdicts | **Real, reproduces.** Seven retained SMT queries replayed tonight: both `decision-equivalence` queries that were accepted are `unsat`, all three mutant queries are `sat`, the rejected first attempt is `sat`, the debt query is `unsat`. These match the records. | `z3.exe` from `.runs/e1c-facdrone/extras/bin` on `docs/experiments/{outcome-hotspot,outcome-llm,self-apply}/*.smt2` |
| E5 candidate keeps behaviour (108 tests, 50 replay sequences) | **Real, reproduces.** 108 passed, 1 skipped; replay 50 of 50 sequences, 317 operations, 0 differences in SQL. | `run_e5.py` on `.runs/e5-state-boundary/cand-e5`, tonight |
| E5 is 24.8% smaller, E6 is 23.0% smaller | **Real in lines, about a third of that in syntax.** E5: lines −22.3%, tokens −7.5%, syntax nodes −8.3%. E6: lines −18.4%, tokens −9.8%, nodes −9.9%. No candidate adds a helper file. | `lesscode-core 2.0.0 deconvolute density` per file, baseline `.runs/e2-facdrone/source` |
| E5 and E6: "0 counterexamples on bounded pure-core equivalence over [−10, 10]" | **Not supported.** The evaluators call the equivalence check only on two small functions written inside the evaluator script. No function of a candidate was ever passed to it. | `e5_evaluator.py`, `e6_evaluator.py`; search for every call site |
| E5 and E6: "100% adverse mutant kill rate" | **Not supported for the specimens.** The four controls are constructed inputs and a four-line sample in the script. No mutant of specimen code was made or run. (R75 said this of R49.) | same files |
| The reports' "PureVerifier: Bounded SMT" | **Wrong name.** The verifier enumerates combinations of listed values. It contains no SMT solver. | `native/src/pure_verifier.rs`; the Python module is deleted and was not read |
| Port `native-deconvolution-closure`: "GO, all requirements covered" (9 November) | **False for requirements 3 and 4, unusable for 1, 2 and 6.** Details below. | code, history, callers |
| Language gates R50 to R53: "4 of 4 controls killed" | **Hand-written for Rust, probes for TypeScript and Java.** Already established in R75; not repeated here. | R75 |

## The port, requirement by requirement

All seven tasks were committed between 17:13 and 17:33 on 2026-11-09 (1,231 lines, nine files).

| Requirement | Module today | Reached by a command? | Finding |
| --- | --- | --- | --- |
| 1 State boundary | `state_frame.rs`, 146 lines | no | Compares two maps handed in by the caller. Nothing observes a running program. |
| 2 Whole-scope compression | `density.rs` is gone; `state_contract.rs`, 86 lines, has no caller and no test | no (the later `deconvolute density` is separate code) | Not delivered as specified. |
| 3 Bounded comparison | `pure_verifier.rs`, 100 lines | no | 3.2 asks to "execute baseline and candidate with their pinned language runtime". The code compares two closures. Until `8e0e2b9` (11 November) a second function returned `Verified` without comparing anything and reported a difference only if the candidate's name contained "divergent". |
| 4 Streams and mutants | `stream_monitor.rs`, 150 lines; `mutant_generator.rs`, 83 lines | no | Until `8e0e2b9` the generator reported every control as killed ("default mock execution"). What is left takes an evaluator from the caller. |
| 5 Specimen runner | `specimen_runner.rs`, 158 lines | yes, its helpers | Hashing and workspace helpers are used by the driver and the step flow. Whether 5.1 to 5.5 are met was not checked. |
| 6 Token guard, governance | `token_guard.rs`, 151 lines | no | Has tests and no caller. |

The gate script of the port (`.kiro/specs/native-deconvolution-closure/run_gate.py`) compares two
identical functions written in the script, reports `"cases_evaluated": 21 + 8` as a literal, and
carries E5's metrics as literals.

The Python modules the port was to replace (`src/lesscode_baseline/state_boundary/`) were deleted the
same day (`4f0aed4`). The E5 and E6 evaluators import them and can no longer run. So the method that
produced the state-boundary results is today neither in Python nor in Rust.

## What stands

- Regeneration by a model produced candidates that pass the specimen's tests and replay, with about
  8% to 10% less syntax and 18% to 22% fewer lines on two subsystems. Functions and call edges fell by
  about 28% to 38% by the earlier measure; that measure's module is deleted and was not rerun.
- Z3 was used in the first week (CEGIS, the outcome classifier, the debt check), from Python, and its
  verdicts reproduce.
- The released tool (planted faults against a project's own tests, the step flow, receipts) is real
  and tested; R75 reran the specimen controls through it on 11 November.

## What does not stand

- Any statement that a candidate's pure core was checked for equivalence, by SMT or by enumeration.
- Any statement that the tests of E5 or E6 were shown to notice planted faults in the candidates.
- That the state-boundary method was ported to Rust.

## Limits

- E6's tests were not rerun. E3 and E4 were not audited. The TypeScript, Java and Rust adapter specs
  were not audited beyond R75. Who or what wrote the port is not recorded in the commits.
- Functions, call edges and write sites of E5 and E6 were not re-measured.
- Line counts tonight are physical lines of the scope files; R42 and R45 counted canonical lines after
  formatting, which is why 22.3% and 18.4% differ from 24.8% and 23.0%.

## Evidence

`D:\BuildArtifacts\lesscode\audit-2026-11-17\` (density receipts per file for E5 and E6). The Z3 replay
and the test run are commands in this entry's method and can be repeated; their output was not saved.

## Tool/spec consequence and next gate

- `native-deconvolution-closure` and `state-boundary-deconvolution` say `completed` in their
  `spec.json`. That is for the owner to change; this entry does not change them.
- The brief `state-consolidation` must not build on "the Rust verifier". It can build on the two
  candidates that pass their tests, on the released driver for fault checks, and on Z3 as a program
  that is on this machine and whose earlier queries replay.

## COBESY check

Each row says how it was checked. Rows that were read and not rerun say so. "Not supported" means no
code was found that did the thing, not that the thing would have failed.
