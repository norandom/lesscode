# Test infrastructure

- Repository: `D:\Source\lesscode\.runs\e2-facdrone\source`; 1860 Python files, 890 test files.
- Scope: `src/facdrone/shells/decision_evidence_publication.py`, `src/facdrone/shells/decision_evidence_store.py`, `src/facdrone/shells/decision_evidence_reader.py`, `src/facdrone/shells/decision_input_slice_store.py`, `src/facdrone/shells/decision_slice_publisher.py`.

## Runner

- Runner: pytest; plugins named: none; implied by gate flags: pytest-timeout, pytest-xdist.

| rootdir | config | testpaths | markers |
|---|---|---|---|
| `.` | `pyproject.toml` | tests | parity, port_baseline, live_broker, db |
| `packages/facdrone-dash` | `packages/facdrone-dash/pyproject.toml` | tests | parity, port_baseline, live_broker, db |
| `packages/facdrone-factor-comm-term-structure` | `packages/facdrone-factor-comm-term-structure/pyproject.toml` | tests |  |
| `packages/facdrone-factor-kit` | `packages/facdrone-factor-kit/pyproject.toml` | tests | parity, port_baseline, live_broker, db |
| `packages/facdrone-factor-short-reversal` | `packages/facdrone-factor-short-reversal/pyproject.toml` | tests |  |
| `packages/facdrone-kernel` | `packages/facdrone-kernel/pyproject.toml` | tests, examples/factor_plugin/tests | parity, port_baseline, live_broker, db |
| `packages/facdrone-reporting` | `packages/facdrone-reporting/pyproject.toml` | tests | parity, port_baseline, live_broker, db |
| `packages/facdrone-research` | `packages/facdrone-research/pyproject.toml` | tests | parity, port_baseline, live_broker, db |

## Fixtures and plugins

- conftest files: 9; pytest_plugins declared in 9 files: facdrone_testing.fixtures, tests.unit.test_thin_protocol_journal.
- Plugin `facdrone_testing.fixtures` (packages/facdrone-testing/facdrone_testing/fixtures.py): fixtures _logging_state_restored, _migrated_schema, capture_repo, clean, evidence_engine, git_checkout, pg_engine, prices_engine, project_params, project_path, reader_engine, sim_engine, store.
- Plugin `tests.unit.test_thin_protocol_journal` (tests/unit/test_thin_protocol_journal.py): fixtures none.

## External store

- Embedded PostgreSQL: True; migrations: True; store environment variables: FACDRONE_TEST_PG_DSN, FACDRONE_TEST_PG_EMBEDDED, FACDRONE_TEST_PG_KEEP.
- quoted-empty-arg: `-c listen_addresses='' -c fsync=off -c full_page_writes=off` at `<site-packages>/facdrone_data/testing/pg_server.py:66`
- symlink: `Path.symlink_to` at `packages/facdrone-testing/facdrone_testing/paths.py:49`
- posix-api: `os.geteuid` at `packages/facdrone-testing/facdrone_testing/fixtures.py:57`
- captured-pipes-on-daemon-start: `server._run(pg_ctl, -D, ... 'start')` at `<site-packages>/facdrone_data/testing/pg_server.py:67`

## Scope references and seeds

- Test files referencing the scope directly: 30; via test support: 1; via fixtures: 0; recommended: 30; production-import upper bound adds 384.
- Unresolved references: 1; parse errors: 0.

## Marker expression

- Suggested: `not port_baseline and not live_broker` (exclude a marker whose description declares it opt-in, or names a hardware, network, broker or port resource without saying the tests skip on their own; declaration order).
- Gate expressions: `db and not port_baseline and not live_broker`, `not port_baseline and not live_broker`; agreement: {'db and not port_baseline and not live_broker': True, 'not port_baseline and not live_broker': True}.

## Baseline deselections

- Recorded node failures: 236 (resolved 236, unresolved 0, ambiguous 0); collection errors: 8.
- Applicable to the recommended seeds: 6; by cause: {'other': 0, 'symlink-privilege': 1, 'fcntl/POSIX': 5, 'file-lock': 0}.
- Applicable to the 191 supplied seed files: 24.
