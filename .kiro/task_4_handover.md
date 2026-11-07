# Task Group 4 Handover — constraint-guided-compression implementation, groups 1–5

Date: 2026-11-07 (night). Scope: session ran the owner-approved cgc task plan
(`.kiro/specs/constraint-guided-compression/tasks.md`) in foreground manual
mode through **task groups 1–5 complete**; group 6 (G1-R spike) is drafted but
not executed. Stopped per the owner's clear-&-resume rule.

## What was done this session (before this spec: native-core close-out, loop task 2.3 + closure, fusion review)

- `7d050cd` native-core marked implementation-complete; `b2ab774` round-21 evidence; `6a4cf96`/`301cb05` loop task 2.3 (R32) + run-report row; `00a3405` loop marked implementation-complete; `7326e09` R33 records committed.
- Independent **ultrathink-fusion design review** (verdict approve-with-fixes, F1–F12), then amendment pass `4469477` (TS descope via requirements Amendment 2 + language-extractor seam; 36/36→19/19; complete G1-R checklist; traceability fixes; pinned Tier-2 formula; evaluator-package boundary; ctrl_feedback_leak gate amendment). Tasks generated `9c3e7c2`; owner amended (a1287de, 94cbb56, 2f6c125) and approved `d52872b`.

## Implementation commits (one per task, TDD, inline review + verify-completion each)

| Task | Commit | Content |
|---|---|---|
| 1 | `715a576` | module skeletons + lazy PEP 562 exports; CLI exit constants |
| 2.1–2.3 | `3840131`, `2e1bf14`, `7abd23d` | Contract-B `derive_surface` + `check_syntax` seam + candidate verification; 21 tests |
| 3.1–3.4 | `4259e57`, `c1e2d93`, `539acd2`, `f7b63e5` | `measure_written` + pluggable `EffectProfile`; separated-form v1 + pinned-ruff canonical (None on unavailable); `measure_all` + parameterized `check_edit_scope` + gaps; 44 tests |
| 4.1–4.2 | `4c8196b`, `959d4e9` | **verbatim** testinfra relocation (+`rule_version`, envelope) — proven: reproduces frozen facdrone output on all 13 keys with the frozen argv; ported suite + envelope tests; 6 tests |
| 5.1–5.5 | `0b45f18`, `7d5a6f8`, `51d80ea`, `1859603`, `81bd58b` | language-parameterized turn contracts + both endpoint styles + ReceiptWriter (.lock, receipt.json.sha256, bindings.json per admission contract); Tier-1 ladder via seams + adaptive schedule; Tier-2 exact-Decimal formula R=0.60+0.25(0.5g+0.5s) + feedback boundary; Refine/BestOfN controller (t/r labels, resume-by-hash, restore, exhaustion retains best, adopted=False); 58 tests incl. six adverse controls with **zero** runtime calls |

**Suite state: 129/129 green** (`test_surface` 21 + `test_measures_scope` 44 + `test_testinfra_unit` 6 + `test_regenerator` 58), fast (≈2 s), fully offline.

## Evidence pointers

- U5 live measurement (temp script, method = design's): canonical lines `recording.py` 372, `surface.py` 281, `measures.py` 558, `regenerator.py` 1,239 → Python total **2,450**; + frozen rustfmt share (admission.rs + receipt.rs + hooks, R27/R31) ≈ 1,333 → **≈3,783 vs ceiling 2,270**.
- Pre-existing machine-local failures (NOT this spec's): 6 Windows process/timeout tests, code last touched `ac68307`; documented in tasks.md Implementation Notes.
- Environment: run everything from `D:\Source\lesscode` (C:\Users\mariu\Source is a junction; the efficiency-baseline integrity layer refuses reparse traversal). Venv `.runs/e2-facdrone`; `PYTHONPATH=src` for direct module runs.

## Current state

- `regenerator.py` never imports `ast` (mechanically checked); all language/domain knowledge behind the surface/measures seams per the owner's amendments.
- **6.1 drafted, NOT run**: `docs/experiments/deconvolution/probabilistic-refine/g1r_spike.py` (uncommitted draft; controls item, E4 three-turn replay via frozen artifacts + s9-style surface4 import, no-progress item, U5 item, admission item; record with negatives retained).
- **Open finding for the owner (blocks G1-R)**: the U5 item will FAIL as measured — the production controller (`regenerator.py` 1,239 canonical) far exceeds the prototype-era budget line (262) the 2,270 ceiling was sized from. Options: (a) record the honest FAIL and keep Slice B locked; (b) slim the Python modules; (c) dated amendment of the ceiling/stratum composition. Do not weaken the check silently.
- 6.2 (live weaker-model run) is owner-go and not started. 7.1 (full verification: `pytest -n 4`, `cargo nextest`, `lesscode-core budgets`, records) pending.
- Owner WIP (`rules/evaluate.py`, `tests/test_rule_evaluation.py`) untouched, uncommitted, never staged.

## Next steps

1. Owner decision on the U5 finding above, then run/finish task 6.1 (driver draft already in the tree).
2. 6.2 only on explicit owner go (spends model budget).
3. 7.1 full-suite verification + spec/research record updates (`8.1–8.6`).
4. Clear context before the next task group (this handover is the resume point).
