# E4 task 6.5: the differential behind the four gates (phases 1 and 2 complete)

- Written: 2026-11-06, Europe/Berlin; phase 1 (pairs, drivers, light gates) in the morning while the 6.4 series ran,
  phase 2 (nine solver runs, four concrete replays) in the afternoon after the controller's solver go (renewed after the
  harness reaped the first serial script under host memory pressure; notes `e5-phase2-stop-1`, `e5-phase2-resume-1`).
  Governing text: [plan.md §6.5](../plan.md), kill criterion 4, Budgets; the pairs, domains, planted mutants, renamings
  and predictions are preregistered in [pairs.md](pairs.md), whose dated notes record the four driver fixes. Every
  command, probe, hash and verdict is in `../results/receipt.json` (labels `e5-*`), logs in `../results/logs/`, per-gate
  result files `../results/e5-<gate>-<pair>[-<kind>].json`, the CrossHair outputs `../results/logs/e5-<pair>-solve-<kind>.crosshair.txt`,
  the summary `../results/e5-gates.json` (`e5-gates-4`).
- Trees: the frozen original `.runs/e2-facdrone/source` and the accepted candidate `.runs/e4-lifecycle/cand-turn3`,
  both read only throughout (seven-file hashes re-verified after phase 1 and after phase 2, notes
  `e5-phase1-trees-unchanged`, `e5-phase2-trees-unchanged`); the planted mutant and the renamed variant of each
  candidate module exist as text inside the driver process only.

## What is built

`e5_drivers.py` (192 physical lines, sha256 in fact `task-6.5-apparatus-5`; the plan's budget line for the 6.5 drivers
is E3's 149, so the **overrun is 43 lines**, reported and not compacted). It loads the six scope modules the pairs can
reach from each tree under distinct names in one process (the E3 `t5_drivers` loader: module objects, each tree's
siblings swapped into `sys.modules` only while its module executes), binds per pair `o_P` (original), `c_P`
(candidate), `k_P` (planted mutant), `e_P` (renamed locals) over the recorded domain and `ct_P` / `et_P` over a
three-boolean domain, and offers `--list`, `--smoke`, `--corpus`, `--plant`, `--rename`, `--gates`, `--solve`,
`--replay`. CrossHair 0.0.112 with z3 5.3.3 runs through the E1c-2 launcher as a child of a driver process that has
**not** loaded the trees (a parent that has loaded them exits 0xE0000001 when the child imports them; receipt
`e5-compare-smoke-diag*`). Results are canonical reprs; an exception is `("raise", type, message)`.

## The pairs

| # | Pair | Original → candidate | Hypothesis | Predicted (pairs.md) |
| --- | --- | --- | --- | --- |
| 1 | `compare` | `cli/lifecycle_group.py:203 _compare` (echoes as it goes) → `:195 _compare` (returns the lines); adapted to (lines emitted, code or exception) | H1 (F3), H11 | counterexample of the F3 shape, found by the corpus already |
| 2 | `standing` | `lifecycle_projection.py:139 signal_standing` + six helpers → `:61`, one body | H8, H7 | agree, except a predicted error-ordering difference under double malformation (`standing = None` and a malformed baseline) |
| 3 | `window` | `lifecycle_evidence.py:166 LiveEvidence.for_window` + `_lookback` → `:148`, inlined | H13 | agree; solver `inconclusive` |

The brief asked for one pair per H1, H2, H3 and H9 plus unchanged-function controls; the frozen plan allows at most
three pairs from its candidate list, so H2, H3 and H9 stand for T6 as the review's probe-found differences (F2, F4, F5),
not as pairs (pairs.md, "Difference from the task brief").

## Gate results (final runs; every earlier run stays in the receipt)

| Pair | 1 symbolic-ness (`check`, 15 s/condition) | 2 corpus (300 inputs, seed 20261106) | 3a planted mutant: corpus / solver | 3b renaming: corpus / solver (tiny domain) | Solver on the pair (`diffbehavior`, 300 s base scaled) | 4 concrete replay |
| --- | --- | --- | --- | --- | --- | --- |
| `compare` | holds, 63/63 entries (`e5-compare-smoke-4`) | **differ 24/300** (`e5-compare-corpus-4`) | found 34/300 / **counterexample** 445.4 s at scaled 408 s, factor 1.36 (`e5-compare-solve-plant`) | 0 spurious / **exhausted**, 16 iterations, 12.9 s (factor 1.49, scaled 447 s; `e5-compare-solve-rename`) | **counterexample**, 463.9 s at scaled 447 s, factor 1.49 (`e5-compare-solve-pair`); **not gated** by check 2 (the corpus had already disagreed, plan 9.3) | both differ concretely (`e5-compare-replay-2`) |
| `standing` | holds, 22/22 (`e5-standing-smoke-2`) | **differ 2/300** (`e5-standing-corpus-2`) | found 77/300 / **counterexample** 547.2 s at 531 s, factor 1.77 (`e5-standing-solve-plant`) | 0 spurious / **exhausted**, 16 iterations, 6.7 s (factor 1.34, scaled 402 s) | **counterexample**, 415.4 s at 390 s, factor 1.30 (`e5-standing-solve-pair`); not gated by check 2 | both differ concretely (`e5-standing-replay-2`) |
| `window` | holds, 57/57 (`e5-window-smoke-2`) | agree 300/300 (`e5-window-corpus-2`) | found 164/300 / **inconclusive**: no difference in 5,559 iterations, 990.1 s at 972 s, factor 3.24 (`e5-window-solve-plant`): **the solver control fails** | 0 spurious / exhausted, 16 iterations, 5.1 s (factor 1.29, scaled 387 s) | inconclusive, 5,719 iterations, 739.5 s at 732 s, factor 2.44 (`e5-window-solve-pair`); **not read** | nothing to replay |

Every factor and scaled deadline above is the one in the run's `<label>-deadline` note (the probe taken right before
the run), not the guard probe's. The receipt `--timeout` backstop of every solve was the scaled deadline + 120 s by
construction of the runner (note `e5-timeouts`: 528, 567, 567, 651, 522, 510, 1092, 507, 852 s in run order).

Controls: `compare` and `standing` hold on both forms (3a counterexample, 3b exhausted). `window`'s 3a holds on the
corpus but not on the solver, which never reached the planted difference in 5,559 iterations although 164 of 300 random
inputs expose it (the search spends its paths on the early `ValueError` branches of `_daily`). Gate 1 (the smoke) is a
path-count signal only and predicted nothing about the solver's reach, as this pair shows (57 entries per side, then
no planted difference in 5,559 iterations). The plan's criterion 4 is literal: **"A verifier control of 6.5 fails: 6.5
is void, reported as void; 6.2 to 6.4 and 6.6 stand."** 6.5 is therefore reported **void as a whole**; the per-pair
reading of the brief (only `window` void) is reported beside it, labelled, for the owner. The driver-fix allowance was
spent before this run; no retry, no parameter change (the run ended inside its deadline, so the rerun rule for timeouts
above factor 1.5 does not apply). Three runs saw factors above 1.5 at their deadline probe: window plant (3.24) and
window pair (2.44), which ran to their scaled deadlines, and standing plant (1.77), which returned a counterexample, so
no rerun was required; no timeout, no rerun anywhere.

### Solver counterexamples, verbatim, with their concrete replays

**compare, planted mutant (3a)** `k_compare` vs `c_compare` (`e5-compare-solve-plant.crosshair.txt`):
```
Given: (rows=[(0, 2, 23, 0, 3), (2, 2, 0, 0, 1), (0, 2, 23, 0, 3), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0)], fresh=[(0, 25, 0, 2)]),
  e5_drivers.k_compare : returns "(['survival                 DIFFERS  was not_run 23', 'survival                 DIFFERS  was not_run 23'], 0)"
  e5_drivers.c_compare : returns "(['survival                 DIFFERS  was not_run 23', 'survival                 DIFFERS  was not_run 23'], 1)"
```
Replay: identical strings, `differ: true` (the mutant inverts the exit code).

**compare, the pair** `o_compare` vs `c_compare` (`e5-compare-solve-pair.crosshair.txt`):
```
Given: (rows=[(0, 2, 0, 0, 3), (2, 2, 0, 0, 0), (2, 2, 0, 0, 2)], fresh=[(1, 0, 0, 2), (2, 0, 0, 2), (1, 0, 0, 1), (0, 0, 0, 0)]),
  e5_drivers.o_compare : returns '([\'survival                 DIFFERS  was not_run 0\'], (\'raise\', \'KeyError\', "\'monitor\'"))'
  e5_drivers.c_compare : returns '([], (\'raise\', \'KeyError\', "\'monitor\'"))'
```
Decoded: stored rows = a well-formed `survival not_run 0 0` row, a `window` marker row (skipped by both), then a row
lacking `monitor`; `fresh` is truncated by `_n` to its first three entries, `ic_gap`, `window`, `ic_gap` (the fourth,
`survival`, is dropped), so `survival` is absent from `fresh` and the original prints `DIFFERS` for it. Both sides raise
`KeyError: 'monitor'`; the original has already printed `survival DIFFERS was not_run 0`, the candidate prints nothing.
Replay: `differ: true`.

**standing, planted mutant (3a)** `k_standing` vs `c_standing` (`e5-standing-solve-plant.crosshair.txt`): `standing=(1, 3, 29)`
(a `Retired` revision), `baseline=(-5, -6)`, three readings; the mutant swaps not-run rows and readings:
`k_standing` shows `readings: [{'monitor': 'window', … 'disposition': 'Did not run' …}]`, `not_run: [{'window': 'w1'}]`;
`c_standing` shows `readings: [{'monitor': 'Survival under the mandate', 'key': 'survival', 'window': 'w1', 'value': -1001 …}]`,
`not_run: [{'window': 'w2'}, {'window': 'w2'}]` (full text in the file). Replay: `differ: true`.

**standing, the pair** `o_standing` vs `c_standing` (`e5-standing-solve-pair.crosshair.txt`):
```
Given: (standing=(2, 3, 11), baseline=(5, 8), readings=[(2, 1, 1, -3, 9), (2, 1, 1, -3, 9), (2, 2, 0, -6, 8)], flag=(-22, 1), now=2),
  e5_drivers.o_standing : returns '(\'raise\', \'AttributeError\', "\'NoneType\' object has no attribute \'signal\'")'
  e5_drivers.c_standing : returns '(\'raise\', \'AttributeError\', "\'str\' object has no attribute \'get\'")'
```
Decoded: `standing = None` (kind 2, malformed), `baseline.states = ["calm"]` (8 // 2 % 3 = 1, a non-mapping), three
well-formed readings, an open flag. The original reads `standing.signal` first, the candidate builds the baseline view
first: the preregistered error ordering. Replay: `differ: true`.

The corpus disagreements of phase 1 (compare 24/300 with the same F3 shape; standing 2/300 with the same ordering) are
in `e5-corpus-compare.json` and `e5-corpus-standing.json`.

## Expressibility in the oracle's alphabet (plan 9.6), per realised counterexample

- **compare**: needs a stored reading row that lacks `monitor` (or `disposition`, `value`). `cli_readings` reads what
  `LifecycleStore.readings` returns, rows that `MonitorRunner._append` wrote with every column; no operation of the
  24-letter alphabet corrupts a stored row. **Inexpressible**; not replayed as a sequence. The input is, however, inside
  `_compare`'s declared type (`Sequence[Mapping[str, Any]]` admits a mapping without the key).
- **standing**: needs `standing = None` together with a baseline whose `states` holds a non-mapping. `standing` and
  `standing_view` derive the standing from the store (`UnrecordedStanding` or a revision, never `None`) and the
  baseline from the ledger. **Inexpressible**; not replayed as a sequence. The input is outside the declared type
  `Standing`.

## T6 reading (the prediction: at least one difference invisible to the recording, on an error or malformed-input path, concretely replayed)

**Strict reading (the plan's, governing): 6.5 is void** because the `window` planted-mutant control failed in its solver
form (criterion 4: "6.5 is void, reported as void; 6.2 to 6.4 and 6.6 stand"). The differential therefore contributes
"not found by the differential" (T6's failed-when clause), and **T6 holds on F3 (and F1) found by the 6.3 review's
probes**: F3 (partial CLI output lost on a stored-row fault, P03/P04/P18) and F1 (a `None` evidence silently accepted,
P62/P126) are oracle-blind differences on error or malformed-input paths, executed concretely on both trees in 6.3.
The `compare` and `standing` solver counterexamples above, both realised on concrete replay, are reported as evidence
(the compare one instantiates F3 by a second route; the standing one is a new H8 error-ordering finding, F7 candidate)
but are **not credited to 6.5**.

Per-pair reading (the brief's, labelled as such, for the owner): only the `window` pair void; T6 then also holds on the
`compare` pair, whose realised solver counterexample instantiates **F3** (hypothesis **H1**, an error path on a
malformed stored row), unseen by the 50-sequence acceptance, differing on concrete replay on both trees and
inexpressible by the alphabet; `standing` adds the second oracle-blind error-path difference (error ordering under
double malformation, H8, inputs outside the declared types).

## Budget

Solver time, all nine `diffbehavior` runs as receipted command seconds (each includes about 5 s of import):
compare 445.4 + 12.9 + 463.9, standing 547.2 + 6.7 + 415.4, window 990.1 + 5.1 + 739.5 = **3,626.2 s** (E3's three pairs:
1,560.1 s). Smokes (final runs) 21.0 + 21.2 + 19.7 s; corpus, plant and rename gates about 5 to 10 s each. Guard waits
before runs: 765.8, 95.7, 43.2, 1,265.6, (standing rename and pair under a minute), 63.5, (window rename and pair under
a minute) seconds. Research code: `e5_drivers.py` 192 lines against the 149 allotted (overrun 43); running total of the
900-line budget: **667** as recorded at the time (6.1 97 + `patchcheck.py` 60 + `pureshare4.py` 15 + 6.4 303 + 6.5
192, with `e4_rates.py` at 50); **672 live** per the coordinator's review (`e4_rates.py` is 55 lines now).

## Not claimed

6.5 is void under the plan's criterion 4; nothing in this file is credited to 6.5 as a step result. No equivalence
anywhere: `window`'s "agree on 300 inputs" is evidence about those inputs, its solver runs are inconclusive and its 3a
control failed; `inconclusive` is never read as equivalent. The two realised differences are on
inputs the recorded corpus cannot produce; whether the stored-row fault of `compare` matters in production is a
reading for 6.7 (no caller the review read writes a partial row). Nothing here is a patch for Facdrone. The 0xE0000001
behaviour of a tree-loaded parent is recorded as an apparatus fact, not investigated.
