"""Preparation freezes declarations and resources before workload execution."""

from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.config import prepare, verify_integrity
from lesscode_baseline.contracts import Failure, PreparedPlan, dumps, loads


ROOT = Path(__file__).resolve().parents[1]


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.evaluator = self.root / "evaluator"
        self.evaluator.mkdir()
        (self.evaluator / "runner.py").write_text("raise RuntimeError('must not run')\n")
        self.profile = json.loads((ROOT / "benchmarks/profiles/python.json").read_text())
        self.profile["evaluator_resources"] = ["profile.json", "runner.py"]
        for check in self.profile["checks"]:
            check["command"]["argv"][1] = "runner.py"
        self.path = self.evaluator / "profile.json"
        self.save()
        for side in ("baseline", "candidate"):
            (self.root / side).mkdir()
            (self.root / side / "module.py").write_text("value = 1\n")

    def save(self):
        self.path.write_text(json.dumps(self.profile), encoding="utf-8")

    def prepare(self, **kwargs):
        return prepare(self.path, self.root / "baseline", self.root / "candidate",
                       self.root / "plan.json", **kwargs)

    def test_repeatable_roundtrip_and_absolute_runner_without_execution(self):
        first, second = self.prepare(), self.prepare()
        self.assertIsInstance(first, PreparedPlan)
        self.assertEqual(dumps(first), dumps(second))
        self.assertEqual(loads(PreparedPlan, dumps(first)), first)
        self.assertFalse((self.root / "plan.json").exists())
        self.assertEqual(first.plan.protection.mode, "local_detection")
        self.assertEqual(len(first.plan.resolved_commands), 2)
        for command in first.plan.resolved_commands:
            self.assertTrue(Path(command.argv[0]).is_absolute())
            self.assertEqual(command.argv[1], str(self.evaluator / "runner.py"))
            self.assertEqual(command.argv[3], ".")
        self.assertEqual(verify_integrity(first).status, "unchanged")

    def test_integrity_identifies_resource_modification_and_deletion(self):
        prepared = self.prepare()
        (self.evaluator / "runner.py").write_text("changed\n")
        self.assertIn("runner.py", verify_integrity(prepared).changed_resources)
        (self.evaluator / "runner.py").unlink()
        self.assertIn("runner.py", verify_integrity(prepared).changed_resources)

    def test_profile_change_and_in_memory_rule_change_are_detected(self):
        prepared = self.prepare()
        self.profile["thresholds"][0]["value"] = 9
        self.save()
        self.assertIn("profile.json", verify_integrity(prepared).changed_resources)
        changed = deepcopy(prepared)
        changed.plan.profile.tools[0].options["language"] = "other"
        self.assertEqual(verify_integrity(changed).status, "changed")

    def test_reloaded_command_changes_invalidate_integrity_without_probes(self):
        prepared = self.prepare()
        original = json.loads(dumps(prepared))
        mutations = {
            "runner": lambda c: c[0]["argv"].__setitem__(1, "different_evaluator.py"),
            "executable": lambda c: c[0]["argv"].__setitem__(0, str(self.root / "other.exe")),
            "environment": lambda c: c[0]["environment"].__setitem__("PYTHONHASHSEED", "1"),
            "cwd": lambda c: c[0].__setitem__("cwd", "elsewhere"),
            "limits": lambda c: c[0]["limits"].__setitem__("timeout_seconds", 999),
            "version": lambda c: c[0].__setitem__("executable_version", "0.0.0"),
            "missing": lambda c: c.pop(),
            "extra": lambda c: c.append({**deepcopy(c[0]), "command_id": "extra"}),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                document = deepcopy(original)
                mutate(document["plan"]["resolved_commands"])
                changed = loads(PreparedPlan, json.dumps(document))
                with patch("lesscode_baseline.config._probe") as probe:
                    result = verify_integrity(changed)
                probe.assert_not_called()
                self.assertEqual(result.status, "changed")
                self.assertIn("_context/resolved-commands", result.changed_resources)
        prepared.plan.resolved_commands[0].environment["PYTHONHASHSEED"] = "1"
        self.assertEqual(verify_integrity(prepared).status, "changed")

    def test_reloaded_tool_path_changes_invalidate_integrity(self):
        prepared = self.prepare()
        for name, attribute in (("python", "executable"), ("jscpd", "entry_point")):
            with self.subTest(name=name, attribute=attribute):
                document = json.loads(dumps(prepared))
                tool = next(t for t in document["tools"] if t["name"] == name)
                tool[attribute] = str(self.root / "other-file")
                changed = loads(PreparedPlan, json.dumps(document))
                self.assertEqual(verify_integrity(changed).status, "changed")

    def test_missing_tools_are_explicit_and_checks_not_authorized(self):
        missing = str(self.root / "missing")
        prepared = self.prepare(tool_paths={"python": missing, "node": missing,
                                           "jscpd_entry": missing})
        self.assertIsInstance(prepared, PreparedPlan)
        self.assertFalse(prepared.plan.resolved_commands)
        self.assertTrue(all(t.status == "unavailable" and t.reason for t in prepared.tools))

    def test_unreadable_resource_fails(self):
        (self.evaluator / "runner.py").unlink()
        self.assertIsInstance(self.prepare(), Failure)

    def test_version_mismatch_is_unavailable_and_never_authorizes_check(self):
        with patch("lesscode_baseline.config._probe", return_value="0.0.0"):
            prepared = self.prepare(tool_paths={"python": __import__("sys").executable})
        self.assertIsInstance(prepared, PreparedPlan)
        self.assertFalse(prepared.plan.resolved_commands)
        self.assertEqual(prepared.tools[0].observed_version, "0.0.0")
        self.assertIn("version mismatch", prepared.tools[0].reason)

    def test_relative_interpreter_and_profile_object_context(self):
        from lesscode_baseline.contracts import Profile
        # Relative paths are anchored at profile location, independent of cwd.
        executable = self.evaluator / "python.exe"
        executable.write_bytes(b"declared interpreter")
        with patch("lesscode_baseline.config._probe", return_value="3.12.10"):
            prepared = prepare(Profile.from_dict(self.profile), self.root / "baseline",
                               self.root / "candidate", self.root / "out", profile_path=self.path,
                               tool_paths={"python": "python.exe"})
        self.assertIsInstance(prepared, PreparedPlan)
        self.assertEqual(prepared.plan.resolved_commands[0].argv[0], str(executable.resolve()))

    def test_tool_file_change_is_identified(self):
        from lesscode_baseline.config import _inventory_hash, _resource
        from lesscode_baseline.contracts import ToolBinding
        from dataclasses import replace
        prepared = self.prepare(tool_paths={})
        executable = self.root / "interpreter"
        executable.write_bytes(b"old")
        resource = _resource("executable", executable)
        binding = ToolBinding("python", "3.12.10", "available", str(executable), None,
                              "3.12.10", _inventory_hash((resource,)), None, (resource,))
        prepared = replace(prepared, tools=(binding, *prepared.tools[1:]))
        executable.write_bytes(b"new")
        self.assertIn("_tools/python/executable", verify_integrity(prepared).changed_resources)

    def test_arbitrary_launcher_is_not_probed_or_run(self):
        launcher = self.evaluator / "custom-check.exe"
        launcher.write_bytes(b"project workload")
        for check in self.profile["checks"]:
            check["command"]["argv"][0] = str(launcher)
        self.save()
        with patch("lesscode_baseline.config._probe") as probe:
            prepared = self.prepare(tool_paths={})
        probe.assert_not_called()
        self.assertIsInstance(prepared, PreparedPlan)
        self.assertFalse(prepared.plan.resolved_commands)
        self.assertTrue(all(t.status == "unavailable" for t in prepared.tools))

    def test_threshold_wrong_unit_rejected(self):
        self.profile["thresholds"][0]["unit"] = "lines"
        self.save()
        with patch("lesscode_baseline.config._probe") as probe:
            self.assertIsInstance(self.prepare(), Failure)
        probe.assert_not_called()

    def test_overlapping_and_existing_output_rejected(self):
        for output in (self.root / "baseline" / "out.json", self.evaluator / "out.json",
                       self.root, self.path):
            with self.subTest(output=output):
                result = prepare(self.path, self.root / "baseline", self.root / "candidate", output)
                self.assertIsInstance(result, Failure)

    def test_source_evaluator_overlap_rejected(self):
        result = prepare(self.path, self.evaluator, self.root / "candidate", self.root / "out")
        self.assertIsInstance(result, Failure)

    def test_bad_threshold_and_limits_rejected_before_probes(self):
        for key, value in (("thresholds", [{"metric_key": "mystery", "operator": "absolute_maximum", "value": 1, "unit": "x"}]),
                           ("limits", {"timeout_seconds": 0, "max_log_bytes": 1})):
            with self.subTest(key=key):
                previous = deepcopy(self.profile)
                self.profile[key] = value
                self.save()
                self.assertIsInstance(self.prepare(), Failure)
                self.profile = previous

    def test_pilot_real_installed_tool_identities(self):
        fixture = ROOT / "tests/fixtures/real/boltons_mathutils/source"
        prepared = prepare(ROOT / "benchmarks/profiles/python.json", fixture, fixture,
                           self.root / "pilot.json")
        self.assertIsInstance(prepared, PreparedPlan)
        tools = {tool.name: tool for tool in prepared.tools}
        for name in ("python", "node", "lizard", "jscpd"):
            self.assertEqual(tools[name].status, "available", tools[name].reason)
            self.assertTrue(tools[name].resources)
        self.assertEqual(verify_integrity(prepared).status, "unchanged")


if __name__ == "__main__":
    unittest.main()
