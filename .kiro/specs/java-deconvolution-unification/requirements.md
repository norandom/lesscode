# Requirements: Java Deconvolution Adapter

Amended 2026-11-09. Requirements approved for design by the subsequent owner request; design/tasks not approved.
See [review amendment](review-amendment.md).

## Scope and adjacent expectations

Qualify a declared Java subset against the common native closure contract.
The core owns outcome semantics, comparator and refinement policy; this adapter
owns language analysis, runtime observation and targeted mutation operators.
Dependencies are the qualified common contract and this adapter's own real-code
prototype, not completion of another language adapter. Full-language parity is
not implied by one passing subset.

Java is an authorized target per owner clarification on 2026-11-09. Parser/runtime dependency selection and implementation remain subject to ordinary review.
Out of scope: universal proofs, changing other adapters or weakening goldens/checks.

### Requirement 1: State and effect coverage
1. When analyzing a declared scope, the adapter shall report mutable locations, source-anchored write sites, permitted shells and unresolved effect assumptions.
2. When checking a candidate, the adapter shall compare observed mutations against a frame frozen independently of that candidate.
3. If an effect cannot be resolved or observed under the supported profile, the adapter shall return unverified and withhold qualification.
4. When reporting purity, the adapter shall distinguish local computation from externally observable writes and require supported callees and runtime observations.

### Requirement 2: Bounded semantic comparison
1. When comparing baseline and candidate, the adapter shall execute both in the pinned target runtime over a frozen finite DomainSpec with exact inputs and Cartesian coverage.
2. If return values, exception/error contracts or observed termination differ, the adapter shall emit a reproducible witness with the stated shrinking guarantee.
3. If enumeration or semantic coverage is incomplete, the adapter shall report unverified; if execution infrastructure fails, it shall report execution-failed.
4. When bounded comparison succeeds, the adapter shall limit the claim to the enumerated domain and report sampled conformance separately.

### Requirement 3: Mutation sensitivity
1. When generating adverse controls, the adapter shall bind each supported operator to source identity, location and an independently expected behavioral fault.
2. When qualifying the oracle, the adapter shall kill every preregistered valid non-equivalent mandatory control and report the complete denominator.
3. If a control survives, is invalid, unexercised, times out without an attributable behavioral witness or fails infrastructure, the adapter shall withhold qualification; equivalent exclusions require recorded evidence.

### Requirement 4: Runtime observation and isolation
1. When recording or replaying, the adapter shall implement the versioned common protocol with explicit state, stream, error and completion events.
2. When capturing output, the adapter shall separate protocol traffic from specimen output and instrument required internal writes/flushes rather than infer them from test-runner stdout.
3. If execution exceeds frozen limits, crashes or emits incomplete/malformed evidence, the runner shall terminate owned processes, clean owned resources and return a non-qualifying outcome.
4. When running candidates, the runner shall isolate them from frozen evaluators and bind evidence to source, tests, runtime, feature/configuration and adapter identities.

### Requirement 5: Qualification and compatibility
1. When measuring a trial, the adapter shall preserve preregistered canonical/statement metric definitions and count the complete replacement, including helpers, configuration, instrumentation and changed callers.
2. When qualifying compression, the adapter shall meet at least 20% canonical-line and 10% statement reduction, with a preregistered absolute useful-reduction target.
3. When production task generation is requested, the workflow shall require this language's real-code prototype to proceed, common-contract conformance, whole-apparatus accounting and normal phase approvals.
4. When validating delivery, the workflow shall preserve existing adapters, CLI and comparator contracts and use the repository's pinned testing and Dagger/native CI workflow.
5. When evidence changes, the workflow shall revalidate affected contracts and shall not inherit another language's qualification.

## Required design decisions

The [provisional design](design.md) records the following decisions and admission blockers:
- Runtime profile: one pinned JDK, one specimen-native Maven or Gradle build and JUnit version.
- Effect boundaries: public/inherited/static fields, aliases, reflection, dynamic dispatch, concurrency and unresolved native calls; final references and records do not imply deep immutability.
- Value codec/comparison: primitive and boxed values, overflow, NaN/infinity/signed zero, null and declared object equality/identity contracts.
- Adverse operators: field mutation, catch cleanup, error ordering and stream flush.
- Reuse candidates: Evaluate tree-sitter-java against required resolved semantics; scope authorization does not approve dependencies or a JVM instrumentation agent.
- Real-code target: one real Java responsibility selected and frozen before prototype execution.
- Exact hook ownership, protocol mapping, subprocess/resource limits, dependency
  decision, metric fixtures, absolute reduction target and full counted inventory.

These are open design/prototype inputs, not claims of implemented capability.
Use the contradictions/disposition table required by the design workflow.
