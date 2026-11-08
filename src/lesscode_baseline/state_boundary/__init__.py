"""State boundary verification package."""
from .frame import (
    StateFrame,
    StateViolation,
    StateViolationType,
    check_static_frame,
    verify_dynamic_state,
)
from .mutant_generator import (
    MutationOutcome,
    MutationResult,
    StateMutant,
    StateMutationKind,
    evaluate_mutant,
    generate_state_mutants,
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
    "generate_state_mutants",
    "evaluate_mutant",
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
