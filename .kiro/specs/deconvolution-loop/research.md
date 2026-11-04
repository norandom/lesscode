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
| The whole loop on one scope (req 1 to 12) | [R25](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md) | T1 to T3 held in one round; T4 0.643 failed; T5 one real oracle-blind difference; packing caveat |
| Verifier rules | [symbolic-execution-tooling.md](../../../docs/symbolic-execution-tooling.md) | placement rule and the four checks |
| Budgets | `docs/experiments/deconvolution/adaptive.py` | probe, scale, wait_for_idle; reference 0.212 s |

Apparatus directories: `docs/experiments/deconvolution/e1b-facdrone-evidence-publication/`,
`e1c2-concolic-rerun/`, `e3-evidence-regeneration/`. Large goldens and
coverage data are on disk only; sha256 sidecars are committed.

Next entries that would change this spec: a canonical-format measure
(req 4.2), the missing-test report (req 8.7), test-infra detection
(req 12.4), E3b or E4 (multi-round or second scope), and any non-Python
regeneration.

Applied recap (2026-11-04): [Regenerate behind the surface](../../../docs/reports/deconvolution-loop-applied.md), an argued recap of R19 to R25 with a preregistered prediction for the next three scopes and the loop's place in spec-driven development.
