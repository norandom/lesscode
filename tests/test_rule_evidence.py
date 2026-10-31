"""Baseline-backed fixture infrastructure; no policy evaluator is implemented here."""

import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from lesscode_baseline import contracts as c
from lesscode_baseline.decision import compare
from lesscode_baseline.report import persist, recover
from lesscode_baseline.snapshot import capture_scope

FIXTURES = Path(__file__).parent / "fixtures/agent-quality-rules"


class BaselinePolicyFixtureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()

    def test_complete_storage_preserves_every_evaluation_outcome(self):
        for name in ("accepted", "rejected", "incomplete", "invalid"):
            with self.subTest(name=name):
                fixture = materialize(self.root / name, name)
                recovered = recover(fixture["run"])
                self.assertEqual(recovered.state, "complete")
                self.assertEqual(recovered.report.decision, name)
                self.assertEqual(recovered.report, fixture["report"])
                expected = fixture["expected"]
                by_id = {rule.rule_id: rule for rule in fixture["policy"].rules}
                self.assertEqual(set(by_id), {finding["rule_id"] for finding in expected["findings"]})
                metrics = {metric.key: metric for metric in recovered.report.candidate.metrics}
                for finding in expected["findings"]:
                    if finding["rule_id"] == "api":
                        self.assertEqual(finding["candidate"], recovered.report.candidate.checks[0].status)
                    else:
                        key = "source_bytes" if finding["rule_id"] == "size" else "duplication"
                        self.assertEqual(finding["candidate"], metrics[key].value)
                self.assertEqual(by_id["size"].absolute_maximum, 100)
                self.assertEqual(expected["baseline_decision"], name)
                self.assertEqual(expected["accepted"], name == "accepted")
                comparison = compare(recovered.report.baseline, recovered.report.candidate,
                                     fixture["prepared"].plan.profile, recovered.report.integrity)
                self.assertEqual(comparison.decision, name)

    def test_every_identity_resolves_to_real_materialized_bytes(self):
        fixture = materialize(self.root, "accepted")
        prepared, report = fixture["prepared"], fixture["report"]
        self.assertEqual(report.plan_hash, digest(c.dumps(prepared).encode()))
        self.assertEqual(prepared.plan.profile_hash, digest(c.dumps(prepared.plan.profile).encode()))
        self.assertEqual(c.loads(c.Profile, Path(prepared.profile_path).read_bytes()), prepared.plan.profile)
        resource_identity = json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                                        for r in prepared.evaluator_resources], separators=(",", ":")).encode()
        self.assertEqual(prepared.plan.evaluator_hash, digest(resource_identity))
        for resource in prepared.evaluator_resources:
            data = Path(resource.path).read_bytes()
            self.assertEqual((resource.content_hash, resource.size_bytes), (digest(data), len(data)))
        for side in ("baseline", "candidate"):
            observation = getattr(report, side)
            self.assertEqual(observation.snapshot, getattr(prepared.plan, side))
            self.assertEqual(capture_scope(Path(observation.snapshot.root), prepared.plan.profile.scope), observation.snapshot)
            for metric in observation.metrics:
                self.assertEqual(metric.scope_hash, observation.snapshot.scope_hash)
                self.assertTrue((fixture["run"] / metric.raw_artifact).is_file())
            self.assertEqual(set(prepared.plan.profile.required_metrics), {m.key for m in observation.metrics})
            for check in observation.checks:
                self.assertEqual((check.source_hash, check.plan_hash), (observation.snapshot.content_hash, report.plan_hash))
                self.assertIn(check.check_id, prepared.plan.profile.checks[0].expected_ids)
        index = json.loads((fixture["run"] / "artifacts.json").read_bytes())
        self.assertIn("plan.json", {a["path"] for a in index["artifacts"]})
        for artifact in index["artifacts"]:
            data = (fixture["run"] / artifact["path"]).read_bytes()
            self.assertEqual((artifact["content_hash"], artifact["size_bytes"]), (digest(data), len(data)))

    def test_corruption_and_interrupted_storage_are_not_evaluation_decisions(self):
        for name in ("corrupt", "partial"):
            fixture = materialize(self.root / name, name)
            self.assertEqual(recover(fixture["run"]).state, "incomplete")
            self.assertIsNone(recover(fixture["run"]).report)
            self.assertEqual(fixture["report"].decision, "accepted")

    def test_independent_findings_retain_insufficient_duplication(self):
        fixture = materialize(self.root, "incomplete")
        metric = fixture["report"].candidate.metrics[-1]
        self.assertEqual(metric.key, "duplication")
        self.assertEqual(metric.status, "unavailable")
        self.assertIsNone(metric.value)
        self.assertIn("inventory", metric.reason)
        self.assertIn("short.py", {f.path for f in fixture["report"].candidate.snapshot.files if f.size_bytes})
        self.assertEqual(fixture["expected"]["findings"][-1], {"rule_id": "duplication", "status": "insufficient", "candidate": None})

    def test_materialization_is_repeatable_without_fixed_machine_paths(self):
        first = materialize(self.root / "one", "accepted")
        second = materialize(self.root / "two", "accepted")
        self.assertEqual(first["report"].candidate.snapshot.content_hash, second["report"].candidate.snapshot.content_hash)
        self.assertEqual(first["prepared"].plan.profile_hash, second["prepared"].plan.profile_hash)
        self.assertEqual(first["expected"], second["expected"])
        self.assertNotEqual(first["report"].plan_hash, second["report"].plan_hash)
        with self.assertRaises(FileExistsError):
            materialize(self.root / "one", "accepted")

    def test_actual_short_nonempty_file_is_not_zero_duplication(self):
        from lesscode_baseline.metrics import measure_duplication
        from tests.test_metrics import DuplicationMetricTests
        node = shutil.which("node")
        entry = Path(__file__).resolve().parents[1] / ".tools/analyzers/node/node_modules/jscpd/bin/jscpd"
        if node is None or not entry.is_file():
            self.skipTest("pinned jscpd runtime unavailable")
        source = self.root / "source"
        source.mkdir()
        (source / "short.py").write_bytes(b"pass\n")
        (source / "other.py").write_bytes(DuplicationMetricTests.duplicate)
        scope = c.ScopeRules(("**/*",), (), (), (), (), ("**/*.py",))
        snapshot = capture_scope(source, scope)
        metric = measure_duplication(snapshot, Path(sys.executable), Path(node), entry,
                                     c.Limits(30, 100000), self.root / "logs")
        self.assertEqual(metric.status, "unavailable", metric.reason)
        self.assertIsNone(metric.value)
        self.assertIn("inventory", metric.reason)
        self.assertEqual(snapshot.files, capture_scope(source, scope).files)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def materialize(root, name="accepted"):
    """Build isolated, hash-consistent synthetic stores for downstream policy tests.

    Baseline contracts, capture_scope, decision.compare and persist/recover remain
    authoritative. This deliberately does not call policy evaluation. Absolute
    roots are captured at materialization time; source/profile identities remain
    portable but plan hashes necessarily change when the root changes. Final
    storage timestamps are owned by RunStore, not deterministic fixture data.
    """
    from lesscode_baseline.rules import contracts as rules

    expected_cases = json.loads((FIXTURES / "expected.json").read_text(encoding="utf-8-sig"))["cases"]
    outcome = "accepted" if name in ("corrupt", "partial") else name
    if outcome not in expected_cases:
        raise ValueError(f"unknown baseline fixture: {name}")
    root = Path(root).resolve()
    root.mkdir(parents=True, exist_ok=True)
    evaluator = root / "evaluator"
    evaluator.mkdir()  # Reusing a fixture root must not overwrite evidence.
    raw = root / "raw"
    raw.mkdir()
    source_bytes = (FIXTURES / ("short.py" if outcome == "incomplete" else "complete.py")).read_bytes()
    scope = c.ScopeRules(("**/*",), (), (), (), (), ("**/*.py",))
    snapshots = []
    for side in ("baseline", "candidate"):
        source = root / side
        source.mkdir()
        (source / "short.py").write_bytes(source_bytes)
        snapshot = capture_scope(source, scope)
        if isinstance(snapshot, c.Failure):
            raise AssertionError(snapshot.message)
        snapshots.append(snapshot)
    limits = c.Limits(30, 100000)
    oracle = evaluator / "oracle.py"
    oracle.write_bytes(b"# Synthetic fixture oracle; normalized evidence is supplied, not executed.\n")
    command = c.CommandSpec("api", ("python", "oracle.py"), ".", {}, limits)
    spec = c.CheckSpec("api", "native", True, command, "checks.json", "normalized", ("test.api",))
    required = ("complexity", "duplication", "source_bytes")
    profile = c.Profile(1, scope, (), (spec,), required, (), limits, ("oracle.py",))
    profile_path = evaluator / "profile.json"
    profile_path.write_text(c.dumps(profile), encoding="utf-8")
    resources = tuple(c.ResourceIdentity(logical, str(path), digest(path.read_bytes()), len(path.read_bytes()))
                      for logical, path in (("oracle.py", oracle), ("_context/profile.json", profile_path)))
    # Matches baseline's documented resource inventory identity, without calling
    # private helpers or substituting a sentinel hash.
    resource_hash = digest(json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                                      for r in resources], separators=(",", ":")).encode())
    protection = c.Protection("local_detection", None)
    resolved = c.ResolvedCommand("api", (str(Path(sys.executable).absolute()), str(oracle)),
                                 ".", {}, limits, "3.12.10")
    plan = c.Plan(1, profile, digest(c.dumps(profile).encode()), resource_hash,
                  *snapshots, (resolved,), protection)
    prepared = c.loads(c.PreparedPlan, c.dumps(c.PreparedPlan(
        1, plan, str(profile_path), str(evaluator), resources, ())))
    plan_hash = digest(c.dumps(prepared).encode())
    observations = []
    for side, snapshot in zip(("baseline", "candidate"), snapshots):
        folder = raw / side
        folder.mkdir()
        size_path = f"{side}/size.json"
        size_value = sum(f.size_bytes for f in snapshot.files if f.category == "product")
        (raw / size_path).write_text(json.dumps({"source_hash": snapshot.content_hash,
            "plan_hash": plan_hash, "source_bytes": size_value}), encoding="utf-8")
        metrics = [c.MetricEvidence("source_bytes", "available", size_value, "bytes",
            "raw-source-bytes-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, size_path)]
        complexity = c.ComplexityPayload("complexity", (c.FileComplexity("short.py", 1 if outcome == "incomplete" else 5),), (), 0, None, None, None, None)
        complexity_path = f"{side}/complexity.json"
        (raw / complexity_path).write_text(c.dumps(complexity), encoding="utf-8")
        metrics.append(c.MetricEvidence("complexity", "available", complexity, "CCN",
            "synthetic-complexity-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, complexity_path))
        if outcome != "incomplete":
            duplication = c.DuplicationPayload("duplication", 0, 20, 0, None, "weak", 0, (), ("short.py",), ())
            duplication_path = f"{side}/duplication.json"
            (raw / duplication_path).write_text(c.dumps(duplication), encoding="utf-8")
            metrics.append(c.MetricEvidence("duplication", "available", duplication, "ratio",
                "synthetic-pair-span-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, duplication_path))
        if outcome == "incomplete":
            reason = "duplication inventory omits nonempty short.py; cannot classify missing coverage as zero"
            artifact = f"{side}/duplication.json"
            (raw / artifact).write_text(json.dumps({"provenance": "synthetic omission case",
                "source_hash": snapshot.content_hash, "plan_hash": plan_hash,
                "scanned_paths": [], "omitted_nonempty_paths": ["short.py"], "reason": reason}), encoding="utf-8")
            metrics.append(c.MetricEvidence("duplication", "unavailable", None, "ratio",
                "synthetic-pair-span-v1", "synthetic-fixture", "1", snapshot.scope_hash, reason, artifact))
        status = "failed" if outcome == "rejected" and side == "candidate" else "passed"
        artifact = f"{side}/checks.json"
        check = c.CheckEvidence("test.api", status, "native", True, snapshot.content_hash, plan_hash, artifact)
        (raw / artifact).write_text(c.dumps(check), encoding="utf-8")
        observations.append(c.Observation(snapshot, tuple(metrics), (check,), (), ()))
    integrity = c.IntegrityResult("changed" if outcome == "invalid" else "unchanged",
                                 ("oracle.py",) if outcome == "invalid" else (), protection)
    comparison = compare(*observations, profile, integrity)
    report = c.RunReport(1, "fixture-" + outcome, plan_hash, *observations, integrity,
        comparison.decision, comparison.reasons, comparison.metric_deltas, c.FileDelta((), (), ()),
        limits, 0, ("Synthetic normalized evidence; no workload or policy evaluator executed.",),
        "2026-10-31T00:00:00Z", "2026-10-31T00:00:00Z", c.EnvironmentInfo("fixture", "3.12.10", False))
    run = root / "run"
    persist(report, run, artifact_root=raw, prepared_plan=prepared)
    if recover(run).state != "complete":
        raise AssertionError("fixture must pass baseline recovery before intentional damage")
    if name == "corrupt":
        (run / "plan.json").write_bytes((run / "plan.json").read_bytes() + b" ")
    elif name == "partial":
        (run / "artifacts.json").unlink()
    policy_data = json.loads((FIXTURES / "policy.json").read_text(encoding="utf-8-sig"))
    if outcome != "incomplete":
        policy_data["rules"] = [rule for rule in policy_data["rules"] if rule["rule_id"] != "duplication"]
    policy = rules.Policy.from_dict(policy_data)
    return {"run": run, "prepared": prepared, "report": report, "policy": policy,
            "expected": expected_cases[outcome]}

class EvidenceSelectionTests(unittest.TestCase):
    def setUp(self):
        from lesscode_baseline.rules.evidence import read_evidence
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        fixture = materialize(Path(temporary.name) / "selection")
        self.bundle = read_evidence(fixture["run"], fixture["policy"])

    def rule(self, kind="function_ccn", key=None, mode="absolute", **changes):
        from lesscode_baseline.rules.contracts import NumericRule
        from tests.test_rule_contracts import numeric
        metric_key = key if key else "complexity" if kind == "function_ccn" else "duplication"
        metric = next(m for m in self.bundle.report.candidate.metrics if m.key == metric_key)
        value = numeric("selected")
        value.update(selector={"kind": kind, "metric_key": key}, mode=mode,
                     evidence={field: getattr(metric, field) for field in
                               ("definition_id", "producer", "producer_version", "unit")})
        if kind == "duplication_clone_pairs":
            value["evidence"]["unit"] = "clone_pairs"
        elif kind in ("duplication_duplicated_tokens", "duplication_total_tokens"):
            value["evidence"]["unit"] = "tokens"
        value.update(changes)
        return NumericRule.from_dict(value)

    def select(self, rule):
        from lesscode_baseline.rules.evidence import select_observations
        return select_observations(self.bundle, rule)

    def alter(self, side, **changes):
        from dataclasses import replace
        observation = replace(getattr(self.bundle.report, side), **changes)
        self.bundle = replace(self.bundle, report=replace(self.bundle.report, **{side: observation}))

    def metric(self, side, key, **changes):
        from dataclasses import replace
        self.alter(side, metrics=tuple(replace(m, **changes) if m.key == key else m
                                       for m in getattr(self.bundle.report, side).metrics))

    def test_confirmed_zero_functions_missing_and_outside_scope_are_distinct(self):
        rule = self.rule()
        result = self.select(rule)
        self.assertEqual((result.candidate.status, result.candidate.entities), ("complete", ()))
        self.assertEqual(result.comparison_status, "not_required")
        self.metric("candidate", "complexity", status="unavailable", value=None, reason="inventory absent")
        self.assertIn("inventory absent", self.select(rule).candidate.reason)
        outside = self.rule(scope={"root": "outside", "include": ["**/*.py"], "exclude": []})
        self.assertEqual(self.select(outside).candidate.status, "unavailable")

    def test_absolute_scalar_retains_candidate_without_reference_and_checks(self):
        rule = self.rule("scalar_metric", "source_bytes")
        checks = self.bundle.report.candidate.checks
        self.alter("baseline", metrics=())
        result = self.select(rule)
        self.assertIsNone(result.baseline)
        self.assertEqual(result.candidate.entities[0].metric.value, 54)
        self.assertEqual(self.bundle.report.candidate.checks, checks)
        self.assertEqual(result.comparison_status, "not_required")

    def test_reference_missing_preserves_candidate_and_reasons(self):
        rule = self.rule("scalar_metric", "source_bytes", mode="ratchet")
        self.alter("baseline", metrics=())
        result = self.select(rule)
        self.assertEqual(result.candidate.status, "complete")
        self.assertEqual(result.baseline.status, "unavailable")
        self.assertEqual(result.comparison_status, "unavailable")
        self.assertTrue(result.comparison_reasons)

    def test_each_definition_difference_is_unavailable_without_losing_candidate(self):
        for field in ("definition_id", "producer", "producer_version", "unit", "scope_hash"):
            original = self.bundle
            with self.subTest(field=field):
                rule = self.rule("scalar_metric", "source_bytes", mode="ratchet")
                self.metric("baseline", "source_bytes", **{field: "f" * 64 if field == "scope_hash" else "other"})
                result = self.select(rule)
                self.assertEqual(result.comparison_status, "unavailable")
                self.assertEqual(result.candidate.status, "complete")
                self.assertIn(field, " ".join(result.comparison_reasons))
            self.bundle = original

    def test_function_projection_retains_locations_and_global_ids(self):
        from dataclasses import replace
        function = c.FunctionComplexity("short.py", "f", "f(x)", 2, 4, 12, 3, 8)
        payload = c.ComplexityPayload("complexity", (c.FileComplexity("short.py", 5),),
                                      (function,), 1, 12, 12, 12, 12)
        self.metric("candidate", "complexity", value=payload)
        result = self.select(self.rule())
        entity = result.candidate.entities[0]
        self.assertEqual((entity.metric.value, entity.long_name, entity.location),
                         (12, "f(x)", c.SourceSpan("short.py", 2, 4)))
        self.assertEqual(entity.evidence_refs, ("candidate/complexity.json",))
        again = self.select(replace(self.rule(), rule_id="overlapping"))
        self.assertEqual(entity.inventory_id, again.candidate.entities[0].inventory_id)

    def test_aggregate_narrowing_and_omitted_nonempty_files_are_unavailable(self):
        from dataclasses import replace
        snapshot = self.bundle.report.candidate.snapshot
        extra = replace(snapshot.files[0], path="other.py")
        self.alter("candidate", snapshot=replace(snapshot, files=(*snapshot.files, extra)))
        for kind, key in (("function_ccn", None), ("duplication_total_tokens", None)):
            with self.subTest(kind=kind):
                self.assertEqual(self.select(self.rule(kind, key)).candidate.status, "unavailable")
        narrowed = self.rule("scalar_metric", "source_bytes",
                             scope={"root": ".", "include": ["short.py"], "exclude": []})
        self.assertIn("population", self.select(narrowed).candidate.reason)

    def test_duplication_projections_keep_pair_span_ratio_above_one_and_denominator(self):
        payload = c.DuplicationPayload("duplication", 30, 20, 1.5, None, "weak", 150,
            (c.ClonePair(c.SourceSpan("short.py", 1, 2), c.SourceSpan("short.py", 3, 4)),),
            ("short.py",), ())
        self.metric("candidate", "duplication", value=payload)
        for kind, expected in (("duplication_clone_pairs", 1), ("duplication_duplicated_tokens", 30),
                               ("duplication_total_tokens", 20), ("duplication_pair_span_ratio", 1.5)):
            with self.subTest(kind=kind):
                result = self.select(self.rule(kind))
                self.assertEqual(result.candidate.entities[0].metric.value, expected)
                self.assertEqual(self.bundle.report.candidate.metrics[-1].value.total_tokens, 20)
        self.metric("candidate", "duplication", value=c.DuplicationPayload(
            "duplication", 0, 0, None, "zero token denominator", "weak", 0, (), ("short.py",), ("short.py",)))
        self.assertIn("denominator", self.select(self.rule("duplication_pair_span_ratio")).candidate.reason)

    def test_lloc_provenance_and_unsupported_structured_scalar(self):
        from dataclasses import replace
        metric = replace(self.bundle.report.candidate.metrics[0], key="lloc", value=7,
                         unit="logical lines", definition_id="supplied-lloc-v1", producer="owner")
        self.alter("candidate", metrics=(*self.bundle.report.candidate.metrics, metric))
        result = self.select(self.rule("scalar_metric", "lloc"))
        self.assertEqual(result.candidate.entities[0].metric, metric)
        rule = self.rule("scalar_metric", "complexity")
        self.assertEqual(self.select(rule).candidate.status, "unavailable")

    def test_selection_order_does_not_mutate_global_inventory(self):
        before = self.bundle
        rules = (self.rule(), self.rule("scalar_metric", "source_bytes"), self.rule("duplication_total_tokens"))
        first = [self.select(rule) for rule in rules]
        second = [self.select(rule) for rule in reversed(rules)]
        self.assertEqual(first, list(reversed(second)))
        self.assertEqual(self.bundle, before)
        global_ids = {e.inventory_id for e in self.bundle.candidate_inventory.entities}
        self.assertTrue(global_ids)
        self.assertTrue(all(e.inventory_id in global_ids for result in first for e in result.candidate.entities))

    def test_percentage_reference_zero_is_explicitly_unavailable(self):
        rule = self.rule("scalar_metric", "source_bytes", mode="ratchet",
                         change_budget={"kind": "percentage_increase", "maximum": 5})
        self.metric("baseline", "source_bytes", value=0)
        result = self.select(rule)
        self.assertEqual(result.comparison_status, "unavailable")
        self.assertIn("zero", " ".join(result.comparison_reasons))

    def test_missing_complexity_does_not_erase_scalar_and_selection_roundtrips(self):
        from lesscode_baseline.rules.contracts import RuleSelection
        complexity = self.rule()
        scalar = self.rule("scalar_metric", "source_bytes", mode="ratchet")
        self.alter("candidate", metrics=tuple(m for m in self.bundle.report.candidate.metrics
                                               if m.key != "complexity"))
        self.assertEqual(self.select(complexity).candidate.status, "unavailable")
        result = self.select(scalar)
        self.assertEqual(result.comparison_status, "comparable")
        self.assertEqual(result, c.loads(RuleSelection, c.dumps(result)))

    def test_empty_product_and_only_empty_files_are_ambiguous(self):
        from dataclasses import replace
        rule = self.rule()
        snapshot = self.bundle.report.candidate.snapshot
        for files in ((), (replace(snapshot.files[0], size_bytes=0),)):
            self.alter("candidate", snapshot=replace(snapshot, files=files))
            self.assertEqual(self.select(rule).candidate.status, "unavailable")

    def test_function_scope_filters_complete_inventory_without_changing_ids(self):
        from dataclasses import replace
        snapshot = self.bundle.report.candidate.snapshot
        files = (*snapshot.files, replace(snapshot.files[0], path="pkg/second.py"))
        self.alter("candidate", snapshot=replace(snapshot, files=files))
        functions = (c.FunctionComplexity("short.py", "f", "f()", 1, 2, 4, 2, 8),
                     c.FunctionComplexity("pkg/second.py", "g", "g()", 1, 2, 15, 2, 8))
        self.metric("candidate", "complexity", value=c.ComplexityPayload("complexity",
            tuple(c.FileComplexity(f.path, 5) for f in files), functions, 2, 4, 15, 9.5, 15))
        full = self.select(self.rule())
        narrow = self.select(self.rule(scope={"root": "pkg", "include": ["**/*.py"], "exclude": []}))
        self.assertEqual(len(full.candidate.entities), 2)
        self.assertEqual(len(narrow.candidate.entities), 1)
        self.assertIn(narrow.candidate.entities[0], full.candidate.entities)

    def test_whole_scope_added_files_are_comparable_with_complete_inventories(self):
        from dataclasses import replace
        snapshot = self.bundle.report.candidate.snapshot
        extra = replace(snapshot.files[0], path="new.py")
        self.alter("candidate", snapshot=replace(snapshot, files=(*snapshot.files, extra)))
        scalar = self.select(self.rule("scalar_metric", "source_bytes", mode="ratchet"))
        self.assertEqual(scalar.comparison_status, "comparable")
        self.metric("candidate", "duplication", value=c.DuplicationPayload(
            "duplication", 0, 30, 0, None, "weak", 0, (), ("short.py", "new.py"), ()))
        duplication = self.select(self.rule("duplication_total_tokens", mode="ratchet"))
        self.assertEqual(duplication.comparison_status, "comparable")
        self.assertEqual(duplication.candidate.entities[0].metric.value, 30)

    def test_policy_expectation_and_selected_units_are_enforced(self):
        from dataclasses import replace
        rule = self.rule("duplication_total_tokens")
        result = self.select(rule)
        self.assertEqual(result.candidate.entities[0].metric.unit, "tokens")
        wrong = replace(rule, evidence=replace(rule.evidence, unit="ratio"))
        self.assertIn("unit", self.select(wrong).candidate.reason)
        wrong = replace(rule, evidence=replace(rule.evidence, producer="unrecorded analyzer"))
        self.assertIn("producer", self.select(wrong).candidate.reason)

    def test_undefined_ratio_does_not_erase_zero_counts(self):
        self.metric("candidate", "duplication", value=c.DuplicationPayload(
            "duplication", 0, 0, None, "zero token denominator", "weak", 0, (), (), ("short.py",)))
        result = self.select(self.rule("duplication_clone_pairs"))
        self.assertEqual(result.candidate.entities[0].metric.value, 0)
        result = self.select(self.rule("duplication_total_tokens"))
        self.assertEqual(result.candidate.entities[0].metric.value, 0)
        self.assertEqual(self.select(self.rule("duplication_pair_span_ratio")).candidate.status, "unavailable")

    def test_report_address_retains_supplied_scalar_without_separate_raw_artifact(self):
        self.metric("candidate", "source_bytes", raw_artifact=None)
        result = self.select(self.rule("scalar_metric", "source_bytes"))
        self.assertEqual(result.candidate.entities[0].evidence_refs, ("report.json",))


class EntityContinuityTests(unittest.TestCase):
    def setUp(self):
        from lesscode_baseline.rules.evidence import read_evidence
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        fixture = materialize(Path(temporary.name) / "continuity")
        self.policy = fixture["policy"]
        self.bundle = read_evidence(fixture["run"], self.policy)
        from dataclasses import replace
        candidate = self.bundle.report.candidate
        source = "c" * 64
        candidate = replace(candidate,
            snapshot=replace(candidate.snapshot, content_hash=source,
                files=tuple(replace(f, content_hash="d" * 64) for f in candidate.snapshot.files)),
            checks=tuple(replace(check, source_hash=source) for check in candidate.checks))
        inventory = self.bundle.candidate_inventory
        self.bundle = replace(self.bundle, report=replace(self.bundle.report, candidate=candidate),
            candidate_inventory=replace(inventory, source_hash=source,
                entities=tuple(replace(e, source_hash=source) for e in inventory.entities)))

    def functions(self, side, names=("work()",), path="short.py", start=1):
        from dataclasses import replace
        from lesscode_baseline.rules.contracts import EvidenceInventory, NumericRule
        from lesscode_baseline.rules.evidence import select_observations
        from tests.test_rule_contracts import numeric
        observation = getattr(self.bundle.report, side)
        functions = tuple(c.FunctionComplexity(path, name, name, start + i, start + i, 56, 1, 2)
                          for i, name in enumerate(names))
        payload = c.ComplexityPayload("complexity", (c.FileComplexity(path, 5),), functions,
                                      len(functions), 56, 56, 56, 56)
        metrics = tuple(replace(m, value=payload) if m.key == "complexity" else m
                        for m in observation.metrics)
        self.bundle = replace(self.bundle, report=replace(self.bundle.report,
                              **{side: replace(observation, metrics=metrics)}))
        metric = next(m for m in metrics if m.key == "complexity")
        rule = numeric()
        rule.update(selector={"kind": "function_ccn", "metric_key": None},
                    evidence={key: getattr(metric, key) for key in
                              ("unit", "definition_id", "producer", "producer_version")})
        selection = select_observations(self.bundle, NumericRule.from_dict(rule))
        # Absolute selection reads candidate only; use the same public selector
        # on a temporary candidate observation to address the baseline side.
        if side == "baseline":
            swapped = replace(self.bundle, report=replace(self.bundle.report, candidate=self.bundle.report.baseline))
            selection = select_observations(swapped, NumericRule.from_dict(rule))
        inventory = getattr(self.bundle, side + "_inventory")
        self.bundle = replace(self.bundle, **{side + "_inventory": EvidenceInventory(
            inventory.source_hash, "complete", None, selection.candidate.entities)})

    def unchanged_file(self):
        from dataclasses import replace
        before = self.bundle.report.baseline.snapshot.files
        observation = self.bundle.report.candidate
        snapshot = replace(observation.snapshot, files=before)
        self.bundle = replace(self.bundle, report=replace(self.bundle.report,
                              candidate=replace(observation, snapshot=snapshot)))

    def continuity(self, treatments=()):
        from lesscode_baseline.rules.evidence import establish_continuity
        return establish_continuity(self.bundle, self.policy, treatments)

    def claim(self, kind, before=None, after=None):
        from lesscode_baseline.rules.contracts import EntityTreatment, policy_hash
        return EntityTreatment(policy_hash(self.policy), self.bundle.baseline_inventory.source_hash,
            self.bundle.candidate_inventory.source_hash, kind, before, after, "owner", "Explicit treatment; no semantic proof")

    def test_identical_file_and_inventory_match_across_different_source_hashes(self):
        self.functions("baseline")
        self.functions("candidate")
        self.unchanged_file()
        result = self.continuity()
        self.assertEqual([r.kind for r in result], ["matched"])
        self.assertEqual(result[0].baseline_id, self.bundle.baseline_inventory.entities[0].inventory_id)
        self.assertIn("identical", result[0].rationale)
        self.assertEqual(self.continuity(), result)

    def test_changed_bytes_never_inherit_debt_from_same_name_and_location(self):
        self.functions("baseline")
        self.functions("candidate")
        result = self.continuity()
        self.assertEqual([r.kind for r in result], ["unresolved", "unresolved"])
        self.assertTrue(all((r.baseline_id is None) != (r.candidate_id is None) for r in result))

    def test_changed_location_or_name_in_identical_file_is_unresolved(self):
        self.functions("baseline")
        self.functions("candidate", ("renamed()",), start=2)
        self.unchanged_file()
        self.assertTrue(all(r.kind == "unresolved" for r in self.continuity()))

    def test_explicit_match_is_preserved_as_producer_claim(self):
        self.functions("baseline")
        self.functions("candidate", ("renamed()",))
        claim = self.claim("matched", self.bundle.baseline_inventory.entities[0].inventory_id,
                           self.bundle.candidate_inventory.entities[0].inventory_id)
        self.assertEqual(self.continuity((claim,)), (claim,))

    def test_split_can_be_explicit_removal_and_new_without_inherited_allowance(self):
        self.functions("baseline")
        self.functions("candidate", ("first()", "second()"))
        claims = (self.claim("removed", self.bundle.baseline_inventory.entities[0].inventory_id),
                  *(self.claim("new", after=e.inventory_id) for e in self.bundle.candidate_inventory.entities))
        self.assertEqual(set(self.continuity(claims)), set(claims))
        self.assertTrue(all(r.kind != "matched" for r in self.continuity()))

    def test_duplicate_conflicting_absent_and_wrong_source_claims_are_invalid(self):
        from dataclasses import replace
        self.functions("baseline")
        self.functions("candidate")
        claim = self.claim("matched", self.bundle.baseline_inventory.entities[0].inventory_id,
                           self.bundle.candidate_inventory.entities[0].inventory_id)
        for records in ((claim, claim), (claim, self.claim("removed", claim.baseline_id)),
                        (replace(claim, candidate_id="f" * 64),),
                        (replace(claim, candidate_source_hash="f" * 64),),
                        (replace(claim, policy_hash="f" * 64),)):
            with self.subTest(records=records), self.assertRaises(c.ContractError):
                self.continuity(records)

    def test_explicit_unresolved_prevents_automatic_match(self):
        self.functions("baseline")
        self.functions("candidate")
        self.unchanged_file()
        claim = self.claim("unresolved", self.bundle.baseline_inventory.entities[0].inventory_id,
                           self.bundle.candidate_inventory.entities[0].inventory_id)
        self.assertEqual(self.continuity((claim,)), (claim,))

    def test_aggregate_requires_identical_product_population(self):
        self.assertTrue(all(r.kind == "unresolved" for r in self.continuity()))
        self.unchanged_file()
        result = self.continuity()
        self.assertTrue(result)
        self.assertTrue(all(r.kind == "matched" for r in result))

    def test_reference_binding_is_checked_even_without_supplied_treatments(self):
        from dataclasses import replace
        from lesscode_baseline.rules.contracts import PolicyReference
        self.policy = replace(self.policy, reference=PolicyReference("f" * 64, self.bundle.profile_hash))
        with self.assertRaises(c.ContractError):
            self.continuity()

    def test_rename_keeps_identical_bytes_but_requires_explicit_treatment(self):
        from dataclasses import replace
        self.functions("baseline")
        candidate = self.bundle.report.candidate
        files = tuple(replace(f, path="renamed.py", content_hash=self.bundle.report.baseline.snapshot.files[0].content_hash)
                      for f in candidate.snapshot.files)
        self.bundle = replace(self.bundle, report=replace(self.bundle.report,
            candidate=replace(candidate, snapshot=replace(candidate.snapshot, files=files))))
        self.functions("candidate", path="renamed.py")
        self.assertTrue(all(r.kind == "unresolved" for r in self.continuity()))

    def test_measurement_identity_changes_do_not_automatically_match(self):
        from dataclasses import replace
        self.functions("baseline")
        self.functions("candidate")
        self.unchanged_file()
        original = self.bundle
        for field, value in (("definition_id", "other"), ("producer", "other"),
                             ("producer_version", "2"), ("unit", "other")):
            with self.subTest(field=field):
                entity = original.candidate_inventory.entities[0]
                inventory = replace(original.candidate_inventory,
                    entities=(replace(entity, metric=replace(entity.metric, **{field: value})),))
                self.bundle = replace(original, candidate_inventory=inventory)
                self.assertTrue(all(r.kind == "unresolved" for r in self.continuity()))

    def test_explicit_removal_retained_when_candidate_inventory_empty(self):
        from dataclasses import replace
        self.functions("baseline")
        self.bundle = replace(self.bundle, candidate_inventory=replace(self.bundle.candidate_inventory, entities=()))
        claim = self.claim("removed", self.bundle.baseline_inventory.entities[0].inventory_id)
        self.assertEqual(self.continuity((claim,)), (claim,))
        self.assertEqual([r.kind for r in self.continuity()], ["unresolved"])

    def test_multiple_claims_are_deterministic_and_merges_stay_unresolved(self):
        self.functions("baseline", ("first()", "second()"))
        self.functions("candidate")
        self.assertTrue(all(r.kind == "unresolved" for r in self.continuity()))
        claims = (*(self.claim("removed", e.inventory_id) for e in self.bundle.baseline_inventory.entities),
                  self.claim("new", after=self.bundle.candidate_inventory.entities[0].inventory_id))
        self.assertEqual(self.continuity(claims), self.continuity(tuple(reversed(claims))))


class EvidenceAdmissionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()

    def read(self, fixture, policy=None):
        from lesscode_baseline.rules.evidence import read_evidence
        return read_evidence(fixture['run'], policy or fixture['policy'])

    def rewrite(self, fixture, filename, mutate):
        path = fixture['run'] / filename
        value = json.loads(path.read_bytes())
        mutate(value)
        path.write_text(json.dumps(value), encoding='utf-8')
        index_path = fixture['run'] / 'artifacts.json'
        index = json.loads(index_path.read_bytes())
        for artifact in index['artifacts']:
            if artifact['path'] == filename:
                artifact['content_hash'] = digest(path.read_bytes())
                artifact['size_bytes'] = len(path.read_bytes())
        index_path.write_text(json.dumps(index), encoding='utf-8')

    def test_preserves_complete_nonaccepted_reports_and_independent_addresses(self):
        from lesscode_baseline.rules.contracts import EvidenceBundle
        for outcome in ('accepted', 'rejected', 'incomplete', 'invalid'):
            with self.subTest(outcome=outcome):
                fixture = materialize(self.root / outcome, outcome)
                bundle = self.read(fixture)
                self.assertIsInstance(bundle, EvidenceBundle)
                self.assertEqual(bundle.report, fixture['report'])
                self.assertEqual(bundle.profile_hash, fixture['prepared'].plan.profile_hash)
                for inventory in (bundle.baseline_inventory, bundle.candidate_inventory):
                    self.assertEqual(inventory.status, 'complete')
                    self.assertIsNone(inventory.reason)
                    self.assertTrue(inventory.entities)
                    self.assertTrue(any(e.selector.metric_key == 'source_bytes' for e in inventory.entities))

    def test_corrupt_hash_invalid_but_missing_completion_insufficient(self):
        for name, kind in (('corrupt', 'invalid'), ('partial', 'insufficient')):
            fixture = materialize(self.root / name, name)
            result = self.read(fixture)
            self.assertEqual(result.kind, kind)
            self.assertTrue(result.diagnostics)

    def test_report_only_store_is_insufficient_and_retains_outcome_in_diagnostic(self):
        fixture = materialize(self.root, 'rejected')
        (fixture['run'] / 'plan.json').unlink()
        index_path = fixture['run'] / 'artifacts.json'
        index = json.loads(index_path.read_bytes())
        index['artifacts'] = [a for a in index['artifacts'] if a['path'] != 'plan.json']
        index_path.write_text(json.dumps(index), encoding='utf-8')
        self.assertEqual(recover(fixture['run']).state, 'complete')
        result = self.read(fixture)
        self.assertEqual(result.kind, 'insufficient')
        self.assertIn('rejected', ' '.join(result.diagnostics))
        self.assertIn('plan.json', ' '.join(result.diagnostics))

    def test_reference_must_select_this_runs_baseline_and_profile(self):
        from dataclasses import replace
        from lesscode_baseline.rules.contracts import EvidenceBundle, PolicyReference
        fixture = materialize(self.root)
        reference = PolicyReference(fixture['report'].baseline.snapshot.content_hash,
                                    fixture['prepared'].plan.profile_hash)
        policy = replace(fixture['policy'], reference=reference)
        self.assertIsInstance(self.read(fixture, policy), EvidenceBundle)
        for field in ('source_hash', 'profile_hash'):
            wrong = replace(policy, reference=replace(reference, **{field: 'f' * 64}))
            self.assertEqual(self.read(fixture, wrong).kind, 'invalid')

    def test_rehashed_artifacts_do_not_bypass_relationship_validation(self):
        cases = {
            'plan_hash': ('plan.json', lambda d: d['plan'].update(profile_hash='f' * 64)),
            'snapshot': ('report.json', lambda d: d['candidate']['snapshot'].update(content_hash='f' * 64)),
            'check_source': ('report.json', lambda d: d['candidate']['checks'][0].update(source_hash='f' * 64)),
            'check_plan': ('report.json', lambda d: d['candidate']['checks'][0].update(plan_hash='f' * 64)),
            'check_kind': ('report.json', lambda d: d['candidate']['checks'][0].update(required=False)),
            'check_id': ('report.json', lambda d: d['candidate']['checks'][0].update(check_id='undeclared')),
            'metric_scope': ('report.json', lambda d: d['candidate']['metrics'][0].update(scope_hash='f' * 64)),
        }
        for name, (filename, mutate) in cases.items():
            with self.subTest(name=name):
                fixture = materialize(self.root / name)
                self.rewrite(fixture, filename, mutate)
                self.assertEqual(recover(fixture['run']).state, 'complete')
                result = self.read(fixture)
                self.assertEqual(result.kind, 'invalid')
                self.assertTrue(result.diagnostics)

    def test_missing_indexed_plan_is_insufficient(self):
        fixture = materialize(self.root)
        (fixture['run'] / 'plan.json').unlink()
        result = self.read(fixture)
        self.assertEqual(result.kind, 'insufficient')
        self.assertTrue(result.diagnostics)

    def test_partial_storage_retains_each_nonaccepted_outcome_and_diagnostics(self):
        from lesscode_baseline.rules.contracts import RuleFailure
        for outcome in ('rejected', 'incomplete', 'invalid', 'baseline_ineligible'):
            for missing in ('artifacts.json', 'manifest.json', 'plan.json'):
                with self.subTest(outcome=outcome, missing=missing):
                    fixture = materialize(self.root / outcome / missing,
                                          'accepted' if outcome == 'baseline_ineligible' else outcome)
                    if outcome == 'baseline_ineligible':
                        def alter(report):
                            report['baseline']['checks'] = []
                            report['decision'] = outcome
                            report['reasons'] = ['baseline test.api: missing']
                        self.rewrite(fixture, 'report.json', alter)
                    report = c.loads(c.RunReport, (fixture['run'] / 'report.json').read_bytes())
                    self.assertTrue(report.reasons)
                    self.assertTrue(report.limitations)
                    (fixture['run'] / missing).unlink()
                    recovered = recover(fixture['run'])
                    result = self.read(fixture)
                    self.assertIsInstance(result, RuleFailure)
                    self.assertEqual(result.kind, 'insufficient')
                    for diagnostic in (*recovered.diagnostics, f'upstream outcome: {outcome}',
                                       *report.reasons, *report.limitations):
                        self.assertIn(diagnostic, result.diagnostics)

    def test_absent_or_malformed_partial_report_has_no_invented_outcome(self):
        for malformed in (False, True):
            with self.subTest(malformed=malformed):
                fixture = materialize(self.root / str(malformed), 'rejected')
                (fixture['run'] / 'artifacts.json').unlink()
                report_path = fixture['run'] / 'report.json'
                if malformed:
                    report_path.write_bytes(b'{}')
                else:
                    report_path.unlink()
                result = self.read(fixture)
                self.assertEqual(result.kind, 'invalid' if malformed else 'insufficient')
                self.assertFalse(any(d.startswith('upstream outcome:') for d in result.diagnostics))

    def test_baseline_ineligible_and_missing_checks_are_preserved(self):
        fixture = materialize(self.root)
        def alter(report):
            report['baseline']['checks'] = []
            report['decision'] = 'baseline_ineligible'
            report['reasons'] = ['baseline test.api: missing']
        self.rewrite(fixture, 'report.json', alter)
        bundle = self.read(fixture)
        self.assertEqual(bundle.report.decision, 'baseline_ineligible')
        self.assertEqual(bundle.report.reasons, ('baseline test.api: missing',))
        self.assertEqual(bundle.report.baseline.checks, ())

    def test_profile_and_inventory_hashes_checked_even_with_rebound_plan(self):
        for field in ('profile_hash', 'evaluator_hash'):
            with self.subTest(field=field):
                fixture = materialize(self.root / field)
                self.rewrite(fixture, 'plan.json', lambda d: d['plan'].update({field: 'f' * 64}))
                prepared = c.loads(c.PreparedPlan, (fixture['run'] / 'plan.json').read_bytes())
                plan_hash = digest(c.dumps(prepared).encode())
                def bind_report(report):
                    report['plan_hash'] = plan_hash
                    for side in ('baseline', 'candidate'):
                        for check in report[side]['checks']:
                            check['plan_hash'] = plan_hash
                self.rewrite(fixture, 'report.json', bind_report)
                self.rewrite(fixture, 'manifest.json', lambda d: d.update(plan_hash=plan_hash))
                index_path = fixture['run'] / 'artifacts.json'
                index = json.loads(index_path.read_bytes())
                index['plan_hash'] = plan_hash
                index_path.write_text(json.dumps(index), encoding='utf-8')
                self.assertEqual(recover(fixture['run']).state, 'complete')
                result = self.read(fixture)
                self.assertEqual(result.kind, 'invalid')
                self.assertIn(field, result.diagnostics[0])

    def test_corruption_takes_precedence_over_missing_completion_evidence(self):
        cases = (
            ('malformed_index', 'plan.json', 'artifacts.json', b'{}'),
            ('malformed_manifest', 'artifacts.json', 'manifest.json', b'{}'),
            ('malformed_report', 'plan.json', 'report.json', b'{}'),
            ('changed_summary', 'plan.json', 'summary.md', b'changed'),
        )
        for name, missing, corrupt, content in cases:
            with self.subTest(name=name):
                fixture = materialize(self.root / name)
                (fixture['run'] / missing).unlink()
                (fixture['run'] / corrupt).write_bytes(content)
                result = self.read(fixture)
                self.assertEqual(result.kind, 'invalid')
                self.assertTrue(result.diagnostics)

    def test_present_binding_corruption_precedes_missing_index(self):
        fixture = materialize(self.root)
        self.rewrite(fixture, 'report.json',
                     lambda d: d['candidate']['checks'][0].update(plan_hash='f' * 64))
        (fixture['run'] / 'artifacts.json').unlink()
        self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_present_completion_identity_conflict_precedes_missing_plan(self):
        fixture = materialize(self.root)
        index_path = fixture['run'] / 'artifacts.json'
        index = json.loads(index_path.read_bytes())
        index['run_id'] = 'foreign-run'
        index_path.write_text(json.dumps(index), encoding='utf-8')
        (fixture['run'] / 'plan.json').unlink()
        self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_clean_missing_completion_records_are_insufficient(self):
        for missing in ('artifacts.json', 'manifest.json', 'plan.json', 'report.json'):
            with self.subTest(missing=missing):
                fixture = materialize(self.root / missing)
                (fixture['run'] / missing).unlink()
                self.assertEqual(self.read(fixture).kind, 'insufficient')

    def test_present_report_binding_corruption_precedes_missing_plan(self):
        fixture = materialize(self.root)
        self.rewrite(fixture, 'report.json',
                     lambda d: d['candidate']['checks'][0].update(source_hash='f' * 64))
        (fixture['run'] / 'plan.json').unlink()
        self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_report_only_store_does_not_hide_report_binding_corruption(self):
        fixture = materialize(self.root)
        self.rewrite(fixture, 'report.json',
                     lambda d: d['candidate']['checks'][0].update(source_hash='f' * 64))
        (fixture['run'] / 'plan.json').unlink()
        index_path = fixture['run'] / 'artifacts.json'
        index = json.loads(index_path.read_bytes())
        index['artifacts'] = [a for a in index['artifacts'] if a['path'] != 'plan.json']
        index_path.write_text(json.dumps(index), encoding='utf-8')
        self.assertEqual(recover(fixture['run']).state, 'complete')
        self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_unindexed_report_references_are_invalid_even_with_missing_plan(self):
        for category in ('metric', 'check', 'failure', 'stdout', 'stderr'):
            for missing_plan in (False, True):
                with self.subTest(category=category, missing_plan=missing_plan):
                    fixture = materialize(self.root / f'{category}-{missing_plan}')
                    def change(report):
                        observation = report['candidate']
                        reference = 'candidate/unindexed.json'
                        if category == 'metric':
                            observation['metrics'][0]['raw_artifact'] = reference
                        elif category == 'check':
                            observation['checks'][0]['diagnostic_artifact'] = reference
                        elif category == 'failure':
                            observation['failures'] = [c.Failure('fixture', 'fixture diagnostic',
                                None, None, None, False, reference).to_dict()]
                        else:
                            execution = c.ExecutionResult(fixture['prepared'].plan.resolved_commands[0],
                                0, 'completed', 0, 'candidate/checks.json', 'candidate/checks.json',
                                '2026-10-31T00:00:00Z', 'f' * 64).to_dict()
                            execution[category + '_artifact'] = reference
                            observation['executions'] = [execution]
                    self.rewrite(fixture, 'report.json', change)
                    if missing_plan:
                        (fixture['run'] / 'plan.json').unlink()
                    result = self.read(fixture)
                    self.assertEqual(result.kind, 'invalid')
                    self.assertIn('unindexed', ' '.join(result.diagnostics))

    def test_final_index_conflicts_with_unfinalized_manifest(self):
        fixture = materialize(self.root)
        self.rewrite(fixture, 'manifest.json', lambda d: d.update(state='running'))
        self.assertEqual(self.read(fixture).kind, 'invalid')

    def rebind_plan_identity(self, fixture):
        prepared = c.loads(c.PreparedPlan, (fixture['run'] / 'plan.json').read_bytes())
        plan_hash = digest(c.dumps(prepared).encode())
        self.rewrite(fixture, 'manifest.json', lambda d: d.update(plan_hash=plan_hash))
        index_path = fixture['run'] / 'artifacts.json'
        index = json.loads(index_path.read_bytes())
        index['plan_hash'] = plan_hash
        index_path.write_text(json.dumps(index), encoding='utf-8')

    def test_plan_internal_corruption_without_report_is_invalid(self):
        mutations = {
            'profile_hash': lambda d: d['plan'].update(profile_hash='f' * 64),
            'evaluator_hash': lambda d: d['plan'].update(evaluator_hash='f' * 64),
            'snapshot_inventory': lambda d: d['plan']['candidate']['files'][0].update(content_hash='f' * 64),
            'snapshot_scope': lambda d: d['plan']['candidate'].update(scope_hash='f' * 64),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                fixture = materialize(self.root / name)
                self.rewrite(fixture, 'plan.json', mutate)
                self.rebind_plan_identity(fixture)
                (fixture['run'] / 'report.json').unlink()
                self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_report_snapshot_corruption_without_plan_is_invalid(self):
        for report_only in (False, True):
            with self.subTest(report_only=report_only):
                fixture = materialize(self.root / str(report_only))
                self.rewrite(fixture, 'report.json', lambda d:
                             d['candidate']['snapshot']['files'][0].update(content_hash='f' * 64))
                (fixture['run'] / 'plan.json').unlink()
                if report_only:
                    index_path = fixture['run'] / 'artifacts.json'
                    index = json.loads(index_path.read_bytes())
                    index['artifacts'] = [a for a in index['artifacts'] if a['path'] != 'plan.json']
                    index_path.write_text(json.dumps(index), encoding='utf-8')
                    self.assertEqual(recover(fixture['run']).state, 'complete')
                self.assertEqual(self.read(fixture).kind, 'invalid')

    def test_available_reference_bindings_checked_without_unrelated_records(self):
        from dataclasses import replace
        from lesscode_baseline.rules.contracts import PolicyReference
        for missing, field in (('report.json', 'source_hash'), ('report.json', 'profile_hash'),
                               ('plan.json', 'source_hash')):
            with self.subTest(missing=missing, field=field):
                fixture = materialize(self.root / (missing + field))
                reference = PolicyReference(fixture['report'].baseline.snapshot.content_hash,
                                            fixture['prepared'].plan.profile_hash)
                (fixture['run'] / missing).unlink()
                valid_policy = replace(fixture['policy'], reference=reference)
                self.assertEqual(self.read(fixture, valid_policy).kind, 'insufficient')
                wrong_policy = replace(valid_policy, reference=replace(reference, **{field: 'f' * 64}))
                self.assertEqual(self.read(fixture, wrong_policy).kind, 'invalid')
