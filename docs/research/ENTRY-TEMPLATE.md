# [ID]: [decision or finding]

- Report date/time and timezone:
- Execution interval: [actual timestamps, not estimated; or not run]
- Status: planned / running / completed / rejected / inconclusive / superseded
- Spec: [relative link and relevant requirement/task, if approved]
- Hypothesis ID and predecessor: [link]
- Type: preregistered / exploratory / retrospective

## Decision

State what this result permits next. Separate extraction success, modeling
fidelity, behavioral evidence, compression and adoption.

## Hypothesis and falsification

What changes from the previous attempt, why, and what observation would reject
the proposal? Record targets and budgets before execution; date any amendment.

## Frozen inputs and method

Source revision and dirty snapshot hashes; license; complete ownership/caller
scope; source/test/support populations; tools and query-pack/model versions;
dependency locks; commands; evaluator hashes; comparison arms; seeds/settings;
pass/query/solver budgets. Record unavailable information explicitly.

## Evidence and results

Link raw results, query text/output, graph slices, model revisions, counterexamples,
candidate patches, test logs and metrics. Include failed attempts. Compare the
whole replacement including support and changed callers. Use charts only for
measured quantities; do not conflate physical lines, statements and LLOC.

## Limits and interpretation

Unsupported semantics, unresolved calls/state, baseline failures, model-fidelity
gaps and solver boundaries. Tests are not universal proofs; unknown is not pass.
Explain what can transfer and what is specific to this specimen.

## Tool/spec consequence and next gate

What capability is now justified, revised or rejected? Link the spec decision
and next experiment. If the result is marginal, stop rather than build machinery
around it. Record adoption separately from candidate validity.

## COBESY check

Outcome visible first; expert reader can replay; disproof explicit; negative
results safe to report; facts and interpretation separated; no inflated claims.
