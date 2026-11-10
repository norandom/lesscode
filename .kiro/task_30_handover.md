# Task handover 30: agnostic driver, task group 2 (pack admission) and the Slice B preregistration

Written 2026-11-10 (evening, Europe/Berlin). Spec: `.kiro/specs/agnostic-annotation-driver/`.
Predecessor: `.kiro/task_29_handover.md`. Start the next task group from this file.

## What was done

- Tasks 2.1 to 2.3 (`0ac7fb3`, one commit for three tightly coupled tasks):
  - `adapters/driver/admission.rs`: checks `json`, `size`, `shape`, `kinds_valid`, `templates` on the typed
    pack; a check that cannot run is reported as not run and keeps the pack out; results cached by pack hash.
  - `adapters/driver/cli.rs`: `deconvolute admit-pack --pack P --lang L [--controls K --source-root S]
    [--out R]`; exit 0 admitted, 1 rejected, 3 bad input (unreadable pack, language without a grammar, usage
    error) with nothing written. The receipt writer `emit_json` moved here.
  - The typed pack now requires the keys the prototype required (`unwrap_kinds`, `test_attribute_markers`,
    `noop_statement`, `statement_wrappers`, `operators`).
- Task 2.4: Slice B preregistration,
  [R66](../docs/research/2026-11-10/12-slice-b-pack-authoring-drift-preregistration.md). No model call made.
- Owner decision recorded earlier in the session: the five gate scope files may be added as test fixtures
  ("3.4 - yes").

## Evidence

- 381 tests pass with cargo-nextest (371 before); `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings` and the budgets check are clean locally.
- `native/tests/driver_admission.rs`: the four shipped packs and the two model-written packs of gate
  attempt 2 are admitted; four wrong packs are rejected with exactly `kinds_valid`, `shape`, `size`,
  `templates`; the three exit codes; requested controls fail closed.
- CI for the group 1 commits was green (`8852d11`, `dd83e0f`, `041ce24`). CI for this group's commits was
  not checked in this session.

Measured lines (total / production / longest function), design figure in brackets:

| File | Measured |
| --- | --- |
| `adapters/driver/admission.rs` (about 180) | 348 / 283 / 50 |
| `adapters/driver/cli.rs` (part of about 280) | 69 / 69 / 25 |
| `adapters/driver/pack.rs` (133) | 234 / 171 / 22 |
| `adapters/driver/mod.rs` (about 60) | 46 / 46 / 10 |
| `cli_deconvolute.rs` | 387 (was 399 of 400) |

Admission is 283 production lines against an estimate of 180; the spike's untyped port was 413.

## Deviations and things to know

- Task 2.2 is done only as specified for this group: the control options are accepted and fail closed
  (check `controls` reported as not run, exit 1). The replay itself needs the site rules and belongs to
  task 3.2.
- Admission differs from the prototype in one respect: when the pack does not have the pack shape,
  `kinds_valid` and `templates` are reported as not run instead of being evaluated on the untyped text. The
  failed-check names for the four wrong packs are the same.
- A language without a compiled grammar is bad input (exit 3), not a failed `kinds_valid` check.
- Tasks 2.1 to 2.3 share one commit; the rule is one commit per task.

## Slice B status

Queued second, not started. Preregistered in R66: five authoring runs per language for four languages, the
current endpoint first and Mistral second; claims on first-reply admission, identical mutants and repair.
Correction made there: the baseline needs the native engine for the mutant comparison, so it starts after
driver task group 3, not after group 2. Before the first call the amended guide (`pack-guide-v3.md`, one
sentence for `test_attribute_kinds`) must be hashed into R66 by dated amendment. Running it needs the owner's
go; the worst case with repairs does not fit the remaining model budget, so the run stops at the cap.

## Next steps

1. Task group 3: site rules and eligible sites (3.1), annotation validation and control replay (3.2),
   operators and classification (3.3), parity tests with the five fixtures and the name scan (3.4).
   Reference: the spike's `validator.rs`, `engine.rs` and `controls.rs`; retained receipts under
   `docs/experiments/annotation-driver/results/attempt2/`.
2. Fixtures for 3.4: the five scope files at their gate commits from
   `D:\BuildArtifacts\lesscode\annotation-driver\scopes-ro`, with origin, commit and licence noted and line
   endings pinned in `.gitattributes` (mutant hashes depend on the bytes; the scope files have CRLF or LF as
   checked out there, so record which).
3. After group 3: ask the owner for the go on the Slice B baseline.

## Standing rules

- Never manage the Podman machine; containers only. Stage files by name.
- Build with `CARGO_TARGET_DIR=D:\BuildArtifacts\cargo-target\lesscode-native`; check the lock diff after
  any resolve.
