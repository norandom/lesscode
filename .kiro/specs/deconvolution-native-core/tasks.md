# Implementation Plan

- [ ] 1. Establish the AST-backed Rust complexity budget checker and baseline budget configuration
  - _Boundary: BudgetsModule, LesscodeCoreCli_
  - _Requirements: 5.1, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [ ] 1.1 Implement the AST-backed Rust source complexity checker with offline lockfile linkage
  - Promote the pinned `syn` (`=2.0.119` with `full` and `visit` features) and `proc-macro2` (`=1.0.107` with `span-locations` feature) crates to direct dependencies of `lesscode-core` using only packages already present in the offline lockfile (`--offline`, with zero new crates or version/checksum drift).
  - Implement the `rust-syntax-units-v1` metric evaluator in `native/src/budgets.rs` to parse Rust source files into syntax trees and separate `#[cfg(test)]` modules and test items from production code before counting.
  - Count one logical syntax unit per production item declaration (`fn`, `impl` method, `struct`, `enum`, `trait`, `type`, `const`, `static`, `mod`) and one per production control-flow or branching construct (`if`, `match` arm, `for`, `while`, `loop`, `?`, `&&`, `||`, closure), and compute each production function's physical line span.
  - Evaluate per-file production physical lines, production logical syntax units, and maximum production function lines against the configured default ceilings (`400` physical lines, `300` logical syntax units, `80` lines per function) and any explicit per-file brownfield ceiling overrides, failing closed whenever a discovered `.rs` file is unlisted without a disposition or exceeds any ceiling.
  - Add synthetic fixture tests in `native/tests/budgets.rs` covering a compliant file, a file exceeding the physical-line limit, a file exceeding the logical-unit limit, a function exceeding the 80-line limit, an unlisted `.rs` file, a file with an explicit brownfield `keep-ceiling` disposition, and a file whose large `#[cfg(test)]` module keeps production counts within budget.
  - When this task is done, `cargo +1.98.1 nextest run --manifest-path native/Cargo.toml --locked -E 'binary(budgets)'` passes all synthetic budget-checker cases offline without modifying any existing `capability-compression-core` module.
  - _Boundary: BudgetsModule_
  - _Requirements: 5.1, 6.2, 6.3, 6.4_

- [ ] 1.2 Freeze the measured brownfield budget configuration and wire the `budgets` CLI subcommand
  - Measure the unchanged pre-existing `native/src/` tree (`compare.rs`, `scope.rs`, `metrics.rs`, `lib.rs`, `model.rs`, `main.rs`, `admission.rs`) with the `rust-syntax-units-v1` evaluator and freeze the exact measured production metrics and ceilings into `native/complexity-budgets.json` with git blob-hash provenance, explicit `keep-ceiling` dispositions for pre-existing `capability-compression-core` modules, a transitional `split-in-task-2` disposition for `admission.rs`, and the `+2` registration line allowance for `lib.rs`.
  - Register `pub mod budgets;` in `native/src/lib.rs` and add the `Budgets` subcommand (`lesscode-core budgets --config <file> --root <dir> --out <file>`) to `native/src/main.rs` while keeping existing `prepare`, `compare`, and `admission` subcommands unchanged.
  - Emit a deterministic receipt JSON (`schema_version`, `metric_version`, `root`, `config_path`, `files`, `unlisted_files`, `violations`, `status`) and companion `<out>.sha256` sidecar when `--out <file>` is supplied, returning exit code `0` when all files pass and exit code `1` on any budget violation or unlisted file.
  - Add an integration test in `native/tests/budgets.rs` that runs the budget evaluator over the live `native/src/` tree against `native/complexity-budgets.json`.
  - When this task is done, `lesscode-core budgets --config native/complexity-budgets.json --root native/src --out <tmp>/budgets-receipt.json` exits `0` with `status == "pass"` and zero violations, and the full native test suite passes under `cargo +1.98.1 nextest run --manifest-path native/Cargo.toml --locked`.
  - _Boundary: BudgetsModule, LesscodeCoreCli_
  - _Depends: 1.1_
  - _Requirements: 6.1, 6.2, 6.4, 6.5, 6.6_

- [ ] 2. Split deterministic receipt serialization from artifact admission and verify native CLI parity
  - _Boundary: ReceiptModule, AdmissionModule, BudgetsModule, LesscodeCoreCli_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 5.1, 5.2, 5.4, 6.1, 6.2, 6.5, 6.6_

- [ ] 2.1 Extract the order-preserving receipt serializer into `receipt.rs` and clear the transitional admission budget exception
  - Move the order-preserving `enum J` value representation (`Null`, `Bool`, `Int`, `Str`, `Arr`, `Obj(Vec<(&'static str, J)>)`), `sha256_hex`, `write_json_str`, `write_value`, and `write_receipt` from `native/src/admission.rs` into `native/src/receipt.rs` as a net-zero logic extraction that preserves insertion key order and `json.dumps(..., indent=1, ensure_ascii=True) + "\n"` byte parity.
  - Register `pub mod receipt;` in `native/src/lib.rs` and re-export `J`, `sha256_hex`, `write_json_str`, `write_value`, and `write_receipt` from `native/src/admission.rs` so existing callers remain source-compatible while `admission.rs` retains only binding schema parsing, path resolution, sidecar digest classification, and admission summary builders.
  - Remove the transitional `split-in-task-2` override for `admission.rs` and register `receipt.rs` with `disposition: null` in `native/complexity-budgets.json` so both `admission.rs` (`<= 280` physical lines) and `receipt.rs` (`<= 150` physical lines) are governed strictly by the default `400 / 300 / 80` ceilings.
  - Verify that `budgets.rs`, `admission.rs`, and `receipt.rs` all satisfy the default `400` physical-line, `300` logical-unit, and `80` function-line ceilings without any brownfield exception.
  - When this task is done, `lesscode-core budgets --config native/complexity-budgets.json --root native/src` exits `0` with `admission.rs`, `receipt.rs`, and `budgets.rs` carrying `disposition: null` under the strict default ceilings.
  - _Boundary: ReceiptModule, AdmissionModule, BudgetsModule_
  - _Depends: 1.2_
  - _Requirements: 2.2, 5.1, 5.2, 5.4, 6.1, 6.2, 6.5_

- [ ] 2.2 Verify receipt serialization parity, artifact admission classifications, and native CLI subcommand contracts
  - Extend `native/tests/admission.rs` with unit tests for `ReceiptModule` covering `Obj` insertion-order preservation on non-alphabetical keys, `indent=1` formatting, empty `[]` and `{}` containers, `ensure_ascii` escaping for control characters (`\b`, `\t`, `\n`, `\f`, `\r`, `\u001f`), `DEL` (`0x7f` -> `\u007f`), BMP and non-BMP surrogate pairs (`U+1F600` -> `\ud83d\ude00`), and companion `<path>.sha256` sidecar writing.
  - Verify `AdmissionModule` classification and exit codes across `verified`, `mismatch`, `missing`, and `no_bound_sidecar` bindings, confirming that missing or mismatched artifacts fail closed with exit code `1`, unbound artifacts pass under default admission (`exit 0`) and fail with exit code `2` under `--require-sidecar`, and historical drift in `docs/experiments/deconvolution/native-prototype/s2-bindings.json` reproduces `21 total / 11 verified / 5 mismatch / 1 missing / 4 no_bound_sidecar` with exit code `1`.
  - Extend `native/tests/cli.rs` to exercise the `budgets` and `admission` subcommands alongside existing `prepare` and `compare` subcommands through the single `lesscode-core` binary.
  - Run both binary/integration tests and doctests across the workspace to confirm zero regressions in existing `capability-compression-core` functionality.
  - When this task is done, `cargo +1.98.1 nextest run --manifest-path native/Cargo.toml --locked` and `cargo +1.98.1 test --manifest-path native/Cargo.toml --locked --doc` pass with zero failures, and `lesscode-core admission --bindings docs/experiments/deconvolution/native-prototype/s2-bindings.json --out <tmp>/s2-admission.json` reproduces the exact `21 / 11 / 5 / 1 / 4` counts and exits `1`.
  - _Boundary: ReceiptModule, AdmissionModule, LesscodeCoreCli_
  - _Depends: 2.1_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 5.2, 5.4, 6.6_

- [ ] 3. Relocate the shared Python recording core into `lesscode_baseline` and wire scope and adverse-control adapters
  - _Boundary: RecordingCore, ScopeE3Adapter, ScopeE4Adapter_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.3, 5.4, 5.5, 6.6_

- [ ] 3.1 Relocate `proto_core.py` to `src/lesscode_baseline/recording.py` and rebind the scope and adverse-control scripts
  - Move `docs/experiments/deconvolution/native-prototype/proto_core.py` verbatim into `src/lesscode_baseline/recording.py` (`251` written lines / `372` canonical lines) with zero logic changes, preserving top-level standard-library-only imports (`dataclasses`, `hashlib`, `io`, `json`, `os`, `pathlib`, `re`, `subprocess`, `sys`, `time`, `types`, `typing`) and function-local specimen imports inside specimen-venv callbacks (`e3_Turn`, `e4_Turn`) so `import lesscode_baseline.recording` succeeds in `.venv-task11`.
  - Delete `docs/experiments/deconvolution/native-prototype/proto_core.py` without leaving a re-export shim in `docs/experiments/deconvolution/native-prototype/`.
  - Update `docs/experiments/deconvolution/native-prototype/scope_e3.py`, `scope_e4.py`, `s10_controls.py`, and `s10_controls2.py` to place `LESSCODE / "src"` on `sys.path` and `import lesscode_baseline.recording as P`, keeping each scope adapter strictly limited to scope-specific target declarations, turn drivers, and normalization bindings while leaving `proto_replay_test.py`, `s10_measure.py`, and `s10_measure2.py` untouched as point-in-time historical prototype records.
  - Preserve the `RecordingCore` runtime contracts: subprocess timeout enforcement (`timeout=180` seconds returning exit code `124`), deterministic normalization (`Norm`), LF-normalized SHA-256 turn digest calculation (`sha256_lf`, `canon_bytes`), temporary per-run workspace isolation (`run_dir`), and fail-closed `verdict()` exit codes (`0` on full parity, `1` on operational/digest/golden divergence, `2` when `arm_exit == 2`).
  - When this task is done, `docs/experiments/deconvolution/native-prototype/proto_core.py` no longer exists, `import lesscode_baseline.recording` succeeds under `.venv-task11/Scripts/python.exe`, and `scope_e3.py`, `scope_e4.py`, `s10_controls.py`, and `s10_controls2.py` resolve `RecordingCore` exclusively from `lesscode_baseline.recording`.
  - _Boundary: RecordingCore, ScopeE3Adapter, ScopeE4Adapter_
  - _Depends: 2.2_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 5.3, 5.4, 5.5_

- [ ] 3.2 Create the recording core unit, three-stratum complexity budget, and specimen replay/control regression test suite
  - Create `tests/test_recording.py` with host-independent unit tests runnable under `.venv-task11` that verify the `verdict()` truth table (including `arm_exit == 2 -> 2` and Level-2 operational divergence `-> 1`), `call()` subprocess timeout handling returning exit code `124`, `Norm` deterministic pointer/path/timestamp scrubbing and idempotence, and `first_diff` structural path reporting.
  - Add a deterministic post-relocation three-stratum canonical line-count test in `tests/test_recording.py` (using `ruff==0.17.3 format --line-length 100 --stdin-filename` for Python files and `rustfmt --edition 2021` for Rust files) over `src/lesscode_baseline/recording.py`, `docs/experiments/deconvolution/native-prototype/scope_e3.py`, `docs/experiments/deconvolution/native-prototype/scope_e4.py`, `native/src/admission.rs`, and `native/src/receipt.rs` (plus the 16-line registration diff in `native/src/{lib,main}.rs`), asserting `C_app <= 1,364`, `B_app - C_app >= 200`, `C_total <= 3,003`, `scope_e3 < 280`, and `scope_e4 < 358` while keeping `native/src/budgets.rs` in the separate size-governance stratum.
  - Add environment-gated integration checks in `tests/test_recording.py` that run when `.runs/e2-facdrone/venv/Scripts/python.exe` and `.runs/native-prototype/proto2-*.json` are present (and skip cleanly with an explicit reason when absent) to verify E3/E4 frozen trace parity (`92a48f77...` and `0b2e383d...`), candidate acceptance (`cand` on E3, `cand-turn3` on E4), and the 9-control adverse suite (`s10_controls2.py`).
  - Verify that all 9 adverse controls (body-mutation, missing-artifact via `lesscode-core admission`, return-value mutation, exception-type change, call-order change, side-effect omission, stdout/stderr drift, exit-code drift, and turn-digest mismatch) fail closed with non-zero exit codes and specific divergence classifications.
  - When this task is done, `.venv-task11/Scripts/python.exe -m pytest -n 4 --dist loadfile tests/test_recording.py` passes all unit, three-stratum complexity, and specimen integration assertions, and `.runs/e2-facdrone/venv/Scripts/python.exe docs/experiments/deconvolution/native-prototype/s10_controls2.py` exits `0` with `all_failed_closed: true` across all 9 controls.
  - _Boundary: RecordingCore, ScopeE3Adapter, ScopeE4Adapter_
  - _Depends: 3.1_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.3, 5.4, 5.5, 6.6_

- [ ] 4. Reconcile solution-unlock governance records and execute full-slice verification
  - _Boundary: LesscodeCoreCli, AdmissionModule, BudgetsModule, RecordingCore_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.3, 3.7, 4.6, 6.4, 6.6_

- [ ] 4.1 Update prototype-gate and evidence-map records with post-migration receipts and run the full verification suite
  - Update `.kiro/specs/deconvolution-native-core/design.md` (§Supporting References) and `.kiro/specs/deconvolution-native-core/research.md` with the exact measured brownfield baseline metrics frozen in `native/complexity-budgets.json` and the post-split `admission.rs` / `receipt.rs` / `budgets.rs` verification metrics.
  - Verify and record in `.kiro/specs/deconvolution-native-core/spec.json` that all `U1–U7` unlock evidence items, the retained Attempt-1 overrun record (`1,808 > 1,400`), the Attempt-2 descoped pass receipt (`results/s10-u5-measurement-2.json`, `C_app = 1,193 <= 1,364`), and the approved inline evidence review (`results/s10-evidence-review-2.json`) remain intact and consistent with the migrated codebase.
  - Execute the complete native verification suite (`cargo +1.98.1 nextest run --manifest-path native/Cargo.toml --locked`, `cargo +1.98.1 test --manifest-path native/Cargo.toml --locked --doc`, `lesscode-core budgets --config native/complexity-budgets.json --root native/src`, and `lesscode-core admission --bindings docs/experiments/deconvolution/native-prototype/s2-bindings.json`).
  - Execute the complete Python verification suite (`.venv-task11/Scripts/python.exe -m pytest -n 4 --dist loadfile tests/test_recording.py` and `.runs/e2-facdrone/venv/Scripts/python.exe docs/experiments/deconvolution/native-prototype/s10_controls2.py`) while verifying via `git status` that unrelated working-tree changes (`src/lesscode_baseline/rules/evaluate.py` and `tests/test_rule_evaluation.py`) remain untouched.
  - When this task is done, all native and Python test suites, the `budgets` gate (`exit 0`), the `s2-bindings.json` admission check (`21 / 11 / 5 / 1 / 4`, `exit 1`), the three-stratum complexity check (`C_app <= 1,364`), and all 9 adverse controls pass with fresh verification evidence and clean spec governance records.
  - _Boundary: LesscodeCoreCli, AdmissionModule, BudgetsModule, RecordingCore_
  - _Depends: 1.2, 2.2, 3.2_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.3, 3.7, 4.6, 6.4, 6.6_

## Deferred Requirements & Scope Notes

- **Explicitly Deferred Rust-Port Items (Requirements 5.3, 5.4, 5.5)**:
  - Per `prototype-gate.md`, `slice-proposal.md`, and the owner-approved Attempt-2 descope (`spec.json.descope_decision`, R31), only the artifact-admission + receipt-serialization slice (`native/src/{admission,receipt}.rs`), the AST complexity budget checker (`native/src/budgets.rs`), and the shared Python recording core (`src/lesscode_baseline/recording.py` + `scope_e3.py` / `scope_e4.py`) are in scope for this specification.
  - The active baseline-served parts of **5.3** (subprocess timeout enforcement via `RecordingCore.call()`), **5.4** (deterministic JSON receipt emission in `ReceiptModule` and specimen turn digests in `RecordingCore`), and **5.5** (per-run workspace isolation in `RecordingCore`) are implemented and verified in Tasks 2 and 3.
  - The deferred Rust-port items — **5.3** Windows Job Object process-tree kill (`windows-sys`), **5.4** Rust specimen turn digest module (`turncheck.rs` / `pyjson.rs`, dropped in Attempt 2), and **5.5** `toml_edit` mutation, JUnit XML parsing (`junit-parser`), and cross-process file lock contention — remain intentionally unimplemented in Rust and locked behind a future prototype gate.
- **Working-Tree Preservation Note (Owner Rule 4)**:
  - Uncommitted changes in `src/lesscode_baseline/rules/evaluate.py` and `tests/test_rule_evaluation.py` belong to earlier owner work and must not be staged, modified, or reverted during task commits.
