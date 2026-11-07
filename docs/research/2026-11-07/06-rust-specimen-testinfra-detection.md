# R32: TestInfraDetector on the Rust specimen — boundary recorded (task 2.3)

- Report date/time and timezone: 2026-11-07, Europe/Berlin.
- Execution interval: 2026-11-07 16:13:20–16:13:23 UTC (one static run; the
  ground-truth inventory commands are recorded in
  `receipt-2.3.json` facts).
- Status: completed.
- Spec: [tasks.md](../../../.kiro/specs/deconvolution-loop/tasks.md) task 2.3
  (optional follow-up outside the gate's Python scope; owner go given
  2026-11-07 in opencode).
- Hypothesis ID and predecessor: none (apparatus boundary check); predecessor
  is the task 2.2 validation on Facdrone
  (`docs/experiments/deconvolution/testinfra/results/facdrone-validation.md`).
- Type: exploratory.

## Decision

Task 2.3 is done: a recorded comparison exists
(`docs/experiments/deconvolution/testinfra/results/mmhmem-rust-comparison.md`).
It licenses no new capability. The 2.2 detector, run unmodified on the E1
frozen mmhmem copy, reports `runner: unknown`, 0 test files, 0 seeds, 0 parse
errors and exits 0 — the detector is root-config-scoped Python tooling and
behaves as designed on a repository outside its alphabet. No Rust
regeneration claim follows (task text), and nothing in the gate's verdicts
changes. This closes the spec's last open task.

## Hypothesis and falsification

Expectation before the run, from the 2.2 design: a detector keyed to pytest
configs and Python import graphs should return unknown/empty on a Rust
repository rather than crash, name a runner, or invent seeds. The record would
be falsified if the run had errored, emitted a runner name, counted Rust files
as test files or seeds, or reported parse errors. None of these occurred:
exit 0, `runner: unknown`, `test_files_total: 0`, all seed counts 0, 0 parse
errors, 0 unresolved references.

## Frozen inputs and method

- Specimen: `.runs/e1-mmhmem/original`, the E1 frozen isolated copy of
  `D:\Source\mmhmem` created 2026-11-03 one commit after frozen HEAD
  `8a9ce348b8927b60047f38a6e1bc6fd0e8f0f497` (drift recorded in the e1 plan
  amendment of 2026-11-03). Read-only static scan; the copy and the upstream
  checkout were not modified.
- Apparatus: `testinfra.py` from commit `a461ee8`, unmodified; sha256
  `1cbb55e2169c48ce0d9c137be781c4b41c98cca938552087d380bff9c0f9057c`.
  Interpreter: `.runs/e2-facdrone/venv/Scripts/python.exe` (the 2.2 receipt's
  interpreter).
- Scope: `src/services/session_bus.rs` (the E1 focus module).
- Known-runner sources (static reads, enumerated in `receipt-2.3.json`):
  `.github/workflows/ci.yml` (cargo test at lines 110/143/176),
  `.config/nextest.toml`, `rust-toolchain.toml`, the E1 plan toolchain record
  (rustc/cargo 1.98.1 MSVC, cargo-nextest 0.9.150), `ci/pyproject.toml`,
  `ci/uv.lock`, directory listings of `tests/` and pattern counts over `src/`.
- Commands, exit codes and timings: `receipt-2.3.json`; log
  `logs/mmhmem-rust-scope.log`; outputs `mmhmem-testinfra.{json,md}`.

## Evidence and results

| Known-runner fact | Reality | Detector |
|---|---|---|
| Runner | cargo test (CI); nextest via `.config/nextest.toml` | unknown |
| Corpus | 22 `tests/*.rs` + 18 inline `#[cfg(test)]` modules + 128 `#[test]` | 0 test files (27 Python files scanned) |
| Nested Python tests | `ci/` uv sub-project, pytest ≥8.0.2 in `uv.lock`, 9 `ci/scripts/test_*.py` | 0 counted (no root testpaths) |
| Scope seeds | 11 inline `session_bus_tests.rs` + 3 `tests/handoff.rs` + `tests/multiprocess.rs` session cases | all seed counts 0, scope_modules empty |
| Markers | none (cargo has no pytest markers) | empty suggestion |
| External store | `docker-compose.yml` compose stack | file fact recorded — the one true positive |

Full table with causes: `mmhmem-rust-comparison.md`.

## Limits and interpretation

One run, one scope file, one specimen. The counts are text-pattern counts
(`#[cfg(test)]`, `#[test]`), not cargo's own inventory; the nested `ci/` uv
project was read statically, never executed; the E1 copy is one commit drifted
from the frozen HEAD (recorded 2026-11-03, outside the scanned scope). The
"one true positive" store signal is a file-name heuristic firing correctly,
not cross-language detection. Everything here is specific to this specimen and
this apparatus version.

## Tool/spec consequence and next gate

Task 2.3 closes the last open task of `deconvolution-loop`; the detector
stays Python-scoped and no follow-on machinery is justified by this record.
Rust-side detection would be new apparatus behind a future gate, and the loop
run report already holds Rust tooling behind its held-out row. No next gate is
opened. Marking the spec phase complete is the owner's call.

## COBESY check

Outcome first: boundary recorded, task closed, no capability claimed. The
expert reader can replay the single command from `receipt-2.3.json` against
the frozen copy. The disproof condition was stated in advance and did not
fire. The empty-result findings are reported as facts with causes, separated
from interpretation; negative space is safe to report; no inflated claims —
the record explicitly denies any Rust regeneration implication.
