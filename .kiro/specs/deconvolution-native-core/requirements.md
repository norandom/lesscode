# Requirements: solution-first native slice

Amended 2026-11-07 at the owner's request (including library and complexity-audit
refinements). Generated draft; no phase or execution approval. Supersedes
requirements-fusion-2026-11-07.md, retained as history. Implementation choices live
in [slice-proposal.md](slice-proposal.md) and [research.md](research.md).

## Boundary Context
In: existing comparison capabilities, small artifact/acceptance/turn core, one
live recording/replay path, explicit scope support and honest protection reports.
Deferred: additional languages, proxy/indexers, new mutation/solver engines, model
transport, twelve-command packaging and five-target expansion.
Out: automatic selection/adoption, test weakening, equivalence claims.
Upstream deconvolution-loop owns behavior; pending R26 changes need separate
ratification. Existing native CLI/API contracts remain protected.

### Requirement 1: Demonstrate the solution before production
1. While solution-unlock evidence is absent, stale or unsuccessful, the workflow shall block production design approval, production task generation and production implementation.
2. When bounded prototype execution is proposed, the workflow shall require an owner-approved frozen plan with inputs, observations, numerical targets, code budget and attempt limit.
3. When assessing unlock, the workflow shall require U1-U7 in prototype-gate.md to pass on one frozen attempt with approved independent evidence review.
4. If a criterion fails or is unavailable, the workflow shall remain locked and record revise, stop or not-run.
5. When unlock passes, the workflow shall permit only the demonstrated slice to seek normal phase approvals, without granting them automatically.
6. If inputs, contracts, dependencies, evaluators or budgets change materially, the workflow shall invalidate affected evidence.

### Requirement 2: Reproducible evidence
1. When evidence is admitted, the tool shall bind it to source, scope, protocol, corpus, apparatus, dependencies and evaluator identities.
2. If artifact bytes are missing from the declared artifact roots or their raw-byte digests disagree with committed sidecars, the tool shall report incomplete or invalid evidence and refuse acceptance.
3. When integrity is checked, the tool shall hash bytes as stored and distinguish raw byte identity from declared semantic normalization.
4. When emitting or verifying canonical digests and receipts, the tool shall preserve the upstream prototype's float, string-escaping and indentation rules byte for byte.
5. When reporting results, the tool shall distinguish artifact replay, live execution and real-scope usefulness and preserve historical failures and limitations.

### Requirement 3: Live recording and acceptance
1. When recording an original corpus twice, the tool shall require byte-identical observations after only preregistered normalization and refuse on mismatch.
2. When replaying the original, the tool shall require zero Level 1 and Level 2 divergence before judging candidates.
3. When accepting candidates, the tool shall require fresh complete results, zero Level 1 divergence, unchanged baseline deselections and passing seed tests/reference harnesses.
4. If Level 2 differs, the tool shall require upstream loop requirement 3.2 query-differential evidence and refuse acceptance without it.
5. If protected files, tests, corpora or apparatus change, the tool shall void the candidate.
6. When reporting acceptance, the tool shall declare observations, sequence/operation counts, sensitivity and residue without claiming equivalence.
7. If execution is interrupted or infrastructure fails, the tool shall report that separately and shall not count it as a mutation kill.

### Requirement 4: Useful whole-replacement reuse
1. When measuring replacement size, the workflow shall count shared implementation once and account for all retained support, adapters, descriptors, rule packs, helpers, configuration and maintained workflow code.
2. When comparing implementations, the workflow shall freeze common canonical formatting and ownership rules and report physical lines and separated-form statements separately across three explicit strata: (a) reusable apparatus and per-scope runner/wiring support, (b) specimen-specific domain operation handlers, and (c) transcribed seed sequences and generation strategies.
3. Where specimen-specific behavior requires code, the workflow shall count an explicit bounded adapter instead of hiding behavior in unmeasured configuration.
4. When onboarding the second scope, the workflow shall freeze shared implementation and descriptor semantics and record whether scope support alone suffices.
5. When evaluating whole-replacement usefulness, the workflow shall require both the predeclared relative and absolute reduction on the replaceable apparatus-and-wiring stratum and a net whole-replacement reduction across all strata combined without shortening or weakening seed sequences or operation coverage.
6. If savings require weaker observations, removed behavior, reduced seed/operation coverage, statement packing or relocated uncounted logic, the workflow shall reject the claim.

### Requirement 5: Compatibility and honest decisions
1. When existing preparation/comparison commands are used, the tool shall preserve executable name, arguments, serialized contracts and exit behavior.
2. When a non-verdict command succeeds, the tool shall distinguish command completion from candidate acceptance.
3. When evaluating turn inputs, the tool shall follow the applicable upstream protocol and preserve failed attempts.
4. If a solver control fails, the tool shall void only the differential step under loop requirement 9.5.
5. When enforcing execution bounds, the tool shall preserve adaptive deadlines, load probes and upstream retry rules.

### Requirement 6: Traceability and bounded expansion
1. When a slice is proposed for approval, the workflow shall map individual applicable acceptance criteria to actual checks/evidence or explicit deferrals and consequences.
2. If a deferred capability is necessary for acceptance, the tool shall refuse acceptance rather than bypass it.
3. When capped production work is proposed, the workflow shall require concrete ownership patterns, a release-specific allocation, explicit disposition of pre-existing oversized files and a validated size checker before execution begins.
4. When accounting for maintained support, the workflow shall include every owned language/directory and separately report tests, fixtures, generated output and frozen evidence.
5. When charging prototype development effort against the bounded attempt budget, the workflow shall charge all new or modified shared modules and new or modified adapter wiring in full, while accounting separately for pre-existing oversized entrypoint files touched only by registration hooks and for verbatim-migrated specimen operation and seed blocks verified by diff/AST parity.
6. When expansion is proposed, the workflow shall require a dated scope amendment with unmet need, replacement hypothesis, bounded evidence plan and additional budget.

Review: numeric EARS criteria and boundaries checked; independent gate review and
2026-11-07 library/complexity audit corrections incorporated. This is draft review,
not phase approval.
[Research amendment](../../../docs/research/2026-11-07/01-solution-first-native-slice.md).
