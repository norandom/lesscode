"""Recoverable local run artifacts, with an index published only after finalization.

RunReport remains the public result contract. The records below describe only
local storage. Human summaries present the same evidence without deciding it.
"""

from dataclasses import dataclass, replace
from datetime import datetime, timezone
import hashlib
import html
import json
import os
from pathlib import Path
from typing import Literal
import uuid

from .contracts import (
    ContractError, ExperimentReport, PreparedPlan, Record, RunReport, dumps, loads,
    _hash, _integer, _object, _relative, _sequence, _string, _timestamp, _version,
)

_RESERVED = {"manifest.json", "plan.json", "report.json", "summary.md", "artifacts.json"}


def _literal(value):
    """Keep imported text inert in Markdown, including HTML, links and newlines."""
    text = str(value).replace("\r", "\\r").replace("\n", "\\n").replace("\t", "\\t")
    text = "".join(c if ord(c) >= 32 and ord(c) != 127 else f"\\u{ord(c):04x}" for c in text)
    return "`" + html.escape(text, quote=True).replace("`", "&#96;") + "`"


def _value(value):
    if value is None:
        return "unavailable"
    if isinstance(value, Record):
        return _literal(json.dumps(value.to_dict(), ensure_ascii=True, sort_keys=True))
    return _literal(value)


def _product_changes(report):
    before = {f.path: f for f in report.baseline.snapshot.files if f.category == "product"}
    after = {f.path: f for f in report.candidate.snapshot.files if f.category == "product"}
    return {"added": sorted(after.keys() - before.keys()),
            "removed": sorted(before.keys() - after.keys()),
            "modified": sorted(p for p in before.keys() & after.keys() if before[p] != after[p])}


def render_summary(report: RunReport) -> str:
    """Render validated recorded evidence; never recalculate decisions or scores."""
    dumps(report)
    if not isinstance(report, RunReport):
        raise ContractError("expected RunReport")
    lines = ["# Evaluation summary", "", f"Report format version: {report.schema_version}",
             f"Run: {_literal(report.run_id)}; plan: {_literal(report.plan_hash)}",
             f"Decision: {_literal(report.decision)}", "", "## Decision reasons", ""]
    lines.extend("- " + _literal(r) for r in report.reasons)
    if not report.reasons:
        lines.append("- No decision reasons recorded.")
    lines += ["", "## Declared checks", "",
              "Passing results mean passing the declared checks; this is not proof of equivalence. Unchecked behavior remains unverified."]
    for label, observation in (("Baseline", report.baseline), ("Candidate", report.candidate)):
        lines += ["", f"### {label}", ""]
        if not observation.checks:
            lines.append("- Check evidence unavailable: no outcomes recorded.")
        for check in observation.checks:
            lines.append(f"- {_literal(check.check_id)}: {_literal(check.status)}; kind {_literal(check.kind)}; {'required' if check.required else 'optional'}; diagnostics {_value(check.diagnostic_artifact)}")
        for execution in observation.executions:
            lines.append(f"- Execution {_literal(execution.command.command_id)}: {_literal(execution.status)}; exit {_value(execution.exit_code)}; elapsed {_literal(execution.elapsed_seconds)} seconds; stdout {_literal(execution.stdout_artifact)}; stderr {_literal(execution.stderr_artifact)}")
        for failure in observation.failures:
            lines.append(f"- Failure {_literal(failure.code)}: {_literal(failure.message)}; path {_value(failure.affected_path)}; check {_value(failure.check_id)}; metric {_value(failure.metric_key)}; diagnostics {_value(failure.raw_artifact)}")
    lines += ["", "## Integrity and protections", "",
              f"Status: {_literal(report.integrity.status)}; protection: {_literal(report.integrity.protection.mode)}."]
    if report.integrity.protection.mode == "local_detection":
        lines.append("Local change detection does not isolate malicious workloads or prevent privileged writes.")
    else:
        lines.append("External isolation is a provider claim; evidence reference: " + _value(report.integrity.protection.provider_evidence))
    lines.extend("- Changed evaluator resource: " + _literal(p) for p in report.integrity.changed_resources)
    lines += ["", "## Structural and other metrics", "",
              "Each producer and definition is separate. Structured payloads are retained as recorded; no combined quality score is calculated."]
    before = {m.key: m for m in report.baseline.metrics}
    after = {m.key: m for m in report.candidate.metrics}
    deltas = {d.key: d for d in report.metric_deltas}
    # Evidence priorities: test strength/API, complexity/duplication, size, cost.
    def priority(key):
        groups = (("coverage", "mutation", "api", "invariant"), ("complexity", "duplication", "nesting", "architecture"), ("bytes", "lines", "size"), ("cost", "token"))
        return next((i for i, words in enumerate(groups) if any(w in key.lower() for w in words)), 2), key
    for key in sorted(before.keys() | after.keys() | deltas.keys(), key=priority):
        lines += ["", "### " + _literal(key), ""]
        for label, metrics in (("Before", before), ("After", after)):
            metric = metrics.get(key)
            if metric is None:
                lines.append(f"- {label}: unavailable (missing evidence).")
                continue
            lines.append(f"- {label}: {_literal(metric.status)}; value {_value(metric.value)}; unit {_literal(metric.unit)}; producer {_literal(metric.producer)} version {_literal(metric.producer_version)}; definition {_literal(metric.definition_id)}; scope {_literal(metric.scope_hash)}; reason {_value(metric.reason)}; raw {_value(metric.raw_artifact)}")
        delta = deltas.get(key)
        if delta is None:
            lines.append("- Delta unavailable: no comparison recorded.")
        else:
            lines.append(f"- Comparison: {_literal(delta.status)}; before {_value(delta.before)}; after {_value(delta.after)}; absolute delta {_value(delta.absolute_delta)} {_literal(delta.unit)}; percentage delta {_value(delta.percentage_delta)} percent; reason {_value(delta.reason)}; percentage reason {_value(delta.percentage_reason)}")
    if not (before or after or deltas):
        lines.append("Metric evidence unavailable: no measurements recorded.")
    lines += ["", "## File changes", ""]
    for label, changes in (("Product files", _product_changes(report)), ("Test files", report.test_file_delta.to_dict())):
        lines.append(f"- {label}:")
        for kind, paths in changes.items():
            lines.append(f"  - {kind}: " + (", ".join(_literal(p) for p in paths) or "none"))
    lines += ["", "## Time and cost", "",
              f"Evaluation elapsed: {_literal(report.elapsed_seconds)} seconds.",
              "Agent execution time: unavailable (not supplied in RunReport).",
              "Agent token usage and provider cost: unavailable (not supplied in RunReport).",
              "", "## Limitations", ""]
    lines.extend("- " + _literal(limit) for limit in report.limitations)
    if not report.limitations:
        lines.append("- No additional limitations recorded; unchecked behavior remains unverified.")
    return "\n".join(lines) + "\n"


class PersistenceError(OSError):
    """Persistence failed; callers must expose the surviving directory (exit 4)."""

    exit_code = 4

    def __init__(self, directory: Path, message: str):
        self.directory = directory
        super().__init__(f"{message}; surviving artifact directory: {directory}")


@dataclass(frozen=True)
class ArtifactIdentity(Record):
    path: str
    content_hash: str
    size_bytes: int

    @classmethod
    def from_dict(cls, value, path="$input"):
        d = _object(value, path, "path content_hash size_bytes")
        return cls(_relative(d["path"], path + ".path"),
                   _hash(d["content_hash"], path + ".content_hash"),
                   _integer(d["size_bytes"], path + ".size_bytes"))


@dataclass(frozen=True)
class ArtifactIndex(Record):
    schema_version: int
    run_id: str
    plan_hash: str
    artifacts: tuple[ArtifactIdentity, ...]

    @classmethod
    def from_dict(cls, value, path="$input"):
        d = _object(value, path, "schema_version run_id plan_hash artifacts")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   _string(d["run_id"], path + ".run_id"),
                   _hash(d["plan_hash"], path + ".plan_hash"),
                   _sequence(d["artifacts"], ArtifactIdentity.from_dict,
                             path + ".artifacts", lambda a: a.path.casefold()))


@dataclass(frozen=True)
class RunManifest(Record):
    schema_version: int
    run_id: str
    plan_hash: str
    state: Literal["prepared", "running", "finalized"]
    created_at: str
    updated_at: str
    raw_artifacts: tuple[ArtifactIdentity, ...]

    @classmethod
    def from_dict(cls, value, path="$input"):
        d = _object(value, path, "schema_version run_id plan_hash state created_at updated_at raw_artifacts")
        if d["state"] not in ("prepared", "running", "finalized"):
            raise ContractError(path + ".state: invalid run state")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   _string(d["run_id"], path + ".run_id"),
                   _hash(d["plan_hash"], path + ".plan_hash"), d["state"],
                   _timestamp(d["created_at"], path + ".created_at"),
                   _timestamp(d["updated_at"], path + ".updated_at"),
                   _sequence(d["raw_artifacts"], ArtifactIdentity.from_dict,
                             path + ".raw_artifacts", lambda a: a.path.casefold()))


@dataclass(frozen=True)
class Recovery:
    directory: Path
    state: Literal["complete", "incomplete"]
    manifest: RunManifest | None
    report: RunReport | None
    index: ArtifactIndex | None
    diagnostics: tuple[str, ...]


def _now():
    return datetime.now(timezone.utc).isoformat()


def _no_links(path):
    for part in (path, *path.parents):
        if part.is_symlink() or part.is_junction():
            raise ContractError(f"artifact path traverses symlink/reparse point: {path}")


def _safe(root, relative):
    _relative(relative, "$artifact")
    path = root / relative
    _no_links(path)
    if not path.resolve().is_relative_to(root):
        raise ContractError(f"artifact path escapes run directory: {relative}")
    return path


def _identity(root, relative):
    path = _safe(root, relative)
    if not path.is_file():
        raise OSError(f"referenced artifact is not a regular file: {relative}")
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        before = os.fstat(handle.fileno())
        while chunk := handle.read(1024 * 1024):
            digest.update(chunk)
            size += len(chunk)
        after = os.fstat(handle.fileno())
    if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
        raise OSError(f"artifact changed while hashing: {relative}")
    return ArtifactIdentity(relative, digest.hexdigest(), size)


def _references(report):
    paths = set()
    for observation in (report.baseline, report.candidate):
        paths.update(m.raw_artifact for m in observation.metrics if m.raw_artifact)
        paths.update(c.diagnostic_artifact for c in observation.checks if c.diagnostic_artifact)
        paths.update(f.raw_artifact for f in observation.failures if f.raw_artifact)
        for execution in observation.executions:
            paths.update((execution.stdout_artifact, execution.stderr_artifact))
    return sorted(paths)


class RunStore:
    """Own one new run directory. Workloads may write logs below this root.

    Call register_artifact as each command finishes, or write_artifact to retain
    supplied bytes. A crash before registration still leaves the actual logs on
    disk; recovery never deletes them. Finalization seals this store instance.
    """

    def __init__(self, directory, manifest):
        self.directory = directory
        self.manifest = manifest
        self.run_id = manifest.run_id
        self._owned = {"manifest.json"}
        self._sealed = False

    @classmethod
    def create(cls, target: Path, plan_hash: str, *, run_id: str | None = None,
               prepared_plan: PreparedPlan | None = None) -> "RunStore":
        target = Path(target).absolute()
        if ".." in target.parts:
            raise ContractError("output target cannot contain traversal")
        _no_links(target)
        target = target.resolve()
        now = _now()
        manifest = RunManifest(1, uuid.uuid4().hex if run_id is None else run_id,
                               plan_hash, "prepared", now, now, ())
        dumps(manifest)
        plan_text = None if prepared_plan is None else dumps(prepared_plan)
        if prepared_plan is not None and not isinstance(prepared_plan, PreparedPlan):
            raise ContractError("prepared_plan must be a PreparedPlan")
        try:
            target.mkdir(parents=True, exist_ok=False)
            store = cls(target, manifest)
            store._atomic("manifest.json", dumps(manifest).encode("utf-8"))
            if plan_text is not None:
                store._atomic("plan.json", plan_text.encode("utf-8"))
            return store
        except OSError as exc:
            raise PersistenceError(target, str(exc)) from exc

    def _open(self):
        _no_links(self.directory)
        if self._sealed or _safe(self.directory, "artifacts.json").exists():
            raise PersistenceError(self.directory, "run is already finalized or finalization was attempted")

    def _atomic(self, relative, data):
        target = _safe(self.directory, relative)
        if target.exists() and relative not in self._owned:
            raise OSError(f"refusing to overwrite unowned artifact: {relative}")
        temporary = _safe(self.directory, "." + relative + "." + uuid.uuid4().hex + ".tmp")
        # Leave interrupted temporary bytes available for diagnosis.
        with temporary.open("xb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        _safe(self.directory, relative)
        os.replace(temporary, target)
        self._owned.add(relative)

    def _manifest(self, **changes):
        manifest = replace(self.manifest, updated_at=_now(), **changes)
        self._atomic("manifest.json", dumps(manifest).encode("utf-8"))
        self.manifest = manifest

    def start(self):
        self._open()
        if self.manifest.state != "prepared":
            raise PersistenceError(self.directory, "only prepared runs can start")
        try:
            self._manifest(state="running")
        except OSError as exc:
            raise PersistenceError(self.directory, str(exc)) from exc

    def _raw_path(self, relative):
        path = _safe(self.directory, relative)
        if relative.split("/")[0].casefold() in _RESERVED:
            raise ContractError("raw artifact conflicts with an owned run document")
        return path

    def write_artifact(self, relative: str, data: bytes) -> ArtifactIdentity:
        """Create raw bytes exclusively; never truncate a previous artifact."""
        self._open()
        path = self._raw_path(relative)
        try:
            path.parent.mkdir(parents=True, exist_ok=True)
            self._raw_path(relative)
            with path.open("xb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
            return self.register_artifact(relative)
        except OSError as exc:
            raise PersistenceError(self.directory, str(exc)) from exc

    def register_artifact(self, relative: str) -> ArtifactIdentity:
        """Retain an identity for a workload's log under this store's root."""
        self._open()
        self._raw_path(relative)
        try:
            identity = _identity(self.directory, relative)
            prior = {a.path.casefold(): a for a in self.manifest.raw_artifacts}
            if relative.casefold() in prior:
                if prior[relative.casefold()] != identity:
                    raise OSError(f"registered artifact changed or aliases another path: {relative}")
                return identity
            self._manifest(raw_artifacts=(*self.manifest.raw_artifacts, identity))
            return identity
        except OSError as exc:
            raise PersistenceError(self.directory, str(exc)) from exc

    def finalize(self, report: RunReport) -> ArtifactIndex:
        """Validate and publish report, final manifest, then the completion index."""
        self._open()
        text = dumps(report)
        if not isinstance(report, RunReport):
            raise ContractError("expected RunReport")
        if (report.run_id, report.plan_hash) != (self.run_id, self.manifest.plan_hash):
            raise PersistenceError(self.directory, "report run/plan identity differs from manifest")
        try:
            for relative in _references(report):
                self.register_artifact(relative)
            # Recheck registered raw outputs so modifications cannot be silently blessed.
            for artifact in self.manifest.raw_artifacts:
                if _identity(self.directory, artifact.path) != artifact:
                    raise OSError(f"registered artifact changed: {artifact.path}")
            self._sealed = True
            self._atomic("report.json", text.encode("utf-8"))
            self._atomic("summary.md", render_summary(report).encode("utf-8"))
            self._manifest(state="finalized")
            paths = ["manifest.json", "report.json", "summary.md"]
            if "plan.json" in self._owned:
                paths.append("plan.json")
            paths += [a.path for a in self.manifest.raw_artifacts]
            index = ArtifactIndex(1, self.run_id, self.manifest.plan_hash,
                                  tuple(_identity(self.directory, p) for p in sorted(paths)))
            self._atomic("artifacts.json", dumps(index).encode("utf-8"))
            return index
        except OSError as exc:
            raise PersistenceError(self.directory, str(exc)) from exc


def recover(target: Path) -> Recovery:
    """Read data only; absent, malformed or mismatched completion is incomplete."""
    directory = Path(target).absolute()
    manifest = None
    try:
        _no_links(directory)
        directory = directory.resolve(strict=True)
        manifest = loads(RunManifest, _safe(directory, "manifest.json").read_bytes())
        index = loads(ArtifactIndex, _safe(directory, "artifacts.json").read_bytes())
        if manifest.state != "finalized" or (index.run_id, index.plan_hash) != (manifest.run_id, manifest.plan_hash):
            raise ContractError("index/manifest state or identity mismatch")
        by_path = {a.path: a for a in index.artifacts}
        required = {"manifest.json", "report.json", "summary.md", *(a.path for a in manifest.raw_artifacts)}
        if _safe(directory, "plan.json").exists():
            required.add("plan.json")
        if not required.issubset(by_path):
            raise ContractError("index omits required run artifacts")
        if any(by_path[a.path] != a for a in manifest.raw_artifacts):
            raise ContractError("index/manifest raw artifact identities differ")
        for artifact in index.artifacts:
            if _identity(directory, artifact.path) != artifact:
                raise ContractError(f"artifact identity mismatch: {artifact.path}")
        report = loads(RunReport, _safe(directory, "report.json").read_bytes())
        if (report.run_id, report.plan_hash) != (manifest.run_id, manifest.plan_hash):
            raise ContractError("report/manifest identity mismatch")
        if not set(_references(report)).issubset(by_path):
            raise ContractError("report references unindexed raw artifacts")
        return Recovery(directory, "complete", manifest, report, index, ())
    except (OSError, ContractError, ValueError) as exc:
        return Recovery(directory, "incomplete", manifest, None, None, (str(exc),))


def persist(report: RunReport, target: Path, *, artifact_root: Path | None = None,
            prepared_plan: PreparedPlan | None = None) -> ArtifactIndex:
    """Persist an already computed report into a new directory.

    Referenced raw artifacts must exist below the explicitly supplied root;
    their bytes are copied rather than fabricating logs or resolving paths
    against the current working directory. Lifecycle callers should use
    RunStore to retain running state and logs before a report exists.
    """
    dumps(report)
    if not isinstance(report, RunReport):
        raise ContractError("expected RunReport")
    store = RunStore.create(target, report.plan_hash, run_id=report.run_id,
                            prepared_plan=prepared_plan)
    try:
        references = _references(report)
        if references and artifact_root is None:
            raise OSError("report references require an explicit artifact_root")
        if artifact_root is not None:
            root = Path(artifact_root).absolute()
            _no_links(root)
            root = root.resolve(strict=True)
            for relative in references:
                source = _safe(root, relative)
                if not source.is_file():
                    raise OSError(f"referenced artifact is not a regular file: {relative}")
                store.write_artifact(relative, source.read_bytes())
        return store.finalize(report)
    except OSError as exc:
        raise PersistenceError(store.directory, str(exc)) from exc


def render_experiment_summary(aggregation) -> str:
    """Render recorded populations, never recompute a decision or infer costs."""
    report = ExperimentReport.from_dict(aggregation.report.to_dict())
    lines = ["# Recorded experiment summary", "", "Format version: 1.",
             "A successful summary command records outcomes; it does not assert candidate acceptance or equivalence.",
             "", "## Corpus and declared controls", "",
             "Experiment: " + _literal(report.manifest.experiment_id)]
    for entry in report.manifest.corpus:
        lines.append("- Corpus/source/license/generation_provenance: " + _value(entry.to_dict()))
    lines += ["- Declared controls: " + _value(report.manifest.controls.to_dict()),
              "- Allowed interventions: " + _value(report.manifest.allowed_interventions)]
    for arm in report.manifest.arms:
        lines.append("- Arm: " + _value(arm.to_dict()))
    for trial in report.manifest.planned_trials:
        lines.append("- Planned task/repetition/prompt: " + _value(trial.to_dict()))
    lines += ["", "## Outcomes and denominators", "",
              "Correctness is passing all required behavior checks among eligible attempts. Acceptance additionally requires the recorded structural rules.",
              "Missing and invalid baseline evidence and baseline-ineligible attempts cannot establish behavior success. Extra records never expand the scheduled denominator."]
    for arm in report.arm_summaries:
        rate = (100 * arm.correct_attempts / arm.eligible_attempts) if arm.eligible_attempts else None
        acceptance = (100 * arm.state_counts["accepted"] / arm.planned_attempts) if arm.planned_attempts else None
        lines += ["", "### Arm " + _literal(arm.arm_id), "",
                  "- Per-state counts: " + _value(arm.state_counts),
                  f"- Correctness: numerator {arm.correct_attempts}; eligible-attempt denominator {arm.eligible_attempts}; rate {_value(rate)} percent.",
                  f"- Acceptance: numerator {arm.state_counts['accepted']}; planned-attempt denominator {arm.planned_attempts}; rate {_value(acceptance)} percent.",
                  f"- Budget-exhausted attempts: {arm.budget_exhausted_attempts}."]
        for label, distributions in (("All-attempt", arm.all_attempt_structural), ("Accepted-only", arm.accepted_only_structural)):
            lines.append("- " + label + " structural absolute deltas (after minus before):")
            lines.extend("  - " + _value(d.to_dict()) for d in distributions)
            if not distributions:
                lines.append("  - unavailable: no comparable scalar samples.")
    lines += ["", "## Paired variation", "",
              "Descriptive n, mean, median, min/max and sample standard deviation only; standard deviation is unavailable for n < 2. No inferential confidence interval. Paired differences are right-arm minus left-arm deltas; confounded comparisons are excluded."]
    lines.extend("- " + _value(pair.to_dict()) for pair in report.paired_summaries)
    for outcome in report.outcomes:
        lines.append("- Outcome/reasons/confounding: " + _value(outcome.to_dict()))
    lines += ["", "## Producer conditions and observed agent telemetry", "",
              "Producer-supplied observations retain their source, unit and timestamp. Unavailable tokens or cost remain unavailable; none are inferred from source size. Currencies and units are never combined."]
    for label, records in (("Scheduled", report.trials), ("Extra (excluded from denominators)", report.extra_trials)):
        for trial in records:
            lines += ["", "### " + label + " trial " + _literal(trial.trial_id), "",
                      "- Starting identity, conditions, execution order, seed and completion: " +
                      _value({k: v for k, v in trial.to_dict().items() if k != "telemetry"}),
                      "- Observed token usage: " + _value(trial.telemetry.token_usage.to_dict()),
                      "- Observed provider cost: " + _value(trial.telemetry.provider_cost.to_dict()),
                      "- Observed agent elapsed: " + _value(trial.telemetry.agent_elapsed.to_dict()),
                      "- Runtime benchmarks (workload/environment separate from structural and agent cost):"]
            lines.extend("  - " + _value(b.to_dict()) for b in trial.telemetry.runtime_benchmarks)
            if not trial.telemetry.runtime_benchmarks:
                lines.append("  - unavailable (not supplied).")
    lines += ["", "## Evaluation elapsed and retained evidence", ""]
    for run in aggregation.runs:
        lines.append(f"- Trial {_literal(run.trial_id)}: Evaluation elapsed {_literal(run.report.elapsed_seconds)} seconds; origin {_literal(run.origin)}; run {_literal(run.report.run_id)}; plan {_literal(run.report.plan_hash)}. Full source identities and observations retained in evaluations.json; raw logs remain in the recorded run artifact.")
    lines += ["", "## Limitations", ""]
    lines.extend("- " + _literal(limit) for limit in report.limitations)
    return "\n".join(lines) + "\n"


@dataclass(frozen=True)
class ExperimentStorageManifest(Record):
    """Storage envelope independent of the RunReport lifecycle."""
    schema_version: int
    experiment_id: str
    report_hash: str
    state: Literal["prepared", "finalized"]

    @classmethod
    def from_dict(cls, value, path="$input"):
        d = _object(value, path, "schema_version experiment_id report_hash state")
        if d["state"] not in ("prepared", "finalized"):
            raise ContractError(path + ".state: invalid experiment storage state")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   _string(d["experiment_id"], path + ".experiment_id"),
                   _hash(d["report_hash"], path + ".report_hash"), d["state"])


@dataclass(frozen=True)
class ExperimentArtifactIndex(Record):
    schema_version: int
    experiment_id: str
    report_hash: str
    artifacts: tuple[ArtifactIdentity, ...]

    @classmethod
    def from_dict(cls, value, path="$input"):
        d = _object(value, path, "schema_version experiment_id report_hash artifacts")
        return cls(_version(d["schema_version"], path + ".schema_version"),
                   _string(d["experiment_id"], path + ".experiment_id"),
                   _hash(d["report_hash"], path + ".report_hash"),
                   _sequence(d["artifacts"], ArtifactIdentity.from_dict, path + ".artifacts", lambda a: a.path.casefold()))


def persist_experiment(aggregation, target: Path) -> ExperimentArtifactIndex:
    """Publish independent machine/human experiment artifacts, index last.

    Failed writes retain manifest, completed files and temporary bytes. A
    finalized manifest alone is insufficient: completion requires the index and
    matching hashes for every indexed file. Input run artifacts remain read-only.
    """
    report = ExperimentReport.from_dict(aggregation.report.to_dict())
    machine = dumps(report).encode("utf-8")
    summary = render_experiment_summary(aggregation).encode("utf-8")
    evidence = []
    records = {trial.trial_id: trial for trial in report.trials}
    for run in aggregation.runs:
        validated = RunReport.from_dict(run.report.to_dict())
        if run.trial_id not in records:
            raise ContractError("experiment evaluation has no scheduled producer record")
        evidence.append({"trial_id": run.trial_id, "origin": run.origin,
                         "run_artifact": records[run.trial_id].run_artifact,
                         "report": validated.to_dict()})
    evaluations = json.dumps({"schema_version": 1, "runs": evidence}, ensure_ascii=False,
                             allow_nan=False, sort_keys=True).encode("utf-8")
    target = Path(target).absolute()
    if ".." in target.parts:
        raise ContractError("output target cannot contain traversal")
    _no_links(target)
    target = target.resolve()
    manifest = ExperimentStorageManifest(1, report.manifest.experiment_id,
                                         hashlib.sha256(machine).hexdigest(), "prepared")
    def atomic(name, data):
        path = _safe(target, name)
        if path.exists() and name != "manifest.json":
            raise OSError("refusing to overwrite an existing experiment artifact: " + name)
        temporary = _safe(target, "." + name + "." + uuid.uuid4().hex + ".tmp")
        with temporary.open("xb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        _safe(target, name)
        os.replace(temporary, path)
    try:
        target.mkdir(parents=True, exist_ok=False)
        atomic("manifest.json", dumps(manifest).encode("utf-8"))
        atomic("report.json", machine)
        atomic("evaluations.json", evaluations)
        atomic("summary.md", summary)
        atomic("manifest.json", dumps(replace(manifest, state="finalized")).encode("utf-8"))
        index = ExperimentArtifactIndex(1, manifest.experiment_id, manifest.report_hash,
                tuple(_identity(target, path) for path in ("manifest.json", "report.json", "evaluations.json", "summary.md")))
        atomic("artifacts.json", dumps(index).encode("utf-8"))
        return index
    except OSError as exc:
        raise PersistenceError(target, str(exc)) from exc


@dataclass(frozen=True)
class ExperimentRecovery:
    state: Literal["complete", "incomplete"]
    report: ExperimentReport | None
    diagnostics: tuple[str, ...]


def recover_experiment(target: Path) -> ExperimentRecovery:
    """A manifest without a matching, hash-verified final index is incomplete."""
    try:
        target = Path(target).absolute()
        _no_links(target)
        target = target.resolve(strict=True)
        manifest = loads(ExperimentStorageManifest, _safe(target, "manifest.json").read_bytes())
        index = loads(ExperimentArtifactIndex, _safe(target, "artifacts.json").read_bytes())
        if manifest.state != "finalized" or (manifest.experiment_id, manifest.report_hash) != (index.experiment_id, index.report_hash):
            raise ContractError("experiment manifest/index state or identity mismatch")
        required = {"manifest.json", "report.json", "evaluations.json", "summary.md"}
        if {a.path for a in index.artifacts} != required:
            raise ContractError("experiment index must identify all final artifacts")
        for artifact in index.artifacts:
            if _identity(target, artifact.path) != artifact:
                raise ContractError("experiment artifact identity mismatch: " + artifact.path)
        data = _safe(target, "report.json").read_bytes()
        report = loads(ExperimentReport, data)
        if hashlib.sha256(data).hexdigest() != manifest.report_hash or report.manifest.experiment_id != manifest.experiment_id:
            raise ContractError("experiment report identity mismatch")
        return ExperimentRecovery("complete", report, ())
    except (OSError, ContractError, ValueError) as exc:
        return ExperimentRecovery("incomplete", None, (str(exc),))
