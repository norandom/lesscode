"""Schedule identities and controlled conditions, independently of run success."""

import unittest

from lesscode_baseline import contracts as c
from lesscode_baseline.experiments import reconcile
from tests.test_experiment_contracts import manifest, trial


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
