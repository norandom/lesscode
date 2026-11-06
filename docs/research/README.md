# Research log

Current direction (2026-11-04): H7's static state models and family-duplication
selection did not verify or compress real scattered code. H8 proposes a recorded,
mutation-scored trace oracle, edge-based convolution measures and regeneration
behind the public API. Gates run so far (R20 to R24): oracle mechanics positive
on Rust and Python, gain predictions failed, edge selector rejected, solver rung
rejected as equivalence authority but useful as a verified counterexample
finder. Regeneration (E3, R25) ran on 2026-11-04: one round met the oracle
with 16% fewer lines and 29% fewer edges, mutation sensitivity missed its bar
(0.643) and one oracle-blind difference on an exception path is real. H8(c)
stands on one scope with a lowered protection claim; the owner decides between
a second round, a second scope, or prevention. The loop as prototyped is
documented algorithm-agnostically in the
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
| H8 | A recorded, mutation-scored trace oracle plus edge measures lets an LLM regenerate scattered responsibilities with fewer edges and equal behaviour | H8(a) oracle mechanics positive on mmhmem (R20) and Facdrone (R22) but the gain predictions failed on both, so H8(a) is rejected as formulated; H8(b) edge selector rejected as formulated (R21); H8(c) regeneration untested and not admitted under the preregistered gates; guided-generation rung failed as run (R23) and, corrected and verified, rejected for DB-entangled predicates while killing 7 of 10 residue targets (R24); H8(c) run as E3 (R25): one regeneration round met the oracle with 16% fewer lines and 29% fewer edges and a checked pure core, but mutation sensitivity missed its bar (0.643) and one oracle-blind difference is real, so H8(c) stands on one scope with a lowered protection claim; owner decision pending: second round, second scope, or prevention |

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
