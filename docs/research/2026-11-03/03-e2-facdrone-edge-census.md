# R21: the edge census finds the hard case but does not separate the easy one; the selector is rejected as formulated

- Report date/time and timezone: 2026-11-03, Europe/Berlin
- Execution interval: 2026-11-03 10:53 to 12:20 Europe/Berlin; run1 all eleven
  batches, unrecorded baseline for two batches, run2 repeat on root and
  reporting (receipt timestamps)
- Status: completed; H8(b) rejected as formulated
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: H8(b), [R19](01-dynamic-deconvolution-direction.md); after [R20](02-e1a-session-bus-oracle.md)
- Type: preregistered, [E2 plan with amendments 1 to 7](../../experiments/deconvolution/e2-facdrone-edge-census/plan.md)

## Decision

Two of five calibration predictions failed, including C2, which the plan named
as a kill criterion. The edge measures as preregistered may not be used to
select the E3 rewrite scope. The census itself is valid and repository-wide:
94.7% of production functions were reached, the recorder provably did not
alter test outcomes, and the human-selected hard case does stand out on two
measures. A corrected measure set is a new preregistration, not a rescue of
this one. E1b on the evidence-publication scope proceeds on its own evidence.

## Results against the preregistered predictions

Population: 2,275 entry responsibilities (dynamic call closures of
test-invoked production callables) over 720 production files and 9,215
callables; 8,405 tests executed, 236 failed or errored.

| Prediction | Result | Held |
| --- | --- | --- |
| C1: case A top decile on write sites and churn | write sites 3 (3 greater), churn 269 (63 greater); top decile threshold is fewer than 228 greater | Yes |
| C2: case C below median on both | C write sites 1 vs median 0; churn 0 vs median 0 | No |
| C3: case B above median on churn | 1,332,416 vs median 0 | Yes |
| C4: at least three unexamined responsibilities in top decile on two measures | 310 | Yes |
| C5: rank correlation with physical lines below 0.7 for each measure | call edges 0.89, degree of scattering 0.71; module hops 0.41, write sites 0.42, churn 0.41, ordering 0.37 | No |

Declared file sets, for comparison with R15 to R18:

| Case | Call edges | Module hops max | Write sites | Churn max | Scattering | Ordering pairs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A evidence publication | 194 | 4 | 3 | 269 | 0.84 | 174 |
| B report envelope | 178 | 3 | 1 | 1,332,416 | 0.79 | 0 |
| C provider orchestration | 97 | 0 | 1 | 0 | 0.69 | 0 |

## Why the predictions failed, from the data

- C2 failed on a population artefact, but failed as preregistered. Most of the
  2,275 closures are tiny and score zero on write sites and churn, so the
  median is zero and "below median" is unreachable. The unit of measurement,
  every test-entered callable, produces a long tail of trivial closures.
- C5 failed because call-edge count and degree of scattering track size.
  Module hops, write sites, churn and ordering carry information that lines
  do not; two of six measures are redundant with size.
- The churn measure is broken as defined: it counts codec calls per
  invocation, so serialisation inside loops produces values in the millions
  (case B, the XLSX renderer, the artifact publisher). Distinct codec call
  sites were the intended quantity.
- The top twenty by these measures are the large orchestrators: the alpha
  runner, the decision route, the simulation driver, the report builders,
  each a closure of 40 to 204 files. The measures reward size, not
  convolution, and no such closure is a candidate for regeneration.

## Validity checks

- Recorder validity: the unrecorded baseline of the two batches with most
  failures reproduces the recorded runs exactly. Root unit: 181 failing or
  erroring test IDs in baseline, run1 and run2, symmetric difference zero.
  Reporting: 31 in all three, symmetric difference zero. The failures belong
  to the specimen on this host, not to the recorder.
- Baseline failures: 131 Windows symlink-privilege errors, 22 Windows
  file-lock errors, 21 artifact-publisher staged-bundle write failures (seen
  in the earlier Facdrone CSV trial, cause undiagnosed), a POSIX-only mypy
  contract, two golden render drifts. Recorded, not repaired.
- Repeat determinism: run2 covered root and reporting (amendment 6). Of 1,708
  responsibilities in that scope, 74 (4.3%) were attributed in run1 but not
  in run2, with closures up to 237 callables and one top-twenty entry among
  them. Entry attribution depends on test-to-worker distribution and fixture
  caching. Calibration cases are declared file sets and are unaffected; the
  per-entry ranking is partly run-order dependent. The 247 remaining
  differences are suites not repeated, by design.
- Coverage of the effect boundary is thinner than on Linux: the artifact
  publisher's file writes fail on this host, so file sinks are under-observed.

## Specimen findings, not fixed

Two Windows defects in Facdrone's test support, worked around in the research
shim without touching the specimen: `pg_ctl` receives a quoted empty
`listen_addresses` literally, and the started server inherits captured pipes
so the start call never returns. With both corrected, Facdrone's embedded
PostgreSQL runs natively on Windows and the database-backed tests pass (B1:
25 passed in 1.8 s). Recorded in plan amendments 5 and 5a.

## Limits

Dynamic edges exist only for executed code; 5.3% of production functions were
not reached and are labelled static-only. Effects routed through pandas,
pyarrow or kedro on Facdrone's behalf are not counted as sinks. Database-
backed tests ran, but few SQL paths are exercised by the hermetic tier. No
compression, no candidate, no equivalence claim.

## Tool/spec consequence and next gate

H8(b) is rejected as formulated. A corrected census, if attempted, must be
preregistered separately with: distinct codec sites instead of calls,
measures normalised by executed size, a minimum executed size for the
population, and entry attribution that is independent of worker
distribution. The prototype gate remains `revise`. E1b proceeds on the
evidence-publication scope selected by R15 and confirmed here on C1.

Evidence: [experiment directory](../../experiments/deconvolution/e2-facdrone-edge-census/),
`results/calibration.md`, `results/edges.json`, `results/receipt.json`,
`results/recorder-validity-diff.txt`, per-batch junit files and failure triage.

## COBESY check

Outcome first; the kill criterion that fired is named; the reasons are drawn
from the data, not excused; the recorder's innocence is shown by diff, not
asserted; what remains valid is separated from what is rejected.
