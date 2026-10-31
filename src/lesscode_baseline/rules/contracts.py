"""Strict version-1 preventive policy inputs, independent of baseline formats.

Readers perform no IO. Explicit bounds and evidence identities remain policy
data; these records neither evaluate observations nor authenticate reviews.
"""

from dataclasses import dataclass
from datetime import datetime, timedelta
import hashlib
import math
import re
from typing import Literal, Self

from lesscode_baseline.contracts import ContractError, Record, dumps, loads

def _fail(path: str, reason: str):
    raise ContractError(f"{path}: {reason}")


def _object(value, path: str, fields: str) -> dict:
    if not isinstance(value, dict):
        _fail(path, "expected object")
    for key in value:
        if key not in fields.split():
            _fail(f"{path}.{key}", "unknown field")
    for key in fields.split():
        if key not in value:
            _fail(f"{path}.{key}", "required field missing")
    return value


def _string(value, path: str) -> str:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        _fail(path, "expected nonempty string without NUL")
    return value


def _choice(value, path: str, choices: tuple[str, ...]) -> str:
    if not isinstance(value, str) or value not in choices:
        _fail(path, f"expected one of {', '.join(choices)}")
    return value


def _number(value, path: str) -> int | float:
    if type(value) not in (int, float):
        _fail(path, "expected finite nonnegative number")
    try:
        valid = math.isfinite(value) and value >= 0
    except OverflowError:
        valid = False
    if not valid:
        _fail(path, "expected finite nonnegative number")
    return value


def _hash(value, path: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        _fail(path, "expected lowercase SHA-256 fingerprint")
    return value


def _relative(value, path: str, *, root=False) -> str:
    value = _string(value, path)
    if root and value == ".":
        return value
    if ("\\" in value or ":" in value
            or any(p in ("", ".", "..") or p.endswith((" ", ".")) for p in value.split("/"))):
        _fail(path, "expected relative POSIX path without traversal or normalized components")
    if root and any(c in value for c in "*?[]"):
        _fail(path, "root must be a literal relative path")
    return value


def _sequence(value, reader, path: str, identity=None) -> tuple:
    if not isinstance(value, (list, tuple)):
        _fail(path, "expected array")
    records = tuple(reader(item, f"{path}[{i}]") for i, item in enumerate(value))
    if identity is not None:
        seen = set()
        for i, record in enumerate(records):
            key = identity(record)
            if key in seen:
                _fail(f"{path}[{i}]", f"duplicate identity {key!r}")
            seen.add(key)
    return records


def _utc(value, path: str) -> str:
    value = _string(value, path)
    try:
        instant = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        _fail(path, "expected UTC ISO-8601 timestamp")
    if instant.utcoffset() != timedelta(0):
        _fail(path, "expected explicit UTC timezone")
    return value


@dataclass(frozen=True)
class PolicyScope(Record):
    """Root-relative POSIX, case-sensitive globs using baseline semantics."""

    root: str
    include: tuple[str, ...]
    exclude: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "root include exclude")
        return cls(_relative(d["root"], path + ".root", root=True),
                   _sequence(d["include"], _relative, path + ".include", lambda x: x),
                   _sequence(d["exclude"], _relative, path + ".exclude", lambda x: x))


@dataclass(frozen=True)
class PolicyReference(Record):
    source_hash: str
    profile_hash: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "source_hash profile_hash")
        return cls(_hash(d["source_hash"], path + ".source_hash"),
                   _hash(d["profile_hash"], path + ".profile_hash"))


@dataclass(frozen=True)
class Selector(Record):
    kind: Literal["function_ccn", "scalar_metric", "duplication_clone_pairs",
                  "duplication_duplicated_tokens", "duplication_total_tokens", "duplication_pair_span_ratio"]
    metric_key: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind metric_key")
        kind = _choice(d["kind"], path + ".kind", ("function_ccn", "scalar_metric", "duplication_clone_pairs",
                       "duplication_duplicated_tokens", "duplication_total_tokens", "duplication_pair_span_ratio"))
        key = d["metric_key"]
        if kind == "scalar_metric":
            key = _string(key, path + ".metric_key")
        elif key is not None:
            _fail(path + ".metric_key", "only scalar_metric accepts a metric key")
        return cls(kind, key)


@dataclass(frozen=True)
class EvidenceExpectation(Record):
    definition_id: str
    producer: str
    producer_version: str
    unit: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        fields = "definition_id producer producer_version unit"
        d = _object(value, path, fields)
        return cls(*(_string(d[key], f"{path}.{key}") for key in fields.split()))


@dataclass(frozen=True)
class ChangeBudget(Record):
    kind: Literal["absolute_increase", "percentage_increase"]
    maximum: int | float

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind maximum")
        return cls(_choice(d["kind"], path + ".kind", ("absolute_increase", "percentage_increase")),
                   _number(d["maximum"], path + ".maximum"))


def _rule_fields(d: dict, path: str, kind: str) -> tuple:
    return (_choice(d["kind"], path + ".kind", (kind,)),
            _string(d["rule_id"], path + ".rule_id"),
            PolicyScope.from_dict(d["scope"], path + ".scope"),
            _choice(d["enforcement"], path + ".enforcement", ("blocking", "advisory")),
            _string(d["description"], path + ".description"))


@dataclass(frozen=True)
class NumericRule(Record):
    kind: Literal["numeric"]
    rule_id: str
    scope: PolicyScope
    enforcement: Literal["blocking", "advisory"]
    description: str
    selector: Selector
    evidence: EvidenceExpectation
    mode: Literal["absolute", "ratchet"]
    absolute_maximum: int | float | None
    change_budget: ChangeBudget | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind rule_id scope enforcement description selector evidence mode absolute_maximum change_budget")
        common = _rule_fields(d, path, "numeric")
        mode = _choice(d["mode"], path + ".mode", ("absolute", "ratchet"))
        maximum = None if d["absolute_maximum"] is None else _number(d["absolute_maximum"], path + ".absolute_maximum")
        budget = None if d["change_budget"] is None else ChangeBudget.from_dict(d["change_budget"], path + ".change_budget")
        if mode == "ratchet" and maximum is None:
            _fail(path + ".absolute_maximum", "ratchet requires an absolute maximum")
        if maximum is None and budget is None:
            _fail(path, "at least one explicit bound is required")
        return cls(*common, Selector.from_dict(d["selector"], path + ".selector"),
                   EvidenceExpectation.from_dict(d["evidence"], path + ".evidence"), mode, maximum, budget)


@dataclass(frozen=True)
class CheckRule(Record):
    kind: Literal["check"]
    rule_id: str
    scope: PolicyScope
    enforcement: Literal["blocking", "advisory"]
    description: str
    check_id: str
    expected: Literal["passed"]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind rule_id scope enforcement description check_id expected")
        return cls(*_rule_fields(d, path, "check"), _string(d["check_id"], path + ".check_id"),
                   _choice(d["expected"], path + ".expected", ("passed",)))


@dataclass(frozen=True)
class ReviewRule(Record):
    kind: Literal["review"]
    rule_id: str
    scope: PolicyScope
    enforcement: Literal["blocking", "advisory"]
    description: str
    criterion_id: str
    criterion_text: str
    trust_requirement: Literal["producer_record", "external_independence"]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "kind rule_id scope enforcement description criterion_id criterion_text trust_requirement")
        return cls(*_rule_fields(d, path, "review"), _string(d["criterion_id"], path + ".criterion_id"),
                   _string(d["criterion_text"], path + ".criterion_text"),
                   _choice(d["trust_requirement"], path + ".trust_requirement", ("producer_record", "external_independence")))


Rule = NumericRule | CheckRule | ReviewRule


def _rule(value, path: str) -> Rule:
    if not isinstance(value, dict):
        _fail(path, "expected rule object")
    rule_id = _string(value.get("rule_id"), path + ".rule_id")
    path = f"{path}({rule_id})"
    kind = _choice(value.get("kind"), path + ".kind", ("numeric", "check", "review"))
    return {"numeric": NumericRule, "check": CheckRule, "review": ReviewRule}[kind].from_dict(value, path)


def _entity_id(value, path: str) -> str:
    value = _string(value, path)
    if any(c in value for c in "*?[]"):
        _fail(path, "entity target must be an exact identity, not a wildcard")
    return value


@dataclass(frozen=True)
class Exception(Record):
    """Bounded numeric waiver; empty entity_ids targets the declared scope."""

    exception_id: str
    rule_id: str
    scope: PolicyScope
    entity_ids: tuple[str, ...]
    justification: str
    valid_from: str
    valid_until: str
    candidate_source_hash: str | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "exception_id rule_id scope entity_ids justification valid_from valid_until candidate_source_hash")
        start = _utc(d["valid_from"], path + ".valid_from")
        end = _utc(d["valid_until"], path + ".valid_until")
        if datetime.fromisoformat(start) >= datetime.fromisoformat(end):
            _fail(path + ".valid_until", "must be after inclusive valid_from (valid_until is exclusive)")
        source_hash = d["candidate_source_hash"]
        return cls(_string(d["exception_id"], path + ".exception_id"), _string(d["rule_id"], path + ".rule_id"),
                   PolicyScope.from_dict(d["scope"], path + ".scope"),
                   _sequence(d["entity_ids"], _entity_id, path + ".entity_ids", lambda x: x),
                   _string(d["justification"], path + ".justification"), start, end,
                   None if source_hash is None else _hash(source_hash, path + ".candidate_source_hash"))


@dataclass(frozen=True)
class Policy(Record):
    schema_version: int
    policy_id: str
    target_scope: PolicyScope
    reference: PolicyReference | None
    rules: tuple[Rule, ...]
    exceptions: tuple[Exception, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version policy_id target_scope reference rules exceptions")
        if type(d["schema_version"]) is not int or d["schema_version"] != 1:
            _fail(path + ".schema_version", "unsupported schema_version (expected integer 1)")
        reference = None if d["reference"] is None else PolicyReference.from_dict(d["reference"], path + ".reference")
        rules = _sequence(d["rules"], _rule, path + ".rules", lambda r: r.rule_id)
        for rule in rules:
            if isinstance(rule, NumericRule) and (rule.mode == "ratchet" or rule.change_budget is not None) and reference is None:
                _fail(f"{path}.rules({rule.rule_id}).reference", "rule requires explicit source/profile reference")
        exceptions = _sequence(d["exceptions"], Exception.from_dict, path + ".exceptions", lambda e: e.exception_id)
        numeric_ids = {r.rule_id for r in rules if isinstance(r, NumericRule)}
        for exception in exceptions:
            if exception.rule_id not in numeric_ids:
                _fail(f"{path}.exceptions({exception.exception_id}).rule_id", "must target a known numeric rule")
        return cls(1, _string(d["policy_id"], path + ".policy_id"),
                   PolicyScope.from_dict(d["target_scope"], path + ".target_scope"), reference,
                   tuple(sorted(rules, key=lambda r: r.rule_id)),
                   tuple(sorted(exceptions, key=lambda e: e.exception_id)))


def policy_hash(policy: Policy) -> str:
    """Hash validated canonical policy; raw input bytes have a separate identity."""
    if not isinstance(policy, Policy):
        raise TypeError("expected Policy")
    canonical = Policy.from_dict(policy.to_dict())
    return hashlib.sha256(dumps(canonical).encode("utf-8")).hexdigest()
