# R47: Deconvolution specification review and hypothesis amendment

- Report date: 2026-11-09, Europe/Berlin.
- Execution interval: no experimental execution; documentation review only.
- Status: completed review; revised hypothesis untested.
- Specs: [native closure](../../../.kiro/specs/native-deconvolution-closure/requirements.md),
  [TypeScript](../../../.kiro/specs/typescript-deconvolution-adapter/requirements.md),
  [Java](../../../.kiro/specs/java-deconvolution-unification/requirements.md),
  [Rust](../../../.kiro/specs/rust-deconvolution-unification/requirements.md).
- Predecessors: [R27](../2026-11-07/01-solution-first-native-slice.md),
  [R41](../2026-11-08/08-state-boundary-ceiling-amendment.md),
  [R46](04-e6-report-envelope-adoption-maturity-gate.md).
- Type: exploratory document/code review and prospective hypothesis revision.

## Decision

Revise the specs before the next phase. The owner clarified that Java was recently
added; Java is now explicitly authorized in steering. This does not select a new
dependency or approve implementation.

Native closure means Rust control/comparison plus counted runtime adapters.
The earlier self-contained semantic evaluator and unconditional shim deletion
are withdrawn. Native design approval is reset and the premature production task
draft is superseded. Adapter specs remain requirements drafts.

## Hypothesis and falsification

The revised native hypothesis is that existing Rust components plus explicit
target-runtime instrumentation can preserve observation fidelity and reduce whole
maintained apparatus. The [gate](../../../.kiro/specs/native-deconvolution-closure/prototype-gate.md)
requires exact recording parity, all mandatory adverse controls killed,
complete declared finite domains, useful net reduction and R41 compliance.
Missing effects, unverified inputs, invalid controls or excess apparatus falsify
proceed. Moving Python code into strings is not removal.

## Frozen inputs and method

This review inspected current spec/steering documents, R27/R41/R46 records,
native/shim/recording_shim.py, state_boundary/pure_verifier.py,
native/src/state_contract.rs and provider counting in regenerator.py.
No execution baseline was frozen and no runtime output is presented as fresh
validation. Future experiments must freeze dirty-source and evaluator hashes,
versions, commands, limits, domains and control inventories before execution.

## Evidence and results

- The shim owns Python imports, SQLAlchemy hooks and sequence execution; IPC
  transport cannot replace these responsibilities.
- The Python verifier invokes callables and varies range parameters individually;
  that does not prove exhaustive Cartesian-domain equivalence.
- R41 freezes 7,173 canonical apparatus lines; per-file caps alone do not enforce it.
- Existing provider token counting cannot be replaced by a heuristic without
  weakening the guard.
- R27 documents Ruff AST/Oxc/JUnit reuse candidates and rejection of serde_jcs
  for golden serialization. They remain alternatives, not adopted dependencies.
- Java's earlier deferred brief and sequential roadmap were inconsistent with
  the owner's new scope clarification. Adapter dependencies are now siblings.

Primary API references checked during review:
[Syn](https://docs.rs/syn/latest/syn/),
[Oxc scope analysis](https://oxc.rs/docs/learn/parser_in_rust/semantic_analysis.html),
[provider token counting](https://platform.claude.com/docs/en/api/http/messages/count_tokens).
These establish component roles, not compatibility of a newly chosen version.

## Limits and interpretation

No prototype, regression suite or platform check ran. Historical E5/E6 evidence
is preserved for its original scopes. The revised design remains provisional;
cost, completeness and reduction targets require actual evidence. No approval
is inferred from documentation consistency.

## Tool/spec consequence and next gate

Apply renewed requirements review, then provisional design/prototype work.
Native production tasks follow proceed and normal approval. Each other language
must define and pass its own bounded gate. Design workflows now require an
explicit contradiction/disposition table and downstream consistency checks.

## COBESY check

Outcome and falsifiers are explicit; prior findings are preserved through this
dated amendment. Review facts, hypotheses and absent execution evidence are
separate. Negative prototype results must remain valid outcomes.
