# R149: The E3 and E4 figures that can be rerun reproduce on the candidates; the mutation runs, the solver step and the model turns were not rerun

- Report date/time and timezone: 2026-11-18, 13:25, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 13:12 to 13:23 (from the clock and the run logs)
- Status: completed
- Spec: [deconvolution-loop](../../../.kiro/specs/deconvolution-loop/requirements.md); task 1.2 of the [corrective tasks](../../../.kiro/corrective-tasks.md)
- Hypothesis ID and predecessor: audits [R25](../2026-11-04/02-e3-evidence-regeneration.md) (E3) and [R26](../2026-11-06/01-e4-lifecycle-regeneration.md) (E4) the way [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md) audited E5
- Type: retrospective audit with reruns; no model call

## Decision

Every figure of E3 and E4 has a script in the experiment directory that takes the candidate's root as
an argument. No figure was found that came from code written inside a script. What was rerun today
reproduces, mostly byte for byte. Three kinds of figure were not rerun: the mutation runs, E3's solver
step and the model turns. They are marked as such and not as confirmed.

This differs from E5 and E6, where R134 found the equivalence and mutation figures unsupported.

| Trial | Figure | Reported | Produced by | Ran on the candidate? | Today |
| --- | --- | --- | --- | --- | --- |
| E3 | Replay on the acceptance corpus | 64 sequences, 311 operations, 0 differences at either level | `run_e3.py` with `test_e3_replay.py` | yes, `.runs/e3-facdrone/cand`, with an import probe | **reproduces** |
| E3 | Seed tests | root 1,403 passed, 2 skipped, 10 deselected; research 39, 3, 6; reporting 92 passed, 8 deselected, 1 expected failure | `run_e3.py` | yes | **reproduces**, same counts |
| E3 | Size as written | lines 1,274 → 1,074; statements 660 → 546; functions 67 → 51 | `measures.py` | yes | **reproduces** (`measures3.py --check`, both trees) |
| E3 | Structure | call edges 84 → 60; write sites 13 → 11, all in declared shells; 36 of 51 functions pure | `measures.py` with `candidate-structure.json` | yes | **reproduces** (same check) |
| E3 | Lines after formatting | 1,274 → 1,188 | `measures2.py` | yes | **reproduces**, file identical |
| E3 | Statements in separated form | 718 → 627 | `measures3.py` | yes | **reproduces**, file identical |
| E3 | Sensitivity (T4) | 18 of 28 killed, 0.643; failed its bar | `t4_arm.py`, 30 mutants of the candidate's publication module | yes; the sample records the module's hash, which matches the file today | **not rerun** |
| E3 | Baseline sensitivity | 0.654 | `t31_arm.py` of a neighbouring experiment | original tree | **not rerun** |
| E3 | Solver comparison (T5) | failed; 151 of 300 capture cases | `t5_drivers.py` | yes | **not rerun**; the kept output matches its recorded hash |
| E4 | Seed tests | 108 passed, 1 skipped, 1 deselected | `run_e4.py` | yes, `.runs/e4-lifecycle/cand-turn3`, with an import probe | **reproduces** |
| E4 | Replay on the acceptance corpus | 50 sequences, 317 operations, 0 differences | `run_e4.py` with `test_e4_replay.py` | yes | **reproduces** |
| E4 | Replay on the held-out corpus | 74 sequences, 0 differences at either level | `e4_oracle.py replay` | yes | **reproduces**: no difference; the corpus matches its recorded hash; the command does not print the sequence count |
| E4 | Size | lines after formatting 1,495 → 1,405; statements 759 → 720; functions 69 → 52 | `measures4.py` | yes | **reproduces**, six files identical |
| E4 | Structure | call edges 89 → 63; write sites 12 → 11; 27 of 52 functions pure (0.519) | `measures4.py`, `pureshare4.py` with `structure-e4-t3.json` | yes | **reproduces**, files identical |
| E4 | Patch-point check | all checks pass | `patchcheck.py` | yes | **reproduces**, file identical |
| E4 | Reviewer probes | 128 probes per tree; seven differences on error paths | `review_probes.py` | yes, one run per tree | **probe outputs reproduce**, both files identical; the count of seven is a reading in the review and was not recomputed |
| E4 | Sensitivity | candidate 19 of 28, 0.679; original 12 of 29, 0.414 | `e4_arm.py` on copies of each tree, rates by `e4_rates.py` | yes, a copy of `cand-turn3` verified by hash | **rates recompute identically** from the kept receipt; the mutant runs were **not rerun** |
| E4 | Missing-test report | 28 entries | `missing_tests4.py` | reads both trees | **not rerun** |
| E3, E4 | Turns and tokens | E3 one round; E4 accepted at turn 3 | model calls | not applicable | **not rerun**; the requests are kept |

## Hypothesis and falsification

The question of task 1.2: does any E3 or E4 figure rest on code that never touched the candidate, as
E5's and E6's did? A figure whose script takes no candidate root, or whose rerun differs, would
answer yes. None did.

## Frozen inputs and method

- Repository at `ba340fb`. Baseline tree `.runs/e2-facdrone/source`; candidates `.runs/e3-facdrone/cand` and `.runs/e4-lifecycle/cand-turn3`.
- Interpreter `.runs/e2-facdrone/venv/Scripts/python.exe`, CPython 3.13.16, with `PYTHONDONTWRITEBYTECODE=1` and the numeric thread pins of the original runs.
- E3: `measures3.py --check results/measures-baseline.json --root <baseline>`; `measures3.py --check results/measures-round1.json --root <cand> --structure candidate-structure.json`; `measures2.py` and `measures3.py` with `--out` into the evidence directory; `run_e3.py <cand> --round a1118`.
- E4: `measures4.py` for `baseline` and `e4-t3`; `pureshare4.py`; `patchcheck.py`; `e4_rates.py --check`; `run_e4.py <cand-turn3> --round a1118`; `e4_oracle.py replay --golden .runs/e4-lifecycle/held-out-a.json` with the candidate first on `PYTHONPATH`; `review_probes.py` on both trees.
- Outputs went to the evidence directory. The two replay files the acceptance scripts write into `results/` were moved there afterwards, so the experiment directories are unchanged.
- No deadline was passed to the acceptance runs. E3 took 431 s and E4 40 s, run side by side.

## Evidence and results

`D:\BuildArtifacts\lesscode\corrective-audit\task74\1.2\` (2,175,674 bytes): the logs of each command,
the regenerated measure files, both acceptance logs with their replay files, the held-out replay log,
both probe outputs and `evidence-sha256.json`.

Twelve regenerated files were compared with the retained ones after normalising line ends: four E3
measure files, six E4 measure files, the pure-share file and the patch-point file. All are identical.
The two probe outputs are identical in bytes.

## Limits and interpretation

- The mutation runs take hours and were not repeated. That they ran on the candidate is read from the
  scripts, the recorded module hashes and the per-mutant replay files, not from a rerun.
- E3's sensitivity and solver targets failed in R25. This audit does not change those verdicts.
- Reproducing a count shows the count is real. It does not show the tests are sensitive; R25 and R26
  report that separately, and E4's original tree scored 0.414.
- The acceptance runs depend on the retained environment under `.runs/`, which is not committed.

## Tool/spec consequence and next gate

- No dated amendment is needed in R25 or R26 for group 4.
- The R134 line "Not audited: E3, E4" is closed by this entry.
- `deconvolution-loop` keeps its status as far as E4 is concerned; task 1.4 looks at its other claims.

## COBESY check

Each row names its script and whether it was rerun. "Not rerun" rows are not counted as confirmed.
