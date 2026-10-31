# Requirements Document

## Introduction
Python maintainers and teams using coding agents need to prevent unnecessary structure before it accumulates. Lesscode already compares supplied snapshots under fixed checks and measurements, and the repository has informal coding guidance. It does not yet provide a configurable preventive policy workflow with scoped limits, change budgets, debt accounting and actionable agent feedback.

The Agent Rules Evaluator will expose applicable rules before editing, assess supplied changes against recorded policy, and provide guidance that can be applied to repository instruction files. It will distinguish executable measurements from contextual review of naming, idiom and responsibilities. Developers retain control of the policy; shorter code, lower counters and supplied review claims do not independently establish preserved behavior or reduced maintenance effort.

## Boundary Context
- In scope: local Python policy evaluation; configurable function-complexity and source-size constraints; duplication constraints where matching evidence exists; scoped change budgets and debt ratchets; contextual review criteria; safe requested AGENTS.md guidance updates; exceptions; inspectable findings and reproducible examples.
- Out of scope: generating or applying source-code refactors, replacement analyzers, automatic semantic certification, AI-authorship detection, universal simplicity scores, inferred maintenance hours, launching coding-agent trials, mandatory MCP and non-Python evaluation.
- Adjacent expectations: efficiency-baseline supplies source identities, measurement definitions, declared behavior evidence and integrity outcomes. Project owners supply checks, reference evidence, policy and review records. Later skills consume these rules and findings rather than replacing executable gates.
- Existing limitations: whole-package duplication in the self-evaluation is incomplete because the pinned scanner omits a nonempty short file. Resolving that coverage limitation is upstream measurement work. LLOC, nesting and architecture evidence are not assumed to be available built-in measurements; supported supplied evidence must retain its own definition and provenance.

## Terms
- Policy: the declared rules, target scopes, reference conditions, enforcement levels and exceptions recorded for a check.
- Blocking rule: a rule that must be satisfied for overall policy acceptance. Advisory findings remain visible without independently blocking acceptance.
- Change-ratchet mode: assess new or worsened violations against an explicit reference; unchanged inherited debt is reported separately.
- Absolute mode: assess every applicable entity against the declared limit, including inherited debt.
- Semantic review criterion: a contextual question requiring an identified review outcome and rationale; it is distinct from a deterministic metric check.
- Policy acceptance: all blocking rules and required review evidence are satisfied under the recorded policy and underlying evaluation. It is not proof of equivalence or a universal quality rating.

## Requirements

### Requirement 1: Declare and inspect preventive policy
**Objective:** As a maintainer, I want to see the applicable rules before coding, so that an agent can avoid preventable convolution.

#### Acceptance Criteria
1. When a developer or agent requests rules for a target scope, the Agent Rules Evaluator shall display the applicable constraints, review criteria, reference conditions and enforcement levels.
2. When a policy is supplied, the Agent Rules Evaluator shall identify each rule by a unique identity, applicable scope, evidence quantity or review criterion, and declared acceptance condition.
3. When a policy check is prepared, the Agent Rules Evaluator shall record the exact policy identity and its reference conditions before assessing the candidate.
4. If a policy contains duplicate rule identities, invalid bounds, unsupported rule kinds or an unsupported policy version, the Agent Rules Evaluator shall reject the configuration with the affected rule and reason.
5. When multiple rules apply to the same code, the Agent Rules Evaluator shall expose all applicable rules and their declared enforcement levels without silently dropping one.
6. When guidance is requested without a candidate evaluation, the Agent Rules Evaluator shall label it as pre-change guidance rather than a verified post-change result.
7. The Agent Rules Evaluator shall apply no undocumented global reduction target or implicit universal simplicity threshold.

### Requirement 2: Evaluate explicitly defined measurements
**Objective:** As a maintainer, I want constraints on identified quantities, so that a measured result has a reproducible meaning.

#### Acceptance Criteria
1. When a function-complexity rule is configured, the Agent Rules Evaluator shall evaluate the identified function-level cyclomatic-complexity values and retain their source locations and measurement definition.
2. When a source-size rule is configured, the Agent Rules Evaluator shall evaluate its declared scalar quantity and unit without substituting a different line or byte definition.
3. When a duplication rule is configured, the Agent Rules Evaluator shall identify the clone or token quantity, measured population and denominator used to evaluate it.
4. Where a logical-line rule is supported by supplied evidence, the Agent Rules Evaluator shall retain the LLOC producer, definition and source scope without substituting physical lines or NLOC.
5. Where nesting, architecture or public-contract evidence is used by a rule, the Agent Rules Evaluator shall identify the supplied findings or declared checks rather than imply that unsupported analysis was performed.
6. When a numeric maximum is evaluated, the Agent Rules Evaluator shall accept equality with the declared maximum and report the observed value, limit and comparison basis.
7. If a percentage comparison has a zero reference value or an undefined scalar quantity, the Agent Rules Evaluator shall report the comparison unavailable with its reason.
8. When a rule constrains individual entities, the Agent Rules Evaluator shall retain each violating entity rather than cancel its regression with improvement elsewhere.

### Requirement 3: Separate inherited debt from new regressions
**Objective:** As a maintainer, I want ratchets that prevent new debt without concealing existing debt, so that adoption does not require an unrelated repository-wide rewrite.

#### Acceptance Criteria
1. When change-ratchet mode is selected, the Agent Rules Evaluator shall bind the comparison to an explicitly selected reference with identifiable source, scope and measurement conditions.
2. When a referenced violation is unchanged in change-ratchet mode, the Agent Rules Evaluator shall report it as inherited debt rather than a newly introduced regression.
3. When a new or matched entity introduces a limit violation or exceeds its applicable change budget, the Agent Rules Evaluator shall report the affected new or worsened violation.
4. When a violation improves but remains above its applicable reference limit, the Agent Rules Evaluator shall report both the improvement and residual debt.
5. When absolute mode is selected, the Agent Rules Evaluator shall apply the absolute limit to inherited debt as well as new code and distinguish this behavior from change-ratchet mode.
6. When a referenced entity is removed, the Agent Rules Evaluator shall record that removal without treating it as independent proof of behavior or API preservation.
7. If passing a blocking ratchet would depend on an uncertain entity match after a rename, split, merge or other change, the Agent Rules Evaluator shall disclose the uncertainty and withhold that pass until its reference treatment is established.
8. When the reference is explicitly updated, the Agent Rules Evaluator shall identify the new reference and preserve prior findings rather than silently rebasing failed checks.

### Requirement 4: Preserve behavior, comparability and incomplete evidence
**Objective:** As a reviewer, I want unknown or failed evidence to remain visible, so that policy cannot manufacture a clean result.

#### Acceptance Criteria
1. If the underlying evaluation is invalid, baseline-ineligible, rejected or incomplete, the Agent Rules Evaluator shall retain that outcome and prevent an overall policy-accepted claim.
2. If evidence needed by a blocking rule is missing, unavailable or non-comparable, the Agent Rules Evaluator shall report the rule as insufficiently evidenced rather than satisfied.
3. When advisory evidence is unavailable, the Agent Rules Evaluator shall disclose its status and reason without converting it into zero or a passed check.
4. If measurement definitions, producers, versions, units, scopes or required populations differ, the Agent Rules Evaluator shall identify the differences and prevent unsupported comparisons.
5. If a rule targets a narrower population than its supplied aggregate evidence covers, the Agent Rules Evaluator shall require matching-population evidence or report the affected comparison unavailable.
6. If required measurement coverage omits nonempty source within the declared scope, the Agent Rules Evaluator shall preserve the incomplete result without silently excluding that source or reclassifying it as zero.
7. The Agent Rules Evaluator shall prevent structural improvements or policy exceptions from compensating for failed declared behavior, required invariants or integrity checks.

### Requirement 5: Review convolution, idiom and role clarity
**Objective:** As a reviewer, I want explicit contextual criteria alongside static measurements, so that lower counters do not stand in for understandable code.

#### Acceptance Criteria
1. When semantic guidance is requested, the Agent Rules Evaluator shall provide criteria for role-revealing names, idiomatic constructs, responsibility boundaries, repeated policy, unnecessary indirection and speculative abstractions.
2. When a semantic review is supplied, the Agent Rules Evaluator shall record the criterion, outcome, reviewer or producer identity, source and policy identities, rationale and supporting code locations.
3. The Agent Rules Evaluator shall distinguish contextual review outcomes from deterministic metric results and shall not certify naming or idiom solely from identifier counts or banned-word lists.
4. If a policy requires semantic review and that review is absent, unresolved or bound to different source or policy identities, the Agent Rules Evaluator shall withhold overall policy acceptance.
5. When a review record is producer-supplied, the Agent Rules Evaluator shall disclose that provenance without claiming that local records authenticate the reviewer's independence or identity.
6. When guidance addresses names or abstractions, the Agent Rules Evaluator shall preserve the distinction between a readability recommendation and authorization to change public fields, signatures or required behavior.
7. When abstraction guidance is presented, the Agent Rules Evaluator shall include review criteria for the demonstrated problem, simpler alternatives and added conceptual or change burden rather than recommend a design pattern as an end in itself.
8. If a required review's trust or independence condition is not established by its supplied evidence, the Agent Rules Evaluator shall report that requirement unverified rather than satisfied.

### Requirement 6: Apply guidance safely at the point of editing
**Objective:** As a maintainer, I want policy-backed repository instructions, so that agents can follow the rules while producing code.

#### Acceptance Criteria
1. When repository guidance is generated, the Agent Rules Evaluator shall identify its target scope, policy identity, executable rules, contextual review criteria and unsupported or unavailable checks.
2. The Agent Rules Evaluator shall include guidance to preserve behavior and evidence boundaries, prefer deletion or reuse before new structure, use idiomatic constructs and choose names that reveal code roles.
3. When a guidance update is requested, the Agent Rules Evaluator shall expose the proposed target and text before writing a designated repository instruction file, including AGENTS.md.
4. When an explicitly requested guidance update is applied, the Agent Rules Evaluator shall preserve unrelated instructions, user-authored content and files outside the designated update.
5. If the designated managed content has changed since the update was prepared, the Agent Rules Evaluator shall report the conflict and leave the existing file unchanged.
6. When unchanged policy is applied again to unchanged managed guidance, the Agent Rules Evaluator shall leave the guidance content unchanged.
7. The Agent Rules Evaluator shall state the applicable scope of generated guidance and preserve existing instruction precedence without claiming to resolve all natural-language policy conflicts automatically.

### Requirement 7: Make exceptions and policy integrity inspectable
**Objective:** As a policy owner, I want visible, bounded exceptions and fixed rules, so that an optimizing agent cannot weaken its own evaluation.

#### Acceptance Criteria
1. When an exception is declared, the Agent Rules Evaluator shall identify its affected rule, target scope, justification and explicit validity conditions.
2. When an exception affects a result, the Agent Rules Evaluator shall show whether it is active, expired or inapplicable and retain the underlying finding.
3. If an exception would waive required behavior, integrity or missing mandatory measurement evidence, the Agent Rules Evaluator shall refuse to present it as satisfying those requirements.
4. If policy, thresholds, scope exclusions, reference conditions or required check inventory change during an assessment, the Agent Rules Evaluator shall invalidate that assessment and identify the changed inputs.
5. When policy changes are intentionally requested, the Agent Rules Evaluator shall require a newly identified policy assessment rather than relabel an earlier failed result.
6. When enforcement protections are described, the Agent Rules Evaluator shall distinguish local change detection, producer claims and externally controlled enforcement.

### Requirement 8: Produce usable local results
**Objective:** As a developer or automation owner, I want actionable results that can be inspected locally, so that rules can fit an existing review workflow.

#### Acceptance Criteria
1. When a policy check completes, the Agent Rules Evaluator shall produce a human-readable result and a machine-readable result with an identified format version.
2. When a rule produces a finding, the Agent Rules Evaluator shall retain the rule identity, scope, observed and expected evidence, comparison basis, status, supporting locations and evidence references.
3. When results are summarized, the Agent Rules Evaluator shall distinguish inherited debt, new or worsened violations, improvements, exceptions, unavailable evidence and advisory versus blocking findings.
4. When measurements are presented, the Agent Rules Evaluator shall retain improving, unchanged and worsening observations rather than report only favorable counters.
5. When results are returned to automated callers, the Agent Rules Evaluator shall distinguish a satisfied assessment, violations, insufficient evidence and invalid inputs or integrity from successful guidance or report production.
6. If an assessment cannot finish or persist all results, the Agent Rules Evaluator shall expose the partial outcome and retain available findings and diagnostics without claiming completion.
7. The Agent Rules Evaluator shall support the documented local workflow on Windows, Linux and macOS without requiring an LLM service, a paid analyzer account or source upload.
8. When any project workload is executed, the Agent Rules Evaluator shall expose the commands and limits beforehand and preserve the developer's original source and unrelated changes.
9. When the same identified policy, evidence, review records and exception-validity context are evaluated again, the Agent Rules Evaluator shall produce the same rule findings and verdict apart from execution metadata.

### Requirement 9: Make examples and benefit claims reproducible
**Objective:** As an adopter, I want evidence about the rules and their limits, so that I can assess usefulness without relying on a prompt's claims.

#### Acceptance Criteria
1. The Agent Rules Evaluator shall provide reproducible examples covering unchanged inherited debt, new or worsened violations, improvement with residual debt, uncertain entity matches, unavailable evidence, exceptions and preservation of existing repository guidance.
2. When a required semantic criterion remains unreviewed despite lower structural counters, the Agent Rules Evaluator shall demonstrate that the lower counters do not alone establish overall policy acceptance.
3. When policy-guided and unguided trial records are compared, the Agent Rules Evaluator shall identify policy and guidance versions and disclose differences in declared controlled conditions.
4. When operational cost or maintenance effort is reported, the Agent Rules Evaluator shall distinguish observed task, review or agent telemetry from static proxies and label unsupported savings claims unmeasured.
5. The Agent Rules Evaluator shall make no general effectiveness, equivalence or maintenance-effort claim from fixture success or a supplied review record alone.
