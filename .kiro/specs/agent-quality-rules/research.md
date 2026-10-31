# Research and Design Decisions

## Summary
- **Feature**: agent-quality-rules.
- **Discovery Scope**: Extension; integration-focused light discovery. No new external library, service, analyzer or runtime is selected.
- **Key Findings**: baseline run recovery is reusable, but storage completion is separate from acceptance; structured metrics need explicit policy selectors; function records do not establish cross-revision identity; scope hashes do not identify actual populations.
- Requirements were approved by the explicit `$kiro-spec-design agent-quality-rules -y` invocation. Design approval remains a separate phase.

## Research Log

### Existing evidence and storage seams
- **Sources consulted**: `src/lesscode_baseline/contracts.py`, `report.py`, `decision.py`, `process.py`, `evaluation.py`; read-only delegated codebase discovery by `rules_design_integration`.
- **Findings**: `report.recover(Path)` verifies indexed bytes and required report references. It does not authenticate producers or validate an optional prepared plan's semantic bindings. `RunReport` contains both observations and the upstream decision. `PreparedPlan` carries fixed profile/check/tool/resource identities. `RunStore` is baseline-specific.
- **Implications**: consume completed run stores, require/validate indexed `plan.json` for policy assessment, preserve upstream outcomes, and use separate policy result storage. Do not change baseline schemas or reuse private persistence helpers as public interfaces.

### Quantities, populations and entity identity
- **Sources consulted**: `FunctionComplexity`, `ComplexityPayload`, `DuplicationPayload`, `MetricEvidence`, `OptionalEvidenceArtifact`; `decision.compare_metrics`; `docs/measurement-guide.md`; published self-evaluation scope limitations.
- **Findings**: generic baseline thresholds have no structured-metric projection. Function location/name tuples are inventory IDs, not stable semantic identities. Scope hash covers selection rules, while snapshot and payload inventories establish actual populations. Duplication is pair-span accounting, not unique-source coverage; clone spans cannot reconstruct a subset denominator. Supplied scalar LLOC remains distinct from physical lines and NLOC.
- **Implications**: finite explicit selectors; match aggregate populations; never filter duplication into a narrower denominator. Unchanged file bytes support automatic matching; changed-file continuity requires a disclosed source-bound treatment. No inferred rename or split/merge equivalence. Absolute-only numeric rules must not unnecessarily depend on reference metrics.

### Existing CLI and experiment integration
- **Sources consulted**: `cli.py`, `pyproject.toml`, `TrialConditions`, `Arm`, `ExperimentManifest`, `TrialRecord`, `experiments.reconcile` and `aggregate`.
- **Findings**: argparse provides additive nested command registration; current startup is analyzer-free. Existing interventions carry named primitive values and enforce declared arm/control differences. No schema extension is necessary for policy/guidance hashes and renderer versions.
- **Implications**: add a nested rules workflow within the installed command and retain existing exits/commands. Reuse supplied-trial summarization; do not introduce an agent trial runner or a second statistical engine.

### Guidance and trust
- **Sources consulted**: root AGENTS instructions, `src/lesscode_baseline/AGENTS.md`, requirements review cases, core steering.
- **Findings**: current instructions cover role-revealing names, idioms, meaningful boundaries and preservation of evidence/public fields. They are guidance, not an executable engine. Hashes catch inconsistent changes but do not establish external ownership or reviewer identity.
- **Implications**: render instructions from the same typed policy, label unsupported review trust, preview before explicit application and preserve unrelated bytes. A local `external_independence` requirement remains unverified in this version. No unsupported authentication service is hidden in the implementation plan. Portable filesystem replacement is not compare-and-swap against arbitrary editors; require a quiescent target and disclose the cooperating-writer limitation.

### Consulted workflow guidance
- Applied `kiro-spec-design` and its design principles, light discovery, synthesis and bounded review rules; used the repository design/research templates.
- Applied core steering and package guidance on typed evidence, explicit roles, no speculative layers and behavior before counters.
- No web dependency research was necessary for this extension: all selected runtime APIs already belong to the pinned standard-library/baseline stack. No third-party API/version recommendation is introduced.

## Architecture Pattern Evaluation

| Option | Strengths | Risks / limitations | Decision |
|---|---|---|---|
| Extend baseline Profile/Threshold and RunReport | One result format | Mixes measured facts with downstream policy; breaks schema and consumer ownership | Reject |
| Separate installed package and console command | Strong deployment separation | Additional packaging/startup surface without a current distribution requirement | Defer |
| Internal rules package with nested existing CLI | Independent policy types and pure evaluation; minimal deployment change | Must enforce imports toward baseline only | Select |
| Generic external policy engine or expression language | Broad policy syntax | Requires translation, dependencies and execution surface; does not solve source/evidence trust | Reject for this finite rule set |
| Skill-only enforcement | Fast instruction delivery | Cannot deterministically preserve unavailable evidence or evaluate ratchets | Downstream consumer only |

## Design Decisions

### Decision: Generalize evidence selection, not analyzer execution
- **Context**: complexity, size, duplication and optional LLOC require related bound comparisons.
- **Selected approach**: one numeric rule variant with explicit finite selectors, metadata and bounds, plus separate check/review variants.
- **Alternatives**: one rule class per analyzer or a general expression evaluator.
- **Rationale**: one comparison contract captures the shared problem without building hypothetical adapters. Semantic review is not another number.
- **Trade-offs**: unsupported structured findings cannot be reduced automatically; use a declared check or explicit supported scalar evidence.
- **Follow-up**: tests distinguish absolute candidate-only evaluation from reference-dependent budgets and ratchets.

### Decision: Adopt baseline evidence and native primitives
- **Context**: recovery, comparisons, trial reconciliation and immutable source identities already exist.
- **Selected approach**: reuse public baseline contracts/recovery, stdlib dataclasses/JSON/hash/filesystem operations and argparse. Keep policy persistence separate.
- **Rationale**: a new library would not remove the domain-specific work of interpreting evidence, reference treatment and trust. Reusing RunStore directly would misrepresent policy reports as baseline runs.
- **Trade-offs**: a small policy storage lifecycle repeats the index-last pattern; private internals remain private. No new dependency maintenance or licensing burden.
- **Follow-up**: native installed smoke tests and unchanged baseline regression tests.

### Decision: Conservative changed-entity treatment
- **Context**: stable symbol identity is absent from current evidence.
- **Selected approach**: automatic matching only for unchanged files; source-bound explicit treatment for changed inventory entities; unresolved grandfathering is insufficient.
- **Rationale**: name similarity is weaker evidence than a ratchet pass implies. An explicit claim is inspectable without claiming semantic authentication.
- **Trade-offs**: additional adopter input, especially in large changed files. No new parser or heuristic matching engine in this spec.
- **Follow-up**: demonstrate rename/split ambiguity and new-entity strict limits. Later automation may propose treatments but cannot silently certify them.

### Decision: Keep workflow seams within one policy feature
- **Context**: guidance and assessment have different IO paths but must share exact policy meaning.
- **Selected approach**: policy contracts are the seam; rendering is a projection, evaluation never consumes prose. Use a small functional core and explicit IO modules.
- **Rationale**: this supports the single capability of applying one preventive policy before/after editing without a second rule interpreter. No service classes or framework are needed.
- **Trade-offs**: guidance application needs its own preservation tests. It is the only source-repository mutation in the feature.
- **Follow-up**: task boundaries follow concrete files; package/CLI registration changes are serialized during implementation.

### Decision: Preserve review limitations and original outcomes
- **Context**: required semantic review, exceptions and baseline acceptance must not compensate for each other.
- **Selected approach**: retain separate upstream outcome, policy verdict and accepted boolean; exceptions apply only to evidenced numeric violations. Local producer reviews can satisfy explicitly permissive policy; externally independent trust remains unverified.
- **Rationale**: missing facts cannot be promoted into passes by prose or numerical improvement.
- **Trade-offs**: some policies are intentionally unsatisfiable locally until evidence/trust support exists; show this in pre-change guidance.
- **Follow-up**: test lower CCN with missing review, claimed reviewer independence and attempted behavior/evidence waivers.

## Risks & Mitigations
- **Identity ambiguity**: disclose matching claims and withhold uncertain inherited-debt passes.
- **Population mismatch**: validate file inventories as well as selection hashes; preserve short-file incompleteness.
- **Metric gaming**: entity findings, semantic criteria, unchanged upstream checks and unfavorable observations remain visible.
- **Local actor can alter everything**: state the limit; externally controlled CI pins authority, outside this feature's configuration responsibilities.
- **Concurrent editor during guidance replacement**: full-file precondition, cooperating lock, temporary replacement and explicit quiescence limitation; no portable atomic-CAS claim.
- **Spec expansion**: no external identity service, general policy language, baseline runner, source parser or MCP is required.

## References
- [Requirements](requirements.md) and [requirements review cases](requirements-review.md).
- [Baseline contracts](../../../src/lesscode_baseline/contracts.py), [run recovery](../../../src/lesscode_baseline/report.py), [comparisons](../../../src/lesscode_baseline/decision.py), [trial reconciliation](../../../src/lesscode_baseline/experiments.py).
- [Measurement guide](../../../docs/measurement-guide.md) and [self-evaluation](../../../docs/examples/self-evaluation.md).
- [Package agent guidance](../../../src/lesscode_baseline/AGENTS.md).
