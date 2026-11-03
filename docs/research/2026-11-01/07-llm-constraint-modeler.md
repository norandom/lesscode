# D01-07: the LLM becomes the modeler, with constraints driving revision

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: follow-up specification initialized, not implemented or phase-approved.
Hypothesis H7 in the [log](../README.md).
Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision

The owner requested a constraint-modeling approach that can run across a codebase
with the LLM acting as modeler. The model describes preserved behavior and the
alternative implementation space; solver and executable feedback challenge it.
Counterexamples drive model or candidate revision rather than more unstructured
local edits. Discovery and metrics support this mechanism.

Separate representation selection from behavioral preservation. Count shared
kernels, tables, wrappers and changed callers in the objective. A solver verdict
is only as faithful as the encoded source semantics. Tests and model-fidelity
checks must remain visible alongside restricted proofs.

## Artifact and boundary

The owner asked to capture a follow-up Kiro document. The resulting
[brief](../../../.kiro/specs/constraint-guided-compression/brief.md) and
[spec metadata](../../../.kiro/specs/constraint-guided-compression/spec.json)
record initialization. There was no new extraction, solver run or compression
measurement in this decision turn.

The next day's discussion added queryable graphs, scattered state, repeated
modeler/critic passes, state-explosion research and CodeQL selection. Those are
later refinements; do not backdate them into this decision. See
[R02](../2026-11-02/02-codeql-direction.md) and the dated amendments in the
[research protocol](../../../.kiro/specs/constraint-guided-compression/research.md).

[Day index](README.md) | [Research log](../README.md)
