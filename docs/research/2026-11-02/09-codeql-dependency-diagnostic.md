# R09: the query loses alias flow and retains an overwritten value

Report/discussion date: 2026-11-02, Europe/Berlin. Experiment: CG-G0d.
Predecessor: [R08](08-codeql-state-boundary.md). Spec:
[constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision

Keep CodeQL as source-linked discovery evidence; do not treat these query
results as a complete, ordered state model. The diagnostic localizes G0c's
missing dependence to the field write/read round trip, after the input reaches
the write assignment. Replacing the write's alias receiver with the original
object restores the flow in both languages. It also reports input flow after
the object has been overwritten with a constant.

This is a reproducible configuration/fixture limitation, not a diagnosis of
CodeQL internals or a claim that every CodeQL analysis has these properties.
No small query correction was demonstrated. The full feasibility gate remains
**revise**; no production code or external specimen changed.

## Controlled contrasts

The [plan](../../experiments/constraint-modeler/g0d/plan.md) was written before
execution. Broad diagnostic sinks expose source and intermediate nodes without
adding flow rules or barriers. Both languages identify entry's input and trace
it through the call argument and write helper parameter to the assignment RHS.
The subsequent read expression and observation lack flow rows.

| Source variant | Input flow at first observation: Python / TS | Input flow after overwrite: Python / TS | Runtime observations: Python / TS |
| --- | --- | --- | --- |
| Original, aliased receiver and helpers | Missing / Missing | No row / No row | 6/6 / 6/6, retained G0c evidence |
| Inline field accesses, alias retained | Missing / Missing | No row / No row | 6/6 / 6/6 |
| Direct receiver, helpers retained | Present / Present | Present / Present | 6/6 / 6/6 |

The direct variant changes only the first write argument from `alias` to
`original`; the alias assignment remains but no longer supplies that write.
The inline variant replaces only field write/read helper calls with direct
accesses, keeping the alias, overwrite, ordering and control flow. Sources,
queries and databases are separate for each variant. Unused fixture helpers
and imports are retained; this is a diagnostic, not a code-reduction trial.

Inlining alone does not explain the gap. Changing the receiver does. This
localizes sensitivity to that alias use in the tested configuration; it does
not identify a specific internal analysis edge or establish general alias
support. The source clearly writes seven before the later observation, and
all six runtime cases confirm seven there. The extra flow at that observation
cannot be interpreted as exact surviving value provenance.

The first informative contrast triggered the stop rule. The planned third
variant, removing the overwrite, was **not run**. Both executed variants use
G0c's byte-identical runtime drivers and expected observations. No test was
weakened or rewritten to fit the variant.

## Focused confirmation and evidence

A [dated adaptive amendment](../../experiments/constraint-modeler/g0d/confirmation-plan.md)
then tested whether widening diagnostic sinks caused the recovery. G0c's
byte-identical focused query on the direct variant reports flow at observations
9 and 12 in both languages. The independent-object and cleanup observations
remain present without flow. Thus the alias/direct contrast also appears under
the earlier focused configuration.

- [Original checkpoint results](../../experiments/constraint-modeler/g0d-01/assessment.json)
- [Inline checkpoint results](../../experiments/constraint-modeler/g0d-inline/assessment.json)
- [Direct checkpoint results](../../experiments/constraint-modeler/g0d-direct/assessment.json)
- [Inline runtime receipt](../../experiments/constraint-modeler/g0d-inline-runtime/receipt.json)
- [Direct runtime receipt](../../experiments/constraint-modeler/g0d-direct-runtime/receipt.json)
- [Focused confirmation receipt](../../experiments/constraint-modeler/g0d-confirm/receipt.json)

Pinned CodeQL 2.26.3, Python pack 7.2.3, JavaScript pack 2.9.0 and lockfiles
remain unchanged. Python scripts run through uv with CPython 3.12.10; Node
24.22.1 executes the TypeScript fixtures. Commands, hashes, runner versions and
frozen inputs are retained in each run directory. No query compile/API repair
or additional propagation step was required.

[Artifact verification](../../experiments/constraint-modeler/g0d/verification.json)
checks exact replay rows for original and direct variants, source/query identity,
runtime-to-query source matching, unchanged focused queries, and the recorded
contrast. All checks match. Replay preserves the limitation; these checks do
not establish semantic completeness or constitute a compression score.

## Consequence for the proposed modeler

The graph can locate calls, field accesses and candidate responsibility slices.
An absent edge must remain unknown. A present edge is possible flow under a
configuration, not proof of a surviving value after an ordered update. Before
rewriting, the LLM must explicitly identify object identity, alias relationships,
write order and effects, then challenge that account against source and
behavioral observations. Any SMT proof applies to that account only until its
connection to source is established.

The next useful experiment is a bounded source-backed state model challenged
by the frozen alias/overwrite/cleanup controls, including deliberately wrong
models. Do not implement a general heap analyzer merely to turn these queries
green. If the required source-model boundary cannot be established with modest
effort, report the exclusion and revise the proposed real-code scope. This
trial supplies no automated modeling loop, source-equivalence proof, actual
compression, or approval to generate production tasks.

COBESY handoff: the governing finding is the difference between query evidence
and exact state semantics. The contrasts explain why; the model obligations
tell the next researcher what to challenge. Historical failures remain intact,
and none of these setup or precision observations is counted as code savings.
