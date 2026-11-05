# TestInfraDetector validation on Facdrone (deconvolution-loop task 2.2)

- Written and executed: 2026-11-05, Europe/Berlin, native Windows, plain user account.
- Specimen: `D:\Source\lesscode\.runs\e2-facdrone\source` (frozen copy, read only). Static only: `testinfra.py`
  parses files and never imports or runs Facdrone code or tests.
- Commands and exit codes: `results/receipt.json` (labels `testinfra-tests`, `testinfra-help`,
  `facdrone-e3-scope`, `facdrone-e1b-scope`, `facdrone-compare`; note `red-phase`). Logs: `results/logs/`.
- Outputs: `facdrone-testinfra.{json,md}` (the five-file E3 scope, the validation scope of the task),
  `facdrone-e1b-scope-testinfra.{json,md}` (the eight-file E1b scope, which the 191 seeds were measured
  against), `facdrone-comparison.json` (every number below, computed by `compare.py` from the hand lists).

## Summary

| Hand list | Hand | Detected | Result |
|---|---|---|---|
| Test files importing or referencing `decision_evidence_publication` (E3 `t4-mutant-sample.json`) | 19 | 19 | identical, 0 extra, 0 missing |
| Baseline failures, node level (E1b `selection-map.json` `deselect`) | 236 | 236 | identical sets after resolving the junit-style ids of `baseline-failures-by-cause.json`; 0 unresolved, 0 ambiguous |
| Deselections applicable to the 191 seed files | 24 | 24 | identical (symlink-privilege 11, other 8, fcntl/POSIX 5, file-lock 0) |
| Marker expression | `not port_baseline and not live_broker` | `not port_baseline and not live_broker` | identical, from the marker descriptions; both gate-script expressions also found in `scripts/gates.sh` |
| Seeds, static candidates (E1b `seeds-static-candidates.txt`) | 115 | 116 | 114 shared; 1 hand-only, 2 detector-only, all three explained below |
| Seeds, coverage-confirmed (E1b `seeds-from-coverage.txt`) | 191 | 101 of 191 in the recommended static set; 191 of 191 inside the static upper bound (548 files) | gap is a coverage-only fact, explained below |
| External store and the two Windows shims (`e2_winshim.py`) | 2 gaps | both gaps detected with file and line | see Host facts |

## 1. Importing test files: 19 of 19

The 19 files of `t4-mutant-sample.json` are the test files (`tests/` or `packages/*/tests/`) that import or
reference `facdrone.shells.decision_evidence_publication`, taken from `public-surface.json` by `t4_arm.py`.
The detector's `scope_references.by_module["facdrone.shells.decision_evidence_publication"]` over the
five-file scope lists the same 19 files. No difference.

Over all five scope modules the detector finds 30 test files (`seeds.direct`), the same 30 that
`public-surface.json` tags `test` once its four `facdrone_testing` helper modules and the non-test helper
`packages/facdrone-research/tests/lifecycle_loop_daemon.py` are set aside. The detector reports those
five as support files, not test files. The one unresolved reference matches surface.py too:
`decision_evidence_publication.row` read in `tests/contracts/test_evidence_compaction_equivalence.py`.

## 2. Baseline failures: 236 of 236, 24 applicable

`baseline-failures-by-cause.json` records 244 entries in junit form (`tests.unit.test_x::name[param]`,
classes in the dotted part, package-relative for the satellite groups). The detector resolves each id
against every pytest rootdir candidate and accepts a resolution only if the AST of the file defines the
function, inside the named classes where the id has any. Result: 236 node ids, set-equal to the 236 in
`selection-map.json` `deselect`, and 8 collection errors (`::tests.test_…`, cause `fcntl/POSIX`, all in
`packages/facdrone-research/tests/`). The E1b deselect list also leaves out those 8, because a module
that cannot be collected has no node to deselect. 244 = 236 + 8.

Applicable deselections are those whose file is a seed file:

- to the 191 coverage seeds (`--seeds-file`): **24**, the number `run_e3.py` prints;
- to the detector's own recommended static seeds: 6 for the five-file scope and 18 for the eight-file
  scope. The difference comes from the seed set, not from the deselection logic.

## 3. Marker expression: identical

Rule, stated in `testinfra.py`: a declared marker is excluded when its description says it is opt-in
("opt-in", "opts in", "never part of the gate"), or when it names a hardware, network, broker or port
resource and does not say its tests skip by themselves. A store resource (database) is reported but does
not exclude a marker, because the fixture plugin may provision the store. Markers keep their declaration
order. Signals on Facdrone (descriptions joined over the six configs that declare markers):

- `port_baseline`: opt-in → excluded. The description never uses the word "port" as a resource; the
  name `port_baseline` does not match, since the regex requires a word boundary after `port`.
- `live_broker`: opt-in, resource broker → excluded.
- `db`: resource store, self-skip ("Skips off a configured host") → kept.
- `parity`: no signal → kept.

Suggested `not port_baseline and not live_broker`, equal to the hand expression. `scripts/gates.sh` holds
the same expression for the fast tier and `db and not port_baseline and not live_broker` for the opt-in
DB tier. Both negate the same two markers.

## 4. Seeds

### 4a. Against the 115 static candidates (eight-file E1b scope)

The detector's static rule: test files that reference the scope directly (108), plus test files that
import a test-support module whose support-only import closure references the scope (9), plus test files
that request a conftest or plugin fixture whose body uses such an import (0 new: only `capture_repo`
qualifies, and every file requesting it is already counted). Files the gate scripts `--ignore` are removed
(1: `packages/facdrone-research/tests/test_lifecycle_full_loop_db.py`, from `skip=(--ignore=…)` in
`gates.sh`, which seeds.md also excludes). Recommended: 116.

| File | In | Cause |
|---|---|---|
| `tests/unit/test_decision_arm_check.py` | hand only | It names `src/facdrone/shells/cppi/state_repository.py` in a path string (line 147) and imports no scope module. A text search finds it; the reference kinds do not. Coverage does not confirm it (not among the 191). This is a ground-truth difference, and the detector is right not to count it. |
| `tests/unit/test_acquisition_acceptance.py` | detector only | Reaches the scope through `tests/unit/acquisition_daemon_fixtures.py`, a support module under `tests/`. The hand rule counted only `facdrone_testing.core.*` helpers. Coverage confirms the file (in the 191). |
| `tests/unit/test_depot_durability.py` | detector only | Same cause, through `tests/unit/depot_durability_fixtures.py`. Coverage confirms it (in the 191). |

### 4b. Against the 191 coverage-confirmed seeds

- Recommended static set: 101 of the 191 (precision 101/116).
- Static upper bound (recommended plus `via_production`: test files whose in-tree import closure through
  production modules reaches the scope, 548 files): 191 of the 191, none outside. Precision 191/548.
- **Gap, 90 files.** They reach the scope only through production modules (state codec, cppi cell and
  daemon, sim drivers, wiring), the "known gap of the static list" that seeds.md describes. Static import
  analysis cannot say which of the 548 upper-bound files execute a scope line. Coverage measures that, so
  closing the gap needs coverage data. The detector accepts a recorded seed list through `--seeds-file`,
  as it accepts the recorded baseline-failure file, and does not run the suite.
- **15 recommended files not in the 191.** They are exactly the hand list's 16 unconfirmed candidates
  minus `test_decision_arm_check.py`. Static cause hints from `compare.py`:
  - 1 file deselected by the marker expression: `test_holdings_g6_gate.py`.
  - 3 files skipped by the fixture plugin's operator-DB rule (`*_e2e` or `db` without `pg_engine` on the
    embedded server): `test_alpha_sim_year_e2e.py`, `tests/parity/test_alpha_sim_e2e.py`,
    `tests/parity/test_report_bundle_e2e.py`.
  - 11 files not explained statically. Their tests run, but coverage saw no scope line inside a test
    context. Coverage contexts exclude import-time execution, so a file whose only contact with the scope
    is an import executes nothing that counts. This is a coverage observation, not a detector defect.
    Among them is `packages/facdrone-research/tests/test_weight_study.py`, one of the 19 importing files.

## 5. Host facts: external store and the Windows shims

Detected statically: the plugin `facdrone_testing.fixtures`, from module-level `pytest_plugins` in 8
conftests. The ninth conftest, `tests/parity/conftest.py`, declares none. The occurrence in
`facdrone_research/commands/plugin_package.py` sits inside a conftest template string, so it is
correctly not counted. The plugin defines 13 fixtures, some re-exported from `facdrone_testing.core.*`. Also found: the `pgembed` import, Alembic (`alembic.ini`,
`migrations/`), and the store environment variables `FACDRONE_TEST_PG_DSN`, `FACDRONE_TEST_PG_EMBEDDED`
and `FACDRONE_TEST_PG_KEEP`. A second `pytest_plugins` in a non-root test module was found as well:
`tests/contracts/test_client_acceptance.py` → `tests.unit.test_thin_protocol_journal`. Gate flags imply
pytest-xdist and pytest-timeout.

Host hazards against `e2_winshim.py` (`--site-packages` follows `facdrone_data.testing.pg_server` out of
the tree):

| Shim (e2_winshim.py) | Detected hazard |
|---|---|
| 1. `os.geteuid` absent on Windows | `posix-api os.geteuid` at `packages/facdrone-testing/facdrone_testing/fixtures.py:57` |
| 2. `listen_addresses=''` passed without a shell | `quoted-empty-arg` at `<site-packages>/facdrone_data/testing/pg_server.py:66` |
| 2, second part: captured pipes block after `pg_ctl -w start` | `captured-pipes-on-daemon-start` at `<site-packages>/facdrone_data/testing/pg_server.py:67` |
| (no shim) | `symlink Path.symlink_to` at `packages/facdrone-testing/facdrone_testing/paths.py:49`. It is the same API class as the `symlink-privilege` cause (136 recorded failures); the link between them is not established here. |

Without `--site-packages`, the two `pg_server.py` hazards stay out of view: that module is a dependency
installed in the venv, not part of the tree.

## Limits

- Static reference kinds only (import, module alias, attribute on an alias, setattr/patch by name). A
  string mention of a scope path is not a reference (see 4a). Dynamic imports (`importlib`) are not followed.
- Relative imports inside an `__init__.py` resolve one package level too high. Facdrone test files use
  absolute imports, so the validation is unaffected.
- The fixture-reach level reads the names a fixture body uses and does not follow fixture-to-fixture
  dependencies. Autouse fixtures that reach the scope are reported and not counted (none on Facdrone).
- The `compare.py` cause hint reads the `pg_engine` request from the test signature only.
