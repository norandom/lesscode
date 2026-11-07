# Research log

## Latest: S10 complete — reuse proven, usefulness targets failed (2026-11-07)
[R30](2026-11-07/04-s10-shared-core.md): the shared S10 engine reproduced BOTH frozen
recorders byte-identically (E3 `92a48f77…` = committed golden sidecar; E4
`0b2e383d…` = golden-a LF-body, 5,479,067 bytes), replayed all goldens with zero
divergence, accepted both frozen candidates exactly as preregistered, and passed 9/9
adverse controls fail-closed. U5/U7 FAILED as frozen: C_app 1,808 vs ceiling 1,364
(the Rust byte-parity core costs more than the duplicated Python it replaces);
gate verdict **revise**, owner decision pending (bounded descope would pass at
~1,072; or stop with the negative result). S9 evidence in
[R29](2026-11-07/03-s9-surface-contracts.md) (contract B ratified); S2 in
[R28](2026-11-07/02-s2-artifact-admission-digest-parity.md) (U7 ceiling raised to
1,400 by owner mid-run).

## Proposal: solution-first native slice (2026-11-07)
[R27](2026-11-07/01-solution-first-native-slice.md): owner-requested spec amendment,
U1-U7 before production expansion, updated 2026-11-07 with empirical canonical baseline
audit (`2,222` written -> `3,963` canonical lines across 14 files; 3-stratum `U5`
accounting; `U7` brownfield/verbatim-extraction rules) and Rust library audit (`Cargo.lock`
pins kept; `serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`,
`toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added).
Prototype plan frozen and owner-approved 2026-11-07; execution started (see R28).
Historical H8 evidence below remains unchanged.

Current direction (2026-11-06): H7's static state models and family-duplication
selection did not verify or compress real scattered code. H8 proposes a recorded,
mutation-scored trace oracle, edge-based convolution measures and regeneration
behind the public API. Gates run so far (R20 to R24): oracle mechanics positive
on Rust and Python, gain predictions failed, edge selector rejected, solver rung
rejected as equivalence authority but useful as a verified counterexample
finder. Regeneration has run on two scopes. E3 (R25, 2026-11-04): one round met
the oracle with 16% fewer lines and 29% fewer edges, mutation sensitivity missed
its bar (0.643; the original scores 0.654 on the same arm) and one oracle-blind
difference on an exception path is real. E4 (R26, 2026-11-06), the first of the
three scopes the report's section 8 preregistered, run with the recorded
regenerator program on the lifecycle monitor group: accepted at turn 3, 25%
fewer functions and 29% fewer edges inside the predicted bands, the line
prediction failed (6% fewer canonical lines against 10 to 20), sensitivity 0.679
inside its band with the original at 0.414 on its own sample, seven oracle-blind
differences on error paths, zero divergence on 74 held-out sequences (26 of
them the acceptance seed sequences replayed again, 47 new; 1 generated sequence
duplicates an acceptance one), the differential void on a failed solver
control. H8(c) stands on two scopes with a lowered protection claim and one
failed prediction; the owner decides between a
third scope, adoption of the E4 candidate outside the loop, or prevention. Four
protocol amendments are proposed from E4 (the surface includes the recorder's
imports; turn parameters preregistered; hashes over bytes as stored; labels
distinct beyond letter case). The loop as prototyped is documented
algorithm-agnostically in the
[deconvolution-loop spec](../../.kiro/specs/deconvolution-loop/requirements.md)
with per-requirement feasibility status. The earlier CodeQL direction is history.

## Dated lab reports

Report for third parties (2026-11-04, no new evidence):
[Regenerate behind the surface](../reports/deconvolution-loop-applied.md),
self-contained: setting, the loop in eight steps with definitions, results
R20 to R25, lessons, limitations, a falsifiable prediction for the next
three scopes, the loop's place in spec-driven development, glossary and
references.

Latest proposal: [R19: dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md).
Owner review of 2026-11-03 recorded; static dataflow and LLM-authored SMT models
withdrawn as equivalence authority; H8 registered with kill criteria per stage.

Latest replacement: [R18: provider representation](2026-11-02/18-provider-representation-trial.md).
Rejected: 30 fewer named-argument bindings but 12 more lines and 10 more statements;
70 selected tests pass on each version and 32 provider observations match.

Latest screen: [R17: provider orchestration](2026-11-02/17-provider-orchestration-screen.md).
Common algorithms are already shared; the narrow orchestration rewrite fails
admission. Repeated constructor fields remain an unresolved representation lead.

Latest trial: [R16: evidence-publication binding](2026-11-02/16-evidence-publication-trial.md).
Rejected: 1,256 → 1,259 AST statements across the complete accounting scope.
128 differential observations matched; lifecycle integration remains untested.

Latest discovery: [R15: CodeQL cross-module cases in Facdrone](2026-11-02/15-codeql-cross-module-cases.md).
781 extracted files; three connected responsibility hypotheses, with state,
caller and behavior boundaries. Discovery evidence, not measured savings.

Latest modeling result: [R14: AALpy learns real snapshot behavior](2026-11-02/14-aalpy-state-learning.md).
Four states under copy-only observations, six with source reads; 2,144 bounded
sequences matched and six wrong merges were rejected. No compression claimed.

Latest deployment correction: [R13: Windows client, Cairn HTTP server](2026-11-02/13-windows-client-cairn-api.md).
Real cross-environment HTTP coordination passed; Windows remains the agent runtime.

Latest: [R12: direct Cairn adoption prototype](2026-11-02/12-cairn-direct-prototype.md).
Upstream API/storage and local execution tests pass; LLM search and compression
remain untested. R11 is a search limitation, not proof of uncompressible code.

Folders group the discussion or research day in Europe/Berlin. Each entry records
its actual writing date separately when backfilled. Execution dates, when known,
remain separate; file modification times are not treated as execution evidence.

| Discussion / record day | Record | Status |
| --- | --- | --- |
| 2026-11-07 | [27: Solution-first native slice](2026-11-07/01-solution-first-native-slice.md) | planned (`solution_unlock=not-run`); spec amendment for U1-U7 before production expansion; Amendment 2 (2026-11-07): read-only canonical baseline audit (`2,222` written -> `3,963` canonical lines across 14 files; `e1b_oracle.py` 406->659, `e4_oracle.py` 534->1,258), 3-stratum `U5` accounting ($B_{\text{app}}, B_{\text{ops}}, B_{\text{seeds}}$), `U7` brownfield/verbatim-extraction rules, and Rust crate audit (`serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`, `toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added) |
| 2026-11-06 | [26: E4 lifecycle regeneration](2026-11-06/01-e4-lifecycle-regeneration.md) | second scope (seven lifecycle monitor files, 1,495 lines) with the recorded regenerator program; T1 held, accepted at turn 3 (turn 1 spent 128k tokens thinking, turn 2 failed on a surface gap: a name only the recorder imports); T2 failed (1,405 canonical lines against at most 1,345, 6.0% fewer); T3 held (52 functions for 69) and T4 held (63 edges for 89); T5 held (candidate 19/28 = 0.679 in the band 0.60 to 0.75; original 12/29 = 0.414 by its sample's composition, stated in advance; 0.70 unmet on both; M2 to M4 caught on both trees, M1 missed as predicted); T6 held on F1 and F3 from 128 review probes (26 disagreements, F1 to F6), 6.5 void under criterion 4 (the window solver control failed), compare and standing counterexamples kept as evidence (F7); T7 held, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one); S held (11 writes in shells, pure 0.519, patch check pass); residue corrected (`lifecycle_root.lifecycle_store` is reached, `bind_lifecycle` unreached and undeclared); missing-test report 28 entries; 771/900 lines; about $6.96 of model time (estimate); owner decision third scope, adopt or stop |
| 2026-11-04 | [25: E3 evidence regeneration](2026-11-04/02-e3-evidence-regeneration.md) | T1-T3 held in 1 round (1,274→1,074 lines, 660→546 statements, 84→60 edges, 11 writes all in shell, 36/51 pure); T4 failed on kill rate 18/28 = 0.643 (M1-M3 caught); T5 found one realizable oracle-blind difference (`capture_payload` absences after an exception) that reading and 300 random inputs had already exposed; reviewer caveat: 26 statements are packing; no kill criterion fired; 545/400 lines; owner decision E3b, E4 or stop. Amendment 2 (2026-11-06, task 3.2): the same arm on the original scores 17/26 = 0.654 (post hoc 0.773), one kill apart from the candidate, so T4 measures the arm's sensitivity on this module, not a loss from regeneration; five real corpus gaps shared by both trees |
| 2026-11-04 | [24: E1c-2 corrected solver rung](2026-11-04/01-e1c2-corrected-solver-rung.md) | S1-S4 failed, S5 held; kill criterion fired on a verified pipeline (4 verifier checks passed); 7 of 10 targets killed (4 by the reader-inclusive alphabet alone, 3 only by realised solver inputs), 3 unresolved, 0 proved equivalent; guided generation rejected as equivalence authority for DB-entangled predicates; owner decision E3 or stop |
| 2026-11-03 | [23: E1c concolic rung](2026-11-03/05-e1c-concolic-rung.md) | R1-R4 failed, R5 held; 1 of 10 targets solved, 1 of 8 killed; cause: driver encoding concretised strings and the oracle never calls the reader functions; corrected rerun pending owner decision |
| 2026-11-03 | [22: E1b Facdrone evidence-publication oracle](2026-11-03/04-e1b-evidence-publication-oracle.md) | Oracle byte-deterministic, catches M1-M3 incl. stored-representation change tests miss; kill rate 0.712 to 0.746 on 60 mutants, Q1 gap-halving failed; H8(a) rejected as formulated; random generation found 2 new behaviours; owner decision pending |
| 2026-11-03 | [21: E2 Facdrone edge census](2026-11-03/03-e2-facdrone-edge-census.md) | 2,275 responsibilities, 94.7% reached; C1/C3/C4 held, C2 and C5 failed; edge measures rejected as the E3 selector; recorder validity proven by diff; two Windows defects in specimen test support recorded |
| 2026-11-03 | [20: E1a session-bus trace oracle](2026-11-03/02-e1a-session-bus-oracle.md) | Oracle built, deterministic, catches all mutants; P2 gain prediction hit a 96% seed-test ceiling; Arm B partial; generated inputs found a real panic; owner decision pending |
| 2026-11-03 | [19: Dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md) | Proposal after owner review; H8 registered; E1 oracle feasibility and E2 edge census preregistered; not run |
| 2026-11-02 | [18: Provider representation trial](2026-11-02/18-provider-representation-trial.md) | Concrete replacement rejected for expansion; scoped behavior checks pass; no adopted savings |
| 2026-11-02 | [17: Provider-orchestration admission screen](2026-11-02/17-provider-orchestration-screen.md) | No rewrite admitted; existing shared helpers; constructor alternative deferred, metric limitation explicit |
| 2026-11-02 | [16: Evidence-publication binding trial](2026-11-02/16-evidence-publication-trial.md) | Candidate rejected: more statements, no adopted savings; full lifecycle not validated |
| 2026-11-01 | [Yesterday's seven experiment and decision entries](2026-11-01/README.md) | Retrospectively written 2026-11-02; original evidence linked |
| 2026-11-02 | [01: Prior experiments and hypothesis revisions](2026-11-02/01-retrospective.md) | Retrospective catalog of retained evidence; no new execution |
| 2026-11-02 | [02: CodeQL selection and setup](2026-11-02/02-codeql-direction.md) | Backend selected; CLI verified; extraction pending |
| 2026-11-02 | [03: Requirements and continuity anchor](2026-11-02/03-requirements.md) | Requirements generated for review; no new trial |
| 2026-11-02 | [04: Prototype-first Kiro gate](2026-11-02/04-prototype-first.md) | Real-code feasibility required before production tasks; gate not run |
| 2026-11-02 | [05: Provisional design and setup evidence](2026-11-02/05-provisional-design.md) | Requirements approved; query packs and real-code feasibility pending |
| 2026-11-02 | [06: CodeQL fixture extraction and replay](2026-11-02/06-codeql-extraction.md) | Revise: Python 13/13, TypeScript 12/13; callback omitted; no compression claim |
| 2026-11-02 | [07: Complete call census and state-flow checks](2026-11-02/07-codeql-call-census.md) | G0b: 9/9 checks each; unnamed-target omission fixed; full feasibility remains revise |
| 2026-11-02 | [08: Alias, overwrite and reachable exception boundary](2026-11-02/08-codeql-state-boundary.md) | G0c: runtime controls pass; both queries miss known alias flow; revise completeness assumption |
| 2026-11-02 | [09: Dependency checkpoints and controlled contrasts](2026-11-02/09-codeql-dependency-diagnostic.md) | Direct receiver restores flow but retains overwritten input; graph evidence is not an exact state model |
| 2026-11-02 | [10: Source-backed state model and adverse controls](2026-11-02/10-source-backed-state-model.md) | 16/16 observations per language; five wrong models rejected; real-code compression remains untested |
| 2026-11-02 | [11: Real-case admission screen](2026-11-02/11-real-case-admission-screen.md) | Fifteen files inventoried, thirteen overlapping leads; no qualifying replacement admitted or executed |

Older source, patches, metrics, charts and checks remain under `docs/experiments/`
and linked reports. Their paths and historical conclusions are preserved. New
narrative lab reports belong in `docs/research/YYYY-MM-DD/NN-topic.md`; place new
durable evidence beside the report or link an existing experiment directory.
Ignored `.runs/` output alone is not durable evidence.

## Hypothesis lineage

These IDs are retrospective navigation labels, not original preregistration IDs.
They do not replace trial identifiers in the original records.

| ID | Proposition | Current evidence / revision |
| --- | --- | --- |
| H1 | A shared synthesized kernel removes divergent flow | Works on verbose restricted fixtures; expands the compact control; no general source-equivalence claim |
| H2 | Local metrics and family factoring identify useful removal | Mixed and often marginal; evaluate the whole replacement |
| H3 | SMT feedback and complexity caps drive further compression | A bad proposal was caught; revised outcome classifier had no additional line/LLOC reduction |
| H4 | Shared framework knowledge and connected graphs improve proposals | Pilot benefit unestablished; graph rewrite increased LLOC; retain negative results |
| H5 | Complete capability scope exposes substantial redundancy | ATS/NPU positive manually selected cases; not automatic discovery efficacy |
| H6 | Native measurement plus agent discovery yields useful general compression | Native workflow exercised; local baselines and marginal trials do not establish this broader claim |
| H7 | Query-grounded, multi-pass constraint modeling drives substantial compression | Never ran end to end in 18 entries; static state models rejected on G0c/G0d; superseded by H8 as the equivalence approach |
| H8 | A recorded, mutation-scored trace oracle plus edge measures lets an LLM regenerate scattered responsibilities with fewer edges and equal behaviour | H8(a) oracle mechanics positive on mmhmem (R20) and Facdrone (R22) but the gain predictions failed on both, so H8(a) is rejected as formulated; H8(b) edge selector rejected as formulated (R21); H8(c) regeneration untested and not admitted under the preregistered gates; guided-generation rung failed as run (R23) and, corrected and verified, rejected for DB-entangled predicates while killing 7 of 10 residue targets (R24); H8(c) run as E3 (R25): one regeneration round met the oracle with 16% fewer lines and 29% fewer edges and a checked pure core, but mutation sensitivity missed its bar (0.643) and one oracle-blind difference is real, so H8(c) stands on one scope with a lowered protection claim; H8(c) run on a second scope as E4 (R26) with the recorded regenerator program: accepted at turn 3, 25% fewer functions and 29% fewer edges inside the predicted bands, canonical lines only 6% fewer (T2 failed), sensitivity 0.679 inside its band with the original at 0.414 on its own sample, seven oracle-blind differences on error paths, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one), the differential void on a failed solver control; H8(c) stands on two scopes with the line prediction failed on the second; owner decision pending: third scope, adoption of the E4 candidate outside the loop, or prevention |
| H9 | Shared recording, replay, acceptance and turn-input apparatus plus thin scope adapters reduces maintained apparatus across E3 and E4 while preserving observations | Registered in [R27](2026-11-07/01-solution-first-native-slice.md); not-run (`solution_unlock=not-run`); baseline canonical anatomy measured (`2,222` written -> `3,963` canonical lines across 14 files; 3-stratum `U5` target and `U7` brownfield/verbatim-extraction rules defined) |

H7 extends scope-wide reasoning and reintroduces solver research for a concrete
purpose. It does not erase H1-H6 or restore previously dropped infrastructure.
Use the [retrospective](2026-11-02/01-retrospective.md) for evidence links and the
[current Kiro brief](../../.kiro/specs/constraint-guided-compression/brief.md)
for the proposed feature boundary.

## Record each expansion before calling it progress

1. Name the hypothesis, its predecessor and a disproof condition.
2. Freeze scope, source identity, evaluator, dependencies and usefulness target.
3. State which missing capability the proposed tool change addresses.
4. Run a bounded prototype; retain failures, counterexamples and intermediate models.
5. Report whole-replacement results and evidence limits. Distinguish candidate,
   verified candidate and adopted code; do not sum overlapping trials.
6. Link the result to the spec decision it informs. Approval of a backend or a
   license does not automatically approve requirements, design and tasks.

Use the [entry template](ENTRY-TEMPLATE.md). Each applicable spec links back here
and each lab entry links to its spec. Update this index in the same change as the
entry. New interpretations get a dated amendment; do not silently overwrite old
results or relabel an exploratory check as preregistered.

## COBESY handoff

Audience and anchor: the owner and agents continuing the research. Target action:
inspect the latest decision and its evidence before extending the tool. The
assumption to challenge is that more infrastructure, passes or lower complexity
must mean less maintained code. The owner welcomes early disproof; rejected
hypotheses are useful outcomes, not contributor failures.

Composition: outcome first, then hypothesis, method, evidence, limitations and
next decision. Keep findings and proposals separate. The date index supplies
chronology; individual reports lead with the decision. A future agent should be
able to reproduce the result without inheriting the conversation.

Validation: no pooled success rates across unlike scopes, no unsupported proof
claims, no bytes-as-benefit or inferred maintenance-hours savings. Quantitative
charts require actual data. Preserve test and apparatus accounting separately.
