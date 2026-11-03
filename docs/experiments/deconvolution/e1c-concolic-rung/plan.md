# E1c plan: constraint-guided inputs against the mutants the oracle could not kill

- Written: 2026-11-03, Europe/Berlin, after R22, before any E1c code or execution.
- Owner decision, 2026-11-03: run the constraint-guided rung before any
  regeneration experiment.
- Hypothesis: the guided-generation claim in [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md)
  and [docs/symbolic-execution-tooling.md](../../../symbolic-execution-tooling.md):
  blind generation does not reach new paths; solver-chosen inputs do.
- Type: preregistered. Predictions fixed below; amendments dated.
- Runtime: native Windows, plain user account, no WSL. Never call a bare
  `bash` from scripts; use native Python or the full Git Bash path.

## Question

Can solver-chosen inputs kill the mutants that 191 seed test files, 40 random
Hypothesis sequences and the trace replay all left alive, and can they prove
some of those mutants equivalent? This targets the exact residue of
[E1b](../e1b-facdrone-evidence-publication/plan.md): 8 covered survivors and
2 uncovered legacy-branch survivors.

## Frozen inputs

- Specimen and copy as in E1b: Facdrone `d49f48bd…`, frozen copy
  `.runs/e2-facdrone/source`, never modified. Mutants are applied only to
  separate copies, as E1b did.
- Oracle: the E1b harness (`e1b_oracle.py`, golden trace sha256 `92a48f77…`),
  unchanged except for an input source added as a separate module.
- Targets, from `e1b-facdrone-evidence-publication/results/cr-counts.json`:

| File | Line | Operator | E1b status |
| --- | ---: | --- | --- |
| state_repository.py | 150 | Eq to LtE | covered, survived both arms |
| state_repository.py | 166 | And to Or | covered, survived |
| state_repository.py | 169 | Eq to LtE | covered, survived |
| state_repository.py | 186 | Eq to IsNot | covered, survived |
| state_repository.py | 200 | Eq to GtE | covered, survived |
| state_repository.py | 418 | Eq to GtE | covered, survived |
| state_repository.py | 584 | Eq to Is | covered, survived |
| state_repository.py | 642 | NumberReplacer | covered, survived |
| state_repository.py | 240, 241 | Eq to Is, Eq to LtE | uncovered legacy branch (`_placed_under`) |

The five annotation-only mutants (`|` in type unions under
`from __future__ import annotations`) and the void mutant are excluded as
equivalent or invalid, with that classification recorded.

## Method

1. **Lift the predicates.** For each target, copy the enclosing pure logic into
   a research-side module as a function of plain data: `opening_tickets` and
   `resting_orders` candidate selection (bodies, history), `_placed_under`
   (rest, body), the comparison sites at 418, 584 and 642 in their enclosing
   functions. The lifted function takes already-loaded rows and history
   objects; the database loop stays outside. Each lifted function is checked
   against the original on the recorded corpus before use (it must reproduce
   the original's outputs on every recorded input), and that check is recorded.
2. **Solve for distinguishing inputs.** For each target mutant, build the
   mutated lifted function and run CrossHair `diffbehavior` between original
   and mutated lifted function, with bounded domains (short strings, small
   lists, the literal enumerations the predicates compare against). Record
   the exact command, bounds, time limit and outcome: a counterexample, a
   proof of no difference within bounds, or exhaustion.
3. **Realise the inputs.** Convert each counterexample into database rows and
   history objects in the oracle's generated-sequence source, replay the
   original to record the expected observation, then run the mutant copy
   under Arm B with the new sequences only.
4. **Classify.** Each target ends as killed by a realised solver input,
   proved equivalent within bounds, or unresolved, with the reason.

Random generation is not expanded; its result from E1b stands as the control
(two new behaviours in 40 sequences).

## Predictions, fixed before execution

- R1: CrossHair produces a counterexample or an equivalence verdict within
  bounds for at least 7 of the 10 targets.
- R2: realised counterexamples kill at least 5 of the 8 covered survivors.
- R3: at least 1 covered survivor is proved equivalent within the declared
  bounds, and inspection confirms the reason.
- R4: the realised inputs raise branch coverage of `state_repository.py`
  above the E1b Arm B figure, by at least 2 points.
- R5: lifting does not change behaviour: every lifted function matches the
  original on every recorded input in the golden corpus.

## Kill criteria

- R5 fails: the lifted predicates are not the originals; stop and report.
- R1 and R2 both fail: solver-guided inputs do not reach what random inputs
  missed on this scope; the guided-generation claim is rejected for
  database-entangled predicates, and E3 must declare that residue
  unprotected.
- CrossHair cannot handle the lifted functions' types (pydantic models in
  history): record the exact failure, lift one level further to plain dicts
  and tuples with a recorded conversion, and retry once; then stop.

## Budget

One agent session for lifting and solving, one for realisation and
measurement. Research code at most 400 lines, counted separately. CrossHair
per-target time limit 300 s; total solver time at most 1 h. Arm B reruns on
the 10 targets only, serial, one heavy job at a time.

## Not claimed

No compression, no candidate, no whole-scope equivalence. A positive E1c
raises the oracle's measured sensitivity on this scope and establishes the
pure-core requirement for E3: regeneration must separate predicates from
effect loops so the solver can reach them without lifting.

## Amendment 1 (2026-11-03, execution session, before the R5 check)

- **Finding.** The E1b golden corpus never reaches any of the ten target functions. A capture
  hook (`capture.py`, wrapping `opening_tickets`, `resting_orders`, `_placed_under`,
  `instruction_history`, `_shadow_seed` and `decision_served` in process, no file changed)
  recorded **0 calls** while replaying the golden trace `92a48f77…` (receipt `r5-capture-golden`,
  replay still "no divergence"). The trace contains no `flow = 'attempt'` or `'instruction'`
  query and no `instructions` payload. R5 on the golden corpus alone is therefore vacuous.
- **Change.** R5 is checked on the golden corpus (0 inputs, recorded as vacuous) **and** on the
  inputs the same hook captures while the E1b seed files that select the target lines run on the
  frozen copy (the union of the selection-map entries for lines 150, 162-174, 177, 186, 200, 201,
  236-243, 418, 584 and 642: 36 files, no deselected node ids among them). Each captured call
  holds the whole `live_decision_event` table in heap order, the identity, the history as
  `(kind, id, parent, target.venue_symbol)` tuples (the one recorded conversion), and the
  original's output. R5 holds only if every captured call is reproduced.
- **Clarification of the R4 baseline.** The E1b README's 88.32 % branch-only / 95.23 % combined
  are the Arm A whole-scope figures; Arm B whole-scope is 88.62 % / 95.36 %. R4 concerns
  `state_repository.py`, whose E1b Arm B per-file figure is 45/56 branches = **80.36 %**
  branch-only and 273/294 = 92.86 % statements + branches. R4 compares against these per-file
  figures, using E1b's `results/cov-b/.coverage` combined with a coverage run of the new
  sequences (same `coverage run --branch --include=<8 scope files>` settings).

## Amendment 2 (2026-11-03, execution session, before the solver runs)

- **R5 result before solving** (Amendment 1 corpus): 472 captured calls, 285 distinct inputs,
  0 mismatches (`results/logs/r5-check-seeds.log`). Golden corpus: 0 calls (vacuous).
- **Type failure, recorded exactly.** The lifted functions take plain data, no pydantic. The first
  drivers declared the enumerated literals as `typing.Literal[...]` inside tuples and lists.
  CrossHair 0.0.112 on CPython 3.13.16 raised, inside its own type unification
  (`crosshair/dynamic_typing.py:179 unify` → `typing.py:572 __subclasscheck__`),
  `TypeError('typing.Literal cannot be used with issubclass()')` on both sides of every path that
  reached a list element. That made both sides fail identically and reported "No differences found"
  (smoke runs `smoke-ch-166`, `smoke-ch-166-2`, 30 s each, plus an 8 s verbose diagnosis; 72 s of
  solver time, counted against the 1 h budget). A difference for 166 exists and is checked by
  hand in plain Python.
- **The one permitted retry.** One level further down: every literal enumeration becomes an int
  index into a fixed tuple of the compared literals plus one extra value, and an identity column
  becomes `Optional[str]` (None = the oracle's identity literal). The conversion is in
  `drivers.py` (`_body`, `_instr`, `_id`). Bounds are unchanged. Solver runs: 290 CPU-seconds
  `--per_condition_timeout` per target, 330 s wall cap, default exception equivalence.

## Amendment 3 (2026-11-03, execution session, after four solver runs, before the rest)

- **Finding: the amendment 2 drivers never reached the lifted code.** Runs `ch-150`, `ch-166`,
  `ch-169` and `ch-186` (290 CPU-s each) all printed "No differences found" (2,359 to 4,138
  iterations). A verbose diagnosis of 166 (`diag-ch-166-verbose`, 20 s) shows **no stack frame in
  `lifted.py` on any path**: every path ended in the drivers' bound checks. Probes
  (`diag-probe-*`, 5 to 8 s each) show each check is satisfiable alone, but their conjunction
  (nine strings each `len <= 12` and NUL-free, three index ranges, per row) was never satisfied:
  CrossHair's search draws long strings with probability 0.75 per string. Those four outcomes are
  therefore **void** (tool did not explore the predicate), not "no difference within bounds". The
  run of 200 was stopped after about 2 min (no result; the stop is not in the receipt because the
  wrapper was killed with it). The difference for 166 exists (checked by hand).
- **Change.** The bounds are imposed by construction instead of by rejection: strings cut to 12
  characters, enumeration indexes taken modulo the enumeration, lists cut to 3. Same domains,
  no rejected inputs. NUL characters are no longer excluded by the driver; a counterexample
  holding one is mapped to another character at realisation, and the realised sequence must
  still distinguish in the oracle. The amendment 2 file is kept as
  `results/superseded/drivers-v2.py.txt` (sha256 `8368b4f9…3b71`).
- **Budget.** Solver time used before this change: about 1,482 s (smokes, four void runs, the
  stopped run, diagnosis). The remaining ~2,100 s of the 1 h total give **200 CPU-s
  `--per_condition_timeout` per target** (230 s wall cap) for all ten targets, rerun in the
  plan's order. This is below the plan's 300 s per target; the plan's 1 h total is kept.

## Amendment 4 (2026-11-03, execution session, after the amendment 3 runs, before realisation)

- **Solver results (amendment 3 drivers, 200 CPU-s each):** one counterexample (418). The other
  nine printed "No differences found" (383 to 3,414 iterations), none "All paths exhausted", so
  none is an equivalence verdict.
- **Diagnosis, within the 1 h total.** A verbose 10 s run of 241 (`diag3-ch-241-verbose`) again
  shows no SMT decision inside `lifted.py`: cutting each string with `x[:12]` makes CrossHair
  realise the string character by character, so the lifted comparisons see concrete values and
  the search degenerates to sampling. A probe without the cut (`diag-probe-241-unsliced-2`,
  30 s, `results/superseded/diag-probe-241.py.txt`) finds a 241 counterexample at once
  (`rest.parent = ''`, `turn_id = '\x00\x00'`, `representation = 'legacy'`). Solver time is now
  ≈ 3,540 s of the 3,600 s budget, so **solving stops here**, as the plan requires.
- **Consequence for the predictions.** R1 to R3 are scored on the preregistered runs only
  (amendment 3 drivers). The nine non-results are classified "unresolved: driver bounds
  concretised the symbolic strings (tool/driver artefact), not explored". The 241 probe
  counterexample is realised and run as a **supplementary** row, labelled as such and excluded
  from R1 and R2 (241 is an uncovered target and not in R2's denominator anyway).
- **Realisation mapping.** PostgreSQL text and JSONB cannot hold NUL, and `InstructionPayload`
  requires a ULID `id` and a non-empty `parent`. Solver strings are therefore mapped
  injectively to printable ones that keep every equality and, where a mutated `<=` is involved,
  the order relation between the strings the predicate compares; solver ids become ULIDs.
  Fields that the lifted predicate does not read but that the enclosing reader requires before it
  calls the predicate (`child_kind = 'limit_order'`, an `order_id`, kind `rest`) are filled
  in. The mapping is listed in the README.
- **Correction (same session, after the runs).** Recorded solver time is 3,411 s; the stopped
  amendment 2 run of 200 (≤ 190 s, no receipt entry) and the first verbose diagnosis (~9 s, run
  outside the receipt wrapper) are not in it. The total is therefore 3,411 to ≈ 3,610 s: the
  1 h budget was reached and may have been exceeded by up to ≈ 10 s.
