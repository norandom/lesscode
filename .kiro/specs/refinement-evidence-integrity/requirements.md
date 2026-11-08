# Requirements Document

## Introduction

Correct the evidence and candidate lifecycle of the existing compression
refinement loop. The maintainer must be able to trace each reported outcome to
the exact evaluated files, completed checks and outstanding obligations, then
recover through a bounded next attempt without losing failures.

These are corrective requirements derived from
[R39](../../../docs/research/2026-11-08/06-refinement-correctness.md), pending owner
review. The [adjusted concept](concept.md) explains the broader verification
approach. No implementation or adoption is approved by this document.

## Boundary Context

- In scope: candidate identity and retention; oracle-result interpretation;
  feedback delivery; obligation transitions; validity/compression reporting;
  regression controls and revalidation of affected claims.
- Out of scope: new compression algorithms, a universal equivalence proof,
  automatic adoption, new language support, new dependency installation and
  implementing the deferred Slice B integrations.
- Adjacent expectations: the evaluator supplies separately identified behavioral
  observations and measurements for the frozen trial. Missing required evidence
  is not a pass. Optional checks may be absent only when the declared protocol
  makes them optional, with the limitation retained.

## Requirements

### Requirement 1: Evaluated and delivered candidate identity

**Objective:** As a maintainer, I want the reported candidate and its files to
agree, so that a passing result cannot describe a different implementation.

#### Acceptance Criteria
1. When a candidate is evaluated, the refinement workflow shall bind the result to the candidate content, frozen original, evaluation protocol and relevant dependency identities.
2. When a candidate is selected as best, the refinement workflow shall retain its exact content and associated evidence independently of later attempts.
3. When a run finishes successfully or exhausts its budget, the refinement workflow shall make the selected candidate available at an explicitly reported location whose content matches the selected evidence.
4. If the selected candidate cannot be recovered or its identity does not match the evidence, the refinement workflow shall report an infrastructure or stale-evidence outcome and withhold validity.
5. If source, checks, dependencies or the observation boundary change, the refinement workflow shall mark affected prior evidence stale before reusing it.

### Requirement 2: Complete and consistent oracle outcomes

**Objective:** As a maintainer, I want incomplete or contradictory executions to
remain visible, so that a success marker cannot override a failed check.

#### Acceptance Criteria
1. If an oracle execution times out or terminates with an infrastructure failure, the refinement workflow shall report that outcome even when an earlier output artifact claims success.
2. When an oracle outcome is admitted, the refinement workflow shall validate its required fields, types, observation counts and consistency against the declared oracle protocol.
3. If required results are missing, malformed, contradictory or from another execution, the refinement workflow shall withhold a passing behavioral gate and retain the reason.
4. If a declared required check was not run, the refinement workflow shall report it as not run and shall withhold full behavioral acceptance.
5. When an oracle produces valid partial results, the refinement workflow shall preserve them as diagnostic evidence without promoting them to a complete pass.

### Requirement 3: Feedback delivery and unresolved obligations

**Objective:** As a researcher, I want subsequent attempts to use discovered
failures, so that refinement can repair them rather than repeat blind samples.

#### Acceptance Criteria
1. When a next attempt is made after a failed or incomplete evaluation, the refinement workflow shall include the applicable permitted diagnostics from its identified predecessor in the submitted prompt and retain evidence of that submission.
2. When a prompt is ready for submission, the refinement workflow shall enforce the declared feedback boundary, including held-out-data exclusions, on the actual content before submitting it.
3. When a counterexample, failed check or missing observation is discovered, the refinement workflow shall retain an individually identifiable unresolved obligation with its evidence and candidate applicability.
4. If the same failure remains observable, the refinement workflow shall keep its obligation unresolved and shall not count its discovery as discharge or progress toward resolution.
5. When an obligation is discharged for a candidate, the refinement workflow shall retain a passing recheck on that candidate or an explicit reviewed disposition explaining why the obligation does not apply.
6. When the run stops or selects a different candidate, the refinement workflow shall report all applicable unresolved obligations and preserve their history across attempts.

### Requirement 4: Independent outcome decisions

**Objective:** As a maintainer, I want search scores, validity, compression and
adoption distinguished, so that convenience settings cannot manufacture success.

#### Acceptance Criteria
1. If a search score meets a configurable stopping threshold while required behavioral evidence is absent or failing, the refinement workflow shall stop only as allowed by the search policy and shall not report the candidate as valid or fully accepted.
2. When validity is reported, the refinement workflow shall require complete, current passing evidence for the declared behavioral boundary and no unresolved blocking obligation for the selected candidate.
3. If required compression targets or measurements are absent, the refinement workflow shall report compression as unmeasured and shall not report that its targets were met.
4. When compression targets are reported as met, the refinement workflow shall provide the frozen targets and matching whole-replacement measurements, including added support and changed callers.
5. When a run reports its result, the refinement workflow shall state search termination, candidate validity, measured compression, observation limits and actual adoption separately.
6. If a candidate fails behavior or lacks required evidence, the refinement workflow shall preserve the original as the fallback regardless of any measured size reduction.

### Requirement 5: Verification of the decision machinery

**Objective:** As a maintainer, I want checks that can falsify the verifier's own
claims, so that a passing reference comparison cannot hide a shared defect.

#### Acceptance Criteria
1. When a correction is qualified, the verification workflow shall run the R39 regressions against the actual controller and oracle adapter and retain both the failing baseline and corrected results.
2. When candidate selection is tested, the verification workflow shall exercise distinct candidate contents, later worse candidates, ties and exhaustion, and verify the selected files as well as the reported selection.
3. When refinement recovery is tested, the verification workflow shall require the repair response to depend on receiving the preceding diagnostic and shall reject a claimed recovery that only replays predetermined verdicts.
4. When oracle acceptance is tested, the verification workflow shall include completed success, behavioral failure, failed process after a success artifact, timeout after a success artifact, inconsistent counts and missing required observations.
5. When evidence or diagnostic availability is reduced without new passing checks, the verification workflow shall verify that the resulting validity decision cannot improve solely because evidence was removed.
6. When an adverse control is used to support evaluator adequacy, the verification workflow shall confirm the intended fault was exercised and rejected for the expected reason, distinguishing unrelated infrastructure failures, equivalent mutations and untested cases.

### Requirement 6: Bounded recovery and claim revalidation

**Objective:** As a researcher, I want a recoverable path from a counterexample
to a new trial, so that failure strengthens the experiment without rewriting history.

#### Acceptance Criteria
1. When a realizable behavior mismatch outside the current observation coverage is found, the verification workflow shall retain it as a counterexample, identify the missing observation and invalidate affected acceptance claims.
2. When the evaluation boundary is expanded to cover a counterexample, the verification workflow shall create a separately identified protocol revision and rerun the original and candidate under that same revision.
3. When a corrected loop is proposed for compression use, the verification workflow shall exercise one complete real-code responsibility end to end with declared exception, state, output and effect-ordering observations where applicable, rather than substitute a replayed verdict for execution.
4. When that trial begins, the verification workflow shall freeze its code-removal hypothesis, whole-replacement usefulness target, evidence inventory, exclusions and attempt budget before generating a replacement.
5. If the attempt or no-progress budget is exhausted, the refinement workflow shall preserve the best recoverable candidate, original and unresolved obligations, and report exhaustion without claiming equivalence or useful compression.
6. When historical evidence is reinterpreted, the verification workflow shall append a dated correction identifying the affected claim while preserving the original artifacts, negative results and phase approvals.
