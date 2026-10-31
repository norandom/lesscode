"""Product-only staging and deterministic size evidence.

Physical lines are ``len(original_bytes.splitlines())``: CRLF, LF and CR
separate lines, with a final unterminated line counted. Compression frames
sorted relative UTF-8 paths and original bytes, each preceded by an unsigned
64-bit big-endian length. No contents are decoded or newline-normalized.
"""

from contextlib import contextmanager
from dataclasses import replace
import gzip
import hashlib
from io import BytesIO
import json
from pathlib import Path
import platform
import stat
import struct
import tempfile
from typing import Iterator
import zlib

from .contracts import Failure, Limits, MetricEvidence, ResolvedCommand, Snapshot
from .process import execute

_DEFINITIONS = (
    ("source_bytes", "bytes", "product-original-bytes-v1"),
    ("physical_lines", "lines", "product-bytes-splitlines-final-unterminated-v1"),
    ("compressed_source_bytes", "bytes",
     "product-sorted-utf8-path-bytes-u64be-lengths-gzip-level9-mtime0-v1"),
)

# -I -S prevents Python startup/project configuration from being inherited.
# compile validates bytes with the chosen runtime but never executes source.
_SYNTAX_WORKER = """
import json, pathlib, platform, sys
paths = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
for name in paths:
    compile(pathlib.Path(name).read_bytes(), name, 'exec', dont_inherit=True)
print(json.dumps({'python': platform.python_version(), 'validated': paths}))
"""


def _real_path(root: Path, path: Path) -> None:
    for entry in (path, *path.parents):
        info = entry.lstat()
        if (stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0)
                & stat.FILE_ATTRIBUTE_REPARSE_POINT):
            raise OSError("input contains symlink/reparse point")
        if entry == root:
            if not stat.S_ISDIR(info.st_mode):
                raise OSError("input root must be a directory")
            return
    raise OSError("input path escapes root")


def _content(root: Path, record) -> bytes:
    path = root / record.path
    _real_path(root, path)
    if not stat.S_ISREG(path.lstat().st_mode):
        raise OSError("input must be a regular file")
    data = path.read_bytes()
    if len(data) != record.size_bytes or hashlib.sha256(data).hexdigest() != record.content_hash:
        raise OSError("input bytes differ from snapshot")
    return data


@contextmanager
def stage_product(snapshot: Snapshot) -> Iterator[Snapshot | Failure]:
    """Yield an owned, isolated tree containing only verified product inputs.

    Callers must pass explicit analyzer options disabling implicit ignores and
    auto-configuration. No repository config or ignore file is copied. The
    source scope identity is retained so all metrics bind to the fixed scope.
    Temporary state is removed on success, input failure and caller exceptions.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    with tempfile.TemporaryDirectory(prefix="lesscode-metrics-") as temporary:
        target = Path(temporary) / "product"
        target.mkdir()
        affected = None
        try:
            source = Path(snapshot.root)
            _real_path(source, source)
            files = tuple(sorted((r for r in snapshot.files if r.category == "product"),
                                 key=lambda r: r.path))
            for record in files:
                affected = record.path
                if Path(record.path).suffix != ".py":
                    raise OSError("unsupported product syntax: Python .py files required")
                data = _content(source, record)
                output = target / record.path
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(data)
                _content(target, record)
            staged = replace(snapshot, root=str(target), files=files, exclusions=())
        except OSError as exc:
            staged = Failure("metrics_input", f"input {affected or snapshot.root}: {exc}",
                             affected, None, None, False, None)
        yield staged


def measure_sizes(snapshot: Snapshot, interpreter: Path, limits: Limits,
                  logs: Path) -> tuple[MetricEvidence, ...]:
    """Validate Python through bounded execution, then measure original bytes.

    ``interpreter`` is the caller-selected absolute Python executable; execution
    uses no inherited environment. Logs are retained at the supplied artifact
    root. Their paths in evidence are relative to that root. Persistence errors
    propagate instead of promising evidence that was never stored.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    limits = Limits.from_dict(limits.to_dict())
    runtime = (f"{platform.python_implementation()} {platform.python_version()}; "
               f"zlib {zlib.ZLIB_VERSION}/{zlib.ZLIB_RUNTIME_VERSION}")
    reason, artifact, values = None, None, (None, None, None)
    with stage_product(snapshot) as staged:
        if isinstance(staged, Failure):
            reason = staged.message
        else:
            # Inventory sits outside the product tree; it cannot be measured by
            # analyzers and avoids platform argv limits for large manifests.
            inventory = Path(staged.root).parent / "inventory.json"
            paths = [record.path for record in staged.files]
            inventory.write_text(json.dumps(paths), encoding="utf-8")
            command = ResolvedCommand("metrics-syntax", (str(interpreter), "-I", "-S", "-c",
                _SYNTAX_WORKER, str(inventory)), ".", {}, limits, "selected Python")
            execution = execute(command, Path(staged.root), logs)
            artifact = execution.stderr_artifact
            if execution.status != "completed" or execution.exit_code != 0:
                reason = f"Python syntax validation {execution.status}; exit {execution.exit_code}"
            else:
                try:
                    report = json.loads((Path(logs) / execution.stdout_artifact).read_text(encoding="utf-8"))
                    if (not isinstance(report, dict) or set(report) != {"python", "validated"}
                            or report["validated"] != paths or not isinstance(report["python"], str)
                            or not report["python"]):
                        raise ValueError("invalid syntax worker inventory/runtime")
                    runtime += "; syntax Python " + report["python"]
                except (ValueError, UnicodeError) as exc:
                    reason = f"Python syntax validation report: {exc}"
                    artifact = execution.stdout_artifact
            if reason is None:
                try:
                    values = _size_values(staged)
                except OSError as exc:
                    reason = f"measurement input: {exc}"
    return tuple(MetricEvidence(key, "unavailable" if reason else "available",
        None if reason else value, unit, definition, "lesscode-size", runtime,
        snapshot.scope_hash, reason, artifact)
        for (key, unit, definition), value in zip(_DEFINITIONS, values))


def _size_values(snapshot: Snapshot) -> tuple[int, int, int]:
    raw_bytes, lines = 0, 0
    compressed = BytesIO()
    with gzip.GzipFile(fileobj=compressed, mode="wb", filename="", compresslevel=9, mtime=0) as output:
        for record in snapshot.files:
            data = _content(Path(snapshot.root), record)
            raw_bytes += len(data)
            lines += len(data.splitlines())
            path = record.path.encode("utf-8")
            output.write(struct.pack(">Q", len(path)))
            output.write(path)
            output.write(struct.pack(">Q", len(data)))
            output.write(data)
    return raw_bytes, lines, len(compressed.getvalue())
