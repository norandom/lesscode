"""Run one frozen local pair, without installing tools or changing its sources.

``evaluate`` returns the durable RunReport. Invalid inputs raise ContractError
before storage is created. Storage failures raise PersistenceError (exit code 4,
with the surviving directory). Unexpected component failures/interruption leave
a running manifest, partial observations and logs for ``report.recover``; they
propagate rather than claiming a completed evaluation. Workspaces are temporary
unless ``retain_workspaces`` is explicitly requested.

Integrity guards surround each public measurement-adapter workload and each
declared behavior command. Adapter-internal workers retain their own bounded
execution lifecycle; local fingerprints provide change detection, not isolation.
"""

from contextlib import ExitStack
from dataclasses import dataclass, replace
from datetime import datetime, timezone
import hashlib
from pathlib import Path
import platform
import tempfile
import time
from typing import Literal

from . import config, decision, metrics, process, snapshot
from .contracts import (CheckEvidence, ContractError, EnvironmentInfo, Failure,
                        FileDelta, IntegrityResult, MetricEvidence, Observation,
                        PreparedPlan, RunReport, dumps, loads)
from .report import PersistenceError, RunStore


@dataclass(frozen=True)
class OptionalEvidenceInput:
    """Explicit local artifact import; generated evidence needs a freshness bound.

    This internal API does not extend the serialized Plan or Profile contracts.
    ``path`` names the envelope itself, with provenance supplied by its producer.
    """

    side: Literal["baseline", "candidate"]
    path: Path
    origin: Literal["generated", "producer_supplied"]
    generated_since: datetime | None = None


def _now():
    return datetime.now(timezone.utc).isoformat()


def _rebase(record, prefix):
    """Translate only typed artifact fields, retaining complete metric payloads."""
    fields = {}
    for name in ("raw_artifact", "diagnostic_artifact", "stdout_artifact", "stderr_artifact"):
        if hasattr(record, name):
            value = getattr(record, name)
            fields[name] = None if value is None else prefix + "/" + value
    return replace(record, **fields)


def _missing_checks(profile, source_hash, plan_hash):
    return tuple(CheckEvidence(identity, "missing", spec.kind, spec.required,
                               source_hash, plan_hash, None)
                 for spec in profile.checks for identity in spec.expected_ids)


def _test_delta(before, after):
    left = {f.path: f for f in before.files if f.category == "tests"}
    right = {f.path: f for f in after.files if f.category == "tests"}
    return FileDelta(tuple(sorted(right.keys() - left.keys())),
                     tuple(sorted(left.keys() - right.keys())),
                     tuple(sorted(p for p in left.keys() & right.keys()
                                  if left[p].content_hash != right[p].content_hash)))


def evaluate(prepared: PreparedPlan, output: Path, *, retain_workspaces: bool = False,
             optional_evidence: tuple[OptionalEvidenceInput, ...] = ()) -> RunReport:
    """Evaluate declared baseline then candidate, once each, under one plan hash.

    Every check gets a fresh copy of its frozen input, so a native check cannot
    change the source seen by an independent check through ordinary writes.
    Missing bindings never authorize execution. Checks retain their entire
    declared inventory even when capture, integrity or tool availability fails.
    """
    if not isinstance(prepared, PreparedPlan):
        raise ContractError("evaluate requires a PreparedPlan")
    for request in optional_evidence:
        if not isinstance(request, OptionalEvidenceInput):
            raise ContractError("optional_evidence requires OptionalEvidenceInput records")
        if request.side not in ("baseline", "candidate") or request.origin not in ("generated", "producer_supplied"):
            raise ContractError("invalid optional evidence side or origin")
        if request.origin == "generated" and (not isinstance(request.generated_since, datetime)
                or request.generated_since.tzinfo is None or request.generated_since.utcoffset() is None):
            raise ContractError("generated optional evidence requires an aware generated_since")
    prepared = loads(PreparedPlan, dumps(prepared))
    output = Path(output).absolute()
    roots = (Path(prepared.plan.baseline.root), Path(prepared.plan.candidate.root),
             Path(prepared.evaluator_root))
    resolved_output = output.resolve()
    if any(resolved_output.is_relative_to(root) or root.is_relative_to(resolved_output)
           for root in roots):
        raise ContractError("output/source/evaluator roots cannot overlap")
    plan_hash = hashlib.sha256(dumps(prepared).encode("utf-8")).hexdigest()
    started, clock = _now(), time.monotonic()
    store = RunStore.create(output, plan_hash, prepared_plan=prepared)
    store.start()
    profile = prepared.plan.profile
    observations = {
        side: Observation(source, (), _missing_checks(profile, source.content_hash, plan_hash), (), ())
        for side, source in (("baseline", prepared.plan.baseline), ("candidate", prepared.plan.candidate))}
    changed, frozen = [], {}
    integrity_unavailable = False
    registered = set()
    checkpoint_number = 0
    optional_notes = []

    def register_logs():
        for side in ("baseline", "candidate"):
            root = store.directory / side
            if root.exists():
                for path in sorted(root.rglob("*")):
                    if path.is_file():
                        relative = path.relative_to(store.directory).as_posix()
                        if relative not in registered:
                            store.register_artifact(relative)
                            registered.add(relative)

    def checkpoint():
        nonlocal checkpoint_number
        register_logs()
        for side, observation in observations.items():
            relative = f"partial/{checkpoint_number:04d}-{side}.json"
            store.write_artifact(relative, dumps(observation).encode("utf-8"))
        checkpoint_number += 1

    def verify():
        nonlocal integrity_unavailable
        result = config.verify_integrity(prepared)
        changed.extend(result.changed_resources)
        integrity_unavailable |= result.status == "unavailable"
        # Source manifests are compared to preparation as well as to frozen
        # measurement copies. Additions/removals/untracked paths all participate.
        for side in ("baseline", "candidate"):
            expected = getattr(prepared.plan, side)
            current = snapshot.capture_scope(Path(expected.root), profile.scope)
            if isinstance(current, Failure) or (current.content_hash, current.scope_hash) != (
                    expected.content_hash, expected.scope_hash):
                changed.append(side + "/source")
            if side in frozen:
                expected = frozen[side]
                current = snapshot.capture_scope(Path(expected.root), profile.scope)
                if isinstance(current, Failure) or (current.content_hash, current.scope_hash) != (
                        expected.content_hash, expected.scope_hash):
                    changed.append(side + "/measurement")
        return not changed and not integrity_unavailable

    def workload(call):
        if not verify():
            return None
        try:
            return call()
        finally:
            verify()
            register_logs()

    bindings = {tool.name: tool for tool in prepared.tools}

    def available(*names):
        return all(name in bindings and bindings[name].status == "available" for name in names)

    def unavailable(side, keys, reason):
        # No analyzer ran: identify this as availability evidence, never as a
        # vendor result or an invented structured/scalar projection.
        observation = observations[side]
        units = {"complexity": "CCN", "duplication": "ratio", "source_bytes": "bytes",
                 "physical_lines": "lines", "compressed_source_bytes": "bytes"}
        absent = tuple(MetricEvidence(key, "unavailable", None, units[key],
                                     "measurement-not-executed-v1", "lesscode-evaluation", "1",
                                     observation.snapshot.scope_hash, reason, None) for key in keys)
        failures = tuple(Failure("measurement_unavailable", reason, None, None, key, False, None)
                         for key in keys)
        observations[side] = replace(observation, metrics=(*observation.metrics, *absent),
                                     failures=(*observation.failures, *failures))

    def import_optional(side):
        identities = set()
        for number, request in enumerate(optional_evidence):
            if request.side != side:
                continue
            observation = observations[side]
            prefix = f"{side}/optional/{number}"
            path = Path(request.path).absolute()
            try:
                if not path.parent.is_dir():
                    envelope, raw, failures = None, None, (Failure("optional_evidence_missing",
                        "optional evidence directory missing", None, None, None, False, None),)
                else:
                    envelope, raw, failures = process.read_optional_evidence(path.name, path.parent,
                        store.directory / prefix, observation.snapshot.content_hash, plan_hash,
                        observation.snapshot.scope_hash, origin=request.origin,
                        generated_since=request.generated_since)
            except ContractError as exc:
                envelope, raw, failures = None, None, (Failure("optional_evidence_invalid",
                    str(exc), None, None, None, False, None),)
            failures = tuple(_rebase(failure, prefix) for failure in failures)
            imported = ()
            if envelope is not None:
                raw_reference = prefix + "/" + raw
                known = {metric.key for metric in observation.metrics}
                collisions = sorted(known & {metric.key for metric in envelope.metrics})
                if collisions or envelope.artifact_id in identities:
                    failures = (*failures, Failure("optional_evidence_duplicate",
                        f"duplicate artifact identity or metric keys: {envelope.artifact_id}; {collisions}",
                        None, None, None, False, raw_reference))
                else:
                    # The captured envelope is authoritative evidence for these
                    # imported claims; its raw bytes retain producer artifact
                    # references and full runtime/population provenance.
                    imported = tuple(replace(metric, raw_artifact=raw_reference) for metric in envelope.metrics)
                    identities.add(envelope.artifact_id)
                    note = f"{side} optional {envelope.artifact_id}: origin={envelope.origin}; retained at {raw_reference}"
                    if envelope.origin == "producer_supplied":
                        note += "; provenance is a producer claim, not independently verified"
                    if envelope.runtime_benchmarks:
                        note += "; runtime benchmarks retained separately in the envelope"
                    optional_notes.append(note)
            observations[side] = replace(observation, metrics=(*observation.metrics, *imported),
                                         failures=(*observation.failures, *failures))
            checkpoint()

    try:
        with ExitStack() as stack:
            # Temporary roots are owned outside product, evaluator and artifact
            # trees. The snapshot context owns cleanup of each created subtree.
            if retain_workspaces:
                workroot = Path(tempfile.mkdtemp(prefix="lesscode-evaluation-retained-"))
            else:
                workroot = Path(stack.enter_context(tempfile.TemporaryDirectory(prefix="lesscode-evaluation-")))
            try:
                verify()
                checkpoint()
                for side in ("baseline", "candidate"):
                    if not verify():
                        break
                    source = getattr(prepared.plan, side)
                    copies = stack.enter_context(snapshot.disposable_workspace(
                        Path(source.root), profile.scope, workroot / side, retain=retain_workspaces))
                    if isinstance(copies, Failure):
                        observations[side] = replace(observations[side], failures=(copies,))
                        checkpoint()
                        continue
                    frozen[side] = copies.measurement
                    if copies.measurement.content_hash != source.content_hash:
                        changed.append(side + "/source")
                    if not verify():
                        checkpoint()
                        break
                    stages = (
                        ("sizes", ("python",), ("source_bytes", "physical_lines", "compressed_source_bytes"),
                         lambda logs: metrics.measure_sizes(copies.measurement, Path(bindings["python"].executable), profile.limits, logs)),
                        ("complexity", ("python", "lizard"), ("complexity",),
                         lambda logs: (metrics.measure_complexity(copies.measurement, Path(bindings["lizard"].executable), profile.limits, logs),)),
                        ("duplication", ("python", "node", "jscpd"), ("duplication",),
                         lambda logs: (metrics.measure_duplication(copies.measurement, Path(bindings["python"].executable),
                             Path(bindings["node"].executable), Path(bindings["jscpd"].entry_point), profile.limits, logs),)))
                    for name, needs, keys, measure in stages:
                        if not available(*needs):
                            reason = "; ".join(f"{key}: {bindings[key].reason}" for key in needs
                                               if key in bindings and bindings[key].status != "available")
                            unavailable(side, keys, reason or "required tool binding missing")
                        else:
                            prefix = f"{side}/metrics/{name}"
                            values = workload(lambda: measure(store.directory / prefix))
                            if values is not None:
                                observation = observations[side]
                                observations[side] = replace(observation, metrics=(*observation.metrics,
                                    *(_rebase(value, prefix) for value in values)))
                        checkpoint()
                    commands = {command.command_id: command for command in prepared.plan.resolved_commands}
                    for number, spec in enumerate(profile.checks):
                        command = commands.get(spec.command.command_id)
                        if command is None:
                            observation = observations[side]
                            failure = Failure("check_unavailable", "resolved check binding unavailable",
                                              None, spec.check_id, None, False, None)
                            observations[side] = replace(observation, failures=(*observation.failures, failure))
                            checkpoint()
                            continue
                        if not verify():
                            break
                        checkcopy = snapshot.capture(Path(copies.measurement.root), profile.scope,
                                                     workroot / side / f"check-{number}")
                        if isinstance(checkcopy, Failure):
                            observation = observations[side]
                            observations[side] = replace(observation, failures=(*observation.failures,
                                replace(checkcopy, check_id=spec.check_id)))
                            checkpoint()
                            continue
                        # capture preserves the frozen read-only modes; restore
                        # owner-write for ordinary test-generated modifications.
                        for record in checkcopy.files:
                            (Path(checkcopy.root) / record.path).chmod(record.mode | 0o200)
                        prefix = f"{side}/checks/{number}"
                        result = workload(lambda: process.execute_check(spec, command, Path(checkcopy.root),
                            store.directory / prefix, source.content_hash, plan_hash))
                        if result is not None:
                            checks, execution, failures = result
                            observation = observations[side]
                            replacements = {check.check_id: _rebase(check, prefix) for check in checks}
                            observations[side] = replace(observation,
                                checks=tuple(replacements.get(check.check_id, check) for check in observation.checks),
                                executions=(*observation.executions, _rebase(execution, prefix)),
                                failures=(*observation.failures, *(_rebase(failure, prefix) for failure in failures)))
                        checkpoint()
                    import_optional(side)
                verify()
                checkpoint()
            finally:
                stack.close()
        integrity = IntegrityResult("changed" if changed else "unavailable" if integrity_unavailable else "unchanged",
                                    tuple(dict.fromkeys(changed)), prepared.plan.protection)
        before, after = observations["baseline"], observations["candidate"]
        comparison = decision.compare(before, after, profile, integrity)
        limitations = ["Local change detection is not adversarial isolation; unchecked behavior remains."]
        limitations.extend(optional_notes)
        for side, observation in observations.items():
            limitations.extend(f"{side} {metric.key}: {metric.reason}" for metric in observation.metrics
                               if metric.status != "available")
            limitations.extend(f"{side} {failure.code}: {failure.message}" for failure in observation.failures)
        if retain_workspaces:
            limitations.append("Retained diagnostic workspace: " + str(workroot))
        secrets = any(value is None for spec in profile.checks for value in spec.command.environment.values())
        report = RunReport(1, store.run_id, plan_hash, before, after, integrity, comparison.decision,
                           comparison.reasons, comparison.metric_deltas, _test_delta(before.snapshot, after.snapshot),
                           profile.limits, time.monotonic() - clock, tuple(limitations), started, _now(),
                           EnvironmentInfo(platform.platform(), platform.python_version(), secrets))
        store.finalize(report)
        return report
    except OSError as exc:
        if isinstance(exc, PersistenceError):
            raise
        raise PersistenceError(store.directory, str(exc)) from exc
    finally:
        # Even an interruption leaves already emitted bytes registered whenever
        # possible; failure to register must propagate as a persistence failure.
        if not (store.directory / "artifacts.json").exists():
            register_logs()
