"""Schedule identities and controlled conditions, independently of run success."""

import unittest

from lesscode_baseline import contracts as c
from lesscode_baseline.experiments import reconcile
from tests.test_experiment_contracts import manifest, trial


def evaluated(delta=-2, decision="accepted", candidate="passed", baseline="passed"):
    from tests.test_contracts import report
    value = report()
    value["decision"] = decision
    value["baseline"]["checks"][0]["status"] = baseline
    value["candidate"]["checks"][0]["status"] = candidate
    value["baseline"]["metrics"][0]["raw_artifact"] = None
    value["candidate"]["metrics"][0]["raw_artifact"] = None
    value["baseline"]["checks"][0]["diagnostic_artifact"] = None
    value["candidate"]["checks"][0]["diagnostic_artifact"] = None
    value["candidate"]["metrics"][0]["value"] = 12 + delta
    return c.RunReport.from_dict(value)


class AggregationTests(unittest.TestCase):
    def aggregate(self, records=(), reports=None, data=None, **kwargs):
        from lesscode_baseline.experiments import aggregate
        return aggregate(c.ExperimentManifest.from_dict(data or manifest()), tuple(records),
                         reports=reports, **kwargs)

    def repetitions(self, count):
        data = manifest()
        data["planned_trials"] = [dict(p, trial_id=f"{p['arm_id']}-{rep}", repetition=rep)
                                  for rep in range(1, count + 1) for p in data["planned_trials"]]
        records = [producer(p["arm_id"], trial_id=p["trial_id"], repetition=p["repetition"])
                   for p in data["planned_trials"]]
        return data, records

    def test_hand_calculated_counts_include_rejected_incomplete_missing_invalid(self):
        data, records = self.repetitions(7)
        records = [r for r in records if r.trial_id != "control-7"]
        reports = {"control-1": evaluated(-2),
                   "control-2": evaluated(2, "rejected"),
                   "control-3": evaluated(-4, "rejected", "failed"),
                   "control-4": evaluated(-1, "incomplete", "missing"),
                   "control-5": evaluated(-1, "baseline_ineligible", baseline="failed"),
                   "control-6": evaluated(-1, "invalid")}
        result = self.aggregate(records, reports, data).report
        arm = result.arm_summaries[0]
        self.assertEqual((arm.planned_attempts, arm.eligible_attempts, arm.correct_attempts), (7, 4, 2))
        self.assertEqual(arm.state_counts, dict(accepted=1, rejected=2, incomplete=2,
                                               baseline_ineligible=1, invalid=1))
        self.assertEqual(result.outcomes[-2].baseline_status, "missing")
        self.assertEqual(c.ExperimentReport.from_dict(result.to_dict()), result)

    def test_structural_selection_and_hand_calculated_paired_variation(self):
        data, records = self.repetitions(3)
        reports = {f"control-{i}": evaluated(d) for i, d in enumerate((-2, -4, -6), 1)}
        reports.update({f"skill-{i}": evaluated(d, "rejected" if i == 2 else "accepted")
                        for i, d in enumerate((-3, -7, -11), 1)})
        result = self.aggregate(records, reports, data).report
        all_values, = result.arm_summaries[1].all_attempt_structural
        accepted, = result.arm_summaries[1].accepted_only_structural
        self.assertEqual((all_values.n, all_values.mean, all_values.median,
                          all_values.minimum, all_values.maximum, all_values.sample_stddev),
                         (3, -7, -7, -11, -3, 4))
        self.assertEqual((accepted.n, accepted.mean), (2, -7))
        paired, = result.paired_summaries[0].differences
        self.assertEqual((paired.n, paired.mean, paired.median, paired.sample_stddev), (3, -3, -3, 2))

    def test_empty_singleton_and_extras_do_not_expand_denominator(self):
        result = self.aggregate([producer(), producer(trial_id="extra")], {"control": evaluated()}).report
        arm = result.arm_summaries[0]
        self.assertEqual((arm.planned_attempts, arm.correct_attempts), (1, 1))
        self.assertEqual([x.trial_id for x in result.extra_trials], ["extra"])
        stat, = arm.all_attempt_structural
        self.assertEqual(stat.n, 1)
        self.assertIsNone(stat.sample_stddev)
        self.assertEqual(result.arm_summaries[1].all_attempt_structural, ())

    def test_exhaustion_retained_even_when_evaluation_complete(self):
        records = [producer(completion_state="budget_exhausted", budget_exhausted=True),
                   producer("skill", completion_state="budget_exhausted", budget_exhausted=True, run_artifact=None)]
        result = self.aggregate(records, {"control": evaluated()}).report
        self.assertEqual([o.decision for o in result.outcomes], ["accepted", "incomplete"])
        self.assertEqual([s.budget_exhausted_attempts for s in result.arm_summaries], [1, 1])
        self.assertTrue(all(t.budget_exhausted for t in result.trials))

    def test_telemetry_and_evaluator_time_remain_distinct_with_provenance(self):
        records = [producer(), producer("skill")]
        result = self.aggregate(records, {"control": evaluated()})
        self.assertEqual(result.report.trials[0].telemetry, records[0].telemetry)
        self.assertIsNone(result.report.trials[0].telemetry.provider_cost.value)
        self.assertEqual(result.runs[0].report.elapsed_seconds, 0.1)
        self.assertEqual(result.runs[0].origin, "supplied_typed")
        self.assertEqual(result.report.trials[0].telemetry.agent_elapsed.value, 12)

    def test_filesystem_requires_completed_verified_recovery(self):
        from pathlib import Path
        import tempfile
        from lesscode_baseline.report import RunStore
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            store = RunStore.create(root / "runs" / "control", "a" * 64, run_id="run-1")
            store.finalize(evaluated())
            result = self.aggregate([producer()], artifact_root=root)
            self.assertEqual(result.report.outcomes[0].decision, "accepted")
            self.assertEqual(result.runs[0].origin, "verified_artifacts")
            original = (store.directory / "report.json").read_bytes()
            (store.directory / "report.json").write_bytes(original + b" ")
            result = self.aggregate([producer()], artifact_root=root)
            self.assertEqual(result.report.outcomes[0].decision, "incomplete")
            self.assertEqual(result.runs, ())
            (store.directory / "report.json").write_bytes(original)
            (store.directory / "artifacts.json").unlink()
            result = self.aggregate([producer()], artifact_root=root)
            self.assertEqual(result.report.outcomes[0].decision, "incomplete")
            self.assertEqual(result.runs, ())

    def test_confounded_pairs_retain_outcomes_but_no_paired_sample(self):
        result = self.aggregate([producer(), producer("skill", seed=99)],
                                {"control": evaluated(), "skill": evaluated()}).report
        self.assertTrue(all(o.decision == "accepted" for o in result.outcomes))
        self.assertTrue(result.outcomes[1].confounded_differences or
                        any("seed" in s for s in result.limitations))
        self.assertEqual(result.paired_summaries[0].differences, ())

    def test_incompatible_definitions_not_pooled(self):
        from dataclasses import replace
        data, records = self.repetitions(2)
        first, second = evaluated(), evaluated()
        second = replace(second, baseline=replace(second.baseline, metrics=(replace(second.baseline.metrics[0], producer_version="2"),)),
                         candidate=replace(second.candidate, metrics=(replace(second.candidate.metrics[0], producer_version="2"),)))
        result = self.aggregate(records, {"control-1": first, "control-2": second}, data).report
        self.assertEqual(result.arm_summaries[0].all_attempt_structural, ())
        self.assertIn("incompatible", " ".join(result.limitations))

    def test_foreign_report_source_and_missing_required_candidate_are_not_successes(self):
        from dataclasses import replace
        foreign = replace(evaluated(), baseline=replace(evaluated().baseline,
                          snapshot=replace(evaluated().baseline.snapshot, content_hash="b" * 64)))
        missing = replace(evaluated(decision="incomplete"), candidate=replace(evaluated().candidate, checks=()))
        result = self.aggregate([producer(), producer("skill")], {"control": foreign, "skill": missing}).report
        self.assertEqual([o.decision for o in result.outcomes], ["invalid", "incomplete"])
        self.assertEqual(result.arm_summaries[0].all_attempt_structural, ())
        self.assertEqual(result.arm_summaries[1].correct_attempts, 0)

    def test_missing_optional_baseline_still_ineligible(self):
        from dataclasses import replace
        run = evaluated()
        optional = replace(run.baseline.checks[0], check_id="optional", required=False, status="skipped")
        run = replace(run, baseline=replace(run.baseline, checks=(*run.baseline.checks, optional)))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "baseline_ineligible")
        self.assertEqual(result.arm_summaries[0].eligible_attempts, 0)

    def test_all_attempt_structural_does_not_select_for_baseline_behavior_eligibility(self):
        from dataclasses import replace
        ineligible = evaluated(-4, "baseline_ineligible", baseline="failed")
        missing = evaluated(-8, "incomplete")
        missing = replace(missing, baseline=replace(missing.baseline, checks=()))
        result = self.aggregate([producer(), producer("skill")],
                                {"control": ineligible, "skill": missing}).report
        self.assertEqual([outcome.baseline_status for outcome in result.outcomes], ["ineligible", "missing"])
        for arm, delta in zip(result.arm_summaries, (-4, -8)):
            self.assertEqual((arm.eligible_attempts, arm.correct_attempts), (0, 0))
            self.assertEqual(arm.accepted_only_structural, ())
            sample, = arm.all_attempt_structural
            self.assertEqual((sample.n, sample.mean), (1, delta))

    def test_absent_candidate_metric_can_be_correct_but_incomplete(self):
        from dataclasses import replace
        run = evaluated(decision="incomplete")
        run = replace(run, candidate=replace(run.candidate, metrics=()))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual((result.arm_summaries[0].eligible_attempts, result.arm_summaries[0].correct_attempts), (1, 1))
        self.assertEqual(result.outcomes[0].decision, "incomplete")
        self.assertEqual(result.arm_summaries[0].all_attempt_structural, ())
        self.assertIn("candidate metric missing", " ".join(result.limitations))

    def test_structured_metric_never_invents_a_scalar_projection(self):
        from dataclasses import replace
        run = evaluated()
        payload = c.PopulationPayload.from_dict({"kind": "population", "population_ids": ["one"],
                   "population_hash": "a" * 64, "numerator": 1, "denominator": 1, "explanation": None})
        metric = replace(run.baseline.metrics[0], value=payload)
        run = replace(run, baseline=replace(run.baseline, metrics=(metric,)),
                      candidate=replace(run.candidate, metrics=(metric,)))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual(result.arm_summaries[0].all_attempt_structural, ())
        self.assertIn("no declared scalar projection", " ".join(result.limitations))

    def test_telemetry_currencies_and_runtime_identities_are_retained_without_pooling(self):
        from tests.test_experiment_contracts import observed, unavailable
        left, right = producer().to_dict(), producer("skill").to_dict()
        left["telemetry"]["provider_cost"] = observed("USD", 2)
        right["telemetry"]["provider_cost"] = observed("EUR", 3)
        right["telemetry"]["token_usage"] = unavailable("tokens")
        right["telemetry"]["runtime_benchmarks"][0]["workload_hash"] = "b" * 64
        records = [c.TrialRecord.from_dict(value) for value in (left, right)]
        result = self.aggregate(records).report
        self.assertEqual([t.telemetry.provider_cost.unit for t in result.trials], ["USD", "EUR"])
        self.assertIsNone(result.trials[1].telemetry.token_usage.value)
        self.assertEqual(result.trials[1].telemetry.runtime_benchmarks[0].workload_hash, "b" * 64)
        self.assertTrue(all(not arm.all_attempt_structural for arm in result.arm_summaries))

    def test_candidate_metadata_cannot_weaken_frozen_required_checks(self):
        from dataclasses import replace
        run = evaluated()
        run = replace(run, candidate=replace(run.candidate,
                      checks=(replace(run.candidate.checks[0], required=False),)))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "invalid")
        self.assertEqual(result.arm_summaries[0].correct_attempts, 0)

    def test_budget_exhaustion_with_missing_required_check_overrides_rejection(self):
        run = evaluated(decision="rejected", candidate="missing")
        result = self.aggregate([producer(budget_exhausted=True, completion_state="budget_exhausted")],
                                {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "incomplete")
        self.assertEqual(result.arm_summaries[0].budget_exhausted_attempts, 1)

    def test_exhausted_failed_and_unfinished_required_checks_remain_incomplete(self):
        from dataclasses import replace
        for missing_status in (None, "missing", "error", "skipped"):
            with self.subTest(status=missing_status):
                run = evaluated(decision="rejected", candidate="failed")
                second = replace(run.baseline.checks[0], check_id="test.second")
                candidate = run.candidate.checks
                if missing_status is not None:
                    candidate = (*candidate, replace(second, status=missing_status))
                run = replace(run, baseline=replace(run.baseline, checks=(*run.baseline.checks, second)),
                              candidate=replace(run.candidate, checks=candidate), reasons=("test.api failed",))
                result = self.aggregate([producer(budget_exhausted=True, completion_state="budget_exhausted")],
                                        {"control": run}).report
                outcome, _ = result.outcomes
                self.assertEqual(outcome.decision, "incomplete")
                self.assertEqual(outcome.required_behavior_status, "failed")
                self.assertIn("test.api failed", outcome.reasons)
                self.assertEqual(result.arm_summaries[0].correct_attempts, 0)

    def execution(self, status="timeout", exit_code=None):
        from tests.test_contracts import command
        value = command()
        value.update(argv=["C:/Python/python.exe", "-m", "unittest"], executable_version="3.12.10")
        return c.ExecutionResult.from_dict({"command": value, "exit_code": exit_code, "status": status,
                "elapsed_seconds": 2, "stdout_artifact": "candidate/out.log",
                "stderr_artifact": "candidate/err.log", "started_at": "2026-10-30T18:00:00Z",
                "environment_hash": "a" * 64})

    def test_unfinished_or_unsuccessful_execution_cannot_be_overridden_by_passed_records(self):
        from dataclasses import replace
        for status, exit_code in (("timeout", None), ("log_limit", None), ("error", None), ("completed", 1)):
            with self.subTest(status=status):
                run = evaluated(decision="incomplete")
                run = replace(run, candidate=replace(run.candidate, executions=(self.execution(status, exit_code),)))
                result = self.aggregate([producer()], {"control": run}).report
                self.assertEqual(result.outcomes[0].required_behavior_status, "incomplete")
                self.assertEqual((result.arm_summaries[0].eligible_attempts,
                                  result.arm_summaries[0].correct_attempts), (1, 0))

    def test_unsuccessful_baseline_execution_prevents_eligibility(self):
        from dataclasses import replace
        run = evaluated()
        run = replace(run, baseline=replace(run.baseline, executions=(self.execution(),)))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "baseline_ineligible")
        self.assertEqual(result.arm_summaries[0].eligible_attempts, 0)

    def test_exhausted_failed_check_with_unfinished_execution_stays_incomplete(self):
        from dataclasses import replace
        run = evaluated(decision="rejected", candidate="failed")
        run = replace(run, candidate=replace(run.candidate, executions=(self.execution(),)),
                      reasons=("test.api failed",))
        result = self.aggregate([producer(budget_exhausted=True, completion_state="budget_exhausted")],
                                {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "incomplete")
        self.assertEqual(result.outcomes[0].required_behavior_status, "failed")

    def test_exhausted_fully_evaluated_rejection_remains_rejected(self):
        from dataclasses import replace
        for executions in ((), (self.execution("completed", 1),)):
            with self.subTest(executions=bool(executions)):
                run = evaluated(decision="rejected", candidate="failed")
                run = replace(run, candidate=replace(run.candidate, executions=executions))
                result = self.aggregate([producer(budget_exhausted=True, completion_state="budget_exhausted")],
                                        {"control": run}).report
                self.assertEqual(result.outcomes[0].decision, "rejected")
                self.assertEqual(result.outcomes[0].required_behavior_status, "failed")

    def test_successfully_completed_execution_preserves_verified_correctness(self):
        from dataclasses import replace
        run = evaluated()
        run = replace(run, baseline=replace(run.baseline, executions=(self.execution("completed", 0),)),
                      candidate=replace(run.candidate, executions=(self.execution("completed", 0),)))
        result = self.aggregate([producer()], {"control": run}).report
        self.assertEqual(result.outcomes[0].decision, "accepted")
        self.assertEqual(result.arm_summaries[0].correct_attempts, 1)

    def test_report_data_is_detached_and_unknown_reports_rejected(self):
        run = evaluated()
        record = producer()
        result = self.aggregate([record], {"control": run})
        record.conditions.settings["temperature"] = 99
        self.assertEqual(result.report.trials[0].conditions.settings["temperature"], 0)
        with self.assertRaisesRegex(c.ContractError, "planned trial"):
            self.aggregate([], {"foreign": run})
        with self.assertRaisesRegex(c.ContractError, "typed RunReport"):
            self.aggregate([producer()], {"control": run.to_dict()})


def producer(arm="control", **changes):
    value = trial()
    value.update(trial_id=arm, arm_id=arm, execution_order=1 if arm == "control" else 2)
    value["conditions"]["interventions"]["skill"] = "off" if arm == "control" else "on"
    value.update(changes)
    return c.TrialRecord.from_dict(value)


class ReconciliationTests(unittest.TestCase):
    def reconcile(self, *records, data=None):
        return reconcile(c.ExperimentManifest.from_dict(data or manifest()), tuple(records))

    def test_schedule_order_and_pairing_ignore_arrival_and_execution_order(self):
        control, skill = producer(), producer("skill", execution_order=99, seed=42)
        control = producer(seed=42)
        result = self.reconcile(skill, control)
        self.assertEqual([x.planned.trial_id for x in result.planned_trials], ["control", "skill"])
        self.assertEqual([x.record for x in result.planned_trials], [control, skill])
        pair, = result.pairings
        self.assertEqual(pair.key, ("example", "simplify", 1))
        self.assertEqual((pair.left_trial_id, pair.right_trial_id), ("control", "skill"))
        self.assertTrue(pair.comparable)
        self.assertEqual(pair.confounded_differences, ())
        self.assertIn("producer", " ".join(result.limitations))

    def test_missing_attempts_explicit_and_extra_excluded(self):
        extra = producer(trial_id="unscheduled", execution_order=3)
        result = self.reconcile(producer(), extra)
        self.assertEqual(len(result.planned_trials), 2)
        self.assertEqual(result.extra_trials, (extra,))
        missing = result.planned_trials[1]
        self.assertIsNone(missing.record)
        self.assertEqual(missing.missing_outcome.decision, "incomplete")
        self.assertEqual(missing.missing_outcome.baseline_status, "missing")
        self.assertIn("missing", " ".join(missing.missing_outcome.reasons))
        self.assertFalse(result.pairings[0].comparable)
        self.assertEqual(result.pairings[0].confounded_differences, ())

    def test_duplicate_ids_fail_including_extras(self):
        for identity in ("control", "extra"):
            with self.subTest(identity=identity), self.assertRaisesRegex(c.ContractError, "duplicate.*trial_id"):
                record = producer(trial_id=identity)
                self.reconcile(record, record)

    def test_every_unintended_control_difference_is_specific(self):
        value = trial()
        value.update(trial_id="skill", arm_id="skill", starting_source_hash="b" * 64, seed=9)
        value["conditions"].update(model="other", settings={"temperature": 1, "extra": True},
                                   prompt_identity="other", permissions=["read_source", "network"])
        value["conditions"]["budget"]["tokens"] = 1
        value["conditions"]["interventions"] = {"skill": "unexpected", "undeclared": True}
        result = self.reconcile(producer(seed=1), c.TrialRecord.from_dict(value))
        pair = result.pairings[0]
        self.assertFalse(pair.comparable)
        differences = "\n".join(pair.confounded_differences)
        for field in ("starting_source_hash", "model", "settings.temperature", "settings.extra",
                      "prompt_identity", "permissions", "budget.tokens", "interventions.skill",
                      "interventions.undeclared", "seed"):
            with self.subTest(field=field):
                self.assertIn(field, differences)

    def test_both_arms_drifting_together_still_confounded(self):
        def changed(arm):
            value = producer(arm).to_dict()
            value["conditions"]["model"] = "same-unplanned-model"
            return c.TrialRecord.from_dict(value)
        result = self.reconcile(changed("control"), changed("skill"))
        self.assertFalse(result.pairings[0].comparable)
        self.assertIn("conditions.model", " ".join(result.pairings[0].confounded_differences))

    def test_planned_identity_mismatch_is_not_reassigned_by_claimed_pair_key(self):
        result = self.reconcile(producer(task_id="different", corpus_id="foreign", repetition=2, arm_id="foreign"), producer("skill"))
        differences = " ".join(result.planned_trials[0].confounded_differences)
        for field in ("task_id", "corpus_id", "repetition", "arm_id"):
            self.assertIn(field, differences)
        self.assertEqual(result.pairings[0].key, ("example", "simplify", 1))
        self.assertFalse(result.pairings[0].comparable)

    def test_tasks_and_repetitions_pair_separately_with_multiple_arms(self):
        data = manifest()
        data["arms"].append({"arm_id": "tools", "interventions": {"skill": "tools"}})
        data["planned_trials"] = [dict(p, trial_id=f"{arm}-{task}-{rep}", arm_id=arm,
                                        task_id=task, repetition=rep)
                                  for p in [data["planned_trials"][0]]
                                  for task in ("simplify", "refactor") for rep in (1, 2)
                                  for arm in ("control", "skill", "tools")]
        records = []
        for index, planned in enumerate(data["planned_trials"]):
            value = trial() | {key: planned[key] for key in ("trial_id", "arm_id", "task_id", "repetition")}
            value["execution_order"] = index + 1
            value["conditions"]["interventions"]["skill"] = {"control": "off", "skill": "on", "tools": "tools"}[planned["arm_id"]]
            records.append(c.TrialRecord.from_dict(value))
        result = self.reconcile(*reversed(records), data=data)
        self.assertEqual(len(result.pairings), 12)
        self.assertTrue(all(pair.comparable for pair in result.pairings))
        self.assertEqual(len({pair.key for pair in result.pairings}), 4)

    def test_preserves_failure_exhaustion_telemetry_and_corpus_provenance(self):
        data = manifest()
        data["corpus"][0]["generation_provenance"] = {"status": "known", "description": "producer declares human authorship", "source": "maintainer"}
        failed = producer(completion_state="failed", run_artifact=None, budget_exhausted=True)
        exhausted = producer("skill", completion_state="budget_exhausted", run_artifact=None, budget_exhausted=True)
        result = self.reconcile(exhausted, failed, data=data)
        self.assertEqual(result.manifest.to_dict(), data)
        self.assertEqual(result.planned_trials[0].record, failed)
        self.assertEqual(result.planned_trials[1].record, exhausted)
        self.assertIsNone(result.planned_trials[0].missing_outcome)
        self.assertEqual(result.planned_trials[0].record.telemetry.provider_cost.status, "unavailable")

    def test_revision_only_corpus_does_not_claim_source_hash_verification(self):
        data = manifest()
        data["corpus"][0]["content_hash"] = None
        result = self.reconcile(producer(), producer("skill"), data=data)
        self.assertIn("content_hash", " ".join(result.limitations))
        self.assertEqual(result.manifest.corpus[0].generation_provenance.status, "unknown")

    def test_settings_types_and_permissions_sets_are_compared_honestly(self):
        data = manifest()
        data["controls"]["permissions"] = ["read_source", "run_tools"]
        records = []
        for arm in ("control", "skill"):
            value = producer(arm).to_dict()
            value["conditions"]["permissions"] = ["run_tools", "read_source"]
            records.append(c.TrialRecord.from_dict(value))
        self.assertTrue(self.reconcile(*records, data=data).pairings[0].comparable)
        value = records[1].to_dict()
        value["conditions"]["settings"]["temperature"] = False
        result = self.reconcile(records[0], c.TrialRecord.from_dict(value), data=data)
        self.assertFalse(result.pairings[0].comparable)

    def test_contract_validation_applies_to_directly_constructed_records(self):
        from dataclasses import replace
        with self.assertRaises(c.ContractError):
            self.reconcile(replace(producer(), repetition=0))

    def test_no_producer_records_retains_entire_schedule(self):
        result = self.reconcile()
        self.assertEqual(len(result.planned_trials), 2)
        self.assertTrue(all(item.missing_outcome.decision == "incomplete"
                            for item in result.planned_trials))
        self.assertFalse(result.pairings[0].comparable)
        self.assertEqual(result.extra_trials, ())

    def test_intervention_named_model_cannot_waive_actual_model_control(self):
        data = manifest()
        data["allowed_interventions"].append("model")
        data["controls"]["interventions"]["model"] = "recorded-model"
        for arm in data["arms"]:
            arm["interventions"]["model"] = "intended"
        records = []
        for arm in ("control", "skill"):
            value = producer(arm).to_dict()
            value["conditions"]["interventions"]["model"] = "intended"
            value["conditions"]["model"] = "unintended"
            records.append(c.TrialRecord.from_dict(value))
        result = self.reconcile(*records, data=data)
        self.assertFalse(result.pairings[0].comparable)
        self.assertIn("conditions.model", " ".join(result.pairings[0].confounded_differences))

    def test_task_specific_scheduled_prompt_overrides_default_controls(self):
        data = manifest()
        for planned in data["planned_trials"]:
            planned["prompt_identity"] = "task-specific-prompt-hash"
        records = []
        for arm in ("control", "skill"):
            value = producer(arm).to_dict()
            value["conditions"]["prompt_identity"] = "task-specific-prompt-hash"
            records.append(c.TrialRecord.from_dict(value))
        self.assertTrue(self.reconcile(*records, data=data).pairings[0].comparable)

    def test_reconciliation_detaches_mutable_supplied_metadata(self):
        declared = c.ExperimentManifest.from_dict(manifest())
        record = producer()
        result = reconcile(declared, (record,))
        record.conditions.settings["temperature"] = 99
        declared.controls.settings["temperature"] = 98
        self.assertEqual(result.planned_trials[0].record.conditions.settings["temperature"], 0)
        self.assertEqual(result.manifest.controls.settings["temperature"], 0)

    def test_revision_only_corpus_still_detects_between_arm_source_drift(self):
        data = manifest()
        data["corpus"][0]["content_hash"] = None
        result = self.reconcile(producer(), producer("skill", starting_source_hash="b" * 64), data=data)
        self.assertFalse(result.pairings[0].comparable)
        self.assertIn("pair.starting_source_hash", " ".join(result.pairings[0].confounded_differences))

    def test_scalar_setting_named_permissions_is_not_a_permission_list(self):
        data = manifest()
        data["controls"]["settings"]["permissions"] = 3
        records = []
        for arm in ("control", "skill"):
            value = producer(arm).to_dict()
            value["conditions"]["settings"]["permissions"] = 3
            records.append(c.TrialRecord.from_dict(value))
        self.assertTrue(self.reconcile(*records, data=data).pairings[0].comparable)


if __name__ == "__main__":
    unittest.main()
