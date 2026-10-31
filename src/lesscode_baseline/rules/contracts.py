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

from lesscode_baseline.contracts import ContractError, MetricEvidence, Record, RunReport, SourceSpan, dumps, loads
from lesscode_baseline.snapshot import matches

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


def _literal_path(value, path: str) -> str:
    value = _relative(value, path)
    if any(c in value for c in "*?[]"):
        _fail(path, "expected literal path, not a pattern")
    return value


def _span(value, path: str) -> SourceSpan:
    span = SourceSpan.from_dict(value, path)
    _literal_path(span.path, path + ".path")
    return span


@dataclass(frozen=True)
class EntityObservation(Record):
    """One selected scalar at an exact inventory address; null never means zero."""

    inventory_id: str
    source_hash: str
    selector: Selector
    metric: MetricEvidence
    location: SourceSpan | None
    long_name: str | None
    evidence_refs: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "inventory_id source_hash selector metric location long_name evidence_refs")
        selector = Selector.from_dict(d["selector"], path + ".selector")
        metric = MetricEvidence.from_dict(d["metric"], path + ".metric")
        if metric.value is not None:
            _number(metric.value, path + ".metric.value")
        location = None if d["location"] is None else _span(d["location"], path + ".location")
        name = None if d["long_name"] is None else _string(d["long_name"], path + ".long_name")
        if selector.kind == "function_ccn":
            if location is None or name is None:
                _fail(path, "function_ccn requires location and long_name")
        elif location is not None or name is not None:
            _fail(path, "aggregate selector requires null location and long_name")
        if selector.kind == "scalar_metric" and selector.metric_key != metric.key:
            _fail(path + ".selector.metric_key", "does not match metric key")
        refs = _sequence(d["evidence_refs"], _literal_path, path + ".evidence_refs", lambda x: x)
        if not refs:
            _fail(path + ".evidence_refs", "at least one evidence address is required")
        return cls(_entity_id(d["inventory_id"], path + ".inventory_id"),
                   _hash(d["source_hash"], path + ".source_hash"), selector, metric, location, name, refs)


@dataclass(frozen=True)
class EvidenceInventory(Record):
    """An explicit complete or unavailable inventory, including known empty ones."""

    source_hash: str
    status: Literal["complete", "unavailable"]
    reason: str | None
    entities: tuple[EntityObservation, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "source_hash status reason entities")
        source = _hash(d["source_hash"], path + ".source_hash")
        status = _choice(d["status"], path + ".status", ("complete", "unavailable"))
        reason = None if d["reason"] is None else _string(d["reason"], path + ".reason")
        entities = _sequence(d["entities"], EntityObservation.from_dict, path + ".entities", lambda e: e.inventory_id)
        if (status == "complete" and reason is not None) or (status == "unavailable" and (reason is None or entities)):
            _fail(path, "complete inventory requires null reason; unavailable requires reason and no entities")
        addresses = set()
        for entity in entities:
            if entity.source_hash != source:
                _fail(path + ".entities", "entity source_hash differs from inventory")
            address = (entity.selector, entity.metric.key, entity.location, entity.long_name)
            if address in addresses:
                _fail(path + ".entities", "duplicate entity address under different inventory IDs")
            addresses.add(address)
        return cls(source, status, reason, entities)


@dataclass(frozen=True)
class EvidenceBundle(Record):
    """In-memory seam; parsing establishes consistency, not artifact admission.

    The reader must separately recover indexed artifacts and bind the prepared
    plan/profile before treating this bundle as admitted evidence.
    """

    schema_version: int
    report: RunReport
    profile_hash: str
    baseline_inventory: EvidenceInventory
    candidate_inventory: EvidenceInventory

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version report profile_hash baseline_inventory candidate_inventory")
        if type(d["schema_version"]) is not int or d["schema_version"] != 1:
            _fail(path + ".schema_version", "unsupported schema_version (expected integer 1)")
        report = RunReport.from_dict(d["report"], path + ".report")
        baseline = EvidenceInventory.from_dict(d["baseline_inventory"], path + ".baseline_inventory")
        candidate = EvidenceInventory.from_dict(d["candidate_inventory"], path + ".candidate_inventory")
        for side, inventory, observation in (("baseline", baseline, report.baseline), ("candidate", candidate, report.candidate)):
            context = path + "." + side + "_inventory"
            if inventory.source_hash != observation.snapshot.content_hash:
                _fail(context + ".source_hash", "does not match report snapshot")
            product_paths = {f.path for f in observation.snapshot.files if f.category == "product"}
            for entity in inventory.entities:
                if entity.metric.scope_hash != observation.snapshot.scope_hash:
                    _fail(context + ".entities", "metric scope_hash differs from snapshot")
                if entity.location is not None and entity.location.path not in product_paths:
                    _fail(context + ".entities", "location is absent from product inventory")
            for check in observation.checks:
                if check.source_hash != inventory.source_hash or check.plan_hash != report.plan_hash:
                    _fail(path + ".report." + side + ".checks", "check source/plan binding differs from report")
        return cls(1, report, _hash(d["profile_hash"], path + ".profile_hash"), baseline, candidate)


@dataclass(frozen=True)
class EntityTreatment(Record):
    """Producer continuity claim, never evidence of semantic equivalence."""

    policy_hash: str
    baseline_source_hash: str
    candidate_source_hash: str
    kind: Literal["matched", "new", "removed", "unresolved"]
    baseline_id: str | None
    candidate_id: str | None
    producer_id: str
    rationale: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "policy_hash baseline_source_hash candidate_source_hash kind baseline_id candidate_id producer_id rationale")
        kind = _choice(d["kind"], path + ".kind", ("matched", "new", "removed", "unresolved"))
        baseline = None if d["baseline_id"] is None else _entity_id(d["baseline_id"], path + ".baseline_id")
        candidate = None if d["candidate_id"] is None else _entity_id(d["candidate_id"], path + ".candidate_id")
        sides = (baseline is not None, candidate is not None)
        if ((kind == "matched" and sides != (True, True))
                or (kind == "new" and sides != (False, True))
                or (kind == "removed" and sides != (True, False))
                or (kind == "unresolved" and sides == (False, False))):
            _fail(path, "treatment kind contradicts baseline_id/candidate_id")
        return cls(*(_hash(d[key], path + "." + key) for key in
                     ("policy_hash", "baseline_source_hash", "candidate_source_hash")),
                   kind, baseline, candidate, _string(d["producer_id"], path + ".producer_id"),
                   _string(d["rationale"], path + ".rationale"))


def validate_treatments(records: tuple[EntityTreatment, ...], evidence: EvidenceBundle,
                        policy: Policy) -> tuple[EntityTreatment, ...]:
    """Validate supplied exact IDs only; omitted treatments remain omitted."""
    evidence = EvidenceBundle.from_dict(evidence.to_dict())
    policy = Policy.from_dict(policy.to_dict())
    expected_policy = policy_hash(policy)
    if policy.reference is not None and (
            policy.reference.source_hash != evidence.baseline_inventory.source_hash
            or policy.reference.profile_hash != evidence.profile_hash):
        _fail("$treatments", "policy reference differs from evidence source/profile")
    records = _sequence(records, lambda r, p: EntityTreatment.from_dict(r.to_dict(), p), "$treatments")
    baseline_ids = {e.inventory_id: e for e in evidence.baseline_inventory.entities}
    candidate_ids = {e.inventory_id: e for e in evidence.candidate_inventory.entities}
    seen_baseline, seen_candidate = set(), set()
    for record in records:
        if (record.policy_hash != expected_policy
                or record.baseline_source_hash != evidence.baseline_inventory.source_hash
                or record.candidate_source_hash != evidence.candidate_inventory.source_hash):
            _fail("$treatments", "policy/source identity differs from evidence")
        for identity, known, seen in ((record.baseline_id, baseline_ids, seen_baseline),
                                      (record.candidate_id, candidate_ids, seen_candidate)):
            if identity is None:
                continue
            if identity not in known:
                _fail("$treatments", "absent inventory ID " + identity)
            if identity in seen:
                _fail("$treatments", "duplicate or contradictory treatment for " + identity)
            seen.add(identity)
        if record.kind == "matched":
            baseline = baseline_ids[record.baseline_id]
            candidate = candidate_ids[record.candidate_id]
            if baseline.selector != candidate.selector or baseline.metric.key != candidate.metric.key:
                _fail("$treatments", "matched entities have different metric selectors")
    return records


@dataclass(frozen=True)
class ReviewRecord(Record):
    """Disclosed local producer assertion, with no identity authentication."""

    policy_hash: str
    candidate_source_hash: str
    rule_id: str
    criterion_id: str
    scope: PolicyScope
    reviewer_id: str
    outcome: Literal["satisfied", "concern", "unresolved"]
    rationale: str
    locations: tuple[SourceSpan, ...]
    provenance: Literal["producer_supplied"]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "policy_hash candidate_source_hash rule_id criterion_id scope reviewer_id outcome rationale locations provenance")
        locations = _sequence(d["locations"], _span, path + ".locations", lambda s: s)
        if not locations:
            _fail(path + ".locations", "at least one supporting code span is required")
        return cls(_hash(d["policy_hash"], path + ".policy_hash"),
                   _hash(d["candidate_source_hash"], path + ".candidate_source_hash"),
                   _string(d["rule_id"], path + ".rule_id"), _string(d["criterion_id"], path + ".criterion_id"),
                   PolicyScope.from_dict(d["scope"], path + ".scope"),
                   _string(d["reviewer_id"], path + ".reviewer_id"),
                   _choice(d["outcome"], path + ".outcome", ("satisfied", "concern", "unresolved")),
                   _string(d["rationale"], path + ".rationale"), locations,
                   _choice(d["provenance"], path + ".provenance", ("producer_supplied",)))

    def trust_status(self, requirement: Literal["producer_record", "external_independence"]) -> Literal["supported", "unverified"]:
        _choice(requirement, "$review.trust_requirement", ("producer_record", "external_independence"))
        return "supported" if requirement == "producer_record" else "unverified"


def _in_scope(filename: str, scope: PolicyScope) -> bool:
    if scope.root != ".":
        prefix = scope.root + "/"
        if not filename.startswith(prefix):
            return False
        filename = filename[len(prefix):]
    return (any(matches(filename, pattern) for pattern in scope.include)
            and not any(matches(filename, pattern) for pattern in scope.exclude))


def validate_reviews(records: tuple[ReviewRecord, ...], evidence: EvidenceBundle,
                     policy: Policy) -> tuple[ReviewRecord, ...]:
    """Check structural bindings; missing reviews and trust remain explicit."""
    evidence = EvidenceBundle.from_dict(evidence.to_dict())
    policy = Policy.from_dict(policy.to_dict())
    expected_policy = policy_hash(policy)
    records = _sequence(records, lambda r, p: ReviewRecord.from_dict(r.to_dict(), p), "$reviews")
    rules = {rule.rule_id: rule for rule in policy.rules if isinstance(rule, ReviewRule)}
    product_paths = {f.path for f in evidence.report.candidate.snapshot.files if f.category == "product"}
    seen = set()
    for record in records:
        if record.policy_hash != expected_policy or record.candidate_source_hash != evidence.candidate_inventory.source_hash:
            _fail("$reviews", "policy/source identity differs from evidence")
        rule = rules.get(record.rule_id)
        if rule is None or rule.criterion_id != record.criterion_id or rule.scope != record.scope:
            _fail("$reviews", "rule/criterion/scope differs from applicable review rule")
        identity = (record.criterion_id, record.scope)
        if identity in seen:
            _fail("$reviews", "duplicate or conflicting criterion/scope review")
        seen.add(identity)
        for span in record.locations:
            if (span.path not in product_paths or not _in_scope(span.path, record.scope)
                    or not _in_scope(span.path, policy.target_scope)):
                _fail("$reviews.locations", "span is outside candidate product inventory or applicable scope")
    return records
