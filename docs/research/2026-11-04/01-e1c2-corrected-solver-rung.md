# R24: the corrected solver rung kills 7 of 10 targets but misses its own bar; the guided-generation claim is rejected for database-entangled predicates on this scope

- Report date/time and timezone: 2026-11-04, Europe/Berlin
- Execution interval: 2026-11-04 07:25 to about 08:55 Europe/Berlin (receipt, UTC+1)
- Status: completed; S1 to S4 failed, S5 held; kill criterion fired; solver session valid (both verifier controls passed)
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: guided-generation claim in [R19](../2026-11-03/01-dynamic-deconvolution-direction.md); corrected rerun of [R23](../2026-11-03/05-e1c-concolic-rung.md)
- Type: preregistered, [E1c-2 plan with amendments 1 to 4](../../experiments/deconvolution/e1c2-concolic-rerun/plan.md); apparatus and per-target detail in the [E1c-2 README](../../experiments/deconvolution/e1c2-concolic-rerun/README.md)

## Decision

As preregistered, the kill criterion fired: S1 and S2 failed while S5 held.
The guided-generation claim, that solver-chosen inputs over lifted predicates
resolve the oracle's residue, is rejected for database-entangled predicates
on the Facdrone evidence-publication scope. This time the rejection stands on
a verified pipeline: every driver provably entered the lifted code, the
lifted predicates reproduce the originals on all 472 recorded calls, the
known-killable control produced a counterexample and the known-equivalent
control exhausted its paths. Nothing in the run can be blamed on the
encoding, which is what R23 could not say.

The result is not empty. Seven of the ten targets are now dead. Four died the
moment the oracle called the reader functions it had never called before,
which confirms R23's first finding: part of the residue was an alphabet gap,
not an input-selection problem. Three died only from realised solver inputs,
and all three are identity or ordering mutants that no reader call exposes.
The remaining three need several conditions to hold at once and the solver
did not find them in the budget; no target was proved equivalent, because
within these bounds the path count never exhausts.

What this permits next: E3 may be preregistered on this scope with the
reader-inclusive alphabet as acceptance and the residue (166, 169, 200, plus
whatever the oracle cannot reach) declared unprotected. It does not permit
calling the solver a verifier of equivalence for this kind of code. The
owner decides between E3 and stopping the retrospective track.

## Results against the preregistered predictions

| Prediction | Result | Held |
| --- | --- | --- |
| S1: counterexample or exhausted no-difference verdict for at least 7 of 10 | 4 of 10: counterexamples for 240, 241, 418, 584; 150, 166, 169, 186, 200, 642 ended "no differences found" without exhaustion (inconclusive, never equivalence) | No |
| S2: at least 6 of 8 covered survivors killed | 5 of 8: 150, 186, 418, 642 by the extended alphabet alone, 584 by a realised solver input; 166, 169, 200 survived | No, by one |
| S3: at least 1 proved equivalent within bounds, and 642 proved or killed | 0 proved; 642 killed (its `LIMIT` parameter is recorded) | No, first clause |
| S4: `state_repository.py` branch coverage at least 84% | 45/56 = 80.36% to 47/56 = 83.93% | No, by 0.07 points |
| S5: every driver smoke enters `lifted.py`, no void runs | 16 of 16 smokes enter, 14 to 255 entries each | Yes |

Per target, with Arm B running the new sequences only:

| Classification | Targets | How |
| --- | --- | --- |
| Killed by realised solver input | 240, 241, 584 | identity (`is`) and ordering (`<=`) mutants; the oracle sees a resting order present on one side only (240, 241) or an intention list of length 1 versus 0 (584) |
| Killed by the extended alphabet alone | 150, 186, 418, 642 | the oracle records SQL text and parameters, so a mutated predicate or `LIMIT` shows in the first reader call; 418 also has a solver counterexample, but the replay stops at the first divergence |
| Unresolved | 166, 169, 200 | each needs a joint condition (a matched attempt row, a history entry and one instruction identity; or two legacy-matched rests) that the solver did not reach in 300 s scaled |
| Proved equivalent | none | exhaustion is out of reach for every target driver within the bounds; only the deliberately tiny control exhausted |

Counterexamples are verbatim in `results/logs/ch-*.log`; the realisation
mapping from solver values to database rows is in the README.

## Verifier (owner amendment, 2026-11-04)

| Check | Result |
| --- | --- |
| 1 Symbolic-ness: more than one explored path into `lifted.py` per driver | holds for all 16 smokes |
| 2 Encoding soundness: lifted equals original on every recorded input | `lifted.py` unchanged from E1c (sha256 `1106c384…`); rerun gives 472 calls, 285 distinct inputs, 0 mismatches |
| 3a Control: known-killable 418 yields a counterexample | holds |
| 3b Control: annotation-only mutant (`decision_evidence_publication.py:191`, `-> str \| None` to `-> str - None`) yields "no differences found" with "all paths exhausted" | holds, 4 iterations; two earlier attempts failed to load the module because of import side effects and ran no path (amendment 4) |
| 4 Realisability: every counterexample differs on concrete replay | holds for all four |

The session is therefore not void, and the six inconclusive runs are
recorded as inconclusive.

## Frozen inputs, method and budget

Specimen, frozen copy, mutant copy, E1b oracle and golden traces exactly as
in E1c; CrossHair 0.0.112 with z3 5.3.3.0; CPython 3.13.16. Drivers
rewritten with bounds imposed by construction: strings from a three-letter
alphabet of symbolic length at most 8 with no slicing, lists of at most 3,
enumerations as integer indexes, `kind` and `representation` decoded through
`json.loads` so they are never the interned literal (CrossHair symbolises
through its pure-Python JSON library; the concatenation fallback was not
needed). Oracle alphabet extended by one `readers` operation calling
`opening_tickets`, `resting_orders`, `attempt_history` and `decision_served`;
`golden-e1c2.json` holds 11 sequences and 17 operations, body sha256
`034f0f31…dabb`, replays on the frozen copy with no divergence.

Adaptive budgets (owner amendment, 2026-11-04) applied from the third solver
target on: each heavy step probed the host and scaled its base deadline by a
measured factor between 1.10 and 1.30; no timeout occurred under load, so
nothing was rerun. The first two solver runs used the fixed 300 s; a probe
during the second showed factor 1.16, so they were kept.

| Budget | Preregistered | Used |
| --- | ---: | ---: |
| Solver time | 5,400 s | 3,857.7 s |
| Per-target solver deadline | 300 s base | 300 s fixed (2 runs), 330 to 390 s scaled (8 runs) |
| Research code | 250 lines | 298 lines (225 in the preregistered scope; 73 for the two owner amendments) |

Native Windows, plain user account; no WSL and no bare `bash` from scripts.
Frozen copy and mutant copy report no changes after the last restore;
`D:\Source\facdrone` HEAD `d49f48bd` with a clean status.

## Limits and interpretation

- Inconclusive is not equivalence. The bounds that make the drivers honest
  also make the path space unexhaustible, so this method proves nothing
  about equivalence on predicates of this shape; it only finds differences.
- Four kills are representation-level: the oracle compares SQL text and
  parameters, so 642, semantically equivalent by inspection, dies on a
  changed `LIMIT`. That is the oracle's design, not evidence about predicate
  semantics, and a regeneration that changes query text while preserving
  results would be flagged the same way.
- The identity kills (240, 584) depend on strings arriving through JSON or
  the database; a caller passing a Python literal would not expose them.
- SQL is modelled in Python inside the lifted predicates (code-point order,
  no collation), as in E1c.
- Post hoc only: counting the seven E1c-2 kills against the E1b sample would
  put Arm B near 51 of 59 (about 0.86), at the Q1 threshold that R22 missed.
  This mixes two alphabets and was not preregistered; it is recorded as a
  pointer for E3's sensitivity declaration, not as a result.
- One scope, ten targets, one solver, one host.

## Tool/spec consequence and next gate

The retrospective track now has a measured protection level for the
evidence-publication scope: seed tests plus trace replay with the
reader-inclusive alphabet, plus three realised solver inputs, with 166, 169
and 200 and every unreached function declared unprotected. The solver's
place is narrowed to what the controls justify: a differential
counterexample finder on lifted pure predicates, gated by the four verifier
checks, never an equivalence authority. This is consistent with the
placement rule in [symbolic-execution-tooling.md](../../symbolic-execution-tooling.md).

Options for the owner, unchanged from R23 but now with evidence:

1. Preregister E3 (regeneration behind the public API) on this scope with
   the pure-core / effect-shell split required, the E1c-2 alphabet as
   acceptance, CrossHair differential with the verifier on the pure core,
   and the declared residue. Predictions must be about edges and behaviour,
   not about the solver.
2. Stop the retrospective track here and move to prevention at generation
   time, carrying the oracle and verifier as the acceptance instruments.

Evidence: [experiment directory](../../experiments/deconvolution/e1c2-concolic-rerun/),
`README.md`, `plan.md`, `results/receipt.json`, `results/logs/`,
`results/golden-e1c2.json`.

## COBESY check

Outcome first; the kill criterion is named and fired; failed predictions are
reported as failed, including the two near misses; the verifier makes the
negative result safe to report because the pipeline, not the encoding, is
what failed to resolve the residue; facts (classification, counts, hashes)
are separated from interpretation (alphabet gap, representation-level kills,
post hoc rate); the decision is the owner's.
