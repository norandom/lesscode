# Requirements: Native Deconvolution Closure

Amended 2026-11-09 after cross-spec review. This revision requires renewed phase
review; prior approvals do not approve the changed contracts. See [review amendment](review-amendment.md).

## Introduction and boundary

The native controller owns analysis, evidence aggregation, comparison, mutation
coordination and model interaction. Target-language runtimes remain necessary to
execute specimen code. Maintained instrumentation is explicitly counted wherever
it resides, including embedded strings and generated-code generators.

In scope: Python adapter qualification and common contracts consumed by the
TypeScript, Java and Rust specs. Out of scope: a new Python interpreter, universal
equivalence proofs, performance targets, changing specimen behavior or frozen
checks, and changes to the four owner WIP files listed in the design.
Preserve existing CLI arguments, exit codes and serialized golden contracts.

### Requirement 1: State boundary enforcement
1. When analyzing a declared source scope, the engine shall report observed write sites, permitted shell locations, resolved effect assumptions and unsupported constructs.
2. When verifying a candidate, the engine shall compare runtime state changes with a frozen baseline frame and reject undeclared externally observable mutations.
3. When execution fails, the engine shall compare declared failure invariants and cleanup observations with the baseline.
4. If a read-only parameter, global configuration or undeclared state location changes, the engine shall produce a source-anchored violation with evidence.
5. If effects through aliases, dynamic calls or external storage cannot be resolved or observed within the declared profile, the engine shall report unverified and withhold qualification.

### Requirement 2: Whole-scope compression
1. When measuring a scope, the engine shall report physical lines, canonical lines, separated-form statements, functions, call edges and write sites with pinned metric definitions and availability.
2. When qualifying compression for these trials, the engine shall require at least 20% canonical-line and 10% statement reduction across the complete replacement, including helpers, configuration, instrumentation and changed callers.
3. If structural consolidation misses either threshold, the engine shall report compression not met.
4. When reporting results, the engine shall distinguish behavioral evidence, structural consolidation and compression qualification.
5. If a metric is unavailable or its definition differs from the frozen baseline, the engine shall withhold the affected qualification rather than substitute zero or incomparable counts.

### Requirement 3: Bounded semantic comparison
1. When auditing a pure function, the engine shall distinguish local computation from externally observable mutation and require resolved pure callees or explicit tested assumptions.
2. When comparing functions, the engine shall execute baseline and candidate with their pinned language runtime over a declared finite domain, recording exact tested inputs, coverage and bounds.
3. If values, exception contracts or observed termination differ, the engine shall emit a reproducible witness and record whether shrinking established minimality within the declared shrink strategy.
4. If domain enumeration, semantic support or observation is incomplete, the engine shall return unverified; if the harness fails, it shall return execution-failed, never equivalent.
5. When all checks pass, the engine shall limit equivalence claims to the enumerated finite domain; sampled runs shall remain sampled-conformance evidence and shall not satisfy an exhaustive-domain gate.

### Requirement 4: Streams and mutation sensitivity
1. When observing execution, the engine shall capture ordered declared write, flush, log and abort events through runtime instrumentation as well as emitted process bytes.
2. If required events are missing, reordered or truncated, the engine shall reject the candidate; if instrumentation coverage is incomplete, it shall withhold qualification.
3. When generating mutants, the engine shall record source identity, operator, location and the independently expected behavioral fault.
4. When qualifying the oracle, the engine shall kill every preregistered valid non-equivalent adverse control and report the full denominator.
5. If a control survives, is not exercised, times out without an attributable behavioral witness, fails compilation or encounters infrastructure failure, the engine shall withhold qualification; equivalent exclusions require recorded evidence before scoring.

### Requirement 5: Specimen execution and recording
1. When recording or replaying, the controller shall use a versioned, source-bound protocol with an explicitly owned target-runtime adapter and validated completion evidence.
2. When managing specimen resources, the runner shall preserve frozen startup, migration, seed/reset, transaction and cleanup semantics within an isolated disposable workspace.
3. If execution diverges, exceeds its deadline, crashes or emits malformed/incomplete protocol data, the runner shall report the step and failure classification, terminate owned child processes and withhold qualification.
4. When replacing the recording path, the runner shall preserve byte-identical frozen LF-body digests, SHA-256 sidecars and existing Level 1/Level 2 comparison contracts.
5. When capturing observations, the runner shall separate protocol traffic from specimen output, bound message/output volume and inventory/hash any intermediate artifacts; transport changes shall not create false implementation-removal credit.

### Requirement 6: Integration and governance
1. When invoking a model, the controller shall preserve provider-specific input counting and limit enforcement, streaming completion/error semantics and cumulative usage accounting.
2. When verification fails, the controller shall feed source-bound causal witnesses into refinement without treating unverified or execution-failed evidence as discharged obligations.
3. When changing maintained code, the workflow shall enforce registered file/function budgets and whole-apparatus accounting, including runtime adapters, before accepting each task.
4. When production task generation or execution is requested, the workflow shall require a proceed prototype result, renewed design approval and an explicit amendment for any breach of the existing 7,173-canonical-line apparatus ceiling.
5. When validating delivery, the workflow shall use pinned nextest and pytest-xdist commands plus the existing Dagger/native platform pipeline, and shall preserve owner WIP and golden/check identities.
