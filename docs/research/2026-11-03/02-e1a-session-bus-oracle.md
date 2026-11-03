# R20: the trace oracle was built and is deterministic; its mutation-score gain hit a ceiling

- Report date/time and timezone: 2026-11-03, Europe/Berlin
- Execution interval: 2026-11-03 08:31 to 09:34 Europe/Berlin (receipt timestamps);
  Arm B of cargo-mutants was interrupted by the owner at 09:34 and is partial
- Status: completed for steps 1 to 7 and Arm A; Arm B partial; decision pending owner
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: H8(a), [R19](01-dynamic-deconvolution-direction.md)
- Type: preregistered, [E1a plan](../../experiments/deconvolution/e1-mmhmem-session-bus/plan.md)

## Decision

Oracle feasibility on a real stateful Rust scope is established: a recorded,
normalised trace of SQLite state and audit lines replays deterministically and
catches every semantic mutant tried. The preregistered gain prediction P2
cannot hold because the seed tests alone already kill 96% of mutants, so the
kill rule as written fires. The owner decides whether E1a counts as
feasibility-positive with a miscalibrated gain prediction, or as a stop. The
reviewing agent recommends the former and a baseline-aware prediction for E1b;
it does not approve that itself.

## Owner disposition, 2026-11-03

After the session restart the owner replied "continue" to the decision
request above. The reviewing agent reads this as proceeding on the recommended
path: E1a is feasibility-positive, the P2 gain prediction is recorded as
miscalibrated rather than as oracle failure, Arm B is completed for the record,
E2 runs, and E1b must state its gain prediction relative to a measured seed
baseline. This interpretation is recorded here so the owner can correct it.

## What ran

Specimen mmhmem frozen at `8a9ce348` by `git archive` (HEAD had drifted one
commit touching only CI files; recorded in the plan amendment). Isolated copy
under `.runs/e1-mmhmem/original`; the checkout was not modified. Harness
`tests/e1_trace_oracle.rs`, 429 research lines; proptest 1.12.0 added as a
dev-dependency only; `Cargo.toml`/`Cargo.lock` diffs retained.

Golden trace recorded once (786 KB JSON, hash retained), replayed four times
with identical results. Seed scenarios: 20 distinct operation signatures.
Generated sequences: 50 distinct signatures, 37 of them never produced by the
seed tests.

## Results against the preregistered predictions

| Prediction | Result | Held |
| --- | --- | --- |
| P1: seed tests alone kill 50 to 75% | 80 caught, 3 missed, 4 unviable, 1 timeout of 88; kill rate 96.4% | No, above range |
| P2: trace arm at least 85% and at least 20 points above | Ceiling: Arm A is already 96.4%; Arm B partial (1 caught, 3 unviable) when interrupted | Cannot hold |
| P3: trace arm catches M1 to M3; seed arm misses at least M1 | Both arms caught all three hand-written mutants | Half: catches yes, seed-miss no |
| P4: trace arm region coverage of `session_bus.rs` at least 90% | 90.58% (Arm A 89.94%) | Yes |
| P5: generated sequences observe behaviour no seed test does | 37 generated-only signatures, including a panic | Yes |

Arm B rerun, 2026-11-03 after the restart: 70 caught, 0 missed, 3 unviable
of 88 before the host ran low on memory and the run was stopped; 15 mutants
unrun, among them the three `now_ms` constants. The interrupted first attempt
is preserved separately. No Arm B mutant has been missed so far.

Arm B completion by owner decision, 2026-11-03 10:48 to 10:52: only the three
`now_ms` mutants were rerun under the Arm B configuration (name filter,
`-j 2`). **All three were caught.** Across the two Arm B runs, 73 of 73
executed viable mutants are caught, 3 are unviable, and 12 of 88 remain
unrun. The expectation stated below that time masking would hide these
mutants was wrong: a constant clock collapses task identities and ordering,
which the first-seen ordinal normalisation still exposes. Against Arm A's
96.4% the measured gain is 3.6 points on the executed set, so P2's 20-point
clause still fails, but the oracle caught precisely the mutants the seed
tests missed.

The three mutants the seed arm missed all replace `now_ms` with a constant.
The oracle normalises time fields by design, so Arm B was expected to miss
them too; the completion run above shows it did not.

## A real defect surfaced by the generated inputs

`checkpoint` with a working state whose 6,000-byte cap falls inside a
multi-byte character panics: `end byte index 6000 is not a char boundary`
(`truncate_to_budget`, `session_bus.rs:355`). No seed test reaches it. This
is a finding about mmhmem, retained in `results/p5-novel-behaviours.json`; it
is not a compression result and was not fixed, since the specimen is frozen.
A second generated-only outcome, `resume` failing with Windows "Access is
denied", needs inspection before it is attributed to the code rather than the
harness's temp-directory handling.

## Limits

Low-scatter scope by the survey's own counts; this does not test oracle
feasibility on scattered state. Mutation score measures sensitivity to
cargo-mutants' syntactic operators plus three hand-written mutants. Time fields
are masked, so time-dependent logic is unprotected by construction. Arm B is
partial. No candidate, no compression, no equivalence claim.

## Next gate

Owner decision on the E1a disposition. If feasibility-positive: preregister
E1b on Facdrone R15-A with the gain prediction stated relative to a measured
seed baseline rather than an assumed band, and run E2 (recorder built and
smoke-tested, full tier staged; see its [plan](../../experiments/deconvolution/e2-facdrone-edge-census/plan.md)).
If stop: close the retrospective track with a dated entry.

Evidence: [experiment directory](../../experiments/deconvolution/e1-mmhmem-session-bus/),
receipt, golden trace, mutant patches, cargo-mutants output for both arms,
coverage summaries.

## COBESY check

Outcome first; the failed prediction is stated as failed; the ceiling is
explained, not excused; the defect is reported as a specimen finding; the
decision is left to the owner.
