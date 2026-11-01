"""Strict version-1 preventive policy inputs, independent of baseline formats.

Readers perform no IO. Explicit bounds and evidence identities remain policy
data; these records neither evaluate observations nor authenticate reviews.
"""

from dataclasses import dataclass
from datetime import datetime, timedelta
import hashlib
import math
from typing import Literal, Self

from lesscode_baseline.contracts import DECISIONS, Decision, ContractError, MetricEvidence, Record, RunReport, SourceSpan, dumps, loads
from lesscode_baseline.contracts import (
    _fail as _baseline_fail, _object as _baseline_object, _choice as _baseline_choice,
    _string, _hash, _sequence,
)
from lesscode_baseline.snapshot import matches


def _fail(path: str, reason: str):
    return _baseline_fail(path, reason)


def _object(value, path: str, fields: str) -> dict:
    return _baseline_object(value, path, fields)


def _choice(value, path: str, choices: tuple[str, ...]) -> str:
    return _baseline_choice(value, path, choices)


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
class RuleSelection(Record):
    """Per-rule availability and compatibility, distinct from entity continuity."""

    schema_version: int
    rule_id: str
    baseline: EvidenceInventory | None
    candidate: EvidenceInventory
    comparison_status: Literal["not_required", "comparable", "unavailable"]
    comparison_reasons: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "schema_version rule_id baseline candidate comparison_status comparison_reasons")
        if type(d["schema_version"]) is not int or d["schema_version"] != 1:
            _fail(path + ".schema_version", "unsupported schema_version (expected integer 1)")
        before = None if d["baseline"] is None else EvidenceInventory.from_dict(d["baseline"], path + ".baseline")
        after = EvidenceInventory.from_dict(d["candidate"], path + ".candidate")
        status = _choice(d["comparison_status"], path + ".comparison_status", ("not_required", "comparable", "unavailable"))
        reasons = _sequence(d["comparison_reasons"], _string, path + ".comparison_reasons", lambda x: x)
        if status == "unavailable":
            if before is None or not reasons:
                _fail(path, "unavailable comparison requires baseline inventory and reasons")
        elif reasons:
            _fail(path + ".comparison_reasons", "supported comparison cannot have failure reasons")
        if status == "comparable" and (before is None or before.status != "complete" or after.status != "complete"):
            _fail(path, "comparable requires complete baseline and candidate inventories")
        return cls(1, _string(d["rule_id"], path + ".rule_id"), before, after, status, reasons)


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


def _artifact(value, path: str, fields: str) -> dict:
    d = _object(value, path, "schema_version " + fields)
    if type(d["schema_version"]) is not int or d["schema_version"] != 1:
        _fail(path + ".schema_version", "unsupported schema_version (expected integer 1)")
    return d


def _absolute_path(value, path: str) -> str:
    from pathlib import PurePosixPath, PureWindowsPath
    value = _string(value, path)
    parsed = PureWindowsPath(value) if PureWindowsPath(value).drive else PurePosixPath(value)
    if not parsed.is_absolute() or ".." in parsed.parts or any(c in value for c in "*?[]"):
        _fail(path, "expected absolute literal path without traversal")
    return value


@dataclass(frozen=True)
class CapturedResource(Record):
    """Original external byte identity; reading/rechecking belongs to services."""

    role: Literal["policy", "run_index", "run_artifact", "review", "treatment"]
    path: str
    sha256: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "role path sha256")
        return cls(_choice(d["role"], path + ".role", ("policy", "run_index", "run_artifact", "review", "treatment")),
                   _absolute_path(d["path"], path + ".path"), _hash(d["sha256"], path + ".sha256"))


@dataclass(frozen=True)
class PreparedAssessment(Record):
    schema_version: int
    policy: Policy
    policy_hash: str
    evidence: EvidenceBundle
    resources: tuple[CapturedResource, ...]
    reviews: tuple[ReviewRecord, ...]
    treatments: tuple[EntityTreatment, ...]
    as_of: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "policy policy_hash evidence resources reviews treatments as_of")
        policy = Policy.from_dict(d["policy"], path + ".policy")
        identity = _hash(d["policy_hash"], path + ".policy_hash")
        if identity != policy_hash(policy):
            _fail(path + ".policy_hash", "does not match captured policy")
        evidence = EvidenceBundle.from_dict(d["evidence"], path + ".evidence")
        if policy.reference is not None and (policy.reference.source_hash != evidence.baseline_inventory.source_hash
                                             or policy.reference.profile_hash != evidence.profile_hash):
            _fail(path + ".policy.reference", "differs from captured evidence")
        resources = _sequence(d["resources"], CapturedResource.from_dict, path + ".resources", lambda r: r.path)
        roles = [r.role for r in resources]
        if roles.count("policy") != 1 or roles.count("run_index") != 1 or "run_artifact" not in roles:
            _fail(path + ".resources", "requires one policy, one run_index and indexed run artifacts")
        reviews = _sequence(d["reviews"], ReviewRecord.from_dict, path + ".reviews")
        treatments = _sequence(d["treatments"], EntityTreatment.from_dict, path + ".treatments")
        for role, records in (("review", reviews), ("treatment", treatments)):
            if records and role not in roles:
                _fail(path + ".resources", "missing captured " + role + " input")
        # Wrong review bindings remain input evidence for the evaluator to classify
        # as insufficient; preparation does not silently discard those records.
        validate_treatments(treatments, evidence, policy)
        return cls(1, policy, identity, evidence, tuple(sorted(resources, key=lambda r: (r.role, r.path))),
                   reviews, treatments, _utc(d["as_of"], path + ".as_of"))


@dataclass(frozen=True)
class EvidenceReference(Record):
    schema_version: int
    run_index_hash: str
    artifact_path: str
    artifact_hash: str
    record_address: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "run_index_hash artifact_path artifact_hash record_address")
        address = d["record_address"]
        if not isinstance(address, str) or (address != "" and not address.startswith("/")) or re.search(r"~(?![01])", address) or "\x00" in address:
            _fail(path + ".record_address", "expected JSON Pointer")
        return cls(1, _hash(d["run_index_hash"], path + ".run_index_hash"),
                   _literal_path(d["artifact_path"], path + ".artifact_path"),
                   _hash(d["artifact_hash"], path + ".artifact_hash"), address)


@dataclass(frozen=True)
class ExceptionEffect(Record):
    exception_id: str
    state: Literal["active", "expired", "inapplicable"]
    reason: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "exception_id state reason")
        return cls(_string(d["exception_id"], path + ".exception_id"),
                   _choice(d["state"], path + ".state", ("active", "expired", "inapplicable")),
                   _string(d["reason"], path + ".reason"))


@dataclass(frozen=True)
class PolicyFinding(Record):
    rule_id: str
    scope: PolicyScope
    entity_id: str
    basis: str
    enforcement: Literal["blocking", "advisory"]
    status: Literal["satisfied", "violation", "insufficient", "invalid"]
    disposition: Literal["improved", "unchanged", "worsened", "inherited_debt", "new_debt", "worsened_debt", "improved_residual_debt", "resolved_debt", "new", "removed", "unresolved", "unavailable", "check", "review"]
    reference: EntityObservation | None
    candidate: EntityObservation | None
    expected: Rule
    locations: tuple[SourceSpan, ...]
    evidence_refs: tuple[EvidenceReference, ...]
    reason: str
    exceptions: tuple[ExceptionEffect, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "rule_id scope entity_id basis enforcement status disposition reference candidate expected locations evidence_refs reason exceptions")
        expected = _rule(d["expected"], path + ".expected")
        scope = PolicyScope.from_dict(d["scope"], path + ".scope")
        if (d["rule_id"] != expected.rule_id or d["enforcement"] != expected.enforcement or scope != expected.scope):
            _fail(path, "rule_id/scope/enforcement differs from expected rule")
        effects = _sequence(d["exceptions"], ExceptionEffect.from_dict, path + ".exceptions", lambda e: e.exception_id)
        status = _choice(d["status"], path + ".status", ("satisfied", "violation", "insufficient", "invalid"))
        if any(e.state == "active" for e in effects) and (not isinstance(expected, NumericRule) or status != "violation"):
            _fail(path + ".exceptions", "active exceptions require an original numeric violation")
        return cls(expected.rule_id, scope, _entity_id(d["entity_id"], path + ".entity_id"),
                   _string(d["basis"], path + ".basis"), expected.enforcement, status,
                   _choice(d["disposition"], path + ".disposition", ("improved", "unchanged", "worsened", "inherited_debt", "new_debt", "worsened_debt", "improved_residual_debt", "resolved_debt", "new", "removed", "unresolved", "unavailable", "check", "review")),
                   None if d["reference"] is None else EntityObservation.from_dict(d["reference"], path + ".reference"),
                   None if d["candidate"] is None else EntityObservation.from_dict(d["candidate"], path + ".candidate"),
                   expected, _sequence(d["locations"], _span, path + ".locations", lambda s: s),
                   _sequence(d["evidence_refs"], EvidenceReference.from_dict, path + ".evidence_refs", lambda r: r),
                   _string(d["reason"], path + ".reason"), tuple(sorted(effects, key=lambda e: e.exception_id)))


@dataclass(frozen=True)
class PolicyResult(Record):
    schema_version: int
    prepared_hash: str
    baseline_decision: Decision
    baseline_reasons: tuple[str, ...]
    policy_verdict: Literal["satisfied", "violations", "insufficient", "invalid"]
    policy_accepted: bool
    findings: tuple[PolicyFinding, ...]
    diagnostics: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "prepared_hash baseline_decision baseline_reasons policy_verdict policy_accepted findings diagnostics")
        baseline = _choice(d["baseline_decision"], path + ".baseline_decision", DECISIONS)
        verdict = _choice(d["policy_verdict"], path + ".policy_verdict", ("satisfied", "violations", "insufficient", "invalid"))
        accepted = d["policy_accepted"]
        if type(accepted) is not bool or accepted != (baseline == "accepted" and verdict == "satisfied"):
            _fail(path + ".policy_accepted", "must equal accepted baseline and satisfied policy")
        def identity(finding):
            return finding.rule_id, dumps(finding.scope), finding.entity_id, finding.basis
        findings = _sequence(d["findings"], PolicyFinding.from_dict, path + ".findings", identity)
        return cls(1, _hash(d["prepared_hash"], path + ".prepared_hash"), baseline,
                   _sequence(d["baseline_reasons"], _string, path + ".baseline_reasons"), verdict, accepted,
                   tuple(sorted(findings, key=identity)), _sequence(d["diagnostics"], _string, path + ".diagnostics"))


@dataclass(frozen=True)
class PolicyArtifact(Record):
    path: str
    sha256: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _object(value, path, "path sha256")
        return cls(_literal_path(d["path"], path + ".path"), _hash(d["sha256"], path + ".sha256"))


@dataclass(frozen=True)
class PolicyArtifactIndex(Record):
    schema_version: int
    prepared_hash: str
    files: tuple[PolicyArtifact, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "prepared_hash files")
        files = _sequence(d["files"], PolicyArtifact.from_dict, path + ".files", lambda f: f.path)
        names = {f.path for f in files}
        if not {"manifest.json", "prepared.json", "result.json", "summary.md"} <= names or "artifacts.json" in names:
            _fail(path + ".files", "requires manifest/prepared/result/summary and cannot index itself")
        return cls(1, _hash(d["prepared_hash"], path + ".prepared_hash"), tuple(sorted(files, key=lambda f: f.path)))


@dataclass(frozen=True)
class PolicyRecovery(Record):
    schema_version: int
    state: Literal["prepared", "running", "finalized"]
    result: PolicyResult | None
    index: PolicyArtifactIndex | None
    external_evidence: Literal["verified", "unverified"]
    diagnostics: tuple[str, ...]

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "state result index external_evidence diagnostics")
        state = _choice(d["state"], path + ".state", ("prepared", "running", "finalized"))
        result = None if d["result"] is None else PolicyResult.from_dict(d["result"], path + ".result")
        index = None if d["index"] is None else PolicyArtifactIndex.from_dict(d["index"], path + ".index")
        if (state == "finalized" and (result is None or index is None)) or (state != "finalized" and index is not None):
            _fail(path, "finalized requires result and index; partial state cannot carry completion index")
        if index is not None and index.prepared_hash != result.prepared_hash:
            _fail(path + ".index.prepared_hash", "differs from result")
        return cls(1, state, result, index,
                   _choice(d["external_evidence"], path + ".external_evidence", ("verified", "unverified")),
                   _sequence(d["diagnostics"], _string, path + ".diagnostics"))

    @property
    def complete(self) -> bool:
        return self.state == "finalized" and self.external_evidence == "verified"


@dataclass(frozen=True)
class RuleFailure(Record):
    schema_version: int
    operation: Literal["show", "prepare", "check", "guidance-preview", "guidance-apply", "recover"]
    kind: Literal["invalid", "insufficient", "conflict", "io", "internal", "persistence"]
    diagnostics: tuple[str, ...]
    partial_result: PolicyResult | None

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "operation kind diagnostics partial_result")
        diagnostics = _sequence(d["diagnostics"], _string, path + ".diagnostics")
        if not diagnostics:
            _fail(path + ".diagnostics", "failure requires a diagnostic")
        return cls(1, _choice(d["operation"], path + ".operation", ("show", "prepare", "check", "guidance-preview", "guidance-apply", "recover")),
                   _choice(d["kind"], path + ".kind", ("invalid", "insufficient", "conflict", "io", "internal", "persistence")),
                   diagnostics, None if d["partial_result"] is None else PolicyResult.from_dict(d["partial_result"], path + ".partial_result"))


@dataclass(frozen=True)
class EncodedBytes(Record):
    """Canonical base64 preserves BOM/newlines and distinguishes empty from absent."""

    encoding: Literal["base64"]
    data: str
    sha256: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        import base64
        import binascii
        d = _object(value, path, "encoding data sha256")
        _choice(d["encoding"], path + ".encoding", ("base64",))
        try:
            raw = base64.b64decode(d["data"], validate=True)
        except (ValueError, TypeError, binascii.Error):
            _fail(path + ".data", "expected canonical base64 bytes")
        if base64.b64encode(raw).decode("ascii") != d["data"]:
            _fail(path + ".data", "expected canonical base64 bytes")
        digest = _hash(d["sha256"], path + ".sha256")
        if hashlib.sha256(raw).hexdigest() != digest:
            _fail(path + ".sha256", "differs from bytes")
        return cls("base64", d["data"], digest)

    def decode(self) -> bytes:
        import base64
        return base64.b64decode(self.data, validate=True)


@dataclass(frozen=True)
class GuidanceDocument(Record):
    schema_version: int
    policy_hash: str
    scope: PolicyScope
    renderer_version: str
    kind: Literal["pre_change_guidance"]
    content: EncodedBytes

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "policy_hash scope renderer_version kind content")
        return cls(1, _hash(d["policy_hash"], path + ".policy_hash"), PolicyScope.from_dict(d["scope"], path + ".scope"),
                   _string(d["renderer_version"], path + ".renderer_version"),
                   _choice(d["kind"], path + ".kind", ("pre_change_guidance",)),
                   EncodedBytes.from_dict(d["content"], path + ".content"))


@dataclass(frozen=True)
class GuidanceUpdate(Record):
    schema_version: int
    document: GuidanceDocument
    policy_resource: CapturedResource
    root: str
    target: str
    before: EncodedBytes | None
    managed_block_hash: str | None
    proposed: EncodedBytes
    diff: str

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "document policy_resource root target before managed_block_hash proposed diff")
        resource = CapturedResource.from_dict(d["policy_resource"], path + ".policy_resource")
        if resource.role != "policy":
            _fail(path + ".policy_resource.role", "must identify original policy bytes")
        before = None if d["before"] is None else EncodedBytes.from_dict(d["before"], path + ".before")
        block = None if d["managed_block_hash"] is None else _hash(d["managed_block_hash"], path + ".managed_block_hash")
        if before is None and block is not None:
            _fail(path + ".managed_block_hash", "absent original cannot have managed block")
        if not isinstance(d["diff"], str) or "\x00" in d["diff"]:
            _fail(path + ".diff", "expected diff text")
        return cls(1, GuidanceDocument.from_dict(d["document"], path + ".document"), resource,
                   _absolute_path(d["root"], path + ".root"), _literal_path(d["target"], path + ".target"),
                   before, block, EncodedBytes.from_dict(d["proposed"], path + ".proposed"), d["diff"])


@dataclass(frozen=True)
class GuidanceApplyResult(Record):
    schema_version: int
    preview_hash: str
    root: str
    target: str
    content_hash: str
    changed: bool

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "preview_hash root target content_hash changed")
        if type(d["changed"]) is not bool:
            _fail(path + ".changed", "expected boolean")
        return cls(1, _hash(d["preview_hash"], path + ".preview_hash"),
                   _absolute_path(d["root"], path + ".root"), _literal_path(d["target"], path + ".target"),
                   _hash(d["content_hash"], path + ".content_hash"), d["changed"])


@dataclass(frozen=True)
class RuleOutput(Record):
    """Successful operation envelope; only check carries an assessment verdict."""

    schema_version: int
    operation: Literal["show", "prepare", "check", "guidance-preview", "guidance-apply"]
    output: GuidanceDocument | PreparedAssessment | PolicyResult | GuidanceUpdate | GuidanceApplyResult

    @classmethod
    def from_dict(cls, value, path="$input") -> Self:
        d = _artifact(value, path, "operation output")
        readers = {"show": GuidanceDocument, "prepare": PreparedAssessment, "check": PolicyResult,
                   "guidance-preview": GuidanceUpdate, "guidance-apply": GuidanceApplyResult}
        operation = _choice(d["operation"], path + ".operation", tuple(readers))
        return cls(1, operation, readers[operation].from_dict(d["output"], path + ".output"))
