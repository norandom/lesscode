# Query-driven, multi-pass compression research

2026-11-03 direction amendment:
[R19](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md)
records the owner's review that static state models and family-duplication
selection failed on real scattered code. It withdraws static dataflow and
LLM-authored SMT models as the equivalence authority and registers H8: a
recorded, mutation-scored trace oracle, edge-based convolution measures and
regeneration behind the public API. Gate remains `revise`; E1 and E2 precede
any further candidate. This amendment approves no design or tasks.

2026-11-02 cross-module discovery:
[R15](../../../docs/research/2026-11-02/15-codeql-cross-module-cases.md) develops
real Facdrone responsibility cases using CodeQL plus explicit source inspection.
Do not revert to single-file target selection or synthetic examples because a
grown repository's compression opportunity is not obvious. Freeze a complete
behavior boundary and test the candidate mechanism; do not require guaranteed
savings as admission to bounded discovery.

2026-11-02 model-learning amendment:
[R14](../../../docs/research/2026-11-02/14-aalpy-state-learning.md) tests AALpy
directly against real source execution. It supports bounded behavioral model
learning and demonstrates observation sensitivity, not source compression.
Retain original runtime replay and independently frozen observers when using
learned models to guide replacements. See the linked prototype gate for limits.

Execution history and amendments live in the [dated research log](../../../docs/research/README.md).
This file holds the research protocol; dated reports hold outcomes and link back
to the spec. See [R02](../../../docs/research/2026-11-02/02-codeql-direction.md)
for the current setup state. Do not confuse a planned experiment with a run.

Owner input: 2026-11-02. Research direction and prototype protocol, not an
approved implementation design. No new graph extraction or compression run has
been performed for this note.

## Decision to test

### Real-case admission amendment, 2026-11-02

[R11](../../../docs/research/2026-11-02/11-real-case-admission-screen.md)
records a bounded negative discovery result after G0e. Fifteen files were
inventoried, with thirteen overlapping leads examined selectively. No untried
complete responsibility had a justified qualifying replacement; no candidate
or real-code comparison ran. Target selection remains unresolved. Keep this
separate from state-model validation and do not claim zero measured savings,
minimality or compressor effectiveness from it. Full gate remains revise.

### G0e amendment, 2026-11-02

[R10](../../../docs/research/2026-11-02/10-source-backed-state-model.md)
records the explicit state-model trial. It matches 16 runtime observations per
language; frozen controls reject all five wrong models. First three variants
also match retained source mutants. Replay agrees. Authorship was source/oracle
informed, not blinded or independent; no automated synthesis, SMT or compression
ran. This supports attempting a bounded real-case model and replacement, with
predeclared behavior/whole-scope reduction criteria. Full gate stays revise.

### G0d amendment, 2026-11-02

[R09](../../../docs/research/2026-11-02/09-codeql-dependency-diagnostic.md)
localizes the known missing dependence after the field write. Inlining helpers
does not recover it; writing through the original object does, but also reports
flow after overwrite. Focused-query confirmation and replay reproduce this.
Both isolated variants preserve the six executable observations per language.
Retain CodeQL as source-linked discovery evidence, not an exact ordered-state
model. The LLM must expose alias identity, write order and effects as separately
challenged obligations. No general custom heap engine is justified by this
result. Full feasibility remains revise; source-backed model trials are next.

### G0c amendment, 2026-11-02

Both runtime originals pass six cases, and all three adverse variants per
language are detected. Yet neither language's configured global value-flow
query reports the known input-to-alias observation dependency. All sites and
required CFG connections are present. The 9/10 static scores must not obscure
the failed positive control: negative flow checks do not establish precision
when all flows are missing. See [R08](../../../docs/research/2026-11-02/08-codeql-state-boundary.md).
Gate remains revise. Locate the first missing edge before extending analysis;
source-backed model obligations and runtime observations remain separate from
graph evidence. No compression or SMT experiment occurred in this trial.

### G0b amendment, 2026-11-02

The callback omission was caused by requiring a target name while excluding
calls with unnamed targets from the unresolved branch. Independent syntactic
site rows fix that loss. Both languages pass 9/9 G0b checks, with state value
flow through helpers and basic CFG reachability. The throw condition is
concretely false, so the latter is not executable exception-path evidence.
Alias/overwrite and reachable exception-state controls remain next; G1 remains
unrun. See [R07](../../../docs/research/2026-11-02/07-codeql-call-census.md)
for the separate preregistered trial and retained outputs. Full gate: revise.

### Prototype result amendment, 2026-11-02

The approved extraction spike ran with locked Python 7.2.3 and JavaScript 2.9.0
libraries on CodeQL 2.26.3. Both extractors and global value-flow queries work on
the retained fixtures. The oracle passes 13/13 Python checks but 12/13 TypeScript
checks because the relationship query omits the supplied callback after two
repairs. Query replay reproduces the outputs. Disposition: revise the query
contract; no progression to real-code synthesis. State sites and control sites
do not establish state-value/alias or exception-path fidelity. No SMT ran.
See [R06 and raw evidence](../../../docs/research/2026-11-02/06-codeql-extraction.md).
The previous missing-pack discovery notes below describe the earlier state.

### Design discovery amendment, 2026-11-02

Requirements approved; [provisional design](design-proposal.md) prepared under
the prototype-first policy. [R05](../../../docs/research/2026-11-02/05-provisional-design.md)
records actual setup evidence. This is complex integration discovery, using Kiro
discovery/synthesis/review guidance and COBESY's dated evidence structure.

Fresh local resolution finds Python/JavaScript extractors but only the
legacy-upgrades query pack. Compatible analysis libraries must be installed and
locked before queries run. Raw outputs and executable hash are retained in the
[receipt](../../../docs/research/2026-11-02/evidence/codeql-design-setup/receipt.json).

A read-only explorer found analyzer orchestration in `metrics.py` as a lead,
with callers in `evaluation.evaluate` and tests in `tests/test_metrics.py` and
`tests/test_evaluation.py`. Distinct stderr, exception, artifact and diagnostic
semantics prevent assuming interchangeable stages. No defensible untried target
meeting the discovery screen of 25% and 30 net statements emerged. Those are
screening values, not an approved universal target or measured outcome. Report
persistence overlaps the earlier T2-B trial. This does not establish minimality.

Synthesis: use CodeQL query exports, explicit model revisions and native comparison;
defer a graph store, orchestration framework and generic source-to-SMT compiler.
The supported state/effect boundary must fit the real target, not the reverse.
Final design remains pending an end-to-end real-code result.

External contracts checked: database creation requires a fresh destination;
query dependencies need compatible versions and a retained lock. Verify commands
against installed help because online docs describe current releases.
[Database create](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/database-create),
[pack install](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/pack-install).
Python AST nodes can map to several CFG nodes, including exception paths; do not
merge them merely because source spans coincide. Dataflow is potential flow, not
execution proof. [CFG](https://codeql.github.com/docs/codeql-language-guides/analyzing-control-flow-in-python/),
[dataflow](https://codeql.github.com/docs/codeql-language-guides/analyzing-data-flow-in-python/).

Owner decision, 2026-11-02: CodeQL is selected for the initial prototype, rather
than merely shortlisted. The first gate remains extraction/modeling fidelity,
not a compression claim. Do not run a multi-backend comparison unless the
CodeQL trial exposes a concrete limitation that warrants it.

Setup verification resolved the local PowerShell wrapper to
`C:/Users/mariu/AppData/Local/Programs/CodeQL/2.26.3/codeql/codeql.exe`.
Running `version --format=json` confirmed CodeQL 2.26.3, build
`7d097a43199effe04ecd9c6bd3ad9bb02a45b3d7`. Query-pack versions and extraction
remain unverified. Initial inspection found no project-level license. The owner
subsequently authorized MIT licensing on 2026-11-02; the repository now contains
the [MIT License](../../../LICENSE) and corresponding package metadata.
This resolves that licensing prerequisite for using Lesscode as an open-source
specimen; third-party fixture notices still apply. Record the selected license,
CLI and query-pack versions, source snapshot and query results before analysis.

Start with a persisted, queryable program representation and let an LLM model
connected responsibilities through repeated queries. Test CodeQL first on an
eligible open-source specimen; retain Joern as a comparison candidate. Keep the
native Lesscode comparison core as the measurement boundary. Do not build a new
graph database, symbolic compiler or agent framework before testing whether the
existing tools expose the facts needed for useful compression.

This changes the mechanism, not just the candidate ranking. The unit of modeling
is a relation between inputs, shared state, effects and outputs across functions.
Translating isolated if/then/else expressions is one supported special case.

## Research questions

- Can a query surface recover a responsibility fragmented across functions and
  modules, including shared-state reads/writes and relevant callers?
- Can a modeler distinguish semantic similarity from actual interchangeable
  behavior, and correct its model using source and executable counterexamples?
- Do additional passes improve verified net compression over an equal-budget
  source-only workflow, or merely produce more proposals?
- Where do extraction gaps, model mismatch and state explosion make the approach
  impractical? A documented negative answer is an acceptable research outcome.

## Tool shortlist and evidence

| Tool | Proposed role | Decision and limitation |
| --- | --- | --- |
| Joern | Persisted code property graph, calls, control/data dependence and queryable slices | Comparison candidate; measure required customization rather than assuming easy integration |
| CodeQL | Persisted extracted database and existing control/dataflow query libraries | First research spike on eligible open-source code; not an unrestricted redistributable runtime |
| DSPy | Typed modeler stages, evaluation and later prompt/program optimization | Trial fixed signatures first; optimization follows a trustworthy evaluator and separate training/held-out scopes |
| LangGraph | Existing stateful orchestration and checkpointed iteration | Candidate if resumable multi-role execution is needed; do not combine multiple orchestration frameworks |
| KLEE | Symbolic-execution research and optional native-component experiment | LLVM-bitcode tool, not a direct verifier of arbitrary Python or TypeScript |
| Z3 | Check bounded semantic models and synthesize parameters/choices | Solver verdict applies to the encoded model; no automatic guarantee of source equivalence |

Joern has a persisted CPG and query language, and documents Python and JavaScript
frontends; the JavaScript frontend exposes TypeScript type-generation options.
Its core repository is Apache-2.0 licensed. Pin the actual release and inspect
bundled components before use. Documentation is not a coverage certificate for
our specimens. [Overview](https://docs.joern.io/),
[frontends](https://docs.joern.io/frontends/),
[TypeScript-related options](https://docs.joern.io/frontends/javascript/),
[license](https://github.com/joernio/joern/blob/master/LICENSE).

Joern exports whole graphs to GraphML, GraphSON and Neo4j CSV, among other
formats. Prefer its native query surface and export small evidence slices first.
A separate database is justified only by a demonstrated missing query or storage
capability. Export documentation illustrates graph representations for C/C++;
test which edges our selected frontends actually produce.
[Export documentation](https://docs.joern.io/export/).

CodeQL provides a database and local/global dataflow queries. Global analysis
trades precision and cost for scope, and dynamic targets, aliases and library
behavior need modeling. Its CLI is free for public repositories under its terms;
private-repository use is licensed. Open query libraries do not make the entire
CLI an unrestricted open-source foundation.
[CLI availability](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-cli),
[Python dataflow](https://codeql.github.com/docs/codeql-language-guides/analyzing-data-flow-in-python/),
[dataflow limitations](https://codeql.github.com/docs/writing-codeql-queries/about-data-flow-analysis/).

DSPy optimizes a program against a supplied metric; it does not supply semantic
correctness. LangGraph provides a stateful orchestration runtime, including
durable execution. These solve different problems: optimization across trials
versus execution of a particular multi-pass trial. Neither should become a
mandatory dependency before its role is exercised.
[DSPy optimizers](https://github.com/stanfordnlp/dspy/blob/main/docs/docs/learn/optimization/optimizers.md),
[LangGraph overview](https://docs.langchain.com/oss/python/langgraph/overview).

## A queryable evidence model

### Candidate-selection correction, 2026-11-02

The owner correctly distinguishes a usable research tool from an unrestricted
product dependency. CodeQL's terms permit analysis of OSI-licensed codebases,
and automated database generation for such codebases hosted and maintained on
GitHub.com. The terms also restrict redistribution and hosted provision of the
CLI. Verify each specimen's license and intended workflow; a public repository
alone is not sufficient. Do not assume applied research automatically qualifies
as academic research. These conditions do not exclude an eligible local prototype.
[Actual CLI terms](https://github.com/github/codeql-cli-binaries/blob/main/LICENSE.md).

CodeQL databases are extracted relational program facts queried with QL libraries,
not a ready-made Neo4j database containing every desired semantic graph. Derived
flow relations still need queries and potentially framework models. Start with
existing Python/TypeScript libraries and a small query pack. Export selected
results through BQRS to JSON/CSV, preserving provenance. Store LLM hypotheses and
revision history separately; SQLite is a possible simple store, not a required
new platform. No full graph export is necessary to test the modeling loop.
[Result export](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/bqrs-decode).

Alternatives worth distinguishing:

- Fraunhofer AISEC CPG has an existing `cpg-neo4j` persistence path. It is a
  concrete database-oriented alternative, but Python/TypeScript coverage and
  required passes must be tested before treating it as lower-effort than Joern.
  [Project and persistence instructions](https://github.com/Fraunhofer-AISEC/cpg).
- SCIP provides serialized definitions, references and implementations, with
  Python/TypeScript indexers and Rust bindings. It is useful for navigation and
  ownership discovery; it is not a replacement for CFG/dataflow/state analysis.
  [Protocol and indexers](https://github.com/scip-code/scip).
- Souffle supplies a Datalog analysis engine, not the ready-made language
  extraction and semantic models we need. Building those would shift effort
  toward infrastructure, so it is not the first prototype choice.
  [Project](https://souffle-lang.github.io/).

Proposed selection test: run the same small queries for caller boundaries,
argument-to-return flow, shared-state reads/writes, exception/control dependence
and unresolved calls. Compare actual coverage, custom query/model code and replay
effort. CodeQL-first is a hypothesis about less integration work, not a benchmark
result or commitment to its product licensing model.

Use an attributed program graph, not only a function-call graph or UML diagram.
UML-style views are useful presentations of selected facts, not the analysis
store. Keep extracted facts, analysis approximations, observed executions and
LLM hypotheses distinct.

Proposed node kinds: module, callable, parameter, return, state location, external
capability and observable effect. Proposed edges: calls, argument/return flow,
reads, writes, control dependence, data dependence and possible aliasing. These
are desired query concepts, not a claim that one frontend supplies them all.
Keep unresolved targets and unavailable edge types explicit.

Store source snapshot, tool/frontend version, query text/version, source spans,
edge provenance, assumptions and exclusions with each result. Use snapshot-local
IDs plus source anchors; do not assume analyzer IDs survive source changes.
Serialize the native graph and machine-readable query results. Similarity links
belong in a separate hypothesis overlay, never disguised as extracted call edges.

Initial queries should answer:

1. Which entry points and owned implementations contribute to this output or
   effect? Return a backward slice and a caller boundary.
2. Which functions read and write the same state, and what ordering or alias
   assumptions connect them?
3. Which paths repeat conversions, policy decisions or representations?
4. Which structurally or semantically similar clusters have different effects,
   exception behavior or dependencies that prohibit unification?
5. Which implementation could disappear if an existing repository/library
   capability replaced the operation, including affected callers?

Graph reachability means potential connection under the analysis assumptions,
not feasible execution. Absence of an edge in an incomplete dynamic-language
graph does not establish dead code. Semantic embeddings and LLM similarity may
propose clusters but cannot establish equivalence.

## How the loop produces direction

Maintain a frontier of unresolved modeling obligations per cluster. Choose the
next query or pass because it resolves a specific uncertainty that prevents a
replacement, not simply because another LLM turn is available.

For example: a state-sharing query reveals three writers; the modeler proposes
one transition function; the critic finds an omitted error-state update; an
execution witness corrects the transition relation; the solver challenges the
revised relation; the replacement is then checked against original execution.
This example is illustrative, not a completed experiment.

Represent stateful behavior as a relation such as
`T(input, state_before, state_after, output, effect_trace, exception)`.
Use transfer functions or summaries on graph edges where justified. Different
internal states may need an explicit abstraction relation; equal final return
values alone do not establish equivalent observable traces.

Collapse strongly connected call components for scheduling, summarize stable
boundaries and refine recursive/stateful components together. A leaf-first order
can help, but shared state and callback cycles mean the repository is not a tree.
Do not enumerate all paths of a large repository before choosing relevant slices.

Proposed roles within an existing orchestrator:

- Explorer/modeler: queries evidence and proposes scoped relations and alternatives.
- Critic: checks source-model omissions and produces adverse cases in a fresh context.
- Candidate author: produces a concrete implementation under the recorded model.
- Deterministic evaluator: runs frozen checks and accounting; agents cannot edit it.

Repeat identification, model construction, critique and simplification as needed.
Record changed hypotheses, counterexamples and remaining obligations at each pass.
Stop on success, unsupported semantics, exhausted budget or repeated lack of new
evidence. Multiple agents are not independent proof authorities.

## Controlling state explosion

Research selective slicing, compositional summaries, abstraction/refinement,
constraint caching, bounded exploration and selective state merging. Each removes
work or changes precision differently; declare the corresponding proof boundary.
Loop bounds and sampled executions remain bounded evidence. Widening or an
over-approximation can introduce spurious counterexamples, which require refinement.
Partial-order reduction is relevant only where independent effects are established.

KLEE's published work provides concrete search and constraint-management methods.
Its normal input is LLVM bitcode. Translating a Python/TypeScript model to C and
running KLEE would test the translation, not establish original source semantics.
Use it only if a justified native boundary or separately validated translation
exists. [KLEE tutorial](https://klee-se.org/releases/docs/v2.3/tutorials/testing-function/),
[KLEE OSDI 2008](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/cadar/cadar.pdf).

State merging is not a universal remedy: it reduces separate states but can make
solver expressions harder. Compare selective merging with separate exploration
on our small fixtures before adopting it.
[Kuznetsov et al., Efficient State Merging, PLDI 2012](https://www.cs.rhul.ac.uk/home/uaac003/papers/pldi12.pdf).

## Staged prototype protocol

Suggested sizes below are planning bands, not measured specimen sizes or success
thresholds. Register exact scopes, budgets and useful reduction targets before
running each stage. Stop at the first failed prerequisite.

| Stage | Question and scope | Evidence needed to proceed |
| --- | --- | --- |
| G0: extraction | Small hand-audited Python and TypeScript fixtures with cross-function flow, callbacks, aliases, shared state and negative controls | Expected edges and omissions checked; serialization/reload and query replay agree; unresolved facts visible |
| G1: small | About 200-1,000 production lines spanning multiple functions, including state and a deliberately non-compressible control | Complete loop, meaningful net deletion where applicable, semantic adverse cases rejected, model gaps exposed |
| G2: medium | About 2,000-10,000 lines across modules and real framework boundaries | Better useful-replacement yield than comparison arms; all affected callers and support code counted |
| G3: large | 100k+ repository census with bounded slices, not whole-program symbolic execution | Query coverage and unsupported population reported; repeated useful results across preselected scopes, including held-out cases |

Comparison arms: A, source-only LLM with the same framework inventory and tests;
B, graph-grounded multi-pass LLM without solver feedback; C, B plus supported
constraint checking and counterexample refinement. Hold model identity, baseline,
scope and total inference/query budget comparable. Record one-pass and later-pass
outcomes within the budget; use repeated runs and repository-separated held-out
scopes before claiming general efficacy. DSPy training must not consume the
held-out acceptance cases or tune away hard behavioral checks.

Measure net production LLOC (with its definition), physical lines, AST statements,
decisions, call layers, added integration and changed callers separately from
tests and apparatus. Also record query coverage on labeled fixtures, unresolved
calls/state, candidate yield, rejected candidates, model defects, counterexamples,
passes and unknown/timeout rates. Runtime and token budgets describe research
feasibility, not application performance benefits. Do not use bytes as savings.

Disproof/stop conditions:

- The graph misses a critical state/call relation without surfacing uncertainty.
- A smaller candidate is accepted despite a meaningful behavioral mutation.
- Repeated passes cannot correct a demonstrated model/source mismatch.
- Added summaries, adapters or tables consume the supposed deletion.
- The bounded campaign produces only marginal changes or no improvement over the
  comparison arm. Report this as a negative result; do not change targets afterward.

General semantic equivalence and globally minimal programs are not promised.
Unsupported domains and an unsuccessful hypothesis are valid outcomes. Whether
this particular combined approach works remains unestablished; avoid claiming
that no related solution exists anywhere.

## COBESY experiment-record blueprint

Audience: the owner and future experiment maintainers. Target behavior: reproduce
a trial, inspect the model's omissions, and decide whether the next stage is
justified. Working assumption to challenge: more passes or more graph structure
necessarily produce better compression. Frame failures as evidence about the
method, not failure by the programmer or specimen. The owner explicitly welcomes
early disproof; no persuasion campaign is needed.

| Report section | Claim to establish | Evidence and writing guidance |
| --- | --- | --- |
| Decision | Continue, revise or stop this hypothesis | Lead with outcome and exact scope; no success claim from graph extraction alone |
| Before versus now | What changed from previous trials | Separate local edit measurement from graph/model/solver contributions |
| Model and omissions | What the system believed and missed | Show a small queryable slice, source anchors and state/effect contracts |
| Revision trace | Why another pass helped or failed | Show counterexample, changed constraint and resulting candidate |
| Whole-replacement result | Whether less maintained code preserves behavior | Before/after chart, all overhead, tests and unsupported semantics |
| Transfer decision | What can generalize and what cannot | Held-out evidence, failure modes and next stop gate |

Visualizations planned for actual runs: a labeled dependency/state slice;
before/after production counts for every arm; and a per-pass plot of unresolved
obligations versus verified net deletion. Keep test/apparatus counts separate.
Do not draw quantitative result charts until measurements exist.

COBESY validation: conclusion-first structure, explicit disproof conditions,
separate evidence from hypotheses, concrete replay handoff, no inflated maturity
claims. This note is a research blueprint; results remain pending.

