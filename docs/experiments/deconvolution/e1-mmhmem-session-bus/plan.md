# E1a plan: trace oracle feasibility on mmhmem's session handoff responsibility

- Written: 2026-11-03, Europe/Berlin, before any harness code or execution.
- Hypothesis: H8(a) in [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md).
- Type: preregistered. Amendments must be dated and must not change the
  predictions below after results are seen.
- Runtime: native Windows only. Nothing executes in WSL.

## Question

Can a recorded, effect-complete trace oracle be built for a real stateful Rust
responsibility, and does it raise mutation kill rate materially above the
existing tests? This is oracle feasibility, not compression. No candidate
rewrite is produced in E1a.

## Why this scope, and what it does not test

Scope: `SessionBus::{resume, checkpoint, breadcrumb}` in
`src/services/session_bus.rs` (363 production lines per survey), plus the
thin tool caller `resume_or_handoff` in `src/mcp/tools.rs:402-445` and the
pure formatter `src/mcp/session_response.rs` as optional context. State: SQLite
tables `active_sessions` and `session_breadcrumbs`; effect log: the audit JSONL
file. No HTTP, no filesystem beyond the database and audit file, no in-memory
state. Ordering invariant of interest: `checkpoint` writes the audit line
inside the transaction before commit, so an audit line can survive a failed
commit (`session_bus.rs:198-228`).

Honesty note: by the survey's own counts this is a low-scatter scope (two
tables, two writers, three to four module hops). E1a establishes the oracle
mechanics on real code with transactions and cross-module callers. It does not
test oracle feasibility on a high-scatter scope; that is E1b on Facdrone R15-A.

## Frozen inputs

- Specimen: `D:\Source\mmhmem` at HEAD `8a9ce348b8927b60047f38a6e1bc6fd0e8f0f497`.
  Working tree: `src/` and `tests/` are clean; the only modification is
  `.github/workflows/ci.yml`, plus untracked release/CI scripts and a Linux
  binary, all outside the scope. Record `git status --short` in the receipt.
- The specimen checkout is never modified. All work happens in an isolated
  copy under `.runs/e1-mmhmem/original/` created from the tracked tree, with
  per-file SHA-256 of `src/` and `tests/` recorded before and after.
- Toolchain: rustc/cargo 1.98.1 stable MSVC (per `rust-toolchain.toml`),
  cargo-nextest 0.9.150, cargo-mutants and cargo-llvm-cov as installed on
  2026-11-03 (record `--version`), `llvm-tools` component. No nightly.
- Dependencies: `Cargo.lock` unchanged. The isolated copy may add `proptest`
  as a dev-dependency only; record the resolved version. No production
  dependency change.
- Seed tests (the "tests alone" arm): the inline tests in
  `src/services/session_bus_tests.rs` (11), `tests/handoff.rs` (3) and the
  session cases in `tests/multiprocess.rs` (the abrupt-exit and committed-
  checkpoint cases around lines 421 and 480). Spawning-binary tests outside
  these are not part of either arm; say so in the receipt.

## Oracle design

A Rust integration test file placed in the isolated copy's `tests/` directory
(research apparatus, not production; the `loc_caps` test only inspects `src/`):

1. Builds `Db::open(tempdir)` and `AuditWriter::new(tempdir/audit.jsonl)` and
   a `SessionBus` from them. No environment mutation.
2. Runs scripted operation sequences over the alphabet
   `resume(project, harness, task?)`, `checkpoint(...)`, `breadcrumb(...)`,
   including the trigger-based insert-failure fixture from
   `session_bus_tests.rs:150` so the rollback path is exercised. Sequences come
   from two sources: (i) seed scenarios transcribed from the seed tests,
   (ii) generated sequences from proptest over small value domains, with the
   state cap boundary (6,000 chars, 5,999, 6,001) and empty/duplicate projects
   included deliberately. The generator seed is fixed and recorded.
3. After every operation, records an observation: the returned value or
   error (type and message), a full ordered dump of both tables, and the new
   audit lines. Volatile fields are normalised, not ignored: `ts`, `epoch_ms`
   and `record_sha256` are masked to placeholders; time-based task IDs are
   replaced by first-seen ordinal labels so identity and reuse remain visible.
4. `E1_MODE=record` writes the observations as the golden trace into this
   experiment directory; `E1_MODE=replay` (default) asserts equality against
   the golden and reports the first divergent operation and field.
5. The golden is recorded once from the unmodified copy and then frozen by
   hash. Every later run, including every mutant, is replay only.

## Measurements

| Measure | Arm A: seed tests alone | Arm B: seed tests + trace replay |
| --- | --- | --- |
| cargo-mutants on `src/services/session_bus.rs` only: caught, missed, unviable, timeout | yes | yes |
| Kill rate = caught / (caught + missed) | yes | yes |
| cargo-llvm-cov region and line coverage of `session_bus.rs` | yes | yes |
| Three hand-written semantic mutants (below) | yes | yes |
| Wall-clock per arm, harness line count | recorded | recorded |

Hand-written semantic mutants, applied one at a time to separate copies:

- M1: move the audit write in `checkpoint` after `commit` (ordering change
  observable only on the failure path).
- M2: in `resume`, stop inheriting the stored task ID when `task` is `None`.
- M3: change the working-state cap from 6,000 to 5,999 characters.

## Predictions, fixed before execution

- P1: Arm A kill rate lies between 50% and 75%.
- P2: Arm B kill rate is at least 85% and at least 20 points above Arm A.
- P3: Arm B catches all three hand-written mutants; Arm A misses at least M1.
- P4: Arm B region coverage of `session_bus.rs` is at least 90%.
- P5: Arm B's generated sequences find at least one behaviour the seed tests
  do not observe (a table state or audit line that no seed scenario produces).

## Kill criteria

- P2 missed after the budget: H8(a) is rejected for this scope class and E1b
  is not started on the strength of E1a.
- The golden trace cannot be made deterministic without hiding a semantic
  field: record as inconclusive, not as pass.
- Any production file in the copy changed to make the oracle work: invalid run.

## Budget and stop rules

- Implementation: at most two agent sessions. Harness at most 500 research
  lines, counted separately from production.
- cargo-mutants: at most 90 minutes wall clock per arm; use `--timeout` and
  restrict the test command to the seed targets plus the harness. If a run
  exceeds the budget, report partial results as partial.
- Stop at the first effect that cannot be captured in-process; record it.

## Accounting

Production lines of the scope, harness lines, generated-sequence count, seed
scenario count, mutant counts per arm, coverage per arm, and all command
receipts with timestamps go to `results/receipt.json`. The golden trace,
mutant patches, raw cargo-mutants output and coverage reports are retained in
this directory. The `.runs/` copy is disposable.

## Not claimed

No compression, no candidate, no equivalence proof, no statement about
Facdrone or about TypeScript/JavaScript/Python oracles. A positive E1a admits
E1b and the E2 edge census; it does not admit E3 on its own.

## Amendment, 2026-11-03 (before execution): specimen HEAD drift

Written before the isolated copy was created. At execution time the specimen
`D:\Source\mmhmem` HEAD is `2f8f4e4eb008f8a9abb397e536246f2dd842c82e`
("ci(release): publish verified tagged binaries automatically"), one commit
after the frozen `8a9ce348b8927b60047f38a6e1bc6fd0e8f0f497`. `git diff --stat
8a9ce348 2f8f4e4` touches only `.github/workflows/ci.yml`,
`.github/workflows/release.yml`, `scripts/publish-ci-release.ps1` and
`scripts/test-publish-ci-release.ps1`; `src/`, `tests/`, `Cargo.toml` and
`Cargo.lock` are identical. The working tree now shows modified
`ci/scripts/probe_install.py` and `ci/scripts/test_probe_install.py`, untracked
`.openidea/` and `mmhmem-linux-x86_64`, all outside scope. To keep the frozen
input, the isolated copy is extracted with `git archive 8a9ce348...` (read-only
on the specimen) rather than `git archive HEAD`; both HEADs and the status are
recorded in the receipt. Predictions are unchanged.
