# E3b: sensitivity on the original (deconvolution-loop tasks 3.1 and 3.2)

Purpose: make the sensitivity of the acceptance comparable between candidate and original (R25).
E3's T4 measured it on the candidate's `decision_evidence_publication.py`. E1b used another seed
selection and sampled two modules, so its rate is not comparable. Here the same procedure runs on the
original's publication module. That means the same seed (20261104), 30 draws from the sorted spec keys,
the same arm (scope-import probe, R16 check, the 19 importing seed files with their 6 deselections, the
64-sequence replay `test_e3_replay.py`), the same deadline rule (scaled(max(120, 5 x unmutated seconds)),
one rerun when idle after a timeout at factor > 1.5), the same classification and the same
restore-and-verify. `t31_arm.py` is a versioned copy of the logic in `../e3-evidence-regeneration/t4_arm.py`.
It imports `FILES` and `REL` from that file; the rest is copied, with the roots parameterised. The
reference is `.runs/e2-facdrone/source`, frozen and only read, with the publication module at sha256
81f3448c…. The mutant copy is `.runs/e3-facdrone/orig-mut`, a `copytree(symlinks=False)` of it whose five
regenerated files are hash-verified.

Every heavy step runs `../guard.py` first, then goes through `receipt_atomic` into `results/receipt.json`
(logs in `results/logs/`) with OPENBLAS/OMP/MKL threads set to 1. The frozen replay writes its Level 2 file
into E3's `results/`. `arm` then moves it here as `results/sql-divergences-t31-<tag>.json`, so E3's
directory stays unchanged.

## Commands (PowerShell, cwd this directory)

```powershell
$PY = "D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe"
& $PY -m pytest test_t31_arm.py -q -p no:cacheprovider   # sampling rule, hunks + restore, classification
& $PY t31_arm.py prepare    # fact t31-apparatus (hashes); t31-copy (guarded); note t31-copy-verify
& $PY t31_arm.py cr-init    # t31-cr-init: uv tool run --python 3.13 --from cosmic-ray==8.7.0 cosmic-ray init cosmic-ray-t31.toml <runs>/t31.sqlite (cwd orig-mut)
& $PY t31_arm.py draw       # t31-sample: same uv tool, `python t31_arm.py sample <runs>/t31.sqlite`; note t31-sample-overlap
& $PY t31_arm.py base       # t31-base: unmutated arm, base 1200 s scaled
& $PY t31_arm.py dry 0 1    # t31-m0, t31-m1: notes t31-m<k>-pre, -deadline, -restore, -class
```

## Sample

The population is 307 specs (E3 candidate: 317). The seed is 20261104, the 30 draws are in
`results/t31-mutant-sample.json` and the file's sha256 is 4a255cb9…. Overlap with E3's sample is
informational only, because the modules differ. 7 operator names occur in both samples. No line number
and no (operator, line) pair occurs in both.

## Dry run (task 3.1), 2026-11-05

| Label | Exit | Seconds | Factor | Deadline (scaled) | Classification | Restore verified |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| t31-base | 0 | 125.6 | 1.87 | 2244.0 | passes unmutated (replay fresh, 64 sequences) | five files identical before the run |
| t31-m0 (AddNot, line 103) | 1 | 4.8 | 2.04 | 1281.1 (base 628.0) | killed: R16 harness differs | identical=true |
| t31-m1 (AddNot, line 192) | 1 | 25.1 | 1.78 | 1117.8 (base 628.0) | killed: `test_evidence_compaction_equivalence.py::test_every_fixture_day_and_route_reconstructs_exact_bytes_and_saves_stored_volume` | identical=true |

The two-mutant dry run is an apparatus check, not a kill-rate sample: no rate is computed or claimed from it. The rate under requirement 8.2 comes only from the full 30-mutant run of task 3.2 below.

## Full sample (task 3.2), executed 2026-11-05 and 2026-11-06

Owner go on 2026-11-05 ("continue the process as planned", menu answer "3.2 full original sample").
Driver `t31_arm.py dry 0 1 ... 29` (`results/driver-full.out`), base 125.6 s so deadline base 628.0 s
scaled per mutant. The series stopped after mutant 24 at 07:01 UTC (08:01 local) on 2026-11-05 when the session ended:
mutant 24's arm B had exited with 1073807364 (the process was terminated, not a test failure), so its
"killed" classification is void, and mutant 25 never started (`t31-m25-pre` verified, nothing after it).
On 2026-11-06 the receipt note `t31-resume-2026-11-06` records this and `t31_arm.py dry 24 25 26 27 28 29`
(`results/driver-resume.out`) completed the sample with the same sample file, deadline rule and
classification. The last `t31-m<k>-class` note per k is authoritative (mutant 24: survived, 143.5 s,
replay fresh with 64 sequences, every seed test passing). Mutants 0 and 1 also appear twice (dry run
and full run), with the same classification both times. No rerun after a timeout was needed (no `-rerun`
or `-wait` note; load factors of the authoritative runs 1.07 to 1.84, m0 reached 2.04 in the dry run). Before and after
every mutant the five files were hash-verified identical to the frozen original (34 `-pre` notes, since
`t31-m25-pre` was written by the interrupted series and again by the resume, and 33 `-restore` notes, all
`identical=true`). The receipt note `t31-resume-2026-11-06-correction` fixes the stop time given in the resume note.

| k | Line | Operator | Class | First failing reason |
| ---: | ---: | --- | --- | --- |
| 0 | 103 | AddNot | killed | R16 harness differs |
| 1 | 192 | AddNot | killed | `tests/contracts/test_evidence_compaction_equivalence.py::test_every_fixture_day_and_route_reconstructs_exact_bytes_and_saves_stored_volume` |
| 2 | 286 | AddNot | killed | R16 harness differs |
| 3 | 91 | And→Or | killed | `test_evidence_compaction_equivalence.py::…reconstructs_exact_bytes…` |
| 4 | 231 | BitOr→BitAnd | killed | R16 harness differs |
| 5 | 366 | BitOr→BitAnd (return annotation) | survived | |
| 6 | 191 | BitOr→FloorDiv (return annotation) | survived | |
| 7 | 436 | BitOr→Mod | killed | R16 harness differs |
| 8 | 191 | BitOr→Mul (parameter annotation) | survived | |
| 9 | 436 | BitOr→Mul | killed | R16 harness differs |
| 10 | 366 | BitOr→Pow (return annotation) | survived | |
| 11 | 220 | Mul→BitAnd (`*,` marker, `bind_evidence`) | incompetent | SyntaxError (INVALID scope) |
| 12 | 238 | Mul→BitOr (`*,` marker, `seal_evidence`) | incompetent | SyntaxError (INVALID scope) |
| 13 | 238 | Mul→RShift (`*,` marker, `seal_evidence`) | incompetent | SyntaxError (INVALID scope) |
| 14 | 238 | Mul→Sub (`*,` marker, `seal_evidence`) | incompetent | SyntaxError (INVALID scope) |
| 15 | 184 | Eq→IsNot (`resolved_inputs` consumed URI) | survived | |
| 16 | 393 | Eq→IsNot (`compact_capture` consumed URI) | survived | |
| 17 | 286 | Eq→LtE (`route.kind <= "scored"`) | killed | `tests/contracts/test_evidence_compaction_equivalence.py::…reconstructs_exact_bytes…` |
| 18 | 86 | NotEq→Eq | killed | R16 harness differs |
| 19 | 140 | NotEq→Gt (`parse_run_slice` digest) | survived | |
| 20 | 265 | NotEq→GtE | killed | R16 harness differs |
| 21 | 348 | NotEq→IsNot | killed | R16 harness differs |
| 22 | 91 | NotEq→Lt (`money_evidence_payload` currency) | survived | |
| 23 | 113 | NotEq→Lt | killed | `tests/test_study_slice_evidence.py::test_invalid_slices_are_refused_before_retention[value1]` |
| 24 | 125 | False→True (`allow_nan` in `_canonical`) | survived (2026-11-06 rerun; the 2026-11-05 run was terminated) | |
| 25 | 139 | Or→And (`parse_run_slice`) | killed | `tests/unit/test_run_slice_namespace.py::test_malformed_reference_refuses[run-slice:/aaaa…]` |
| 26 | 348 | Or→And | killed | R16 harness differs |
| 27 | 408 | Or→And | killed | R16 harness differs |
| 28 | 369 | Not deleted (`bound_payload`) | killed | R16 harness differs |
| 29 | 110 | ZeroIterationForLoop (`slice_bytes` rows) | killed | `tests/test_study_slice_evidence.py::test_invalid_slices_are_refused_before_retention[value1]` |

Mutant positions use cosmic-ray's row numbers for the module at sha256 81f3448c…; the operator row
numbers in `t31-mutant-sample.json` are 1-based. Four mutants of the sample fall on a `*` keyword marker
(line 220 in `bind_evidence`, line 238 in `seal_evidence`) and are syntax errors; the candidate's sample had two such.

**Rates (R22 convention, survivors in the denominator, incompetent excluded; bar 0.70, unchanged):**

| Arm | Killed | Survived | Incompetent | Rate | Bar 0.70 | Post hoc equivalent-excluded (labelled, not the result) |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| original (`e2-facdrone/source`, this series) | 17 | 9 | 4 | 17/26 = 0.654 | not met | 17/22 = 0.773 (4 annotation-only excluded) |
| candidate (`e3-facdrone/cand`, E3 T4) | 18 | 10 | 2 | 18/28 = 0.643 | not met | 18/23 = 0.783 (5 excluded, R25) |

`t32_summary.py` computes both arms from the receipts only (last class note per k here; last `t4-m<k>` or
`-rerun` command exit in E3's receipt, mapped as E3's README table: 0 survived, 2 incompetent, else
killed) and writes `results/t32-summary.json` (sha256 a824b29d…); `--check` reproduces it byte for byte.
`results/t32-survivor-readings.json` (hand-written, sha256 c633cd54…) classifies the nine survivors: 5, 6,
8, 10 are annotation-only (equivalent under `from __future__ import annotations`); 15, 16, 19, 22 and 24
are real gaps of the acceptance corpus (a malformed `consumed:` reference whose digest is present; a
run-slice id that differs from its digest and sorts below it; a ledger currency sorting above the row's;
a non-finite float in a canonicalised value). Two of the five real gaps (16, 22) sit at the sites of the
candidate's real survivors m17 and m21 in R25 and a third (15) repeats m17's condition in `resolved_inputs`; the fourth (line 140) is the site where the candidate's
m23 was killed only because `is` also fails the equal-id case.

Reading: the acceptance arm is about as sensitive on the original as on the candidate (0.654 against
0.643, one mutant apart on samples of 26 and 28 competent mutants). The candidate's failed T4 therefore
measures the arm's sensitivity on this module, not a loss introduced by the regeneration; it is not a
pass either, since neither arm meets 0.70. The difference between the two samples is within one kill and
no significance is claimed from thirty draws.
