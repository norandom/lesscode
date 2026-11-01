# Requirements Document

## Introduction
Make capability-scoped code compression repeatable: less maintained implementation
for the same declared software behavior. Two positive manual trials establish
examples of the mechanism, not automatic discovery efficacy.

## Boundary Context
- In scope: supplied capability pairs, complete replacement accounting, bound
  validation evidence and reports usable by maintainers and coding agents.
- Out of scope: hardware/performance measurement, feature deletion, application
  redesign, guaranteed equivalence, autonomous repository-wide rewriting, Java,
  a general policy engine, generic transaction/rewrite infrastructure, MCP/plugins,
  solver integration and a separate design-pattern adviser.
- Adjacent expectations: target projects supply their behavioral/type checks;
  the skill selects capabilities and proposes candidates. Existing specialist
  evidence may be recorded, but this slice does not implement solver adapters.
- Delivery constraint: the proposed core is native compiled and local, usable
  without a commercial analyzer or hosted analysis account. Python and TypeScript
  are target languages, with separate evidence required for each.

## Requirements

### Requirement 1: Complete capability scope
**Objective:** As a maintainer, I want a fixed capability boundary so that moving
or fragmenting code cannot manufacture a reduction.

#### Acceptance Criteria
1. When a comparison is prepared, Lesscode shall record the entry points, owned
   implementation, wider accounting boundary, exclusions and both source identities.
2. When a candidate introduces helpers, wrappers, tables, configuration or changed
   callers, Lesscode shall include their maintained implementation in replacement
   accounting, including additions outside the original files.
3. If the compared scopes omit declared owned implementation or narrow the baseline
   boundary, Lesscode shall mark the comparison incomplete and identify the omission.
4. When structural analysis cannot resolve a relationship, Lesscode shall report
   the unresolved relationship without classifying its target as dead code.

### Requirement 2: Comparable measurements
**Objective:** As a reviewer, I want comparable whole-replacement measurements
and structural context rather than an isolated function score.

#### Acceptance Criteria
1. When reporting a pair, Lesscode shall separate production implementation,
   tests, fixtures, research support, configuration and dependency changes.
2. When reporting supported source metrics, Lesscode shall show before/after
   physical lines, logical-statement counts and aggregate decisions with metric
   definitions, versions and scope identities.
3. If a metric is unavailable or its definitions differ between versions,
   Lesscode shall report that condition instead of zero or a comparable delta.
4. When reporting a replacement, Lesscode shall include function counts and
   supported internal call-layer measurements, marking unresolved portions.
5. If a candidate only splits or relocates functions without reducing counted
   implementation, Lesscode shall not classify that change as code compression.
6. The Lesscode comparison shall not use bytes, compressed size or runtime
   performance as compression-benefit criteria.
7. When a trial declares a compression target, Lesscode shall record its scope,
   metric definitions, reduction threshold and permitted structural tradeoffs
   before comparison and evaluate the candidate against that unchanged target.
8. If no compression target is declared, Lesscode shall report metric deltas
   without issuing a successful compression classification.

### Requirement 3: Preserved behavioral boundary
**Objective:** As a maintainer, I want code reductions checked against unchanged
software contracts rather than weakened validation.

#### Acceptance Criteria
1. When a trial starts, Lesscode shall record the check inventory and its identity
   separately from the candidate's editable source.
2. When accepting supplied check results, Lesscode shall verify their association
   with the declared source versions and check inventory.
3. If a required check fails, is missing or changes during a comparison, Lesscode
   shall prevent a successful compression result and state the reason.
4. When reporting validation, Lesscode shall distinguish observed behavior failures
   from incomplete evidence and state the tested domain and unverified behavior.
5. When the declared boundary includes public APIs, error behavior, types, safeguards
   or interaction contracts, Lesscode shall require their specified checks on both versions.
6. The Lesscode report shall not describe finite test success as universal equivalence.

### Requirement 4: Usable empirical workflow
**Objective:** As a coding-agent user, I want reusable evidence to drive subsequent
compression proposals and prevent recurrence.

#### Acceptance Criteria
1. When evaluating a proposal, Lesscode shall retain its replacement hypothesis,
   scope, check outcomes, metric deltas and acceptance or rejection explanation.
2. When producing a report, Lesscode shall provide machine-readable results and
   a human-readable summary including failed and incomplete trials.
3. When the same frozen inputs and metric definitions are compared, Lesscode shall
   produce the same measurement and decision results apart from execution metadata.
4. While analyzing a source specimen, Lesscode shall leave its files unchanged.
5. Where a repository capability inventory is supplied, Lesscode shall associate
   it with its source snapshot and identify proposed dependency changes for review.
6. When a comparison has no valid behavioral evidence, Lesscode shall allow
   measurement reporting while withholding successful compression classification.

### Requirement 5: Self-application maturity gate
**Objective:** As a maintainer, I want Lesscode to exercise its own workflow on
its code after it works, without forcing unsafe reductions or blocking bootstrap.

#### Acceptance Criteria
1. When the first working comparison workflow has passed its external fixtures,
   the Lesscode maturity review shall require a recorded self-application trial
   on a complete Lesscode capability before declaring that workflow mature.
2. When preparing the self-trial, the review shall record the selected capability,
   selection rationale, baseline revision, evaluator revision, frozen checks,
   compression target and supported measurement coverage.
3. While evaluating its own code, Lesscode shall compare isolated source versions
   using a frozen evaluator and check inventory outside the candidate workspace.
4. If the candidate changes measurement or decision logic, the maturity review
   shall require independent expected-result fixtures, including a known adverse
   case, rather than accepting the candidate's own verdict as evidence of correctness.
5. When considering adoption of the self-trial candidate, the maturity review shall
   require the unchanged relevant behavior checks and the existing project
   regression suite to pass, with whole-replacement accounting and structural
   tradeoffs reported separately from test and research code.
6. If no safe beneficial replacement is demonstrated, the maturity review shall
   retain the baseline and record a rejected or inconclusive compression result
   without requiring code deletion or weakening checks to meet a reduction quota.
7. When reporting maturity, the review shall distinguish completion of the
   self-application workflow from demonstrated self-compression; missing required
   evidence shall keep the maturity gate open without blocking implementation work.
8. If some Lesscode implementation languages are unsupported, the maturity review
   shall state those exclusions and limit its maturity claim to the exercised
   capability and supported workflow rather than claim whole-project coverage.
