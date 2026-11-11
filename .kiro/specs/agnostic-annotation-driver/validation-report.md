# Validation report: agnostic-annotation-driver

Written 2026-11-11 (Europe/Berlin) for task 8.3, following the `kiro-validate-impl` protocol inline in the main
conversation (owner rule: no subagents). The reviewer is the implementer; this is not an independent review.

## DECISION

GO for task groups 1 to 8 as implemented, with one acceptance criterion met only in part (5.7) and stated
below. No release contains the driver; releasing is a separate owner decision.

## MECHANICAL_RESULTS

| Check | Result |
| --- | --- |
| Tests | PASS: `cargo nextest run` in `native/`, 382 of 382, exit 0, at the working tree of commit `c2a0a1b` |
| Lints | PASS: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` |
| Budgets | PASS: `lesscode-core budgets --config complexity-budgets.json --root src`, exit 0; twenty driver files, largest 348 lines |
| TBD/TODO grep | CLEAN in `src/adapters/driver`, the adapter modules and `cli_deconvolute.rs` |
| Secrets grep | CLEAN; no endpoint key in the committed tree (checked against the two local key files) |
| Smoke | PASS: `--version`; `deconvolute analyze` and `mutate --limit 5 --seed 1` on a Python fixture (exit 0, 233 sites, 5 mutants); an unsupported language exits 3; fixtures unchanged |
| CI | see the last section |

## INTEGRATION

- Cross-task contracts: the four commands share one pack type, one site rule and one operator engine.
  `run` reuses `mutate`'s generation and the annotation validator; admission and the control replay use the
  same site rule (parity tests `driver_parity.rs`).
- Shared state consistency: the outcome store is keyed by original hash, mutated hash and configuration
  identity; errors are never stored (`driver_run.rs`).
- Boundary audit: nothing in `src/adapters/driver` names a language or a project (name scan in
  `driver_parity.rs`); the removed text fragments are absent from `src/adapters` (`driver_contract.rs`).
  Driver tests read nothing outside `native/`.
- Found during validation and fixed: a void closing run did not say what failed (commit `0d0da64`).
- Found during validation and documented, not prevented: copies that share a build directory through the
  environment contaminate each other; the closing run voids such a run (R75, `docs/driver.md`).

## COVERAGE

| Requirement | State | Evidence |
| --- | --- | --- |
| 1 Language packs | met | `driver_admission.rs`, shipped packs admitted in the suite, `admit-pack` |
| 2 Site discovery and analysis | met | `driver_parity.rs` (sites equal the prototype's on five scopes), `cli_deconvolute.rs` |
| 3 Mutant generation | met | R67 (mutants equal under the gate packs; 38 repeats dropped by 3.4), seeded limit |
| 4 Running mutants | met | `driver_run.rs`; R73: 556 of 556 classes as the prototype |
| 5 Protection of the examined code | 5.1 to 5.6 met; **5.7 in part** | No scope file changed in any run of R73 or R75. 5.7: no process is left after a normal end or a time limit; a driver killed from outside can leave a project command running, because there is no signal handler (design Amendment 2) |
| 6 External annotations | met | `annotations.rs`, 98 controls replayed, annotated run in `driver_run.rs` |
| 7 Replacement of specimen-bound code | met | six files deleted (`964921c`); mapping and losses in `contract.rs` and R75; earlier records amended, not altered |
| 8 Single runtime, size, platforms | met | one binary; R65 sizing; 13.26 MB against 10.99 MB; licences recorded; five CI platforms. 8.4: the shipped packs are admitted against the compiled grammars in the test suite, so a grammar update that breaks a pack fails the suite |
| 9 Evidence against the prototype | met | R67, R73, R75; research index updated; 9.6: Slice B has a dated status in the roadmap (2026-11-11) |

Coverage gaps: 5.7 as stated. Requirement 9.1 holds with the declared differences (dropped repeats, two
Python pairs adopted by the owner).

## DESIGN

- Architecture drift: recorded in design Amendments 1 to 3 (location under `adapters/driver/`, own process
  supervision, `gaps` as a pack key, one contract file instead of per-adapter implementations, a new
  `NotRun` outcome).
- Dependency direction: the driver depends on `adapter_traits` for carrier types and on `specimen_runner`
  for hashing; nothing outside `adapters/` and the command line depends on the driver.
- File Structure Plan vs actual: twenty files against eleven planned; every file within its budget.

## OWNERSHIP

LOCAL.

## UPSTREAM_SPEC

N/A. `constraint-guided-compression` (Slice B) consumes the driver's admission; its status is in the roadmap.

## BLOCKED_TASKS

None. Open items outside the task list, for the owner:

- A signal handler, to close 5.7.
- `PureCoreVerifier::compare` on the adapters and `NativeMutantGenerator::generate` report success without
  running anything (R75). Not reachable from the command line.
- Operators for a changed argument and a changed literal inside a call (three of the seven lost controls).
- CI does not build `tools/guide-optimizer/`.
- A release that contains the driver.

## CI

Run 42978911372 for commit `c2a0a1b` (which contains every code change of task group 7 and the fix
`0d0da64`): success on all five jobs: Linux x86_64 through Dagger, Linux aarch64, Windows, macOS on Intel and
on Apple silicon. Finished 2026-11-11, 08:29 local. The commits after it change documents only.
