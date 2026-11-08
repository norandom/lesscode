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


def test_stream_capture_intercepts_stdout_and_stderr():
    import sys
    from lesscode_baseline.state_boundary.stream_monitor import (
        StreamCapture,
        StreamContract,
        verify_stream_preservation,
    )

    with StreamCapture(sinks=["stdout", "stderr"]) as cap:
        sys.stdout.write("info message\n")
        sys.stderr.write("warning message\n")

    events = cap.events
    assert len(events) == 2
    assert events[0].sink == "stdout"
    assert events[0].payload == "info message\n"
    assert events[1].sink == "stderr"
    assert events[1].payload == "warning message\n"


def test_stream_capture_intercepts_logging_sink():
    import logging
    from lesscode_baseline.state_boundary.stream_monitor import StreamCapture

    logger = logging.getLogger("test_logger")
    logger.setLevel(logging.INFO)

    with StreamCapture(sinks=["logging"]) as cap:
        logger.info("lifecycle event fired")

    events = cap.events
    assert len(events) == 1
    assert "logging:test_logger" in events[0].sink
    assert "lifecycle event fired" in events[0].payload



def test_stream_monitor_catches_truncated_output():
    """Simulates E4 F3: candidate drops intermediate buffer emitted prior to abort."""
    from lesscode_baseline.state_boundary.stream_monitor import (
        StreamContract,
        StreamDivergenceType,
        StreamEvent,
        verify_stream_preservation,
    )

    original_events = [
        StreamEvent(sink="stdout", payload="Starting process...\n", order=0),
        StreamEvent(sink="stdout", payload="Processing batch 1 of 5\n", order=1),
        StreamEvent(sink="stdout", payload="Error encountered: out of memory\n", order=2),
    ]

    # Candidate dropped the intermediate progress line before aborting
    candidate_events = [
        StreamEvent(sink="stdout", payload="Starting process...\n", order=0),
        StreamEvent(sink="stdout", payload="Error encountered: out of memory\n", order=1),
    ]

    divergences = verify_stream_preservation(original_events, candidate_events)
    assert len(divergences) > 0
    assert any(
        d.sink == "stdout" and d.divergence_type in (StreamDivergenceType.TRUNCATED, StreamDivergenceType.CONTENT_MISMATCH)
        for d in divergences
    )
    # Check obligation string formatting
    assert "StreamDivergence:stdout" in divergences[0].to_obligation_string()


def test_stream_monitor_enforces_relative_effect_ordering():
    from lesscode_baseline.state_boundary.stream_monitor import (
        StreamContract,
        StreamDivergenceType,
        StreamEvent,
        verify_stream_preservation,
    )

    # Original order: stdout first, then stderr
    original = [
        StreamEvent(sink="stdout", payload="Action initialized\n", order=0),
        StreamEvent(sink="stderr", payload="Warning: fallback route used\n", order=1),
    ]

    # Candidate reversed the order
    candidate = [
        StreamEvent(sink="stderr", payload="Warning: fallback route used\n", order=0),
        StreamEvent(sink="stdout", payload="Action initialized\n", order=1),
    ]

    divergences = verify_stream_preservation(original, candidate)
    assert any(d.divergence_type == StreamDivergenceType.ORDER_MISMATCH for d in divergences)


def test_stream_monitor_allows_commutative_sinks():
    from lesscode_baseline.state_boundary.stream_monitor import (
        StreamContract,
        StreamEvent,
        verify_stream_preservation,
    )

    contract = StreamContract(commutative_sinks={"metrics", "diagnostics"})

    original = [
        StreamEvent(sink="metrics", payload="cpu=20%", order=0),
        StreamEvent(sink="diagnostics", payload="status=ok", order=1),
    ]
    # Inverted order on commutative sinks
    candidate = [
        StreamEvent(sink="diagnostics", payload="status=ok", order=0),
        StreamEvent(sink="metrics", payload="cpu=20%", order=1),
    ]

    divergences = verify_stream_preservation(original, candidate, contract=contract)
    assert divergences == []


