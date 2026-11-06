# E4 scope rationale: the lifecycle monitor group

- Written: 2026-11-06, for deconvolution-loop task 5.2. Read-only measurement of the frozen copy
  `.runs/e2-facdrone/source` (Facdrone `d49f48bd`); no test was run. Paths below are relative to it.

## Conclusion

The group is a sound second scope. It meets the four criteria (1,495 lines, one responsibility, a
store, tests that reach about 90 percent of its statements by static estimate) and its scattering is
concrete. Three findings weigh against it and should be accepted knowingly: patching is by name
across lazy imports, the review command reads the wall clock, and the two commands that no test
exercises on this host form residue. The survey's numbers hold except two, shown in the verification
list. The recommendation is to proceed with the residue declared.

## 1. Scope definition

| File | Lines | Statements | Functions | Write-site matches |
| --- | ---: | ---: | ---: | ---: |
| `shells/lifecycle_monitor_runner.py` | 397 | 189 | 14 | 3 |
| `shells/lifecycle_store.py` | 289 | 142 | 17 | 8 |
| `shells/cli/lifecycle_group.py` | 278 | 158 | 9 | 0 |
| `shells/lifecycle_evidence.py` | 215 | 114 | 11 | 1 |
| `shells/lifecycle_projection.py` | 161 | 61 | 9 | 0 |
| `shells/cppi/lifecycle_seams.py` | 97 | 56 | 5 | 0 |
| `shells/wiring/lifecycle_root.py` | 58 | 39 | 4 | 0 |
| total | 1,495 | 759 | 69 | 12 |

Intra-set call edges, by the E3 method: 89 (55 inside a file, 34 across files, 12 module pairs).
Of the 12 write-site matches only three are writes: the reading insert at
`lifecycle_monitor_runner.py:329-330` and the two event inserts at `lifecycle_store.py:170,173`. The
rest are `SELECT` executes, so the effect shell is small.

The responsibility is the signal lifecycle monitor. After a served decision the runner
(`lifecycle_monitor_runner.py:172`) plans due windows, assesses evidence, appends readings to
`signal_monitor_reading` and, when a breach persists, raises one flag through a writer bound into the
same transaction. The store (`lifecycle_store.py:66`) is the sole transition authority over
`signal_lifecycle_event`. Evidence projects retained ICs and the census baseline into monitor inputs.
Projection renders the analyst standing. Seams and root bind these to the daemon and the operator
commands, and the CLI group exposes `retire`, `enroll`, `admit`, `readings` and `review`.

Why it is one scattered responsibility:

- The same ledger-baseline lookup, `parse_ledger` then `{e.reference(): r ... admission_evidence(r)}`,
  is written twice verbatim (`cppi/lifecycle_seams.py:43-44`, `wiring/lifecycle_root.py:31-32`) and
  a third time as a set (`cli/lifecycle_group.py:254,258`).
- One baseline row is projected twice for two readers: `lifecycle_evidence.py:48-72` and
  `lifecycle_projection.py:73-84` read the same keys (`traded_ic`, `survival`, `states`).
- `signal_monitor_reading` has two owners: the runner reads and writes it directly
  (`:308`, `:329`), the store reads it (`lifecycle_store.py:150-160`), and `lifecycle_root.py:53` and
  `lifecycle_group.py:155,162` filter the same rows again by rules revision.
- `_rules_revision` (`lifecycle_monitor_runner.py:149`) is private yet imported by the CLI
  (`:147`) and the root (`lifecycle_root.py:43`), and the test support re-derives its formula.
- Call chains cross files: `lifecycle_readings` goes group, root `study_baseline`, store
  `census_references`, then group, evidence `live_evidence`, `read_evidence`, `_consumed`.
  `bind_lifecycle_monitor` (`lifecycle_seams.py:22`) fans out to six lazy imports.
- The store is built three ways: `lifecycle_root.py:16-23`, `lifecycle_group.py:60-61` (`retire`
  bypasses the root) and `cppi/binding.py:62`.
- The CLI reads `store._engine` twice (`lifecycle_group.py:176-177`).

## 2. Fit against the scope criteria

- Size: 1,495 lines, inside 800 to 2,500.
- One responsibility: yes, with the caveat that the CLI group also carries unrelated re-census
  admission (`lifecycle_admit`, `:103-132`).
- Store behind it: `signal_lifecycle_event` (insert-only; unique partial index on applied rows,
  `schema.py:600-626`) and `signal_monitor_reading` (`schema.py:629-652`), plus a read of
  `decision_evidence` (`lifecycle_evidence.py:75-92`, already in E1b's table set).
- Tests: 18 test-tree files import the seven modules (11 in `tests/unit`, 7 in
  `packages/facdrone-research/tests`); 15 are test modules, three are helpers. Counting
  `test_*` definitions in the lifecycle and monitor-worker test files gives 72.
- Executed share, estimated statically (name reachability from the tests, plus CLI commands invoked
  through `CliRunner`): 65 of 69 functions and about 90 percent of statements. This is an upper
  bound and not coverage. Unreached on this host: `lifecycle_admit`, `lifecycle_review`,
  `pod_lifecycle_standing` and root `lifecycle_store`. Only `test_lifecycle_full_loop_db.py` drives
  `admit` and `review` (`:182`, `lifecycle_loop_support.py:113`), and it imports `fcntl` through the
  research app, so it cannot run on Windows.
- Heavy patching (requirement 6.6 applies; all names must stay as looked up):
  - `test_lifecycle_evidence_intervals.py` patches only `lifecycle_evidence._consumed` (`:39`) and
    passes `engine=None`, so `read_evidence` must keep calling `_consumed` by module global. The
    same name is patched in `test_lifecycle_run_evidence.py:49,65` and
    `test_lifecycle_monitor_binding_db.py:64`, with `LiveEvidence.for_window` at `:65`.
  - `test_lifecycle_monitor_runner_db.py` patches the instance attribute `store.standing` (`:412`)
    and, through `facdrone_testing/core/lifecycle.py` (`method_revisions`), the globals
    `_lifecycle_store`, `read_parameters`, `_cited_baseline`, `_pod_of`, `configured_pods` of
    `lifecycle_group` and `baseline_view`, `live_evidence` of `lifecycle_evidence`. It also sets
    `runner._rules`, and `test_lifecycle_window_commits.py:87` replaces `runner._append`.
  - `lifecycle_loop_support.py:57-59` patches `project_root` in `lifecycle_group` and
    `lifecycle_root`.

## 3. Public surface to preserve

Derived by AST and string scan of `src/`, `packages/` and `tests/` (26 outside files: 7 prod, 18
test, 1 test-support). The apparatus deriver is authoritative and will add attribute and patch kinds.

| Module | Names (tag) |
| --- | --- |
| `lifecycle_monitor_runner` | `MonitorRunner` (both), `MonitorSources`, `MonitoredSignal`, `MonitorSummary`, `Emit`, `FlagWriter`, `Evidence` (test); private `_rules_revision`, `_flag_reason`, `_rules`, `_append`, `_evidence`, `_monitored` (test) |
| `lifecycle_store` | `LifecycleStore` (both), `bind_writers`, `TransitionRequest` (test); `_engine`, `standing` (test) |
| `lifecycle_evidence` | `baseline_view`, `live_evidence`, `read_evidence`, `Readings`, `LiveEvidence`, `_consumed`, `_consumed_query`, `_observed_day` (test) |
| `lifecycle_projection` | `signal_standing`, `monitor_label` (test) |
| `cppi/lifecycle_seams` | `bind_lifecycle` (both), `bind_lifecycle_monitor`, `LifecycleSeams` (test) |
| `wiring/lifecycle_root` | `lifecycle_store`, `pod_lifecycle_standing` (prod), `study_baseline` (both), `project_root` (test patch) |
| `cli/lifecycle_group` | `lifecycle_app` (prod); module globals patched by tests (see section 2) |

Unchanged but in oracle scope: `cli/app.py:29`, `cppi/binding.py:54,62`, `cppi/monitor_worker.py:12`,
`cppi/seams.py:36`, `cppi/snapshot_sources.py:27`, `facdrone_research/commands/app.py:13,16` and
`frozen_study.py:18,21`.

## 4. Oracle feasibility

Alphabet: store ops (`enroll`, `admit`, `flag`, `retire` through `bind_writers`, with and without a
caller connection; stale revision; reads `events`, `standing`, `admitted`, `open_flags`,
`overdue_flags`, `readings`); `tick` (set the injected clock); runner `run(as_of, decision_id, emit)`
over scripted sources (evidence values, `str` for not-run, a raising engine, a racing flag writer),
with the emitted events as a Level 1 observation; evidence `live_evidence` over seeded
`decision_evidence` rows; pure `signal_standing`, `monitor_label`, `labeled_reason`; seams
`bind_lifecycle_monitor` over a temporary project root with a ledger file; CLI `retire`, `enroll`,
`readings` via `CliRunner` (exit code and output).

Level 2 is available through the E1b mechanism, a `before_cursor_execute` listener
(`e1b_oracle.py:104,328`) on the embedded PostgreSQL engine. Concrete changes: add the two lifecycle
tables to the dumped table set (E1b dumps ten); the migration `0020_signal_lifecycle.py` already runs
in driver mode. The CLI builds engines of its own (`lifecycle_group.py:60`, `lifecycle_root.py:23`),
so the listener must sit on the `Engine` class, or the driver must patch `_lifecycle_store` as the
tests do. Wrap `MonitorRunner.run`, `LifecycleStore.transition` and the CLI commands. The database
fixture is `pg_engine` (`facdrone_testing/fixtures.py:144`), reproduced by E1b's `embedded_engine`.

Determinism risks, with where each enters:

- Wall clock: `LifecycleStore.__init__` defaults `now` to `datetime.now(UTC)` (`:69`), so the driver
  must inject a clock; `lifecycle_review` calls `datetime.now(UTC)` at `lifecycle_group.py:255` and
  feeds `schedule_outcomes(..., now.date())`. That command cannot be recorded byte-identically
  across days unless `datetime` is replaced, which a lazy import at `:234` defeats.
- Row order: `admitted` and `open_flags` use `select(...).distinct()` with no `ORDER BY`
  (`lifecycle_store.py:121,134`), so plan order, flag order and emit order rest on PostgreSQL
  behaviour. `events` orders by `known_at, revision` (`:80`); with a fixed clock a refused row and an
  applied row can tie.
- Ids: `reading_id` and `event_id` are content digests (`:143-146`, `:59-63`), so they are stable
  when the clock is.

## 5. Risks and what would make this a bad choice

- Patching by name across lazy imports. The CLI imports `baseline_view` and `live_evidence` inside
  the function (`lifecycle_group.py:146`); a regenerator that hoists imports, merges modules or
  inlines `_consumed` breaks the tests even if the recorded trace still replays clean. The seed tests must therefore run
  in full on every round.
- Repository gates outside the acceptance. `pyproject.toml:549-596` names these modules in an
  import-linter contract (store private to listed roots; `lifecycle_projection` may not import it),
  and `.sentrux/rules.toml` forbids a fan-out of 16 or more while `lifecycle_group.py` already has 14
  first-party imports. The scattering is partly deliberate (the files are marked "carved", for
  example `lifecycle_group.py:8`), so the natural edge-reducing move, merging files, may be refused
  by the project's own gates. The loop does not run them; add a structure check or accept the gap.
- Host: of 244 baseline failures (the spec prose says 236; the file lists 244, not reconciled)
  `baseline-failures-by-cause.json` places three in the importing set: `test_recensus_db` and
  `test_controlled_study_producer` (`fcntl`, whole module) and one `other`,
  `test_lifecycle_store_db...session_time_zone`. The 191-file E1b seed list contains only five
  lifecycle importers, so seeds must be re-derived by detection (task 2.2).
- CLI output formatting is observed verbatim (`lifecycle_group.py:68-71,213`), so any rewording is a
  Level 1 divergence; this protects, and it constrains.
- Kernel dependency: `lifecycle_evidence.py:24` imports `decode_consumed_evidence` from the E3 scope.
  Keep the frozen copy so the two scopes stay independent.
- Prediction fit: edges to remove are real (duplicates above), but the cross-file edge count of 34 is
  inflated by about 5 to 7 name collisions (`standing`, `for_window`), which makes the edge target
  easier to reach on paper.

Alternatives (the survey text was not available to me; these are inferences from the frozen copy).
Candidate 2, CPPI handover (`cppi/handover*.py` and roots, about 730 lines), falls under the
800-line floor and shares `state_repository` with E3, so it adds little independence. Candidate 3,
CPPI live decision events, centres on `live_decision_event`, a table E1b already exercises through
the same repository, so it would repeat E3's oracle more than test the loop on new ground.

## 6. Recommendation and decision

Proceed with candidate 1, declaring `lifecycle_admit`, `lifecycle_review`, `pod_lifecycle_standing`
and root `lifecycle_store` as residue, excluding `review` from the alphabet (clock), and adding the
repository gates to the structure check.

Owner chose candidate 1 on 2026-11-06 (menu).

Approved by the owner: 2026-11-06, menu answer "Confirm lifecycle monitor" after the four problems above were put to the owner; conditions: the four unreached functions (`lifecycle_admit`, `lifecycle_review`, `pod_lifecycle_standing`, root `lifecycle_store`) are declared residue, `lifecycle_review` (wall-clock read) is excluded from the oracle alphabet, and the lazy-import patching risk is carried as a kill criterion in the E4 plan (task 6.1).

## Numbers verified

All on the frozen copy, with `.runs/e2-facdrone/venv/Scripts/python.exe`, `PYTHONDONTWRITEBYTECODE=1`.

- Lines 1,495 (397, 289, 278, 215, 161, 97, 58), matching the survey: `len(read_text().splitlines())`.
- Statements 759: count of `ast.stmt` nodes per file.
- Definitions: the survey's 83 equals 69 functions (`FunctionDef` incl. methods and nested) plus 14
  classes. Mine: 69 functions.
- Edges: 89 by `docs/experiments/deconvolution/e3-evidence-regeneration/measures.py` pointed at the
  seven files (survey: about 101). Function-keyed 91; distinct name pairs 84.
- Write-site matches 12 (survey: 12) by the same script; real writes 3.
- Importing files 26 outside the set (7 prod, 18 test, 1 support) by AST scan of `src`, `packages`,
  `tests`; the survey's 18 matches the test-tree count.
- Executed share (65 of 69 functions, about 90 percent of statements): static name reachability, not
  coverage; measure it with coverage on the seven files in the first E4 task.
- Fan-out and gate figures: first-party `from` imports per file by AST; gate text read from
  `.sentrux/rules.toml` and `pyproject.toml`.
