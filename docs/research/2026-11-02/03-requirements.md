# R03: continuity anchored and requirements generated

Report/discussion date: 2026-11-02, Europe/Berlin. Status: requirements generated
for owner review. No new extraction, modeling or compression experiment.
Predecessor: [R02](02-codeql-direction.md). Hypothesis: H7 in the [log](../README.md).

## Decision and result

The owner requested a persistent research-log anchor and the next spec step.
Root [AGENTS.md](../../../AGENTS.md) now requires consulting the log, recording
failed attempts and updating reciprocal spec links as work proceeds.

[Requirements](../../../.kiro/specs/constraint-guided-compression/requirements.md)
contain 8 areas / 53 criteria. Graph evidence, model fidelity and whole-replacement
compression are separate claims. Repeated passes require budgets and retained
revisions. Staged experiments include controls and held-out evaluation.

A read-only explorer confirmed native measurement interfaces and the limited
call-layer graph. Persisted semantic queries and LLM modeling are new scope.
The generation gate was reviewed inline, as recorded in the
[review](../../../.kiro/specs/constraint-guided-compression/requirements-review.md).

## Next gate

Requirements approval precedes design. Backend selection does not approve every
phase. No efficacy result is implied. Design must resolve the first supported
semantic subset and integration while keeping the prototype bounded.

COBESY check: outcome visible, evidence linked, pending decision explicit;
spec progress is distinguished from experimental success.
