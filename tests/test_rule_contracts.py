"""Policy inputs retain explicit meaning and reject ambiguous configuration."""

from copy import deepcopy
from dataclasses import FrozenInstanceError, replace
import hashlib
import json
import unittest

from lesscode_baseline.rules import contracts as c
from tests import test_contracts as baseline_fixtures


HASH = "a" * 64


def scope():
    return {"root": ".", "include": ["**/*.py"], "exclude": ["vendor/**"]}


def numeric(rule_id="size"):
    return {"kind": "numeric", "rule_id": rule_id, "scope": scope(),
            "enforcement": "blocking", "description": "Explicit byte limit",
            "selector": {"kind": "scalar_metric", "metric_key": "source_bytes"},
            "evidence": {"definition_id": "raw-source-bytes-v1", "producer": "lesscode",
                         "producer_version": "0.1", "unit": "bytes"},
            "mode": "absolute", "absolute_maximum": 100, "change_budget": None}


def check():
    return {"kind": "check", "rule_id": "api", "scope": scope(),
            "enforcement": "blocking", "description": "Declared API check",
            "check_id": "test.api", "expected": "passed"}


def review():
    return {"kind": "review", "rule_id": "roles", "scope": scope(),
            "enforcement": "advisory", "description": "Review names",
            "criterion_id": "role-names", "criterion_text": "Do names reveal roles?",
            "trust_requirement": "producer_record"}


def exception():
    return {"exception_id": "temporary-size", "rule_id": "size", "scope": scope(),
            "entity_ids": [], "justification": "Migration scheduled",
            "valid_from": "2026-01-01T00:00:00Z", "valid_until": "2027-01-01T00:00:00Z",
            "candidate_source_hash": None}


def policy():
    return {"schema_version": 1, "policy_id": "team-policy", "target_scope": scope(),
            "reference": None, "rules": [numeric()], "exceptions": []}


class PolicyContractTests(unittest.TestCase):
    def read(self, value):
        return c.loads(c.Policy, json.dumps(value))

    def reject(self, value, field):
        with self.assertRaisesRegex(c.ContractError, field):
            self.read(value)

    def test_all_variants_round_trip_with_explicit_bounds(self):
        value = policy()
        value["rules"] = [numeric(), check(), review()]
        value["exceptions"] = [exception()]
        result = self.read(value)
        self.assertEqual([r.rule_id for r in result.rules], ["api", "roles", "size"])
        self.assertIsInstance(result.rules[0], c.CheckRule)
        self.assertIsInstance(result.rules[1], c.ReviewRule)
        self.assertIsInstance(result.rules[2], c.NumericRule)
        self.assertEqual(c.loads(c.Policy, c.dumps(result)), result)
        self.assertEqual(result.rules[2].absolute_maximum, 100)

    def test_overlapping_rules_are_retained_with_enforcement(self):
        value = policy()
        value["rules"].append({**numeric("advisory-size"), "enforcement": "advisory"})
        result = self.read(value)
        self.assertEqual([(r.rule_id, r.enforcement) for r in result.rules],
                         [("advisory-size", "advisory"), ("size", "blocking")])

    def test_canonical_hash_ignores_json_layout_and_rule_order(self):
        value = policy()
        value["rules"].append(check())
        result = self.read(value)
        value["rules"].reverse()
        reordered = c.loads(c.Policy, json.dumps(value, indent=4, sort_keys=True).encode())
        self.assertEqual(c.policy_hash(result), c.policy_hash(reordered))
        self.assertEqual(c.policy_hash(result), hashlib.sha256(c.dumps(result).encode()).hexdigest())
        value["rules"][1]["absolute_maximum"] = 101
        self.assertNotEqual(c.policy_hash(result), c.policy_hash(self.read(value)))

    def test_inputs_are_detached_and_records_frozen(self):
        value = policy()
        result = self.read(value)
        value["rules"][0]["scope"]["include"].append("other/**")
        self.assertEqual(result.rules[0].scope.include, ("**/*.py",))
        with self.assertRaises(FrozenInstanceError):
            result.policy_id = "changed"
        with self.assertRaises(c.ContractError):
            c.dumps(replace(result, schema_version=True))

    def test_duplicate_json_keys_at_any_depth(self):
        for text in ('{"schema_version":1,"schema_version":1}',
                     json.dumps(policy()).replace('"kind": "numeric"', '"kind":"numeric","kind":"numeric"')):
            with self.subTest(text=text), self.assertRaisesRegex(c.ContractError, "duplicate JSON"):
                c.loads(c.Policy, text)

    def test_duplicate_rule_and_exception_ids(self):
        for field, item in (("rules", numeric()), ("exceptions", exception())):
            value = policy()
            value[field] = [item, deepcopy(item)]
            self.reject(value, "duplicate identity")

    def test_unknown_fields_and_versions(self):
        for version in (0, 2, True, 1.0, "1"):
            self.reject({**policy(), "schema_version": version}, "schema_version")
        for location in ("policy", "rule", "scope", "selector", "evidence", "exception"):
            value = policy()
            value["exceptions"] = [exception()]
            target = {"policy": value, "rule": value["rules"][0], "scope": value["target_scope"],
                      "selector": value["rules"][0]["selector"], "evidence": value["rules"][0]["evidence"],
                      "exception": value["exceptions"][0]}[location]
            target["typo"] = 1
            self.reject(value, "typo.*unknown field")

    def test_unsupported_kinds_selectors_and_missing_scalar_key(self):
        value = policy()
        value["rules"][0]["kind"] = "expression"
        self.reject(value, "kind")
        for selector in ({"kind": "automatic", "metric_key": None},
                         {"kind": "scalar_metric", "metric_key": None},
                         {"kind": "function_ccn", "metric_key": "complexity"}):
            value = policy()
            value["rules"][0]["selector"] = selector
            self.reject(value, "selector")

    def test_every_declared_selector_is_supported_without_projection(self):
        for name in ("function_ccn", "duplication_clone_pairs", "duplication_duplicated_tokens",
                     "duplication_total_tokens", "duplication_pair_span_ratio"):
            value = policy()
            value["rules"][0]["selector"] = {"kind": name, "metric_key": None}
            self.assertEqual(self.read(value).rules[0].selector.kind, name)

    def test_invalid_bounds_and_no_implicit_limit(self):
        for bound in (-1, True, float("nan"), float("inf"), -float("inf"), "10", 10**1000):
            value = policy()
            value["rules"][0]["absolute_maximum"] = bound
            self.reject(value, "size.*absolute_maximum")
        value = policy()
        value["rules"][0]["absolute_maximum"] = None
        self.reject(value, "size.*bound")
        del value["rules"][0]["absolute_maximum"]
        self.reject(value, "absolute_maximum")
        value = policy()
        value["rules"][0]["absolute_maximum"] = 0
        self.assertEqual(self.read(value).rules[0].absolute_maximum, 0)

    def test_ratchet_and_budgets_require_explicit_reference(self):
        for mode, budget in (("ratchet", None), ("absolute", {"kind": "absolute_increase", "maximum": 0})):
            value = policy()
            value["rules"][0].update(mode=mode, change_budget=budget)
            self.reject(value, "size.*reference")
            value["reference"] = {"source_hash": HASH, "profile_hash": "b" * 64}
            self.assertEqual(self.read(value).reference.source_hash, HASH)
        value["rules"][0].update(mode="ratchet", absolute_maximum=None)
        self.reject(value, "absolute_maximum")

    def test_budget_variants_and_bad_bounds(self):
        for kind in ("absolute_increase", "percentage_increase"):
            value = policy()
            value["reference"] = {"source_hash": HASH, "profile_hash": HASH}
            value["rules"][0].update(absolute_maximum=None, change_budget={"kind": kind, "maximum": 0})
            self.assertEqual(self.read(value).rules[0].change_budget.maximum, 0)
            for bad in (-1, True, float("nan")):
                value["rules"][0]["change_budget"]["maximum"] = bad
                self.reject(value, "maximum")

    def test_unsafe_scopes_are_rejected_without_normalization(self):
        for bad in ("../src", "/src", "C:/src", "src\\x", "src//x", "src/./x", "src/x.", "src/x ", "\x00"):
            for field in ("root", "include", "exclude"):
                value = policy()
                value["target_scope"][field] = bad if field == "root" else [bad]
                self.reject(value, field)
        value = policy()
        value["target_scope"]["root"] = "src/*"
        self.reject(value, "root")

    def test_scope_case_and_patterns_are_preserved(self):
        value = policy()
        value["target_scope"] = {"root": "Src", "include": ["**/[a-z]?.py", "Top.py"], "exclude": []}
        self.assertEqual(self.read(value).target_scope.root, "Src")
        self.assertEqual(self.read(value).target_scope.include, ("**/[a-z]?.py", "Top.py"))

    def test_checks_and_reviews_have_exact_conditions(self):
        for rule, field, bad in ((check(), "expected", True), (check(), "expected", "failed"),
                                 (review(), "trust_requirement", "claimed_independent"),
                                 (review(), "criterion_text", ""), (numeric(), "enforcement", "ignore")):
            value = policy()
            rule[field] = bad
            value["rules"] = [rule]
            self.reject(value, field)
        value = policy()
        value["rules"] = [{**review(), "trust_requirement": "external_independence"}]
        self.assertEqual(self.read(value).rules[0].trust_requirement, "external_independence")

    def test_reference_hashes_are_exact(self):
        for bad in (None, "", "A" * 64, "a" * 63):
            self.reject({**policy(), "reference": {"source_hash": bad, "profile_hash": HASH}}, "source_hash")

    def test_exceptions_are_bounded_numeric_targets(self):
        for rule_id in ("unknown", "api", "roles"):
            value = policy()
            value["rules"] += [check(), review()]
            value["exceptions"] = [{**exception(), "rule_id": rule_id}]
            self.reject(value, "rule_id")
        for field, bad in (("justification", ""), ("candidate_source_hash", "bad"),
                           ("entity_ids", ["entity-1", "entity-1"]), ("entity_ids", ["*"])):
            value = policy()
            value["exceptions"] = [{**exception(), field: bad}]
            self.reject(value, field)

    def test_exception_interval_is_ordered_explicit_utc(self):
        for field, bad in (("valid_from", "2026-01-01"), ("valid_until", "never"),
                           ("valid_until", "2025-01-01T00:00:00Z"),
                           ("valid_until", "2026-01-01T00:00:00Z"),
                           ("valid_from", "2026-01-01T01:00:00+01:00")):
            value = policy()
            value["exceptions"] = [{**exception(), field: bad}]
            self.reject(value, "valid_")
        value = policy()
        value["exceptions"] = [{**exception(), "candidate_source_hash": HASH, "entity_ids": ["entity-1"]}]
        self.assertEqual(self.read(value).exceptions[0].entity_ids, ("entity-1",))

    def test_malformed_containers_and_missing_identities(self):
        for field in ("rules", "exceptions"):
            for bad in (None, {}, "all", [None]):
                self.reject({**policy(), field: bad}, field)
        for field in ("rule_id", "description"):
            value = policy()
            value["rules"][0][field] = " "
            self.reject(value, field)
        for field in ("include", "exclude"):
            value = policy()
            value["target_scope"][field] = ["**/*.py", "**/*.py"]
            self.reject(value, field)
        value = policy()
        value["rules"][0]["change_budget"] = {"kind": "reduction_target", "maximum": 1}
        self.reject(value, "change_budget.kind")

    def test_direct_construction_is_revalidated_before_hashing(self):
        result = self.read(policy())
        invalid = replace(result, rules=(replace(result.rules[0], absolute_maximum=True),))
        for operation in (c.dumps, c.policy_hash):
            with self.assertRaisesRegex(c.ContractError, "absolute_maximum"):
                operation(invalid)
        with self.assertRaises(TypeError):
            c.policy_hash(result.rules[0])

    def test_exception_changes_and_reference_changes_alter_policy_identity(self):
        value = policy()
        value["exceptions"] = [exception()]
        original = c.policy_hash(self.read(value))
        value["exceptions"][0]["valid_until"] = "2028-01-01T00:00:00Z"
        self.assertNotEqual(original, c.policy_hash(self.read(value)))
        value = policy()
        original = c.policy_hash(self.read(value))
        value["reference"] = {"source_hash": HASH, "profile_hash": HASH}
        self.assertNotEqual(original, c.policy_hash(self.read(value)))



# Source-bound rule inputs use baseline records without changing their schema.


def entity(source=HASH, identity="function-1"):
    return {"inventory_id": identity, "source_hash": source,
            "selector": {"kind": "function_ccn", "metric_key": None},
            "metric": {**baseline_fixtures.metric(), "key": "complexity", "value": 12,
                       "unit": "CCN", "definition_id": "lizard-ccn-v1"},
            "location": {"path": "src/main.py", "start_line": 1, "end_line": 4},
            "long_name": "main()", "evidence_refs": ["baseline/complexity.json"]}


def inventory(source=HASH):
    return {"source_hash": source, "status": "complete", "reason": None,
            "entities": [entity(source)]}


def bundle():
    result = baseline_fixtures.report()
    result["candidate"]["snapshot"]["content_hash"] = "b" * 64
    result["candidate"]["checks"][0]["source_hash"] = "b" * 64
    return {"schema_version": 1, "report": result, "profile_hash": HASH,
            "baseline_inventory": inventory(), "candidate_inventory": inventory("b" * 64)}


def treatment(kind="matched"):
    return {"policy_hash": c.policy_hash(c.Policy.from_dict(policy())),
            "baseline_source_hash": HASH, "candidate_source_hash": "b" * 64,
            "kind": kind, "baseline_id": "function-1" if kind != "new" else None,
            "candidate_id": "function-1" if kind != "removed" else None,
            "producer_id": "maintainer", "rationale": "Explicit continuity review"}


def review_policy():
    return {**policy(), "rules": [review()]}


def review_record():
    return {"policy_hash": c.policy_hash(c.Policy.from_dict(review_policy())),
            "candidate_source_hash": "b" * 64, "rule_id": "roles", "criterion_id": "role-names",
            "scope": scope(), "reviewer_id": "maintainer", "outcome": "satisfied",
            "rationale": "Names describe the data roles", "locations": [entity()["location"]],
            "provenance": "producer_supplied"}


class EvidenceInputContractTests(unittest.TestCase):
    def read(self, name, value):
        return c.loads(getattr(c, name), json.dumps(value))

    def test_round_trip_inputs_and_public_baseline_types(self):
        for name, value in (("EntityObservation", entity()), ("EvidenceInventory", inventory()),
                            ("EvidenceBundle", bundle()), ("EntityTreatment", treatment()),
                            ("ReviewRecord", review_record())):
            with self.subTest(name=name):
                parsed = self.read(name, value)
                self.assertEqual(c.loads(type(parsed), c.dumps(parsed)), parsed)
        parsed = self.read("EvidenceBundle", bundle())
        self.assertIsInstance(parsed.report, baseline_fixtures.c.RunReport)
        self.assertIsInstance(parsed.baseline_inventory.entities[0].metric, baseline_fixtures.c.MetricEvidence)
        self.assertIsInstance(parsed.baseline_inventory.entities[0].location, baseline_fixtures.c.SourceSpan)
        self.assertEqual(parsed.report.reasons, ("All declared checks passed",))

    def test_unavailable_is_not_zero_and_zero_inventory_is_explicit(self):
        value = entity()
        value["metric"].update(status="unavailable", value=None, reason="producer missing")
        self.assertIsNone(self.read("EntityObservation", value).metric.value)
        for bad in (0, True):
            value["metric"]["value"] = bad
            with self.assertRaises(c.ContractError):
                self.read("EntityObservation", value)
        empty = {**inventory(), "entities": []}
        self.assertEqual(self.read("EvidenceInventory", empty).status, "complete")
        missing = {**empty, "status": "unavailable", "reason": "no inventory"}
        self.assertEqual(self.read("EvidenceInventory", missing).reason, "no inventory")
        for bad in ({**missing, "entities": [entity()]}, {**missing, "reason": None},
                    {**empty, "reason": "missing"}):
            with self.assertRaises(c.ContractError):
                self.read("EvidenceInventory", bad)

    def test_observation_requires_exact_function_address_and_scalar_value(self):
        for field, bad in (("inventory_id", "*"), ("source_hash", "bad"),
                           ("location", None), ("long_name", ""), ("evidence_refs", []),
                           ("evidence_refs", ["../escape"]), ("evidence_refs", ["x", "x"])):
            with self.subTest(field=field), self.assertRaises(c.ContractError):
                self.read("EntityObservation", {**entity(), field: bad})
        for bad in (True, -1, float("inf"), {"kind": "findings", "findings": []}):
            value = entity()
            value["metric"]["value"] = bad
            with self.assertRaises(c.ContractError):
                self.read("EntityObservation", value)
        value = entity()
        value.update(selector={"kind": "scalar_metric", "metric_key": "source_bytes"},
                     location=None, long_name=None)
        value["metric"] = baseline_fixtures.metric()
        self.assertIsNone(self.read("EntityObservation", value).location)
        value["selector"]["metric_key"] = "other"
        with self.assertRaises(c.ContractError):
            self.read("EntityObservation", value)

    def test_inventory_rejects_duplicate_addresses_and_source_mismatch(self):
        for extra in (entity(), entity(identity="other"), entity("b" * 64, "other")):
            with self.assertRaises(c.ContractError):
                self.read("EvidenceInventory", {**inventory(), "entities": [entity(), extra]})

    def test_bundle_rejects_internal_source_location_scope_and_check_conflicts(self):
        mutations = [lambda v: v["candidate_inventory"].update(source_hash=HASH),
                     lambda v: v["baseline_inventory"]["entities"][0]["location"].update(path="absent.py"),
                     lambda v: v["baseline_inventory"]["entities"][0]["metric"].update(scope_hash="c"*64),
                     lambda v: v["report"]["candidate"]["checks"][0].update(source_hash=HASH),
                     lambda v: v["report"]["candidate"]["checks"][0].update(plan_hash="c"*64)]
        for mutate in mutations:
            value = bundle()
            mutate(value)
            with self.assertRaises(c.ContractError):
                self.read("EvidenceBundle", value)

    def test_frozen_detached_inputs_and_versions(self):
        value = bundle()
        parsed = self.read("EvidenceBundle", value)
        value["baseline_inventory"]["entities"].clear()
        self.assertEqual(len(parsed.baseline_inventory.entities), 1)
        with self.assertRaises(FrozenInstanceError):
            parsed.profile_hash = "c" * 64
        for bad in (True, 2, "1", 1.0):
            with self.assertRaises(c.ContractError):
                self.read("EvidenceBundle", {**bundle(), "schema_version": bad})

    def test_all_treatments_require_unambiguous_sides(self):
        for kind in ("matched", "new", "removed", "unresolved"):
            self.assertEqual(self.read("EntityTreatment", treatment(kind)).kind, kind)
        for kind, left, right in (("matched", None, "function-1"), ("new", "function-1", "function-1"),
                                  ("removed", "function-1", "function-1"), ("unresolved", None, None)):
            with self.assertRaises(c.ContractError):
                self.read("EntityTreatment", {**treatment(kind), "baseline_id": left, "candidate_id": right})
        for field in ("producer_id", "rationale", "baseline_source_hash", "policy_hash"):
            with self.assertRaises(c.ContractError):
                self.read("EntityTreatment", {**treatment(), field: ""})

    def test_treatment_set_is_one_to_one_and_bound_to_inventory_and_policy(self):
        evidence = self.read("EvidenceBundle", bundle())
        p = c.Policy.from_dict(policy())
        record = self.read("EntityTreatment", treatment())
        self.assertEqual(c.validate_treatments((record,), evidence, p), (record,))
        self.assertEqual(c.validate_treatments((), evidence, p), ())
        for records in ((record, record),
                        (record, self.read("EntityTreatment", treatment("new"))),
                        (record, self.read("EntityTreatment", treatment("removed")))):
            with self.assertRaises(c.ContractError):
                c.validate_treatments(records, evidence, p)
        for field, bad in (("baseline_id", "absent"), ("candidate_id", "absent"),
                           ("policy_hash", "c"*64), ("baseline_source_hash", "c"*64),
                           ("candidate_source_hash", "c"*64)):
            record = self.read("EntityTreatment", {**treatment(), field: bad})
            with self.assertRaises(c.ContractError):
                c.validate_treatments((record,), evidence, p)

    def test_reviews_keep_provenance_without_authenticating_independence(self):
        evidence = self.read("EvidenceBundle", bundle())
        p = c.Policy.from_dict(review_policy())
        record = self.read("ReviewRecord", review_record())
        self.assertEqual(record.provenance, "producer_supplied")
        self.assertEqual(record.trust_status("producer_record"), "supported")
        self.assertEqual(record.trust_status("external_independence"), "unverified")
        self.assertEqual(c.validate_reviews((record,), evidence, p), (record,))
        self.assertEqual(c.validate_reviews((), evidence, p), ())
        for outcome in ("concern", "unresolved"):
            self.assertEqual(self.read("ReviewRecord", {**review_record(), "outcome": outcome}).outcome, outcome)
        for field, bad in (("provenance", "authenticated_independent"), ("reviewer_id", ""),
                           ("rationale", ""), ("locations", []), ("outcome", "passed")):
            with self.assertRaises(c.ContractError):
                self.read("ReviewRecord", {**review_record(), field: bad})

    def test_review_conflicts_bindings_membership_and_scope_fail_closed(self):
        evidence = self.read("EvidenceBundle", bundle())
        p = c.Policy.from_dict(review_policy())
        record = self.read("ReviewRecord", review_record())
        for second in (record, self.read("ReviewRecord", {**review_record(), "outcome": "concern"})):
            with self.assertRaises(c.ContractError):
                c.validate_reviews((record, second), evidence, p)
        for field, bad in (("policy_hash", "c"*64), ("candidate_source_hash", HASH),
                           ("rule_id", "unknown"), ("criterion_id", "other"),
                           ("scope", {**scope(), "include": ["other/**"]}),
                           ("locations", [{"path": "absent.py", "start_line": 1, "end_line": 2}])):
            value = self.read("ReviewRecord", {**review_record(), field: bad})
            with self.assertRaises(c.ContractError):
                c.validate_reviews((value,), evidence, p)
        outside = bundle()
        outside["report"]["candidate"]["snapshot"]["files"].append(
            {**baseline_fixtures.snapshot()["files"][0], "path": "vendor/main.py"})
        value = self.read("ReviewRecord", {**review_record(), "locations": [
            {"path": "vendor/main.py", "start_line": 1, "end_line": 2}]})
        with self.assertRaises(c.ContractError):
            c.validate_reviews((value,), self.read("EvidenceBundle", outside), p)

    def test_review_spans_are_positive_ordered_literal_and_unique(self):
        for bad in ({"path": "src/main.py", "start_line": 0, "end_line": 2},
                    {"path": "src/main.py", "start_line": True, "end_line": 2},
                    {"path": "src/main.py", "start_line": 3, "end_line": 2},
                    {"path": "../main.py", "start_line": 1, "end_line": 2}):
            with self.assertRaises(c.ContractError):
                self.read("ReviewRecord", {**review_record(), "locations": [bad]})
        with self.assertRaises(c.ContractError):
            self.read("ReviewRecord", {**review_record(), "locations": [entity()["location"]]*2})

    def test_unknown_fields_and_duplicate_json_fail_closed_for_every_record(self):
        for name, value in (("EntityObservation", entity()), ("EvidenceInventory", inventory()),
                            ("EvidenceBundle", bundle()), ("EntityTreatment", treatment()),
                            ("ReviewRecord", review_record())):
            with self.subTest(name=name), self.assertRaises(c.ContractError):
                self.read(name, {**value, "unexpected": True})
            with self.assertRaises(c.ContractError):
                c.loads(getattr(c, name), '{"x":1,"x":2}')

    def test_reference_binding_and_mismatched_entity_kinds_are_rejected(self):
        evidence = self.read("EvidenceBundle", bundle())
        for reference in ({"source_hash": "c"*64, "profile_hash": HASH},
                          {"source_hash": HASH, "profile_hash": "c"*64}):
            p = c.Policy.from_dict({**policy(), "reference": reference})
            record = self.read("EntityTreatment", {**treatment(), "policy_hash": c.policy_hash(p)})
            with self.assertRaises(c.ContractError):
                c.validate_treatments((record,), evidence, p)
        value = bundle()
        candidate = value["candidate_inventory"]["entities"][0]
        candidate.update(selector={"kind": "scalar_metric", "metric_key": "source_bytes"},
                         metric=baseline_fixtures.metric(), location=None, long_name=None)
        with self.assertRaises(c.ContractError):
            c.validate_treatments((self.read("EntityTreatment", treatment()),),
                                  self.read("EvidenceBundle", value), c.Policy.from_dict(policy()))

    def test_literal_evidence_and_span_paths_reject_patterns(self):
        for bad in ("src/*.py", ".", "src/[a].py"):
            value = entity()
            value["location"]["path"] = bad
            with self.assertRaises(c.ContractError):
                self.read("EntityObservation", value)
            value = entity()
            value["evidence_refs"] = [bad]
            with self.assertRaises(c.ContractError):
                self.read("EntityObservation", value)

    def test_new_removed_and_one_sided_unresolved_remain_distinct(self):
        evidence = self.read("EvidenceBundle", bundle())
        p = c.Policy.from_dict(policy())
        records = tuple(self.read("EntityTreatment", treatment(kind)) for kind in ("new", "removed"))
        self.assertEqual(c.validate_treatments(records, evidence, p), records)
        for side in ("baseline_id", "candidate_id"):
            record = self.read("EntityTreatment", {**treatment("unresolved"), side: None})
            self.assertEqual(c.validate_treatments((record,), evidence, p)[0].kind, "unresolved")




def captured():
    return {"role": "policy", "path": "D:/project/policy.json", "sha256": HASH}


def prepared():
    return {"schema_version": 1, "policy": policy(), "policy_hash": c.policy_hash(c.Policy.from_dict(policy())),
            "evidence": bundle(), "resources": [captured(), {**captured(), "role": "run_index", "path": "D:/run/artifacts.json"},
            {**captured(), "role": "run_artifact", "path": "D:/run/plan.json"}],
            "reviews": [], "treatments": [], "as_of": "2026-10-31T12:00:00Z"}


def finding():
    return {"rule_id": "size", "scope": scope(), "entity_id": "module", "basis": "absolute",
            "enforcement": "blocking", "status": "satisfied", "disposition": "unchanged",
            "reference": None, "candidate": None, "expected": numeric(), "locations": [],
            "evidence_refs": [{"schema_version": 1, "run_index_hash": HASH, "artifact_path": "report.json",
                               "artifact_hash": HASH, "record_address": "/candidate/metrics/0"}],
            "reason": "Observed limit satisfied", "exceptions": []}


def result():
    return {"schema_version": 1, "prepared_hash": HASH, "baseline_decision": "accepted",
            "baseline_reasons": ["Checks passed"], "policy_verdict": "satisfied", "policy_accepted": True,
            "findings": [finding()], "diagnostics": []}


def byte_content(raw):
    import base64
    return {"encoding": "base64", "data": base64.b64encode(raw).decode(), "sha256": hashlib.sha256(raw).hexdigest()}


def document():
    return {"schema_version": 1, "policy_hash": HASH, "scope": scope(), "renderer_version": "1",
            "kind": "pre_change_guidance", "content": byte_content(b"Guidance\r\n")}


def update():
    return {"schema_version": 1, "document": document(), "policy_resource": captured(),
            "root": "D:/project", "target": "AGENTS.md", "before": byte_content(b"\xef\xbb\xbfUser\r\n"),
            "managed_block_hash": None, "proposed": byte_content(b"\xef\xbb\xbfUser\r\nGuidance\r\n"), "diff": "+Guidance"}


class ArtifactContractTests(unittest.TestCase):
    def roundtrip(self, cls, value):
        parsed = c.loads(cls, json.dumps(value))
        self.assertEqual(parsed, c.loads(cls, c.dumps(parsed)))
        with self.assertRaises(FrozenInstanceError):
            parsed.schema_version = 2
        return parsed

    def test_prepared_captures_fixed_inputs_and_time(self):
        parsed = self.roundtrip(c.PreparedAssessment, prepared())
        self.assertEqual(parsed.as_of, "2026-10-31T12:00:00Z")
        self.assertEqual(parsed.evidence.report, c.EvidenceBundle.from_dict(bundle()).report)
        self.assertEqual(len(parsed.resources), 3)

    def test_prepared_rejects_version_hash_time_and_resource_errors(self):
        for key, invalid in [("schema_version", True), ("schema_version", 2), ("policy_hash", "b" * 64),
                             ("as_of", "2026-10-31"), ("resources", []), ("unknown", 1)]:
            with self.subTest(key=key, invalid=invalid), self.assertRaises(c.ContractError):
                c.PreparedAssessment.from_dict({**prepared(), key: invalid})
        for path in ["relative.json", "D:/run/../policy.json", "D:policy.json"]:
            with self.subTest(path=path), self.assertRaises(c.ContractError):
                c.CapturedResource.from_dict({**captured(), "path": path})

    def test_result_roundtrip_and_independent_outcomes(self):
        self.roundtrip(c.PolicyResult, result())
        for baseline in ["accepted", "rejected", "incomplete", "baseline_ineligible", "invalid"]:
            for verdict in ["satisfied", "violations", "insufficient", "invalid"]:
                accepted = baseline == "accepted" and verdict == "satisfied"
                value = {**result(), "baseline_decision": baseline, "policy_verdict": verdict,
                         "policy_accepted": accepted}
                self.roundtrip(c.PolicyResult, value)
                with self.assertRaises(c.ContractError):
                    c.PolicyResult.from_dict({**value, "policy_accepted": not accepted})
        with self.assertRaises(c.ContractError):
            c.PolicyResult.from_dict({**result(), "policy_accepted": 1})

    def test_findings_retain_expected_rule_and_exact_evidence(self):
        parsed = c.PolicyFinding.from_dict(finding())
        self.assertIsInstance(parsed.expected, c.NumericRule)
        self.assertEqual(parsed.evidence_refs[0].record_address, "/candidate/metrics/0")
        for field, bad in [("rule_id", "other"), ("enforcement", "advisory"), ("basis", ""), ("extra", 1)]:
            with self.subTest(field=field), self.assertRaises(c.ContractError):
                c.PolicyFinding.from_dict({**finding(), field: bad})
        for field, bad in [("schema_version", 2), ("artifact_path", "../result.json"),
                           ("artifact_hash", "bad"), ("record_address", "candidate"), ("record_address", "/bad~2")]:
            ref = finding()["evidence_refs"][0]
            with self.subTest(field=field), self.assertRaises(c.ContractError):
                c.EvidenceReference.from_dict({**ref, field: bad})

    def test_findings_order_and_duplicate_identity(self):
        value = result()
        value["findings"] = [{**finding(), "entity_id": "z"}, finding()]
        self.assertEqual([f.entity_id for f in c.PolicyResult.from_dict(value).findings], ["module", "z"])
        value["findings"] = [finding(), finding()]
        with self.assertRaises(c.ContractError):
            c.PolicyResult.from_dict(value)

    def test_guidance_preserves_bytes_and_absence(self):
        parsed = self.roundtrip(c.GuidanceUpdate, update())
        self.assertEqual(parsed.before.decode(), b"\xef\xbb\xbfUser\r\n")
        self.roundtrip(c.GuidanceDocument, document())
        self.roundtrip(c.GuidanceUpdate, {**update(), "before": None})
        for key, invalid in [("target", "../AGENTS.md"), ("root", "."), ("schema_version", 2)]:
            with self.subTest(key=key), self.assertRaises(c.ContractError):
                c.GuidanceUpdate.from_dict({**update(), key: invalid})
        with self.assertRaises(c.ContractError):
            c.GuidanceUpdate.from_dict({**update(), "before": None, "managed_block_hash": HASH})

    def test_encoded_bytes_fail_closed(self):
        for key, bad in [("encoding", "utf8"), ("data", "%%%"), ("data", "Zh=="), ("sha256", HASH)]:
            with self.subTest(key=key), self.assertRaises(c.ContractError):
                c.EncodedBytes.from_dict({**byte_content(b"f"), key: bad})

    def test_storage_and_partial_failures_do_not_claim_completion(self):
        files = [{"path": p, "sha256": HASH} for p in ["manifest.json", "prepared.json", "result.json", "summary.md"]]
        index = {"schema_version": 1, "prepared_hash": HASH, "files": files}
        self.roundtrip(c.PolicyArtifactIndex, index)
        with self.assertRaises(c.ContractError):
            c.PolicyArtifactIndex.from_dict({**index, "files": files[:-1]})
        partial = {"schema_version": 1, "state": "running", "result": result(), "index": None,
                   "external_evidence": "unverified", "diagnostics": ["Interrupted"]}
        self.roundtrip(c.PolicyRecovery, partial)
        with self.assertRaises(c.ContractError):
            c.PolicyRecovery.from_dict({**partial, "state": "finalized"})
        failure = {"schema_version": 1, "operation": "check", "kind": "persistence", "diagnostics": ["Disk full"],
                   "partial_result": result()}
        self.roundtrip(c.RuleFailure, failure)
        complete = {**partial, "state": "finalized", "index": index, "external_evidence": "verified"}
        self.roundtrip(c.PolicyRecovery, complete)
        with self.assertRaises(c.ContractError):
            c.PolicyRecovery.from_dict({**complete, "index": {**index, "prepared_hash": "b" * 64}})

    def test_output_envelope_distinguishes_operations_from_assessment(self):
        value = {"schema_version": 1, "operation": "check", "output": result()}
        self.roundtrip(c.RuleOutput, value)
        shown = {**value, "operation": "show", "output": document()}
        self.roundtrip(c.RuleOutput, shown)
        with self.assertRaises(c.ContractError):
            c.RuleOutput.from_dict({**shown, "policy_accepted": True})
        with self.assertRaises(c.ContractError):
            c.RuleOutput.from_dict({**shown, "output": result()})
        applied = {"schema_version": 1, "preview_hash": HASH, "root": "/repo", "target": "AGENTS.md",
                   "content_hash": HASH, "changed": False}
        self.roundtrip(c.GuidanceApplyResult, applied)
        self.roundtrip(c.RuleOutput, {**value, "operation": "guidance-apply", "output": applied})
        with self.assertRaises(c.ContractError):
            c.GuidanceApplyResult.from_dict({**applied, "changed": 0})

    def test_partial_and_external_evidence_remain_distinct(self):
        files = [{"path": p, "sha256": HASH} for p in ["manifest.json", "prepared.json", "result.json", "summary.md"]]
        value = {"schema_version": 1, "state": "finalized", "result": result(),
                 "index": {"schema_version": 1, "prepared_hash": HASH, "files": files},
                 "external_evidence": "unverified", "diagnostics": ["Run moved"]}
        self.assertFalse(c.PolicyRecovery.from_dict(value).complete)
        self.assertTrue(c.PolicyRecovery.from_dict({**value, "external_evidence": "verified"}).complete)

    def test_prepared_review_claims_and_duplicate_resource_detection(self):
        value = prepared()
        value["reviews"] = [review_record()]
        with self.assertRaisesRegex(c.ContractError, "missing captured review"):
            c.PreparedAssessment.from_dict(value)
        value["resources"].append({**captured(), "role": "review", "path": "/reviews.json"})
        parsed = c.PreparedAssessment.from_dict(value)
        self.assertEqual(parsed.reviews[0].policy_hash, review_record()["policy_hash"])
        value["resources"].append(value["resources"][0])
        with self.assertRaisesRegex(c.ContractError, "duplicate identity"):
            c.PreparedAssessment.from_dict(value)

    def test_exception_effect_preserves_original_violation(self):
        effect = {"exception_id": "temporary", "state": "active", "reason": "In validity window"}
        value = {**finding(), "status": "violation", "exceptions": [effect]}
        parsed = c.PolicyFinding.from_dict(value)
        self.assertEqual(parsed.status, "violation")
        self.assertEqual(parsed.exceptions[0].state, "active")
        with self.assertRaises(c.ContractError):
            c.PolicyFinding.from_dict({**value, "status": "satisfied"})
        with self.assertRaises(c.ContractError):
            c.PolicyFinding.from_dict({**value, "expected": check(), "rule_id": "api"})


if __name__ == "__main__":
    unittest.main()
