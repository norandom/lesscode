# E1c-2: corrected solver rung and the oracle's missing readers

Preregistration and amendments: [plan.md](plan.md). Amendment 1 sets the smoke harness and the
S5 detection rule, records the step 2 choice (`json.loads`) and extends the same non-interned
domain to 240. Amendment 2 is the owner's solver-encoding verifier. Amendment 3 is the owner's
adaptive budgets. Amendment 4 covers the control launcher and the 584 realisation. S1 to S5 were
not changed.

**Status (2026-11-04): executed, native Windows, plain user account.** The research entry under
`docs/research/` has not been written.

## Results

### Predictions

| | Prediction | Measured | Verdict |
| --- | --- | --- | --- |
| S1 | counterexample or exhausted no-difference verdict for ≥ 7 of 10 | **4 of 10**: counterexamples for 418, 584, 240 and 241. The other six (150, 166, 169, 186, 200, 642) ended "No differences found" **without** "All paths exhausted", so they are inconclusive | **fails** |
| S2 | ≥ 6 of the 8 covered survivors killed (realised solver input or extended alphabet) | **5 of 8**: 150, 186, 418 and 642 by the extended alphabet alone, 584 by a realised solver input. 166, 169 and 200 survived | **fails** (by one) |
| S3 | ≥ 1 proved equivalent within bounds, and 642 among the proved or the killed | 642 is **killed** (the `LIMIT` parameter is recorded). **0** proved equivalent | **fails** (first clause) |
| S4 | `state_repository.py` branch coverage ≥ 84 % under Arm B plus the new sequences | 45/56 = 80.36 % → **47/56 = 83.93 %** (statements + branches 92.86 % → 93.88 %) | **fails** (by 0.07 points; 48/56 would be needed) |
| S5 | every driver smoke enters `lifted.py`; no void runs | **16 of 16** smokes enter (14 to 255 entries each) | **holds** |

**Kill criterion.** S1 and S2 both fail while S5 holds. Under the plan, the guided-generation claim
is **rejected for database-entangled predicates on this scope**, this time with symbolic drivers
that verifiably reach the lifted code and a solver pipeline that passes both controls. E3 must
declare the residue unprotected: 166, 169 and 200 under this method, and every other target the
oracle could not reach.

The verdict needs two qualifications. First, the solver did contribute: 3 of the 7 kills (584,
240, 241) come only from realised solver inputs, and all three are identity or ordering mutants
that no reader call alone exposes. Second, the extended alphabet alone kills 4 targets,
including 642, which E1c could only call equivalent by inspection. The 3 survivors all need a
joint condition: a matched attempt row, a history entry and one instruction identity (166, 169),
or two legacy-matched rests (200). The solver did not find these in 300 s.

### Per target

Smoke: `crosshair check` with the trivial postcondition, 15 s, entry markers (Amendment 1).
CrossHair: `diffbehavior`, `--per_condition_timeout` 300 s (fixed for 150 and 166; `scaled(300)`
from 169 on, Amendment 3). Arm B: E1b's `run_arm.py` (R16 check, the selected seed files, then
`test_e1c2_replay.py`), `-x`, deadline `scaled(E1b measured deadline)`. Each mutant was applied in
`.runs/e1c-facdrone/mut` and restored afterwards.

| Line | Operator | Smoke (entries) | CrossHair outcome (wall s, factor) | Realised | Arm B (E1c-2 seqs) | Classification |
| ---: | --- | --- | --- | --- | --- | --- |
| 150 | Eq→LtE, SQL `strategy_id <=` | m150 97 | no differences, 2,006 it., not exhausted (310.7, —) | — | **killed**, 89.7 s: `R-S01 op 0 readers /sql[2][0]` (query text) | killed by extended alphabet alone |
| 166 | And→Or | m166 76 | no differences, 2,336 it., not exhausted (307.6, —) | — | survived, 60.9 s | unresolved: inconclusive solver run; needs an attempt row, an `open` history entry and a single joined identity at once |
| 169 | Eq→LtE, `representation <=` | m169 65 | no differences, 2,723 it., not exhausted (366.7, 1.16) | — | survived, 63.6 s | unresolved: same joint condition, plus a non-`legacy` representation below `"legacy"` |
| 186 | Eq→IsNot, SQL account predicate | m186 81 | no differences, 2,697 it., not exhausted (340.9, 1.11) | — | **killed**, 77.5 s: `R-S01 op 0 readers /sql[3][0]` | killed by extended alphabet alone |
| 200 | Eq→GtE, `len(matched) >= 1` | m200 100 | no differences, 2,354 it., not exhausted (350.2, 1.13) | — | survived, 33.6 s | unresolved: needs two rests that the same attempt matches |
| 418 | Eq→GtE, SQL `rank >= 1` | m418 225 | **counterexample** (343.9, 1.10) | C418 | **killed**, 104.0 s: `R-S01 op 0 readers /sql[0][0]` | killed by extended alphabet alone (the realised C418 also distinguishes; the replay stops at the first divergence) |
| 584 | Eq→Is, `kind is "set_stops"` | m584 179 | **counterexample** (399.9, 1.28) | C584 | **killed**, 86.7 s: `C584 op 1 commit_instr /sql[2][1]/payload_json/intention_ids (length 1 recorded vs 0 replayed)` | **killed by realised solver input** |
| 642 | NumberReplacer, `limit(2)` | m642 144 | no differences, 2,988 it., not exhausted (378.4, 1.21) | — | **killed**, 41.7 s: `R-S01 op 0 readers /sql[5][1]/param_1` | killed by extended alphabet alone (semantically equivalent by inspection; the SQL parameter differs) |
| 240 | Eq→Is, `representation is "legacy"` | m240 109 | **counterexample** (402.9, 1.30) | C240 | **killed**, 34.7 s (replay only): `C240 op 1 readers /result/resting_orders/01E1C…01 (present on one side only)` | **killed by realised solver input** |
| 241 | Eq→LtE, `rest.parent <=` | m241 14 (exit 1: mutant `TypeError`) | **counterexample** (349.2, 1.13) | C241 | **killed**, 32.6 s (replay only): `C241 op 1 readers /result/resting_orders/01E1C…02 (present on one side only)` | **killed by realised solver input** |

Totals: killed by realised solver input **3** (584, 240, 241); killed by the extended alphabet
alone **4** (150, 186, 418, 642); proved equivalent **0**; unresolved **3** (166, 169, 200).
Every Arm B kill failed only in `test_e1c2_replay` (all seeds passed).

Counterexamples (verbatim in `results/logs/ch-*.log`):

- 418: `rows=[(None,None,2,0,None,0), (None,None,2,0,None,0), (None,None,2,0,[],0), …]`. That is two
  instruction rows without a key and one with the empty key, all version 0. The original returns
  `[0, 0]` and the mutant `[0, 0, 0]`. Only the first three rows are within the bound.
- 584: `cands=[(3,[0,0],[0]*8), None, (3,[0,0],[0]*8)]`. Each dict has kind `"set_stops"` decoded from
  JSON and id `"aaaaaaaa"`. The original returns both ids and the mutant `[]`.
- 240: `rest=(3,[0],[],[])`,
  `body=(limit_order, unavailable, None, None, None, rep 1 = "legacy", …, parent_id '', venue '')`.
  The original returns True and the mutant False.
- 241: `rest=(2,[],[],[])`, `body=(market_order, accepted, …, "legacy", turn_id None, parent_id 'a', venue '')`.
  `''` is not equal to `'a'`, but `'' <= 'a'`, so the original returns False and the mutant True.

### Realisation mapping

| Solver value | Realised | Why |
| --- | --- | --- |
| 418 instruction token 0 | `instr(0, "open", "p", "v")` (ULID `01E1C0…0`) | the original validates the body |
| 418 key `None` / `''` | payload without `key` / `"key": ""` | kept exactly: JSONB holds both; `->>` gives NULL and `''` |
| 584 candidates | one wire-decoded (`json.loads(json.dumps(…))`) `set_stops` instruction, id ULID `…0`, after `boot` | `instruction_records` validates every candidate and derives row ids from the id, so the `None` candidate and the duplicate id would fail the commit identically on both sides (Amendment 4) |
| 240/241 strings `s` | `"p" + s`: `''` → `"p"`, `'a'` → `"pa"` | `parent` must be non-empty; the common prefix keeps equality and order |
| 240/241 rest id | ULID `…1` / `…2` | `InstructionPayload.id` pattern |
| fields the enclosing `resting_orders` needs | rest kind `rest`, `child_kind = limit_order`, `order_id = o1`, no outcome | it filters on them before it calls `_placed_under` (240's solver outcome `unavailable` is skipped by the reader, so it is dropped) |
| row decision ids | `e1c:0`, `e1c:1`, … | primary key |

### Oracle alphabet extension

`e1c2_source.py` adds one operation, `readers`. It calls `opening_tickets` and
`resting_orders` with the stored `instruction_history` or a given history, `attempt_history`,
and `decision_served` for every stored decision id plus an absent one. Each reader's failure is a
separate observation. `e1b_oracle.py` and `e1c_source.py` are imported and unchanged. The
sequences are:

- the readers on the 7 live E1b seed states (`R-S01` … `R-S06`, `R-S11`, via `start`);
- the 4 realised counterexamples (`C418`, `C584`, `C240`, `C241`).

`results/golden-e1c2.json` holds 11 sequences, 17 operations and no errors. The body sha256 is
`034f0f3113e664a6040816689b25c522b39de67d814e887c2b9ddc383b25dabb`, the digest written to
`.sha256` (on-disk bytes differ by CRLF). It was recorded from the frozen copy
(`scope.changed = []`). Replay on the frozen copy reports **"replay: no divergence"**
(`replay-e1c2-frozen`).

## Verifier (Amendment 2)

| Check | Result |
| --- | --- |
| 1 Symbolic-ness: > 1 explored path into `lifted.py` per driver smoke | holds: entries per smoke o_open 66, m150 97, m166 76, m169 65, o_rest 63, m186 81, m200 100, o_placed 99, m240 109, m241 14, o_hist 255, m418 225, o_intent 217, m584 179, o_served 150, m642 144 |
| 2 Encoding soundness: lifted = original on every recorded input | holds: `lifted.py` sha256 `1106c384…4840` = E1c's; `check_r5.py` rerun gives 472 calls, 285 distinct inputs, **0 mismatches** (`r5-recheck`) |
| 3a Control: 418 yields a counterexample under `drivers2` | holds (`ch-418`) |
| 3b Control: an annotation-only mutant yields "No differences found" **and** "All paths exhausted" | holds: `decision_evidence_publication.py:191`, `-> str \| None` → `-> str - None`, compiled in memory from the frozen text. Result: "No differences found. (attempted 4 iterations) All paths exhausted, functions are likely the same!" (`ctl-191-3`). Two earlier attempts failed to load the module (import side effects, Amendment 4) and ran no path |
| 4 Realisability: each counterexample differs when replayed concretely | holds for all 4 (`cx-replay`): 418 `[0, 0]` vs `[0, 0, 0]`; 584 two ids vs `[]`; 240 True vs False; 241 False vs True |
| Inconclusive runs | 150, 166, 169, 186, 200, 642: "No differences found" without exhaustion, recorded as inconclusive, never as equivalence |

The solver session is **not void**: both controls passed.

## CrossHair, bounds and time

- `crosshair-tool==0.0.112`, `z3-solver==5.3.3.0` (`.runs/e1c-facdrone/extras`), CPython 3.13.16,
  `PYTHONPATH=<extras>;<e1c2 dir>`, thread variables 1, default options except
  `--per_condition_timeout`.
- Bounds (`drivers2.py`), all imposed by construction, with no string slicing and no rejection:
  - a string is `"".join(chr(97 + c % 3) …)` over at most 8 elements of a symbolic `list[int]`
    (`zip(range(8), …)`), so the alphabet is `a b c`, the length is 0 to 8, and the characters stay
    symbolic (CrossHair's `_join` realises only the length);
  - lists hold at most 3 elements;
  - enumerations are int indexes modulo: flow, child kind, outcome, instruction kind;
  - `representation` (index over None, `"legacy"`, `"instructions"`, or a free string) and the
    candidate `kind` are decoded with `json.loads('"' + v + '"')`, so they are never the interned
    literal; CrossHair symbolises through its pure-Python `jsonlib`;
  - identity columns are None (the oracle's identity) or a free string;
  - `sl` is `Optional[int]` cast to float.
- The adaptive factor ranged from 1.10 to 1.30 for the solver runs. No timeout occurred at
  factor > 1.5, so nothing was rerun. 150 and 166 ran before Amendment 3; a probe during 166 gave
  1.16.

| Solver time | Seconds |
| --- | ---: |
| Smokes and harness trials (19 runs) | 295.2 |
| `ch-150`, `ch-166` (fixed 300 s) | 618.3 |
| `ch-169` … `ch-241` (scaled 330 to 390 s) | 2,932.0 |
| Control runs (`ctl-191`, `-2`, `-3`) | 12.1 |
| **Total** | **3,857.7** (budget 5,400 s base) |

Other times: record 24.7 s, frozen replay 21.5 s, R5 recheck 1.5 s, concrete counterexample replay
4.3 s, ten Arm B runs 625.0 s (32.6 to 104.0 s), coverage run 60.7 s.

## Research code (physical lines, `wc -l`)

| File | Lines | Role |
| --- | ---: | --- |
| `drivers2.py` | 130 | drivers, smoke function and entry markers |
| `e1c2_source.py` | 68 | `readers` operation, sequences, record/replay |
| `run_e1c2_arm.py` | 13 | Arm B with the E1c-2 replay target |
| `receipt2.py` | 9 | E1c's receipt rebound to this directory |
| `test_e1c2_replay.py` | 5 | Arm B pytest target |
| *subtotal, preregistered scope* | *225* | *budget 250: within* |
| `ctl2.py` | 33 | Amendment 2 control (b) and concrete counterexample replay |
| `step2.py` | 40 | Amendment 3 probe, scaled deadline, rerun-once rule |
| **Total** | **298** | **budget 250: exceeded by 48**, all of it code for the two owner amendments |

No separate classification script was written; the classification is in the receipt note
`results` and in the table above. E1c modules are reused by import or path
(`lifted.py`, `e1c_source.py`, `mutate.py`, `receipt.py`, `run_e1c_arm.py`, `check_r5.py`). They
are unchanged: their sha256 values equal E1c's receipt facts.

## Commands (PowerShell; full argv, env, UTC times, exits and adaptive probes in `results/receipt.json`)

```powershell
$V="D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe"   # cwd = this directory
# S5 smokes
& $V receipt2.py run smoke-<fn> --env PYTHONPATH=<extras;e1c2> --env E1C2_SMOKE=<fn> -- $V -m crosshair check --per_condition_timeout 15 drivers2.smoke
# solver (150, 166 via receipt2.py with 300; the rest via step2.py)
& $V step2.py ch-<L> 300 --env PYTHONPATH=<extras;e1c2> -- $V -m crosshair diffbehavior --per_condition_timeout "{T}" drivers2.<o_fn> drivers2.m<L>
# verifier
& $V step2.py ctl-191-3 300 ... -- $V -c "import sys, ctl2; from crosshair.main import main; sys.argv=['crosshair','diffbehavior','--per_condition_timeout',sys.argv[1],'ctl2.o_ctl','ctl2.m_ctl']; sys.exit(main())" "{T}"
& $V receipt2.py run cx-replay ... -- $V ctl2.py replay results/logs/ch-418.log ... ch-241.log
& $V step2.py r5-recheck 600 --env PYTHONPATH=<e1c> -- $V <e1c>/check_r5.py <e1c>/results/r5-capture-seeds.jsonl
# oracle
& $V step2.py record-e1c2 600 --env PYTHONPATH=<e1c2;e1c;e1b;e1b extras;e2> -- $V e1c2_source.py record
& $V step2.py replay-e1c2-frozen 600 ... -- $V e1c2_source.py replay
# Arm B, one target at a time
foreach L: apply-<L>: <e1c>/mutate.py apply <mut> <L> ; armB-<L>: step2.py armB-<L> armB:<mut> --cwd <mut> -- $V run_e1c2_arm.py B --deadline "{T}" ; restore-<L>: mutate.py restore <mut>
# S4
& $V step2.py e1c2-cov-frozen 600 --cwd <frozen> --env E1B_SEEDS= -- $V run_e1c2_arm.py B --cov results/cov-e1c2
copy E1b results/cov-b/.coverage and the cov-e1c2 data file into results/cov-s4 ; coverage combine --keep ; <e1b>/run_arm.py report results/cov-s4 <frozen>
```

## Deviations and incidents

1. **Amendment 1.** The smoke harness needs the postcondition in the source. The first trial
   (`smoke-o_placed`, 1.5 s) found no checkable function. Its log was later overwritten by the
   series smoke of the same label; both receipt entries remain. The verbose trial
   (`smoke-o_placed-2`) showed that missing decision frames do not imply missing entry, so S5 is
   scored by entry markers. 240 received the same `json.loads` domain as 584.
2. **Amendment 2** (owner, during the solver series, before any verdict was read): the verifier.
3. **Amendment 3** (owner, after `ch-166`): adaptive budgets. The series loop was stopped
   between `ch-166` and `ch-169` (no partial run) and resumed through `step2.py`.
4. **Amendment 4.** Control launcher (two load failures without paths), and the minimal 584
   realisation.
5. No `wsl.exe` and no bare `bash` were invoked from Python or PowerShell. All research commands
   ran from PowerShell with the venv interpreter. Git Bash was used only interactively by the
   agent for file inspection and file writing.
6. Scratch check outside the receipt (Amendment 1): about 15 s on the embedded database, which
   confirmed that DB-loaded strings are not the interned literal.

## Limitations

1. **Inconclusive is not equivalent.** Six runs stopped at the time limit without exhausting
   paths. Within these bounds (alphabet `abc`, length ≤ 8, ≤ 3 rows) exhaustion is out of reach
   for every target driver: the path count grows with every string length. Only the deliberately
   tiny control exhausted.
2. **Kills by query text.** 150, 186, 418 and 642 die because the oracle records SQL text and
   parameters. 642's mutant is semantically equivalent (primary-key lookup, `.first()`), so its
   kill is a representation-level detection. That fits the oracle's design but is not evidence
   about predicate semantics.
3. **Identity mutants depend on string origin.** 584 and 240 are killable because JSON- and
   DB-decoded strings are fresh objects. A caller that builds `{"kind": "set_stops"}` from a
   Python literal would not expose 584. The realised C584 uses the wire path
   (`commit_instr` with `wire`).
4. **Minimal realisation for 584 and 240.** Parts of the solver input that would make the
   enclosing code fail identically on both sides, or skip the predicate, were dropped. Every
   realised sequence still distinguishes in the oracle.
5. **SQL modelled in Python** in `lifted.py`, as in E1c (code-point order for 150; collation not
   exercised). R5 still holds on the 472 seed calls.
6. **Arm B runs the seeds first with `-x`.** No seed failure occurred. Every kill came from the
   replay.
7. **Adaptive scaling** applied only from `ch-169` on. 150 and 166 ran with a fixed 300 s; a
   probe during 166 shows no load (factor 1.16).

## Integrity

- Frozen copy `.runs/e2-facdrone/source`: `scope.record` reports `changed: []` after all runs.
  The mutant copy `.runs/e1c-facdrone/mut` reports `changed: []` after the last restore and after
  every restore (`restore-*.log`).
- `D:\Source\facdrone` HEAD `d49f48bd13c4e3da8a219d5bf73a974a9f982999`; `git status --short` is
  empty.
- `e1b_oracle.py` sha256 `c4c93118…`, and the E1b golden trace (`92a48f77…`) and `golden-e1c.json`
  (`c69cb957…`) are untouched. `lifted.py`, `e1c_source.py`, `mutate.py`, `receipt.py` and
  `run_e1c_arm.py` hash to E1c's receipt facts.
- Mutants existed only in `.runs/e1c-facdrone/mut`, one at a time, each applied as cosmic-ray's
  exact diff line and restored by copying the frozen file back. The control's annotation mutant
  existed only in memory.

## Files

| File | Role |
| --- | --- |
| `plan.md` | preregistration and amendments 1 to 4 |
| `results/receipt.json` | every command (argv, cwd, env, UTC times, seconds, exit, adaptive probe and factor), notes `verifier`, `solver-time`, `results`, `integrity`, facts (versions, hashes, line counts) |
| `results/logs/` | output of every recorded command (`smoke-*`, `ch-*`, `ctl-191*`, `cx-replay`, `r5-recheck`, `record-e1c2`, `replay-e1c2-frozen`, `apply-*`, `armB-*`, `restore-*`, coverage) |
| `results/golden-e1c2.json` (+ `.sha256`) | the original's observations of the E1c-2 sequences |
| `results/cov-e1c2/`, `results/cov-s4/` | coverage of the E1c-2 sequences alone, and combined with E1b Arm B (S4) |
