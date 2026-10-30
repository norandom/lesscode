"""Bounded local execution. Artifact paths are relative to the supplied logs root.

POSIX process groups manage ordinary descendants, not hostile processes which
deliberately leave the group. Windows containment is provided separately.
"""

from datetime import datetime, timezone
import hashlib
from typing import BinaryIO, Literal
import json
import os
from pathlib import Path
import selectors
import signal
import subprocess
import time
import uuid

from .contracts import CommandSpec, ContractError, ExecutionResult, ResolvedCommand

def execute(command: CommandSpec, workspace: Path, logs: Path) -> ExecutionResult:
    """Execute once with exactly the declared environment and an absolute argv[0].

    Invalid inputs fail before execution. Log persistence errors propagate because
    no result may promise artifacts which were not persisted. Environment identity
    hashes variable names only: secret values never enter the environment hash.
    """
    if not isinstance(command, ResolvedCommand):
        raise ContractError("execution requires a ResolvedCommand with tool version")
    command = ResolvedCommand.from_dict(command.to_dict())
    executable = Path(command.argv[0])
    if not executable.is_absolute():
        raise ContractError("executable must be absolute on the current platform")
    root = Path(workspace).resolve(strict=True)
    cwd = (root / command.cwd).resolve(strict=True)
    if not cwd.is_dir() or not cwd.is_relative_to(root):
        raise ContractError("working directory must stay within workspace")
    logs = Path(logs).resolve()
    logs.mkdir(parents=True, exist_ok=True)
    identifier = uuid.uuid4().hex
    stdout_name, stderr_name = identifier + ".stdout.log", identifier + ".stderr.log"
    started_at = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    environment_hash = hashlib.sha256(json.dumps(
        {"names": sorted(command.environment), "values_supplied": bool(command.environment)},
        sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    status, exit_code = "error", None
    with (logs / stdout_name).open("xb") as stdout, (logs / stderr_name).open("xb") as stderr:
        if os.name != "posix":
            stderr.write(b"process_containment: Windows Job Object lifecycle is not available\n"[:command.limits.max_log_bytes])
        else:
            status, exit_code = _run_posix(command, cwd, stdout, stderr, start)
    return ExecutionResult(command, exit_code, status, time.monotonic() - start,
                           stdout_name, stderr_name, started_at, environment_hash)


def _kill_group(process: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def _run_posix(command: ResolvedCommand, cwd: Path, stdout: BinaryIO,
               stderr: BinaryIO, start: float) -> tuple[
                   Literal["completed", "timeout", "log_limit", "error"], int | None]:
    try:
        process = subprocess.Popen(command.argv, cwd=cwd, env=command.environment,
                                   stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, shell=False, start_new_session=True)
    except OSError as exc:
        # Only launch failures become execution errors. Persistence errors propagate.
        stderr.write(("process_error: " + str(exc) + "\n").encode()[:command.limits.max_log_bytes])
        return "error", None
    captured = 0
    status: Literal["completed", "timeout", "log_limit", "error"] = "completed"
    cleanup_at = None
    try:
        with selectors.DefaultSelector() as selector:
            for stream, target in ((process.stdout, stdout), (process.stderr, stderr)):
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, target)
            while selector.get_map() or process.poll() is None:
                now = time.monotonic()
                if cleanup_at is None:
                    if now - start >= command.limits.timeout_seconds:
                        status = "timeout"
                        _kill_group(process)
                        cleanup_at = now
                    elif process.poll() is not None:
                        # Reap ordinary background children even when the leader exits.
                        _kill_group(process)
                        cleanup_at = now
                elif now - cleanup_at > 0.2:
                    break  # An out-of-group process cannot hold our pipe wait open.
                for key, _ in selector.select(0.02):
                    data = os.read(key.fileobj.fileno(), 65536)
                    if not data:
                        selector.unregister(key.fileobj)
                        continue
                    remaining = command.limits.max_log_bytes - captured
                    retained = data[:remaining]
                    key.data.write(retained)
                    key.data.flush()
                    captured += len(retained)
                    if len(data) > remaining and status == "completed":
                        status = "log_limit"
                        _kill_group(process)
                        cleanup_at = time.monotonic()
        return status, process.wait()
    finally:
        _kill_group(process)
        process.wait()
        process.stdout.close()
        process.stderr.close()
