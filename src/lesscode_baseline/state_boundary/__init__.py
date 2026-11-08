"""State boundary verification package."""
from .frame import (
    StateFrame,
    StateViolation,
    StateViolationType,
    check_static_frame,
    verify_dynamic_state,
)

__all__ = [
    "StateFrame",
    "StateViolation",
    "StateViolationType",
    "check_static_frame",
    "verify_dynamic_state",
]

