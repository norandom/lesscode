# R08: executable state observations challenge the graph model

Report/discussion date: 2026-11-02, Europe/Berlin. Experiment: CG-G0c.
Predecessor: [R07](07-codeql-call-census.md). Spec:
[constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision: revise the completeness assumption

Both language queries miss the known alias dependency demonstrated by execution.
They retain all observation sites and the expected return/throw-to-cleanup CFG
connections, but produce **zero input-flow rows**. These results reject the
assumption that this query configuration supplies a complete state-dependency
model for the fixture. They do not locate the underlying cause or disprove
CodeQL's ability to represent it through other APIs.

The [static assessment](../../experiments/constraint-modeler/g0c-01/assessment.json)
is 9/10 for each language and exits nonzero. That ratio is misleading without
the failed positive control: negative flow assertions pass because no flows
are reported at all. Do not call this 90% semantic fidelity or claim proven
alias/overwrite precision.

| Static observation | Python | TypeScript | Interpretation |
| --- | --- | --- | --- |
| All four observation sites | Present | Present | Site coverage works |
| Input reaches alias observation | Missing | Missing | Known dependence omitted |
| Input reaches independent/overwrite/cleanup | No rows | No rows | Does not establish precision after positive failure |
| Return and throw connect to cleanup | Both | Both | Required CFG connectivity present |
| Cleanup connects back to throw | No | No | Reverse control excluded |

All extraction and query commands succeeded; no query repair was made. The
failed assertion is a semantic observation, not a compiler or setup failure.

[Fresh-process replay](../../experiments/constraint-modeler/g0c-01/assessment-replay.json)
reproduces all four result sets, including the missing dependencies. The
[verification receipt](../../experiments/constraint-modeler/g0c-01/verification.json)
confirms matching input hashes between runtime fixtures, query fixtures and
replay. Reproducibility does not repair the failed positive control.

## Runtime evidence

The original fixture passes six executable cases per language. Both normal
return and a reachable exception run finally cleanup, with the expected event
order. Three adverse variants per language change alias identity, the overwrite
value, or the cleanup flag. All six variants are detected by observation
mismatches; all processes exit successfully. These are targeted controls, not
a general mutation score or proof over all inputs.

| Variant | Python cases differing / executed | TypeScript cases differing / executed |
| --- | ---: | ---: |
| Original | 0 / 6 | 0 / 6 |
| Alias points to independent object | 4 / 6 | 4 / 6 |
| Overwrite stores 8 instead of 7 | 6 / 6 | 6 / 6 |
| Cleanup leaves closed false | 6 / 6 | 6 / 6 |

The zero-input alias cases pass even in the wrong implementation. Nonzero inputs
expose the mistake. A single default-valued smoke test would have missed it.
Unlike G0b's unreachable exception branch, `fail=true` actually throws here.

## Frozen method

The [plan](../../experiments/constraint-modeler/g0c/plan.md) preceded execution.
Entry receives an input value and failure flag. Two independently allocated
objects start at value zero. An alias writes the input into the first object;
the fixture observes both objects, overwrites the first with seven, and observes
it again. A finally block sets and observes its closed flag on either exit.

For input `v`, observations are exactly `alias=v`, `independent=0`,
`overwrite=7`, `cleanup=true`, in that order. Normal output is `ok`; exceptional
output is `ValueError("requested")` in Python and `Error("requested")` in TS.
The three input values are -3, 0 and 11, each with both failure flags.

[Runtime receipt](../../experiments/constraint-modeler/g0c-runtime/receipt.json),
[frozen runtime harness](../../experiments/constraint-modeler/g0c-runtime/runtime.py),
[extraction/query receipt](../../experiments/constraint-modeler/g0c-01/receipt.json),
and [reproduction instructions](../../experiments/constraint-modeler/g0c/README.md)
retain commands, source copies, observations and versions. Python is uv-managed
3.12.10; Node 24.22.1 executes TypeScript with native type stripping. This is
execution evidence, not a TypeScript type-check. CodeQL 2.26.3 uses the unchanged
locked Python 7.2.3 and JavaScript 2.9.0 packs. No new library was installed.

The graph oracle asks for input flow to the alias observation only, while
retaining all observation sites. Separately it asks for CFG connectivity from
return and throw to finally cleanup, excluding the reverse edge. Graph
connectivity does not establish that cleanup is mandatory on every path.

## Interpretation boundary

Runtime observations and graph facts are different evidence. An absent flow row
must not authorize deletion when a concrete execution exhibits dependence.
Likewise, a possible flow after overwrite would need checking against ordered
state updates. These obligations belong in the LLM's model and its validation
record; graph extraction alone cannot discharge them.

The next bounded diagnostic should locate the first missing edge: input to
write-helper argument, helper parameter to field write, field read to helper
return, and helper result to observation. Compare one-factor isolated variants
(alias versus direct object, helper versus inline access, overwrite absent
versus present) without replacing this trial's oracle. Stop at the first
explanatory difference; do not begin a generic state-analysis engine.

If recovery needs substantial custom analysis, restrict CodeQL to candidate
discovery and source navigation for this boundary. The LLM must then state a
source-backed ordered-state model and have it challenged by separate behavior
checks. That fallback is a proposal, not an implemented or validated modeling
loop. Neither a missing edge nor agreement on these six inputs proves a
transformation safe. A later SMT result would prove only its encoded model.

No production implementation, external specimen, or acceptance threshold changed.
No compression was measured and no SMT solver ran. Full real-code feasibility
remains open under the [prototype gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md).

COBESY structure: make disagreement visible, separate test sensitivity from
analyzer precision, and lead the next agent toward a falsifiable model obligation.
Failed precision checks are evidence about this query configuration, not a
blanket claim about every CodeQL API or the viability of the entire project.
