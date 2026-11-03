# R02: select CodeQL and test evidence fidelity before compression

Report date: 2026-11-02, Europe/Berlin. Status: research direction selected;
setup check completed; extraction and modeling experiment not run.
Hypothesis: H7 in the [research log](../README.md).
Predecessor: [R01](01-retrospective.md).
Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision and rationale

The owner selected CodeQL after comparing queryable program representations.
Use its persisted database and existing Python/TypeScript analysis libraries.
Joern remains a fallback if a concrete limitation warrants comparison. Semgrep
may help future pattern detection/prevention; it is not the selected database.
No separate graph store or new generic agent framework is justified yet.

The owner also authorized MIT licensing of Lesscode. The root license, package
metadata and notices now distinguish project code from third-party components.
This resolves the previously missing project-license declaration; it is not a
claim that all external specimens share that license.

## Actual setup evidence

The local wrapper was inspected and resolved to:

```text
C:/Users/mariu/AppData/Local/Programs/CodeQL/2.26.3/codeql/codeql.exe
```

`version --format=json` reported CodeQL 2.26.3 and build
`7d097a43199effe04ecd9c6bd3ad9bb02a45b3d7` during this session. This is a
transcribed setup observation, not a retained raw extraction receipt. Query-pack
versions, extractor coverage and database contents are not yet verified.
Capture raw version output, executable hash and pack locks with the first run.

## Hypothesis and first falsification gate

A query surface can supply enough source-linked evidence for an LLM to model
behavior scattered across functions and state. Test small labeled Python and
TypeScript fixtures before searching a large repository. Query caller boundaries,
argument/return flow, state reads/writes, control/exception relationships and
unresolved calls. Retain expected missing/uncertain relationships as well as hits.

Fail this prerequisite if a critical relationship is silently omitted or the
modeler treats potential reachability as proven execution. A database that loads
successfully is not evidence of faithful modeling or compression.

## Systematic expansion

G0 extraction fidelity -> G1 small modeled responsibility -> G2 connected real
modules -> G3 bounded slices within a 100k+ codebase. Each step needs a recorded
result and explicit continuation decision. The
[research protocol](../../../.kiro/specs/constraint-guided-compression/research.md)
defines comparison arms and state-exploration limits. Keep the modeler, critic,
candidate author and deterministic evaluator distinct; framework choices beyond
CodeQL remain experimental, not approved dependencies.

No new savings, solver verdicts or graph-coverage percentages are available.
Do not create a result chart for this setup record. Future charts should compare
whole replacements and show how counterexamples and unresolved obligations change
over passes. The formal spec remains initialized: backend/license approval does
not silently mark requirements, design or tasks approved.

COBESY check: the owner and continuing agents can see what is decided, what ran,
what could disprove it, and which result is required next. Negative outcomes
remain publishable. [Research log](../README.md).
