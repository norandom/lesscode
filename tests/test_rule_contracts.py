"""Policy inputs retain explicit meaning and reject ambiguous configuration."""

from copy import deepcopy
from dataclasses import FrozenInstanceError, replace
import hashlib
import json
import unittest

from lesscode_baseline.rules import contracts as c


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


if __name__ == "__main__":
    unittest.main()
