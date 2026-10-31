# Requirements review: agent-quality-rules

Status: draft quality gate PASS. This is not human requirements approval,
design approval or implementation validation. All phase approvals remain false.

## Scope reviewed

Nine requirement areas contain 65 acceptance criteria. Inputs include the
discovery brief, core steering, baseline contracts and outcomes, the worked
self-evaluation, and the user's requirements for convolution prevention,
idiomatic code, role-revealing names and meaningful measurements.

Read-only brownfield exploration confirmed that baseline scalar thresholds do
not already implement function-level policy or debt ratchets. Structured
complexity and duplication evidence must retain their definitions. LLOC and MI
remain auxiliary measurements; semantic review and maintenance effort cannot
be inferred from lower counters.

## Gate results

| Check | Result |
| --- | --- |
| Numeric requirement areas | PASS: 1 through 9 |
| Acceptance numbering | PASS: consecutive local numbering, 65 criteria |
| EARS triggers and named subject | PASS: every criterion uses When, If, Where or The with Agent Rules Evaluator shall |
| Testable outcomes | PASS: policy disclosure, evidence treatment, findings, updates and failure outcomes are observable |
| Implementation choices | PASS: no prescribed storage, parser, transport, schema layout or architecture; AGENTS.md is a requested user-facing artifact |
| Boundaries | PASS: measurement remains upstream; rules own preventive policy; skills and rewrites remain downstream |
| Unknown evidence | PASS: no unavailable metric or unverified required review becomes a pass |
| Guidance safety | PASS: preview, scoped changes, conflict handling, preservation and idempotency are explicit |
| Claims | PASS: no universal simplicity score or inferred maintenance savings |

One local repair pass clarified newly introduced violations in existing entities,
uncertain entity matches, required review trust conditions and repeatable
findings. No unresolved scope contradiction was found. Human review may still
change the proposed scope before design.

## Concrete cases for human review

| Declared conditions | Required outcome | Criteria |
| --- | --- | --- |
| Change-ratchet mode, limit 10, reference CCN 56, candidate 56 | Inherited debt, no new regression solely from unchanged debt | 3.1, 3.2 |
| Same reference, candidate 54 | Improvement with residual debt | 3.4 |
| Same reference, candidate 57, allowed increase 0 | New or worsened violation | 3.3 |
| Reference CCN 4, candidate 11, applicable limit 10 | Newly introduced limit violation in an existing entity | 3.3 |
| Absolute mode, limit 10, unchanged CCN 56 | Absolute violation remains blocking when so declared | 3.5 |
| Claimed grandfathering depends on an uncertain split/rename match | No match-based blocking-rule pass until reference treatment is established | 3.7 |
| Required duplication coverage omits the nonempty short file from the self-example | Incomplete evidence; no quiet exclusion or zero | 4.1, 4.2, 4.6 |
| Lower CCN, required naming/responsibility review absent | Overall policy acceptance withheld | 5.3, 5.4, 9.2 |
| Required independent review has only an unsupported independence claim | Required trust condition remains unverified | 5.5, 5.8 |
| Managed AGENTS.md text changed after preview | Conflict reported; existing file unchanged | 6.4, 6.5 |
| Rule exception offered for failed behavior or missing mandatory evidence | No waiver-based satisfaction of those requirements | 7.3 |
| New policy requested after a failed assessment | New assessment identity; prior result retained | 7.5 |

These are configured examples, not globally prescribed numeric limits.

## Dependency and approval notes

The short-file duplication coverage limitation belongs to efficiency-baseline.
This spec must preserve the resulting incomplete state; it cannot manufacture a
whole-package pass or redefine an upstream metric. A design claiming complete
whole-package duplication would first need the upstream contract addressed.

The next phase is design after human requirements approval. There are no
generated design or implementation tasks in this phase.
