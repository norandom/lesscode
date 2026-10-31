"""Durable prepared plans retain identities without probing local resources."""

from copy import deepcopy
from dataclasses import replace
import json
import unittest
from unittest.mock import patch

from lesscode_baseline import contracts as c
from tests.test_contracts import HASH, plan


def resource(logical_path="checks/oracle.py", path="C:/evaluator/checks/oracle.py"):
    return {"logical_path": logical_path, "path": path, "content_hash": HASH, "size_bytes": 12}


def tool():
    return {"name": "lizard", "requested_version": "1.17.31", "status": "available",
            "executable": "C:/Python/python.exe", "entry_point": None,
            "observed_version": "1.17.31", "fingerprint": HASH, "reason": None,
            "resources": [resource("lizard/package.py", "C:/Python/lizard.py")]}


def prepared():
    return {"schema_version": 1, "plan": plan(), "profile_path": "C:/evaluator/profile.json",
            "evaluator_root": "C:/evaluator", "evaluator_resources": [resource()], "tools": [tool()]}


class PreparedPlanContractTests(unittest.TestCase):
    def read(self, record_name, data):
        return c.loads(getattr(c, record_name), json.dumps(data))

    def reject(self, record_name, data, field):
        with self.assertRaisesRegex(c.ContractError, field):
            self.read(record_name, data)

    def test_round_trip_and_preserve_original_record_shapes(self):
        for name, data in (("ResourceIdentity", resource()), ("ToolBinding", tool()),
                           ("PreparedPlan", prepared())):
            with self.subTest(name=name):
                record = self.read(name, data)
                self.assertEqual(json.loads(c.dumps(record)), data)
                self.assertEqual(type(record).from_dict(record.to_dict()), record)
        record = self.read("PreparedPlan", prepared())
        self.assertEqual(record.plan.to_dict(), plan())
        self.assertEqual(record.plan.profile.to_dict(), plan()["profile"])

    def test_posix_and_windows_paths_and_additional_namespaces(self):
        data = prepared()
        data.update(profile_path="/opt/evaluator/profile.json", evaluator_root="/opt/evaluator")
        data["evaluator_resources"] += [resource("evaluator/contracts.py", "/opt/code/contracts.py"),
                                        resource("locks/tools.lock", "D:\\locks\\tools.lock")]
        data["tools"][0]["entry_point"] = "/opt/lizard/main.py"
        with patch("builtins.open", side_effect=AssertionError("contract must not read files")):
            self.assertEqual(len(self.read("PreparedPlan", data).evaluator_resources), 3)

    def test_missing_and_partial_unavailable_tools_keep_diagnostics(self):
        for updates in ({"executable": None, "entry_point": None, "observed_version": None,
                         "fingerprint": None, "resources": []},
                        {"observed_version": "1.17.30"}):
            with self.subTest(updates=updates):
                data = prepared()
                data["tools"][0].update(status="unavailable", reason="tool absent or version mismatch", **updates)
                record = self.read("PreparedPlan", data)
                self.assertEqual(record.tools[0].status, "unavailable")
                self.assertEqual(json.loads(c.dumps(record)), data)

    def test_resource_field_validation(self):
        for field, invalid in (("logical_path", "../oracle.py"), ("logical_path", "C:/oracle.py"),
                               ("logical_path", "checks\\oracle.py"), ("path", "relative.py"),
                               ("path", "C:/checks/../oracle.py"), ("content_hash", "xyz"),
                               ("size_bytes", -1), ("size_bytes", True), ("size_bytes", 1.5)):
            with self.subTest(field=field, invalid=invalid):
                data = resource()
                data[field] = invalid
                self.reject("ResourceIdentity", data, field)

    def test_available_tool_requires_full_verified_identity(self):
        for field, invalid in (("executable", None), ("executable", "python"),
                               ("entry_point", "main.py"), ("observed_version", None),
                               ("observed_version", "1.17.30"), ("fingerprint", None),
                               ("fingerprint", "bad"), ("resources", []),
                               ("reason", "not available"), ("status", "missing"),
                               ("name", ""), ("requested_version", "")):
            with self.subTest(field=field, invalid=invalid):
                data = tool()
                data[field] = invalid
                self.reject("ToolBinding", data, field)

    def test_unavailable_still_validates_partial_identity_and_reason(self):
        for field, invalid in (("reason", None), ("reason", ""), ("fingerprint", "bad"),
                               ("executable", "python"), ("entry_point", "relative.py"),
                               ("observed_version", "")):
            with self.subTest(field=field, invalid=invalid):
                data = tool()
                data.update(status="unavailable", reason="missing dependency")
                data[field] = invalid
                self.reject("ToolBinding", data, field)

    def test_duplicate_identities(self):
        for field in ("evaluator_resources", "tools"):
            data = prepared()
            data[field].append(deepcopy(data[field][0]))
            self.reject("PreparedPlan", data, field)
        data = tool()
        data["resources"].append(resource("lizard/package.py", "C:/other.py"))
        self.reject("ToolBinding", data, "resources")

    def test_declared_resource_and_tool_coverage_and_pins(self):
        for field in ("evaluator_resources", "tools"):
            data = prepared()
            data[field] = []
            self.reject("PreparedPlan", data, field)
        data = prepared()
        data["tools"][0].update(requested_version="9", observed_version="9")
        self.reject("PreparedPlan", data, "requested_version")

    def test_strict_wrapper_schema_paths_and_nested_unknown_fields(self):
        for field, invalid in (("schema_version", 2), ("schema_version", True),
                               ("profile_path", "profile.json"), ("evaluator_root", "../evaluator")):
            data = prepared()
            data[field] = invalid
            self.reject("PreparedPlan", data, field)
        for name, data in (("ResourceIdentity", resource()), ("ToolBinding", tool()),
                           ("PreparedPlan", prepared())):
            data["unknown"] = 1
            self.reject(name, data, "unknown")
            del data["unknown"]
            field = next(iter(data))
            del data[field]
            self.reject(name, data, field)

    def test_serialization_revalidates_direct_construction(self):
        record = self.read("PreparedPlan", prepared())
        for invalid in (replace(record, tools=()), replace(record, schema_version=2),
                        replace(record.tools[0], resources=()),
                        replace(record.evaluator_resources[0], size_bytes=-1)):
            with self.subTest(record=invalid):
                with self.assertRaises(c.ContractError):
                    c.dumps(invalid)


if __name__ == "__main__":
    unittest.main()
