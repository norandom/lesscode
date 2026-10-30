"""Validate fixture truth without invoking the future evaluator."""

import ast
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

from lesscode_baseline.contracts import CheckEvidence


ROOT = Path(__file__).parent / "fixtures" / "python"


class FixtureTests(unittest.TestCase):
    def manifest(self):
        return json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))

    def test_pinned_inventory_and_repeatable_manifest_identity(self):
        manifest = self.manifest()
        files = manifest["files"]
        actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*")
                  if p.is_file() and p.name != "manifest.json"}
        self.assertEqual(set(files), actual)
        for path, expected in files.items():
            raw = (ROOT / path).read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), expected["sha256"], path)
            self.assertEqual(len(raw), expected["bytes"], path)
            self.assertNotIn(b"\r", raw, path)
        canonical = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(), manifest["inventory_sha256"])

    def test_known_pair_behavior_independently(self):
        truth = json.loads((ROOT / "oracle/golden.json").read_text())
        for pair in self.manifest()["pairs"]:
            for side in ("baseline", "candidate"):
                module = runpy.run_path(str(ROOT / pair[side]))
                actual = [module["clamp"](*case["args"]) for case in truth]
                self.assertEqual(actual, pair["outputs"][side])
                if side == "baseline" or pair["behavior_preserved"]:
                    self.assertEqual(actual, [case["expected"] for case in truth])
            if pair["id"] == "unchanged":
                self.assertEqual((ROOT / pair["baseline"]).read_bytes(),
                                 (ROOT / pair["candidate"]).read_bytes())
            if pair["id"] == "shorter_broken":
                self.assertLess((ROOT / pair["candidate"]).stat().st_size,
                                (ROOT / pair["baseline"]).stat().st_size)

    def test_hand_counted_structure_and_exact_duplicate_locations(self):
        manifest = self.manifest()
        for pair in manifest["pairs"]:
            for side in ("baseline", "candidate"):
                tree = ast.parse((ROOT / pair[side]).read_text())
                function, = [n for n in tree.body if isinstance(n, ast.FunctionDef)]
                expected = pair["complexity"][side]
                self.assertEqual((function.name, function.lineno, function.end_lineno),
                                 (expected["name"], expected["start_line"], expected["end_line"]))
                # These deliberately simple functions contain only sequential if decisions.
                self.assertFalse(any(isinstance(n, (ast.For, ast.While, ast.BoolOp,
                    ast.IfExp, ast.Try, ast.Match, ast.comprehension)) for n in ast.walk(function)))
                self.assertEqual(1 + sum(isinstance(n, ast.If) for n in ast.walk(function)),
                                 expected["ccn"])
        duplicate = manifest["duplicate_truth"]
        first, second = [ROOT / path for path in duplicate["paths"]]
        self.assertEqual(first.read_bytes(), second.read_bytes())
        self.assertEqual(len(duplicate["candidate_paths"]), 1)
        self.assertEqual(duplicate["expected_clone_pairs"], 1)

    def test_external_oracle_reports_real_outcomes_in_both_formats(self):
        for pair in self.manifest()["pairs"]:
            for side in ("baseline", "candidate"):
                for fmt in ("normalized", "junit"):
                    with self.subTest(pair=pair["id"], side=side, format=fmt), tempfile.TemporaryDirectory() as tmp:
                        output = Path(tmp) / "result"
                        source = ROOT / pair[side]
                        result = subprocess.run([sys.executable, str(ROOT / "oracle/check.py"),
                            str(source), str(output), fmt, "a" * 64],
                            capture_output=True, text=True, timeout=10)
                        passed = side == "baseline" or pair["behavior_preserved"]
                        self.assertEqual(result.returncode, 0 if passed else 1, result.stderr)
                        if fmt == "normalized":
                            records = [CheckEvidence.from_dict(v) for v in json.loads(output.read_text())]
                            self.assertEqual([r.check_id for r in records], self.manifest()["expected_ids"])
                            self.assertEqual(all(r.status == "passed" for r in records), passed)
                            self.assertTrue(all(r.required and r.kind in ("golden", "invariant") for r in records))
                            self.assertTrue(all(r.source_hash == hashlib.sha256(source.read_bytes()).hexdigest()
                                                and r.plan_hash == "a" * 64 for r in records))
                        else:
                            suite = ET.parse(output).getroot()
                            self.assertEqual([c.attrib["name"] for c in suite], self.manifest()["expected_ids"])
                            self.assertEqual(not bool(suite.findall("testcase/failure")), passed)

    def test_evidence_examples_and_adversarial_inputs(self):
        manifest = self.manifest()
        for example in manifest["normalized_examples"]:
            records = json.loads((ROOT / example["path"]).read_text())
            self.assertEqual([CheckEvidence.from_dict(v).status for v in records], example["statuses"])
        for example in manifest["junit_examples"]:
            suite = ET.parse(ROOT / example["path"]).getroot()
            self.assertEqual(len(suite.findall("testcase")), example["testcases"])
        for path in manifest["absent_outputs"]:
            self.assertFalse((ROOT / path).exists())
        for case in manifest["suppression"]:
            self.assertIn(case["marker"], (ROOT / case["path"]).read_text())
        for change in manifest["tampering"]:
            self.assertNotEqual((ROOT / change["before"]).read_bytes(),
                                (ROOT / change["after"]).read_bytes())


if __name__ == "__main__":
    unittest.main()
