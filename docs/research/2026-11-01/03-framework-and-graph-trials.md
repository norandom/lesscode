# D01-03: global knowledge and graphs supplied context, not proven extra savings

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: negative or mixed pilot results. Hypothesis H4 in the [log](../README.md).
Spec context: [follow-up research](../../../.kiro/specs/constraint-guided-compression/research.md).

## Decision and evidence

The owner proposed one global capability-discovery pass so local agents reuse
the installed stack instead of independently adding frameworks. The next idea
used graph relationships to organize compression across connected responsibilities.

The [capability pilot](../../experiments/capability-guided/run-01/README.md)
found the same 12-line / 9-LLOC reuse in both arms. Shared knowledge did not
produce additional verified removal; restraint on an unsafe candidate is useful
but not a demonstrated causal compression benefit.

The [report graph trial](../../experiments/report-graph/README.md) passed behavioral
checks but increased subgraph LLOC from 69 to 73. The
[feature-path probes](../../experiments/feature-path/README.md) found that proposed
intermediate-representation deletions lost distinct information. Both negative
results remain evidence, not reasons to weaken the boundary.

## Hypothesis revision

Connected scope is necessary for honest accounting, but a graph alone does not
identify redundant behavior. Stop rewrites with no defensible deletion premise.
Compare capability-sized alternatives and include all newly shared machinery.

[Next: capability scope](04-capability-scope-and-native-core.md) | [Day index](README.md)
