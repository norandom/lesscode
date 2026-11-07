# Task Group 3 Handover — deconvolution-native-core production tasks

Date: 2026-11-07. Scope: `.kiro/specs/deconvolution-native-core/tasks.md` tasks 1.1–4.1, executed in the foreground per the 2026-11-07 owner rules. All four task groups items COMPLETE.

## What was done

- **Task 1.1 — 0c3073b**: `rust-syntax-units-v1` AST complexity checker (`native/src/budgets.rs`, syn + proc-macro2 promoted offline from the existing lockfile); synthetic fixture tests.
- **Task 1.2 — 62c2e63, 28a8650**: brownfield budgets frozen into `native/complexity-budgets.json` (blob-hash provenance, `#[cfg(test)]` split); `budgets` CLI subcommand (`exit 0/1/3`); budgets.rs re-frozen at the owner-amended 330 with the measured-history correction (checker measured 354; the first "324" was a non-blank PowerShell count).
- **Task 2.1 — deac232, 4fe74db**: order-preserving receipt serializer extracted verbatim to `native/src/receipt.rs` (public surface exactly `J`/`sha256_hex`/`write_receipt`); `admission.rs` re-frozen at 272/72/81; combined 422 ≤ 430 (design A1), `disposition: null`.
- **Task 2.2 — 1ab1569**: receipt parity tests byte-identical to Python `json.dumps(indent=1, ensure_ascii=True)` (incl. DEL, BMP, surrogate pairs), admission classification/exit-code matrix (0/1/3 only; no `--require-sidecar`), R28 smoke reproducing `21/11/5/1/4 exit 1`, budgets/admission subcommand tests through the single binary.
- **Task 3.1 — f82d92f**: `proto_core.py` → `src/lesscode_baseline/recording.py` verbatim (git rename 100%, no shim); four adapters rebound (`scope_e3.py`, `scope_e4.py`, `s10_controls.py`, `s10_controls2.py`); frozen drivers (`s10_measure*.py`, `proto_replay_test.py`) untouched.
- **Task 3.2 — d6a4b1f**: `tests/test_recording.py` (19 tests): verdict/call/Norm/first_diff truth tables; three-stratum canonical line counts (ruff==0.17.3 / rustfmt, hooks frozen +2/+15): `C_app = 1,203 ≤ 1,364`, saved 502 ≥ 200, `C_total = 2,701 ≤ 3,003`, e3 share 175.5 < 280, e4 share 216.5 < 358; env-gated specimen integration (double-record identities `92a48f77…`/`0b2e383d…`, E3 `cand` + E4 `cand-turn3` round-21 acceptance, 9-control adverse suite).
- **Task 4.1 — 815eade**: design.md §Supporting References evidence row (6.3/6.4) updated with frozen metrics; research.md implementation-verification record appended; spec.json evidence extended (post-migration verification, Attempt-1/Attempt-2 retention, owner-WIP preservation).

## Evidence pointers

- Full-suite verification (2026-11-07, fresh): fmt 0 · clippy `-D warnings` 0 · **nextest 123/123** · doc tests 0 failures · budgets self-run **exit 0** · admission R28 smoke **21/11/5/1/4, exit 1** · `pytest -n 4 --dist loadfile tests/test_recording.py` **19/19** (920 s) · standalone `s10_controls2.py --with-pg` **exit 0, all_fail_closed: true, 9/9**.
- Frozen budgets: `native/complexity-budgets.json` (compare 805/230/253, scope 863/300/265, metrics 886/286/220, model 331/73/24, lib 587/180/302 +2 registration, main 251/82/63, budgets 330, admission 272/72/81, receipt 150).
- Records: `.kiro/specs/deconvolution-native-core/{design,research,spec}.json` (updated at 815eade); unlock receipts unchanged under `docs/experiments/deconvolution/native-prototype/results/` (attempt-1 `1,808`, attempt-2 `1,193` retained point-in-time; post-migration `C_app 1,203`, +10 delta fully attributed).
- New untracked run evidence (intentionally not committed): `docs/experiments/deconvolution/{e3-evidence-regeneration,e4-lifecycle}/results/sql-divergences-round21.json`.

## Current state

- Working tree: only owner WIP modified (`src/lesscode_baseline/rules/evaluate.py`, `tests/test_rule_evaluation.py`) — never staged, never touched by task commits. Everything else clean.
- Spec phase: `tasks-generated` with all tasks implemented and verified; `solution_unlock` remains `proceed` with approved inline evidence review. Production code paths: `native/` gate-locked by budgets; Python recording core at `src/lesscode_baseline/recording.py`.

## Next steps (suggestions, not started)

1. Owner reviews this handover and the 815eade record updates; optionally marks the spec phase complete in `spec.json`.
2. Remaining deferred items stay behind a future prototype gate (Rust turn digest/turncheck, Job-Object kills, toml_edit/JUnit/file-lock work — design.md deferral notes).
3. Owner WIP (`rules/evaluate.py` + `tests/test_rule_evaluation.py`) needs its own commit path from the owner.
4. If the untracked round-21 divergence reports should become durable evidence, commit them separately; otherwise they are reproducible via `tests/test_recording.py`.

Clear the conversation context before starting the next task group.
