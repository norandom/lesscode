"""Fixed probes cannot execute ambient runtime startup configuration."""

import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.config import _probe, prepare, verify_integrity
from lesscode_baseline.contracts import PreparedPlan


ROOT = Path(__file__).resolve().parents[1]


class ProbeEnvironmentTests(unittest.TestCase):
    def setUp(self):
        self.paths = json.loads((ROOT / ".tools/analyzers/entrypoints.json").read_text())
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_real_jscpd_preload_cannot_execute_during_preparation(self):
        marker = self.root / "preload-executed"
        preload = self.root / "preload.cjs"
        preload.write_text("require('node:fs').writeFileSync("
                           + json.dumps(str(marker)) + ", 'executed');\n", encoding="utf-8")
        node_options = "--require " + json.dumps(str(preload))
        fixture = ROOT / "tests/fixtures/real/boltons_mathutils/source"
        with patch.dict(os.environ, {"NODE_OPTIONS": node_options}):
            before = dict(os.environ)
            prepared = prepare(ROOT / "benchmarks/profiles/python.json", fixture, fixture,
                               self.root / "plan.json")
            self.assertEqual(dict(os.environ), before)
        self.assertIsInstance(prepared, PreparedPlan)
        tools = {tool.name: tool for tool in prepared.tools}
        for name, version in (("python", "3.12.10"), ("node", "24.22.1"),
                              ("lizard", "1.17.31"), ("jscpd", "4.0.5")):
            self.assertEqual(tools[name].status, "available", tools[name].reason)
            self.assertEqual(tools[name].observed_version, version)
        self.assertEqual(tools["jscpd"].entry_point, self.paths["jscpd_entry"])
        self.assertEqual(verify_integrity(prepared).status, "unchanged")
        self.assertFalse(marker.exists(), "jscpd version probe executed inherited Node preload")

    def test_actual_child_environment_excludes_unrelated_startup_configuration(self):
        ambient = {"NODE_PATH": str(self.root), "PYTHONPATH": str(self.root),
                   "HOME": str(self.root), "LESSCODE_UNDECLARED_SECRET": "not-for-probes"}
        if os.name == "nt":
            ambient["SystemRoot"] = str(self.root / "untrusted-windows-directory")
        with patch.dict(os.environ, ambient):
            before = dict(os.environ)
            environment = json.loads(_probe([self.paths["node"], "-e",
                                            "console.log(JSON.stringify(process.env))"]))
            self.assertEqual(dict(os.environ), before)
        for name in ambient.keys() - {"SystemRoot"}:
            self.assertTrue(name not in environment, name + " leaked into the probe environment")
        if os.name == "nt":
            self.assertTrue(Path(environment["SystemRoot"]).is_dir())
            self.assertNotEqual(environment["SystemRoot"], ambient["SystemRoot"])


if __name__ == "__main__":
    unittest.main()
