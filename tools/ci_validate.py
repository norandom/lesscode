"""Native CI integration, with durable commands and diagnostics for every stage.

Run with uv and the pinned managed interpreter. Setup may download locked tools;
tests and installed pilot operate locally on the checked-in public fixtures.
"""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time


def venv_python(root):
    # resolve() on bin/python discards POSIX virtual-environment context.
    return Path(os.path.abspath(root)) / ("Scripts/python.exe" if os.name == "nt" else "bin/python")


def npm_entry(node):
    node = Path(node).resolve(strict=True)
    candidates = (node.parent / "node_modules/npm/bin/npm-cli.js",
                  node.parent.parent / "lib/node_modules/npm/bin/npm-cli.js")
    for candidate in candidates:
        if candidate.is_file():
            return candidate.resolve(strict=True)
    raise FileNotFoundError(f"npm JavaScript entry missing beside {node}; checked {candidates}")


class Evidence:
    def __init__(self, root):
        self.root = root
        root.mkdir(parents=True, exist_ok=True)
        self.commands = []

    def run(self, name, argv, *, cwd, env=None, timeout=1800):
        argv = [str(x) for x in argv]
        started = time.time()
        record = {"stage": name, "argv": argv, "cwd": str(cwd),
                  "started_at_epoch": started, "timeout_seconds": timeout,
                  "pythonpath": (os.environ if env is None else env).get("PYTHONPATH")}
        print(f"[{name}] {argv}", flush=True)
        with (self.root / f"{name}.stdout.txt").open("wb") as stdout, (
                self.root / f"{name}.stderr.txt").open("wb") as stderr:
            try:
                result = subprocess.run(argv, cwd=cwd, env=env, stdout=stdout,
                                        stderr=stderr, timeout=timeout, check=False)
                record["exit_code"] = result.returncode
            except (OSError, subprocess.TimeoutExpired) as error:
                record.update(exit_code=None, error=str(error))
                stderr.write(str(error).encode("utf-8"))
        record["elapsed_seconds"] = time.time() - started
        record["passed"] = record["exit_code"] == 0
        self.commands.append(record)
        (self.root / "commands.json").write_text(json.dumps(self.commands, indent=2) + "\n", encoding="utf-8")
        print(f"[{name}] exit={record['exit_code']} elapsed={record['elapsed_seconds']:.3f}s", flush=True)
        return record["passed"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    checkout = Path(__file__).resolve().parents[1]
    output = args.out.resolve()
    if output.is_relative_to(checkout):
        parser.error("CI evidence and installed environment must be outside checkout")
    if output.exists():
        parser.error("use a fresh evidence directory")
    if platform.python_version() != "3.12.10":
        parser.error("CI requires CPython 3.12.10")
    evidence = Evidence(output)
    clean = dict(os.environ)
    clean.pop("PYTHONPATH", None)
    uv = shutil.which("uv.exe" if os.name == "nt" else "uv")
    node = shutil.which("node")
    if not uv or not node:
        parser.error("uv and Node must be provisioned explicitly")
    uv = str(Path(uv).resolve(strict=True))
    # abspath deliberately preserves the venv symlink used for invocation.
    python = Path(os.path.abspath(sys.executable))
    runtime = {"platform": platform.platform(), "python": sys.version,
               "python_invocation": str(python), "python_binary": str(python.resolve()),
               "uv": uv, "node": str(Path(node).resolve()),
               "runner_os": os.environ.get("RUNNER_OS"),
               "runner_arch": os.environ.get("RUNNER_ARCH"),
               "github_sha": os.environ.get("GITHUB_SHA")}
    (output / "runtime.json").write_text(json.dumps(runtime, indent=2) + "\n", encoding="utf-8")
    def run(name, argv, **kwargs):
        return evidence.run(name, argv, cwd=kwargs.pop("cwd", checkout), env=kwargs.pop("env", clean), **kwargs)
    run("uv-version", [uv, "--version"])
    run("node-version", [node, "--version"])
    run("source-revision", ["git", "rev-parse", "HEAD"])
    run("source-status", ["git", "status", "--porcelain"])
    tools = checkout / ".tools/analyzers"
    setup = run("locked-setup", [python, checkout / "tools/setup_toolchain.py", "--node", node,
                               "--npm-cli", npm_entry(node), "--tool-root", tools])
    if setup:
        run("analyzer-probe", [python, checkout / "tools/probe_toolchain.py",
                              "--tool-root", tools, "--output", output / "analyzer-probe.json"])
        shutil.copyfile(tools / "entrypoints.json", output / "entrypoints.json")
    source_env = dict(clean, PYTHONPATH=str(checkout / "src"))
    # Verbose canonical output retains every executed test and host-only skip.
    run("canonical-tests", [python, "-m", "unittest", "discover", "-v"], env=source_env)
    native = ("WindowsProcessTests" if os.name == "nt" else "ProcessTests")
    run("native-process-timeout", [python, "-m", "unittest", "-v",
        f"tests.test_process.{native}.test_timeout_terminates_parent_and_child_and_retains_logs"], env=source_env)
    wheel_dir = output / "wheel"
    built = run("build-wheel", [uv, "build", "--python", python, "--wheel", "--out-dir", wheel_dir])
    if built:
        wheel, = wheel_dir.glob("*.whl")
        installed = output / "installed"
        created = run("clean-venv", [uv, "venv", "--managed-python", "--python", "3.12.10", installed])
        installed_python = venv_python(installed)
        if created and run("install-wheel", [uv, "pip", "install", "--python", installed_python,
                                              "--no-deps", wheel]):
            console = installed_python.with_name("lesscode-baseline.exe" if os.name == "nt" else "lesscode-baseline")
            for name, prefix in (("console", [console]), ("module", [installed_python, "-m", "lesscode_baseline"])):
                for option in ("--help", "--version"):
                    run(name + option, [*prefix, option], cwd=output)
            pilot = [installed_python, checkout / "tools/validate_pilot.py", "--checkout", checkout,
                     "--tool-paths", tools / "entrypoints.json", "--out", output / "pilot"]
            if run("installed-pilot", pilot, cwd=output):
                run("installed-readonly-resume", [*pilot, "--resume"], cwd=output)
    passed = all(record["passed"] for record in evidence.commands)
    (output / "result.json").write_text(json.dumps({"passed": passed, "stages": len(evidence.commands),
        "platform": sys.platform, "limitations": ["native runner only", "checked-in fixtures; no LLM efficacy claim"]}, indent=2) + "\n", encoding="utf-8")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
