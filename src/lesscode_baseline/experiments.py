"""Reconcile supplied experiment records against a predeclared schedule.

Matching metadata establishes only declared comparability. This module neither
opens run references nor certifies producer claims or causal effects.
"""

from dataclasses import dataclass
from itertools import combinations

from .contracts import (ContractError, ExperimentManifest, PlannedTrial,
                        TrialOutcome, TrialRecord)


@dataclass(frozen=True)
class ReconciledTrial:
    planned: PlannedTrial
    record: TrialRecord | None
    missing_outcome: TrialOutcome | None
    confounded_differences: tuple[str, ...]


@dataclass(frozen=True)
class Pairing:
    key: tuple[str, str, int]
    left_trial_id: str
    right_trial_id: str
    confounded_differences: tuple[str, ...]
    limitations: tuple[str, ...]

    @property
    def comparable(self) -> bool:
        """Declared controls match; does not imply verified evaluation evidence."""
        return not self.confounded_differences and not self.limitations


@dataclass(frozen=True)
class Reconciliation:
    manifest: ExperimentManifest
    planned_trials: tuple[ReconciledTrial, ...]
    extra_trials: tuple[TrialRecord, ...]
    pairings: tuple[Pairing, ...]
    limitations: tuple[str, ...]


def _differences(expected, observed, path):
    """Leaf diagnostics distinguish absent keys and JSON booleans from numbers."""
    if isinstance(expected, dict) and isinstance(observed, dict):
        result = []
        for key in sorted(expected.keys() | observed.keys()):
            location = f"{path}.{key}"
            if key not in expected:
                result.append(f"{location}: undeclared value {observed[key]!r}")
            elif key not in observed:
                result.append(f"{location}: missing expected {expected[key]!r}")
            else:
                result.extend(_differences(expected[key], observed[key], location))
        return result
    if path in ("conditions.permissions", "pair.conditions.permissions"):
        equal = set(expected) == set(observed)
    else:
        equal = expected == observed and (isinstance(expected, bool) == isinstance(observed, bool))
    return [] if equal else [f"{path}: expected {expected!r}, observed {observed!r}"]


def _fixed_conditions(record):
    value = record.conditions.to_dict()
    del value["interventions"]
    return value


def reconcile(manifest: ExperimentManifest,
              trials: tuple[TrialRecord, ...]) -> Reconciliation:
    """Retain every scheduled attempt and keep unscheduled records separate.

    The manifest's IDs and pairing keys are authoritative. A record claiming a
    different arm/task/corpus/repetition stays at its scheduled ID and is flagged.
    Arm interventions may differ only at the manifest's declared intervention
    keys, and must match the arm's declared values. Fixed controls are never
    waived by an intervention key that happens to have the same name.
    """
    # Revalidate even records built with direct dataclass constructors. Re-reading
    # also detaches mutable producer dictionaries from the returned evidence.
    manifest = ExperimentManifest.from_dict(manifest.to_dict(), "$manifest")
    records = {}
    for index, supplied in enumerate(trials):
        record = TrialRecord.from_dict(supplied.to_dict(), f"$trials[{index}]")
        if record.trial_id in records:
            raise ContractError(f"$trials[{index}].trial_id: duplicate trial_id {record.trial_id!r}")
        records[record.trial_id] = record

    arms = {arm.arm_id: arm for arm in manifest.arms}
    corpus = {entry.corpus_id: entry for entry in manifest.corpus}
    planned_ids = {planned.trial_id for planned in manifest.planned_trials}
    reconciled = []
    limitations = ["Conditions, source identities and generation provenance are producer claims; "
                   "metadata agreement does not independently verify them or establish causal effects."]
    for entry in manifest.corpus:
        if entry.content_hash is None:
            limitations.append(f"corpus {entry.corpus_id}: content_hash unavailable; "
                               "recorded revision cannot verify a supplied starting source hash.")

    for planned in manifest.planned_trials:
        record = records.get(planned.trial_id)
        if record is None:
            missing = TrialOutcome(planned.trial_id, "incomplete", "missing", "incomplete", (),
                                   ("planned producer record missing; baseline and candidate evidence unavailable",))
            reconciled.append(ReconciledTrial(planned, None, missing, ()))
            continue
        differences = []
        for field in ("arm_id", "corpus_id", "task_id", "repetition"):
            differences.extend(_differences(getattr(planned, field), getattr(record, field), field))
        expected_hash = corpus[planned.corpus_id].content_hash
        if expected_hash is not None:
            differences.extend(_differences(expected_hash, record.starting_source_hash,
                                            "starting_source_hash"))
        expected = manifest.controls.to_dict()
        # Prompts can be task-specific; the scheduled trial freezes that identity.
        expected["prompt_identity"] = planned.prompt_identity
        expected["interventions"] = arms[planned.arm_id].interventions
        differences.extend(_differences(expected, record.conditions.to_dict(), "conditions"))
        reconciled.append(ReconciledTrial(planned, record, None, tuple(differences)))

    groups = {}
    for item in reconciled:
        p = item.planned
        groups.setdefault((p.corpus_id, p.task_id, p.repetition), {})[p.arm_id] = item
    pairings = []
    for key, group in groups.items():
        # Stable arm order comes from the manifest, never from producer arrival.
        ordered = [group[arm.arm_id] for arm in manifest.arms if arm.arm_id in group]
        for left, right in combinations(ordered, 2):
            differences = [f"{item.planned.trial_id}: {difference}"
                           for item in (left, right) for difference in item.confounded_differences]
            missing = tuple(f"{item.planned.trial_id}: producer record missing"
                            for item in (left, right) if item.record is None)
            if left.record is not None and right.record is not None:
                differences.extend(_differences(left.record.starting_source_hash,
                                                right.record.starting_source_hash,
                                                "pair.starting_source_hash"))
                differences.extend(_differences(_fixed_conditions(left.record),
                                                _fixed_conditions(right.record), "pair.conditions"))
                differences.extend(_differences(left.record.seed, right.record.seed, "pair.seed"))
            pairings.append(Pairing(key, left.planned.trial_id, right.planned.trial_id,
                                    tuple(differences), missing))
    extras = tuple(records[identity] for identity in sorted(records.keys() - planned_ids))
    return Reconciliation(manifest, tuple(reconciled), extras, tuple(pairings), tuple(limitations))
