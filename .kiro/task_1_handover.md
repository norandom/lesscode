# Task-group handover 1: solution-first prototype executed (2026-11-07)

Written at the end of the first task group under owner rule 6 (AGENTS.md). Start the
next group from this file plus the mmhmem checkpoint and `.kiro/handover.md`.

## What was done (commits on main, all pushed)
1. Housekeeping: 1d69021 spec amendment + R27; 084b739 handover consolidation;
   049f6a1 owner workflow rules 1-4 + steering; 4835b62 spec status; 103e1aa MIT license.
2. Owner rules made permanent in AGENTS.md/tech.md: foreground implementation, PowerShell,
   nextest/pytest-xdist (docs/testing.md), commit-per-task, Dagger-CI-only WSL (rule 5),
   task-group handover + context clear (rule 6, d844f69).
3. f11179e: prototype-plan.md frozen; owner execution approval recorded in spec.json.
4. f5c6e8f: S2 (R28) — artifact admission (21 bindings; E4 goldens/turns verified, T4
   turns inadmissible) + Rust digest parity PROVEN byte-identical on frozen 5.6 MB traces.
5. U7 overrun 774>600 recorded; owner raised ceiling to 1,400 (4ebdb7c).
6. e4a5fa4 + 2eb04eb: S9 (R29) — contract B delta measured (+1 E3 name, +3 E4 names);
   owner ratified B + four R26 changes + R25 rules.
7. 6263070: S10 shared core — one engine reproduces BOTH frozen recorders byte-identically
   (E3 92a48f77…, E4 0b2e383d… 5,479,067 bytes), zero-divergence replays.
8. c90c0fe: turn checks pass on frozen E4 turns (R30 amd 2; thinking-only ⇒ no feedback).
9. 4710665: candidate arms accepted exactly as preregistered; 9/9 adverse controls
   fail-closed; U5/U7 measurement FAILED as frozen; gate verdict **revise** (R30 amd 3).

## Evidence pointers
- Research: docs/research/2026-11-07/{01,02,03,04}-*.md (R27-R30) + README index.
- Receipts: docs/experiments/deconvolution/native-prototype/results/ (s2-*, s9-*,
  s10-turncheck, s10-controls, s10-u5-measurement + sidecars; big goldens in
  .runs/native-prototype/ with sidecars). Candidate rounds: e3/e4 results/sql-divergences-rounds10.json.
- Code: native/src/{pyjson,admission,turncheck}.rs + bin/turncheck.rs (111/111 nextest,
  run 23b9da3e, fmt/clippy/doc green); shared Python core docs/experiments/deconvolution/
  native-prototype/{proto_core,scope_e3,scope_e4,proto_replay_test}.py.
- State: .kiro/specs/deconvolution-native-core/spec.json (status=revise,
  evidence_review=not-reviewed, evidence list current).

## Current state
- The reuse hypothesis is mechanically demonstrated but the usefulness gate failed:
  C_app 1,808 canonical lines vs ceiling 1,364 (Rust byte-parity core = 1,046 of it);
  replacement is 103 lines larger than the 1,705 baseline; U7 1,808 > 1,400.
- Per-scope adapters ARE smaller (E3 173.5 < 280; E4 216.5 < 358); C_seeds = B_seeds.
- Production remains locked: requirements/design/tasks unapproved; only the demonstrated
  slice may seek design approval after the revise decision + independent evidence review.

## Next steps (owner decision required first)
Choose one and record it in spec.json + handover:
1. Bounded descope (recommended if reuse is the goal): drop the Rust digest module from
   the measured stratum (Python owns digests; Rust keeps sidecar admission ~310 lines)
   → C_app ≈ 1,072 ≤ 1,364, saves 633; re-measure, re-run U1-U7, evidence review.
2. Stop with the negative result: the slice proves byte-parity reuse costs more than the
   duplicated apparatus; keep R28-R30 as the finding; no production native core.
3. Another owner-defined revision (requires a dated plan amendment; do not lower U5
   targets — raise or restructure with justification).
Then: independent evidence review of whatever attempt stands, before any design phase.

## Environment / gotchas
- Arms and recordings run under .runs/e2-facdrone/venv python with E1B/E4_SOURCE_ROOT
  set to the frozen copy; PYTHONDONTWRITEBYTECODE=1; one heavy job at a time.
- Adapter `accept` CLI parses options from argv[2:] (fixed after the first arms
  mislabeled round s10 as round 0; frozen round0 receipts restored untouched).
- .runs/ and untracked experiment trees stay out of git except committed .sha256 sidecars.
