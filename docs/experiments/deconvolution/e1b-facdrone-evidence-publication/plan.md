# E1b plan: trace oracle feasibility on Facdrone's evidence-publication responsibility

- Written: 2026-11-03, Europe/Berlin, before any harness code or execution.
  Draft status: the seed set depends on whether Facdrone's embedded
  PostgreSQL runs natively on Windows (probe in progress, see E2 amendment 5).
  A dated amendment will fix the seed set before the harness is written.
- Hypothesis: H8(a) in [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md),
  after [R20](../../../research/2026-11-03/02-e1a-session-bus-oracle.md).
- Type: preregistered. Predictions are stated relative to a measured baseline
  so a strong seed suite cannot produce a ceiling artefact as in E1a.
- Runtime: native Windows only, uv-managed CPython 3.13. Nothing in WSL.

## Question

Can the oracle mechanics established in E1a be built for the scope that
defeated the static approach: scattered, transactional, multi-representation
state across eight Python modules? And does the oracle add discrimination
beyond the existing tests, measured against that baseline?

## Frozen inputs

- Specimen: Facdrone at `d49f48bd13c4e3da8a219d5bf73a974a9f982999`, the R15
  manifest; frozen copy `.runs/e2-facdrone/source` (hash-matched). The
  checkout is never modified; the harness lives in this directory and is
  loaded by `-p` plugin and `PYTHONPATH`, as E2 does.
- Scope: the R15 case A file set from
  `docs/experiments/constraint-modeler/facdrone-discovery/run-02/case-evidence-publication.json`
  (8 files, 2,682 physical lines, 1,256 AST statements). Entry callables:
  `seal_evidence`, `seal_evidence_with_run_slices`, `compact_capture`,
  `payload_batches`, `stored_evidence_payload` in
  `shells/decision_evidence_publication.py`; the commit and `_publish_evidence`
  paths of `cppi/state_repository.py`; the simulation publication steps of
  `wiring/run_evidence_store.py`; the publisher and input-slice store APIs;
  the evidence reader.
- Environment: `.runs/e2-facdrone/venv` (CPython 3.13.16, `uv.lock`, plus
  coverage 7.13.0); add cosmic-ray 8.7.0 and hypothesis in a separate research
  environment or as recorded extras. mutmut is excluded: it refuses Windows.
- Seed tests (Arm A), to be fixed by amendment once the probe result is known:
  `tests/unit/test_decision_slice_publisher_db.py` and every other test file
  that coverage shows executing scope lines, plus the R16 codec harness
  (128 observations) as a non-pytest seed. If the embedded database cannot run
  natively, the database-backed seeds are unavailable and E1b is inconclusive
  for this scope; it does not fall back to codec-only checks and call them a
  lifecycle oracle.

## Oracle design

A pytest plugin plus a scripted driver, research apparatus only:

1. Database: Facdrone's own embedded PostgreSQL fixture (`pg_engine`), one
   server per process, schema migrated by Facdrone's Alembic chain. Effects
   are captured two ways: a SQLAlchemy `before_cursor_execute` listener on
   the engine recording statement text and parameters, and a full ordered
   dump of the scope's tables after every operation.
2. Operations: the entry callables above, driven in sequences: capture, bind,
   seal, publish, retry, read back, with injected rollback (the trigger or a
   forced constraint failure), namespace change, duplicate and forward slice
   references, legacy dictionary-shaped evidence beside typed envelopes.
   Seed sequences are transcribed from the seed tests and the R16 harness;
   generated sequences come from Hypothesis strategies over the same small
   domains with a fixed seed. State is part of the input: generated sequences
   start from recorded table states, not only from empty tables.
3. Observations per operation: return value or exception type and message,
   table dumps, SQL statement sequence, and files written under the scope's
   artifact roots if any. Normalisation is minimal and declared: knowledge
   times and timestamps masked to first-seen ordinals; digests are kept
   verbatim and the first two recordings must agree on them, otherwise the
   digest input that varies is identified and masked with a dated note.
4. Record once from the unmodified copy; freeze by hash; replay only after.

## Measurements

| Measure | Arm A: seed tests | Arm B: seed tests + trace replay |
| --- | --- | --- |
| cosmic-ray on a fixed random sample of 150 mutants drawn (seed recorded) from `decision_evidence_publication.py` and `cppi/state_repository.py`; same sample for both arms | yes | yes |
| Kill rate = killed / (killed + survived), incompetent mutants excluded | yes | yes |
| coverage.py branch coverage of the 8 scope files | yes | yes |
| Three hand-written semantic mutants | yes | yes |
| Wall clock per arm; harness lines | recorded | recorded |

Hand-written semantic mutants:

- M1: in `state_repository`, adopt the slice ledger before the transaction
  commits instead of after (observable only when the commit fails).
- M2: in `_publish_evidence`, skip reference validation on the legacy route.
- M3: in the capture codec, change the compression level so stored bytes
  differ while decoding is unchanged (tests whether the oracle sees stored
  representation, not only decoded content).

## Predictions, relative to the measured baseline

Let k_A be Arm A's kill rate on the sample, measured before Arm B runs.

- Q1: Arm B's kill rate is at least k_A + 0.5 × (1 − k_A). The oracle closes
  at least half of whatever gap the seed tests leave. With k_A = 0.6 this
  requires 0.8; with k_A = 0.96 it requires 0.98.
- Q2: Arm B catches M1, M2 and M3.
- Q3: Arm B branch coverage of the scope is at least 85%, and at least 5
  points above Arm A.
- Q4: generated sequences observe at least three behaviours no seed sequence
  produces, at least one of them on the failure or rollback path.
- Q5: the first two recordings agree on every digest without masking.

## Kill criteria

- Embedded PostgreSQL cannot be started natively within one session: E1b is
  inconclusive for this scope; record and choose a scope with a sandboxable
  store before trying again.
- Q1 fails after the budget: H8(a) is rejected for the high-scatter class.
- Q5 fails and the varying input cannot be identified: inconclusive, since an
  oracle that must mask digests cannot protect them.
- Any production file in the copy changed: invalid run.

## Budget and stop rules

Two agent sessions. Harness at most 700 research lines. cosmic-ray at most
three hours wall clock per arm on the 150-mutant sample, with a per-mutant
timeout of five times the baseline test time. Stop at the first effect that
cannot be captured in-process; record it.

## Not claimed

No compression, no candidate, no equivalence. A positive E1b, together with
E1a, establishes H8(a) across a low-scatter Rust scope and a high-scatter
Python scope. E3 still requires its own preregistration on an E2-selected
scope.

## Amendment 1, 2026-11-03 (before any harness code): seed set fixed

B1 (E2 results/smoke-db-shim.log) ran `tests/unit/test_decision_slice_publisher_db.py`
natively on Windows: 25 passed in 1.80 s on Facdrone's embedded PostgreSQL,
with the two shim corrections recorded in E2 amendments 5 and 5a and the
gate's marker expression `not port_baseline and not live_broker` (E2 amendment
7; the draft's `not db` would have deselected every database-backed seed).
The seed set is therefore: that file, every other test file that coverage
shows executing lines of the eight scope files under the gate's expression,
and the R16 codec harness. The database kill criterion is retired. The
reusable shim is `e2_winshim.py`, loaded by `-p`. Predictions Q1 to Q5 unchanged.

## Amendment 2, 2026-11-03 (development phase; before the golden recording and before either arm)

The harness fixes details the plan left open. None changes a prediction; Q1 to Q5 unchanged.

1. R16 harness as a seed. `check.py` runs unchanged, in a subprocess, against the root under
   test, and its stdout must equal `results/r16-baseline.json` byte for byte; a difference kills
   the mutant in both arms. That baseline is recorded once from the frozen copy with the E2 venv
   (`run_arm.py r16-record`). R16's own `results/baseline.json` is not used: in this venv 8 of
   its 128 observations differ, only in pydantic's documentation URL (`/2.12/` there, `/2.13/` here).
2. Mutant sites. M1 assigns the copied ledger to `self._slice_ledger` before
   `engine.begin()` and keeps the later assignment (a no-op after success). M2 deletes the
   `require_referenced_slices` call on the `bound is None` branch of `_publish_evidence`. M3
   changes `compact_capture`'s zlib level from 1 to 9. `compact_capture` is the entry callable the
   plan names. `capture_payload` would need a policy `ConsumedCapture`, which the oracle does not
   construct.
3. Observation boundary. After each operation the oracle dumps `decision_input_slice`,
   `decision_evidence`, `live_decision_event`, `live_runtime_state`, `sim_runs`, `sim_events`,
   `sim_equity`, `sim_trades`, `sim_holdings` and `sim_cost_events`, with JSON/JSONB columns cast
   to their stored text. Normalisation masks only Python `datetime` values, to first-seen
   ordinals per sequence. Dates, identifiers, ISO strings inside stored text and every digest
   stay verbatim.
4. Generated sequences. Hypothesis 6.170.14 is installed into `.runs/e1b-facdrone/extras` (added
   to `PYTHONPATH`), not into the shared E2 venv. The fixed seed is 20261103, with 40
   sequences of 3 to 8 operations. A sequence optionally starts from the recorded end state of
   seed S01, S02, S03 or S07. The generated operations are stored in the golden trace, so
   replay re-executes them and does not generate again.
5. Per-mutant timeout. `run_arm.py --deadline` enforces 5 × the arm's measured unmutated run
   time and kills the process tree with `taskkill /T`, because cosmic-ray's process-group kill is
   POSIX-only. cosmic-ray's own timeout is set 120 s higher as a backstop. Timeouts count as
   killed (cosmic-ray semantics) and are reported separately.
6. Void runs. A cosmic-ray result whose output contains `run_arm: INVALID` (scope imported
   from a root other than the working copy, or the deadline left unset) is void. It counts
   neither as killed nor as survived.
7. Seed confirmation. Amendment 1's coverage condition is evaluated by one serial
   gate-tier run with a coverage context per test file (`run_arm.py all`, then `run_arm.py seeds`).
   It runs before Arm A. The static candidate list in `seeds.md` is then replaced.
8. Budget check before cosmic-ray. 150 mutants within three hours allows at most 72 s per
   unmutated arm run. If the measured T_A or T_B exceeds that, the run stops before
   `cosmic-ray exec` for an owner decision recorded as a further amendment. The sample is not
   shrunk silently, and the stop rule is not overrun silently.

## Amendment 3, 2026-11-03 (owner decision, before any recording or cosmic-ray run): budget

Measured during development: each Facdrone test process spends 10 to 15 s on
embedded-PostgreSQL start and imports, and the seed set exceeds 115 test files,
so one full seed run per mutant is on the order of half an hour. The owner
chose, from three options, coverage-selected test execution with a reduced
sample:

- Per mutant, both arms run only the seed test files whose per-file coverage
  (from the seed-confirmation coverage run) executes at least one line of the
  mutated module, plus, in Arm B, the trace replay. The selection map is
  frozen before `cosmic-ray init` and recorded with hashes.
- The fixed random sample is 60 mutants (seed 20261103), not 150, drawn once
  and shared by both arms. Kill rates are reported with this denominator.
- The harness line budget rises from 700 to 760 to hold the selection logic;
  the overrun is accounted, not hidden.
- Q1 to Q5 are unchanged. Arm A's k_A is measured before Arm B runs.

The owner declined enabling the Windows symlink privilege; the 131
symlink-privilege failures remain baseline failures and the behaviour they
cover is reported as unprotected on this host.

## Amendment 4, 2026-11-03 (execution session, before step 0 and before the seed-confirmation run)

Implementation details of amendment 3 that it left open. Q1 to Q5 unchanged.

1. Which module is mutated. `run_arm.py` hashes the two mutated modules of the root under test
   against the R15 manifest (`scope.record`). Exactly the changed one(s) select their seed files
   from `results/selection-map.json`. If neither is changed (the unmutated arm runs that measure
   T_A, T_B and coverage) or a file outside the two is changed, the union of both lists runs.
   T_A and T_B are therefore measured on the union, an upper bound for any single mutant, and
   the per-mutant deadline (5 × T) is derived from that bound.
2. Baseline failures. The seed-confirmation run (`run_arm.py all`) now runs every group even
   after a failing one, and records the node id of every failing test (setup, call or teardown)
   per group. Without this, the first failing root group would end the census before the
   satellites ran. Tests that fail on the unmodified frozen copy in this environment (for example
   the symlink-privilege and file-lock failures) are passed to both arms as `--deselect`
   when their file is selected. Otherwise every mutant would count as killed by a baseline
   failure. They are not repaired. The behaviour they cover counts as unprotected in both arms.
   The deselect list is frozen inside the selection map and hashed with it.
3. Logs. Every receipt-recorded command's console output is also kept under `results/logs/`
   (through `tee`). The receipt's own fields are unchanged.

## Amendment 5, 2026-11-03 (during step 1, after the root group and before the research rerun)

In the serial seed-confirmation run, `packages/facdrone-research` stopped at collection: 8 test
modules import `fcntl`, which Windows lacks (`results/logs/seeds-coverage.log`), and pytest without
xdist aborts the whole session on collection errors. None of the group's tests ran. E2's
xdist runs did not stop there. `run_arm.py all` now passes `--continue-on-collection-errors`, and
an `E1B_GROUP` variable reruns a single group. The research group is rerun alone into the same
coverage directory. The aborted attempt's data holds only collection-time lines, which have an
empty context and therefore select no seed. The 8 modules stay baseline failures (POSIX-only
import) and are never repaired. Q1 to Q5 unchanged.

## Amendment 6, 2026-11-03 (after T_A was measured, before any cosmic-ray command)

Measured T_A = 409.5 s, the unmutated Arm A over the 151-file union (receipt `arm-a-base`).
That exceeds amendment 2.8's 72 s check, which was sized for 150 mutants. Amendment 3 already
holds the owner's decision on that budget, and the owner's execution instruction for this session
fixes the stop rule: cosmic-ray `exec` runs under a hard cap of three hours wall clock per arm.
At the cap the process tree is killed (`run_arm.py cap`, the same `taskkill /T` path as the
per-mutant deadline). Any source file the in-flight mutant left changed in that arm's own working
copy is restored from the frozen copy and verified with `scope.py`. The counts are reported as
**partial**. The sample is not shrunk further. cosmic-ray chooses pending work in random order
(`order_by(func.random())` in `work_db.pending_work_items`), so partial arms can complete
different subsets. Each partial arm is therefore reported on its own completed set, and the
two are also compared on the intersection of completed mutants. Arm A's k_A, partial or not,
is written into the receipt before Arm B starts. `cosmic-ray baseline` is run and timed
separately and does not count against the cap. Q1 to Q5 unchanged.

## Amendment 4, 2026-11-03 (owner decision, before cosmic-ray init): per-line test selection

Measured: with module-level selection (Amendment 3) one arm run takes about
ten minutes, because the publication module is imported widely and more than a
thousand tests execute at least one of its lines. Sixty mutants would need
about ten hours per arm. The owner chose, from three options, to keep the
60-mutant sample and select tests per mutant by the mutated line: for each
sampled mutant, both arms run only the seed test files whose per-test-file
line coverage (from the step-1 coverage run) executes the mutated line; Arm B
adds the trace replay. A mutant whose line no seed test executes is reported
as `uncovered` and counted as survived in both arms, since no arm can kill it;
the count of such mutants is reported separately. The selection map is frozen
with hashes before `cosmic-ray init`. Harness line budget rises to 820;
overrun recorded. Q1 to Q5 unchanged; the arms remain comparable because both
use the same per-mutant selection and differ only by the replay.

## Amendment 7, 2026-11-03 (execution session, before `cosmic-ray init`): implementing the per-line decision

Numbering note: the owner's per-line decision above is headed "Amendment 4". It was written while
this session's amendments 4 to 6 were being appended, and it comes after them in the file. It is
cited here as "the owner's per-line amendment". Neither text has been edited. Its implementation:

1. Line map. `run_arm.py seeds` rebuilds `results/selection-map.json` from the step-1 coverage data
   (`results/cov-all`, retained): for each mutated module, line -> seed files whose per-file
   context executed that line, plus the 191-file seed list and the unchanged deselect list. The
   module-level map of amendment 3 is kept as `results/selection-map.module-level.json`.
2. A mutant's lines. `scope.changed_lines` diffs the mutated module against the frozen copy. An
   insertion counts as both neighbouring lines. Each changed line is widened to its innermost
   enclosing statement, or to a compound statement's header lines, because coverage does not
   record every physical line of a multi-line statement. `run_arm.py` applies this to the root
   under test (cosmic-ray's mutant, M1 to M3). `sample_mutants.py sample`, run with cosmic-ray's
   interpreter, applies the same function to cosmic-ray's in-memory mutation of each sampled spec.
   Both therefore select the same set.
3. Unmutated root: the whole 191-file seed set, for coverage per arm and for T_A and T_B.
   Uncovered mutant (no seed file executes a changed line): Arm A runs only the R16 check, and
   Arm B runs the R16 check plus the replay. Following the owner's text, an uncovered mutant is
   counted as survived in both arms in the primary figures. Its observed exit is reported
   separately, because in Arm B the replay can still kill it.
4. Per-mutant timeout. "5 × the measured per-mutant selected-set time, floor 120 s": after the
   sample is drawn, `run_arm.py time A|B` runs every distinct selected set of the sample unmutated
   on the frozen copy, together with the whole seed set (cosmic-ray `baseline`). It writes
   `results/deadlines.json` (key = arm + hash of the selected file list; deadline =
   max(120, 5 × t)). A set that fails unmutated stops the step. `--deadline auto` looks the set up
   at run time and returns `run_arm: INVALID` if it is missing. The cosmic-ray backstop is the
   arm's largest deadline + 120 s. The order is therefore init, sample, time, configs. Timeouts
   cannot be written into the configs before the sample exists, and `cosmic-ray init` does not
   read the timeout.
5. The three-hour cap of amendment 6 still applies to each arm's `exec`. Harness lines after this
   amendment: 810 of 820.
