# R06: extraction prototype exposes an incomplete TypeScript call view

Report/discussion date: 2026-11-02, Europe/Berlin. Status: bounded prerequisite
experiment completed with a **revise** decision; no compression trial.
Actual command intervals are in the linked attempt receipts.
Predecessor: [R05](05-provisional-design.md). Spec:
[constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Outcome

The owner approved proceeding with the prototype. Matching CodeQL libraries were
installed and locked. Both language databases extract, and cross-file value-flow
queries distinguish propagated input from an unrelated constant.

The [frozen checks](../../experiments/constraint-modeler/g0-04/assessment.json)
pass 13/13 for Python and 12/13 for TypeScript. Our TS relationship query omits
the supplied callback. A second repair did not resolve it, so work stopped at
the query-repair budget. Separate-process query replay reproduces all four result
sets, including the omission. This is repeatability, not correctness of that view.

## Evidence and method

[Complete lab report](../../experiments/constraint-modeler/README.md),
[preregistered plan](../../experiments/constraint-modeler/plan.md),
[source/oracle](../../experiments/constraint-modeler/fixtures/expected.json),
[final command receipt](../../experiments/constraint-modeler/g0-04/receipt.json),
[replay assessment](../../experiments/constraint-modeler/g0-04/assessment-replay.json).

CodeQL 2.26.3 uses Python library 7.2.3 and JavaScript library 2.9.0; lockfiles
retain transitive versions. CPython 3.12.10 runs through uv; Node is v24.22.1.
Failed environment and query attempts are preserved. Resolving the inherited
PATH problem required a scoped runtime PATH and the documented Python executable
option; no Windows Store Python or new launcher was installed.

## Interpretation and next decision

The oracle caught precisely the risk identified in the requirements: a modeler
could receive incomplete relationship evidence without seeing the missing call.
This is a defect in the current query contract; CodeQL's ability to represent the
callback is not disproven. Test a call-site census with explicit unresolved and
anonymous-target states in a separately registered G0b experiment.

State-access and control-site detection are weaker than state-value/alias and
exception-path fidelity. Those checks remain outstanding. No LLM modeling loop,
SMT proof, real-project rewrite or compression saving occurred. The full
[prototype gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
is revise; real-code G1 has not run. No production tasks are authorized.

COBESY check: failed assertions are visible, setup and semantic evidence are
separate, and the next change follows a specific counterexample rather than
an expanded infrastructure plan.
