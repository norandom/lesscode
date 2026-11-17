# Requirements: model simplification

Generated 2026-11-17 from the [brief](brief.md) and [current-versus-desired analysis](research.md).
Status: generated for owner review; not approved. No design or production tasks are approved.
Analysis record: [R114](../../../docs/research/2026-11-17/01-model-simplification-measurement-attribution.md).

## Introduction

Developers can already plan and check small refactoring steps with lesscode, but
must find and write the changes themselves. Existing reports show tests, planted
faults and source size. They do not establish whether branch conditions or scattered
state were reduced, and the Python pack counts local assignments as state writes.

This feature lets a configured model propose a bounded simplification, judges the
candidate through the existing step flow, and reports what became simpler over the
whole responsibility. It separates actual code changes from corrected measurement,
stronger tests, model repair and human assistance.

Owner decision, 2026-11-17: simplicity is reported separately from the behavior
verdict. A step can be accepted without becoming simpler. Neither a smaller source
file nor an accepted model reply establishes useful simplification.

## Terms

- **Workflow**: the model-simplification workflow described here, including its
  planning, proposal and reporting behavior.
- **Responsibility**: the declared behavior and its owned implementation, including
  helpers, changed callers and support needed by the replacement.
- **Baseline**: the fixed source revision against which a candidate is compared.
- **Measurement profile**: the versioned definitions, supported constructs and
  counting rules used for branch and state measurements, including pack identity.
- **State target**: an identifiable nonlocal storage location or external effect
  target within the profile's declared support. A local name binding is not by
  itself a nonlocal write; a mutation through a local alias may be.
- **Comparable**: both revisions were measured under the same profile and declared
  responsibility, with sufficient coverage and matching identities for the claim.
- **Useful candidate**: a behavior-accepted candidate that meets the structural
  target declared before generation, without violating its companion controls.
  This is a reported evaluation outcome, not another step-acceptance condition.
- **Trial**: one declared task taken through a first proposal and its allowed repairs.
- **Pin**: a test established on unchanged code and checked against a named fault
  before evaluating the rewrite.

## Boundary Context

- **In scope**: branch/state planning; correction of local assignments counted as
  nonlocal writes; model proposals and bounded repair through the model seam;
  existing step checks; independent simplicity reporting; reproducible attribution;
  bounded real-code evidence.
- **Out of scope**: onboarding levels/plugin delivery; model/provider rankings;
  new transport clients; automatic pack generation per project; prompt optimizers;
  an unrestricted tool-using agent; automatic upstream adoption, commits or pushes;
  universal equivalence proofs; size reduction as the effectiveness objective.
- **Adjacent expectations**: `lesscode-model` owns endpoint configuration, calls,
  waits and repair transport. The step flow owns behavioral verdicts, pins, guards
  and ledgers. Planning and judging remain usable without a model. Existing size
  limits remain safeguards and are not reinterpreted as simplicity rewards.
- **Initial evidence scope**: `x_likes_exporter/exporter.py` for state and
  `x_likes_mcp/tools.py` for branching in `x_likes_scraper`. These are case studies,
  not evidence of automatic candidate selection or efficacy across all languages.

## Requirements

### Requirement 1: Declare and preserve the responsibility
**Objective:** As a developer, I want a bounded comparison, so that the proposal cannot hide changed behavior or move work out of the measurement.

#### Acceptance Criteria
1. When a simplification trial is requested, the workflow shall identify the baseline revision, declared responsibility, source files, test inputs, step declaration and allowed attempt budget before requesting a proposal.
2. If a required trial input is missing or inconsistent with the identified baseline, then the workflow shall report the input problem and make no model call.
3. While a trial is running, the workflow shall preserve the original source, tests and declaration and evaluate candidates in isolated copies.
4. When a candidate introduces helpers, support code or changed callers, the workflow shall include their contribution in the responsibility comparison.
5. If a candidate changes files outside its authorized scope, then the workflow shall reject the candidate and name those files.
6. If the workflow cannot account for implementation needed by the replacement within the declared responsibility, then the workflow shall mark the affected simplicity comparison as not measurable and identify the missing boundary.
7. The workflow shall not commit, push, merge or apply a candidate to the user's working source tree.

### Requirement 2: Measure branches and state with explicit limits
**Objective:** As a developer, I want measurements of branch conditions and scattered state, so that I can distinguish structural changes from smaller files or corrected counting.

#### Acceptance Criteria
1. When source is measured, the workflow shall identify the measurement profile and its supported and unsupported branch and state constructs.
2. When branch conditions are measured, the workflow shall report their count and locations over the whole responsibility, with per-function attribution and sites outside functions identified.
3. When branch conditions are combined into compound expressions, the workflow shall report the retained predicate and short-circuit decision work within the profile's support.
4. When Python assignments are classified, the workflow shall distinguish local rebinding from identifiable nonlocal mutation and unresolved mutation, including writes through aliases and declared global or nonlocal names.
5. When state dispersion is measured, the workflow shall report the identified state targets, their write sites and their distribution across functions and files.
6. If a call's effect or a write's target cannot be resolved within the profile's support, then the workflow shall report that effect as unresolved rather than treating it as a known absence of state mutation.
7. If state-target or function identities cannot be matched across revisions, then the workflow shall mark comparisons dependent on that matching as not measurable and state the reason.
8. When the same source is measured again with the same profile and scope, the workflow shall return the same measurements.
9. When only whitespace, comments or local identifier spelling changes without changing binding or behavior, the workflow shall report no branch or state simplicity improvement.

### Requirement 3: Plan by branches or state without hiding protection gaps
**Objective:** As a developer, I want to find responsibilities with concentrated branches or scattered state, so that I can choose a useful small step.

#### Acceptance Criteria
1. When branch-based planning is requested, the workflow shall rank the measured candidates by the declared branch-ranking rule and show the values that determine their order.
2. When state-based planning is requested, the workflow shall rank comparable candidates by the declared state-ranking rule and show their state-target and writer-distribution evidence.
3. When a plan is reported, the workflow shall retain the available noticed and unnoticed fault counts, outside-frame writes and unprotected-function information beside the ranking.
4. If a candidate lacks the measurements required by the selected ranking, then the workflow shall identify it as unranked with reasons rather than assigning it a zero score.
5. When ranking values are equal, the workflow shall produce a repeatable order and state the tie-breaking rule.
6. The workflow shall describe ranking as a selection signal rather than evidence that a candidate rewrite exists or preserves behavior.

### Requirement 4: Request bounded proposals through the model seam
**Objective:** As a developer, I want a model to propose a small change against the declared problem, so that I can inspect an actual candidate and its evidence.

#### Acceptance Criteria
1. When a proposal is requested, the workflow shall use the configured model through the existing model seam.
2. When constructing a proposal request, the workflow shall supply the declared responsibility, available implementation context, public behavior constraints, branch/state findings and requested simplification objective.
3. When a model returns a proposal, the workflow shall retain the raw answer and identify the exact candidate derived from it before running behavioral checks.
4. If a proposal cannot be interpreted as a candidate within the declared scope, then the workflow shall record it as a rejected proposal with reasons rather than a behavior-accepted step.
5. If a candidate is rejected and the trial permits another repair, then the workflow shall provide the rejected answer and the actual rejection reasons in the next request.
6. When a trial is run without an explicit repair limit, the workflow shall permit at most two repair calls after the first proposal.
7. If the attempt budget is exhausted, the model seam reports a terminal call failure or the model refuses the request, then the workflow shall stop the trial and report the reason together with the completed attempts.
8. When a candidate is behavior-accepted without a simplicity improvement, the workflow shall end the trial with that outcome rather than inventing a behavioral rejection to obtain another repair.
9. The workflow shall preserve the model seam's configured waiting, failure-reporting and secret-handling behavior.

### Requirement 5: Preserve the meaning of behavioral acceptance
**Objective:** As a developer, I want the established step checks to judge the change, so that a simplicity score cannot excuse a regression or overstate protection.

#### Acceptance Criteria
1. When a candidate is ready to be judged, the workflow shall obtain its behavioral result through the existing step flow against the declared baseline.
2. When baseline and candidate are judged in a trial, the workflow shall use the same established test suite and admitted measurement/mutation pack for both revisions.
3. If a candidate changes the frozen tests, pins, check settings or declaration, then the workflow shall reject that candidate as outside the trial's authorized change.
4. When pins are added for a trial, the workflow shall record their admission on unchanged code before judging the candidate rewrite.
5. When the behavioral result is reported, the workflow shall include the available test and fault outcomes, unprotected or out-of-reach functions, and known coverage gaps.
6. If a check is incomplete, invalid or unable to judge the candidate, then the workflow shall preserve that outcome and its reasons rather than substitute a passing result from simplicity measurements.
7. The workflow shall not describe behavioral acceptance as a proof of equivalence or of protection for behavior the checks did not exercise.

### Requirement 6: Report simplicity independently and comparably
**Objective:** As a developer, I want separate answers for behavior and simplicity, so that I can see what the change achieved and what remains uncertain.

#### Acceptance Criteria
1. When a trial produces a candidate, the workflow shall report its behavioral result separately from its simplicity result.
2. When a comparable measurement is reported, the workflow shall show the baseline value, candidate value and signed change.
3. When a percentage reduction is reported, the workflow shall calculate it relative to a positive baseline value or report it as unavailable when the baseline value is zero.
4. If measurement profiles, responsibility boundaries or coverage prevent a valid comparison, then the workflow shall report the affected measure as not measurable with reasons rather than substituting zero.
5. When simplicity is assessed, the workflow shall distinguish improvement, no change, regression, mixed trade-offs and an unmeasurable result for the applicable measures.
6. When one measure improves while another declared companion control worsens, the workflow shall report the trade-off instead of combining them into an unqualified improvement claim.
7. If apparent improvement comes only from moving implementation outside the accounting boundary, changing measurement rules or treating unresolved effects as absent, then the workflow shall withhold the affected improvement claim.
8. If a lower condition or writer count results from expression folding, function merging or indirect writes, then the workflow shall not claim simplicity improvement from that count alone without accounting for the retained decision and state work.
9. The workflow shall not count removed lines, tokens or syntax nodes alone as branch or state simplicity improvement.
10. The workflow shall preserve the existing step verdict, exit-code meaning, guard decision and ledger acceptance semantics regardless of the independent simplicity result.

### Requirement 7: Retain evidence of who and what produced the result
**Objective:** As a reviewer, I want a traceable record of each attempt, so that I can attribute an outcome to the actual candidate and intervention.

#### Acceptance Criteria
1. When a trial outcome is saved, the workflow shall bind it to the baseline and candidate contents, responsibility, test/pin identities, measurement profile, pack, tool revision and declared execution settings.
2. When a model attempt is saved, the workflow shall link the request context, raw reply, candidate contents, sent model settings and check results for that attempt.
3. When a recorded candidate is changed before re-evaluation, the workflow shall identify the earlier verdict as inapplicable to the new candidate contents.
4. When a trial includes supplied human selection, rewrite hints, pins or candidate edits, the workflow shall record those interventions separately from the model's unedited output.
5. If intervention provenance is missing, then the workflow shall label it unknown rather than count the result as unassisted model success.
6. When a trial ends without a usable candidate, the workflow shall retain its completed attempts and failure reasons in the trial record.
7. When a candidate is checked only in an isolated copy, the workflow shall report adoption as not established rather than implying the original project has adopted it.

### Requirement 8: Attribute measurable gains without mixing causes
**Objective:** As the owner, I want gains attributed to their actual source, so that an improved instrument or extra assistance is not presented as model effectiveness.

#### Acceptance Criteria
1. When a measurement correction is evaluated, the evaluation workflow shall report its effect on unchanged source separately from the candidate's effect under one fixed measurement profile.
2. When additional pins improve fault detection on baseline source, the evaluation workflow shall report that as increased test protection separately from structural change in the candidate.
3. When repair outcomes are summarized, the evaluation workflow shall report first-answer results and recovery within the allowed repairs as separate counts.
4. If an evaluation claims that repair feedback improves outcomes beyond another attempt alone, then the evaluation workflow shall provide a comparison with fresh attempts without failure feedback under the same tasks, model settings and attempt budget.
5. If an evaluation claims improved candidate selection from a ranking, then the evaluation workflow shall compare rankings over the same eligible population, selection limits, proposal method and attempt budgets and report the selected sets and outcomes.
6. When outcomes are attributed to the model, the evaluation workflow shall separate declared human-assisted and unknown-provenance trials from unassisted trials.

### Requirement 9: Measure useful yield separately from answer acceptance
**Objective:** As the owner, I want to know how often trials deliver a useful simplification, so that passing replies do not inflate effectiveness claims.

#### Acceptance Criteria
1. When useful-candidate yield is to be evaluated, the evaluation workflow shall record the target measures, absolute and relative usefulness thresholds and companion controls before generating candidates.
2. When a trial is classified as useful, the evaluation workflow shall require behavioral acceptance, comparable measurements and satisfaction of that trial's predeclared structural target and companion controls.
3. When a set of trials is summarized, the evaluation workflow shall report scheduled, attempted, endpoint-failed, candidate-producing, behavior-accepted, measurable, useful and adopted counts with the definitions of each category.
4. When yield rates are reported, the evaluation workflow shall name each denominator and include useful per scheduled trial, useful per attempted trial and behavior-accepted per attempted trial.
5. When a responsibility is attempted repeatedly, the evaluation workflow shall distinguish the number of trials from the number of unique responsibilities and retain per-responsibility outcomes.
6. When the existing model-measure indicator is included, the evaluation workflow shall label its acceptance rate with the task set and check it measures rather than present it as a simplicity rate.
7. When resource use is reported, the evaluation workflow shall report available call and token usage separately from structural improvement.

### Requirement 10: Preserve model independence and measurement compatibility
**Objective:** As an adopter, I want the deterministic checks to remain dependable after this feature is added, so that neither a model nor a new metric silently changes prior evidence.

#### Acceptance Criteria
1. The planning, measurement and step-checking functions shall remain usable without the model seam installed or any calls to a model.
2. The workflow shall derive language-specific classification from admitted language packs rather than rules naming a specimen project or a model.
3. If a requested language construct is outside an admitted profile's supported measurement boundary, then the workflow shall identify the unsupported construct and withhold the affected claim.
4. When a pack correction changes eligible mutation sites or operators, the evaluation workflow shall identify the lost and gained mutation coverage separately from changes in candidate fault outcomes.
5. If prior evidence uses a different profile or pack that makes a requested comparison invalid, then the workflow shall identify the evidence as incompatible for that comparison and require a new compatible evaluation rather than silently treating it as improvement.
6. The workflow shall preserve historical measurements and acceptance decisions with their original profile identities unchanged.

### Requirement 11: Qualify the measures and demonstrate real steps
**Objective:** As the owner, I want bounded evidence before production implementation, so that we build on demonstrated simplification rather than a plausible score.

#### Acceptance Criteria
1. When a prototype run is requested, the evaluation workflow shall require frozen source and dependency identities, responsibility boundaries, measurement definitions, behavioral checks, adverse controls, useful-reduction targets, trial/call/run budgets and stop conditions before execution.
2. When qualifying the measurement profile, the evaluation workflow shall exercise controls for unchanged source, formatting, local rebinding, compound-condition rewrites, helper/file relocation, function merging, indirect setters and unresolved state effects.
3. When qualifying behavioral protection, the evaluation workflow shall exercise known-bad output and state-effect changes and record whether the declared checks detect them.
4. When eligibility for dependent production tasks is assessed, the evaluation workflow shall require recorded attempts of both the exporter state case and the tools branching case with their declared targets and all attempted outcomes.
5. When a prototype outcome is assessed, the evaluation workflow shall distinguish measurement fidelity, behavioral evidence, useful simplification and adoption and record a proceed, revise, stop or not-run decision against the frozen gate.
6. If the mandatory behavioral boundary cannot be checked or a mandatory usefulness target is not met, then the evaluation workflow shall withhold a proceed decision for the affected production scope and preserve the contrary evidence.
7. When experimental results or material hypothesis revisions are recorded, the evaluation workflow shall add a dated research entry and reciprocal links to the research index and this spec without replacing earlier interpretations.
8. The evaluation workflow shall not treat passing synthetic controls alone or successful model transport alone as qualification of real-code simplification.
9. When a proceed decision is proposed, the evaluation workflow shall require at least one representative real responsibility to meet its frozen useful-reduction target and behavioral controls over the complete replacement, with any unsupported production scope explicitly excluded.

## Adjacent contract changes to resolve in design

- Gradual-refactoring-steps Requirement 2 currently ranks by unnoticed faults.
  Requirements 3.1-3.5 add requested branch/state ranking with visible protection
  information; the existing ranking is not declared removed by this spec.
- Its Requirement 5 and the guard/ledger retain their acceptance meaning. Requirements
  6 and 9 add independent simplicity and useful-yield outcomes.
- Correcting pack classification affects mutation eligibility and frame labels as
  well as measurement; Requirements 5.2 and 10.4-10.6 require compatible comparisons.
- Model-seam repair remains bounded and stops on acceptance. Requirement 4.8 means
  this feature does not introduce a further search loop after an accepted answer.
- Onboarding retains its broader protection-policy and user-journey decisions.
  This feature reports gaps without silently changing those policies.

## Deferred to design and the prototype gate

Exact branch/predicate counting rules, state identity and dispersion formulas,
ranking/default/tie rules, result formats, integration points and supported semantic
subsets belong in design. The analysis's proposed formulas are not approved by this
requirements document. Concrete usefulness thresholds and execution budgets must be
frozen before trials; no numerical baseline or improvement is claimed here.

The prototype must follow the repository's execution and artifact-storage rules,
including resolution of the target Python test-runtime route before running it.
Production implementation still requires a qualifying prototype and phase approvals.
