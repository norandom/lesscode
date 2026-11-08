"""State boundary verification package."""
from .frame import (
    StateFrame,
    StateViolation,
    StateViolationType,
    check_static_frame,
    verify_dynamic_state,
)
from .mutant_generator import (
    MutationGateReport,
    MutationOutcome,
    MutationResult,
    StateMutant,
    StateMutationKind,
    evaluate_mutant,
    generate_state_mutants,
    run_mutation_sensitivity_gate,
)
from .pure_verifier import (
    InputDomainBounds,
    PureDivergenceWitness,
    PurityViolation,
    PurityViolationType,
    check_static_purity,
    verify_bounded_equivalence,
)
from .stream_monitor import (
    StreamCapture,
    StreamContract,
    StreamDivergence,
    StreamDivergenceType,
    StreamEvent,
    verify_stream_preservation,
)

__all__ = [
    "StateFrame",
    "StateViolation",
    "StateViolationType",
    "check_static_frame",
    "verify_dynamic_state",
    "StateMutationKind",
    "MutationOutcome",
    "StateMutant",
    "MutationResult",
    "MutationGateReport",
    "generate_state_mutants",
    "evaluate_mutant",
    "run_mutation_sensitivity_gate",
    "PurityViolation",
    "PurityViolationType",
    "check_static_purity",
    "InputDomainBounds",
    "PureDivergenceWitness",
    "verify_bounded_equivalence",
    "StreamCapture",
    "StreamContract",
    "StreamDivergence",
    "StreamDivergenceType",
    "StreamEvent",
    "verify_stream_preservation",
]
