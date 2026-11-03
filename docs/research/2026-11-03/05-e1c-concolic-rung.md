# R23: the solver rung failed as run; the failure is in the drivers and in what the oracle calls, not in the solver

- Report date/time and timezone: 2026-11-03, Europe/Berlin
- Execution interval: 2026-11-03 about 18:45 to 20:25 Europe/Berlin (receipt)
- Status: completed; R1 to R4 failed, R5 held; kill criterion fired
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: guided-generation claim in [R19](01-dynamic-deconvolution-direction.md); after [R22](04-e1b-evidence-publication-oracle.md)
- Type: preregistered, [E1c plan with amendments 1 to 4](../../experiments/deconvolution/e1c-concolic-rung/plan.md)

## Decision

As preregistered, E1c is rejected: one of ten targets resolved by the solver,
one of eight covered survivors killed by a realised solver input, none
proved equivalent. The record attributes most non-results to the research
drivers, not to CrossHair: a typing defect in the first encoding voided four
runs, and the second encoding cut strings to a fixed length, which made the
solver treat them as concrete and degrade to sampling. A diagnostic probe
without that cut found a counterexample for target 241 in under 30 seconds.
The one properly encoded target, 418, was killed by its realised input. A
corrected rerun is a new bounded attempt and needs the owner's decision.

## Results against the preregistered predictions

| Prediction | Result | Held |
| --- | --- | --- |
| R1: counterexample or equivalence verdict for at least 7 of 10 | 1 of 10 (line 418); nine runs ended "no differences found" without path exhaustion, which is not an equivalence verdict | No |
| R2: at least 5 of 8 covered survivors killed by realised inputs | 1 of 8 (418); 2 of 8 with a supplementary probe sequence, excluded by rule | No |
| R3: at least 1 covered survivor proved equivalent | 0; line 642 is equivalent by inspection (`limit(1)` to `limit(2)` before `.first()` on a primary key) but not proved | No |
| R4: `state_repository.py` branch coverage up at least 2 points over E1b Arm B | 80.36% to 80.36% with the preregistered input; 83.93% with the supplementary sequence | No |
| R5: lifted predicates match the originals on every recorded input | 472 of 472 captured calls, 285 distinct inputs, 0 mismatches | Yes |

Per target: 418 killed by solver input; 186 and 241 killed only by the
supplementary sequence; 150, 166, 169, 200, 240 unresolved because the driver
bounds concretised strings; 584 unresolved because interned literals make
`is` and `==` coincide on the driver's domain; 642 unresolved by the solver,
equivalent by inspection.

## Two findings that outweigh the verdict

1. **The oracle never called the readers.** The E1b golden corpus reaches
   none of the ten target functions (R5 was vacuous on it and had to use
   seed-test captures). Four of the eight covered survivors mutate SQL
   predicates or parameters, and the oracle records every SQL statement, so
   any sequence that merely calls `opening_tickets`, `resting_orders`,
   `attempt_history` or `decision_served` exposes them. Target 186 died
   exactly that way. Part of E1b's residue is therefore about which functions
   the oracle drives, not which inputs it chooses. That is a gap in the
   oracle's operation alphabet, cheap to close.
2. **The solver works when the encoding is honest.** 418 was killed by a
   realised counterexample; 241's counterexample appeared in seconds once
   strings were left symbolic. The failures came from `x[:12]` string cuts,
   from `typing.Literal` inside containers (a CrossHair 0.0.112 defect,
   worked around with integer indexes), and from drivers that rejected
   out-of-bound inputs so no path entered the lifted code.

## Apparatus and budget

CrossHair 0.0.112. Solver time 3,411 s recorded, about 3,610 s with one
stopped run; the 1 h budget was reached. Research code 535 lines against the
400 budget, 135 over, mostly the R5 capture and check that the empty golden
corpus made necessary. PostgreSQL rejects NUL characters, so solver strings
were mapped to printable ones preserving every equality and ordering the
predicates use; the mapping is in the README. New golden file
`golden-e1c.json`, two sequences, replays without divergence. Frozen copy,
mutant copies and specimen unchanged; no WSL or bare `bash` was invoked.

## Limits

One scope, ten targets, one solver, one hour. "No differences found" without
exhaustion is not evidence of equivalence. The supplementary sequence is a
diagnostic, not a preregistered result.

## Tool/spec consequence and next gate

The guided-generation claim is neither supported nor refuted by this run; it
is untested as intended because the drivers did not present symbolic
inputs. Options for the owner: (1) a corrected bounded rerun, E1c-2, with
symbolic strings, integer-indexed literals, drivers that constrain by
construction, 300 s per target, and the oracle's alphabet extended to call
the four reader functions, predicting at least six of eight covered survivors
killed; (2) stop here and carry the residue into E3 as declared unprotected
behaviour; (3) stop the retrospective track. Gate remains `revise`.

Evidence: [experiment directory](../../experiments/deconvolution/e1c-concolic-rung/),
`README.md`, `results/receipt.json`, `results/logs/`.

## COBESY check

Outcome first; the kill criterion is named; driver defects are attributed to
the research apparatus, not to the tool or the specimen; the two findings
that change the next design are separated from the failed predictions; the
decision is the owner's.
