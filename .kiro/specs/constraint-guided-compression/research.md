# Query-driven, multi-pass compression research

## Design Discovery & Synthesis (2026-11-07, `$kiro-spec-design` phase — [R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md))

### Summary
- **Feature**: `constraint-guided-compression`
- **Discovery Scope**: Complex Integration / Extension (reconciling `constraint-guided-compression` with completed [`capability-compression-core`](../capability-compression-core/spec.json) and [`deconvolution-native-core`](../deconvolution-native-core/spec.json), promoting demonstrated [`deconvolution-loop`](../deconvolution-loop/design.md) detection/measurement modules into `src/lesscode_baseline/`, and adding DSPy-style probabilistic multi-turn refinement `H10` for weaker/non-deterministic models).
- **Consulted Skills & Guidance**: `kiro-validate-design` (`rules/design-review.md`), `kiro-spec-design` (`rules/design-principles.md`, `rules/design-discovery-full.md`, `rules/design-discovery-light.md`, `rules/design-synthesis.md`, `rules/design-review-gate.md`), `.kiro/steering/{product,tech,structure,prototype-first,complexity,roadmap}.md`, `src/lesscode_baseline/AGENTS.md`, and `docs/symbolic-execution-tooling.md`.
- **Key Findings**:
  1. **H7 Withdrawal & H8/H9 Substrate Reuse**: Static CodeQL state-flow models and LLM-authored SMT equivalence proofs (`H7`) failed empirical controls (`R08`–`R18`) and were withdrawn in [`R19`](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md). The completed `lesscode-core` (`prepare`, `compare`, `admission`, `budgets`) and `lesscode_baseline.recording` (`C_app = 1,203 <= 1,364`, [`R31`](../../../docs/research/2026-11-07/05-descoped-stratum-rerun.md)) already provide the production scope, metric, admission, receipt, and trace-oracle substrate.
  2. **Two-Tier Failure Anatomy in Multi-Turn Regeneration (`R26` / E4)**: In E4 (`R26`), Turn 1 failed on `stop_reason == "max_tokens"` (128k thinking tokens, 0 JSON output) and Turn 2 failed on a missing recorder-imported surface symbol (`labeled_reason`). Both failures are **statically detectable in $< 1\text{ s}$** without spinning up embedded PostgreSQL or running 70+ `pytest` seeds, motivating a two-tier reward/assertion gate.
  3. **DSPy v2.6+ `Refine` & `BestOfN` Pattern**: Modern DSPy (`>= 2.6`) replaces deprecated `dspy.Assert`/`dspy.Suggest` with `dspy.BestOfN(module, N, reward_fn, threshold, fail_count)` and `dspy.Refine(module, N, reward_fn, threshold, fail_count)` where `reward_fn(args, pred) -> float` scores each rollout and `Refine` injects structured diagnostic feedback into subsequent attempts when `reward < threshold`. Implementing this exact `(module, N, reward_fn, threshold, fail_count)` contract stdlib-first in `src/lesscode_baseline/regenerator.py` (with an optional lazy-loaded `dspy` adapter) keeps `src/lesscode_baseline/` import-clean while supporting multi-turn recovery on weaker/non-deterministic models (`anthropic` and `openai`-compatible endpoints).

### Research Log (2026-11-07)

#### Topic 1: DSPy `Refine` / `BestOfN` vs. Legacy `Assert` / `Suggest`
- **Context**: Owner requested adopting DSPy-style probabilistic prompt iterations so weaker models or non-deterministic responses recover via additional turns rather than failing a trial.
- **Sources Consulted**: DSPy v2.6+ official documentation (`dspy.Refine`, `dspy.BestOfN`, `dspy.Signature`, `MIPROv2`), `docs/experiments/deconvolution/regenerator/regen_turn.py`, and [`R26`](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md).
- **Findings**:
  - `dspy.BestOfN` executes up to `N` rollouts with distinct rollout IDs / temperature schedules and returns the first candidate meeting `reward >= threshold` (or highest reward overall).
  - `dspy.Refine` extends `BestOfN` with an automatic feedback loop: failed rollouts produce structured critique/feedback injected into the next rollout's prompt.
  - Separating the reward function into **Tier 1 (`StaticAssertionGate`: $R \in [0.0, 0.49]$)** and **Tier 2 (`RuntimeOracleGate`: $R \in [0.50, 1.0]$)** avoids running expensive database/test-suite replays on syntactically invalid, out-of-scope, surface-breaking, or write-leaking rollouts.
- **Implications**: `src/lesscode_baseline/regenerator.py` implements `RefineController` and `BestOfNSelector` natively over `TurnRequest`/`TurnResult` using the two-tier `reward_fn`, plus a lazy `DspyRefineAdapter` when `dspy` is installed.

#### Topic 2: Consolidating Demonstrated Loop Apparatus into `src/lesscode_baseline/`
- **Context**: `surface.py` (60 lines), `s9_surface_contracts.py` (79 lines), `measures.py` (54 lines), `measures2.py` (46 lines), `measures3.py` (69 lines), `pureshare.py` (37 lines), `editscope.py` (65 lines), `testinfra.py` (626 lines), and `regen_turn.py` (337 lines) currently live scattered across `docs/experiments/deconvolution/`.
- **Sources Consulted**: All 9 prototype files, `src/lesscode_baseline/recording.py`, `src/lesscode_baseline/AGENTS.md`, and [`R27`](../../../docs/research/2026-11-07/01-solution-first-native-slice.md)–[`R32`](../../../docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md).
- **Findings**:
  - `surface.py` and `surface4.py` are identical except for the `SET` list; parameterizing `scope_files: Sequence[str]` and `recorder_files: Sequence[str] = ()` unifies Contract A and Contract B (`S9`/`R29`) in one ~90-line module (`src/lesscode_baseline/surface.py`) including Tier-1 static `patched`-global local-rebinding checks (`patchcheck.py` `rebound()`).
  - `measures.py`, `measures2.py`, `measures3.py`, `pureshare.py`, and `editscope.py` all operate on `root: Path` + `scope_files: Sequence[str]` using Python's stdlib `ast`, `re`, `hashlib`, and `subprocess` (`ruff==0.17.3 format`). Unifying them into `src/lesscode_baseline/measures.py` (~190 lines) eliminates `sys.path` mutation hacks (`_m.SET = SET` in `measures4.py`/`pureshare4.py`).
  - `testinfra.py` uses only Python stdlib (`ast`, `configparser`, `tomllib`, `fnmatch`, `shlex`) and relocates cleanly to `src/lesscode_baseline/testinfra.py`.

### Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|---|---|---|---|---|
| **1. H7 Static Graph + Authored SMT (2026-11-02 proposal)** | CodeQL dataflow $\to$ LLM state model $T(\dots)$ $\to$ Z3 equivalence check | Minimal runtime execution | Falsified in `G0c`/`G0d` and `R16`–`R18`; unmeasurable model fidelity | **Rejected / Withdrawn (`R19`)** |
| **2. Mandatory Full-Stack DSPy + Rust Port** | Require `dspy-ai` at top-level import and port all Python AST/turn logic to Rust immediately | Single compiled binary | Breaches `U5`/`U7` apparatus budgets (`R30`: $C_{\text{app}} = 1,808 > 1,364$); adds heavy LLM SDK dependencies to baseline package | **Rejected (`R30`, `R31`)** |
| **3. Two-Tier Probabilistic `Refine`/`BestOfN` over Consolidated Python + Native Core (Selected)** | Promote `surface`, `measures`, `testinfra`, and two-tier `regenerator` (`Refine`/`BestOfN` + optional `dspy` adapter) into `src/lesscode_baseline/`, backed by `recording.py` and `lesscode-core` | Zero new required dependencies; fast Tier-1 static recovery (<1 s); preserves `U5` savings and all `R25`–`R32` contracts | Multi-rollout recovery on weaker models requires bounded `G1-R` validation before claiming general efficacy | **Selected (`R33`)** |

### Design Synthesis (`rules/design-synthesis.md`)

1. **Generalization**:
   - Parameterizing `scope_files` and `recorder_files` generalizes `surface.py`/`surface4.py`/`s9_surface_contracts.py` and `measures.py`/`measures4.py`/`pureshare4.py` across any Python scope without monkey-patching module-level `SET`.
   - Modelling candidate evaluation as a composable two-tier reward function $R(\text{req}, \text{cand}) \to \text{RewardEvaluation}(\text{score} \in [0, 1], \text{tier}, \text{passed}, \text{feedback})$ unifies single-turn validation (`N=1`), inner static assertion backtracking (`Tier 1`), outer runtime counterexample refinement (`Tier 2`), and `dspy.Refine`/`dspy.BestOfN` rollouts behind a single interface.
2. **Build vs. Adopt**:
   - **Adopt from stdlib / locked stack**: Python `ast`, `tomllib`, `configparser`, `hashlib`, `urllib.request`; pinned `ruff==0.17.3` CLI for canonical formatting; `cosmic-ray 8.7.0` and `CrossHair` via `uv tool run`; `lesscode-core` (`syn`, `proc-macro2`, `ast-grep-core`, `sha2`, `clap`) for native budgets, admission, receipts, and scope comparison.
   - **Adopt via optional adapter**: `dspy` (`dspy.Refine`, `dspy.BestOfN`, `dspy.Signature`, `MIPROv2`) lazy-loaded when `--backend dspy` is requested, while providing a 100% compatible stdlib `RefineController` default so tests and offline runs require zero external LLM frameworks.
   - **Reject**: CodeQL/Joern as behavioral state engines (`R08`–`R09`), `serde_jcs` (`R27`), `pg_query` (`R27`), `mutmut` (POSIX `fork` only), KLEE (`symbolic-execution-tooling.md`).
3. **Simplification**:
   - Collapse 7 scattered experiment scripts (`surface.py`, `surface4.py`, `s9_surface_contracts.py`, `measures.py`, `measures2.py`, `measures3.py`, `pureshare.py`, `editscope.py`) into 2 cohesive product modules (`src/lesscode_baseline/surface.py` and `src/lesscode_baseline/measures.py`).
   - Keep `recording.py` and `native/src/{admission,receipt,budgets}.rs` untouched (0 diff lines on `deconvolution-native-core` modules).

### Design Decisions

#### Decision 1: Two-Tier Programmatic Reward & Assertion Gate (`StaticAssertionGate` $\to$ `RuntimeOracleGate`)
- **Context**: Weaker models and non-deterministic sampling frequently fail on output truncation (`stop_reason == "max_tokens"`), JSON formatting, out-of-scope paths, syntax errors, missing Contract-B surface symbols (`labeled_reason`), or placing DB writes inside `pure` functions. Running the full PostgreSQL + `pytest` acceptance arm (`recording.py`, ~60–180 s) on such candidates wastes budget and obscures fast-recoverable errors.
- **Alternatives Considered**:
  1. Single-tier runtime evaluation after every LLM call (the E4 `regen_turn.py` manual workflow).
  2. Unchecked multi-sample temperature sweeps scored only by code length.
- **Selected Approach**: Partition candidate evaluation into:
  - **Tier 1 (`StaticAssertionGate`, $R \in [0.0, 0.49]$)**: Checks completion/JSON ($0.10$), `editscope` path confinement ($0.20$), Python `ast.parse` syntax ($0.30$), Contract-B `surface` + `patched`-global local-rebinding check ($0.40$), and `measures` `pure`/`shell` structural check + static edge/function/line budgets ($0.49$). Any Tier-1 failure triggers immediate inner `Refine` retry (`max_static_retries <= 3`) with parameter adaptation (e.g., capping thinking tokens on `max_tokens`, incrementing `rollout_id`/temperature on syntax/surface errors) and exact diagnostic feedback without running `recording.py`.
  - **Tier 2 (`RuntimeOracleGate`, $R \in [0.50, 1.0]$)**: Runs `recording.py` (`probe`, `run_groups`, `verdict`) and canonical/separated-form `measures`. Partial seed/replay progress scores in $[0.50, 0.89]$; passing replay with measure gaps scores in $[0.90, 0.99]$; full acceptance (`verdict == 0`, `structure.pass == True`, all targets met) scores $1.0$.
- **Rationale**: Directly prevents E4 Turn 1 and Turn 2 failure modes from consuming runtime oracle turns while remaining 100% deterministic and fail-closed.
- **Trade-offs**: Adds ~120 lines of structured reward/retry logic in `regenerator.py`, offset by eliminating bespoke per-experiment retry wrappers.
- **Follow-up**: Validate on frozen E3/E4 artifacts and adverse controls in spike `G1-R`.

#### Decision 2: Dual-Endpoint (`anthropic` + `openai`) & Lazy `dspy` Adapter Architecture
- **Context**: Weaker or open-weight models are commonly served via OpenAI-compatible HTTP endpoints (`v1/chat/completions` with `response_format={"type": "json_schema", ...}`), whereas `regen_turn.py` previously raised `NotImplementedError` for `style == "openai"`.
- **Selected Approach**: Support both `anthropic` and `openai` styles in `regenerator.py` (using stdlib `urllib.request` for `openai` JSON-schema chat completions so no new SDK is required, while keeping the existing `anthropic` streaming client path and lazy `dspy.LM`/`dspy.Refine` bridge).
- **Rationale**: Enables testing weaker/local models and OpenAI-compatible providers without expanding mandatory dependencies.

### Risks & Mitigations
- **Risk 1: Multi-turn retry loops masking oracle weakness or overfitting to weak tests** — *Mitigation*: Frozen acceptance checks (`recording.py`) and mutation sensitivity (`t4_arm`) are read-only and outside the regenerator's reach; `Req 7.5` strictly separates any DSPy prompt-optimization training scopes from held-out evaluation scopes.
- **Risk 2: Unbounded token consumption across `Refine` retries** — *Mitigation*: Enforce hard preregistered budgets (`max_turns`, `max_static_retries`, `max_total_input_tokens`, `max_total_output_tokens`) and a deterministic `no_progress_limit` (`k = 2` outer turns without strict improvement in $\max(R)$ or a newly discharged obligation, `Req 4.5, 4.6`).
- **Risk 3: Split spec ownership between `constraint-guided-compression`, `deconvolution-loop`, and `deconvolution-native-core`** — *Mitigation*: Explicit Boundary Commitments in `design.md`: `deconvolution-native-core` owns `native/src/{admission,receipt,budgets}.rs` and `src/lesscode_baseline/recording.py`; this spec owns `src/lesscode_baseline/{surface,measures,testinfra,regenerator}.py` and the `G1-R` probabilistic refinement gate.

### References
- [R33: H8/H9 + DSPy-style probabilistic turn refinement design](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md)
- [R32: TestInfraDetector on the Rust specimen](../../../docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md)
- [R31: Descoped replacement stratum — attempt 2](../../../docs/research/2026-11-07/05-descoped-stratum-rerun.md)
- [R29: S9 surface contracts (Contract A vs Contract B)](../../../docs/research/2026-11-07/03-s9-surface-contracts.md)
- [R26: E4 lifecycle regeneration](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md)
- [R25: E3 evidence regeneration](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md)
- [R19: Dynamic deconvolution direction](../../../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md)
- [DSPy `Refine` and `BestOfN` documentation](https://dspy.ai/)

---

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


Amendment, 2026-11-04: the guided-generation rung was run twice on the Facdrone
evidence-publication scope. [R23](../../../docs/research/2026-11-03/05-e1c-concolic-rung.md)
failed because of driver encoding; [R24](../../../docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md)
reran it with a solver-encoding verifier and adaptive budgets, killed 7 of 10
residue targets, proved none equivalent, and fired the preregistered kill
criterion. Solver verdicts count in this spec only after the four verifier
checks in [symbolic-execution-tooling.md](../../../docs/symbolic-execution-tooling.md)
pass, and never as equivalence authority on database-entangled predicates.

Amendment, 2026-11-04 (evening): H8(c) regeneration ran as E3 and is recorded as
[R25](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md). One
round met the recorded oracle on the five evidence modules with 16% fewer lines,
29% fewer intra-set edges and a statically checked pure core; mutation
sensitivity missed its preregistered bar (18/28 = 0.643) and one oracle-blind
difference on an exception path is real. Two protocol rules are proposed, not
yet adopted: report statement counts in the original's formatting as well as the
candidate's, and draw sensitivity samples from the original and the candidate
alike. Gate remains `revise`; the owner decides between a second round (E3b), a
second scope (E4), or prevention at generation time.

Amendment, 2026-11-06: H8(c) ran on a second Facdrone scope as E4 and is recorded
as [R26](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md). The
recorded regenerator program accepted at its third turn on the seven lifecycle
monitor files with 25% fewer functions and 29% fewer intra-scope edges inside the
preregistered bands, 11 write sites in declared shells and a checked pure core;
the line prediction failed (1,405 canonical lines against at most 1,345, 6%
fewer). Sensitivity 0.679 on the candidate held its band, the original scored
0.414 on its own sample by composition, and neither meets 0.70; seven
oracle-blind differences on error and malformed-input paths are concrete; a
corpus of 74 sequences recorded after acceptance (26 of them the acceptance
seed sequences replayed again, 47 new; 1 generated sequence duplicates an
acceptance one) replays with zero divergence;
the differential is void on a failed solver control. Gate remains `revise`; the
owner decides between a third scope, adoption of the E4 candidate outside the
loop, or prevention. Four protocol amendments are proposed for the
deconvolution-loop spec: the surface includes the recorder's imports, turn
parameters are preregistered, content hashes are over bytes as stored, and
receipt labels are distinct beyond letter case.
