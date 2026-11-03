# E1c: constraint-guided inputs against the E1b residue

Preregistration and amendments: [plan.md](plan.md). Amendment 1 changes the R5 corpus and states
the R4 baseline. Amendment 2 records the type failure and the one permitted retry. Amendment 3
voids four runs and changes how the bounds are imposed. Amendment 4 stops solving at the 1 h
budget and adds one supplementary row. The predictions R1 to R5 were not changed.

**Status (2026-11-03): executed, native Windows, plain user account.** The solver budget is spent.
The research entry under `docs/research/` has not been written.

## Results

### Predictions

| | Prediction | Measured | Verdict |
| --- | --- | --- | --- |
| R1 | counterexample or equivalence verdict for ≥ 7 of 10 targets | **1 of 10**: a counterexample for 418. The other 9 ended with "No differences found" without "All paths exhausted", so none is an equivalence verdict | **fails** |
| R2 | realised counterexamples kill ≥ 5 of the 8 covered survivors | **1 of 8** (418) from preregistered solver output. With the supplementary probe sequence: 2 of 8 (418, 186). 186 was killed by the recorded SQL text, not by the predicate's behaviour | **fails** |
| R3 | ≥ 1 covered survivor proved equivalent within bounds, confirmed by inspection | **0**. For 642 (`limit(1)` → `limit(2)` before `.first()`), inspection says equivalent: `decision_id` is the primary key, so at most one row matches. The solver did not prove it (1,874 iterations, not exhausted) | **fails** |
| R4 | branch coverage of `state_repository.py` ≥ E1b Arm B + 2 points | E1b Arm B: 45/56 = **80.36 %**. Plus the preregistered realised input (C418): 45/56 = **80.36 %** (+0.00). Plus the supplementary X241 too: 47/56 = 83.93 % (+3.57; statements + branches 92.86 % → 93.88 %) | **fails** on preregistered inputs; the +3.57 comes only from the supplementary sequence |
| R5 | every lifted function matches the original on every recorded input | Golden corpus: **0 calls** reach any target function (vacuous; amendment 1). Seed-test corpus (36 seed files): **472 of 472** calls reproduced, 285 distinct inputs, 0 mismatches | **holds** on the seed corpus; vacuous on the golden corpus |

**Kill criterion.** R1 and R2 both fail. Under the plan, the guided-generation claim is
**rejected for database-entangled predicates on this scope, under the method as amended**, and E3
must declare this residue unprotected. R5 holds, so the lifting kill criterion does not fire.

The diagnostics show a caveat that this verdict should carry. Eight of the nine non-results
are mostly a driver-encoding artefact, not a demonstrated limit of solver reach:

- The amendment 2 drivers rejected out-of-bound inputs. That starved the search, and no path
  entered `lifted.py`.
- The amendment 3 drivers cut strings with `x[:12]`. That made CrossHair realise strings
  character by character, so the lifted comparisons saw concrete values and the search
  degenerated to sampling.
- A 30 s probe without the cut found the 241 counterexample at once.

The ninth, 584, is different: its domain cannot express the distinguishing input (see below).
Solving stopped because the plan's 1 h total was reached. The rejection is therefore of
"CrossHair 0.0.112 with these drivers within 1 h", not of solver-guided inputs in general.

### Per target

CrossHair outcome: the preregistered amendment 3 run, 200 CPU-s `--per_condition_timeout`, 230 s
wall cap. Arm B: E1b's `run_arm.py` with the replay target swapped for the E1c sequences only.
That means the R16 check, then the selected seed files, then `test_e1c_replay.py`, with `-x` and
`--deadline auto`.

| Line | Operator (cosmic-ray) | Lifted function | CrossHair (v3) | Realised | Arm B (E1c seqs) | Classification |
| ---: | --- | --- | --- | --- | --- | --- |
| 150 | Eq→LtE (SQL `strategy_id <=`) | `opening_tickets` | no difference, 670 it., not exhausted | — | survived (exit 0, 89.6 s) | unresolved: driver bounds concretised strings; the reader is never called by an E1c sequence |
| 166 | And→Or | `opening_tickets` | no difference, 383 it. | — | survived (49.4 s) | unresolved: same artefact. A difference exists (hand-checked) |
| 169 | Eq→LtE (`representation <=`) | `opening_tickets` | no difference, 429 it. | — | survived (48.6 s) | unresolved: same artefact |
| 186 | Eq→IsNot (SQL account predicate becomes `True`) | `resting_orders` | no difference, 683 it. | — | **killed** (exit 1, 67.0 s) by supplementary X241: `/sql[0][0]` text differs | killed by a realised (supplementary) input, through the recorded SQL text; no preregistered solver input |
| 200 | Eq→GtE (`len(matched) >= 1`) | `resting_orders` | no difference, 734 it. | — | survived (27.7 s) | unresolved: same artefact (needs two legacy-matched rests) |
| 418 | Eq→GtE (SQL `rank >= 1`) | `instruction_history` | **counterexample** (200.8 s run) | C418 | **killed** (exit 1, 89.6 s): `C418 op 1 {"op":"ihist"}: /result (length 1 recorded vs 2 replayed)` | **killed by realised solver input** |
| 584 | Eq→Is (`kind is "set_stops"`) | `intention_ids` | no difference, 3,414 it. | — | survived (97.2 s) | unresolved: the driver draws `kind` from the interned literals, so `is` equals `==` on the whole domain. A distinguishing input needs a non-interned string, such as a JSON-decoded payload. The domain cannot express it |
| 642 | NumberReplacer (`limit(2)`) | `decision_served` | no difference, 1,874 it., not exhausted | — | survived (32.4 s) | unresolved by the solver; equivalent by inspection (primary key `decision_id`). Not a proof |
| 240 | Eq→Is (`representation is "legacy"`) | `placed_under` | no difference, 2,523 it. | — | survived (24.2 s, uncovered: replay only) | unresolved: string concretisation. A DB-loaded `"legacy"` is a fresh string, so this mutant is very likely killable |
| 241 | Eq→LtE (`rest.parent <=`) | `placed_under` | no difference, 2,474 it.; **probe without the cut: counterexample in ≤ 30 s** | X241 (supplementary) | **killed** (exit 1, 24.2 s, replay only): `/result/01E1C…01 (present on one side only)` | killed by a realised probe input (supplementary, outside R1/R2) |

Totals: preregistered solver kills **1** (418). With the supplementary sequence: **3** (418, 186, 241).
Proved equivalent: **0**. Unresolved: **7** (150, 166, 169, 200, 584, 642, 240).

Counterexample 418 (`results/logs/ch3-418.log`):
`rows=[(None, None, 2, 0, '\x00…\x01…', 0), (same), (None, None, 0, 0, None, 0), …]`. Here `None`
is the oracle identity, index 2 is flow `instruction`, both rows have version 0, the same key
(13 characters, cut to 12) and instruction token 0, and the third row has flow NULL. The lifted
original returns `[0]` and the mutant returns `[0, 0]`. Only the first three rows are within the
bound.

Counterexample 241 (probe, `results/logs/diag-probe-241-unsliced-2.log`):
`rest=('', '\x00', '', '\x00\x00\x00\x00')`,
`body=(…, representation='legacy', turn_id='\x00\x00', venue_symbol='\x00\x00\x00\x00')`.
The original returns False and the mutant returns True.

### Realisation mapping (amendment 4)

| Solver value | Realised | Why |
| --- | --- | --- |
| 418 key `'\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x01'` | `"k"` | JSONB cannot hold NUL; only equality between the two keys matters |
| instruction token `0` | one valid `InstructionPayload` dict (`id` ULID `01E1C0…0`, kind `open`, parent `p`, venue `v`, grant n=1) | the original validates the body |
| row decision ids (`"d"` in the lifted model) | `e1c:0`, `e1c:1`, `e1c:2` | primary key |
| 241 `rest.parent ''` < `turn_id '\x00\x00'` | `"a"` < `"b"` | `parent` must be non-empty; order and inequality kept |
| 241 venue `'\x00\x00\x00\x00'` on both sides | `"v"` | equality kept |
| fields not read by `_placed_under` | `child_kind = limit_order`, `order_id = o1`, rest kind `rest`, rest id ULID `01E1C0…1` | the enclosing `resting_orders` filters on them before calling the predicate |

The original's observations: `results/golden-e1c.json`, sha256
`c69cb957e2de07faae121262024e10ab070fac5431fc3eef45834f3c8937d487`, with 2 sequences, 4
operations and no errors. Replay on the frozen copy reports "no divergence". The E1b golden trace
`92a48f77…` is untouched, and so is `e1b_oracle.py` (sha256 `c4c93118…f074`, the same as in all
E1b receipt facts).

### A finding outside the predictions

Four of the E1b survivors (150, 186, 418, 642) mutate SQL predicates or parameters. The E1b oracle
records every SQL statement with its parameters, so any sequence that merely calls the enclosing
reader would expose them. 186 was killed that way by X241, through the query text alone. They
survived E1b because its golden corpus never calls `opening_tickets`, `resting_orders`,
`instruction_history` or `decision_served` (0 captured calls). This residue is a gap in **reach**
(which entry callables the oracle drives), not in input choice.

## CrossHair, bounds and time

- CrossHair `crosshair-tool==0.0.112` with `z3-solver==5.3.3.0`, installed into
  `D:\Source\lesscode\.runs\e1c-facdrone\extras` with the E2 venv interpreter (CPython 3.13.16).
  Run as `python -m crosshair diffbehavior --per_condition_timeout 200 drivers.o_X drivers.mL`.
  `PYTHONPATH=<extras>;<e1c dir>`, thread variables 1, default exception equivalence.
- Bounds (amendment 3 drivers, `drivers.py`):
  - at most 3 rows, 3 instructions and 3 candidates;
  - strings cut to 12 characters;
  - enumerations as modulo indexes: flow {None, attempt, instruction, decision, other}, child kind
    {None, market_order, stop_attach, limit_order, other}, outcome {None, rejected, skipped,
    unavailable, accepted}, instruction kind {open, rest, set_stops, other};
  - identity columns are None (= `e1b-account` / `e1b-strategy`) or a free string;
  - NaN stop levels map to the same sentinel on both sides.

| Solver time | Seconds |
| --- | ---: |
| Smokes (Literal drivers) | 64.6 |
| Amendment 2 runs 150, 166, 169, 186 (void) | 1,171.1 |
| Amendment 2 run 200, stopped (not in receipt) | ≤ ~190 |
| Diagnosis (verbose runs, probes) | 162.6 recorded + ~9 unrecorded |
| Amendment 3 runs, 10 targets (preregistered results) | 2,013.1 |
| **Total** | **3,411 recorded; ≈ 3,610 upper estimate** (budget 3,600 s, reached, possibly exceeded by ≈ 10 s) |

Other times: R5 seed capture 96.4 s + 24.2 s, golden capture 32.4 s, golden-e1c record 15.0 s,
replay 13.2 s, ten Arm B mutant runs 549.9 s in total (24 to 97 s each), coverage runs 28.7 s and
30.6 s.

## Research code (physical lines, `wc -l`)

| File | Lines | Role |
| --- | ---: | --- |
| `lifted.py` | 95 | the ten sites as pure functions; `mut` selects the mutation |
| `drivers.py` | 100 | CrossHair drivers (amendment 3 version) |
| `e1c_source.py` | 104 | new oracle operations (`rows`, `reader`, `ihist`, `served`, `commit_instr`), the realised sequences, record/replay of `golden-e1c.json` |
| `e1c_capture.py` | 74 | R5 capture hook (pytest plugin `-p e1c_capture`, or `golden` mode) |
| `check_r5.py` | 46 | R5 comparison |
| `mutate.py` | 40 | apply or restore one target's exact cosmic-ray line in the copy |
| `receipt.py` | 44 | receipt with per-command logs |
| `run_e1c_arm.py` | 19 | E1b `run_arm.py` with the E1c replay target |
| `test_e1c_replay.py` | 7 | Arm B pytest target |
| `results/analysis/replay_only.py` | 6 | replay a subset (R4 split) |
| **Total** | **535** | **budget 400: exceeded by 135** |

The overrun went to the R5 capture and check (120 lines, needed because the golden corpus is
vacuous), the receipt with logs, and the copy mutator. Not counted: the superseded amendment 2
drivers (`results/superseded/drivers-v2.py.txt`, 117 lines) and the diagnostic probe
(`results/superseded/diag-probe-241.py.txt`).

## Commands as executed (PowerShell; full argv, env, times and exits in `results/receipt.json`)

```powershell
$V="D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe"; $S="D:/Source/lesscode/.runs/e2-facdrone/source"
# extras
& $V receipt.py run extras-install-crosshair -- uv.exe pip install --python $V --target D:/Source/lesscode/.runs/e1c-facdrone/extras crosshair-tool
# R5: golden replay with the hook (0 calls), seed files with the hook, check
& $V receipt.py run r5-capture-golden-2 --env PYTHONPATH=<e1c;e1b;e1b extras;e2> --env E1C_CAPTURE=results/r5-capture-golden.jsonl -- $V e1c_capture.py golden
& $V receipt.py run r5-capture-seeds-root-2 --cwd $S --env PYTHONPATH=<S/src;e1c;e1b;extras;e2> --env E1C_CAPTURE=results/r5-capture-seeds.jsonl -- $V -m pytest -q -c pyproject.toml --rootdir . -p no:cacheprovider -p e2_winshim -p e1c_capture --timeout=600 -m "not port_baseline and not live_broker" <35 files>
& $V receipt.py run r5-capture-seeds-reporting-2 --cwd $S/packages/facdrone-reporting ... tests/test_p1_explanation_acceptance.py
& $V receipt.py run r5-check-seeds -- $V check_r5.py results/r5-capture-seeds.jsonl
# solver (amendment 3 drivers), one target at a time
& $V receipt.py run ch3-<L> --timeout 230 -- $V -m crosshair diffbehavior --per_condition_timeout 200 drivers.<o_fn> drivers.m<L>
# realisation and Arm B
& $V receipt.py run record-e1c -- $V e1c_source.py record ; & $V receipt.py run replay-e1c-frozen -- $V e1c_source.py replay
& $V receipt.py run copy-mut-native -- $V -c "import shutil; shutil.copytree(r'$S', r'D:/Source/lesscode/.runs/e1c-facdrone/mut', ignore=shutil.ignore_patterns('__pycache__'))"
foreach L: apply-<L>: mutate.py apply <mut> <L> ; armB-e1c-<L> (cwd mut): run_e1c_arm.py B --deadline auto ; restore-<L>: mutate.py restore <mut>
# R4
& $V receipt.py run e1c-cov-frozen --cwd $S --env E1B_SEEDS= -- $V run_e1c_arm.py B --cov results/cov-e1c
coverage combine (E1b cov-b/.coverage copy + E1c data) -> results/cov-r4 ; run_arm.py report results/cov-r4 $S
coverage run --branch --include=<8 scope files> results/analysis/replay_only.py C418-superseded-instruction -> results/cov-r4-primary (+ E1b cov-b copy)
```

## Deviations and incidents

1. **Amendment 1.** The golden corpus never reaches the targets (0 captured calls), so R5 also
   uses a seed-test corpus. The R4 baseline is stated as the per-file Arm B figure, 80.36 %.
   The task brief cited 88.32 % / 95.23 % as "Arm B". Those are E1b's Arm A whole-scope figures;
   Arm B whole-scope is 88.62 % / 95.36 %.
2. **Pytest plugin name clash.** The capture module was first named `capture.py`. `-p capture`
   then loaded pytest's built-in `capture` plugin, and the first seed capture
   (`r5-capture-seeds-root`, `-reporting`) recorded nothing. It was renamed `e1c_capture.py` and
   rerun (`-2` labels). The golden capture was repeated after the rename (`r5-capture-golden-2`):
   still 0 calls.
3. **Amendment 2.** `typing.Literal` inside containers breaks CrossHair 0.0.112's type
   unification (`TypeError: typing.Literal cannot be used with issubclass()`, both sides, every
   path). Literals became int indexes: the one permitted retry.
4. **Amendment 3.** The rejection-guard drivers never reached `lifted.py`. Runs `ch-150` to
   `ch-186` are void, and `ch-200` was stopped. The bounds were imposed by construction, with the
   per-target time cut from 300 to 200 CPU-s to stay inside the 1 h total.
5. **Amendment 4.** The cut-by-slicing drivers concretised strings (diagnosed). Solving stopped at
   the budget. One probe counterexample (241) was realised as a labelled supplementary row.
6. No `wsl.exe` and no bare `bash` were invoked from Python or PowerShell. Git Bash was used only
   interactively by the agent for file inspection. Copies used `shutil.copytree`.
7. Frozen copy: `scope.py` reports `changed: []` after all runs, and so does the mutant copy
   `.runs/e1c-facdrone/mut` after its last restore. `D:\Source\facdrone` HEAD is `d49f48bd…`
   and `git status --short` is empty. Mutants existed only in `.runs/e1c-facdrone/mut`, one at a
   time, each applied as cosmic-ray's exact diff line (CRLF kept) and restored by copying the
   frozen file back.

## Limitations

1. **The verdict is tool- and driver-bound.** Eight of nine non-results trace to how the bounds
   were encoded (rejection, then slicing), and the ninth, 584, to a domain that cannot express
   the input. None traces to a demonstrated limit of constraint solving. The probe shows CrossHair
   solves 241 in seconds with unsliced strings. A rerun with unsliced, length-capped-by-type
   drivers was not possible inside the 1 h budget.
2. **SQL modelled in Python.** The lifted `_attempts`, `instruction_history` and
   `decision_served` model the WHERE clause, the window ordering and NULL handling in Python. The
   150 mutant (`<=` on text) uses code-point order, while PostgreSQL uses the database collation.
   R5 checks agreement only on the 472 captured calls. They include 7 `instruction_history` calls
   with a superseded key, but no call that exercises collation.
3. **Identity-sensitive mutants (240, 584).** `is` against a literal differs from `==` only for
   non-interned strings. The drivers draw enumerated values from the interned literals, so 584 is
   indistinguishable within its domain. Realising either mutant needs strings from JSON or the
   database: a property of object identity, not of value.
4. **Supplementary evidence.** The 186 and 241 kills come from a probe outside the
   preregistered solver runs, and they are reported separately. 186's kill shows the SQL-recording
   oracle's strength, not the predicate's semantics.
5. **R5 corpus.** The seed corpus never reaches the `_placed_under` legacy branch (1
   `placed_under` call, on the `instruction_id` branch) or `decision_served` with more than 2
   rows. Lifting of those paths is checked only by construction and by the realised replays.
6. **Arm B runs the seeds first, with `-x`.** A seed failure would mask the replay. None occurred:
   every killed run fails only in `test_e1c_replay`.

## Files

| File | Role |
| --- | --- |
| `plan.md` | preregistration and amendments 1 to 4 |
| `results/receipt.json` | every command (argv, cwd, env, UTC start/end, seconds, exit), notes (solver-time accounting, R5, golden-e1c, Arm B, R4), facts (versions, hashes, line counts) |
| `results/logs/` | output of every recorded command (`ch3-*` solver outcomes, `armB-e1c-*`, `r5-*`, `diag*`) |
| `results/r5-capture-seeds.jsonl`, `results/r5-corpus-stats.json` | R5 corpus (472 calls) and its statistics |
| `results/golden-e1c.json` (+ `.sha256`) | the original's observations of the E1c sequences |
| `results/cov-e1c/`, `results/cov-r4/`, `results/cov-r4-primary/` | coverage: E1c only; E1b Arm B + all E1c; E1b Arm B + C418 only |
| `results/superseded/` | amendment 2 drivers and the diagnostic probe |
