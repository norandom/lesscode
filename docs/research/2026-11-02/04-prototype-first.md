# R04: real-code feasibility precedes production task planning

Date: 2026-11-02, Europe/Berlin. Status: process decision; no experiment executed.
Predecessor: [R03](03-requirements.md). Spec:
[constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision

The owner requires a fail-early, prototype-first Kiro workflow. A design proposal
must demonstrate that its mechanism can handle real code and produce useful
compression before agents work through production tasks.

The [project policy](../../../.kiro/steering/prototype-first.md) and
[spec gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
make this explicit. A bounded research spike may precede production tasks;
otherwise a gate could block the very evidence needed to pass it.

## What changed

Requirements 7.9-7.11 now cover the feasibility gate, blocked downstream tasks and
revalidation. Requirements remain unapproved. The gate is `not-run`: installed
CodeQL and older manually selected successes do not demonstrate the new mechanism.

Before running, design must identify the real scope, removed implementation,
expected net effect, useful-reduction target, checks, adverse controls and budget.
After running, it must record actual results and choose proceed, revise or stop.
Only proceed enables final design approval and production tasks. Fast-track
approval flags do not substitute for empirical evidence.

## Falsification and next action

Missing critical relationships, wrong source models, changed behavior or marginal
net reduction reject or defer the mechanism. Preserve those findings rather than
build more infrastructure around them. Next: a provisional design and concrete
prototype proposal after requirements approval; no efficacy claim or new chart.

COBESY check: the future implementer can identify the evidence required to proceed,
and report failure without presenting task completion as research success.
