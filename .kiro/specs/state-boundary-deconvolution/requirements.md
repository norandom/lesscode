# Requirements Document

## Introduction

Define the requirements for state-boundary deconvolution and formal verification of consolidated code.
When complex, scattered code is restructured into a functional core and imperative shell, the deconvolution workflow must ensure that all mutable state boundaries, exception-side state transitions, and intermediate output effects are soundly verified, preventing unobserved behavioral divergence and function density traps.

These requirements address the empirical findings of R40, building upon the repaired controller foundations established in `refinement-evidence-integrity` and the structural measures introduced in `deconvolution-loop`.

## Boundary Context

- **In scope:**
  - Specification and verification of state frame conditions across function and module boundaries.
  - Verification of state invariants and post-conditions on both normal completion and exception/abort exits.
  - Bounded semantic equivalence verification for extracted pure functions across valid and invalid inputs.
  - Adverse state mutation injection and sensitivity enforcement on stateful write sites and error paths.
  - Output stream and partial buffer preservation checks.
  - Integration of state counterexamples into causal multi-turn refinement obligations.
  - Whole-responsibility compression evaluation guarding against function density without code reduction.
- **Out of scope:**
  - Unbounded whole-program mathematical proofs for arbitrary Turing-complete loops.
  - Heavy external theorem-prover runtimes that exceed the R38 7,004 canonical line ceiling.
  - Automated production deployment or upstream code commits without maintainer approval.
- **Adjacent expectations:**
  - The refinement controller executes under the verified evidence rules of `refinement-evidence-integrity` (binding delivered files to evaluated hashes, preventing false passes on timeout, and preserving obligations).
  - Target codebases provide or allow the generation of reproducible execution harnesses.

## Requirements

### Requirement 1: Mutable State Frame Conditions and Invariant Verification

**Objective:** As a maintainer, I want all state mutations explicitly bounded and verified, so that consolidating scattered functions cannot quietly drop or alter side effects.

#### Acceptance Criteria
1. When a function or module is admitted for deconvolution, the deconvolution workflow shall declare its mutable state frame condition identifying all permitted global, instance, storage, and environment locations.
2. When a deconvolution candidate executes, the state verification workflow shall observe all state locations outside the declared frame condition and shall fail acceptance if any undeclared location is modified.
3. When an execution terminates with an unhandled exception or controlled error abort, the state verification workflow shall evaluate post-condition state invariants and shall reject the candidate if expected failure-side state transitions are omitted.
4. If a declared state location is mutated with an incompatible type, structure, or lifecycle invariant, the state verification workflow shall record a state violation and withhold candidate validity.
5. While verifying state frame conditions, the state verification workflow shall distinguish between read-only observations, authorized mutations, and unauthorized side effects.

### Requirement 2: Formal Equivalence for Extracted Pure Functions

**Objective:** As a researcher, I want extracted pure functions verified mathematically across their input domain, so that structural consolidation does not depend solely on sparse execution traces.

#### Acceptance Criteria
1. When functions are partitioned into a functional core, the state verification workflow shall statically verify that each core function contains zero state write sites, zero global lookups, and zero external side-effect invocations.
2. When pure functions are evaluated for behavioral equivalence, the state verification workflow shall verify semantic equivalence against the corresponding original logic over bounded ranges of both valid and out-of-domain inputs.
3. If an input combination yields divergent return values, distinct exception types, or non-equivalent termination behaviors between original and candidate pure logic, the state verification workflow shall generate a minimal reproducible counterexample.
4. Where a pure function relies on unbounded recursion, external nondeterminism, or unmodeled operations, the state verification workflow shall report bounded equivalence as unverified and require fallback dynamic test coverage.
5. The state verification workflow shall prohibit claiming full functional core equivalence if any declared core function was evaluated solely by happy-path sample inputs.

### Requirement 3: Adverse State Mutation Sensitivity Gates

**Objective:** As a maintainer, I want the verification suite challenged with deliberate state faults, so that weak test assertions cannot accept an erroneous consolidation.

#### Acceptance Criteria
1. When candidate verification is initiated, the state verification workflow shall inject targeted adverse mutations into state mutation sites, exception-handling blocks, and cleanup routines.
2. When adverse mutations are executed, the state verification workflow shall require the candidate's test battery to reject every non-equivalent mutant.
3. If an adverse state mutation survives execution without triggering a verification failure, the state verification workflow shall withhold candidate acceptance and identify the untested state boundary.
4. When reporting mutation results, the state verification workflow shall distinguish between behavioral mutant kills, compilation/syntax failures, timeouts, and equivalent mutations.
5. The state verification workflow shall require that at least one adverse mutation targeting exception-side state and one targeting partial output persistence are killed before granting verification qualification.

### Requirement 4: Output Stream and Partial Buffer Preservation

**Objective:** As an operator, I want partial outputs and stream side effects preserved during error paths, so that consolidating functions does not destroy diagnostic traces.

#### Acceptance Criteria
1. When an execution encounters an error after emitting intermediate output to standard streams or logging sinks, the state verification workflow shall compare emitted stream bytes prior to the abort point.
2. If a candidate drops, buffers without flushing, or truncates intermediate output that the original implementation emitted prior to failure, the state verification workflow shall reject the candidate for stream divergence.
3. When multiple external side effects are emitted during an operation, the state verification workflow shall enforce relative effect ordering constraints matching the original execution contract.
4. Where an external sink is explicitly declared commutative, the state verification workflow shall permit order variation while requiring content equality.
5. The state verification workflow shall record any lost stream or logging event as a blocking diagnostic obligation.

### Requirement 5: Causal State Obligation Tracking and Refinement Recovery

**Objective:** As a maintainer, I want state-boundary failures fed causally into subsequent refinement turns, so that the model can repair missing side effects rather than sample randomly.

#### Acceptance Criteria
1. When a candidate fails a state frame condition, pure-core equivalence check, or stream preservation check, the refinement workflow shall retain an individually identifiable unresolved state obligation.
2. When the next refinement turn is prepared, the refinement workflow shall inject the exact failing state witness, affected variable or buffer location, and expected post-condition into the submitted prompt.
3. When prompt content is submitted, the refinement workflow shall enforce declared feedback boundaries and verify that held-out verification sequences are not exposed to the model.
4. If a subsequent candidate passes the specific failing state witness but fails a different frame condition, the refinement workflow shall retain both the historical resolution and the new active obligation.
5. When candidate search exhausts its turn or token budget without resolving all state obligations, the refinement workflow shall restore the original implementation and report unresolved state obligations.

### Requirement 6: Whole-Responsibility Compression Qualification and Density Defense

**Objective:** As a maintainer, I want structural consolidation distinguished from genuine code compression, so that compacting statements into denser functions cannot masquerade as less code.

#### Acceptance Criteria
1. When a deconvolution candidate is evaluated, the deconvolution workflow shall measure function count, intra-scope call edges, state write sites, canonical lines, and AST statement volume across the complete containing scope.
2. If a candidate achieves function count or call-edge reductions while failing to achieve the preregistered whole-responsibility statement or line reduction target, the deconvolution workflow shall report the candidate as structurally consolidated but non-compressed.
3. When reporting deconvolution outcomes, the deconvolution workflow shall separately state function count reduction, call-edge reduction, pure core proportion, write-site consolidation, and net maintained statement reduction.
4. If an implementation adds adapter scaffolding, helper wrappers, or external lookup tables to achieve local function consolidation, the deconvolution workflow shall include all added code volume in the whole-replacement measurement.
5. Where net code reduction is unmeasured or targets are unmet, the deconvolution workflow shall prohibit reporting compression targets as met.
