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
