# E2 apparatus: edge-based convolution census of Facdrone

Preregistration and amendments: [plan.md](plan.md). Status (2026-11-03): **development
phase only.** The recorder and analysis have been smoke-tested on two small test files.
The full hermetic tier has **not** been run, and none of C1 to C5 has been measured. The
smoke numbers under `results/smoke/` come from 7 tests and say nothing about the predictions.

## Files

| File | Role | Lines |
| --- | --- | ---: |
| `sinks.py` | frozen effect-sink list (data) | 24 |
| `codecs.py` | frozen codec list (data); loaded by path, so it never shadows stdlib `codecs` | 15 |
| `e2_recorder.py` | pytest plugin `-p e2_recorder`: `sys.monitoring` edges, effects, codec counts, coverage.py API | 178 |
| `e2_winshim.py` | pytest plugin `-p e2_winshim`: `os.geteuid` shim for Facdrone's fixture plugin on Windows (amendment 2) | 9 |
| `static_facts.py` | AST pass: callables, own lines, statements, write sites, codec call sites | 127 |
| `analyze.py` | joins everything into `edges.json` and `calibration.md`, ranks, C1 to C5, in-file Spearman | 181 |
| `receipt.py` | appends commands, times, exit codes, versions and hashes to `results/receipt.json` | 42 |
| **Total** | at most 600 allowed | **576** |

These are physical lines, docstrings and blank lines included. The same counts and the file
hashes are recorded in `results/receipt.json` under `facts`.

## Environment (already built)

- Frozen source: `D:\Source\lesscode\.runs\e2-facdrone\source`, a `git archive HEAD` of
  Facdrone `d49f48bd…`. Its 781 production files hash-match the R15 manifest (amendment 1).
  `D:\Source\facdrone` is only read.
- Interpreter: `D:\Source\lesscode\.runs\e2-facdrone\venv`, CPython 3.13.16, from the copy's
  `uv.lock` (`uv sync --frozen --python 3.13 --extra acquisition --all-groups`) plus
  `coverage==7.13.0`. The workspace members are editable installs that point into the
  frozen copy, so every Facdrone frame has a path under the frozen root. The resolved
  package list (208 entries) is in the receipt.

## Full run (on the orchestrator's instruction only)

Run from Git Bash. `RUN=run1` for the recording and `RUN=run2` for the determinism repeat.
The suites and flags follow `scripts/gates.sh` `run_tests`: root `tests`, then the seven
satellites, `--dist loadgroup --timeout=300`, with `--ignore=tests/test_lifecycle_full_loop_db.py`
for research. The marker expression adds `not db`, as the plan requires.
`N=auto` matches the gates. Use a fixed count such as `N=4` while other experiments share
the machine.

```bash
ED=D:/Source/lesscode/docs/experiments/deconvolution/e2-facdrone-edge-census
S=D:/Source/lesscode/.runs/e2-facdrone/source
V=D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe
MAN=D:/Source/lesscode/docs/experiments/constraint-modeler/facdrone-discovery/run-02/source-manifest.json
RUN=run1; N=auto
MK="not port_baseline and not live_broker"  # amendment 7
ENVS="--env PYTHONPATH=$ED --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env E2_SOURCE_ROOT=$S --env E2_MANIFEST=$MAN --env COVERAGE_CORE=sysmon"
PYT="uv.exe run --no-project --python $V python -m pytest -q -p no:cacheprovider -p e2_winshim -p e2_recorder -n $N --dist loadgroup --timeout=300"
cd $ED
uv.exe run --no-project --python 3.13 python receipt.py run $RUN-root --cwd $S $ENVS --env E2_OUT=$ED/results/$RUN/root -- \
  $PYT tests -m "$MK" --junitxml=$ED/results/$RUN/root.xml
for u in research reporting dash kernel factor-kit factor-comm-term-structure factor-short-reversal; do
  skip=; [ "$u" = research ] && skip=--ignore=tests/test_lifecycle_full_loop_db.py
  uv.exe run --no-project --python 3.13 python receipt.py run $RUN-$u --cwd $S/packages/facdrone-$u $ENVS --env E2_OUT=$ED/results/$RUN/$u -- \
    $PYT -m "$MK" $skip --junitxml=$ED/results/$RUN/$u.xml
done
```

Then, after both runs:

```bash
uv.exe run --no-project --python 3.13 python receipt.py run static-facts -- \
  uv.exe run --no-project --python 3.13 python static_facts.py $S $MAN results/static_facts.json
uv.exe run --no-project --python 3.13 python receipt.py run analyze -- \
  uv.exe run --no-project --python $V python analyze.py results/run1 results/static_facts.json $S results results/run2
uv.exe run --no-project --python 3.13 python receipt.py facts $V
```

This writes `results/edges.json` and `results/calibration.md`. To get an unrecorded
baseline time for the four-hour kill criterion, run the same loop with `-p e2_recorder`
and the `E2_*`/`COVERAGE_CORE` variables removed and `RUN=base`.

## Smoke evidence (2026-11-03)

- Files: `tests/unit/test_run_slice_sealing.py` (6 tests, case A code) and
  `packages/facdrone-reporting/tests/test_record_simulation_request.py` (1 test, case B code),
  collected together from the copy root. All 7 tests pass with and without the recorder.
- Determinism: two serial recordings and one `-n 2` recording give identical per-entry
  call-edge and closure counts (`results/smoke/analysis*/edges.json`, `repeat_differences`
  empty).
- Overhead, measured as the sum of junit testcase times, recorded over unrecorded: 2.10x,
  1.43x and 1.75x in three pairs. pytest's collection-plus-run time: 1.10x and 1.21x.
  Process wall time is dominated by imports and by a concurrent cargo-mutants run
  (0.77x to 1.54x), so it is not an overhead estimate.
- SQLAlchemy: a class-level `before_cursor_execute` listener on `sqlalchemy.engine.Engine`
  works without touching Facdrone. A probe (`EvidenceLedger(sqlite engine).ensure_schema()`)
  recorded the cursor execution with `EvidenceLedger.ensure_schema` on top of the Facdrone
  stack (receipt label `sqlalchemy-listener-probe`). With `not db` and no DSN, most SQL
  paths will not run in the hermetic tier.

## Known limitations found during development

1. Effects are detected only when Facdrone code calls a sink directly. A file written by
   pandas, pyarrow, openpyxl or kedro on Facdrone's behalf is not counted. Sinks outside
   the list (for example `shutil`, `tempfile`, HTTP clients above `socket`) are not counted.
2. `json.dumps` is both a sink and a codec, as the plan lists it. Churn counts codec calls
   *before* each effect, so an effect does not count itself. A responsibility that
   serialises in a loop gets a large churn value (721 in the case B smoke).
3. Write sites are textual store paths with `self`/`cls` replaced by the class. Stores through
   locally bound names are dropped as temporaries. Stores through parameters keep the
   parameter name, so two functions writing `state.x` merge only if both call the
   parameter `state`. SQL writes become `table:<name>` by parsing INSERT/UPDATE/DELETE text.
4. Callables that Facdrone passes to library code (pandas `apply` callbacks and similar)
   attach to the nearest Facdrone frame on the shadow stack. Threads that Facdrone starts
   begin their own invocation, with the thread target as entry.
5. Test-support modules (61 files) are treated as test frames (amendment 3). A production
   call made through a helper such as `facdrone_testing.core.run_slices.capture_details`
   therefore counts under the production function the helper calls.
6. Windows only: modules that import `fcntl` fail where they run. Failures are recorded from
   junit and from the JSONL `out` fields, not repaired (amendment 2). `os.geteuid` is shimmed
   to take Facdrone's own "no embedded PostgreSQL" branch.
7. Python code that runs inside a `sys.monitoring` callback fires monitoring events again.
   An early recorder version overflowed the stack through `os.path.normcase`. The recorder
   now does only C-level string work on that path. Any later edit to the callbacks needs the
   same care.
8. Early recorder versions produced the recordings under `results/dev-superseded/`. They are
   kept as development evidence and are not inputs to the analysis.
