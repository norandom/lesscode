# E2 plan: edge-based convolution census of Facdrone

- Written: 2026-11-03, Europe/Berlin, before any recorder code or execution.
- Hypothesis: H8(b) in [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md).
- Type: preregistered. Calibration predictions below are fixed; amendments
  must be dated and may not alter them after results are seen.
- Runtime: native Windows only, uv-managed CPython 3.13. Nothing in WSL.
- Independence: E2 does not depend on E1a's outcome. It tests whether edge
  measures select convoluted scopes, not whether an oracle can be built.

## Question

Do structural edge measures, computed from the original's own execution plus
static facts, rank the responsibilities a human recognises as convoluted above
those they do not, and do they surface responsibilities not yet examined?

## Frozen inputs

- Specimen: `D:\Source\facdrone` at HEAD `d49f48bd13c4e3da8a219d5bf73a974a9f982999`,
  identical to the R15 frozen manifest, working tree clean at planning time.
  Record `git status --short` at execution. The checkout is not modified;
  recording runs against the R15 frozen copy under `.runs/facdrone-discovery-02/source`
  if it still hash-matches, otherwise a fresh `git archive` copy.
- Driver: the hermetic tier-1 test selection from `scripts/gates.sh` (`-m "not db
  and not live_broker and not port_baseline"` plus the satellite package tests),
  run with pytest on native Windows. Tests that fail or error on the baseline
  are recorded and their traces still count; they are not repaired.
- Calibration cases, defined by the R15 bundles and not re-derived:
  A `case-evidence-publication.json` (8 files, 1,256 AST statements),
  B `case-report-envelope.json` (6 files, 925), C `case-index-provider-orchestration.json`
  (5 files, 693). R16 and R17 were executed on A and C respectively.

## Unit of measurement

A responsibility is the dynamic call closure, restricted to Facdrone packages,
of one entry callable. Entry callables are the production functions and
methods that tests invoke directly (first Facdrone frame under a test frame in
the recorded call stack). Closures overlap; overlap is reported, never summed.
The three calibration cases are additionally measured as declared file sets so
their numbers are comparable with R15 to R18.

## Recorder

Research apparatus in this directory, run through `uv.exe run --no-project
--python 3.13`; no Facdrone source change, no Facdrone dependency change.

1. `sys.monitoring` call and return events, filtered to code objects whose file
   lies under the frozen source root, produce caller to callee edges per test
   and the call stack at each effect.
2. Effects are detected as calls into a fixed sink list: SQLAlchemy cursor
   execution, `open`/`pathlib.Path.write_*`/`os.replace`, `json.dump*`,
   `pickle`, `subprocess`, `socket`. The list is frozen in `sinks.py` before
   the run; sinks not on the list are not counted and that is reported.
3. Static facts from `ast` over the same frozen files: attribute and subscript
   stores per callable (write sites), module membership, and calls to the
   frozen codec list (`json.dumps/loads`, `base64.*`, `zlib`/`gzip`,
   `pickle`, `model_dump*`/`model_validate*`, `dataclasses.asdict`,
   `str`/`bytes` constructor calls with a non-literal argument).
4. Coverage.py runs alongside so that unexecuted callables are labelled
   `static-only` in every table.

## Measures, per responsibility

| Measure | Source | Definition |
| --- | --- | --- |
| Call edges | dynamic | distinct caller to callee pairs in the closure |
| Module hops per effect | dynamic | distinct modules on the recorded stack from entry to each effect, averaged and maximum |
| Write sites per state location | static, confirmed dynamic where executed | distinct callables storing to the same attribute path or table name |
| Representation churn | dynamic | codec-list calls on the recorded path from entry to each effect |
| Degree of scattering | static plus dynamic | Eaddy-style concentration of closure lines over modules, 0 concentrated to 1 uniform |
| Effect ordering constraints | dynamic | count of ordered effect pairs that occur in the same order in every recording |
| Size context | static | files, physical lines, AST statements, as in R15 |

## Predictions, fixed before execution

- C1: Case A ranks in the top decile of all responsibilities on write sites
  per state location and on representation churn.
- C2: Case C ranks below the median on both of those measures.
- C3: Case B ranks above the median on representation churn.
- C4: At least three responsibilities not among A, B, C or the R11 leads rank
  in the top decile on at least two of the first five measures.
- C5: Rank correlation between physical lines and each edge measure across
  responsibilities is below 0.7 (the measures carry information lines do not).

## Kill criteria

- C1 or C2 fails: the edge measures do not reproduce the human calibration,
  H8(b) is rejected as formulated, and E3 may not use them for selection.
- Fewer than 60% of Facdrone package callables are reached by the hermetic
  tier: report the census as partial and do not claim repository-wide ranking.
- Recorder overhead makes the tier exceed four hours on this machine: stop,
  record, and reduce to the satellite packages containing A, B and C with an
  explicit scope reduction.

## Budget

At most two agent sessions. Recorder and analysis scripts at most 600
research lines. One full recording run plus one repeat to check determinism of
the edge counts; counts that differ between runs are reported with both values.

## Outputs

`results/edges.json` (per-responsibility measures), `results/calibration.md`
(the five predictions with numbers), `results/receipt.json` (commands,
versions, hashes, timings, baseline test failures), the frozen `sinks.py` and
`codecs.py`, and a ranked table of the top twenty responsibilities with their
file sets, so an E3 preregistration can cite one without re-deriving it.

## Not claimed

No compression, no equivalence, no oracle. A passing E2 supplies a selector
for E3 and a population statement for Facdrone; it says nothing about other
repositories until repeated there.

## Amendment, 2026-11-03 (development phase, before any recording)

Written before the recorder existed and before any measure was computed. The
predictions C1 to C5 and the kill criteria are unchanged.

1. Source copy. The R15 copy at `.runs/facdrone-discovery-02/source` still
   hash-matches its manifest (781 of 781 files), but it holds only the
   production `.py` files: no tests, conftests, `pyproject.toml` or `uv.lock`,
   so the driver cannot run against it. Recording therefore runs against a full
   `git archive HEAD` copy at `.runs/e2-facdrone/source`, whose 781 production
   files are checked against the same R15 manifest before use. "Facdrone
   packages" and the frozen source root mean exactly those 781 files; test code
   is not monitored and counts as the test frame.
2. Environment. `.runs/evidence-publication-venv` lacks `coverage`,
   `pytest-timeout`, `pgembed`, `httpx` and further dev-group packages that the
   hermetic tier imports. A new environment is built at `.runs/e2-facdrone/venv`
   from the copy's own `uv.lock` (`uv sync --frozen --python 3.13 --extra acquisition --all-groups`, the
   install `docs/Setup_New_Instance.md` prescribes; the `conformal` extra is left
   out because the composition root refuses it under the default declaration),
   with `coverage` added as a research-only tool. The resolved package list goes
   into the receipt.
3. Operational definitions fixed now, where the table leaves a choice. A
   responsibility's scalar per measure is: call edges = distinct pairs; module
   hops = maximum over effects (mean reported); write sites = maximum over state
   locations of distinct writer callables within the scope; representation
   churn = maximum over effects of codec-list calls in the same entry invocation
   before that effect (0 if no effect); scattering = Eaddy DOS of scope lines
   over all 781 production modules; ordering = ordered effect-key pairs (sink,
   issuing callable) that never occur reversed in any recording and co-occur in
   at least one. A case measured as a file set uses the same functions with the
   scope set to the executed callables defined in its files, over the
   recordings whose closure touches them, effect stacks restricted to those
   callables. That file-set value is the one ranked for C1 to C3; the best
   entry-responsibility inside each case is reported as secondary.
   "Top decile" = fewer than ceil(0.1 N) population values strictly greater
   and the value is non-zero (so a measure that is zero almost everywhere does
   not place its zeros in the top decile);
   "below/above median" = strictly below/above the population median. For C4
   a responsibility is "among A, B, C or the R11 leads" when its entry callable
   is defined in a case file or in `src/facdrone/shells/cli/burn_in_cache.py`
   or `src/facdrone/shells/wiring/run_config.py` (the R11 Facdrone leads). C5
   uses the six edge measures. Module-level code (`<module>`) is never an entry.
4. Known capture limits fixed now: sinks are detected only when called directly
   from Facdrone code (a file written inside pandas or pyarrow on Facdrone's
   behalf is not counted); SQL writes are attributed to tables by parsing
   `INSERT`/`UPDATE`/`DELETE` statement text; file paths are not treated as state
   locations because they are per-test temporary paths.

## Amendment 2, 2026-11-03 (development phase, before the first smoke run)

`facdrone_testing.fixtures.pytest_configure`, loaded by every Facdrone conftest, calls
`os.geteuid()`, which does not exist on native Windows, so pytest aborts with an internal
error before collection. Without touching Facdrone, a research plugin `e2_winshim.py`
(`-p e2_winshim`, in recorded and unrecorded runs alike) defines `os.geteuid = lambda: 0`
when absent. Facdrone's own branch for that value starts no embedded PostgreSQL and leaves
the DSN unset, so DB-backed tests skip as they do on a host without a DSN. This matches the
plan's `not db` driver; it is an environment adaptation, not a test repair. Modules that
import POSIX-only `fcntl` (`facdrone_client.evidence`,
`facdrone_research.commands.census_verdict_store`) will fail where executed and are
recorded as baseline failures, not repaired.

## Amendment 3, 2026-11-03 (development phase, after the first two-file smoke run, before any full run)

The smoke run showed fixture and helper functions from test-support modules recorded as
entry callables (for example `facdrone_testing.fixtures._migrated_schema`,
`facdrone_reporting.testing.reporting_vocabulary.reporting_config`), which absorbed the
production calls made through them. The plan defines entries as production functions under
a test frame. Test-support modules are therefore treated as test frames: the 61 manifest
files under `packages/facdrone-testing/`, under any `testing/` package directory, or named
`testing.py` (6,193 physical lines) are not monitored, are not entries, are not in any
closure and are excluded from the 60% reach denominator. The production population is the
remaining 720 files; the Eaddy denominator stays the module count of the static facts for
those 720 files. None of the A, B, C case files is affected. The smoke numbers that led to
this change are development output only and say nothing about the predictions.

In the same change, `os.open` and `os.fdopen` were added to the frozen sink list under the
plan's `open` entry, because Facdrone's artifact publisher writes through them
(`facdrone_reporting/shells/artifact_publisher.py`). The sink list hash in the receipt is
taken after this change and before the full run.

## Amendment 4, 2026-11-03 (development phase, before any full run)

Under pytest-xdist on this machine, worker start-up failed with `MemoryError` and
`WinError 1455` (paging file too small) in unrecorded and recorded runs alike, while a
concurrent cargo-mutants experiment held most of the commit charge. With
`OPENBLAS_NUM_THREADS=1`, `OMP_NUM_THREADS=1` and `MKL_NUM_THREADS=1` the same two-file
smoke passed under `-n 2` both ways. The full-run commands therefore set these three
variables for recorded and unrecorded runs. They change BLAS threading only; a test whose
numeric result depended on the BLAS thread count would show as a difference against a
Linux gate run, not against this baseline. The recorder file is named `e2_recorder.py` so
that it loads as `-p e2_recorder`.

## Amendment 5, 2026-11-03 (before any full run): embedded PostgreSQL runs natively on Windows

Written after amendment 4 and before the first full recording. A direct probe on
2026-11-03 showed that Facdrone's embedded PostgreSQL (pgembed 0.2.0, PostgreSQL
17.9, win_amd64 wheel) starts natively on Windows once one option is unquoted:
`pg_ctl -o "... -c listen_addresses='' ..."` reaches the server with the literal
quotes on Windows, which then fails to resolve the host name `''`; the unquoted
empty value starts the server on its Unix socket. This is a Windows defect in the
specimen's test support, recorded as a finding, not fixed in Facdrone.

Consequence: `e2_winshim.py` now returns `geteuid() == 1000` instead of 0 and
patches the server start with the one-option correction, so DB-backed tests run
instead of skipping. This widens the executed population relative to the smoke
runs and makes SQL sinks observable. The marker expression is unchanged
(`not db` still deselects the operator tier). Predictions C1 to C5 are unchanged.
The apparatus grows by about 40 lines, taking the total past the 600-line budget;
the overrun is recorded rather than hidden.

## Amendment 6, 2026-11-03 (before the first full run): batching and repeat scope

Owner decision after a host memory stop on 2026-11-03: heavy jobs run one at a
time; xdist uses two workers; the root `tests` suite is run in directory batches
(unit, contracts, property, then the remainder) and each satellite package in
its own process, each batch with its own JSONL and junit output. The
determinism repeat (run2) is reduced to the root suite plus the reporting
satellite, which contain calibration cases A, B and C. The first recording
(run1) still covers every suite. Predictions C1 to C5 are unchanged; the
repeat's narrower scope is reported with the results.

### Amendment 5a, 2026-11-03: second Windows defect in the embedded server start

The first B1 smoke attempt with the corrected option hung for 16 minutes with
no test executed. A py-spy stack showed pytest blocked in the shim's
`pg_ctl -w start` call: the started postgres inherits the captured stdout and
stderr pipes on Windows, so `subprocess.run(capture_output=True)` waits for an
EOF that never arrives, even though `pg_ctl` itself had returned and the server
was ready. The test database was never created. The shim now runs that one
call with detached standard handles; `-w` still provides the readiness wait.
This is the same upstream `_run` helper as the quoting defect and is recorded
as a second specimen finding. Predictions unchanged.

## Amendment 7, 2026-11-03 (before the first full run): marker expression aligned to the gate

The B1 smoke with the corrected server start collected nothing: all 25 tests in
`tests/unit/test_decision_slice_publisher_db.py` carry `pytest.mark.db` and the
plan's expression `not db and not live_broker and not port_baseline` deselected
them. Facdrone's own hermetic tier (`scripts/gates.sh`, `run_tests`) runs with
`-m "not port_baseline and not live_broker"`: `db`-marked tests execute on the
embedded server, and the fixture plugin itself skips the operator-tier subset
that needs populated data. The plan's `not db` was copied from the marker
description in `pyproject.toml`, which predates that gate decision. The
recording now uses the gate's expression. Predictions unchanged; the executed
population is the gate's, as the plan intended.
