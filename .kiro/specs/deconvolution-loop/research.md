# Research links: deconvolution-loop

This spec documents a prototyped loop. Its evidence lives in the dated
research log; this file maps spec sections to entries and keeps the
reciprocal link required by the research protocol.

| Spec part | Entry | What it established |
| --- | --- | --- |
| Hypothesis H8 and the loop's shape | [R19](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md) | oracle + edge measures + regeneration, with kill criteria per stage |
| Recorder on Rust (req 2) | [R20](../../../docs/research/2026-11-03/02-e1a-session-bus-oracle.md) | in-process trace oracle, deterministic, catches all mutants; gain prediction hit a seed-test ceiling |
| Scope selection (out of scope) | [R21](../../../docs/research/2026-11-03/03-e2-facdrone-edge-census.md) | edge census rejected as selector; recorder validity proven by diff |
| Recorder and acceptance on Python with an external store (req 2, 3, 8) | [R22](../../../docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md) | byte-identical recordings; replay catches stored-representation change; kill rate 0.712 to 0.746 |
| Differential, first attempt | [R23](../../../docs/research/2026-11-03/05-e1c-concolic-rung.md) | failed on driver encoding; alphabet gap identified |
| Verifier gates (req 9) | [R24](../../../docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md) | four checks, controls, 7 of 10 killed, 0 proved; solver narrowed to finder |
| The whole loop on one scope (req 1 to 12) | [R25](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md) | T1 to T3 held in one round; T4 0.643 failed; T5 one real oracle-blind difference; packing caveat. Amendment 2 (2026-11-06, task 3.2): original 17/26 = 0.654 on the same arm, candidate 0.643; neither meets 0.70; T4 measures the arm, not the regeneration (req 8.2 pair executed) |
| The loop on a second scope with the regenerator program (req 1 to 12; 6.7, 6.8, 8.6, 8.7, 10.1 to 10.3, 11.3) | [R26](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md) | E4, lifecycle monitor group (seven files, 1,495 lines), 2026-11-06. 6.1: plan frozen with T1 to T7 and S, kill criteria, budgets. 6.2: accepted at turn 3 (T1 held; turn 1 thinking-only, turn 2 failed on a surface gap); T2 failed (1,405 canonical lines against at most 1,345); T3 held (52 functions), T4 held (63 edges), S held (11 writes in shells, pure 0.519, patch check pass). 6.3: 128 probes on both trees, 26 disagreements in F1 to F6. 6.4: candidate 19/28 = 0.679 (T5 held), original 12/29 = 0.414 by its sample's composition, 0.70 unmet on both; M2 to M4 caught on both trees, M1 missed as predicted; `lifecycle_root.lifecycle_store` reached by a seed test, `bind_lifecycle` unreached and undeclared. 6.5: void under criterion 4 (window planted-mutant solver control); compare and standing counterexamples replayed on both trees as evidence (F3 by a second route, F7). 6.6: held-out corpus 74 sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one), zero divergence at both levels (T7 held). 6.7: missing-test report (28 entries), residue statement, 771 of 900 research lines. Amendments proposed for this spec: the surface includes the recorder's imports (req 1.5); turn parameters preregistered and a thinking-only turn counted (req 6.8); content hashes over bytes as stored (req 11.2); receipt labels distinct beyond letter case (req 11.3) |
| Verifier rules | [symbolic-execution-tooling.md](../../../docs/symbolic-execution-tooling.md) | placement rule and the four checks |
| Budgets | `docs/experiments/deconvolution/adaptive.py` | probe, scale, wait_for_idle; reference 0.212 s |

Apparatus directories: `docs/experiments/deconvolution/e1b-facdrone-evidence-publication/`,
`e1c2-concolic-rerun/`, `e3-evidence-regeneration/`, `e3b-original-sensitivity/`,
`regenerator/`, `t51-integration/`, `e4-lifecycle/`. Large goldens and
coverage data are on disk only; sha256 sidecars are committed.

Executed since the first version of this file (tasks 1 to 6, 2026-11-05 and
2026-11-06): the canonical and separated statement readings (req 4.2, R25
Amendments 1 and 3), the missing-test report and its usefulness check
(req 8.7), test-infra detection (req 12.4), the original-side sensitivity
(req 8.6, R25 Amendment 2), the regenerator program (req 6.7, 6.8) and E4
(R26). Next entries that would change this spec: adoption of the four
amendments R26 proposes, a third scope under the same protocol, and any
non-Python regeneration.

Applied recap (2026-11-04): [Regenerate behind the surface](../../../docs/reports/deconvolution-loop-applied.md), an argued recap of R19 to R25 with a preregistered prediction for the next three scopes and the loop's place in spec-driven development.
