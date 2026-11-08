# Technical Design: State Boundary Deconvolution

## Overview

This specification provides the technical architecture and interfaces for **State Boundary Deconvolution**.
When complex, scattered code is restructured into a functional core and imperative shell, this system provides rigorous software verification to ensure that mutable state boundaries, exception-side state transitions, and intermediate output streams are soundly preserved, while preventing function density traps from masquerading as code compression.

**Purpose:** Eliminate oracle blindness on scattered state and verify semantic equivalence for deconvoluted code.  
**Users:** Maintainers and automated refinement agents operating the Lesscode pipeline.  
**Impact:** Restricts deconvolution acceptance to candidates with mathematically checked pure cores, verified frame conditions, and proven adverse mutant kill rates.

### Goals
- Formally verify extracted functional cores (>50% pure functions) across bounded input spaces.
- Guarantee that all mutable state writes are confined to declared frame conditions.
- Enforce exception-side state invariants (such as audit logging on abort) and partial stdout/stderr stream preservation.
- Gate candidate acceptance on killing 100% of planted adverse state mutants.
- Integrate state counterexamples causally into the refinement loop, preserving unresolved obligations.

### Non-Goals
- Unbounded whole-program mathematical proofs for arbitrary Turing-complete external loops.
- Adding heavy theorem prover dependencies into the production apparatus (`lesscode-core`).
- Application-specific redesign or feature reduction.

---

## Boundary Commitments

### This Spec Owns
- Specification and evaluation of mutable state frame conditions (`FrameCondition`).
- Static write-site analysis and dynamic state violation detection.
- Exception-side post-condition invariant assertions and stream buffer preservation verification.
- Bounded semantic equivalence harness for extracted pure functions.
- Targeted adverse state mutation generator and sensitivity evaluation gate.
- Integration of state counterexamples into the refinement controller's active obligations.

### Out of Boundary
- Core refinement loop process execution and candidate snapshotting (owned by `refinement-evidence-integrity`).
- Macro-level AST parsing and syntax checking (owned by `tree-sitter` / `ast-grep` in `lesscode-core`).
- Upstream specimen git commits or automated production deployments.

### Allowed Dependencies
- `lesscode-core` (Rust native substrate, capped at 7,004 canonical lines).
- `recording.rs` and `receipt.rs` for admission-verified receipt generation.
- Python standard library (`ast`, `sys`, `io`) and `pytest-xdist` / `cargo-nextest` runners.
- Bounded property testing harness (`Hypothesis` in test environment; lightweight internal evaluator in native core).

### Revalidation Triggers
- Any change to `FrameCondition` schema or state snapshot protocols.
- Any modification to the R38 7,004 canonical line ceiling.
- Changes in trace oracle verdict formats or admission rules.

---

## Architecture

### Architecture Pattern & Boundary Map

The system separates the deconvolution process into three cooperative layers: **Static Contract Declarant**, **Dynamic State & Stream Monitor**, and the **Pure Core Equivalence Harness**, connected directly into the causal refinement loop.

```mermaid
flowchart TD
    subgraph Candidate Scope
        Core[Extracted Pure Core Functions]
        Shell[Declared Imperative Shell Functions]
    end

    subgraph Verification Pipeline
        FC[Frame Condition & Invariant Checker]
        PE[Pure Core Bounded Equivalence Engine]
        SM[Stream & Buffer Preservation Monitor]
        AM[Adverse State Mutation Generator]
    end

    subgraph Controller & Refinement Loop
        Loop[Refinement Loop Controller]
        Receipt[Admission-Verified Receipt]
        Obligations[Active State Obligations]
    end

    Shell -->|Inspect write sites| FC
    Core -->|Symbolic / Property Checks| PE
    Shell -->|Intercept Output| SM
    AM -->|Plant Mutants| Candidate Scope

    FC -->|Frame / Post-Condition Violation| Obligations
    PE -->|Counterexample Witness| Obligations
    SM -->|Stream Truncation Witness| Obligations

    Obligations -->|Causal Prompt Injection| Loop
    FC -->|All Passing + Mutants Killed| Receipt
    PE -->|Equivalence Verified| Receipt
    SM -->|Streams Intact| Receipt
```

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|---|---|---|---|
| **Native Core** | Rust 1.98.1 (`lesscode-core`) | Lightweight state frame structures, stream assertions, and receipt admission | Zero-overhead; strictly respects 7,004 line cap |
| **Verification Engine** | Python 3.12+ / Rust | Frame condition monitor, output buffer interceptor, mutation generator | Test & research support scope |
| **Pure Verifier** | Property testing / Bounded SMT | Proves input/output equivalence on extracted functional core | Bounded exploration; emits minimal counterexamples |
| **Test Runners** | `cargo-nextest 0.9.150` & `pytest-xdist 3.8.0` | Parallel execution of test batteries and mutation gates | Native Windows execution |

---

## File Structure Plan

### New Files
- `native/src/state_contract.rs`: Lean native structures for state frame conditions, invariant checks, and stream assertions.
- `src/lesscode_baseline/state_boundary/frame.py`: Frame condition extraction, static write-site checking, and dynamic state observation.
- `src/lesscode_baseline/state_boundary/pure_verifier.py`: Bounded property and equivalence checking for pure core functions.
- `src/lesscode_baseline/state_boundary/mutant_generator.py`: Adverse state mutation injector for exception handlers and write sites.
- `src/lesscode_baseline/state_boundary/stream_monitor.py`: Stdout/stderr buffer interception and effect ordering checker.
- `tests/test_state_boundary.py`: Unit and regression tests covering frame violations, pure equivalence, and adverse mutant kills.

### Modified Files
- `native/src/lib.rs`: Expose `state_contract` module under lean feature gating.
- `native/src/regenerator.rs`: Integrate state failure kinds (`StateFrameViolation`, `PureCoreDivergence`, `StreamDivergence`) into `run_refine_loop`.
- `src/lesscode_baseline/regenerator.py`: Synchronize state failure kinds and obligation tracking in the Python reference.

---

## System Flows

```mermaid
sequenceDiagram
    autonumber
    participant Loop as Refinement Controller
    participant Harness as Verification Pipeline
    participant State as State & Stream Monitor
    participant Pure as Pure Core Verifier
    participant Model as LLM Generator

    Loop->>Harness: Evaluate Candidate
    Harness->>State: Check Static Write Sites & Frame Condition
    alt Write site outside shell
        State-->>Loop: Fail: StateFrameViolation (undeclared location)
        Loop->>Loop: Retain active obligation & snapshot
    else Writes inside declared shell
        Harness->>Pure: Verify Bounded Equivalence on Pure Core
        alt Divergence found on input X
            Pure-->>Loop: Fail: PureCoreDivergence (counterexample X)
            Loop->>Loop: Retain active obligation
        else Pure core verified
            Harness->>State: Execute with Adverse Planted Mutants
            alt Any mutant survives
                State-->>Loop: Fail: MutationSurvivor (untested boundary)
            else All mutants killed & Streams Intact
                Harness-->>Loop: Pass: Verified State & Core Equivalence
                Loop->>Loop: Discharge obligation & Deliver exact candidate
            end
        end
    end
    Loop->>Model: Next Turn (incorporates exact failing witness)
```

---

## Requirements Traceability

| Requirement ID | Summary | Components | Interfaces |
|---|---|---|---|
| **1.1** | Frame condition declaration | `FrameChecker` | `FrameContract::declare()` |
| **1.2** | Observe undeclared state mutations | `FrameChecker` | `StateObserver::observe_leak()` |
| **1.3** | Failure-side post-condition invariants | `FrameChecker` | `InvariantEvaluator::check_failure_state()` |
| **1.4** | Incompatible state structure/type | `FrameChecker` | `StateObserver::validate_schema()` |
| **1.5** | Distinguish read vs. write vs. leak | `FrameChecker` | `StateObserver::classify_access()` |
| **2.1** | Static pure core verification (0 writes, 0 I/O) | `PureVerifier` | `PureChecker::verify_static_purity()` |
| **2.2** | Bounded semantic equivalence on valid/invalid inputs | `PureVerifier` | `PureChecker::verify_bounded_equivalence()` |
| **2.3** | Minimal counterexample generation | `PureVerifier` | `CounterexampleGenerator::minimize()` |
| **2.4** | Bounded equivalence fallback on recursion/nondeterminism | `PureVerifier` | `PureChecker::fallback_dynamic()` |
| **2.5** | Prohibit claiming equivalence from happy-path traces | `PureVerifier` | `EquivalenceReporter::assert_exhaustive()` |
| **3.1** | Inject targeted adverse state mutants | `StateMutantHarness` | `MutantGenerator::inject_state_mutants()` |
| **3.2** | Require candidate suite to kill all mutants | `StateMutantHarness` | `MutantRunner::require_kill_all()` |
| **3.3** | Withhold acceptance on mutant survival | `StateMutantHarness` | `MutantRunner::gate_acceptance()` |
| **3.4** | Classify mutant kills vs compile errors/timeouts | `StateMutantHarness` | `MutantRunner::classify_outcome()` |
| **3.5** | Mandatory exception-state and buffer-flush mutant kills | `StateMutantHarness` | `MutantRunner::enforce_mandatory_kills()` |
| **4.1** | Compare emitted streams prior to abort | `StreamMonitor` | `StreamObserver::compare_pre_abort_bytes()` |
| **4.2** | Reject dropped, buffered, or truncated streams | `StreamMonitor` | `StreamObserver::assert_stream_equality()` |
| **4.3** | Enforce relative effect ordering constraints | `StreamMonitor` | `EffectGraph::verify_order()` |
| **4.4** | Permit order variation on commutative sinks | `StreamMonitor` | `EffectGraph::allow_commutative()` |
| **4.5** | Record lost stream as blocking obligation | `StreamMonitor` | `ObligationBridge::record_stream_failure()` |
| **5.1** | Retain identifiable state obligations on failure | `ObligationBridge` | `RefineLoop::retain_state_obligation()` |
| **5.2** | Inject failing state witness into next prompt | `ObligationBridge` | `PromptBuilder::inject_state_witness()` |
| **5.3** | Enforce held-out verification boundaries | `ObligationBridge` | `FeedbackBoundary::enforce()` |
| **5.4** | Retain historical resolution with new active obligations | `ObligationBridge` | `ObligationTracker::transition()` |
| **5.5** | Restore original on search exhaustion | `ObligationBridge` | `RefineLoop::restore_original()` |
| **6.1** | Scope-wide measurement of lines, statements, functions | `DensityGate` | `ComplexityScorer::measure_scope()` |
| **6.2** | Distinguish structural consolidation from compression | `DensityGate` | `OutcomeReporter::report_density()` |
| **6.3** | Separate reporting of functions, edges, pure share, LLOC | `DensityGate` | `OutcomeReporter::report_breakdown()` |
| **6.4** | Account for all added scaffolding and helpers | `DensityGate` | `ComplexityScorer::account_helpers()` |
| **6.5** | Prohibit reporting compression targets met if unmet | `DensityGate` | `OutcomeReporter::assert_compression()` |

---

## Components and Interfaces

### 1. State Frame & Invariant Checker (`FrameChecker`)
* **Intent:** Enforce frame conditions and failure-side post-conditions.
* **Requirements:** 1.1–1.5, 4.1–4.5

```rust
pub struct StateFrame {
    pub allowed_writes: Vec<String>,
    pub required_failure_invariants: Vec<String>,
}

pub struct StateViolation {
    pub location: String,
    pub violation_type: StateViolationType,
    pub witness: String,
}

pub enum StateViolationType {
    UnauthorizedWrite,
    MissingFailureInvariant,
    TypeMutationConflict,
}

pub trait FrameVerifier {
    fn verify_frame(&self, before: &StateSnapshot, after: &StateSnapshot, exit_status: Result<(), &str>) -> Result<(), StateViolation>;
}
```

### 2. Pure Core Equivalence Engine (`PureVerifier`)
* **Intent:** Statically check purity and verify bounded equivalence across input ranges.
* **Requirements:** 2.1–2.5

```rust
pub struct PureEquivalenceCheck {
    pub function_name: String,
    pub input_bounds: InputDomainBounds,
    pub sample_depth: usize,
}

pub struct PureDivergenceWitness {
    pub function_name: String,
    pub inputs: Vec<serde_json::Value>,
    pub original_output: Result<serde_json::Value, String>,
    pub candidate_output: Result<serde_json::Value, String>,
}
```

### 3. Adverse State Mutant Generator (`StateMutantHarness`)
* **Intent:** Inject targeted faults into exception handlers, output buffer flushes, and write sites to challenge the test battery.
* **Requirements:** 3.1–3.5

```python
class StateMutant(NamedTuple):
    mutant_id: str
    target_file: str
    target_node: str
    mutation_kind: Literal["drop_failure_state", "buffer_truncation", "error_order_swap"]

class MutationResult(NamedTuple):
    mutant_id: str
    outcome: Literal["killed", "survived", "syntax_error", "timeout"]
    killer_test: Optional[str]
```

### 4. Density Defense & Outcome Gate (`DensityGate`)
* **Intent:** Decouple function consolidation from statement/line compression.
* **Requirements:** 6.1–6.5

```rust
pub struct DeconvolutionOutcome {
    pub function_reduction_pct: f64,
    pub call_edge_reduction_pct: f64,
    pub pure_function_pct: f64,
    pub statement_reduction_pct: f64,
    pub canonical_line_reduction_pct: f64,
    pub structurally_consolidated: bool,
    pub compression_met: bool,
}
```

---

## Error Handling

1. **State Leaks (Unauthorized Writes):** Immediately flags `StateViolationType::UnauthorizedWrite`, records the variable/field path, and converts into an unresolved obligation.
2. **Missing Failure Invariants:** If an exception occurs and expected audit records or state flags are absent, flags `MissingFailureInvariant`.
3. **Mutant Survival:** If a planted state mutant survives the test battery, candidate acceptance is denied, identifying the untested state boundary.
4. **Solver / Equivalence Timeouts:** If bounded property exploration exceeds budget, marks equivalence as unverified and enforces dynamic regression fallback.

---

## Testing Strategy

### Unit Tests
- `test_frame_condition_detects_unauthorized_write`: Verify detection when a candidate writes to an undeclared global or attribute.
- `test_failure_invariant_requires_audit_state`: Replay E3 `AuditedAbsence` failure and assert rejection.
- `test_stream_monitor_catches_truncated_output`: Replay E4 F3 and assert rejection when CLI output is truncated prior to error.

### Integration Tests
- `test_pure_verifier_finds_divergence_on_out_of_domain_inputs`: Challenge extracted pure functions with boundary inputs and verify counterexample generation.
- `test_adverse_mutant_generator_rejects_surviving_fault`: Ensure a candidate with green happy-path tests is rejected if an adverse mutant survives.
- `test_causal_feedback_delivers_state_witness_to_prompt`: Verify that state obligations reach subsequent prompts and are retained until resolved.
