# E1c-2 plan: corrected solver rung and the oracle's missing readers

- Written: 2026-11-03 evening, Europe/Berlin, after R23, before any E1c-2 code
  or execution. Owner decision, 2026-11-03: run this on 2026-11-04.
- Predecessor: [E1c](../e1c-concolic-rung/plan.md) and
  [R23](../../../research/2026-11-03/05-e1c-concolic-rung.md). E1c failed as
  preregistered; R23 attributes the non-results to the research drivers and
  to the oracle's operation alphabet, not to the solver.
- Type: preregistered. Predictions fixed below; amendments dated.
- Runtime: native Windows, plain user account, no WSL, never a bare `bash`
  from scripts. One heavy job at a time; serial pytest.

## Question

With symbolic strings left symbolic, literals encoded as integers, bounds
enforced by construction, and the oracle able to call the four reader
functions, do solver-chosen inputs kill the mutants that E1b and E1c left
alive, and does the solver prove any of them equivalent?

## Frozen inputs

- Specimen, frozen copy, mutant copy and E1b oracle exactly as in E1c
  (`.runs/e2-facdrone/source`, `.runs/e1c-facdrone/mut`, `e1b_oracle.py`,
  golden trace `92a48f77…`, `golden-e1c.json` `c69cb957…`).
- Targets: the ten E1c targets (state_repository.py 150, 166, 169, 186, 200,
  418, 584, 642, 240, 241). 418 is already killed by a solver input and 186
  and 241 by diagnostic sequences; they are rerun under the corrected method
  so every target has one comparable preregistered outcome.
- Lifted functions: `e1c-concolic-rung/lifted.py`, R5-verified on 472 calls
  (E1c). Unchanged unless a target needs a non-interned string domain, see
  method step 2.
- CrossHair 0.0.112 with z3-solver 5.3.3.0 in `.runs/e1c-facdrone/extras`.

## Method

1. **Drivers rewritten** (`drivers2.py`): no string slicing, no rejection
   guards. Bounds are imposed by construction: strings drawn from a small
   symbolic alphabet with symbolic length at most 8, lists of length at most
   3, enumerated fields as integer indexes mapped to the literal set inside
   the driver. Each driver is smoke-run under `crosshair check` with a trivial
   postcondition to confirm paths enter `lifted.py` before any target run.
2. **Non-interned domain for 584.** `kind is "set_stops"` differs from `==`
   only for a non-interned equal string. The driver builds `kind` through
   `json.loads` of a symbolic JSON string, which is what the production code
   does. If CrossHair cannot symbolise through `json.loads`, the driver
   constructs the string by concatenation; the choice is recorded.
3. **Solver runs**: `crosshair diffbehavior` original versus mutated lifted
   function, `--per_condition_timeout 300`, one target at a time, total
   solver budget 1.5 h. Outcomes recorded verbatim: counterexample, "no
   differences found" with or without exhaustion, error.
4. **Oracle alphabet extended** (`e1c_source.py`, already has `reader`,
   `ihist`, `served`, `commit_instr`): add sequences that call
   `opening_tickets`, `resting_orders`, `attempt_history` and
   `decision_served` on recorded and realised states. Record the original's
   observations into `golden-e1c2.json` with sha256; replay once on the frozen
   copy; zero divergence required before any mutant run.
5. **Realise** each counterexample as rows and history (NUL-safe mapping as
   in E1c), record, replay, then run the mutant copy under Arm B with the new
   sequences only, one target at a time.
6. **Classify** each target: killed by realised solver input; killed by the
   extended alphabet alone (reader called, no solver input needed); proved
   equivalent within bounds (exhausted, no difference); unresolved with reason.

## Predictions, fixed before execution

- S1: the solver returns a counterexample or an exhausted no-difference
  verdict for at least 7 of 10 targets.
- S2: at least 6 of the 8 covered survivors are killed, by realised solver
  input or by the extended alphabet.
- S3: at least 1 target is proved equivalent within bounds, and 642 is among
  the proved or the killed.
- S4: branch coverage of `state_repository.py` under Arm B plus the new
  sequences is at least 84%, from the E1b per-file figure of 80.36%.
- S5: every driver smoke run shows at least one path entering `lifted.py`
  before the target run starts; no void runs.

## Kill criteria

- S5 fails for any driver: fix once, record; fails again, that target is
  void and reported as such.
- S1 and S2 both fail with S5 holding: the guided-generation claim is
  rejected for database-entangled predicates on this scope, honestly this
  time, and E3 declares the residue unprotected.
- Golden-e1c2 replay diverges on the frozen copy: stop, the alphabet
  extension is defective.

## Budget

One agent session. Research code at most 250 new lines on top of E1c's
(drivers2.py, alphabet additions, classification script); counted. Solver 1.5
h total, 300 s per target. Arm B reruns on 10 targets only.

## Not claimed

No compression, no candidate, no whole-scope equivalence. A positive E1c-2
raises the oracle's measured sensitivity on this scope and supplies E3 with
the reader-inclusive alphabet and the pure-core requirement.
