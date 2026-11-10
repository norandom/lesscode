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
import math
import os
import re
import shutil
import signal
import subprocess
import sys
import threading
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import engine as e
import validator as v

HERE = Path(__file__).parent
CMD_SPECIAL = set(' \t&|<>^%"')  # characters cmd /c would reinterpret in an argument
_last_write: dict[str, int] = {}  # per file path: the wall-clock second of its last write


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


def expand(text: str, scopes_dir: Path, limit: int | None = None) -> str:
    """Placeholders: `{scopes}`, `{work}` (its parent), `{experiment}`, `{id}` (the scopes directory's name), `{limit}`."""
    for key, path in (("{scopes}", scopes_dir), ("{work}", scopes_dir.parent), ("{experiment}", HERE)):
        text = text.replace(key, str(path.resolve()))
    text = text.replace("{id}", scopes_dir.resolve().name)
    return text.replace("{limit}", str(limit)) if limit is not None else text


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


def scope_env(cfg: dict, scopes_dir: Path) -> dict[str, str]:
    """`base_env` plus the scope's own variables, expanded for `scopes_dir`."""
    return {**base_env(), **{k: expand(val, scopes_dir) for k, val in cfg.get("env", {}).items()}}


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


def check(cfg: dict, root: Path, scopes_dir: Path, out_dir: Path, label: str, limit: int | None = None) -> dict:
    """Optional build, then test, classified as in Amendment 3e.

    `limit` is the time allowed per step (gate Amendment 9); a command that
    contains `{limit}` ends itself at that time and reports it through one of
    `timeout_exit_codes`, and the harness then only waits a little longer.
    """
    limit = limit or cfg["timeout_s"]
    env = scope_env(cfg, scopes_dir)
    runs: list[dict] = []
    for step in ("build", "test"):
        if not cfg.get(step):
            continue
        argv = [expand(a, scopes_dir, limit) for a in cfg[step]]
        grace = 15 if any("{limit}" in a for a in cfg[step]) else 0
        result = run(step, argv, root, env, limit + grace, out_dir / f"{label}-{step}.log")
        runs.append(asdict(result))
        if result.timed_out or result.exit_code in cfg.get("timeout_exit_codes", ()):
            return {"class": "timeout", "reason": f"{step}_timeout", "runs": runs}
        if step == "build" and result.exit_code != 0:
            return {"class": "invalid", "reason": "build_failed", "runs": runs}
    test = runs[-1]
    output = (out_dir / test["log"]).read_text(encoding="utf-8", errors="replace")
    # Running out of memory or disk while building says nothing about the mutant.
    if any(re.search(pattern, output) for pattern in cfg.get("error_output_patterns", ())):
        return {"class": "error", "reason": "resource_failure", "runs": runs}
    if test["exit_code"] in cfg["invalid_exit_codes"]:
        return {"class": "invalid", "reason": "invalid_exit_code", "runs": runs}
    if any(re.search(pattern, output) for pattern in cfg["invalid_output_patterns"]):
        return {"class": "invalid", "reason": "invalid_output", "runs": runs}
    # A failing exit counts as killed only when the test runner's own output shows
    # that it ran; a command that failed to launch (e.g. "Access is denied.") is an
    # error. A zero exit of the real command is itself evidence: quiet runners
    # (Maven -q) print nothing on a pass.
    evidence = cfg.get("runner_evidence_patterns")
    if test["exit_code"] != 0 and evidence and not any(re.search(p, output) for p in evidence):
        return {"class": "error", "reason": "no_runner_output", "runs": runs}
    return {"class": "survived" if test["exit_code"] == 0 else "killed", "reason": None, "runs": runs}


def cache_root(scopes_dir: Path) -> Path:
    """The outcome store of a scopes directory; worker copies (pool.py) pass the main one's."""
    return scopes_dir.parent / "result-cache"


def _cache_file(cfg: dict, cache_dir: Path, scope_name: str, digest: str, mutated: str) -> Path | None:
    """Where the outcome of this exact mutated file under this exact check is kept.

    The outcome of a mutant is a function of the original file, the mutated
    text and the scope's build and test settings, so a second run of the same
    mutant repeats the first (gate Amendment 8). `AD_RESULT_CACHE=off` disables reuse.
    """
    if os.environ.get("AD_RESULT_CACHE", "").lower() == "off":
        return None
    settings = json.dumps({k: cfg.get(k) for k in ("build", "test", "env", "timeout_s", "invalid_exit_codes",
                                                   "invalid_output_patterns")}, sort_keys=True)
    key = sha256("\n".join((digest, sha256(mutated.encode("utf-8")), settings)).encode("utf-8"))
    return cache_dir / scope_name / f"{key}.json"


def put_text(path: Path, text: str) -> None:
    """Write under a temporary name, then replace, so a concurrent reader never sees part of a file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f"{path.name}.{os.getpid()}-{threading.get_ident()}.tmp")
    tmp.write_text(text, encoding="utf-8", newline="\n")
    for attempt in range(20):  # Windows refuses the replace while another process has the target open
        try:
            os.replace(tmp, path)
            return
        except PermissionError:
            if attempt == 19:
                raise
            time.sleep(0.1)


def _write(path: Path, data: bytes) -> None:
    # Every write to one path lands in a new wall-clock second, so a cache keyed
    # on a whole-second mtime and the size cannot take one version for another.
    # Kept per path: concurrent batches write different copies (pool.py).
    key = os.path.normcase(os.path.abspath(path))
    try:
        while int(time.time()) <= _last_write.get(key, 0):
            time.sleep(0.05)
    finally:
        path.write_bytes(data)
        _last_write[key] = int(time.time())


def _restore(path: Path, original: bytes, digest: str) -> None:
    if sha256(path.read_bytes()) != digest:
        _write(path, original)
    if sha256(path.read_bytes()) != digest:
        message = f"RESTORE FAILED: {path} does not match its original SHA-256 {digest}. Batch stopped."
        print("!" * 78, message, "Restore the file from git before running anything else.", "!" * 78,
              sep="\n", file=sys.stderr, flush=True)
        raise SystemExit(message)


def _run_mutant(number: int, mutant: e.Mutant, scope: v.Scope, cfg: dict, scopes_dir: Path,
                out_dir: Path, original: bytes, digest: str, limit: int | None = None,
                cache_dir: Path | None = None, unsaved: list | None = None) -> dict:
    """One mutant, its file restored afterwards.

    An `error` (the runner did not run) is retried once with the mutant still
    in place. The outcome to cache is appended to `unsaved` (when given) instead
    of being written, so the batch writes it only once it is known not to be
    void; an `error` is never cached.
    """
    record = {"number": number, "site": asdict(mutant.site), "operator": mutant.operator}
    verdict = v.classify_mutant(scope.language, original.decode("utf-8"), mutant.text)
    if verdict != "changed":
        return {**record, "class": "invalid", "reason": verdict, "exit_code": None, "seconds": 0.0, "log": None}
    path = scope.root / mutant.site.file
    entry = _cache_file(cfg, cache_dir or cache_root(scopes_dir), scope.name, digest, mutant.text)
    if entry is not None and entry.exists():
        return {**record, **json.loads(entry.read_text(encoding="utf-8")), "seconds": 0.0, "log": None, "cached": True}
    retried = False
    try:
        _write(path, mutant.text.encode("utf-8"))
        result = check(cfg, scope.root, scopes_dir, out_dir, f"m{number:03d}", limit)
        if result["class"] == "error":
            first, retried = result, True
            result = check(cfg, scope.root, scopes_dir, out_dir, f"m{number:03d}-retry", limit)
            result["runs"] = first["runs"] + result["runs"]
    finally:
        _restore(path, original, digest)
    test = result["runs"][-1]
    if entry is not None and result["class"] != "error":
        outcome = (entry, json.dumps({"class": result["class"], "reason": result["reason"],
                                      "exit_code": test["exit_code"], "first_run": str(out_dir)}))
        if unsaved is None:
            put_text(*outcome)
        else:
            unsaved.append(outcome)
    return {**record, "class": result["class"], "reason": result["reason"], "exit_code": test["exit_code"],
            "seconds": round(sum(r["seconds"] for r in result["runs"]), 2), "log": test["log"],
            "runs": result["runs"], **({"retried": True} if retried else {})}


class Batch:
    """One batch in one working copy, in steps; ``record`` is the batch record.

    `start` runs the optional setup and the baseline before and fixes the time
    limit, `mutant` runs one mutant and restores its file, `restore` restores
    every scope file and `finish` restores and runs the baseline after.
    `run_batch` is these steps in order; pool.py runs one per worker copy.
    """

    def __init__(self, scope: v.Scope, cfg: dict, scopes_dir: Path, out_dir: Path,
                 cache_dir: Path | None = None) -> None:
        self.scope, self.cfg, self.scopes_dir, self.out_dir = scope, cfg, scopes_dir, out_dir
        self.cache_dir = cache_dir or cache_root(scopes_dir)
        out_dir.mkdir(parents=True, exist_ok=True)
        self.originals = {f: (scope.root / f).read_bytes() for f in scope.files}
        self.digests = {f: sha256(data) for f, data in self.originals.items()}
        self.record = {"scope_files_sha256": self.digests, "void": None, "baseline_before": None,
                       "mutants": [], "baseline_after": None, "restored": False}
        self.limit: int | None = None
        self.unsaved: list[tuple[Path, str]] = []  # outcomes cached by `finish` unless the batch is void

    def start(self) -> bool:
        """Setup and baseline before; False when the batch is void."""
        cfg, batch, root = self.cfg, self.record, self.scope.root
        if cfg.get("setup"):  # e.g. start a long-lived test container once per batch
            setup = run("setup", [expand(a, self.scopes_dir) for a in cfg["setup"]], root,
                        scope_env(cfg, self.scopes_dir), 600, self.out_dir / "setup.log")
            if setup.timed_out or setup.exit_code != 0:
                batch["void"] = "setup_failed"
                return False
        batch["baseline_before"] = check(cfg, root, self.scopes_dir, self.out_dir, "baseline-before")
        if batch["baseline_before"]["class"] != "survived":
            batch["void"] = "baseline_before_failed"
            return False
        if "timeout_floor_s" in cfg:  # a mutant gets a multiple of what the unmutated run needed
            slowest = max(r["seconds"] for r in batch["baseline_before"]["runs"])
            self.limit = min(cfg["timeout_s"],
                             max(cfg["timeout_floor_s"], math.ceil(cfg.get("timeout_factor", 5) * slowest)))
        batch["limit_s"] = self.limit or cfg["timeout_s"]
        return True

    def mutant(self, number: int, mutant: e.Mutant) -> dict:
        f = mutant.site.file
        return _run_mutant(number, mutant, self.scope, self.cfg, self.scopes_dir, self.out_dir,
                           self.originals[f], self.digests[f], self.limit, self.cache_dir, self.unsaved)

    def restore(self) -> None:
        for f, data in self.originals.items():
            _restore(self.scope.root / f, data, self.digests[f])

    def finish(self) -> dict:
        self.restore()
        batch = self.record
        batch["restored"] = True
        batch["baseline_after"] = check(self.cfg, self.scope.root, self.scopes_dir, self.out_dir, "baseline-after")
        if batch["baseline_after"]["class"] != "survived":
            batch["void"] = "baseline_after_failed"
        if batch["void"] is None:  # results of a void batch are never reused
            for outcome in self.unsaved:
                put_text(*outcome)
        self.unsaved.clear()
        return batch


def run_batch(scope: v.Scope, cfg: dict, scopes_dir: Path, mutants: list[e.Mutant], out_dir: Path) -> dict:
    """Baseline, every mutant with restoration, baseline again; ``void`` names a broken protocol."""
    batch = Batch(scope, cfg, scopes_dir, out_dir)
    if not batch.start():
        return batch.record
    try:
        for number, mutant in enumerate(mutants, 1):
            batch.record["mutants"].append(batch.mutant(number, mutant))
    finally:
        batch.restore()
    return batch.finish()


def revision(root: Path) -> str:
    git = shutil.which("git") or "git"
    return subprocess.run([git, "-C", str(root), "rev-parse", "HEAD"], capture_output=True,
                          text=True, check=True).stdout.strip()
