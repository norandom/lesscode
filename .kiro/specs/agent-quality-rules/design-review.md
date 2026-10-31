# Design Draft Review

## Result
PASS, 2026-10-31. This is the design-generation quality gate, not human design approval or implementation validation. Review was inline in the main context; the codebase discovery was independently delegated, but no independent design-review claim is made.

The draft was reviewed in `.git/agent-quality-rules-design-draft.md` before publication to `design.md`. Two bounded local repair passes were used; no requirements change was needed.

## Mechanical Checks
- All 65 criteria extracted from the nine numbered requirement sections appear in the design's traceability table and map to concrete contracts/flows.
- Boundary Commitments, This Spec Owns, Out of Boundary, Allowed Dependencies and Revalidation Triggers are populated.
- File Structure Plan assigns concrete paths to all seven new components plus their shared contracts/package, tests, fixtures and documentation. Baseline recovery, comparison and experiment components are explicitly existing dependencies with unchanged source locations.
- No TODO, FIXME or TBD placeholders; Markdown fences are balanced. File plan stays inside policy interpretation, guidance and integration, with no analyzer/agent runner or rewrite engine implied.

## Judgment Review

| Dimension | Finding | Resolution |
|---|---|---|
| Upstream ownership | Stored completion is distinct from evaluation acceptance; PreparedPlan needs explicit validation | Require completed recovery and indexed plan bindings; retain upstream outcome separately |
| Population meaning | Scope hash alone does not prove measurement population | Check snapshot/payload inventories; never slice aggregate denominators |
| Entity continuity | Function names/locations cannot prove cross-revision identity | Automatic matches only in unchanged files; changed entities need source-bound treatment, uncertain passes withheld |
| Comparison scope | Absolute-only rules could accidentally inherit reference requirements | Candidate-only absolute evaluation; comparator compatibility applies to reference-dependent comparisons |
| Behavioral safety | Numerical exceptions must not waive upstream failure or missing evidence | Exceptions restricted to evidenced numeric violations; original baseline outcome remains authoritative |
| Semantic trust | Producer identity text cannot authenticate independence | Disclosed producer-record mode; external-independence condition remains unverified in version 1 |
| Semantic locations | Inventory membership cannot establish rationale accuracy | Validate paths/ordered spans; explicitly preserve producer-claim limitation |
| Named checks | Rule behavior must be explicit for missing/failed evidence | Exact candidate check selection; binding validation and insufficient/violation handling; upstream check inventory unchanged |
| Guidance preservation | Prior managed-block ownership and repeated apply needed concrete treatment | Marker/body digest, full-file precondition, explicit preview/apply, no-op repeat and conflict handling |
| Concurrent write | Portable atomic replacement is not compare-and-swap | Cooperating lock and immediate precondition check; quiescent-target limitation stated, no adversarial guarantee |
| Determinism | Exception expiry could depend on wall clock | Explicit captured as_of and canonical finding order |
| Scope growth | New package/engine/service could be unnecessary | Nested existing CLI, finite selectors, no new dependencies, reuse existing trial interventions |
| Executability | Implementers need bounded ownership and verification | Concrete module paths, typed service contracts, lifecycle and failure semantics, requirement-derived tests and native installed workflows |

## Repair Passes
1. Added actual-population versus scope-hash distinction, candidate-only absolute evaluation, explicit review scope and managed-body digest semantics.
2. Clarified the limit of review location validation and specified named-check evaluation, including missing evidence and preservation of all upstream behavior checks.

## Remaining Deliberate Limitations
- Changed-function treatments and review records are inspectable producer claims; no semantic or identity authentication is introduced.
- Required external independence cannot be established by this local version.
- Short-file duplication incompleteness remains upstream and blocks acceptance when required.
- Guidance application requires a quiescent target; noncooperating concurrent writers are outside its synchronization guarantee.
- Fixture success and policy output do not establish effectiveness or maintenance savings.

## Next Gate
Human review of design is required by root AGENTS.md: "Human review required each phase; use `-y` only for intentional fast-track". `$kiro-spec-tasks agent-quality-rules -y` approves this concrete design and generates the next phase. Tasks and implementation are not authorized by design generation alone.
