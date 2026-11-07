# Design Review Gate (`$kiro-spec-design` & `$kiro-validate-design`)

Date: 2026-11-07 ([R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md)).
Mode: Inline review in main foreground context per owner rule (2026-11-07).

## Review Summary
[`design.md`](design.md) replaces the falsified 2026-11-02 H7 static-CodeQL/authored-SMT proposal with the demonstrated H8/H9 deconvolution architecture and adds H10 DSPy-style two-tier probabilistic prompt refinement (`Refine` / `BestOfN`). All 56 numeric acceptance criteria (`1.1`–`8.6`) are mapped, responsibility boundaries with [`deconvolution-native-core`](../deconvolution-native-core/design.md) and [`capability-compression-core`](../capability-compression-core/design.md) are clean, and demonstrated (`Slice A`, `proceed`) vs. provisional (`Slice B`, gated by `G1-R` in [`prototype-gate.md`](prototype-gate.md)) capabilities are explicitly separated.

## Mechanical Gate Verification (`rules/design-review-gate.md`)
1. **Requirements Traceability**: PASS — All 56 numeric IDs (`1.1–1.5`, `2.1–2.6`, `3.1–3.6`, `4.1–4.7`, `5.1–5.8`, `6.1–6.7`, `7.1–7.11`, `8.1–8.6`) appear in the traceability table and component blocks.
2. **Boundary Commitments Populated**: PASS — `This Spec Owns`, `Out of Boundary`, `Allowed Dependencies`, and `Revalidation Triggers` are concrete and enforce stdlib-only top-level imports in `src/lesscode_baseline/`.
3. **File Structure Plan Populated**: PASS — Concrete file paths for `src/lesscode_baseline/{surface,measures,testinfra,regenerator}.py`, `tests/test_{surface,measures_scope,testinfra_unit,regenerator}.py`, and `docs/experiments/deconvolution/probabilistic-refine/g1r_spike.py`.
4. **Boundary ↔ File Structure Alignment**: PASS — Zero overlap or edits to `src/lesscode_baseline/recording.py` or `native/src/*.rs`.
5. **No Orphan Components**: PASS — Every component (`SurfaceDeriver`, `ScopeMeasurer`, `TestInfraDetector`, `ProbabilisticRegenerator`) maps 1:1 to a concrete file path.

## Resolution of Prior Review Issues
1. **H7 vs. H8/H9 Alignment**: Resolved — Static dataflow state models and LLM-authored SMT equivalence models are withdrawn; `SurfaceDeriver` (Contract B), `ScopeMeasurer` (3-stratum canonical + separated-form v1), and `RecordingCore` supply the verification boundary.
2. **Core Deduplication**: Resolved — Reuses `lesscode-core` (`admission`, `receipt`, `budgets`, `compare`) and `lesscode_baseline.recording` without modification.
3. **Resilience to Weaker / Non-Deterministic Models (`H10`)**: Resolved — `ProbabilisticRegenerator` implements DSPy v2.6+ `Refine`/`BestOfN` with a two-tier reward function (`Tier1 StaticAssertionGate` in $[0.0, 0.49]$ catching `max_tokens`, syntax, `editscope`, Contract-B surface, and `pure`/`shell` write violations in $< 2\text{ s}$ with zero DB spin-up; `Tier2 RuntimeOracleGate` in $[0.50, 1.0]$ running `recording.py` and canonical measures).

## Disposition
- **Design Review Gate**: **PASS** (0 blocking issues remaining; 1 Mermaid label repair pass applied).
- **Phase Status**: `design-generated` (`approvals.design.approved = false`, awaiting owner review).
  - **Slice A (`surface`, `measures`, `testinfra`, base `regenerator`)**: `proceed` (backed by `R25`–`R32`).
  - **Slice B (multi-rollout `Refine`/`BestOfN` & optional `dspy` adapter)**: `provisional / revise` behind bounded spike `G1-R` in [`prototype-gate.md`](prototype-gate.md).

## Addendum: Independent Fusion Design Review (2026-11-07, later same day)

Mode: `ultrathink-fusion` (`grounded-critique-revise-vote`), two independent panel agents plus an arbiter, owner-dispatched; both panels converged and the arbiter confirmed. This is the first **independent** adversarial review of `design.md` (the PASS above was inline, and it checked components→files but not traceability-rows→interfaces).

- **Verdict**: `approve-with-fixes`.
- **Findings F1–F12** (severity/confidence in the session record) and their amendment-pass dispositions, all applied 2026-11-07:
  - **F1 (HIGH) TypeScript scope gap** → owner disposition recorded as requirements **Amendment 2** (Python now, TypeScript deferred to a named later stage; G0 TS fixtures preserved with revalidation note; **language extractor seam** binds design and tasks so adding languages is additive — design.md "Deferred Target Languages"); traceability row 7.1 updated.
  - **F2 (MED-HIGH)** "36/36 Python tests" figure → corrected to **19/19** with dated note (no recorded source for 36/36 exists).
  - **F3 (MED-HIGH)** G1-R checklist gap → design.md Testing Strategy now enumerates the **complete** `prototype-gate.md` L18–28 checklist (live weaker-model recovery run, U5 20% apparatus ceiling included).
  - **F4 (MED-HIGH)** dangling traceability interfaces (`diff_measures`, `verify_surface_contract`, `resume_check`, `ExternalSolverVerifier`, `DifferentialReport`) → rows corrected to real mechanisms; `measure_all()` added to the `measures.py` interface; rows 5.1–5.3 now name the located-external-tools reality.
  - **F5 (MED-HIGH)** evaluator-package integration → `metrics.py`/`contracts.py`/`evaluation.py` boundary decision recorded (no duplicate representation — refuted by grep); `__init__.py` wiring + `python -m` CLI plan added.
  - **F6 (MED)** Tier-2 scaling → formula pinned ($R = 0.60 + 0.25\,(0.5g + 0.5s)$, fail-closed keys), added to Revalidation Triggers; **Feedback Content Boundary** stated; `ctrl_feedback_leak` added as sixth control by dated amendment to `prototype-gate.md`.
  - **F7–F12 (LOW/MED-LOW)**: `Req 3.7` citation corrected; Rollout/Retry nesting paragraph added; revalidation triggers extended (Tier-1 rungs, scaling, testinfra rules); row 5.8 prior-evidence vs owned-test clarified; exits `3`/`6` defined from `regen_turn.py`; G1-R failure fallback pointer added.
- **Rejected during fusion** (evidence-based): Rust detector addition (scope expansion), "Slice B relabel" (design labels it honestly), "duplicate representation" (refuted by `metrics.py` grep).
- **Disposition after the amendment pass**: **PASS** — all fusion gates cleared; owner approved the design for **Slice A task generation**; **Slice B tasks remain withheld pending G1-R (`Req 7.10`)**.

---

## Historical Record (2026-11-02 Provisional Design Review)

Date: 2026-11-02. Inline review with read-only codebase discovery from a separate
explorer. Not a final design PASS or independent adversarial review.
Disposition at 2026-11-02: provisional research proposal only (`design-proposal.md` retained as historical record).
