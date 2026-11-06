# Task 5.1: integration check of the regenerator's t4-turn2 candidate on the E3 scope

Spec `.kiro/specs/deconvolution-loop`, task 5.1 (requirements 5.1-5.4, 6.4, 6.5). Written and
executed 2026-11-06. Everything here is recorded in `results/receipt.json` (commands, facts, notes;
logs under `results/logs/`). Nothing outside this directory was written except the formatter's
copy of the five files in `.runs/e4-regen/fmt-t4` (measures2's `--fmt-dir`).

**What this task does:** on the candidate tree that task 4's turn `t4-turn2` produced
(`.runs/e4-regen/cand-t4`), run the edit-scope hash check against the frozen original
(`.runs/e2-facdrone/source`), run the structure check with the turn's own declaration, and measure
the tree with the E3 measurers. **What it does not do:** no acceptance arm, no Facdrone tests, no
sensitivity, no behavioural claim. The t4-turn2 candidate is an untested regeneration; its behaviour
is unknown until an acceptance run, and a failing verdict here would have been a valid result. The
"meets" columns below are information against the preregistered E3 targets, not an assertion of the
structural prediction (requirement 5.5 is judged by a round controller, not by this task).

## Files

| File | Lines | Role |
| --- | --- | --- |
| `editscope.py` | 65 | edit-scope hash check (req 6.4, 6.5): hashes repo-root files and every file under `src`, `tests`, `packages` of both trees, skipping `__pycache__`, `.pytest_cache`, `.git`, `*.pyc`; `--mentions` scans the model's text for `*.py` tokens outside the scope |
| `test_editscope.py` | 59 | five pytest cases on tmp trees: identical, extra file, scope file differs only, file outside the scope differs, mentions; run red (module absent, exit 2) then green (5 passed) |
| `pureshare.py` | 37 | pure share by statement count (req 5.4), attributing every `ast.stmt` to its innermost def with the measurer's own `defs` |
| `structure-t4.json` | | `structure` extracted from `regenerator/results/turns/t4-turn2/result.json` (36 pure, 17 shell); sha256 `02ef9fa8…d850` |
| `results/editscope-t4.json` | | edit-scope result |
| `results/measures-t4.json` | | `measures.py` (as written) with the structure check |
| `results/measures-canonical-t4.json` | | `measures2.py`: as written plus ruff-format 0.17.3 canonical lines and statements |
| `results/measures-separated-t4.json` | | `measures3.py`: as written plus separated-form statements (rule version 1) |
| `results/pureshare-t4.json` | | pure share by statements, per-function statement counts |
| `results/receipt.json` | | commands, facts, notes (append-only, `receipt_atomic.py`) |

The E3 measurers were run unchanged from `../e3-evidence-regeneration` (read only; cwd there so
`from surface import SET` resolves; `PYTHONDONTWRITEBYTECODE=1`, no bytecode written there or in
the candidate tree: 0 `.pyc`, 0 `__pycache__` under `cand-t4` after the runs).

## Edit-scope check (req 6.4, 6.5): **pass**

Original `.runs/e2-facdrone/source` versus candidate `.runs/e4-regen/cand-t4`, repo-root files plus
`src/`, `tests/`, `packages/`:

- files hashed: 1,965 on each side, 1,965 compared; only in original: none; only in candidate: none
- differing: exactly the five scope files
  `src/facdrone/shells/{decision_evidence_publication,decision_evidence_reader,decision_evidence_store,decision_input_slice_store,decision_slice_publisher}.py`
- differing outside the scope: none; scope files unchanged: none
- `edit_scope`: pass (nothing added, removed or changed outside the scope); `verdict`: pass (the
  differing set is exactly the scope)
- the candidate's five hashes equal the five recorded in the regenerator receipt note
  `t4-turn2-candidate` (4d328cc0…, 27eba0f0…, bdefe6b6…, 842cb8d5…, fbf61509…), so the tree
  measured here is the tree task 4 recorded
- mentions: the model's response (`response.json`) names `publication.py`, `store.py`, `reader.py`,
  `slice_store.py`, `publisher.py` in its summarised thinking. These are its shorthand for the five
  scope files (each is the suffix of one scope file name and appears in sentences tallying the
  functions of "publication.py, store.py, reader.py and slice_store.py"); no file outside the scope
  is mentioned. `result.json` (files, change log) mentions only the five. The tool reports the bare
  tokens literally; the classification is this paragraph and receipt note `mentions-classification`.

Not covered by the walk: `docs/`, `scripts/`, `migrations/`, `conf/`, `research/`, `wiki/`, `bin/`,
`tmp/` and the dot-directories of the two trees. The task text prescribed `src`, `tests`, `packages`
and the repo-root files; `editscope.py --dirs` can widen the walk if a round controller needs the
whole tree (req 6.5 says every file outside the scope).

## Structure check (req 5.1-5.4): **pass**

Declaration: 36 pure, 17 shell, from the turn's own `result.json`. Result from `measures.py`:

- every one of the 53 defs in the five files is declared (undeclared: none; declared but not
  defined: none)
- 36 pure checks: 0 failures (no connection, session or engine parameter; no `.execute/.begin/
  .connect/.commit/.add/.flush/.merge/.delete` call)
- 12 write sites, all in declared shell functions (innermost enclosing def):

| Write site | Enclosing function (declared shell) |
| --- | --- |
| `decision_evidence_store.py` 246 `.execute(`, 247 `insert(`, 258 `.execute(` | `append_sealed_evidence` |
| `decision_evidence_store.py` 287 `.execute(` | `read_decision_evidence` |
| `decision_input_slice_store.py` 107 `.execute(`, 108 `insert(`, 113 `.execute(` | `append_slice` |
| `decision_input_slice_store.py` 125 `.execute(` | `_load` |
| `decision_input_slice_store.py` 151 `.add(` | `_walk` |
| `decision_slice_publisher.py` 99 `.execute(` | `SlicePublication.retained` |
| `decision_slice_publisher.py` 163 `update(`, 164 `.execute(` | `require_referenced_slices` |

- pure share by function count: 36 / 53 = **0.679**
- pure share by statement count (`pureshare.py`): 246 pure, 146 shell, 0 undeclared statements inside
  defs (392), 157 statements outside every def (imports, dataclass bodies, constants): **0.628** of
  the statements inside defs, 0.448 of all 549 statements. Requirement 5.4 names the share by
  statements but the preregistered E3 minimum (0.50) was stated by function count; both readings
  are given, the one over all statements is below 0.50 because module-level statements belong to
  neither class.

## Measures and comparison

Original = `../e3-evidence-regeneration/results/measures-*-baseline.json`; E3 round 1 =
`measures-round1.json`, `measures-canonical-round1.json`, `measures-separated-round1.json`; t4-turn2 =
`results/measures-*-t4.json`. Targets: E3 preregistration T2/T3 (`prototype-gate.md`). "Meets" is
information only.

| Measure | Original | E3 round 1 | t4-turn2 | Target (T2/T3) | t4-turn2 meets |
| --- | ---: | ---: | ---: | --- | --- |
| lines, as written | 1,274 | 1,074 | 1,214 | ≤ 1,083 | no (+131) |
| lines, canonical (ruff format 0.17.3) | 1,274 | 1,188 | 1,210 | ≤ 1,083 | no (+127) |
| statements, as written (`ast.stmt`) | 660 | 546 | 549 | ≤ 561 | yes |
| statements, canonical | 660 | 546 | 549 | ≤ 561 | yes |
| statements, separated form (rule v1) | 718 | 627 | 610 | ≤ 561 if read on this form | no (+49) |
| functions (defs) | 67 | 51 | 53 | ≤ 67 | yes |
| call edges | 84 | 60 | 63 | ≤ 67 | yes |
| module hops | 7 | 7 | 7 | none | |
| write sites | 13 | 11 | 12 | ≤ 13, all in declared shell | yes (12, all in shell) |
| pure share by functions | n/a | 0.706 (36/51) | 0.679 (36/53) | ≥ 0.50 | yes |
| pure share by statements in defs | n/a | not computed here | 0.628 | none preregistered | |
| structure check | n/a (self-test only) | pass, 0 undeclared | pass, 0 undeclared, 0 failures | passes | yes |

Separated-form extras for t4-turn2: from-import 58, tuple assignment 3, import 0, multi-target 0,
chained compare 0 (round 1: 58, 16, 7, 0, 0; original: 56, 2, 0, 0, 0). Canonical formatting
changes 4 lines (1,214 to 1,210) and no statement, so packing is not what the line figure rests on.

Per file (lines / statements / functions / write sites), as written:

| File | Original | E3 round 1 | t4-turn2 |
| --- | --- | --- | --- |
| `decision_evidence_publication.py` | 442 / 207 / 22 / 1 | 359 / 159 / 19 / 0 | 427 / 163 / 20 / 0 |
| `decision_evidence_store.py` | 313 / 171 / 20 / 4 | 280 / 145 / 13 / 4 | 298 / 145 / 13 / 4 |
| `decision_evidence_reader.py` | 102 / 42 / 5 / 0 | 82 / 39 / 4 / 0 | 82 / 37 / 3 / 0 |
| `decision_input_slice_store.py` | 239 / 143 / 14 / 5 | 207 / 124 / 11 / 5 | 225 / 121 / 12 / 5 |
| `decision_slice_publisher.py` | 178 / 97 / 6 / 3 | 146 / 79 / 4 / 2 | 182 / 83 / 5 / 3 |

Reading, for information: the t4-turn2 candidate is close to the E3 round-1 candidate on
statements, functions and edges, is longer in lines (1,214 against 1,074 as written; the publication
module alone is 68 lines longer), and like round 1 misses the line bar and the separated-form
statement bar while meeting the statement, function, edge, write-site and pure-share bars. None of
this says anything about behaviour.

## Provenance note on the turn files

The five candidate hashes match the regenerator receipt. The two turn files that this task read,
`regenerator/results/turns/t4-turn2/result.json` and `response.json`, hash on disk to
`8655a97b…9b7b` and `ea216ab0…9cce`, which differ from their `.sha256` sidecars and the receipt
note (`396a5179…`, `05e7b990…`). The difference is line endings only: the files on disk have CRLF
(94 and 37 line ends; written in Windows text mode), and the sha256 of the same bytes with LF
endings equals the sidecar value for both files (checked, receipt fact `turn-file-hashes`). So the
on-disk `result.json` is the recorded one, modulo newline translation; the structure extracted
from it (36/17) agrees with the note. Files under `regenerator/` were not touched by this task.

## Not claimed

- No acceptance run, so no claim about the 64 golden sequences, the seed tests or Level 1/Level 2
  divergence; the candidate's behaviour is unknown.
- No sensitivity sample, no kill rate, no reviewer step, no differential pairs.
- The structural prediction (req 5.5) is not asserted; the structure check and the measures are
  reported as recorded.
- The edit-scope walk covers the repo root, `src`, `tests` and `packages`; other top-level
  directories were not hashed.

## Commands

`$PY` = `.runs/e2-facdrone/venv/Scripts/python.exe` (3.13.16), `$RA` =
`docs/experiments/deconvolution/receipt_atomic.py`, `$R` = `t51-integration/results/receipt.json`,
`$T` = `docs/experiments/deconvolution/t51-integration`, `$E3` = `../e3-evidence-regeneration`,
`$C` = `.runs/e4-regen/cand-t4`, `$O` = `.runs/e2-facdrone/source`, `$D` =
`regenerator/results/turns/t4-turn2`; every `run` had `--env PYTHONDONTWRITEBYTECODE=1 --env
PYTHONIOENCODING=utf-8 --no-probe` (light work, no host guard). The receipt holds the exact argv.

```
& $PY $RA $R run test-editscope-red   --cwd $T  -- $PY -m pytest test_editscope.py -q -p no:cacheprovider   # exit 2, module absent
& $PY $RA $R run test-editscope-green --cwd $T  -- $PY -m pytest test_editscope.py -q -p no:cacheprovider   # 5 passed
& $PY $RA $R run extract-structure    --cwd $T  -- $PY -c "<json.loads($D/result.json)['structure'] -> structure-t4.json; print hashes>" $D\result.json
& $PY $RA $R run editscope-t4         --cwd $T  -- $PY editscope.py $O $C --scope <the five> --mentions $D\result.json $D\response.json --out results\editscope-t4.json
& $PY $RA $R run measures-t4          --cwd $E3 -- $PY measures.py $C --structure $T\structure-t4.json --out $T\results\measures-t4.json
& $PY $RA $R run measures-canonical-t4 --cwd $E3 -- $PY measures2.py --root $C --structure $T\structure-t4.json --fmt-dir .runs\e4-regen\fmt-t4 --out $T\results\measures-canonical-t4.json
& $PY $RA $R run measures-separated-t4 --cwd $E3 -- $PY measures3.py --root $C --structure $T\structure-t4.json --out $T\results\measures-separated-t4.json
& $PY $RA $R run pureshare-t4         --cwd $T  -- $PY pureshare.py $C --structure structure-t4.json --out results\pureshare-t4.json
```
