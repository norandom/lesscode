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

The full sample (task 3.2, mutants 0 to 29 with the kill rate killed / (killed + survived)) has **not
run**. It needs the owner's go.

The two-mutant dry run is an apparatus check, not a kill-rate sample: no rate is computed or claimed from it. The rate under requirement 8.2 comes only from the full 30-mutant run of task 3.2, which needs the owner's go.
