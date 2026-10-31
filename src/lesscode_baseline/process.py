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
import re
import selectors
import signal
import subprocess
import time
import uuid
import xml.etree.ElementTree as ET

from .contracts import (CheckEvidence, CheckSpec, CommandSpec, ContractError,
                        ExecutionResult, Failure, ResolvedCommand)

def execute_check(spec: CheckSpec, command: ResolvedCommand, workspace: Path,
                  logs: Path, source_hash: str, plan_hash: str) -> tuple[
                      tuple[CheckEvidence, ...], ExecutionResult, tuple[Failure, ...]]:
    """Execute a declared check once and retain its raw report beside its logs.

    The result path is relative to the command cwd. Remove previous output before
    launch and require a newly written regular file. JUnit identity is
    ``classname.name`` (or ``name`` without classname); only declared IDs are
    accepted. JSON records must already have the current binding and declaration.
    The caller retains the execution and failures together with the check tuple.
    """
    spec = CheckSpec.from_dict(spec.to_dict())
    command = ResolvedCommand.from_dict(command.to_dict())
    # Resolution may replace argv, but must not silently alter execution policy.
    if (command.command_id, command.cwd, command.environment, command.limits) != (
            spec.command.command_id, spec.command.cwd, spec.command.environment, spec.command.limits):
        raise ContractError("resolved check command differs from declared execution policy")
    # Validate bindings before touching the filesystem or launching a command.
    CheckEvidence.from_dict(CheckEvidence(spec.expected_ids[0], "missing", spec.kind,
        spec.required, source_hash, plan_hash, None).to_dict())
    root = Path(workspace).resolve(strict=True)
    report = root / command.cwd / spec.result_path
    _check_report_path(report, root)
    if report.exists():
        if not report.is_file():
            raise ContractError("check result path must be a regular file")
        report.unlink()
    logs = Path(logs).resolve()
    launch_ns = time.time_ns()
    execution = execute(command, root, logs)
    failures = []
    raw_artifact = None
    statuses = {}
    fallback = "missing"
    try:
        _check_report_path(report, root)
        if report.exists():
            if not report.is_file():
                raise ContractError("check report is not a regular file")
            report_mtime = report.stat().st_mtime_ns
            raw = report.read_bytes()
            raw_artifact = uuid.uuid4().hex + ".check." + (
                "json" if spec.result_format == "normalized" else "xml")
            # Persistence errors propagate; do not promise an absent artifact.
            with (logs / raw_artifact).open("xb") as target:
                target.write(raw)
            if report_mtime < launch_ns:
                raise ContractError("check report is stale")
            statuses = _read_check_report(raw, spec, source_hash, plan_hash)
        else:
            failures.append(Failure("check_report_missing", "check command did not write a fresh report",
                spec.result_path, spec.check_id, None, False, execution.stderr_artifact))
    except (ContractError, ValueError, ET.ParseError, UnicodeError) as exc:
        fallback = "error"
        failures.append(Failure("check_report_invalid", str(exc), spec.result_path,
                                spec.check_id, None, False, raw_artifact or execution.stderr_artifact))
    if execution.status != "completed" or execution.exit_code != 0:
        failures.append(Failure("check_execution", f"execution {execution.status}; exit {execution.exit_code}",
            None, spec.check_id, None, False, execution.stderr_artifact))
        statuses = {identity: "error" if status == "passed" else status
                    for identity, status in statuses.items()}
        if execution.status != "completed":
            fallback = "error"
    checks = tuple(CheckEvidence(identity, statuses.get(identity, fallback), spec.kind,
        spec.required, source_hash, plan_hash, raw_artifact or execution.stderr_artifact)
        for identity in spec.expected_ids)
    return checks, execution, tuple(failures)


def _check_report_path(path: Path, root: Path) -> None:
    if not path.resolve().is_relative_to(root):
        raise ContractError("check result path escapes workspace")
    current = root
    for part in path.relative_to(root).parts:
        current = current / part
        if current.is_symlink() or current.is_junction():
            raise ContractError("check result path contains a link or junction")


def _json_unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ContractError(f"duplicate JSON field: {key}")
        result[key] = value
    return result


def _read_check_report(raw: bytes, spec: CheckSpec, source_hash: str,
                       plan_hash: str) -> dict[str, str]:
    statuses = {}
    if spec.result_format == "normalized":
        records = json.loads(raw, object_pairs_hook=_json_unique)
        if not isinstance(records, list):
            raise ContractError("normalized check report must be an array")
        for value in records:
            record = CheckEvidence.from_dict(value)
            if (record.kind, record.required, record.source_hash, record.plan_hash) != (
                    spec.kind, spec.required, source_hash, plan_hash):
                raise ContractError(f"check declaration or source/plan binding mismatch: {record.check_id}")
            _add_check_status(statuses, record.check_id, record.status, spec)
    else:
        # Strip zero bytes only for the declaration scan, covering UTF-16/32 XML.
        # The original bytes are parsed and preserved without transformation.
        if re.search(br"<!\s*(DOCTYPE|ENTITY)\b", raw.replace(b"\x00", b""), re.I):
            raise ContractError("DTD and entity declarations are forbidden")
        tree = ET.fromstring(raw)
        if tree.tag not in ("testsuite", "testsuites"):
            raise ContractError("JUnit root must be testsuite or testsuites")
        _validate_junit_tree(tree)
        for case in tree.iter("testcase"):
            name = case.get("name", "")
            if not name.strip():
                raise ContractError("JUnit testcase requires a name")
            classname = case.get("classname", "")
            identity = classname + "." + name if classname else name
            status = ("error" if case.find("error") is not None else
                      "failed" if case.find("failure") is not None else
                      "skipped" if case.find("skipped") is not None else "passed")
            _add_check_status(statuses, identity, status, spec)
        for suite in tree.iter():
            if suite.tag not in ("testsuite", "testsuites"):
                continue
            cases = list(suite.iter("testcase"))
            for field, actual in (("tests", len(cases)),
                    ("failures", sum(c.find("failure") is not None for c in cases)),
                    ("errors", sum(c.find("error") is not None for c in cases)),
                    ("skipped", sum(c.find("skipped") is not None for c in cases))):
                if field in suite.attrib and int(suite.attrib[field]) != actual:
                    raise ContractError(f"JUnit {field} count does not match testcase evidence")
    return statuses


def _add_check_status(statuses, identity, status, spec):
    if identity in statuses:
        raise ContractError(f"duplicate check identity: {identity}")
    if identity not in spec.expected_ids:
        raise ContractError(f"undeclared check identity: {identity}")
    statuses[identity] = status


def _validate_junit_tree(element):
    """Support explicit testcase outcome elements; reject ambiguous dialects."""
    if element.tag in ("testsuite", "testsuites"):
        allowed = {"testsuite", "properties", "system-out", "system-err"}
        if element.tag == "testsuite":
            allowed.add("testcase")
        for child in element:
            if child.tag not in allowed:
                raise ContractError(f"unsupported JUnit element: {child.tag}")
            if child.tag in ("testsuite", "testcase"):
                _validate_junit_tree(child)
    elif element.tag == "testcase":
        if element.get("status", "run") not in ("run", "passed", "failed"):
            raise ContractError("unsupported JUnit testcase status; use explicit outcome elements")
        # Some producers include a redundant status. A failed status must never
        # become success merely because its required failure/error tag is absent.
        if element.get("status") == "failed" and not any(
                child.tag in ("failure", "error") for child in element):
            raise ContractError("JUnit failed status lacks failure/error evidence")
        for child in element:
            if child.tag not in ("failure", "error", "skipped", "properties", "system-out", "system-err"):
                raise ContractError(f"unsupported JUnit testcase element: {child.tag}")
    # Metadata and outcome payloads cannot introduce executable structure. The
    # descendant scans used for identity/count reconciliation must see only the
    # suite/testcase hierarchy validated above.
    for child in element:
        if child.tag not in ("testsuite", "testcase") and any(
                node.tag in ("testsuites", "testsuite", "testcase") for node in child.iter()):
            raise ContractError(f"JUnit executable elements inside {child.tag} are forbidden")

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
        if os.name == "nt":
            from .process_windows import run_windows
            status, exit_code = run_windows(command, cwd, stdout, stderr, start)
        elif os.name != "posix":
            stderr.write(b"process_containment: unsupported operating system\n"[:command.limits.max_log_bytes])
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
