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


def test_pure_verifier_passes_pure_function():
    from lesscode_baseline.state_boundary.pure_verifier import check_static_purity

    code = """
def pure_transform(data, factor):
    res = []
    for item in data:
        if item > 0:
            res.append(item * factor)
    return sorted(res)
"""
    violations = check_static_purity(code, core_functions={"pure_transform"})
    assert violations == []


def test_pure_verifier_flags_state_mutation():
    from lesscode_baseline.state_boundary.pure_verifier import (
        PurityViolationType,
        check_static_purity,
    )

    code = """
def bad_core_function(self, x):
    self.counter += 1
    return x * 2
"""
    violations = check_static_purity(code, core_functions={"bad_core_function"})
    assert len(violations) >= 1
    assert any(v.violation_type == PurityViolationType.STATE_MUTATION for v in violations)


def test_pure_verifier_flags_global_lookup():
    from lesscode_baseline.state_boundary.pure_verifier import (
        PurityViolationType,
        check_static_purity,
    )

    code = """
def bad_core_with_global(x):
    return x + GLOBAL_MULTIPLIER
"""
    violations = check_static_purity(code, core_functions={"bad_core_with_global"})
    assert len(violations) >= 1
    assert any(v.violation_type == PurityViolationType.GLOBAL_LOOKUP for v in violations)


def test_pure_verifier_flags_io_invocation():
    from lesscode_baseline.state_boundary.pure_verifier import (
        PurityViolationType,
        check_static_purity,
    )

    code = """
def bad_core_with_io(x):
    print("processing", x)
    return x + 1
"""
    violations = check_static_purity(code, core_functions={"bad_core_with_io"})
    assert len(violations) >= 1
    assert any(v.violation_type == PurityViolationType.IO_INVOCATION for v in violations)


def test_bounded_equivalence_passes_on_equivalent_pure_core():
    from lesscode_baseline.state_boundary.pure_verifier import (
        InputDomainBounds,
        verify_bounded_equivalence,
    )

    def orig(x: int, y: int) -> int:
        if x < 0:
            return 0
        return x + y * 2

    def candidate(x: int, y: int) -> int:
        return 0 if x < 0 else x + 2 * y

    bounds = InputDomainBounds(
        ranges={"x": (-10, 10), "y": (-10, 10)},
        edge_cases={"x": [0, -1, 1, -10, 10], "y": [0, -1, 1, -10, 10]},
    )
    witness = verify_bounded_equivalence(orig, candidate, domain_bounds=bounds)
    assert witness is None


def test_bounded_equivalence_finds_divergence_and_counterexample():
    from lesscode_baseline.state_boundary.pure_verifier import (
        InputDomainBounds,
        verify_bounded_equivalence,
    )

    def orig(x: int) -> int:
        # returns 42 when x == 0
        if x == 0:
            return 42
        return x * 2

    def candidate(x: int) -> int:
        # bug: missed x == 0 special case
        return x * 2

    bounds = InputDomainBounds(
        ranges={"x": (-5, 5)},
        edge_cases={"x": [0, 1, -1]},
    )
    witness = verify_bounded_equivalence(orig, candidate, domain_bounds=bounds)
    assert witness is not None
    assert witness.inputs == {"x": 0}
    assert witness.original_output == 42
    assert witness.candidate_output == 0
    assert "PureCoreDivergence" in witness.to_obligation_string()


def test_bounded_equivalence_catches_exception_divergence_on_out_of_domain():
    from lesscode_baseline.state_boundary.pure_verifier import (
        InputDomainBounds,
        verify_bounded_equivalence,
    )

    def orig(x: int) -> int:
        if x < 0:
            raise ValueError("negative not allowed")
        return x * 2

    def candidate(x: int) -> int:
        if x < 0:
            return 0  # silently returns 0 instead of raising ValueError
        return x * 2

    bounds = InputDomainBounds(
        ranges={"x": (-5, 5)},
        edge_cases={"x": [-1, 0, 1]},
    )
    witness = verify_bounded_equivalence(orig, candidate, domain_bounds=bounds)
    assert witness is not None
    assert witness.divergence_kind in ("exception_mismatch", "unhandled_exception")
    assert witness.inputs["x"] < 0


def test_bounded_equivalence_minimizes_integer_counterexample():
    from lesscode_baseline.state_boundary.pure_verifier import (
        InputDomainBounds,
        verify_bounded_equivalence,
    )

    def orig(x: int) -> int:
        if x >= 10:
            return 999
        return x

    def candidate(x: int) -> int:
        # candidate bug: never returns 999
        return x

    bounds = InputDomainBounds(
        ranges={"x": (0, 100)},
        edge_cases={"x": [100]},
    )
    witness = verify_bounded_equivalence(orig, candidate, domain_bounds=bounds)
    assert witness is not None
    assert witness.inputs == {"x": 10}
    assert witness.original_output == 999
    assert witness.candidate_output == 10


def test_generate_state_mutants_targets_exception_and_flush():
    from lesscode_baseline.state_boundary.mutant_generator import (
        StateMutationKind,
        generate_state_mutants,
    )

    code = """
def run_job(self, task):
    try:
        sys.stdout.write("processing")
        sys.stdout.flush()
        execute_task(task)
    except Exception as ex:
        self.audit_log.append(f"failed:{ex}")
        raise
"""
    mutants = generate_state_mutants(code, target_file="job.py")
    assert len(mutants) >= 2
    kinds = {m.mutation_kind for m in mutants}
    assert StateMutationKind.DROP_FAILURE_STATE in kinds
    assert StateMutationKind.BUFFER_TRUNCATION in kinds


def test_generate_state_mutants_swaps_error_order():
    from lesscode_baseline.state_boundary.mutant_generator import (
        StateMutationKind,
        generate_state_mutants,
    )

    code = """
def validate_and_run(token, payload):
    if not token:
        raise ValueError("missing token")
    if not payload:
        raise ValueError("missing payload")
    return True
"""
    mutants = generate_state_mutants(code, target_file="val.py")
    assert any(m.mutation_kind == StateMutationKind.ERROR_ORDER_SWAP for m in mutants)


def test_evaluate_mutant_classifies_outcomes():
    from lesscode_baseline.state_boundary.mutant_generator import (
        MutationOutcome,
        StateMutant,
        StateMutationKind,
        evaluate_mutant,
    )

    mutant = StateMutant(
        mutant_id="mut_001",
        target_node="audit_log",
        mutation_kind=StateMutationKind.DROP_FAILURE_STATE,
        original_code="audit_log.append('err')",
        mutated_code="pass",
        line=10,
    )

    # 1. Test runner catches mutation -> KILLED
    res_kill = evaluate_mutant(mutant, test_runner=lambda code: (False, "test_failure_audit"))
    assert res_kill.outcome == MutationOutcome.KILLED
    assert res_kill.killer_test == "test_failure_audit"

    # 2. Test runner passes -> SURVIVED
    res_surv = evaluate_mutant(mutant, test_runner=lambda code: (True, None))
    assert res_surv.outcome == MutationOutcome.SURVIVED

    # 3. Syntax error -> SYNTAX_ERROR
    mut_syntax = StateMutant(
        mutant_id="mut_bad",
        target_node="bad",
        mutation_kind=StateMutationKind.DROP_FAILURE_STATE,
        original_code="x = 1",
        mutated_code="def broken( :",
        line=1,
    )
    res_err = evaluate_mutant(mut_syntax, test_runner=lambda code: (True, None))
    assert res_err.outcome == MutationOutcome.SYNTAX_ERROR


def test_mutation_gate_passes_when_all_mutants_killed():
    from lesscode_baseline.state_boundary.mutant_generator import (
        StateMutationKind,
        run_mutation_sensitivity_gate,
    )

    code = """
def run_action(self, item):
    try:
        sys.stdout.flush()
        process(item)
    except Exception:
        self.audit_log.append("fail")
        raise
"""
    # Test runner kills every mutant
    report = run_mutation_sensitivity_gate(
        code,
        test_runner=lambda mutated: (False, "test_catches_all"),
    )
    assert report.total_mutants >= 2
    assert report.survived == 0
    assert report.mandatory_kills_met is True
    assert report.qualified is True


def test_mutation_gate_rejects_when_mutant_survives():
    from lesscode_baseline.state_boundary.mutant_generator import (
        run_mutation_sensitivity_gate,
    )

    code = """
def run_action(self, item):
    try:
        sys.stdout.flush()
        process(item)
    except Exception:
        self.audit_log.append("fail")
        raise
"""
    # Test runner passes on mutants (weak test suite) -> mutants survive!
    report = run_mutation_sensitivity_gate(
        code,
        test_runner=lambda mutated: (True, None),
    )
    assert report.survived > 0
    assert report.qualified is False
    assert len(report.surviving_mutants) > 0


def test_obligation_bridge_injects_state_witness_and_retains_unresolved():
    from lesscode_baseline.regenerator import (
        RolloutRecord,
        TurnRequest,
        _known_obligations,
        _turn_feedback_from,
        build_prompt,
    )

    # 1. Turn 1 fails with StateFrameViolation
    record_turn1 = RolloutRecord(
        label="candidate_turn1",
        turn=1,
        rollout_id=1,
        attempt=1,
        score=0.40,
        tier=1,
        failure_kind="StateFrameViolation",
        hashes={},
        diagnostics={
            "state_violations": [
                "StateViolation:secret_unauthorized:unauthorized_write:write inside non-shell"
            ],
            "gaps": {"statements": -2},
        },
    )

    # Obligations capture failure kind and specific state violation witness
    obs = _known_obligations(record_turn1)
    assert "StateFrameViolation" in obs
    assert "StateViolation:secret_unauthorized:unauthorized_write:write inside non-shell" in obs

    # Feedback extracts state violation into first_divergence
    fb = _turn_feedback_from(record_turn1)
    assert fb is not None
    assert "state_violations" in fb["first_divergence"]
    assert "secret_unauthorized" in fb["first_divergence"]


def test_density_scorer_measures_functions_edges_writes_statements():
    from lesscode_baseline.state_boundary.density import (
        measure_scope_source,
    )

    code = """
GLOBAL_V = 0

def step_one(x):
    return x * 2

def step_two(y):
    val = step_one(y)
    return val + 1

def shell_action(self, z):
    self.log = step_two(z)
    GLOBAL_V = 1
"""
    metrics = measure_scope_source(code, pure_functions={"step_one", "step_two"}, shell_functions={"shell_action"})
    assert metrics.function_count == 3
    assert metrics.pure_functions == 2
    assert metrics.pure_function_pct == pytest.approx(66.67, rel=1e-2)
    assert metrics.call_edge_count >= 2  # step_two calls step_one; shell_action calls step_two
    assert metrics.write_site_count >= 2  # self.log and GLOBAL_V


def test_density_gate_distinguishes_consolidation_from_compression():
    """Simulates E4 Density Trap: function reduction (-40%) without statement reduction (-2%)."""
    from lesscode_baseline.state_boundary.density import (
        ScopeMetrics,
        evaluate_deconvolution_outcome,
    )

    baseline = ScopeMetrics(
        function_count=10,
        call_edge_count=30,
        write_site_count=12,
        ast_statements=100,
        canonical_lines=150,
        pure_functions=2,
    )

    # Candidate consolidated 4 functions and 15 call edges, but only shaved 2 statements and 5 lines
    candidate = ScopeMetrics(
        function_count=6,
        call_edge_count=15,
        write_site_count=8,
        ast_statements=98,
        canonical_lines=145,
        pure_functions=4,
    )

    outcome = evaluate_deconvolution_outcome(
        baseline,
        candidate,
        min_statement_reduction_pct=20.0,
        min_line_reduction_pct=20.0,
    )

    assert outcome.function_reduction_pct == 40.0
    assert outcome.call_edge_reduction_pct == 50.0
    assert outcome.statement_reduction_pct == 2.0
    assert outcome.canonical_line_reduction_pct == pytest.approx(3.33, rel=1e-2)
    assert outcome.structurally_consolidated is True
    assert outcome.compression_met is False  # Density trap caught!
    assert "Density trap" in outcome.disposition_note


def test_density_gate_grants_compression_when_statements_and_lines_reduced():
    from lesscode_baseline.state_boundary.density import (
        ScopeMetrics,
        evaluate_deconvolution_outcome,
    )

    baseline = ScopeMetrics(
        function_count=10,
        call_edge_count=30,
        write_site_count=12,
        ast_statements=100,
        canonical_lines=150,
        pure_functions=2,
    )

    candidate = ScopeMetrics(
        function_count=6,
        call_edge_count=15,
        write_site_count=6,
        ast_statements=70,
        canonical_lines=100,
        pure_functions=5,
    )

    outcome = evaluate_deconvolution_outcome(
        baseline,
        candidate,
        min_statement_reduction_pct=20.0,
        min_line_reduction_pct=20.0,
    )

    assert outcome.structurally_consolidated is True
    assert outcome.statement_reduction_pct == 30.0
    assert outcome.canonical_line_reduction_pct == pytest.approx(33.33, rel=1e-2)
    assert outcome.compression_met is True


def test_adversarial_state_boundary_end_to_end_integration():
    """End-to-end integration test demonstrating 100% detection of adverse state faults:

    1. Static write leak detection
    2. Dynamic failure-side invariant evaluation (E3 AuditedAbsence)
    3. Stream buffer preservation (E4 F3)
    4. Adverse mutant survival rejection
    """
    from lesscode_baseline.state_boundary.density import (
        measure_scope_source,
    )
    from lesscode_baseline.state_boundary.frame import (
        StateFrame,
        StateViolationType,
        check_static_frame,
        verify_dynamic_state,
    )
    from lesscode_baseline.state_boundary.mutant_generator import (
        run_mutation_sensitivity_gate,
    )
    from lesscode_baseline.state_boundary.stream_monitor import (
        StreamDivergenceType,
        StreamEvent,
        verify_stream_preservation,
    )

    # 1. Static check rejects write inside non-shell function
    bad_code = """
def pure_core(x):
    self.leak = x
    return x * 2
"""
    frame = StateFrame(allowed_writes=["self.buffer"])
    violations = check_static_frame(bad_code, frame, shell_functions=set())
    assert len(violations) == 1
    assert violations[0].violation_type == StateViolationType.UNAUTHORIZED_WRITE

    # 2. Dynamic check catches E3 AuditedAbsence
    before = {"audit": [], "data": 1}
    after = {"audit": [], "data": 2}  # audit omitted!
    frame_inv = StateFrame(allowed_writes=["data", "audit"], required_failure_invariants=["audit:AuditedAbsence"])
    inv_violations = verify_dynamic_state(before, after, exit_error=RuntimeError("db crash"), frame=frame_inv)
    assert len(inv_violations) == 1
    assert inv_violations[0].violation_type == StateViolationType.MISSING_FAILURE_INVARIANT

    # 3. Stream preservation catches E4 F3
    orig_stream = [
        StreamEvent(sink="stdout", payload="Starting\n", order=0),
        StreamEvent(sink="stdout", payload="Partial buffer\n", order=1),
    ]
    cand_stream = [StreamEvent(sink="stdout", payload="Starting\n", order=0)]
    stream_divs = verify_stream_preservation(orig_stream, cand_stream)
    assert len(stream_divs) == 1
    assert stream_divs[0].divergence_type == StreamDivergenceType.TRUNCATED

    # 4. Mutant gate rejects candidate with surviving adverse mutant
    code_with_mutants = """
def run_op(self):
    try:
        sys.stdout.flush()
    except Exception:
        self.audit.append("fail")
        raise
"""
    # Weak test suite that passes regardless of mutant
    gate_report = run_mutation_sensitivity_gate(code_with_mutants, test_runner=lambda c: (True, None))
    assert gate_report.qualified is False
    assert gate_report.survived > 0










