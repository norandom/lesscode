"""Authoritative version-1 evaluation records and strict JSON boundaries.

Readers never execute commands or inspect the filesystem. Relative paths are
portable POSIX names; resolved source roots/executables may be POSIX or Windows.
Constructors are useful internally; ``loads``/``from_dict`` and ``dumps`` validate
external boundaries, including records constructed directly by callers.
"""

from dataclasses import asdict, dataclass
from datetime import datetime
import json
import math
from pathlib import PurePosixPath, PureWindowsPath
import re
from typing import Literal, Self


class ContractError(ValueError):
    """A field in an evaluation document violates the public contract."""


def _fail(path: str, message: str):
    raise ContractError(f"{path}: {message}")


def _object(value, path: str, required: str, optional: str = "") -> dict:
    if not isinstance(value, dict):
        _fail(path, "expected object")
    allowed = set(required.split()) | set(optional.split())
    for key in value:
        if key not in allowed:
            _fail(f"{path}.{key}", "unknown field")
    for key in required.split():
        if key not in value:
            _fail(f"{path}.{key}", "required field missing")
    return value


def _string(value, path: str) -> str:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        _fail(path, "expected nonempty string without NUL")
    return value


def _boolean(value, path: str) -> bool:
    if type(value) is not bool:
        _fail(path, "expected boolean")
    return value


def _number(value, path: str, minimum=None):
    if type(value) not in (int, float):
        _fail(path, "expected finite number")
    try:
        finite = math.isfinite(value)
    except OverflowError:
        finite = False
    if not finite or (minimum is not None and value < minimum):
        _fail(path, f"expected finite number >= {minimum}" if minimum is not None else "expected finite number")
    return value


def _integer(value, path: str, minimum=0) -> int:
    if type(value) is not int or value < minimum:
        _fail(path, f"expected integer >= {minimum}")
    return value


def _choice(value, path: str, choices):
    if not isinstance(value, str) or value not in choices:
        _fail(path, f"expected one of {', '.join(choices)}")
    return value


def _hash(value, path: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        _fail(path, "expected lowercase SHA-256 fingerprint")
    return value


def _relative(value, path: str, dot=False) -> str:
    value = _string(value, path)
    if dot and value == ".":
        return value
    if "\\" in value or ":" in value or any(part in ("", ".", "..") for part in value.split("/")):
        _fail(path, "expected relative POSIX path without traversal")
    if any(part.endswith((" ", ".")) for part in value.split("/")):
        _fail(path, "path components cannot end with Windows-normalized spaces or dots")
    if PurePosixPath(value).is_absolute():
        _fail(path, "path escapes root")
    return value


def _absolute(value, path: str) -> str:
    value = _string(value, path)
    windows = PureWindowsPath(value)
    if not (PurePosixPath(value).is_absolute() or windows.is_absolute()):
        _fail(path, "expected resolved absolute path")
    if ".." in value.replace("\\", "/").split("/"):
        _fail(path, "resolved path cannot contain traversal")
    return value


def _nullable(value, reader, path):
    return None if value is None else reader(value, path)


def _sequence(value, reader, path: str, identity=None) -> tuple:
    if not isinstance(value, (list, tuple)):
        _fail(path, "expected array")
    result = tuple(reader(item, f"{path}[{i}]") for i, item in enumerate(value))
    if identity is not None:
        seen = set()
        for i, item in enumerate(result):
            key = identity(item)
            if key in seen:
                _fail(f"{path}[{i}]", f"duplicate identity {key!r}")
            seen.add(key)
    return result


def _strings(value, path: str, reader=_string) -> tuple[str, ...]:
    return _sequence(value, reader, path, lambda x: x)


def _mapping(value, path: str, reader=_string) -> dict:
    if not isinstance(value, dict):
        _fail(path, "expected object")
    return {_string(k, path): reader(v, f"{path}.{k}") for k, v in value.items()}


def _setting(value, path: str):
    # Tool options are declared data, never vendor evidence or executable code.
    if type(value) in (int, float):
        return _number(value, path)
    if type(value) is bool or value is None:
        return value
    return _string(value, path)


def _version(value, path: str) -> int:
    if type(value) is not int or value != 1:
        _fail(path, "unsupported schema_version (expected integer 1)")
    return value


def _timestamp(value, path: str) -> str:
    value = _string(value, path)
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        _fail(path, "expected ISO-8601 timestamp")
    if parsed.tzinfo is None:
        _fail(path, "timestamp must include timezone")
    return value


class Record:
    """Serialization shared by the explicit domain readers below."""

    def to_dict(self) -> dict:
        return json.loads(dumps(self))


@dataclass(frozen=True)
class Limits(Record):
    timeout_seconds: float
    max_log_bytes: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "timeout_seconds max_log_bytes")
        timeout = _number(d["timeout_seconds"], path + ".timeout_seconds", 0)
        if timeout == 0:
            _fail(path + ".timeout_seconds", "must be positive")
        return cls(timeout, _integer(d["max_log_bytes"], path + ".max_log_bytes", 1))


@dataclass(frozen=True)
class ScopeRules(Record):
    include: tuple[str, ...]
    exclude: tuple[str, ...]
    generated: tuple[str, ...]
    vendor: tuple[str, ...]
    tests: tuple[str, ...]
    product: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        names = "include exclude generated vendor tests product"
        d = _object(value, path, names)
        return cls(*(_strings(d[n], f"{path}.{n}", _relative) for n in names.split()))


@dataclass(frozen=True)
class ToolSpec(Record):
    name: str
    version: str
    options: dict[str, str | int | float | bool | None]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "name version options")
        return cls(_string(d["name"], path + ".name"), _string(d["version"], path + ".version"),
                   _mapping(d["options"], path + ".options", _setting))


@dataclass(frozen=True)
class CommandSpec(Record):
    command_id: str
    argv: tuple[str, ...]
    cwd: str
    environment: dict[str, str]
    limits: Limits

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "command_id argv cwd environment limits")
        argv = _sequence(d["argv"], _string, path + ".argv")
        if not argv:
            _fail(path + ".argv", "must contain an executable")
        environment = _mapping(d["environment"], path + ".environment", _environment_value)
        if any("=" in key for key in environment):
            _fail(path + ".environment", "variable names cannot contain =")
        return cls(_string(d["command_id"], path + ".command_id"), argv,
                   _relative(d["cwd"], path + ".cwd", dot=True), environment,
                   Limits.from_dict(d["limits"], path + ".limits"))


def _environment_value(value, path):
    if not isinstance(value, str) or "\x00" in value:
        _fail(path, "expected environment string without NUL")
    return value


@dataclass(frozen=True)
class ResolvedCommand(CommandSpec):
    executable_version: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "command_id argv cwd environment limits executable_version")
        command = CommandSpec.from_dict({k: v for k, v in d.items() if k != "executable_version"}, path)
        _absolute(command.argv[0], path + ".argv[0]")
        return cls(command.command_id, command.argv, command.cwd, command.environment, command.limits,
                   _string(d["executable_version"], path + ".executable_version"))


CheckKind = Literal["native", "golden", "invariant", "public_api"]
CHECK_KINDS = ("native", "golden", "invariant", "public_api")


@dataclass(frozen=True)
class CheckSpec(Record):
    check_id: str
    kind: CheckKind
    required: bool
    command: CommandSpec
    result_path: str
    result_format: Literal["normalized", "junit"]
    expected_ids: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "check_id kind required command result_path result_format expected_ids")
        ids = _strings(d["expected_ids"], path + ".expected_ids")
        if not ids:
            _fail(path + ".expected_ids", "at least one expected check is required")
        return cls(_string(d["check_id"], path + ".check_id"), _choice(d["kind"], path + ".kind", CHECK_KINDS),
                   _boolean(d["required"], path + ".required"), CommandSpec.from_dict(d["command"], path + ".command"),
                   _relative(d["result_path"], path + ".result_path"),
                   _choice(d["result_format"], path + ".result_format", ("normalized", "junit")), ids)


@dataclass(frozen=True)
class Threshold(Record):
    metric_key: str
    operator: Literal["absolute_maximum", "delta_maximum", "percentage_delta_maximum"]
    value: float
    unit: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "metric_key operator value unit")
        return cls(_string(d["metric_key"], path + ".metric_key"),
                   _choice(d["operator"], path + ".operator", ("absolute_maximum", "delta_maximum", "percentage_delta_maximum")),
                   _number(d["value"], path + ".value"), _string(d["unit"], path + ".unit"))


@dataclass(frozen=True)
class Profile(Record):
    schema_version: int
    scope: ScopeRules
    tools: tuple[ToolSpec, ...]
    checks: tuple[CheckSpec, ...]
    required_metrics: tuple[str, ...]
    thresholds: tuple[Threshold, ...]
    limits: Limits
    evaluator_resources: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version scope tools checks required_metrics thresholds limits evaluator_resources")
        version = _version(d["schema_version"], path + ".schema_version")
        checks = _sequence(d["checks"], CheckSpec.from_dict, path + ".checks", lambda x: x.check_id)
        if not any(check.required for check in checks):
            _fail(path + ".checks", "at least one required behavior check is required")
        ids = [i for check in checks for i in check.expected_ids]
        if len(ids) != len(set(ids)):
            _fail(path + ".checks", "duplicate expected check identities across commands")
        command_ids = [check.command.command_id for check in checks]
        if len(command_ids) != len(set(command_ids)):
            _fail(path + ".checks", "duplicate command identities")
        metrics = _strings(d["required_metrics"], path + ".required_metrics")
        if not {"complexity", "duplication", "source_bytes"}.issubset(metrics):
            _fail(path + ".required_metrics", "complexity, duplication and source_bytes are mandatory")
        return cls(version, ScopeRules.from_dict(d["scope"], path + ".scope"),
                   _sequence(d["tools"], ToolSpec.from_dict, path + ".tools", lambda x: x.name), checks, metrics,
                   _sequence(d["thresholds"], Threshold.from_dict, path + ".thresholds", lambda x: (x.metric_key, x.operator)),
                   Limits.from_dict(d["limits"], path + ".limits"),
                   _strings(d["evaluator_resources"], path + ".evaluator_resources", _relative))


FrozenProfile = Profile


@dataclass(frozen=True)
class FileRecord(Record):
    path: str
    category: Literal["excluded", "generated", "vendor", "tests", "product", "support"]
    size_bytes: int
    content_hash: str
    mode: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "path category size_bytes content_hash mode")
        mode = _integer(d["mode"], path + ".mode")
        if mode > 0o7777:
            _fail(path + ".mode", "expected permission bits only")
        return cls(_relative(d["path"], path + ".path"),
                   _choice(d["category"], path + ".category", ("excluded", "generated", "vendor", "tests", "product", "support")),
                   _integer(d["size_bytes"], path + ".size_bytes"), _hash(d["content_hash"], path + ".content_hash"), mode)


@dataclass(frozen=True)
class Snapshot(Record):
    root: str
    content_hash: str
    scope_hash: str
    revision: str | None
    files: tuple[FileRecord, ...]
    exclusions: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "root content_hash scope_hash revision files exclusions")
        return cls(_absolute(d["root"], path + ".root"), _hash(d["content_hash"], path + ".content_hash"),
                   _hash(d["scope_hash"], path + ".scope_hash"), _nullable(d["revision"], _string, path + ".revision"),
                   _sequence(d["files"], FileRecord.from_dict, path + ".files", lambda x: x.path.casefold()),
                   _strings(d["exclusions"], path + ".exclusions", _relative))


@dataclass(frozen=True)
class Protection(Record):
    mode: Literal["local_detection", "external_isolation_claimed"]
    provider_evidence: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "mode provider_evidence")
        mode = _choice(d["mode"], path + ".mode", ("local_detection", "external_isolation_claimed"))
        evidence = _nullable(d["provider_evidence"], _string, path + ".provider_evidence")
        if mode == "external_isolation_claimed" and evidence is None:
            _fail(path + ".provider_evidence", "external isolation requires provider evidence reference")
        return cls(mode, evidence)


@dataclass(frozen=True)
class Plan(Record):
    schema_version: int
    profile: Profile
    profile_hash: str
    evaluator_hash: str
    baseline: Snapshot
    candidate: Snapshot
    resolved_commands: tuple[ResolvedCommand, ...]
    protection: Protection

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version profile profile_hash evaluator_hash baseline candidate resolved_commands protection")
        return cls(_version(d["schema_version"], path + ".schema_version"), Profile.from_dict(d["profile"], path + ".profile"),
                   _hash(d["profile_hash"], path + ".profile_hash"), _hash(d["evaluator_hash"], path + ".evaluator_hash"),
                   Snapshot.from_dict(d["baseline"], path + ".baseline"), Snapshot.from_dict(d["candidate"], path + ".candidate"),
                   _sequence(d["resolved_commands"], ResolvedCommand.from_dict, path + ".resolved_commands", lambda x: x.command_id),
                   Protection.from_dict(d["protection"], path + ".protection"))


@dataclass(frozen=True)
class ResourceIdentity(Record):
    """A logical evaluator/tool resource and its resolved byte identity."""

    logical_path: str
    path: str
    content_hash: str
    size_bytes: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "logical_path path content_hash size_bytes")
        return cls(_relative(d["logical_path"], path + ".logical_path"),
                   _absolute(d["path"], path + ".path"),
                   _hash(d["content_hash"], path + ".content_hash"),
                   _integer(d["size_bytes"], path + ".size_bytes"))


@dataclass(frozen=True)
class ToolBinding(Record):
    """Resolved tool evidence; unavailable bindings never authorize execution."""

    name: str
    requested_version: str
    status: Literal["available", "unavailable"]
    executable: str | None
    entry_point: str | None
    observed_version: str | None
    fingerprint: str | None
    reason: str | None
    resources: tuple[ResourceIdentity, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path,
                    "name requested_version status executable entry_point observed_version fingerprint reason resources")
        name = _string(d["name"], path + ".name")
        requested = _string(d["requested_version"], path + ".requested_version")
        status = _choice(d["status"], path + ".status", ("available", "unavailable"))
        executable = _nullable(d["executable"], _absolute, path + ".executable")
        entry_point = _nullable(d["entry_point"], _absolute, path + ".entry_point")
        observed = _nullable(d["observed_version"], _string, path + ".observed_version")
        fingerprint = _nullable(d["fingerprint"], _hash, path + ".fingerprint")
        reason = _nullable(d["reason"], _string, path + ".reason")
        resources = _sequence(d["resources"], ResourceIdentity.from_dict, path + ".resources",
                              lambda resource: resource.logical_path.casefold())
        if status == "available":
            if executable is None:
                _fail(path + ".executable", "available tool requires a resolved executable")
            if observed != requested:
                _fail(path + ".observed_version", "available tool must match requested_version")
            if fingerprint is None:
                _fail(path + ".fingerprint", "available tool requires a fingerprint")
            if not resources:
                _fail(path + ".resources", "available tool requires resource identity")
            if reason is not None:
                _fail(path + ".reason", "available tool must have null reason")
        elif reason is None:
            _fail(path + ".reason", "unavailable tool requires a diagnostic reason")
        return cls(name, requested, status, executable, entry_point, observed, fingerprint, reason, resources)


@dataclass(frozen=True)
class PreparedPlan(Record):
    """Durable plan context; freshness and containment belong to runtime validation."""

    schema_version: int
    plan: Plan
    profile_path: str
    evaluator_root: str
    evaluator_resources: tuple[ResourceIdentity, ...]
    tools: tuple[ToolBinding, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version plan profile_path evaluator_root evaluator_resources tools")
        version = _version(d["schema_version"], path + ".schema_version")
        plan = Plan.from_dict(d["plan"], path + ".plan")
        profile_path = _absolute(d["profile_path"], path + ".profile_path")
        evaluator_root = _absolute(d["evaluator_root"], path + ".evaluator_root")
        resources = _sequence(d["evaluator_resources"], ResourceIdentity.from_dict, path + ".evaluator_resources",
                              lambda resource: resource.logical_path.casefold())
        tools = _sequence(d["tools"], ToolBinding.from_dict, path + ".tools", lambda tool: tool.name)
        resource_names = {resource.logical_path for resource in resources}
        for declared in plan.profile.evaluator_resources:
            if declared not in resource_names:
                _fail(path + ".evaluator_resources", f"missing declared resource {declared!r}")
        tool_by_name = {tool.name: tool for tool in tools}
        for declared in plan.profile.tools:
            if declared.name not in tool_by_name:
                _fail(path + ".tools", f"missing declared analyzer {declared.name!r}")
            if tool_by_name[declared.name].requested_version != declared.version:
                _fail(path + ".tools.requested_version", f"must match profile pin for {declared.name!r}")
        return cls(version, plan, profile_path, evaluator_root, resources, tools)


@dataclass(frozen=True)
class FunctionComplexity(Record):
    path: str
    name: str
    long_name: str
    start_line: int
    end_line: int
    ccn: int
    nloc: int
    token_count: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "path name long_name start_line end_line ccn nloc token_count")
        start = _integer(d["start_line"], path + ".start_line", 1)
        end = _integer(d["end_line"], path + ".end_line", start)
        return cls(_relative(d["path"], path + ".path"), _string(d["name"], path + ".name"),
                   _string(d["long_name"], path + ".long_name"), start, end,
                   _integer(d["ccn"], path + ".ccn", 1), _integer(d["nloc"], path + ".nloc"),
                   _integer(d["token_count"], path + ".token_count"))


@dataclass(frozen=True)
class FileComplexity(Record):
    path: str
    nloc: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "path nloc")
        return cls(_relative(d["path"], path + ".path"), _integer(d["nloc"], path + ".nloc"))


@dataclass(frozen=True)
class ComplexityPayload(Record):
    kind: Literal["complexity"]
    files: tuple[FileComplexity, ...]
    functions: tuple[FunctionComplexity, ...]
    count: int
    minimum: float | None
    maximum: float | None
    median: float | None
    p95: float | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind files functions count minimum maximum median p95")
        _choice(d["kind"], path + ".kind", ("complexity",))
        files = _sequence(d["files"], FileComplexity.from_dict, path + ".files", lambda x: x.path)
        functions = _sequence(d["functions"], FunctionComplexity.from_dict, path + ".functions",
                              lambda x: (x.path, x.start_line, x.end_line, x.long_name))
        count = _integer(d["count"], path + ".count")
        if count != len(functions):
            _fail(path + ".count", "must equal function inventory size")
        file_paths = {f.path for f in files}
        if any(f.path not in file_paths for f in functions):
            _fail(path + ".functions", "function path absent from file inventory")
        distribution = tuple(_nullable(d[n], _number, path + "." + n) for n in ("minimum", "maximum", "median", "p95"))
        if (count == 0 and any(v is not None for v in distribution)) or (count > 0 and any(v is None for v in distribution)):
            _fail(path + ".count", "empty distributions must be null; nonempty distributions require values")
        return cls("complexity", files, functions, count, *distribution)


@dataclass(frozen=True)
class SourceSpan(Record):
    path: str
    start_line: int
    end_line: int

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "path start_line end_line")
        start = _integer(d["start_line"], path + ".start_line", 1)
        return cls(_relative(d["path"], path + ".path"), start, _integer(d["end_line"], path + ".end_line", start))


@dataclass(frozen=True)
class ClonePair(Record):
    first: SourceSpan
    second: SourceSpan

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "first second")
        return cls(SourceSpan.from_dict(d["first"], path + ".first"), SourceSpan.from_dict(d["second"], path + ".second"))


@dataclass(frozen=True)
class DuplicationPayload(Record):
    kind: Literal["duplication"]
    duplicated_tokens: int
    total_tokens: int
    ratio: float | None
    ratio_reason: str | None
    mode: str
    upstream_percentage: float
    clones: tuple[ClonePair, ...]
    scanned_paths: tuple[str, ...]
    zero_token_paths: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind duplicated_tokens total_tokens ratio ratio_reason mode upstream_percentage clones scanned_paths zero_token_paths")
        _choice(d["kind"], path + ".kind", ("duplication",))
        tokens = _integer(d["total_tokens"], path + ".total_tokens")
        duplicated = _integer(d["duplicated_tokens"], path + ".duplicated_tokens")
        ratio = _nullable(d["ratio"], _number, path + ".ratio")
        reason = _nullable(d["ratio_reason"], _string, path + ".ratio_reason")
        if tokens == 0:
            if ratio is not None or reason is None or duplicated != 0:
                _fail(path + ".ratio", "zero denominator requires null ratio, reason and zero duplicated tokens")
        elif ratio is None or reason is not None or ratio < 0 or not math.isclose(ratio, duplicated / tokens):
            _fail(path + ".ratio", "must equal duplicated_tokens / total_tokens")
        return cls("duplication", duplicated, tokens, ratio, reason, _string(d["mode"], path + ".mode"),
                   _number(d["upstream_percentage"], path + ".upstream_percentage", 0),
                   _sequence(d["clones"], ClonePair.from_dict, path + ".clones", lambda x: (x.first, x.second)),
                   _strings(d["scanned_paths"], path + ".scanned_paths", _relative),
                   _strings(d["zero_token_paths"], path + ".zero_token_paths", _relative))


@dataclass(frozen=True)
class PopulationPayload(Record):
    kind: Literal["population"]
    population_ids: tuple[str, ...]
    population_hash: str
    numerator: int
    denominator: int
    explanation: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind population_ids population_hash numerator denominator explanation")
        _choice(d["kind"], path + ".kind", ("population",))
        numerator = _integer(d["numerator"], path + ".numerator")
        denominator = _integer(d["denominator"], path + ".denominator")
        if numerator > denominator:
            _fail(path + ".numerator", "cannot exceed denominator")
        return cls("population", _strings(d["population_ids"], path + ".population_ids"),
                   _hash(d["population_hash"], path + ".population_hash"), numerator, denominator,
                   _nullable(d["explanation"], _string, path + ".explanation"))


@dataclass(frozen=True)
class Finding(Record):
    finding_id: str
    location: SourceSpan
    message: str
    value: float | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "finding_id location message value")
        return cls(_string(d["finding_id"], path + ".finding_id"), SourceSpan.from_dict(d["location"], path + ".location"),
                   _string(d["message"], path + ".message"), _nullable(d["value"], _number, path + ".value"))


@dataclass(frozen=True)
class FindingsPayload(Record):
    kind: Literal["findings"]
    findings: tuple[Finding, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind findings")
        _choice(d["kind"], path + ".kind", ("findings",))
        return cls("findings", _sequence(d["findings"], Finding.from_dict, path + ".findings", lambda x: x.finding_id))


MetricPayload = ComplexityPayload | DuplicationPayload | PopulationPayload | FindingsPayload


def _metric_value(value, path):
    if value is None:
        return None
    if type(value) in (int, float):
        return _number(value, path)
    if not isinstance(value, dict) or "kind" not in value:
        _fail(path + ".kind", "structured metric requires a typed payload kind")
    readers = {"complexity": ComplexityPayload, "duplication": DuplicationPayload,
               "population": PopulationPayload, "findings": FindingsPayload}
    kind = _choice(value["kind"], path + ".kind", tuple(readers))
    return readers[kind].from_dict(value, path)


@dataclass(frozen=True)
class MetricEvidence(Record):
    key: str
    status: Literal["available", "unavailable", "non_comparable"]
    value: float | MetricPayload | None
    unit: str
    definition_id: str
    producer: str
    producer_version: str
    scope_hash: str
    reason: str | None
    raw_artifact: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "key status value unit definition_id producer producer_version scope_hash reason raw_artifact")
        status = _choice(d["status"], path + ".status", ("available", "unavailable", "non_comparable"))
        payload = _metric_value(d["value"], path + ".value")
        reason = _nullable(d["reason"], _string, path + ".reason")
        if status == "available" and (payload is None or reason is not None):
            _fail(path + ".value", "available metric requires value and null reason")
        if status != "available" and (payload is not None or reason is None):
            _fail(path + ".value", "unavailable/non-comparable metric requires null value and reason")
        return cls(_string(d["key"], path + ".key"), status, payload, _string(d["unit"], path + ".unit"),
                   _string(d["definition_id"], path + ".definition_id"), _string(d["producer"], path + ".producer"),
                   _string(d["producer_version"], path + ".producer_version"), _hash(d["scope_hash"], path + ".scope_hash"),
                   reason, _nullable(d["raw_artifact"], _relative, path + ".raw_artifact"))


@dataclass(frozen=True)
class CheckEvidence(Record):
    check_id: str
    status: Literal["passed", "failed", "skipped", "error", "missing"]
    kind: CheckKind
    required: bool
    source_hash: str
    plan_hash: str
    diagnostic_artifact: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "check_id status kind required source_hash plan_hash diagnostic_artifact")
        return cls(_string(d["check_id"], path + ".check_id"),
                   _choice(d["status"], path + ".status", ("passed", "failed", "skipped", "error", "missing")),
                   _choice(d["kind"], path + ".kind", CHECK_KINDS), _boolean(d["required"], path + ".required"),
                   _hash(d["source_hash"], path + ".source_hash"), _hash(d["plan_hash"], path + ".plan_hash"),
                   _nullable(d["diagnostic_artifact"], _relative, path + ".diagnostic_artifact"))


@dataclass(frozen=True)
class Failure(Record):
    code: str
    message: str
    affected_path: str | None
    check_id: str | None
    metric_key: str | None
    retryable: bool
    raw_artifact: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "code message affected_path check_id metric_key retryable raw_artifact")
        return cls(_string(d["code"], path + ".code"), _string(d["message"], path + ".message"),
                   _nullable(d["affected_path"], _relative, path + ".affected_path"),
                   _nullable(d["check_id"], _string, path + ".check_id"),
                   _nullable(d["metric_key"], _string, path + ".metric_key"),
                   _boolean(d["retryable"], path + ".retryable"),
                   _nullable(d["raw_artifact"], _relative, path + ".raw_artifact"))


@dataclass(frozen=True)
class ExecutionResult(Record):
    command: ResolvedCommand
    exit_code: int | None
    status: Literal["completed", "timeout", "log_limit", "error"]
    elapsed_seconds: float
    stdout_artifact: str
    stderr_artifact: str
    started_at: str
    environment_hash: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "command exit_code status elapsed_seconds stdout_artifact stderr_artifact started_at environment_hash")
        exit_code = d["exit_code"]
        if exit_code is not None and type(exit_code) is not int:
            _fail(path + ".exit_code", "expected integer or null")
        status = _choice(d["status"], path + ".status", ("completed", "timeout", "log_limit", "error"))
        if status == "completed" and exit_code is None:
            _fail(path + ".exit_code", "completed execution requires an exit outcome")
        return cls(ResolvedCommand.from_dict(d["command"], path + ".command"), exit_code, status,
                   _number(d["elapsed_seconds"], path + ".elapsed_seconds", 0),
                   _relative(d["stdout_artifact"], path + ".stdout_artifact"),
                   _relative(d["stderr_artifact"], path + ".stderr_artifact"),
                   _timestamp(d["started_at"], path + ".started_at"), _hash(d["environment_hash"], path + ".environment_hash"))


@dataclass(frozen=True)
class Observation(Record):
    snapshot: Snapshot
    metrics: tuple[MetricEvidence, ...]
    checks: tuple[CheckEvidence, ...]
    executions: tuple[ExecutionResult, ...]
    failures: tuple[Failure, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "snapshot metrics checks executions failures")
        return cls(Snapshot.from_dict(d["snapshot"], path + ".snapshot"),
                   _sequence(d["metrics"], MetricEvidence.from_dict, path + ".metrics", lambda x: x.key),
                   _sequence(d["checks"], CheckEvidence.from_dict, path + ".checks", lambda x: x.check_id),
                   _sequence(d["executions"], ExecutionResult.from_dict, path + ".executions", lambda x: x.command.command_id),
                   _sequence(d["failures"], Failure.from_dict, path + ".failures"))


@dataclass(frozen=True)
class IntegrityResult(Record):
    status: Literal["unchanged", "changed", "unavailable"]
    changed_resources: tuple[str, ...]
    protection: Protection

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "status changed_resources protection")
        status = _choice(d["status"], path + ".status", ("unchanged", "changed", "unavailable"))
        resources = _strings(d["changed_resources"], path + ".changed_resources", _relative)
        if status == "unchanged" and resources or status == "changed" and not resources:
            _fail(path + ".changed_resources", "changed status must match changed-resource inventory")
        return cls(status, resources, Protection.from_dict(d["protection"], path + ".protection"))


@dataclass(frozen=True)
class MetricDelta(Record):
    key: str
    status: Literal["available", "unavailable", "non_comparable"]
    before: float | None
    after: float | None
    absolute_delta: float | None
    percentage_delta: float | None
    reason: str | None
    percentage_reason: str | None
    unit: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "key status before after absolute_delta percentage_delta reason percentage_reason unit")
        status = _choice(d["status"], path + ".status", ("available", "unavailable", "non_comparable"))
        numbers = tuple(_nullable(d[n], _number, path + "." + n) for n in ("before", "after", "absolute_delta", "percentage_delta"))
        reason = _nullable(d["reason"], _string, path + ".reason")
        percentage_reason = _nullable(d["percentage_reason"], _string, path + ".percentage_reason")
        if status == "available":
            if any(v is None for v in numbers[:3]) or reason is not None:
                _fail(path + ".absolute_delta", "available delta requires before, after, delta and null reason")
        elif any(v is not None for v in numbers[2:]) or reason is None:
            _fail(path + ".reason", "unavailable/non-comparable delta requires null changes and reason")
        if (numbers[3] is None) != (percentage_reason is not None):
            _fail(path + ".percentage_reason", "null percentage requires reason; available percentage requires null reason")
        return cls(_string(d["key"], path + ".key"), status, *numbers, reason, percentage_reason, _string(d["unit"], path + ".unit"))


@dataclass(frozen=True)
class FileDelta(Record):
    added: tuple[str, ...]
    removed: tuple[str, ...]
    modified: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "added removed modified")
        groups = tuple(_strings(d[n], f"{path}.{n}", _relative) for n in ("added", "removed", "modified"))
        all_paths = [p for group in groups for p in group]
        if len(all_paths) != len(set(all_paths)):
            _fail(path, "a path cannot appear in multiple change categories")
        return cls(*groups)


Decision = Literal["accepted", "rejected", "incomplete", "baseline_ineligible", "invalid"]
DECISIONS = ("accepted", "rejected", "incomplete", "baseline_ineligible", "invalid")


@dataclass(frozen=True)
class Comparison(Record):
    decision: Decision
    reasons: tuple[str, ...]
    metric_deltas: tuple[MetricDelta, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "decision reasons metric_deltas")
        return cls(_choice(d["decision"], path + ".decision", DECISIONS),
                   _sequence(d["reasons"], _string, path + ".reasons"),
                   _sequence(d["metric_deltas"], MetricDelta.from_dict, path + ".metric_deltas", lambda x: x.key))


@dataclass(frozen=True)
class EnvironmentInfo(Record):
    os: str
    python: str
    secrets_supplied: bool

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "os python secrets_supplied")
        return cls(_string(d["os"], path + ".os"), _string(d["python"], path + ".python"),
                   _boolean(d["secrets_supplied"], path + ".secrets_supplied"))


@dataclass(frozen=True)
class RunReport(Record):
    schema_version: int
    run_id: str
    plan_hash: str
    baseline: Observation
    candidate: Observation
    integrity: IntegrityResult
    decision: Decision
    reasons: tuple[str, ...]
    metric_deltas: tuple[MetricDelta, ...]
    test_file_delta: FileDelta
    limits: Limits
    elapsed_seconds: float
    limitations: tuple[str, ...]
    started_at: str
    finished_at: str
    environment: EnvironmentInfo

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version run_id plan_hash baseline candidate integrity decision reasons metric_deltas test_file_delta limits elapsed_seconds limitations started_at finished_at environment")
        return cls(_version(d["schema_version"], path + ".schema_version"), _string(d["run_id"], path + ".run_id"),
                   _hash(d["plan_hash"], path + ".plan_hash"), Observation.from_dict(d["baseline"], path + ".baseline"),
                   Observation.from_dict(d["candidate"], path + ".candidate"), IntegrityResult.from_dict(d["integrity"], path + ".integrity"),
                   _choice(d["decision"], path + ".decision", DECISIONS), _sequence(d["reasons"], _string, path + ".reasons"),
                   _sequence(d["metric_deltas"], MetricDelta.from_dict, path + ".metric_deltas", lambda x: x.key),
                   FileDelta.from_dict(d["test_file_delta"], path + ".test_file_delta"), Limits.from_dict(d["limits"], path + ".limits"),
                   _number(d["elapsed_seconds"], path + ".elapsed_seconds", 0),
                   _sequence(d["limitations"], _string, path + ".limitations"),
                   _timestamp(d["started_at"], path + ".started_at"), _timestamp(d["finished_at"], path + ".finished_at"),
                   EnvironmentInfo.from_dict(d["environment"], path + ".environment"))


@dataclass(frozen=True)
class GenerationProvenance(Record):
    status: Literal["known", "unknown"]
    description: str | None
    source: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "status description source")
        status = _choice(d["status"], path + ".status", ("known", "unknown"))
        description = _nullable(d["description"], _string, path + ".description")
        source = _nullable(d["source"], _string, path + ".source")
        if status == "known" and (description is None or source is None):
            _fail(path + ".source", "known provenance requires description and attributed source")
        if status == "unknown" and (description is not None or source is not None):
            _fail(path + ".description", "unknown provenance requires null description and source")
        return cls(status, description, source)


@dataclass(frozen=True)
class CorpusEntry(Record):
    corpus_id: str
    origin: str
    revision: str | None
    content_hash: str | None
    spdx_license: str | None
    license_artifact: str | None
    generation_provenance: GenerationProvenance

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "corpus_id origin revision content_hash spdx_license license_artifact generation_provenance")
        revision = _nullable(d["revision"], _string, path + ".revision")
        content_hash = _nullable(d["content_hash"], _hash, path + ".content_hash")
        license_id = _nullable(d["spdx_license"], _string, path + ".spdx_license")
        license_artifact = _nullable(d["license_artifact"], _relative, path + ".license_artifact")
        if revision is None and content_hash is None:
            _fail(path + ".content_hash", "revision or content identity is required")
        if license_id is None and license_artifact is None:
            _fail(path + ".spdx_license", "SPDX license or recorded license artifact is required")
        origin = _string(d["origin"], path + ".origin")
        if not origin.startswith(("https://", "http://")):
            # Local origins are references, never evaluated or opened here.
            if PureWindowsPath(origin).is_absolute() or PurePosixPath(origin).is_absolute():
                _absolute(origin, path + ".origin")
            else:
                _relative(origin, path + ".origin", dot=True)
        return cls(_string(d["corpus_id"], path + ".corpus_id"), origin, revision, content_hash,
                   license_id, license_artifact, GenerationProvenance.from_dict(d["generation_provenance"], path + ".generation_provenance"))


@dataclass(frozen=True)
class ObservedMeasurement(Record):
    """Externally supplied observation; absence is explicit and never estimated."""

    status: Literal["available", "unavailable"]
    value: float | None
    unit: str
    source: str | None
    observed_at: str | None
    reason: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "status value unit source observed_at reason")
        status = _choice(d["status"], path + ".status", ("available", "unavailable"))
        number = None if d["value"] is None else _number(d["value"], path + ".value", 0)
        unit = _string(d["unit"], path + ".unit")
        source = _nullable(d["source"], _string, path + ".source")
        timestamp = _nullable(d["observed_at"], _timestamp, path + ".observed_at")
        reason = _nullable(d["reason"], _string, path + ".reason")
        if status == "available":
            for field, item in (("value", number), ("source", source), ("observed_at", timestamp)):
                if item is None:
                    _fail(path + "." + field, "available observation requires value, source and timestamp")
            if reason is not None:
                _fail(path + ".reason", "available observation requires null reason")
        else:
            if number is not None:
                _fail(path + ".value", "unavailable observation requires null value")
            if reason is None:
                _fail(path + ".reason", "unavailable observation requires reason")
        return cls(status, number, unit, source, timestamp, reason)


@dataclass(frozen=True)
class RuntimeBenchmark(Record):
    benchmark_id: str
    workload_hash: str
    environment_hash: str
    measurement: ObservedMeasurement

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "benchmark_id workload_hash environment_hash measurement")
        return cls(_string(d["benchmark_id"], path + ".benchmark_id"),
                   _hash(d["workload_hash"], path + ".workload_hash"),
                   _hash(d["environment_hash"], path + ".environment_hash"),
                   ObservedMeasurement.from_dict(d["measurement"], path + ".measurement"))


@dataclass(frozen=True)
class OptionalEvidenceArtifact(Record):
    """Source/plan provenance for generated or producer-supplied optional evidence."""

    schema_version: int
    artifact_id: str
    source_hash: str
    plan_hash: str
    produced_at: str
    origin: Literal["generated", "producer_supplied"]
    metrics: tuple[MetricEvidence, ...]
    runtime_benchmarks: tuple[RuntimeBenchmark, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version artifact_id source_hash plan_hash produced_at origin metrics runtime_benchmarks")
        metrics = _sequence(d["metrics"], MetricEvidence.from_dict, path + ".metrics", lambda x: x.key)
        benchmarks = _sequence(d["runtime_benchmarks"], RuntimeBenchmark.from_dict,
                               path + ".runtime_benchmarks", lambda x: x.benchmark_id)
        if not metrics and not benchmarks:
            _fail(path, "at least one evidence item is required")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   _string(d["artifact_id"], path + ".artifact_id"),
                   _hash(d["source_hash"], path + ".source_hash"), _hash(d["plan_hash"], path + ".plan_hash"),
                   _timestamp(d["produced_at"], path + ".produced_at"),
                   _choice(d["origin"], path + ".origin", ("generated", "producer_supplied")), metrics, benchmarks)


@dataclass(frozen=True)
class Telemetry(Record):
    token_usage: ObservedMeasurement
    provider_cost: ObservedMeasurement
    agent_elapsed: ObservedMeasurement
    runtime_benchmarks: tuple[RuntimeBenchmark, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "token_usage provider_cost agent_elapsed runtime_benchmarks")
        tokens = ObservedMeasurement.from_dict(d["token_usage"], path + ".token_usage")
        cost = ObservedMeasurement.from_dict(d["provider_cost"], path + ".provider_cost")
        elapsed = ObservedMeasurement.from_dict(d["agent_elapsed"], path + ".agent_elapsed")
        _choice(tokens.unit, path + ".token_usage.unit", ("tokens",))
        if tokens.value is not None:
            _integer(tokens.value, path + ".token_usage.value")
        _choice(elapsed.unit, path + ".agent_elapsed.unit", ("seconds",))
        return cls(tokens, cost, elapsed, _sequence(d["runtime_benchmarks"], RuntimeBenchmark.from_dict,
                   path + ".runtime_benchmarks", lambda x: x.benchmark_id))


@dataclass(frozen=True)
class Budget(Record):
    time_seconds: float | None
    tokens: int | None
    provider_cost: float | None
    cost_unit: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "time_seconds tokens provider_cost cost_unit")
        seconds = None if d["time_seconds"] is None else _number(d["time_seconds"], path + ".time_seconds", 0)
        if seconds == 0:
            _fail(path + ".time_seconds", "time budget must be positive")
        tokens = None if d["tokens"] is None else _integer(d["tokens"], path + ".tokens", 1)
        cost = None if d["provider_cost"] is None else _number(d["provider_cost"], path + ".provider_cost", 0)
        unit = _nullable(d["cost_unit"], _string, path + ".cost_unit")
        if (cost is None) != (unit is None):
            _fail(path + ".cost_unit", "provider cost budget requires its unit; absent cost requires null unit")
        return cls(seconds, tokens, cost, unit)


@dataclass(frozen=True)
class TrialConditions(Record):
    model: str
    settings: dict[str, str | int | float | bool | None]
    prompt_identity: str
    permissions: tuple[str, ...]
    budget: Budget
    interventions: dict[str, str | int | float | bool | None]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "model settings prompt_identity permissions budget interventions")
        return cls(_string(d["model"], path + ".model"), _mapping(d["settings"], path + ".settings", _setting),
                   _string(d["prompt_identity"], path + ".prompt_identity"),
                   _strings(d["permissions"], path + ".permissions"), Budget.from_dict(d["budget"], path + ".budget"),
                   _mapping(d["interventions"], path + ".interventions", _setting))


@dataclass(frozen=True)
class Arm(Record):
    arm_id: str
    interventions: dict[str, str | int | float | bool | None]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "arm_id interventions")
        return cls(_string(d["arm_id"], path + ".arm_id"), _mapping(d["interventions"], path + ".interventions", _setting))


@dataclass(frozen=True)
class PlannedTrial(Record):
    trial_id: str
    arm_id: str
    corpus_id: str
    task_id: str
    repetition: int
    prompt_identity: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        names = "trial_id arm_id corpus_id task_id repetition prompt_identity"
        d = _object(value, path, names)
        return cls(*(_integer(d[n], path + "." + n, 1) if n == "repetition" else _string(d[n], path + "." + n)
                     for n in names.split()))


@dataclass(frozen=True)
class ExperimentManifest(Record):
    schema_version: int
    experiment_id: str
    corpus: tuple[CorpusEntry, ...]
    controls: TrialConditions
    allowed_interventions: tuple[str, ...]
    arms: tuple[Arm, ...]
    planned_trials: tuple[PlannedTrial, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version experiment_id corpus controls allowed_interventions arms planned_trials")
        version = _version(d["schema_version"], path + ".schema_version")
        corpus = _sequence(d["corpus"], CorpusEntry.from_dict, path + ".corpus", lambda x: x.corpus_id)
        controls = TrialConditions.from_dict(d["controls"], path + ".controls")
        allowed = _strings(d["allowed_interventions"], path + ".allowed_interventions")
        arms = _sequence(d["arms"], Arm.from_dict, path + ".arms", lambda x: x.arm_id)
        trials = _sequence(d["planned_trials"], PlannedTrial.from_dict, path + ".planned_trials", lambda x: x.trial_id)
        for name, items in (("corpus", corpus), ("arms", arms), ("planned_trials", trials)):
            if not items:
                _fail(path + "." + name, "must not be empty")
        for i, arm in enumerate(arms):
            if set(arm.interventions) != set(allowed):
                _fail(f"{path}.arms[{i}].interventions", "each arm must declare exactly the allowed intervention fields")
        if set(controls.interventions) != set(allowed):
            _fail(path + ".controls.interventions", "control interventions must match declared allowed fields")
        seen = set()
        for i, trial in enumerate(trials):
            for name, ids in (("arm_id", {a.arm_id for a in arms}), ("corpus_id", {c.corpus_id for c in corpus})):
                if getattr(trial, name) not in ids:
                    _fail(f"{path}.planned_trials[{i}].{name}", "unknown manifest reference")
            identity = (trial.arm_id, trial.corpus_id, trial.task_id, trial.repetition)
            if identity in seen:
                _fail(f"{path}.planned_trials[{i}]", "duplicate arm/corpus/task/repetition identity")
            seen.add(identity)
        return cls(version, _string(d["experiment_id"], path + ".experiment_id"), corpus, controls, allowed, arms, trials)


@dataclass(frozen=True)
class TrialRecord(Record):
    """Producer-supplied conditions; validation does not certify their truth."""

    schema_version: int
    trial_id: str
    arm_id: str
    corpus_id: str
    task_id: str
    repetition: int
    starting_source_hash: str
    run_artifact: str | None
    conditions: TrialConditions
    execution_order: int
    seed: int | None
    completion_state: Literal["completed", "failed", "incomplete", "budget_exhausted", "missing"]
    budget_exhausted: bool
    telemetry: Telemetry

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        names = "schema_version trial_id arm_id corpus_id task_id repetition starting_source_hash run_artifact conditions execution_order seed completion_state budget_exhausted telemetry"
        d = _object(value, path, names)
        state = _choice(d["completion_state"], path + ".completion_state", ("completed", "failed", "incomplete", "budget_exhausted", "missing"))
        exhausted = _boolean(d["budget_exhausted"], path + ".budget_exhausted")
        run = _nullable(d["run_artifact"], _relative, path + ".run_artifact")
        if state == "completed" and run is None:
            _fail(path + ".run_artifact", "completed trial requires run evidence reference")
        if state == "budget_exhausted" and not exhausted:
            _fail(path + ".budget_exhausted", "exhausted completion requires exhaustion flag")
        seed = d["seed"]
        if seed is not None and type(seed) is not int:
            _fail(path + ".seed", "expected integer or null")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   *(_string(d[n], path + "." + n) for n in ("trial_id", "arm_id", "corpus_id", "task_id")),
                   _integer(d["repetition"], path + ".repetition", 1), _hash(d["starting_source_hash"], path + ".starting_source_hash"),
                   run, TrialConditions.from_dict(d["conditions"], path + ".conditions"),
                   _integer(d["execution_order"], path + ".execution_order", 1), seed, state, exhausted,
                   Telemetry.from_dict(d["telemetry"], path + ".telemetry"))


@dataclass(frozen=True)
class TrialOutcome(Record):
    trial_id: str
    decision: Decision
    baseline_status: Literal["passed", "ineligible", "invalid", "missing"]
    required_behavior_status: Literal["passed", "failed", "incomplete"]
    confounded_differences: tuple[str, ...]
    reasons: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "trial_id decision baseline_status required_behavior_status confounded_differences reasons")
        return cls(_string(d["trial_id"], path + ".trial_id"), _choice(d["decision"], path + ".decision", DECISIONS),
                   _choice(d["baseline_status"], path + ".baseline_status", ("passed", "ineligible", "invalid", "missing")),
                   _choice(d["required_behavior_status"], path + ".required_behavior_status", ("passed", "failed", "incomplete")),
                   _strings(d["confounded_differences"], path + ".confounded_differences"),
                   _sequence(d["reasons"], _string, path + ".reasons"))


@dataclass(frozen=True)
class Distribution(Record):
    """Descriptive statistics only; null values preserve undefined populations."""

    key: str
    unit: str
    n: int
    mean: float | None
    median: float | None
    minimum: float | None
    maximum: float | None
    sample_stddev: float | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "key unit n mean median minimum maximum sample_stddev")
        n = _integer(d["n"], path + ".n")
        numbers = tuple(_nullable(d[k], _number, path + "." + k) for k in ("mean", "median", "minimum", "maximum"))
        stddev = None if d["sample_stddev"] is None else _number(d["sample_stddev"], path + ".sample_stddev", 0)
        if (n == 0 and any(x is not None for x in numbers)) or (n > 0 and any(x is None for x in numbers)):
            _fail(path + ".n", "empty samples require null statistics; nonempty samples require statistics")
        if (n < 2) != (stddev is None):
            _fail(path + ".sample_stddev", "sample standard deviation is null exactly when n < 2")
        if n and not (numbers[2] <= numbers[0] <= numbers[3] and numbers[2] <= numbers[1] <= numbers[3]):
            _fail(path + ".minimum", "mean and median must lie within min/max")
        return cls(_string(d["key"], path + ".key"), _string(d["unit"], path + ".unit"), n, *numbers, stddev)


@dataclass(frozen=True)
class ArmSummary(Record):
    arm_id: str
    planned_attempts: int
    state_counts: dict[str, int]
    eligible_attempts: int
    correct_attempts: int
    budget_exhausted_attempts: int
    all_attempt_structural: tuple[Distribution, ...]
    accepted_only_structural: tuple[Distribution, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "arm_id planned_attempts state_counts eligible_attempts correct_attempts budget_exhausted_attempts all_attempt_structural accepted_only_structural")
        count = _integer(d["planned_attempts"], path + ".planned_attempts")
        counts = _mapping(_object(d["state_counts"], path + ".state_counts", " ".join(DECISIONS)), path + ".state_counts", _integer)
        eligible = _integer(d["eligible_attempts"], path + ".eligible_attempts")
        correct = _integer(d["correct_attempts"], path + ".correct_attempts")
        exhausted = _integer(d["budget_exhausted_attempts"], path + ".budget_exhausted_attempts")
        if sum(counts.values()) != count:
            _fail(path + ".state_counts", "state counts must cover planned attempts")
        if correct > eligible or eligible > count or exhausted > count:
            _fail(path + ".eligible_attempts", "correct <= eligible <= planned and exhausted <= planned required")
        return cls(_string(d["arm_id"], path + ".arm_id"), count, counts, eligible, correct, exhausted,
                   _sequence(d["all_attempt_structural"], Distribution.from_dict, path + ".all_attempt_structural", lambda x: x.key),
                   _sequence(d["accepted_only_structural"], Distribution.from_dict, path + ".accepted_only_structural", lambda x: x.key))


@dataclass(frozen=True)
class PairedSummary(Record):
    baseline_arm: str
    candidate_arm: str
    differences: tuple[Distribution, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "baseline_arm candidate_arm differences")
        before = _string(d["baseline_arm"], path + ".baseline_arm")
        after = _string(d["candidate_arm"], path + ".candidate_arm")
        if before == after:
            _fail(path + ".candidate_arm", "paired arms must differ")
        return cls(before, after, _sequence(d["differences"], Distribution.from_dict, path + ".differences", lambda x: x.key))


@dataclass(frozen=True)
class ExperimentReport(Record):
    schema_version: int
    manifest: ExperimentManifest
    trials: tuple[TrialRecord, ...]
    extra_trials: tuple[TrialRecord, ...]
    outcomes: tuple[TrialOutcome, ...]
    arm_summaries: tuple[ArmSummary, ...]
    paired_summaries: tuple[PairedSummary, ...]
    limitations: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version manifest trials extra_trials outcomes arm_summaries paired_summaries limitations")
        version = _version(d["schema_version"], path + ".schema_version")
        manifest = ExperimentManifest.from_dict(d["manifest"], path + ".manifest")
        trials = _sequence(d["trials"], TrialRecord.from_dict, path + ".trials", lambda x: x.trial_id)
        extras = _sequence(d["extra_trials"], TrialRecord.from_dict, path + ".extra_trials", lambda x: x.trial_id)
        outcomes = _sequence(d["outcomes"], TrialOutcome.from_dict, path + ".outcomes", lambda x: x.trial_id)
        summaries = _sequence(d["arm_summaries"], ArmSummary.from_dict, path + ".arm_summaries", lambda x: x.arm_id)
        paired = _sequence(d["paired_summaries"], PairedSummary.from_dict, path + ".paired_summaries", lambda x: (x.baseline_arm, x.candidate_arm))
        planned_ids = {t.trial_id for t in manifest.planned_trials}
        arm_ids = {a.arm_id for a in manifest.arms}
        if any(t.trial_id not in planned_ids for t in trials):
            _fail(path + ".trials", "unplanned trial belongs in extra_trials")
        if any(t.trial_id in planned_ids for t in extras):
            _fail(path + ".extra_trials", "extra trial cannot reuse a planned identity")
        if {o.trial_id for o in outcomes} != planned_ids:
            _fail(path + ".outcomes", "each scheduled attempt requires exactly one outcome, including missing trials")
        if {s.arm_id for s in summaries} != arm_ids:
            _fail(path + ".arm_summaries", "each declared arm requires a summary")
        for pair in paired:
            if pair.baseline_arm not in arm_ids or pair.candidate_arm not in arm_ids:
                _fail(path + ".paired_summaries", "unknown arm reference")
        return cls(version, manifest, trials, extras, outcomes, summaries, paired,
                   _sequence(d["limitations"], _string, path + ".limitations"))


def _unique_json_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            _fail("$input." + key, "duplicate JSON object key")
        result[key] = value
    return result


def loads(record_type: type[Record], text: str | bytes) -> Record:
    """Read a declared record type; never guess type from imported content."""
    if not isinstance(record_type, type) or not issubclass(record_type, Record) or record_type is Record:
        raise TypeError("record_type must be an evaluation record class")
    try:
        if isinstance(text, bytes):
            text = text.decode("utf-8")
        value = json.loads(text, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise ContractError(f"$input: invalid UTF-8 JSON: {exc}") from exc
    return record_type.from_dict(value)


def dumps(record: Record) -> str:
    """Validate then serialize stable UTF-8 JSON without NaN or infinity."""
    if not isinstance(record, Record) or type(record) is Record:
        raise TypeError("expected an evaluation record")
    value = asdict(record)
    type(record).from_dict(value)
    return json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":"))

