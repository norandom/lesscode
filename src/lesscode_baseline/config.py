"""Prepare local, inspectable plans without executing project workloads.

Only fixed runtime/version probes execute here, never a profile command. The
returned records are validated copies; verify_integrity must run before use.
Local fingerprints detect changes and do not provide hostile-code isolation.
"""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from .contracts import (Failure, IntegrityResult, Plan, PreparedPlan,
                        Profile, Protection, ResolvedCommand, ResourceIdentity,
                        ToolBinding, dumps, loads)
from .snapshot import capture_scope

_ROOT = Path(__file__).resolve().parents[2]
_DATA = Path(__file__).resolve().parent / "data"
_UNITS = {"complexity": "CCN", "duplication": "ratio", "source_bytes": "bytes",
          "physical_lines": "lines", "compressed_source_bytes": "bytes",
          "coverage": None, "mutation": None, "nesting": None,
          "architecture": None}


def _hash(value):
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def _resource(logical, path):
    path = Path(path)
    for part in (path, *path.parents):
        if part.is_symlink() or part.is_junction():
            raise ValueError("resource cannot traverse a symlink/reparse point: " + str(path))
    path = path.resolve(strict=True)
    content = path.read_bytes()
    return ResourceIdentity(logical, str(path), hashlib.sha256(content).hexdigest(), len(content))


def _inventory_hash(resources):
    return _hash(json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                             for r in resources], separators=(",", ":")))


def _overlap(a, b):
    return a.is_relative_to(b) or b.is_relative_to(a)


def _probe_environment():
    # Absolute executables need no PATH or runtime startup configuration. On
    # Windows, derive the required OS directory from the OS, not parent input.
    environment = {}
    if os.name == "nt":
        import ctypes
        from ctypes import wintypes

        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        directory = kernel32.GetWindowsDirectoryW
        directory.argtypes = [wintypes.LPWSTR, wintypes.UINT]
        directory.restype = wintypes.UINT
        buffer = ctypes.create_unicode_buffer(32768)
        length = directory(buffer, len(buffer))
        if not length or length >= len(buffer):
            raise OSError("cannot determine Windows runtime directory")
        environment["SystemRoot"] = buffer.value
    return environment


def _probe(argv):
    # Fixed probes do not inherit project PYTHONPATH, cwd or profile environment.
    return subprocess.check_output(argv, cwd=Path(__file__).resolve().parent, text=True, encoding="utf-8",
                                   stderr=subprocess.STDOUT, timeout=10,
                                   env=_probe_environment()).strip()


def _binding(name, requested, executable, entry=None):
    resources = []
    observed = None
    executable = str(Path(executable).resolve()) if executable else None
    entry = str(Path(entry).resolve()) if entry else None
    try:
        if not executable:
            raise ValueError("executable was not found")
        resources.append(_resource("executable", executable))
        if name == "python" or name.startswith("check/"):
            observed = _probe([executable, "-I", "-c", "import platform; print(platform.python_version())"])
        elif name == "node":
            observed = _probe([executable, "--version"]).removeprefix("v")
        elif name == "lizard":
            data = json.loads(_probe([executable, "-I", "-c",
                "import importlib.metadata as m,json; d=m.distribution('lizard'); "
                "print(json.dumps({'version':d.version,'files':[[n+'/'+str(p),str(dist.locate_file(p).resolve())] "
                "for n in ('lizard','pathspec','pygments') for dist in [m.distribution(n)] "
                "for p in dist.files if str(p).endswith(('.py','METADATA'))]}))"]))
            observed = data["version"]
            for logical, path in sorted(data["files"]):
                resources.append(_resource("package/" + logical, path))
        elif name == "jscpd":
            if not entry:
                raise ValueError("jscpd entry point was not found")
            resources.append(_resource("entry_point", entry))
            observed = _probe([executable, entry, "--version"])
            modules = Path(entry).parents[2]
            for path in sorted(modules.rglob("*")):
                if path.is_file() and path.suffix in (".js", ".json", ".cjs", ".mjs"):
                    resources.append(_resource("modules/" + path.relative_to(modules).as_posix(), path))
        else:
            raise ValueError("unsupported analyzer; no version probe is declared")
        if observed != requested:
            raise ValueError(f"version mismatch: expected {requested}, observed {observed}")
        return ToolBinding(name, requested, "available", executable, entry, observed,
                           _inventory_hash(resources), None, tuple(resources))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as exc:
        return ToolBinding(name, requested, "unavailable", executable, entry, observed,
                           _inventory_hash(resources) if resources else None, str(exc), tuple(resources))


def prepare(config, baseline, candidate, output, *, profile_path=None, tool_paths=None):
    """Return a PreparedPlan or Failure; create no outputs and install nothing.

    config is a profile filename or a validated Profile accompanied by its
    profile_path. tool_paths uses setup_toolchain.py's entrypoints.json keys;
    explicit mappings replace setup defaults, permitting absent-tool diagnosis.
    Relative configured tool paths and declared runner resources use profile dir.
    Command cwd/result paths stay relative to the later disposable source copy.
    """
    try:
        supplied_path = Path(profile_path if isinstance(config, Profile) else config).absolute()
        _resource("profile.json", supplied_path)
        path = supplied_path.resolve(strict=True)
        profile = loads(Profile, dumps(config)) if isinstance(config, Profile) else loads(Profile, path.read_bytes())
        if loads(Profile, path.read_bytes()) != profile:
            raise ValueError("supplied Profile differs from profile_path content")
        evaluator = path.parent
        source_paths = (Path(baseline).absolute(), Path(candidate).absolute())
        baseline, candidate = (p.resolve(strict=True) for p in source_paths)
        output = Path(output).resolve()
        if output.exists():
            raise ValueError("output must be new")
        if any(_overlap(output, root) for root in (baseline, candidate, evaluator)):
            raise ValueError("output/source/evaluator roots cannot overlap")
        if any(_overlap(evaluator, root) for root in (baseline, candidate)):
            raise ValueError("source and evaluator roots cannot overlap")
        for key in profile.required_metrics:
            if key not in _UNITS:
                raise ValueError("unsupported required metric: " + key)
        if not {"lizard", "jscpd"}.issubset({tool.name for tool in profile.tools}):
            raise ValueError("required structural analyzers lizard and jscpd must be declared")
        for threshold in profile.thresholds:
            expected = "percent" if threshold.operator == "percentage_delta_maximum" else _UNITS.get(threshold.metric_key)
            if threshold.metric_key not in _UNITS or (expected is not None and threshold.unit != expected):
                raise ValueError("unknown threshold metric or incompatible unit: " + threshold.metric_key)
            if threshold.operator == "absolute_maximum" and threshold.value < 0:
                raise ValueError("absolute maximum cannot be negative")
        resources = []
        for logical in profile.evaluator_resources:
            resource = _resource(logical, evaluator / logical)
            if not Path(resource.path).is_relative_to(evaluator):
                raise ValueError("evaluator resource escapes profile directory")
            resources.append(resource)
        # Preserve profile identity even when the producer omitted it from inventory.
        if not any(r.path == str(path) for r in resources):
            resources.append(_resource("_context/profile.json", path))
        for module in sorted(Path(__file__).parent.glob("*.py")):
            resources.append(_resource("_implementation/" + module.name, module))
        for logical in ("requirements-tools.lock", "tools/package-lock.json"):
            resources.append(_resource("_locks/" + logical, _DATA / logical))
        metadata = _ROOT / ".tools/analyzers/entrypoints.json"
        if tool_paths is None:
            tool_paths = json.loads(metadata.read_text()) if metadata.is_file() else {}
            if metadata.is_file():
                resources.append(_resource("_context/entrypoints.json", metadata))
        tool_paths = dict(tool_paths)
        def resolve_tool(key):
            value = tool_paths.get(key)
            if value:
                p = Path(value)
                return str((evaluator / p).resolve() if not p.is_absolute() else p.resolve())
            return None
        python = resolve_tool("python")
        node = resolve_tool("node")
        entry = resolve_tool("jscpd_entry")
        bindings = [_binding("python", "3.12.10", python), _binding("node", "24.22.1", node)]
        for tool in profile.tools:
            runtime = bindings[0] if tool.name == "lizard" else bindings[1] if tool.name == "jscpd" else None
            if runtime is not None and runtime.status != "available":
                bindings.append(ToolBinding(tool.name, tool.version, "unavailable", runtime.executable,
                                            entry if tool.name == "jscpd" else None, None, None,
                                            "required pinned runtime unavailable: " + runtime.reason, ()))
            else:
                bindings.append(_binding(tool.name, tool.version,
                                         runtime.executable if runtime else None,
                                         entry if tool.name == "jscpd" else None))
        commands = []
        resource_names = {r.logical_path: r.path for r in resources}
        for check in profile.checks:
            spec = check.command
            executable = spec.argv[0]
            if executable in ("python", "python3", "python3.12"):
                binding = bindings[0]
            else:
                p = Path(executable)
                resolved = str((evaluator / p).resolve()) if p.is_absolute() or len(p.parts) > 1 else shutil.which(executable)
                # Never invoke an arbitrary check launcher merely to discover
                # its version: plugins/startup hooks could execute project work.
                if p.name.lower() in ("python", "python.exe", "python3", "python3.12"):
                    binding = _binding("check/" + spec.command_id, "3.12.10", resolved)
                else:
                    binding = ToolBinding("check/" + spec.command_id, "unspecified", "unavailable",
                                          resolved, None, None, None,
                                          "check executable is not a declared Python interpreter; no safe version probe", ())
                bindings.append(binding)
            if binding.status != "available":
                continue
            argv = (binding.executable, *(resource_names.get(arg, arg) for arg in spec.argv[1:]))
            commands.append(ResolvedCommand(spec.command_id, argv, spec.cwd,
                                             dict(spec.environment), spec.limits, binding.observed_version))
        sources = [capture_scope(root, profile.scope) for root in source_paths]
        for source in sources:
            if isinstance(source, Failure):
                return source
        plan = Plan(1, profile, _hash(dumps(profile)), _inventory_hash(resources),
                    *sources, tuple(commands), Protection("local_detection", None))
        return loads(PreparedPlan, dumps(PreparedPlan(1, plan, str(path), str(evaluator),
                                                     tuple(resources), tuple(bindings))))
    except (OSError, ValueError, TypeError) as exc:
        return Failure("plan_preparation", str(exc), None, None, None, False, None)


def _command_integrity(prepared):
    """Reconstruct executable checks from the frozen declarations, without probes."""
    changed = []
    bindings = {tool.name: tool for tool in prepared.tools}
    expected_pins = {"python": "3.12.10", "node": "24.22.1"}
    expected_pins.update((tool.name, tool.version) for tool in prepared.plan.profile.tools)
    expected_names = set(expected_pins)
    resources = {resource.logical_path: resource.path for resource in prepared.evaluator_resources}
    evaluator = Path(prepared.evaluator_root)
    if Path(prepared.profile_path).parent != evaluator:
        changed.append("_context/evaluator-root")
    for logical in prepared.plan.profile.evaluator_resources:
        if resources.get(logical) != str((evaluator / logical).resolve()):
            changed.append("_context/evaluator-resource-paths")
    commands = []
    for check in prepared.plan.profile.checks:
        spec = check.command
        declared = spec.argv[0]
        name = "python" if declared in ("python", "python3", "python3.12") else "check/" + spec.command_id
        expected_names.add(name)
        binding = bindings.get(name)
        if binding is None:
            changed.append("_context/tool-bindings")
            continue
        if name.startswith("check/"):
            path = Path(declared)
            expected_pins[name] = "3.12.10" if path.name.lower() in (
                "python", "python.exe", "python3", "python3.12") else "unspecified"
            if path.is_absolute() or len(path.parts) > 1:
                resolved = str((Path(prepared.evaluator_root) / path).resolve())
                if binding.executable != resolved:
                    changed.append("_context/tool-bindings")
        if binding.status == "available":
            commands.append(ResolvedCommand(spec.command_id,
                (binding.executable, *(resources.get(arg, arg) for arg in spec.argv[1:])),
                spec.cwd, dict(spec.environment), spec.limits, binding.observed_version))
    if tuple(commands) != prepared.plan.resolved_commands:
        changed.append("_context/resolved-commands")
    if set(bindings) != expected_names:
        changed.append("_context/tool-bindings")
    for name, tool in bindings.items():
        prefix = "_tools/" + _hash(name) + "/"
        identities = {resource.logical_path: resource.path for resource in tool.resources}
        if tool.requested_version != expected_pins.get(name):
            changed.append(prefix + "version")
        for attribute, logical in (("executable", "executable"), ("entry_point", "entry_point")):
            recorded = identities.get(logical)
            value = getattr(tool, attribute)
            if (recorded is not None and value != recorded) or (
                    tool.status == "available" and
                    (attribute == "executable" or value is not None) and recorded is None):
                changed.append(prefix + logical)
        if tool.status == "available" and name in ("lizard", "jscpd"):
            runtime = bindings.get("python" if name == "lizard" else "node")
            if runtime is None or runtime.status != "available" or runtime.executable != tool.executable:
                changed.append(prefix + "runtime")
    return changed


def verify_integrity(prepared):
    """Identify modified/deleted resources after reloading a durable plan.

    Does not run any command, including version probes. A mismatch invalidates
    evaluation; unavailable bindings remain unavailable independently of this.
    """
    changed = []
    plan = prepared.plan
    changed.extend(_command_integrity(prepared))
    if _hash(dumps(plan.profile)) != plan.profile_hash:
        changed.append("_context/frozen-profile")
    if _inventory_hash(prepared.evaluator_resources) != plan.evaluator_hash:
        changed.append("_context/evaluator-inventory")
    for resource in prepared.evaluator_resources:
        try:
            if _resource(resource.logical_path, resource.path) != resource:
                changed.append(resource.logical_path)
        except (OSError, ValueError):
            changed.append(resource.logical_path)
    for tool in prepared.tools:
        prefix = "_tools/" + tool.name + "/"
        if tool.fingerprint is not None and _inventory_hash(tool.resources) != tool.fingerprint:
            changed.append(prefix + "inventory")
        for resource in tool.resources:
            try:
                if _resource(resource.logical_path, resource.path) != resource:
                    changed.append(prefix + resource.logical_path)
            except (OSError, ValueError):
                changed.append(prefix + resource.logical_path)
    return IntegrityResult("changed" if changed else "unchanged", tuple(dict.fromkeys(changed)), plan.protection)
