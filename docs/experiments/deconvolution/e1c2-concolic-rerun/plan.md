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

## Amendment 1 (2026-11-04, execution session, after the driver smokes, before any target run)

- **Smoke harness and the S5 detection rule.** `crosshair check` reads PEP 316 contracts from the
  source, so the trivial postcondition sits on one generic function, `drivers2.smoke`
  (`"""post: True"""`), which takes the signature of the driver named in `E1C2_SMOKE`. The first
  trial (`smoke-o_placed`, 1.5 s) found no checkable function because the docstring had been set
  at run time; recorded, not a solver result. The second trial (`smoke-o_placed-2`, `--verbose`,
  15 s, 69 iterations, every path CONFIRMED) showed **no SMT decision with a `lifted.py` frame**,
  yet the paths did run the lifted code: with string lengths fixed by construction, the lifted
  comparisons build symbolic booleans without branching, so a missing decision frame does not mean
  a missing entry (E1c's diagnostic rule is too strict for these drivers). S5 is therefore scored
  by **entry markers**: in smoke mode only, each lifted function is wrapped to print
  `ENTER lifted.<name>` before calling the unchanged original. A driver passes S5 if its smoke log
  holds at least one marker. Target runs do not set `E1C2_SMOKE` and run the lifted code unwrapped.
- **Step 2 choice.** `kind` (584) is built by `json.loads('"' + v + '"')`, where `v` is either an
  enumerated literal or a free symbolic string. CrossHair 0.0.112 symbolises through `json.loads`
  (its pure-Python `jsonlib` with symbolic regex matching): the verbose trial shows decisions in
  `jsonlib.py`/`relib.py` frames and 0 realisations. The concatenation fallback is not used.
- **Same domain for 240.** `_placed_under` compares `representation` by identity under mutant 240,
  and production reads that field from JSONB. A scratch check on the embedded database (frozen
  copy, not in the receipt, about 15 s): a stored `"legacy"` read back through the SQLAlchemy
  engine is equal to but **not** identical with the literal. The drivers therefore pass
  `representation` through the same `json.loads` encoding as `kind`. `lifted.py` is unchanged.
- **Smoke m241.** Exit 1 after 14 entries: the mutant raises `TypeError` for `rest.parent <= None`
  (a legacy body without `turn_id` and `parent_id`). That is the mutant's behaviour, and the path
  entered `lifted.placed_under`, so S5 holds for this driver.
- **Budget.** Smokes and trials: 295.2 s of solver time, counted against the 1.5 h total.

## Amendment, 2026-11-04 (owner request, before classification): solver-encoding verifier

Before any solver verdict is counted, the solver pipeline must pass four
checks, recorded in the receipt:

1. Symbolic-ness: each driver's smoke run explores more than one path into
   `lifted.py` (S5, unchanged).
2. Encoding soundness: each lifted predicate reproduces the original on every
   recorded input (E1c's R5 check, rerun if `lifted.py` changed).
3. Pipeline controls: the known-killable target 418 must yield a
   counterexample, and one known-equivalent annotation-only mutant must yield
   "no differences found" with "all paths exhausted". If either control
   fails, the solver session is void as a whole, not only that target.
4. Counterexample realizability: every counterexample is replayed
   concretely through the original and the mutated lifted function and must
   differ before it is realised as rows. "No differences found" without
   exhaustion is recorded as inconclusive and never as equivalence.

Predictions S1 to S5 are unchanged. A void session is reported as void.

## Amendment 2 (2026-11-04, execution session, owner request "solver-encoding verifier", during the target runs, before any verdict is counted)

Owner request relayed by the coordinator while `ch-150` to `ch-241` were running; no solver
verdict had been read or counted. S1 to S5 are unchanged. Before any solver verdict counts toward
S1 to S3, four checks are run and recorded in `results/receipt.json` and in the README section
"Verifier":

1. **Symbolic-ness.** Each driver smoke shows more than one explored path into `lifted.py`,
   scored by the Amendment 1 entry markers (more than one `ENTER` line per smoke log).
2. **Encoding soundness.** Each lifted predicate reproduces the original on every recorded
   input: E1c's R5 check (`check_r5.py` on `r5-capture-seeds.jsonl`, 472 calls). `lifted.py` is
   unchanged (sha256 compared with the E1c receipt); the check is rerun anyway because it is cheap.
3. **Pipeline controls.** (a) Target 418 must yield a counterexample under `drivers2`. (b) One
   known-equivalent annotation-only mutant from the E1b sample must yield "No differences found"
   **with** "All paths exhausted". Chosen: `decision_evidence_publication.py` line 191,
   `ReplaceBinaryOperator_BitOr_Sub` occurrence 1 (`-> str | None` becomes `-> str - None`, under
   `from __future__ import annotations`). The mutated function is compiled **in memory** from the
   frozen file's text with cosmic-ray's exact diff line (`ctl2.py`; no mutant file is written
   outside `.runs/e1c-facdrone/mut`). Its driver chooses between the two concrete arguments the
   function distinguishes, `None` and one recorded `ConsumedCapture` (Facdrone's
   `capture_details()`), so a correct pipeline must exhaust the paths. Same command form and
   `--per_condition_timeout 300`. If (a) or (b) fails, the solver session is **void as a whole**
   and no target counts.
4. **Counterexample realisability.** Every counterexample is replayed concretely (no CrossHair)
   through the original and mutated lifted functions via the drivers, and must differ before it is
   realised as rows. "No differences found" without "All paths exhausted" is recorded as
   inconclusive, never as equivalence.

The verifier's code (`ctl2.py`, the concrete replay) is counted in the line budget and itemised.

## Amendment, 2026-11-04 (owner request, during solver runs): adaptive budgets

The host is shared and sometimes busy, so fixed wall-clock caps can turn load
into false timeouts. From this point every deadline in E1c-2 is adaptive,
using `docs/experiments/deconvolution/adaptive.py`:

- Before each heavy step, record `adaptive.py probe` in the receipt.
- Deadlines are `scaled(base)`: the preregistered base (300 s per solver
  target, Arm B deadlines, the 1.5 h solver budget) multiplied by the
  measured load factor, clamped to [1, 4]. Base values are unchanged; the
  budget means work on an idle machine.
- A timeout observed while the factor exceeded 1.5 is rerun once after
  `wait_for_idle`, and only a second timeout under factor at most 1.5 is a
  counted outcome. Both attempts stay in the receipt.
- The idle reference (`adaptive-reference.json`) was calibrated on
  2026-11-04 while the solver itself was running on one core; it is to be
  recalibrated on a quiet host and the change recorded.

Predictions S1 to S5 are unchanged.

## Amendment 3 (2026-11-04, execution session, owner request "adaptive budgets", after `ch-150` and `ch-166`, before `ch-169`)

- **Change.** Fixed wall-clock caps are replaced by `../adaptive.py` (stdlib; idle reference
  `../adaptive-reference.json`, `ref_bench_s` 0.311, calibrated 2026-11-04 07:41 local). Every
  heavy step from `ch-169` on runs through `step2.py`: it probes the host, sets the deadline to
  `scaled(base)` with the preregistered bases unchanged (300 s per solver target as
  `--per_condition_timeout`, E1b's measured Arm B deadline per selected seed set, the 1.5 h solver
  total), and stores the probe and factor in that step's receipt entry. A timeout-type outcome
  (receipt timeout, `run_arm` "deadline exceeded", or CrossHair "No differences found" without
  "All paths exhausted") at factor > 1.5 waits for idle (`wait_for_idle`, at most 30 min) and
  reruns that step once as `<label>-rerun`; only a second timeout at factor <= 1.5 counts; both
  attempts stay in the receipt.
- **Runs already completed.** `ch-150` (310.7 s wall) and `ch-166` (307.6 s wall) ran with the
  fixed 300 s and ended "No differences found" without exhaustion, a timeout-type outcome. No probe
  was taken before them. A probe taken at 07:41:45 local, while `ch-166` was running, gave factor
  **1.16** (`cpu_other` 0.139 including the running solver on one of 24 cores, bench 0.283 s vs
  0.311 s). Wall time exceeded the 300 s condition timeout by only 7 to 11 s (process start and
  import), so neither run shows load. Both are therefore kept as completed, not marked "rerun
  pending". The series loop was stopped after `ch-166` and before `ch-169` started; no partial run.
- **Budget.** The 1.5 h solver total is compared with `scaled(5400)` at the time of each check.

## Amendment 4 (2026-11-04, execution session, after two control load failures, before the counted control run)

- **Control launcher.** `crosshair diffbehavior ctl2.o_ctl ctl2.m_ctl` refused to load the
  module twice, before any path ran: `SideEffectDetected` for `socket.gethostname` (`ctl-191`,
  2.0 s) and, with that unblocked, for `os.add_dll_directory` from numpy (`ctl-191-2`, 2.5 s).
  These are import-time effects of Facdrone's package, not of the function under test. The
  counted control (`ctl-191-3`) imports `ctl2` first and then calls CrossHair's own `main()` with
  the same arguments (`diffbehavior --per_condition_timeout <scaled 300> ctl2.o_ctl ctl2.m_ctl`),
  so CrossHair finds the module already loaded. No CrossHair option and no driver was changed.
- **Realisation, 584.** The solver's input holds two `set_stops` candidates with the same id and
  one non-dict candidate. `instruction_records` validates every candidate after `_shadow_seed`
  and derives row ids from the instruction id, so a non-dict or a duplicate id makes the whole
  commit fail identically on both sides. The realised sequence keeps the distinguishing part,
  one JSON-decoded `set_stops` candidate with an id; the mapping is in the README.
