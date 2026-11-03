# R22: the Facdrone oracle is deterministic and sees stored representation, but adds little kill rate; H8(a) rejected as formulated for the high-scatter scope

- Report date/time and timezone: 2026-11-03, Europe/Berlin
- Execution interval: 2026-11-03 11:40 to about 18:30 Europe/Berlin, 5.51 h
  wall clock (receipt timestamps)
- Status: completed; Q1, Q3 and Q4 failed, Q2 and Q5 held; kill criterion fired
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: H8(a), [R19](01-dynamic-deconvolution-direction.md); after [R20](02-e1a-session-bus-oracle.md) and [R21](03-e2-facdrone-edge-census.md)
- Type: preregistered, [E1b plan with amendments 1 to 7](../../experiments/deconvolution/e1b-facdrone-evidence-publication/plan.md)

## Decision

Under the plan's kill criteria, Q1 failing rejects H8(a) for the high-scatter
class as formulated: a recorded trace oracle driven by seed tests plus random
generation does not close half the gap the seed tests leave. What the run
establishes positively: the oracle for an eight-module, transactional,
multi-representation Python scope is byte-deterministic across separate
database servers, catches all three hand-written semantic mutants, and is the
only arm that sees a change of stored representation. What limits it is the
executions that drive it, not its observation power. The owner decides
whether E3 proceeds with the measured protection level, whether a
constraint-guided generation rung is tried first, or whether the track stops.

## Results against the preregistered predictions

Sample: 60 mutants drawn once (seed 20261103) from 805 cosmic-ray mutants in
the two mutated modules; per-line test selection (amendment 4); one void
mutant (syntax error) excluded from both arms.

| Prediction | Result | Held |
| --- | --- | --- |
| Q1: k_B at least k_A + 0.5 × (1 − k_A) | k_A = 42/59 = 0.712, required 0.856, k_B = 44/59 = 0.746 | No |
| Q2: Arm B catches M1, M2, M3 | all three caught; Arm A misses M3 | Yes |
| Q3: Arm B branch coverage at least 85% and 5 points above A | 95.36% vs 95.23% statements plus branches; 88.62% vs 88.32% branch-only | Half: level yes, gain no |
| Q4: at least three generated-only behaviours, one on a failure path | two, both on failure paths (`commit: EventConflictError`, `sim_persist: ValueError`) | No |
| Q5: two recordings agree on every digest without masking | byte-identical, sha256 `92a48f77…`, 51 sequences, 290 operations | Yes |

| Arm | Killed | Survived | Uncovered among survived | Void | Kill rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| A: seed tests | 42 | 17 | 8 | 1 | 0.712 |
| B: seed tests + trace replay | 44 | 15 | 8 | 1 | 0.746 |

Arm B killed two mutants Arm A missed, both on line 299 of the publication
module; Arm A killed nothing B missed. Of the eight uncovered mutants, five
change `|` in type annotations under `from __future__ import annotations`
and are probably equivalent; two sit on a legacy branch no seed test reaches,
a real gap; one is the void.

Hand-written mutants: M1 (ledger adopted before commit) and M2 (reference
validation skipped on the legacy route) caught by both arms. M3 (compression
level changed, stored bytes differ, decoding unchanged) missed by the tests
and caught by the replay. That is the oracle doing the one job tests cannot:
protecting stored representation.

## Interpretation

Per-line selection means Arm A already runs every seed test that touches a
mutated line. The replay re-observes largely the same executions, because its
sequences were transcribed from those tests plus random Hypothesis
generation, with a finer lens: table dumps, SQL, stored bytes. The gain is
therefore the finer lens (M3, line 299), not new paths. Random generation
found two new behaviours in 40 sequences. This is the result the owner
predicted from SAGE: blind generation does not reach new paths; constraint-
guided generation is the missing rung, documented in
[symbolic-execution tooling](../../symbolic-execution-tooling.md).

The gap-halving prediction was the wrong shape for this mechanism. A trace
oracle's reach is bounded by the executions that drive it; its contribution
is observation depth. A future prediction should separate the two: kill rate
on mutants whose line is executed by the recorded sequences, and the count of
new paths from generation.

## Timing and apparatus

Unmutated selected-set runs: T_A 400 s, T_B 514 s over 191 seed files.
cosmic-ray: Arm A 46.5 min, Arm B 51.0 min, no timeouts; per-mutant deadline
max(120 s, 5 × measured). Harness 811 research lines against the 820 budget
(700 originally); counting script 65 lines outside it. Seeds confirmed by
coverage: 191 files, up from the 115 static candidates.

## Baseline and unprotected behaviour

236 tests fail on the unmodified copy on this host: 136 symlink-privilege, 22
file-lock, 19 POSIX-only, 67 other by the agent's classification (R21 used
131 for the symlink class; the difference is classification, not new
failures). Not repaired; the 10 inside seed files are skipped in both arms.
What those tests guard is unprotected on this host: mandate and daemon-profile
records built from `conf/base`, the exit-observation caches, POSIX file
locking in the research stores. The owner declined the host privilege change
and ruled that the desired implementation must not assume it either.

## Process incident

Two copy commands issued as `bash -c cp …` from Python executed in WSL,
because Windows resolves a bare `bash` to the System32 WSL launcher before
PATH. Both failed at the copy and changed nothing; the copies were redone with
native Python; the incident is in the receipt and README. The rule is
unchanged; the trap is now recorded in project memory.

## Limits

Mutation score covers cosmic-ray's syntactic operators on two of eight files.
Uncovered mutants are counted as survived by rule. Effects outside the
intercepted boundary are not captured; the fail-closed property is not
enforced. Generated sequences are random, not constraint-guided. No
compression, no candidate, no equivalence claim.

## Owner decision, 2026-11-03

From the three options below the owner chose the constraint-guided rung
first. [E1c](../../experiments/deconvolution/e1c-concolic-rung/plan.md) is
preregistered against the ten surviving mutants: lift the database-entangled
predicates to pure functions, solve for distinguishing inputs with CrossHair,
realise them as database rows in the oracle, and classify each target as
killed, proved equivalent within bounds, or unresolved.

## Tool/spec consequence and next gate

H8(a) rejected as formulated for this class; the oracle mechanics are
positive on both languages (R20, R22). H8(b) was rejected in R21. H8(c),
regeneration, is untested and under the preregistered gates not admitted.
Options for the owner: (1) preregister a constraint-guided rung, CrossHair
over the publication module's pure functions with state seeds, predicting new
paths and kill-rate gain on covered survivors; (2) preregister E3 on this
scope with the measured protection level of 0.746 declared as the oracle's
sensitivity, adding CrossHair differential checks on the pure core of the
candidate; (3) stop the retrospective track. The prototype gate remains
`revise`.

Evidence: [experiment directory](../../experiments/deconvolution/e1b-facdrone-evidence-publication/),
`README.md`, `results/receipt.json`, `results/cr-counts.json`,
`results/selection-map.json`, `results/golden-trace.json.sha256`, raw logs.

## COBESY check

Outcome first; the kill criterion that fired is named; the positive mechanics
are separated from the failed gain prediction; the reason for the small gain
is drawn from the selection method, not excused; the process breach is
disclosed; the decision is the owner's.
