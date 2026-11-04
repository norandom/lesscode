# E3 apparatus: regeneration of the evidence-publication core

Preregistration: [plan.md](plan.md). Step 0 (the apparatus) is built and validated on the frozen copy.
Amendment 1 in the plan records the apparatus decisions the plan left open. No candidate exists yet.
Nothing was regenerated.

**Status (2026-11-04): step 0 complete, native Windows, plain user account.** All runs are in
`results/receipt.json`, with logs in `results/logs/`.

## Files

| File | Lines | Role |
| --- | ---: | --- |
| `surface.py` | 60 | public surface → `results/public-surface.json` |
| `measures.py` | 54 | size, edge and structure measures → `results/measures-<label>.json` |
| `run_e3.py` | 44 | acceptance: E1b `run_arm.run` (imported) with all 191 seed files and the replay target swapped |
| `test_e3_replay.py` | 33 | pytest target: replays the 64 golden sequences, Level 1 fails, Level 2 collected |
| `qdiff.py` | 108 | Level 2 differential on perturbed states |
| `step3.py` | 17 | adaptive wrapper: E1c-2 `step2.py` imported, receipt rebound here |
| `receipt3.py` | 10 | E1c `receipt.py` imported, rebound to `results/` |
| total | **326** | budget for all of E3: 400, which leaves 74 lines for `t5_drivers.py` |

Line counts are physical lines, and `receipt3.py facts` records them. E1b, E1c and E1c-2 modules
(`run_arm`, `scope`, `e1b_oracle`, `e1c_source`, `e1c2_source`, `receipt`, `step2`, `adaptive`,
`e2_winshim`) are imported unchanged and are not copied.

## Commands

The interpreter is `.runs/e2-facdrone/venv/Scripts/python.exe` (`PY`). Set the thread variables
`OPENBLAS/OMP/MKL_NUM_THREADS=1`. The cwd is this directory.

```
uv run --no-project --python 3.13 python surface.py <ROOT> --out results/public-surface.json
uv run --no-project --python 3.13 python measures.py <ROOT> [--structure candidate-structure.json] --out results/measures-<label>.json
PY step3.py <label> 2569.2 -- PY run_e3.py .runs/e3-facdrone/cand --round <N> --deadline {T}
PY step3.py <label> 600 -- PY qdiff.py results/sql-divergences-round<N>.json      # or --controls [--weak]
uv run --no-project --python 3.13 python receipt3.py facts '<json>'
```

`run_e3.py` exits 0 only if all of the following hold:

- the scope-import probe passes;
- the R16 output equals `r16-baseline.json`;
- every pytest group passes (serial, `-x`, `-p e2_winshim`, marks `not port_baseline and not live_broker`, `ROOT/src` first on `PYTHONPATH`);
- the replay wrote a fresh `sql-divergences-round<N>.json` covering all 64 sequences.

If the fixture skips, no fresh file is written and the run fails closed.

## Measures, definitions

- **lines**: physical lines.
- **stmts**: count of `ast.stmt` nodes.
- **funcs**: distinct `def` names per file, counting nested functions and methods. `qualnames` counts every def.
- **writes**: matches of `\.(execute|add|add_all|commit|flush|delete|merge)\(|\b(insert|update|delete)\(` on the raw text.
- **edges**: distinct (file, caller, file, callee) pairs. Each pair is a call inside a def, by simple or attribute name, that names a def in one of the five files. Self-calls are excluded.
- **module_hops**: distinct (file, file) pairs among the edges, for different files.

These are the definitions of the plan's scratch script.

`--structure` reads `{"pure": ["<file>.py::<qualname>", ...], "shell": [...]}` and checks two things:

- A declared pure function has no parameter matching `^(engine|conn|connection|session|bind)$` or `_(engine|conn|connection|session)$`. It also has no attribute call to `execute/begin/connect/commit/add/flush/merge/delete`.
- Each write site lies in a declared shell function, judged by its innermost enclosing def.

The check also reports undeclared defs and the pure share.

## Validation on the frozen copy

| Check | Result |
| --- | --- |
| Candidate root `.runs/e3-facdrone/cand` | `shutil.copytree(symlinks=False)` of the frozen copy (`copy-cand`). 4,851 files; the whole tree is identical by sha256. The eight scope files match the frozen copy and the R15 manifest (`verify-cand`). Not modified afterwards |
| Baseline measures (`measures-baseline-2`) | lines **1,274**, stmts **660**, funcs **67**, writes **13**, edges **84** (reproduced), module hops **7** |
| Structure-check self-test | Every def of the original declared pure, no shell. 17 of 67 pure checks fail; 13 of 13 write sites fall outside a shell; overall fail, as expected. One failure is a false positive of the given rule: `deduplicated_inputs` calls `set.add` (line 169) |
| `run_e3.py` on the identical copy, round 0 (`validate-frozen-round0`) | **exit 0**, 546.0 s. Adaptive factor 1.24 (cpu_other 0.20, bench 0.202 s); deadline 3,185.8 s (base 2,569.2). 191 seed files; 24 of the 236 deselections apply. Root group: 1,403 passed, 2 skipped, 10 deselected, which includes `test_e3_replay`. Research: 39 passed, 3 skipped, 6 deselected. Reporting: 92 passed, 8 deselected, 1 xfailed |
| Replay | 64 sequences, 311 operations. **Level 1 divergence: none. Level 2 operations differing: 0** |
| `qdiff.py` on round 0 | 0 pairs, exit 0 |
| qdiff control (i): identical statements | pass, 102 states (8 perturbed states refused by the schema and skipped) |
| qdiff control (ii): R24 m150 shape (`strategy_id =` → `<=` in the recorded `R-S01` attempt query) | **fails** at state 20, as required. That state has a boundary row `account_id=e1b-account, flow=attempt, strategy_id=e1b-strateg` |

The qdiff perturbation needed strengthening. With `--weak` (string flip by reversal only), control
(ii) passed on 102 states, so the perturbation was too weak (`qdiff-controls-weak`,
`qdiff-controls-weak-2`). A first strengthening also failed to detect the mutant: it flipped by
prefix, `~` suffix or a parameter value (`qdiff-controls`). The detecting mutant needs two
coordinated column values on one row. The final version starts every perturbed state with a
**boundary row** (`qdiff-controls-2`, `qdiff-controls-3`). The boundary row is a row of a table the
statements name, with a new key. Every column that a parameter `<column>_<n>` binds is set to `p`,
`p[:-1]` or `p+"~"` (for integers, `p-1`, `p` or `p+1`). Receipt note: `qdiff-strengthening`.

## Public surface (derived, authoritative) versus the plan's transcription

The derived surface has 62 names over 1,754 scanned files. Every transcribed name is present.
The derivation adds the following names:

- `decision_evidence_publication`: `CONSUMED_URI`, `_CAPTURE`, `_Captured`, `deduplicated_inputs`, `parse_run_slice`, `resolved_inputs`, `slice_identity` (test, attribute reads on a module alias); `bound_payload` (prod, `state_repository.py`); `stored_bound_payload` (prod `run_evidence_store.py`, test).
- `decision_evidence_store`: `_BATCH` (test, attribute).
- `decision_evidence_reader`: `read_decision_evidence`, a re-export that a test monkeypatches on the reader module (`setattr(reader, "read_decision_evidence")`).
- `decision_input_slice_store`: `Scope` (test, attribute); `_load` (test, monkeypatched).
- `decision_slice_publisher`: `SliceResolver` (re-export, prod: `state_repository.py`, `run_evidence_store.py`); `referenced_slices` (prod, `run_evidence_store.py`).

Two further derived entries are not part of the surface:

- `unresolved`: `publication.row` in `tests/contracts/test_evidence_compaction_equivalence.py`, where a local variable shadows the alias.
- `intra_set`: imports among the five files, listed for reference. They are not part of the surface.

The two monkeypatched names put a constraint on the candidate. The test only takes effect if the
patched module global is the one the module's functions actually look up.

## Deviations and notes

- **Deselections.** The 236 deselections are E1b's baseline failures over the whole gate tier. Only 24 of them name nodes in the 191 seed files. `run_e3` passes those 24, the same as `run_arm.seeds` for the full set. The other 212 cannot match a collected node.
- **Satellite packages.** They are imported from the frozen copy through the venv's editable installs. Only `ROOT/src` comes from the candidate, as in E1b, which is equivalent because only five `src` files may differ.
- **Receipt overwrite.** E1c's `receipt.main` loads `receipt.json` before it runs a command and writes the file afterwards. Entries added during the 546 s validation were overwritten. They were rerun or re-added (note `receipt-overwrite`), and no receipt is written while a step runs.
- **Compaction.** `surface.py`, `measures.py` and `qdiff.py` were compacted for the line budget after their first runs and rerun with identical outputs. `run_e3.py`, `test_e3_replay.py`, `step3.py` and `receipt3.py` have not changed since the validation.
- **Candidate verification.** It was run from a scratch script (`verify-cand`, path in the receipt), which is not counted as apparatus.

## Step 3 (T4, T5), 2026-11-04

Decisions are in [plan.md](plan.md), Amendment 2. Every command, its adaptive probe and its guard
note (`<label>-guard`) are in `results/receipt.json`, with logs in `results/logs/`. The mutant
copy is `.runs/e3-facdrone/mut`; the candidate in `.runs/e3-facdrone/cand` was only read.

### Commands (PowerShell, cwd this directory, thread variables 1)

```powershell
$PY = "D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe"
$UV = "uv.exe"                                   # WinGet uv; full path in the receipt
$PP = "PYTHONPATH=<cand>/src;<e1c extras>;<e1b extras>;<this dir>"
$LAUNCH = "import sys, t5_drivers; from crosshair.main import main; sys.argv = ['crosshair', *sys.argv[1:]]; sys.exit(main())"
& $PY receipt3.py run copy-mut -- $PY -c "<shutil.copytree(cand, mut, symlinks=False)>"    # then verify-mut (scratch verify_mut.py)
& $PY receipt3.py run t4-cr-init --cwd <mut> -- $UV tool run --python 3.13 --from cosmic-ray==8.7.0 cosmic-ray init cosmic-ray-t4.toml <runs>/t4.sqlite
& $PY receipt3.py run t4-sample -- $UV tool run --python 3.13 --from cosmic-ray==8.7.0 python t4_arm.py sample <runs>/t4.sqlite
& $PY step3.py t4-base 1200 -- $PY t4_arm.py arm --deadline "{T}"
# for k in 0..29, after the guard note t4-m<k>-guard:
& $PY step3.py t4-m<k> 999.5 -- $PY t4_arm.py mutant <k> --deadline "{T}"
# for m in m1, m2, m3, after the guard: t4-apply-<m> (git apply <patch>, cwd <mut>), then t4-restore-<m> (copy back, sha256)
& $PY step3.py t4-hand-<m> 2569.2 -- $PY run_e3.py <mut> --round t4-<m> --deadline "{T}"
& $PY step3.py t5-sound-2 600 --env $PP -- $PY t5_drivers.py sound
& $PY step3.py t5-smoke-<pair>-2 60 --env $PP --env T5_SMOKE=<pair> -- $PY -c $LAUNCH check --per_condition_timeout 15 t5_drivers.smoke   # seal again as -3 with 60
& $PY step3.py t5-ctl-kill-2 300 --env $PP -- $PY -c $LAUNCH diffbehavior --per_condition_timeout "{T}" t5_drivers.k_dedup t5_drivers.c_dedup
& $PY step3.py t5-ctl-equiv-2 300 --env $PP -- $PY -c $LAUNCH diffbehavior --per_condition_timeout "{T}" t5_drivers.e_tiny t5_drivers.c_tiny
& $PY step3.py t5-ch-<pair> 300 --env $PP -- $PY -c $LAUNCH diffbehavior --per_condition_timeout "{T}" t5_drivers.o_<pair> t5_drivers.c_<pair>
& $PY step3.py t5-cx-replay 60 --env $PP -- $PY t5_drivers.py replay results/logs/t5-ctl-kill-2.log results/logs/t5-ch-capture.log
```

All T5 runs also set `PYTHONDONTWRITEBYTECODE=1`. The guard is a scratch script, `guard.py`
(`adaptive.wait_for_idle(1800, 0.35, 8.0)`). It is not apparatus and is not counted.

### T4: sensitivity to 30 cosmic-ray mutants of the candidate's `decision_evidence_publication.py`

The population has 317 specs and the seed is 20261104 (`results/t4-mutant-sample.json`). The arm
uses the 19 importing seed files, 6 deselections and the 64-sequence replay. The unmutated arm
(`t4-base`) took 199.9 s (factor 2.3) and passed, so the deadline base is 999.5 s.

| # | Line | Operator | Outcome | First failing reason |
| ---: | ---: | --- | --- | --- |
| 0 | 83 | AddNot | killed | R16 harness differs |
| 1 | 189 | AddNot | killed | R16 harness differs |
| 2 | 237 | AddNot | killed | R16 harness differs |
| 3 | 320 | NumberReplacer `1`→`0` | killed | replay `golden-trace.json S09-codecs-compact op 10 … /result/consumed_capture` |
| 4 | 287 | BitOr→Add (return annotation) | survived | — |
| 5 | 353 | BitOr→Add | killed | R16 harness differs |
| 6 | 284 | BitOr→Div | killed | R16 harness differs |
| 7 | 189 | BitOr→Mod | killed | R16 harness differs |
| 8 | 216 | BitOr→Mod (parameter annotation) | survived | — |
| 9 | 189 | BitOr→Mul | killed | R16 harness differs |
| 10 | 353 | BitOr→Mul | killed | R16 harness differs |
| 11 | 287 | BitOr→Sub (return annotation) | survived | — |
| 12 | 353 | BitOr→Sub | killed | R16 harness differs |
| 13 | 195 | Mul→LShift (`*,` → `<<,`) | incompetent | SyntaxError (INVALID scope) |
| 14 | 195 | Mul→Mod | incompetent | SyntaxError (INVALID scope) |
| 15 | 267 | Eq→GtE `route.kind >= "scored"` | survived | — |
| 16 | 256 | Eq→Is `route.kind is "scored"` | survived | — (rerun from scratch after the memory kill) |
| 17 | 313 | Eq→IsNot `value.uri is not f"…"` (`compact_capture`) | survived | — |
| 18 | 273 | Is→IsNot (`schema_version`) | killed | R16 harness differs |
| 19 | 86 | NotEq→Eq (duplicate columns) | killed | R16 harness differs |
| 20 | 86 | NotEq→Gt | killed | seed `packages/facdrone-research/tests/test_study_slice_evidence.py::test_invalid_slices_are_refused_before_retention[value0]` |
| 21 | 76 | NotEq→Is (`money_evidence_payload` currency) | survived | — |
| 22 | 86 | NotEq→Is | killed | R16 harness differs |
| 23 | 114 | NotEq→Is (`parse_run_slice`) | killed | seed `tests/unit/test_run_slice_namespace.py::test_malformed_reference_refuses[run-slice:ns/bbbb…]` |
| 24 | 86 | NotEq→Lt | survived | — |
| 25 | 90 | NotEq→LtE (row length) | killed | R16 harness differs |
| 26 | 219 | NotEq→LtE (`details` key filter) | killed | R16 harness differs |
| 27 | 237 | NotEq→LtE | killed | R16 harness differs |
| 28 | 136 | True→False `zip(strict=False)` | survived | — |
| 29 | 226 | True→False pydantic `strict=False` | survived | — |

**Kill rate (R22 convention, survivors in the denominator, incompetent excluded): 18 / (18 + 10)
= 18/28 = 0.643.** The threshold is 0.70, so T4's first clause **fails**. E1b Arm B scored
0.746 (44/59), but on a different population: the original module plus
`state_repository.py`, with per-line seed selection.

*Post hoc reading, labelled, not the preregistered measure:* five survivors are equivalent by
inspection, so excluding them gives 18/23 = 0.783.
- 4, 8, 11: annotation-only. They are never evaluated under `from __future__ import annotations`.
- 24: `len(set(c)) < len(c)` equals `!=` because a set is never longer than its source.
- 28: `strict` does not matter for two sequences of equal length.

Survivors 15 and 16 are equivalent only for the closed set of route kinds and for pydantic's
return of the literal object. Survivors 17, 21 and 29 are behavioural gaps of the acceptance:
- 17: a malformed `consumed:` reference in `compact_capture`.
- 21: a ledger without `currency` in `money_evidence_payload`.
- 29: lax pydantic validation of the captured JSON.

**Hand-written mutants: full acceptance, all caught (T4's second clause holds).**

| Mutant | Seconds (factor) | First divergence, verbatim |
| --- | ---: | --- |
| M1 ledger adopted before commit | 620.4 (1.48) | `FAILED tests/unit/test_run_slice_writers_db.py::test_live_failure_after_batch_insert_rolls_back_then_can_retry`: `AssertionError: assert {('prices', '...a8eef4db', 0)} == {}` |
| M2 legacy route skips reference validation | 601.4 (2.29) | `FAILED tests/unit/test_run_slice_writers_db.py::test_live_legacy_dictionary_cannot_bypass_reference_presence`: `Failed: DID NOT RAISE <class 'facdrone.shells.decision_input_slice_store.SliceConflictError'>` |
| M3 port, line 320, level 1→9 | 950.6 (1.67) | the seeds pass; replay `golden-trace.json S09-codecs-compact op 10 {"op":"compact","shape":"single",…}: /result/consumed_capture: recorded "eAG9Vttu4zYQ…"` |

### T5: verifier-gated solver differential, original vs candidate

| Check | dedup | capture | seal |
| --- | --- | --- | --- |
| (1) symbolic-ness, entries per side (orig / cand) | 41 / 40 | 8 / 8 (and 8 / 8 into `deduplicated_inputs`) | at 15 s: 1 / 1 (**fails**, `t5-smoke-seal-2`); at 60 s: `seal_evidence` 4 / 4, `seal_evidence_with_run_slices` 2 / 2 (`t5-smoke-seal-3`) |
| (2) soundness, 300 Hypothesis inputs, seed 20261104 | 300 agree | **151 of 300 differ**, all of the H4 shape: the same exception (`ValueError … nan/inf` or `TypeError … bytes`) on both sides, the original's absences `[AuditedAbsence('scoring', …)]`, the candidate's `[]` | 300 agree |

Session controls and realisability:

| Check | Result |
| --- | --- |
| (3a) planted mutant, `k_dedup` vs `c_dedup` | **counterexample** (`t5-ctl-kill-2`, 363.6 s) |
| (3b) equivalent variant, `e_tiny` vs `c_tiny` | **"No differences found. (attempted 16 iterations) All paths exhausted, functions are likely the same!"** (`t5-ctl-equiv-2`, 5.1 s) |
| Session | both controls hold, so the session is not void |
| (4) realisability | 3a and the capture counterexample both differ on concrete replay (`t5-cx-replay`) |

The capture pair's soundness gate as written ("agree on the corpus before solving") does not
pass. The disagreement is not an encoding fault: both drivers call the real functions with
identical arguments. It is the H4 behavioural difference itself, found by the corpus before the
solver ran. A capture solver verdict is therefore not gated by check (2) in the plan's sense.
Whether check (2) means encoding soundness (driver against function) or prior agreement of the
pair is a reading for the owner.

| Pair | `diffbehavior` outcome (verbatim) | Seconds (scaled limit, factor) |
| --- | --- | ---: |
| dedup | "No differences found. (attempted 1379 iterations) Consider increasing the --max_uninteresting_iterations option." **Inconclusive** (not exhausted) | 351.3 (312, 1.04) |
| capture | **Counterexample**, see below | 353.4 (348, 1.16) |
| seal | "No differences found. (attempted 511 iterations) Consider increasing the --max_uninteresting_iterations option." **Inconclusive** (not exhausted) | 342.0 (336, 1.12) |

Both inconclusive runs ran at factor < 1.5, so the rerun rule did not apply. There were no
timeouts and no reruns.

Counterexample `t5-ch-capture`, verbatim:

```
Given: (items=[(1, (0, 0, []), [], [[(2, -179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792, []), (6, 1, [0]), (1, 0, [])]])]),
  t5_drivers.o_capture : returns "(('raise', 'ValueError', 'Out of range float values are not JSON compliant: inf'), [AuditedAbsence(field='scoring', reason='policy returned before scoring')])"
  t5_drivers.c_capture : returns "(('raise', 'ValueError', 'Out of range float values are not JSON compliant: inf'), [])"
```

Concrete replay (`t5-cx-replay`), verbatim:
- `o_capture` returns `(('raise', 'TypeError', 'Object of type bytes is not JSON serializable'), [AuditedAbsence(field='scoring', reason='policy returned before scoring')])`;
- `c_capture` returns `(('raise', 'TypeError', 'Object of type bytes is not JSON serializable'), [])`;
- `differ: true`.

The input is one "previous slice object" item. No slice exists yet, so it builds one with the
cells `-n/4`, `b"x"` and `0`. Under the solver, the first cell evaluated to inf and the error
was a ValueError. Concretely `-n/4` is finite, so the bytes cell fails first. Either way both
sides raise the same exception, and they differ only in the mutation of `capture.absences`: the
original runs `capture.route(...)` before `deduplicated_inputs`, the candidate after
(review H4).

**Interpretation (fixed rule).** The candidate passed acceptance, so this realisable difference
is, by construction, one the acceptance did not catch.
- **Reachability.** `capture_payload` is public, and the input is within its type contract:
  `InputScalar` admits a float NaN or inf, and nothing excludes a non-JSON cell at runtime. The
  difference is observable only if the caller reads the capture object after the exception. The
  production caller `decision_evidence_payload.evidence_payload` (line 285) builds the payload in
  one expression and lets the exception propagate. I found no caller that reads
  `capture.absences` after a failed `capture_payload`, so the difference is reachable through
  the public function but not observed by any caller I read.
- **Oracle.** It cannot express this input. Its alphabet (`boot`, `restart`, `handover`,
  `commit`, `tamper`, `read`, `sim_persist`, `sim_supersede`, `seal`, `compact`, `r16`, plus the
  E1c/E1c-2 readers) has no operation that calls `capture_payload` on a caller-built capture.
  `vspec` builds only finite cells. Its observation model is the result or error plus tables and
  files, so it does not record an argument mutated after an exception, and both sides raise the
  identical error. A replay "as a new sequence" would need a new oracle operation. That is an
  extension of the frozen E1b/E1c-2 apparatus, which step 3 may not modify, so it was **not
  executed**.

**T5 reading.** The solver found one realisable counterexample that the acceptance missed
(capture, H4), and T5's prediction holds on that reading. Qualifications:
- the capture pair's check (2) did not pass as written, see above;
- dedup and seal ended inconclusive, with no exhaustion and no counterexample;
- the H1 one-shot iterator is outside the dedup domain by design (the reviewer's probe fact).

Solver time, all CrossHair processes, gate iterations included: smokes 142.1 s, controls 371.3 s,
pairs 1,046.8 s, **total 1,560.1 s**. Other T5 times: soundness 71.9 s over two runs, concrete
replay 2.8 s.

### Integrity

- `verify-mut` after the copy found all 4,851 files identical to the candidate.
- After the memory kill, `t4-restore-after-kill` restored mutant 16's file, and
  `verify-mut-after-kill` found all files identical.
- After T4, only two `.pyc` files differed, because `run_e3` writes bytecode. They were
  restored (`t4-restore-pyc`), and `verify-mut-after-t4-2` found all 4,851 files identical.
- Each mutant's restoration was verified by sha256 inside `t4_arm.py`. M1 to M3 were verified
  in `t4-restore-m*`.
- `t5_drivers.py` sets `sys.dont_write_bytecode`, so no `.pyc` is written into the frozen copy.

### Research code (physical lines)

| File | Lines | Status |
| --- | ---: | --- |
| step-0 apparatus (`surface`, `measures`, `run_e3`, `test_e3_replay`, `qdiff`, `step3`, `receipt3`) | 326 | unchanged, hashes in the facts |
| `t4_arm.py` | 70 | new: T4 sample, arm and mutant runner |
| `t5_drivers.py` | 149 | new: T5 drivers, controls, smoke, soundness corpus, replay |
| **total** | **545** | budget 400: **exceeded by 145** (`t4_arm.py` 70 + `t5_drivers.py` 149 = 219 against 74 remaining). Accepted by the coordinator as a reported fact; nothing was compacted |

`cosmic-ray-t4.toml` and `t4-m3-port.patch` are configuration and data and are not counted. The
scratch orchestration (`guard.py`, `verify_mut.py`, `t4_rest.ps1`, `t5_a.ps1`, `t5_b.ps1`) is
in the session scratchpad and is not apparatus.
