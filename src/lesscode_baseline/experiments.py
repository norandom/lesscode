"""Reconcile supplied experiment records against a predeclared schedule.

Matching metadata establishes only declared comparability, never causal effects
or verification of generation provenance. Aggregation opens references only via
completed, hash-verified RunStore recovery. Explicit typed reports are supported
for callers and tests, and labeled supplied rather than filesystem-verified.

Structural distributions contain absolute per-trial metric deltas (candidate
minus baseline). Paired distributions subtract the left-arm delta from the
right-arm delta. Only explicit comparable scalar measurements are used. Matching
keys pool only with identical unit, definition, producer and producer version;
incompatible groups are omitted with a limitation. Scope compatibility within a
run is checked by the metric comparator; between runs it is governed by declared
controls. Run reports retain measurement identities and evaluator elapsed time;
producer agent time and observed telemetry remain separate in trial records.
Statistics are descriptive; sample standard deviation is undefined for n < 2.
RunReport does not map command IDs to check IDs. Unfinished executions therefore
conservatively prevent verified correctness/completeness, and unsuccessful
baseline executions prevent eligibility. A completed nonzero candidate command
can accompany complete observed rejection, but never override into correctness.
"""

from dataclasses import dataclass
from itertools import combinations
from pathlib import Path
import math
import statistics
from collections.abc import Mapping

from .contracts import (ArmSummary, ContractError, DECISIONS, Distribution,
                        ExperimentManifest, ExperimentReport, PairedSummary,
                        PlannedTrial, RunReport, TrialOutcome, TrialRecord)
from .decision import compare_metrics
from .report import recover


@dataclass(frozen=True)
class EvaluatedTrial:
    trial_id: str
    report: RunReport
    origin: str


@dataclass(frozen=True)
class Aggregation:
    report: ExperimentReport
    runs: tuple[EvaluatedTrial, ...]


def _distribution(key, unit, values):
    n = len(values)
    return Distribution(key, unit, n, statistics.mean(values) if n else None,
                        statistics.median(values) if n else None,
                        min(values) if n else None, max(values) if n else None,
                        statistics.stdev(values) if n > 1 else None)


def _summaries(samples, limitations, label):
    """Each sample is (key, definition identity, number); no unit conversion."""
    groups = {}
    for key, identity, number in samples:
        groups.setdefault(key, []).append((identity, number))
    result = []
    for key in sorted(groups):
        group = groups[key]
        identities = {identity for identity, _ in group}
        if len(identities) != 1:
            limitations.append(f"{label}: {key} omitted: incompatible metric definitions/producers/units")
            continue
        identity, = identities
        values = [number for _, number in group]
        try:
            distribution = _distribution(key, identity[0], values)
            # Reject arithmetic overflow instead of emitting an invalid public report.
            Distribution.from_dict(distribution.to_dict())
        except (OverflowError, ValueError):
            limitations.append(f"{label}: {key} omitted: descriptive arithmetic is not finite")
            continue
        result.append(distribution)
    return tuple(result)


def _load_run(record, reports, artifact_root):
    if record.trial_id in reports:
        supplied = reports[record.trial_id]
        if not isinstance(supplied, RunReport):
            raise ContractError(f"$reports.{record.trial_id}: expected typed RunReport")
        return RunReport.from_dict(supplied.to_dict()), "supplied_typed", ()
    if record.run_artifact is None or artifact_root is None:
        return None, None, ("run evidence unavailable; completed verified artifact or typed report required",)
    root = Path(artifact_root).absolute()
    reference = root / record.run_artifact
    # Trial references may name the run directory or its canonical report.json.
    directory = reference.parent if reference.name == "report.json" else reference
    if not directory.resolve().is_relative_to(root.resolve()):
        return None, None, ("run artifact escapes the declared artifact root",)
    recovered = recover(directory)
    if recovered.state != "complete":
        return None, None, ("run artifact incomplete: " + "; ".join(recovered.diagnostics),)
    return recovered.report, "verified_artifacts", ()


def _outcome(item, run, diagnostics):
    if item.record is None:
        return item.missing_outcome
    record = item.record
    reasons = [*diagnostics]
    if record.budget_exhausted:
        reasons.append("producer reports exhausted budget; exhaustion retained regardless of evaluation outcome")
    if run is None:
        return TrialOutcome(record.trial_id, "incomplete", "missing", "incomplete",
                            item.confounded_differences, tuple(reasons))
    reasons.extend(run.reasons)
    invalid = run.decision == "invalid" or run.integrity.status != "unchanged"
    if run.baseline.snapshot.content_hash != record.starting_source_hash:
        invalid = True
        reasons.append("evaluation baseline source differs from producer starting source")
    for observation in (run.baseline, run.candidate):
        if any(check.source_hash != observation.snapshot.content_hash or
               check.plan_hash != run.plan_hash for check in observation.checks):
            invalid = True
            reasons.append("check source/plan binding differs from evaluation report")
    unsuccessful = {}
    for side, observation in (("baseline", run.baseline), ("candidate", run.candidate)):
        unsuccessful[side] = tuple(execution for execution in observation.executions
                                   if execution.status != "completed" or execution.exit_code != 0)
        for execution in unsuccessful[side]:
            reasons.append(f"{side} execution {execution.command.command_id}: {execution.status}; "
                           f"exit {execution.exit_code}; command-to-required-check mapping unavailable in RunReport; "
                           "execution cannot be overridden by passing check records")
    baseline = ("invalid" if invalid else "ineligible" if run.decision == "baseline_ineligible" or
                unsuccessful["baseline"] or any(check.status != "passed" for check in run.baseline.checks) else
                "passed" if run.baseline.checks else "missing")
    expected = {check.check_id: check for check in run.baseline.checks if check.required}
    actual = {check.check_id: check for check in run.candidate.checks}
    baseline_checks = {check.check_id: check for check in run.baseline.checks}
    if any(identity in baseline_checks and
           (check.kind, check.required) != (baseline_checks[identity].kind, baseline_checks[identity].required)
           for identity, check in actual.items()):
        invalid = True
        baseline = "invalid"
        reasons.append("candidate check kind/required metadata differs from baseline")
    required = {**expected, **{check.check_id: check for check in run.candidate.checks if check.required}}
    statuses = [actual[identity].status if identity in actual else "missing" for identity in required]
    for identity in required:
        status = actual[identity].status if identity in actual else "missing"
        if status != "passed":
            reasons.append(f"candidate required check {identity}: {status}")
    # Failure and completeness are independent: one observed failure cannot hide
    # an absent second check or a terminated execution after exhausting a budget.
    required_complete = bool(statuses) and all(status in ("passed", "failed") for status in statuses)
    if any(execution.status != "completed" for execution in unsuccessful["candidate"]):
        required_complete = False
    if unsuccessful["candidate"] and "failed" not in statuses:
        # No definitive failed required record explains a completed nonzero exit.
        required_complete = False
    behavior = ("failed" if "failed" in statuses else
                "passed" if required_complete and all(status == "passed" for status in statuses) else "incomplete")
    decision = run.decision
    if invalid:
        decision = "invalid"
        behavior = "incomplete"
    elif baseline == "ineligible":
        decision = "baseline_ineligible"
    elif baseline == "missing":
        decision = "incomplete"
    elif record.budget_exhausted and (decision not in ("accepted", "rejected") or not required_complete):
        decision = "incomplete"
    elif behavior == "failed":
        decision = "rejected"
    elif behavior == "incomplete" and decision == "accepted":
        decision = "incomplete"
    return TrialOutcome(record.trial_id, decision, baseline, behavior,
                        item.confounded_differences, tuple(reasons))


def aggregate(manifest: ExperimentManifest, trials: tuple[TrialRecord, ...], *,
              reports: Mapping[str, RunReport] | None = None,
              artifact_root: Path | None = None) -> Aggregation:
    """Summarize the schedule, preserving run evidence and producer telemetry.

    ``reports`` explicitly supplies typed evidence by trial ID; unspecified trials
    use recovery below ``artifact_root``. A bare or interrupted report.json never
    establishes a completed filesystem run. Accepted outcomes are distinct from
    required-check correctness. Invalid/ineligible/missing baselines are excluded
    from its denominator, while eligible rejected and incomplete attempts remain.
    """
    reconciliation = reconcile(manifest, trials)
    reports = {} if reports is None else dict(reports)
    known = {item.planned.trial_id for item in reconciliation.planned_trials if item.record is not None}
    if set(reports) - known:
        raise ContractError("$reports: report references require a supplied planned trial")
    limitations = [*reconciliation.limitations,
                   "All-attempt structural samples include available comparable scalar deltas regardless of baseline behavior eligibility; invalid integrity/source-bound evidence is excluded, while every planned outcome remains counted. Missing measurements do not become zero; each distribution reports its observed sample count.",
                   "Descriptive statistics only; sample standard deviation is undefined for n < 2; no inferential confidence interval."]
    outcomes, runs, samples = [], [], {}
    for item in reconciliation.planned_trials:
        run, origin, diagnostics = ((None, None, ()) if item.record is None else
                                    _load_run(item.record, reports, artifact_root))
        outcome = _outcome(item, run, diagnostics)
        outcomes.append(outcome)
        samples[item.planned.trial_id] = []
        if run is None:
            continue
        runs.append(EvaluatedTrial(item.planned.trial_id, run, origin))
        if origin == "supplied_typed":
            limitations.append("Explicit typed evaluation reports are supplied evidence; artifact completeness/hashes were not independently recovered for those reports.")
        limitations.extend(f"{item.planned.trial_id}: {reason}" for reason in run.limitations)
        if outcome.decision == "invalid":
            continue
        for metric in compare_metrics(run.baseline.metrics, run.candidate.metrics):
            delta = metric.delta
            if delta.status != "available":
                limitations.append(f"{item.planned.trial_id}: {delta.key}: {delta.reason}")
                continue
            evidence = metric.before
            identity = (evidence.unit, evidence.definition_id, evidence.producer, evidence.producer_version)
            samples[item.planned.trial_id].append((delta.key, identity, delta.absolute_delta))
    by_id = {outcome.trial_id: outcome for outcome in outcomes}
    arms = []
    for arm in reconciliation.manifest.arms:
        items = [item for item in reconciliation.planned_trials if item.planned.arm_id == arm.arm_id]
        selected = [by_id[item.planned.trial_id] for item in items]
        states = {state: sum(outcome.decision == state for outcome in selected) for state in DECISIONS}
        eligible = [outcome for outcome in selected if outcome.baseline_status == "passed" and outcome.decision != "invalid"]
        all_samples = [sample for item in items for sample in samples[item.planned.trial_id]]
        accepted = [sample for item in items if by_id[item.planned.trial_id].decision == "accepted"
                    for sample in samples[item.planned.trial_id]]
        arms.append(ArmSummary(arm.arm_id, len(items), states, len(eligible),
                               sum(outcome.required_behavior_status == "passed" for outcome in eligible),
                               sum(item.record is not None and item.record.budget_exhausted for item in items),
                               _summaries(all_samples, limitations, f"{arm.arm_id} all attempts"),
                               _summaries(accepted, limitations, f"{arm.arm_id} accepted only")))
    paired = {(left.arm_id, right.arm_id): [] for left, right in combinations(reconciliation.manifest.arms, 2)}
    planned_by_id = {item.planned.trial_id: item.planned for item in reconciliation.planned_trials}
    for pairing in reconciliation.pairings:
        if not pairing.comparable:
            limitations.extend(f"pair {pairing.left_trial_id}/{pairing.right_trial_id}: {reason}"
                               for reason in (*pairing.confounded_differences, *pairing.limitations))
            continue
        left = {key: (identity, value) for key, identity, value in samples[pairing.left_trial_id]}
        right = {key: (identity, value) for key, identity, value in samples[pairing.right_trial_id]}
        target = paired[(planned_by_id[pairing.left_trial_id].arm_id, planned_by_id[pairing.right_trial_id].arm_id)]
        for key in sorted(left.keys() & right.keys()):
            if left[key][0] != right[key][0]:
                limitations.append(f"pair {pairing.left_trial_id}/{pairing.right_trial_id}: {key} incompatible definitions/producers/units")
            else:
                difference = right[key][1] - left[key][1]
                if math.isfinite(difference):
                    target.append((key, left[key][0], difference))
                else:
                    limitations.append(f"pair {pairing.left_trial_id}/{pairing.right_trial_id}: {key} non-finite difference")
    report = ExperimentReport(1, reconciliation.manifest,
                              tuple(item.record for item in reconciliation.planned_trials if item.record is not None),
                              reconciliation.extra_trials, tuple(outcomes), tuple(arms),
                              tuple(PairedSummary(left, right, _summaries(values, limitations, f"pair {left}/{right}"))
                                    for (left, right), values in paired.items()),
                              tuple(dict.fromkeys(limitations)))
    return Aggregation(ExperimentReport.from_dict(report.to_dict()), tuple(runs))


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
