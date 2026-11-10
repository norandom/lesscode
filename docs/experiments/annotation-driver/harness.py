"""Build-and-test harness for the annotation-driver prototype gate (Amendment 3e).

Runs a scope's optional build and its test command from ``scopes.json`` in the
isolated clone. Each mutant is written in turn and the original bytes are
restored and verified after every run. Commands, environment, limits and the
signals of an invalid run all come from the scope entry; nothing here is
specific to a language or repository.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import engine as e
import validator as v

HERE = Path(__file__).parent
CMD_SPECIAL = set(' \t&|<>^%"')  # characters cmd /c would reinterpret in an argument
_last_write = [0]


@dataclass
class Run:
    step: str
    command: list[str]
    exit_code: int | None
    timed_out: bool
    seconds: float
    log: str


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def scope_config(name: str) -> dict:
    manifest = json.loads((HERE / "scopes.json").read_text(encoding="utf-8"))
    return next(s for s in manifest["scopes"] if s["name"] == name)


def expand(text: str, scopes_dir: Path) -> str:
    for key, path in (("{scopes}", scopes_dir), ("{work}", scopes_dir.parent), ("{experiment}", HERE)):
        text = text.replace(key, str(path.resolve()))
    return text


def base_env() -> dict[str, str]:
    """The parent environment without the harness's own interpreter environment.

    Variables pointing at this interpreter's prefix, and PATH entries under
    it, would otherwise redirect a scope's own tools to this environment.
    """
    def norm(path: str) -> str:
        return os.path.normcase(os.path.normpath(path or "."))

    own = norm(sys.prefix)
    env = {k: val for k, val in os.environ.items() if norm(val) != own}
    paths = env.get("PATH", "").split(os.pathsep)
    env["PATH"] = os.pathsep.join(p for p in paths if norm(p) != own and not norm(p).startswith(own + os.sep))
    return env


def resolve(argv: list[str]) -> list[str]:
    """Absolute executable; a .cmd or .bat shim runs through cmd /c, without a shell."""
    exe = shutil.which(argv[0])
    if exe is None:
        raise FileNotFoundError(f"executable not found on PATH: {argv[0]}")
    if not exe.lower().endswith((".cmd", ".bat")):
        return [exe, *argv[1:]]
    unsafe = [a for a in argv[1:] if CMD_SPECIAL & set(a)]
    if unsafe:
        raise ValueError(f"arguments that cmd /c would reinterpret: {unsafe}")
    return [os.environ.get("COMSPEC", "cmd.exe"), "/c", exe, *argv[1:]]


def _kill_tree(proc: subprocess.Popen) -> None:
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True)
    else:
        os.killpg(proc.pid, signal.SIGKILL)
    proc.wait()


def run(step: str, argv: list[str], cwd: Path, env: dict, timeout_s: int, log_path: Path) -> Run:
    """One command, stdout and stderr to ``log_path``; the process tree is killed on timeout."""
    start = time.monotonic()
    with log_path.open("wb") as log:
        proc = subprocess.Popen(resolve(argv), cwd=cwd, env=env, stdin=subprocess.DEVNULL, stdout=log,
                                stderr=subprocess.STDOUT, start_new_session=os.name != "nt")
        try:
            code, timed_out = proc.wait(timeout=timeout_s), False
        except subprocess.TimeoutExpired:
            _kill_tree(proc)
            code, timed_out = None, True
        except BaseException:
            _kill_tree(proc)
            raise
    return Run(step, argv, code, timed_out, round(time.monotonic() - start, 2), log_path.name)


def check(cfg: dict, root: Path, scopes_dir: Path, out_dir: Path, label: str) -> dict:
    """Optional build, then test, classified as in Amendment 3e."""
    env = {**base_env(), **{k: expand(val, scopes_dir) for k, val in cfg.get("env", {}).items()}}
    runs: list[dict] = []
    for step in ("build", "test"):
        if not cfg.get(step):
            continue
        argv = [expand(a, scopes_dir) for a in cfg[step]]
        result = run(step, argv, root, env, cfg["timeout_s"], out_dir / f"{label}-{step}.log")
        runs.append(asdict(result))
        if result.timed_out:
            return {"class": "timeout", "reason": f"{step}_timeout", "runs": runs}
        if step == "build" and result.exit_code != 0:
            return {"class": "invalid", "reason": "build_failed", "runs": runs}
    test = runs[-1]
    output = (out_dir / test["log"]).read_text(encoding="utf-8", errors="replace")
    if test["exit_code"] in cfg["invalid_exit_codes"]:
        return {"class": "invalid", "reason": "invalid_exit_code", "runs": runs}
    if any(re.search(pattern, output) for pattern in cfg["invalid_output_patterns"]):
        return {"class": "invalid", "reason": "invalid_output", "runs": runs}
    return {"class": "survived" if test["exit_code"] == 0 else "killed", "reason": None, "runs": runs}


def _write(path: Path, data: bytes) -> None:
    # Every write lands in a new wall-clock second, so a cache keyed on a
    # whole-second mtime and the size cannot take one version for another.
    try:
        while int(time.time()) <= _last_write[0]:
            time.sleep(0.05)
    finally:
        path.write_bytes(data)
        _last_write[0] = int(time.time())


def _restore(path: Path, original: bytes, digest: str) -> None:
    if sha256(path.read_bytes()) != digest:
        _write(path, original)
    if sha256(path.read_bytes()) != digest:
        message = f"RESTORE FAILED: {path} does not match its original SHA-256 {digest}. Batch stopped."
        print("!" * 78, message, "Restore the file from git before running anything else.", "!" * 78,
              sep="\n", file=sys.stderr, flush=True)
        raise SystemExit(message)


def _run_mutant(number: int, mutant: e.Mutant, scope: v.Scope, cfg: dict, scopes_dir: Path,
                out_dir: Path, original: bytes, digest: str) -> dict:
    record = {"number": number, "site": asdict(mutant.site), "operator": mutant.operator}
    verdict = v.classify_mutant(scope.language, original.decode("utf-8"), mutant.text)
    if verdict != "changed":
        return {**record, "class": "invalid", "reason": verdict, "exit_code": None, "seconds": 0.0, "log": None}
    path = scope.root / mutant.site.file
    try:
        _write(path, mutant.text.encode("utf-8"))
        result = check(cfg, scope.root, scopes_dir, out_dir, f"m{number:03d}")
    finally:
        _restore(path, original, digest)
    test = result["runs"][-1]
    return {**record, "class": result["class"], "reason": result["reason"], "exit_code": test["exit_code"],
            "seconds": round(sum(r["seconds"] for r in result["runs"]), 2), "log": test["log"],
            "runs": result["runs"]}


def run_batch(scope: v.Scope, cfg: dict, scopes_dir: Path, mutants: list[e.Mutant], out_dir: Path) -> dict:
    """Baseline, every mutant with restoration, baseline again; ``void`` names a broken protocol."""
    out_dir.mkdir(parents=True, exist_ok=True)
    originals = {f: (scope.root / f).read_bytes() for f in scope.files}
    digests = {f: sha256(data) for f, data in originals.items()}
    batch = {"scope_files_sha256": digests, "void": None, "baseline_before": None,
             "mutants": [], "baseline_after": None, "restored": False}
    batch["baseline_before"] = check(cfg, scope.root, scopes_dir, out_dir, "baseline-before")
    if batch["baseline_before"]["class"] != "survived":
        batch["void"] = "baseline_before_failed"
        return batch
    try:
        for number, mutant in enumerate(mutants, 1):
            f = mutant.site.file
            batch["mutants"].append(_run_mutant(number, mutant, scope, cfg, scopes_dir, out_dir,
                                                originals[f], digests[f]))
    finally:
        for f, data in originals.items():
            _restore(scope.root / f, data, digests[f])
    batch["restored"] = True
    batch["baseline_after"] = check(cfg, scope.root, scopes_dir, out_dir, "baseline-after")
    if batch["baseline_after"]["class"] != "survived":
        batch["void"] = "baseline_after_failed"
    return batch


def revision(root: Path) -> str:
    git = shutil.which("git") or "git"
    return subprocess.run([git, "-C", str(root), "rev-parse", "HEAD"], capture_output=True,
                          text=True, check=True).stdout.strip()
