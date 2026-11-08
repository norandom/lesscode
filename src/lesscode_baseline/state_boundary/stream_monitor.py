"""Output stream and partial buffer preservation monitor."""
from __future__ import annotations

import io
import logging
import sys
from collections import defaultdict
from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any, Optional, Sequence, Set


class StreamDivergenceType(str, Enum):
    TRUNCATED = "truncated"
    MISSING = "missing"
    ORDER_MISMATCH = "order_mismatch"
    CONTENT_MISMATCH = "content_mismatch"


@dataclass
class StreamEvent:
    sink: str
    payload: str
    order: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StreamEvent:
        return cls(
            sink=str(data["sink"]),
            payload=str(data["payload"]),
            order=int(data.get("order", 0)),
        )


@dataclass
class StreamContract:
    monitored_sinks: list[str] = field(default_factory=lambda: ["stdout", "stderr"])
    commutative_sinks: Set[str] = field(default_factory=set)

    def to_dict(self) -> dict[str, Any]:
        return {
            "monitored_sinks": list(self.monitored_sinks),
            "commutative_sinks": list(self.commutative_sinks),
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StreamContract:
        return cls(
            monitored_sinks=list(data.get("monitored_sinks", ["stdout", "stderr"])),
            commutative_sinks=set(data.get("commutative_sinks", [])),
        )


@dataclass
class StreamDivergence:
    sink: str
    divergence_type: StreamDivergenceType
    witness: str
    expected: str = ""
    actual: str = ""

    def to_obligation_string(self) -> str:
        return f"StreamDivergence:{self.sink}:{self.divergence_type.value}:{self.witness}"


class _InterceptionStream(io.StringIO):
    def __init__(self, sink_name: str, capture: StreamCapture, passthrough: Optional[Any] = None):
        super().__init__()
        self.sink_name = sink_name
        self.capture = capture
        self.passthrough = passthrough

    def write(self, s: str) -> int:
        if s:
            self.capture._record(self.sink_name, s)
        if self.passthrough is not None and hasattr(self.passthrough, "write"):
            try:
                self.passthrough.write(s)
            except Exception:
                pass
        return super().write(s)

    def flush(self) -> None:
        if self.passthrough is not None and hasattr(self.passthrough, "flush"):
            try:
                self.passthrough.flush()
            except Exception:
                pass
        super().flush()


class _StreamCaptureLogHandler(logging.Handler):
    def __init__(self, capture: StreamCapture, sink_prefix: str = "logging"):
        super().__init__()
        self.capture = capture
        self.sink_prefix = sink_prefix

    def emit(self, record: logging.LogRecord) -> None:
        try:
            msg = self.format(record)
            sink = f"{self.sink_prefix}:{record.name}" if record.name else self.sink_prefix
            self.capture._record(sink, msg + "\n")
        except Exception:
            self.handleError(record)


class StreamCapture:
    """Captures stream emissions to stdout, stderr, and logging sinks with ordering."""

    def __init__(self, sinks: Sequence[str] = ("stdout", "stderr")):
        self.sinks = set(sinks)
        self.events: list[StreamEvent] = []
        self._seq = 0
        self._orig_stdout: Optional[Any] = None
        self._orig_stderr: Optional[Any] = None
        self._log_handler: Optional[_StreamCaptureLogHandler] = None

    def _record(self, sink: str, payload: str) -> None:
        event = StreamEvent(sink=sink, payload=payload, order=self._seq)
        self._seq += 1
        self.events.append(event)

    def __enter__(self) -> StreamCapture:
        if "stdout" in self.sinks:
            self._orig_stdout = sys.stdout
            sys.stdout = _InterceptionStream("stdout", self, self._orig_stdout)
        if "stderr" in self.sinks:
            self._orig_stderr = sys.stderr
            sys.stderr = _InterceptionStream("stderr", self, self._orig_stderr)

        log_sinks = [s for s in self.sinks if s.startswith("log")]
        if log_sinks:
            self._log_handler = _StreamCaptureLogHandler(self)
            root_logger = logging.getLogger()
            root_logger.addHandler(self._log_handler)

        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        if self._orig_stdout is not None:
            sys.stdout = self._orig_stdout
        if self._orig_stderr is not None:
            sys.stderr = self._orig_stderr
        if self._log_handler is not None:
            logging.getLogger().removeHandler(self._log_handler)

    def get_stream_text(self, sink: str) -> str:
        return "".join(e.payload for e in self.events if e.sink == sink)


def verify_stream_preservation(
    original_events: Sequence[StreamEvent],
    candidate_events: Sequence[StreamEvent],
    contract: Optional[StreamContract] = None,
) -> list[StreamDivergence]:
    """Verifies that candidate execution preserved emitted stream bytes and ordering prior to abort."""
    if contract is None:
        contract = StreamContract()

    divergences: list[StreamDivergence] = []

    # 1. Per-sink content check
    orig_by_sink: dict[str, list[str]] = defaultdict(list)
    cand_by_sink: dict[str, list[str]] = defaultdict(list)

    for e in original_events:
        orig_by_sink[e.sink].append(e.payload)
    for e in candidate_events:
        cand_by_sink[e.sink].append(e.payload)

    all_sinks = set(orig_by_sink.keys()) | set(cand_by_sink.keys())

    for sink in sorted(all_sinks):
        orig_text = "".join(orig_by_sink.get(sink, []))
        cand_text = "".join(cand_by_sink.get(sink, []))

        # If it's a commutative sink, allow line-order permutations
        is_commutative = sink in contract.commutative_sinks

        if not cand_text and orig_text:
            divergences.append(
                StreamDivergence(
                    sink=sink,
                    divergence_type=StreamDivergenceType.MISSING,
                    witness=f"Candidate completely omitted output on sink '{sink}'",
                    expected=orig_text,
                    actual=cand_text,
                )
            )
        elif is_commutative:
            # Check multiset of lines
            orig_lines = sorted(orig_text.splitlines())
            cand_lines = sorted(cand_text.splitlines())
            if orig_lines != cand_lines:
                divergences.append(
                    StreamDivergence(
                        sink=sink,
                        divergence_type=StreamDivergenceType.CONTENT_MISMATCH,
                        witness=f"Commutative sink '{sink}' content mismatch",
                        expected=orig_text,
                        actual=cand_text,
                    )
                )
        else:
            if orig_text != cand_text:
                if orig_text.startswith(cand_text):
                    truncated_bytes = len(orig_text) - len(cand_text)
                    divergences.append(
                        StreamDivergence(
                            sink=sink,
                            divergence_type=StreamDivergenceType.TRUNCATED,
                            witness=f"Candidate dropped or failed to flush {truncated_bytes} bytes on sink '{sink}' prior to completion/abort",
                            expected=orig_text,
                            actual=cand_text,
                        )
                    )
                else:
                    divergences.append(
                        StreamDivergence(
                            sink=sink,
                            divergence_type=StreamDivergenceType.CONTENT_MISMATCH,
                            witness=f"Output on sink '{sink}' did not match original output",
                            expected=orig_text,
                            actual=cand_text,
                        )
                    )

    # 2. Relative ordering check across non-commutative sinks
    orig_non_comm = [e for e in original_events if e.sink not in contract.commutative_sinks]
    cand_non_comm = [e for e in candidate_events if e.sink not in contract.commutative_sinks]

    orig_order_sinks = [e.sink for e in orig_non_comm]
    cand_order_sinks = [e.sink for e in cand_non_comm]

    if orig_order_sinks != cand_order_sinks:
        # Check if set of events match but order differs
        if sorted(orig_order_sinks) == sorted(cand_order_sinks):
            mismatch_idx = next(
                (i for i, (o, c) in enumerate(zip(orig_order_sinks, cand_order_sinks)) if o != c),
                0,
            )
            sink_culprit = cand_order_sinks[mismatch_idx] if mismatch_idx < len(cand_order_sinks) else "stream"
            divergences.append(
                StreamDivergence(
                    sink=sink_culprit,
                    divergence_type=StreamDivergenceType.ORDER_MISMATCH,
                    witness=f"Relative ordering between sinks was inverted: expected sequence {orig_order_sinks}, got {cand_order_sinks}",
                    expected=str(orig_order_sinks),
                    actual=str(cand_order_sinks),
                )
            )

    return divergences
