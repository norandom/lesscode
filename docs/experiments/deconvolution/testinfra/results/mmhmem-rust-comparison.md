# TestInfraDetector on the Rust specimen (deconvolution-loop task 2.3)

- Written and executed: 2026-11-07, Europe/Berlin, native Windows, plain user account.
- Specimen: `D:\Source\lesscode\.runs\e1-mmhmem\original` — the E1 frozen isolated copy of
  `D:\Source\mmhmem` (created 2026-11-03 one commit after frozen HEAD
  `8a9ce348…`, drift recorded in the e1 plan amendment). Read only; static scan
  only; nothing in the copy or in `D:\Source\mmhmem` was modified.
- Apparatus: `testinfra.py` unmodified from task 2.2 (commit `a461ee8`, sha256
  `1cbb55e2…`, identity in `receipt-2.3.json`). Commands and exit codes:
  `receipt-2.3.json` (label `mmhmem-rust-scope`). Log: `logs/mmhmem-rust-scope.log`.
- Outputs: `mmhmem-testinfra.{json,md}`. This file records the comparison against
  the specimen's known runner. Task 2.3 is the optional follow-up outside the
  gate's Python scope; per its text, **no claim about Rust regeneration follows**
  from anything here.

## Summary

| # | Known-runner fact (source) | Reality | Detector says | Result |
|---|---|---|---|---|
| 1 | Test runner | `cargo test` in CI (`ci.yml` lines 110, 143, 176); cargo-nextest configured by `.config/nextest.toml` (E1 ran cargo-nextest 0.9.150 on rustc/cargo 1.98.1 MSVC per `rust-toolchain.toml`) | `runner: unknown; plugins named: none; implied by gate flags: none` | not detected — runner discovery is pytest-config-driven (root `pyproject.toml`/`pytest.ini`/`setup.cfg`/`tox.ini`) and the specimen root has none |
| 2 | Test corpus | 22 integration test files in `tests/`, 18 inline `#[cfg(test)]` modules in `src/`, 128 `#[test]` functions in total | `0 test files`; `files_scanned: 27` (all Python); `test_files_total: 0` | not detected and no crash — Rust files are outside the `test_*.py`/`*_test.py` alphabet; 0 parse errors |
| 3 | The Python tests the repo does carry | `ci/` is a nested uv sub-project (`ci/pyproject.toml`, requires-python ≥3.14) whose `ci/uv.lock` pins `pytest >=8.0.2`; nine `ci/scripts/test_*.py` files match pytest's default patterns (two more `scripts/test-*.py` use hyphens and would not match) | 0 of them counted as test files; `import_roots: ., ci, ci/src, src` | missed by construction — the only `pyproject.toml` is `ci/pyproject.toml`, a packaging manifest without a pytest section, so no root testpaths exist |
| 4 | Scope seeds (scope `src/services/session_bus.rs`) | the tests that exercise the scope are the E1 seed tests: 11 inline tests in `src/services/session_bus_tests.rs`, 3 in `tests/handoff.rs`, the session cases in `tests/multiprocess.rs` (≈ lines 421 and 480) | seeds all 0 (`direct`, `via_support`, `via_fixture`, `via_production`, `recommended`); `scope_modules` empty; 0 unresolved references, 0 parse errors | not applicable by construction — a `.rs` scope maps to no Python module, so no import graph can bind |
| 5 | Marker expression | the runner has no pytest markers | suggested ` `` ` (empty); gate expressions: none | consistent, vacuously |
| 6 | External store | `docker-compose.yml` at the root (compose stack, `/healthz` readiness, audit-directory bind mount) | `external_store.facts` records `docker-compose.yml` (kind `file`); `embedded_postgres: false`, `migrations: false`, store env none | the one true positive — the file-level store signal generalises beyond Python; nothing deeper applies |

## 1. Runner

The detector names a runner only when a pytest config declares `testpaths` (or
gate flags imply one). The specimen root carries `Cargo.toml`,
`rust-toolchain.toml`, `clippy.toml` and no pytest-recognised config file, so
the honest `unknown` is returned. The nested `ci/pyproject.toml` was read
during validation: it is a uv packaging manifest (`dagger-io`, `uv_build`)
with no `[tool.pytest]` or `[pytest]` section, so even a recursive config scan
would not have named a runner for the root.

## 2. Corpus

`files_scanned: 27` counts the repository's Python files (18 in `ci/scripts`
and `ci/src`, 5 in `scripts`, 4 elsewhere — `deploy/check_worker.py`,
`scripts/check-*.py`). None is counted as a test file because test-file
collection runs under declared `testpaths`, which do not exist. The Rust
corpus (22 + 18 files, 128 `#[test]` functions) is invisible to every stage:
filename patterns, import graph, seeds.

## 3. The nested Python tests

The repository does ship nine pytest-style files (`ci/scripts/test_*.py`) plus
two hyphen-named ones (`scripts/test-install-menus.py`,
`scripts/test-worker-check.py`), owned by the `ci/` uv sub-project whose lock
pins `pytest >=8.0.2`. The workflows in `.github/workflows/` never invoke them
directly (they run through the Dagger CI program under `ci/src/`). For the
detector this is a scoping fact, not an error: it models one root-level Python
test configuration, and this specimen's Python tests live one manifest down.

## 4. Scope

`--scope src/services/session_bus.rs` was accepted without error; the run
treats it as a non-Python scope file: no module name is derived, no references
resolve, every seed count is 0 with 0 parse errors and 0 unresolved
references. The known seed tests exist and are enumerated in the table above;
binding them to the scope would require a Rust import model the apparatus does
not have.

## 5. Store signal

`external_store.facts` flagged `docker-compose.yml` as a store-adjacent file
— factually correct: the specimen's daemon ships a compose stack. The deeper
probes (embedded PostgreSQL, migrations, store environment variables) read
Python-specific signals and correctly report nothing.

## What this record does and does not license

- The detector ran unmodified, exited 0 and produced well-formed output on a
  Rust repository. Its boundary is now recorded on a second language:
  everything downstream of runner discovery (test files, seeds, references,
  markers) is empty or vacuous when no root pytest config exists, without
  errors or invented facts.
- The one cross-language signal is the file-level external-store fact
  (`docker-compose.yml`).
- Per the task text, no claim about Rust regeneration follows. Detection for a
  Rust scope would need a cargo/nextest runner model, `#[cfg(test)]`/`tests/`
  corpus discovery and a Rust import graph — none exists in the apparatus, and
  per the loop run report Rust tooling waits on its held-out row in any case.
