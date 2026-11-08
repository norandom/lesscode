"""Unit tests for state boundary frame conditions and write-site analysis."""
import pytest
from lesscode_baseline.state_boundary.frame import (
    StateFrame,
    StateViolation,
    StateViolationType,
    check_static_frame,
)


def test_frame_declaration_roundtrip():
    frame = StateFrame(
        allowed_writes=["GLOBAL_LOG", "self.buffer", "self._monitored"],
        required_failure_invariants=["AuditedAbsence", "self._failed = True"],
    )
    d = frame.to_dict()
    restored = StateFrame.from_dict(d)
    assert restored.allowed_writes == frame.allowed_writes
    assert restored.required_failure_invariants == frame.required_failure_invariants


def test_static_frame_passes_when_all_writes_inside_declared_shell():
    code = """
def shell_flush(self, item):
    self.buffer = item
    GLOBAL_LOG.append(item)

def pure_calculate(a, b):
    x = a + b
    return x * 2
"""
    frame = StateFrame(allowed_writes=["self.buffer", "GLOBAL_LOG"])
    violations = check_static_frame(code, frame, shell_functions={"shell_flush"})
    assert violations == []


def test_static_frame_flags_write_site_outside_shell():
    code = """
GLOBAL_COUNTER = 0

def pure_calculate(a, b):
    global GLOBAL_COUNTER
    GLOBAL_COUNTER += 1
    return a + b
"""
    frame = StateFrame(allowed_writes=["GLOBAL_COUNTER"])
    violations = check_static_frame(code, frame, shell_functions=set())
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.UNAUTHORIZED_WRITE
    assert violations[0].location == "GLOBAL_COUNTER"
    assert violations[0].line == 6
    assert "pure_calculate" in violations[0].witness


def test_static_frame_flags_write_to_undeclared_location_inside_shell():
    code = """
def shell_run(self):
    self.buffer = 1
    self.secret_unauthorized = 2
"""
    frame = StateFrame(allowed_writes=["self.buffer"])
    violations = check_static_frame(code, frame, shell_functions={"shell_run"})
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.UNAUTHORIZED_WRITE
    assert violations[0].location == "self.secret_unauthorized"
    assert violations[0].line == 4


def test_static_frame_distinguishes_read_only_access():
    code = """
def pure_reader(self, item):
    # Reading an allowed or external variable without mutating is allowed
    val = GLOBAL_CONFIG.get("flag")
    buf = self.buffer
    return val + buf + item
"""
    frame = StateFrame(allowed_writes=[])
    violations = check_static_frame(code, frame, shell_functions=set())
    assert violations == []


def test_dynamic_frame_passes_when_authorized_write_and_invariants_hold():
    from lesscode_baseline.state_boundary.frame import verify_dynamic_state

    frame = StateFrame(
        allowed_writes=["buffer", "audit_log"],
        required_failure_invariants=["audit_log:AuditedAbsence"],
    )
    before = {"buffer": [], "audit_log": [], "config": "read_only"}
    after = {"buffer": [1, 2], "audit_log": [], "config": "read_only"}

    # Normal success exit
    violations = verify_dynamic_state(before, after, exit_error=None, frame=frame)
    assert violations == []


def test_dynamic_frame_catches_omitted_failure_invariant_on_exception():
    """Simulates E3 AuditedAbsence: exception occurs but required failure state is omitted."""
    from lesscode_baseline.state_boundary.frame import verify_dynamic_state

    frame = StateFrame(
        allowed_writes=["buffer", "audit_log"],
        required_failure_invariants=["audit_log:AuditedAbsence"],
    )
    before = {"buffer": [], "audit_log": [], "config": "read_only"}
    # Candidate raised an exception, but audit_log remained empty!
    after = {"buffer": [], "audit_log": [], "config": "read_only"}

    violations = verify_dynamic_state(
        before, after, exit_error=RuntimeError("serialization failed"), frame=frame
    )
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.MISSING_FAILURE_INVARIANT
    assert violations[0].location == "audit_log"
    assert "AuditedAbsence" in violations[0].witness


def test_dynamic_frame_catches_unauthorized_dynamic_write_leak():
    from lesscode_baseline.state_boundary.frame import verify_dynamic_state

    frame = StateFrame(allowed_writes=["buffer"])
    before = {"buffer": 0, "secret_unauthorized": 100}
    # An undeclared field was modified
    after = {"buffer": 1, "secret_unauthorized": 101}

    violations = verify_dynamic_state(before, after, exit_error=None, frame=frame)
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.UNAUTHORIZED_WRITE
    assert violations[0].location == "secret_unauthorized"


def test_dynamic_frame_catches_type_conflict():
    from lesscode_baseline.state_boundary.frame import verify_dynamic_state

    frame = StateFrame(allowed_writes=["counter"])
    before = {"counter": 10}
    # counter was replaced with incompatible string type
    after = {"counter": "ten"}

    violations = verify_dynamic_state(before, after, exit_error=None, frame=frame)
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.TYPE_CONFLICT
    assert violations[0].location == "counter"

