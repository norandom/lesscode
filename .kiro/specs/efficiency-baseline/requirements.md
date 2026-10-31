# Requirements Document

## Introduction
Developers maintaining AI-generated codebases need reproducible evidence that simplification improves maintainability without changing required behavior. The efficiency-baseline feature establishes a local, open-source evaluation workflow, starting with Python projects, to compare supplied baseline and candidate snapshots. It provides the evidence later used to assess plain Codex, a simplification skill and deterministic tooling.

## Boundary Context
- **In scope**: local measurement, fixed scope and input identity, declared behavior checks, evaluator-integrity checks, reproducible artifacts and controlled experiment reporting.
- **Out of scope**: generating candidate code, refactoring or rollback, skill implementation, launching LLM experiments, MCP packaging, automatic AI-origin detection and universal correctness proofs. JS/TS and Java evaluation are later increments.
- **Adjacent expectations**: developers or later workflows supply source snapshots, executable checks, evaluation profiles and trial records. The evaluator executes declared project checks and consumes optional coverage, mutation, API, architecture and runtime evidence; it does not promise automatic inference of complete test or public-API contracts. Agent accounts and costs belong to experiment producers. Exact thresholds and required optional checks are declared before each experiment; no fixed global reduction percentage is required.
- **Corpus scope**: the initial validation corpus includes synthetic duplication/ceremony, licensed real-project examples and shorter but behavior-breaking adversarial examples. These establish harness behavior, not general effectiveness of a future skill.

## Requirements

### Requirement 1: Local, accessible evaluation
**Objective:** As a developer, I want local evaluation without commercial analysis licenses, so that I can reproduce results.

#### Acceptance Criteria
1. When a developer requests evaluation of a Python project, the Baseline Evaluator shall provide a command-line workflow for measuring a baseline and a supplied candidate.
2. The Baseline Evaluator shall provide its required analysis capabilities using open-source tools without requiring a paid analysis account or proprietary license.
3. The Baseline Evaluator shall support evaluation without requiring Codex or another LLM service.
4. The Baseline Evaluator shall support the documented evaluation workflow on Windows, Linux and macOS.
5. When evaluation runs, the Baseline Evaluator shall keep source analysis and result storage local without requiring source upload.

### Requirement 2: Scope and input identity
**Objective:** As an evaluator, I want identifiable inputs and consistent scope, so that comparisons describe the intended changes.

#### Acceptance Criteria
1. When a run starts, the Baseline Evaluator shall record the identity and content fingerprint of each supplied source snapshot, including uncommitted content included in the run.
2. When a run starts, the Baseline Evaluator shall record included paths, exclusions and the classification of product source, tests, generated files and vendor files.
3. When product-source metrics are calculated, the Baseline Evaluator shall exclude tests, generated files, vendor files and evaluation outputs from product-source totals and report test changes separately.
4. If baseline and candidate use different scope rules, metric definitions or analyzer versions, the Baseline Evaluator shall mark affected comparisons as non-comparable and identify the differences.
5. When a candidate adds or removes files within the fixed scope rules, the Baseline Evaluator shall include those changes in the comparison.
6. If an input cannot be read or identified, the Baseline Evaluator shall report the affected input and prevent a successful evaluation result.

### Requirement 3: Structural measurements
**Objective:** As a developer, I want inspectable measurements, so that reductions in unnecessary structure can be assessed.

#### Acceptance Criteria
1. When supported source is evaluated, the Baseline Evaluator shall report function-level cyclomatic complexity, its distribution and the locations of highest-complexity functions.
2. When supported source is evaluated, the Baseline Evaluator shall report detected duplicate locations, duplicate token counts and duplication ratio with its denominator.
3. When source size is measured, the Baseline Evaluator shall report source bytes, line counts with their definitions and compressed source bytes.
4. When a metric is reported, the Baseline Evaluator shall identify its analyzer, version, definition, unit and measured scope.
5. When identical content is evaluated with identical structural measurement configuration and tool versions, the Baseline Evaluator shall produce identical structural metric values.
6. Where nesting or architecture checks are configured, the Baseline Evaluator shall report their findings separately from complexity and duplication.
7. If a tool is missing, source syntax is unsupported or measurement fails, the Baseline Evaluator shall mark the affected metric unavailable with a reason rather than reporting zero.

### Requirement 4: Behavior and test evidence
**Objective:** As a developer, I want independent behavior checks, so that shorter code cannot conceal regressions.

#### Acceptance Criteria
1. When a comparison is evaluated, the Baseline Evaluator shall run the declared baseline and candidate checks and retain their outcomes and diagnostic output.
2. If baseline checks fail or cannot complete, the Baseline Evaluator shall mark the trial ineligible for a behavior-preservation success claim.
3. If a baseline-passing check fails on the candidate, the Baseline Evaluator shall reject the candidate regardless of structural improvements.
4. When independent expected-output, invariant or public-API checks are supplied, the Baseline Evaluator shall evaluate the candidate against them and reject mismatches.
5. If a required behavior check is missing, skipped or cannot complete, the Baseline Evaluator shall report insufficient evidence rather than behavior preservation.
6. Where coverage or mutation evidence is supplied, the Baseline Evaluator shall report comparable before/after values with their scopes and denominators.
7. If coverage or mutation populations differ, the Baseline Evaluator shall disclose the differences and require a recorded explanation before presenting their score change as comparable.
8. When behavior checks pass, the Baseline Evaluator shall describe the result as passing the declared checks and disclose unchecked behavior rather than claim proof of equivalence.

### Requirement 5: Evaluator integrity and decision rules
**Objective:** As an evaluator, I want fixed acceptance rules, so that a rewriting agent cannot improve its result by weakening evaluation.

#### Acceptance Criteria
1. When an experiment begins, the Baseline Evaluator shall record the declared checks, required versus optional evidence, thresholds, scope rules and evaluator fingerprints before evaluating candidates.
2. When a candidate is evaluated, the Baseline Evaluator shall use the same declared evaluator configuration and independent test inventory as its baseline.
3. If evaluator configuration, exclusions or independent tests change during a trial, the Baseline Evaluator shall invalidate the trial and identify the changed evidence.
4. When mandatory evidence is complete and all declared acceptance rules pass, the Baseline Evaluator shall identify the candidate as accepted under that recorded evaluation profile.
5. If a mandatory check is unavailable, the Baseline Evaluator shall mark the decision incomplete rather than accepted.
6. When outcomes are presented, the Baseline Evaluator shall prioritize behavior and invariants, test strength, public API preservation, complexity and duplication, size, then cost without combining them into a compensating universal score.
7. When integrity results are reported, the Baseline Evaluator shall disclose whether evaluation protections rely on local change detection or an externally controlled environment.

### Requirement 6: Comparison and reporting
**Objective:** As a developer, I want auditable before/after reports, so that conclusions can be checked independently.

#### Acceptance Criteria
1. When a comparison completes, the Baseline Evaluator shall provide a human-readable summary and a machine-readable report with an identified format version.
2. When comparable numeric metrics exist, the Baseline Evaluator shall report before and after values and the absolute delta calculated as after minus before.
3. When the baseline value is nonzero, the Baseline Evaluator shall report percentage delta as 100 multiplied by the absolute delta divided by the baseline value.
4. If a baseline value is zero or either metric is unavailable, the Baseline Evaluator shall mark the percentage delta unavailable with its reason.
5. When a report is produced, the Baseline Evaluator shall retain raw tool outputs, exit outcomes, commands, source identities, tool versions, configuration, timestamps and environment information sufficient to repeat the declared evaluation.
6. When different analyzers measure a related property, the Baseline Evaluator shall keep their results separately identified.
7. When a run terminates with errors or partial results, the Baseline Evaluator shall expose that outcome to both command-line users and automated callers and retain available diagnostics.

### Requirement 7: Controlled experiment evidence
**Objective:** As a researcher, I want comparable trials, so that the benefit of the skill and later tooling can be distinguished.

#### Acceptance Criteria
1. When an experiment is recorded, the Baseline Evaluator shall identify its corpus, task, intervention arms, planned repetitions, budgets and declared comparison conditions.
2. When trial records are compared, the Baseline Evaluator shall record initial source identity, model and settings, prompt identity, tool permissions, execution order and seeds where supplied.
3. If paired trials differ in declared controlled conditions beyond the intended intervention, the Baseline Evaluator shall flag the comparison as confounded and identify those differences.
4. When experiment results are summarized, the Baseline Evaluator shall include failed attempts and exhausted budgets alongside successful trials.
5. When correctness rate is reported, the Baseline Evaluator shall disclose the eligible-attempt denominator and separately count accepted, rejected, incomplete and baseline-ineligible trials.
6. When structural improvement is summarized, the Baseline Evaluator shall distinguish improvements for accepted candidates from outcomes across all attempts.
7. When repeated comparable trials are available, the Baseline Evaluator shall report variation across repetitions and identify the method and sample count used for any uncertainty estimate.
8. When corpus entries are recorded, the Baseline Evaluator shall retain their source, revision or content identity, license and known generation provenance without inferring AI authorship from style.

### Requirement 8: Operational cost and execution safety
**Objective:** As a developer, I want bounded evaluation and honest cost accounting, so that measurement remains usable.

#### Acceptance Criteria
1. When evaluation executes, the Baseline Evaluator shall record elapsed evaluation time separately from supplied agent execution time.
2. Where observed token usage or provider cost is supplied, the Baseline Evaluator shall report it with its source and units.
3. If token usage or cost is not available, the Baseline Evaluator shall label it unavailable rather than infer it from source size.
4. Where runtime benchmark results are supplied, the Baseline Evaluator shall identify the workload and environment separately from structural and agent-cost results.
5. When evaluation commands execute, the Baseline Evaluator shall preserve the developer's original source and uncommitted changes.
6. When an evaluation workload exceeds its configured time budget, the Baseline Evaluator shall stop that workload and report the timeout and incomplete evidence.
7. When evaluation is requested, the Baseline Evaluator shall expose the project commands to be executed before running them.
8. When supplied candidate changes fail evaluation, the Baseline Evaluator shall retain the rejection evidence without rewriting or reverting the developer's original source.
