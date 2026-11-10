r"""Run one batch over N worker copies of the scopes directory at once.

A worker is a replica of the scopes directory, ``<workers_dir>/wK/<repository>``,
so for the harness it is just another ``scopes_dir``: every per-worker path,
container and target directory derives from it through the ``{scopes}`` and
``{id}`` placeholders of ``scopes.json``. Each worker runs the setup and the
baseline before, then takes mutants from one shared queue, then restores its
files and runs the baseline after (``harness.Batch``). A worker whose baseline
fails is void and takes no more mutants; the mutants it ran go back on the queue
for the others. The pool is void only when a mutant could not be run by any
worker. Outcomes are shared through the one result cache of the main scopes
directory, so a pool run and a serial run reuse each other's outcomes.

    uv run --no-sync python run_arm.py --scope NAME --scopes-dir DIR --out-dir DIR --all-sites --workers 6
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import threading
import time
from collections import deque
from pathlib import Path

import engine as e
import harness as h
import validator as v

GIT = shutil.which("git") or "git"
SETUP_TIMEOUT_S = 1800  # per worker_setup command
WARM_UP_LIMIT_S = 3600  # the first build in a fresh copy (e.g. a cold Rust target directory)


def _git(*args: str) -> str:
    return subprocess.run([GIT, *args], capture_output=True, text=True, check=True).stdout.strip()


def _expected_digests(cfg: dict, scopes_dir: Path) -> dict[str, str]:
    """SHA-256 of each scope file as checked out at the pinned revision.

    Taken from the main clone when it is at that revision and the file is
    unmodified, else from the pristine copy in ``scopes-ro`` (a batch running
    on the main clone keeps the file mutated while it runs).
    """
    main = scopes_dir / cfg["repository"]
    pristine = scopes_dir.parent / "scopes-ro" / cfg["repository"]
    at_revision = _git("-C", str(main), "rev-parse", "HEAD") == cfg["revision"]
    digests = {}
    for f in cfg["files"]:
        clean = at_revision and not _git("-C", str(main), "status", "--porcelain", "--", f)
        source = main if clean else pristine
        if not (source / f).is_file():
            raise RuntimeError(f"no unmodified copy of {f}: {main} is modified and {pristine} is missing")
        digests[f] = h.sha256((source / f).read_bytes())
    return digests


def _clone(main: Path, dest: Path, revision: str) -> None:
    """``git clone --local --no-hardlinks`` of the main clone, checked out at ``revision``.

    The copy keeps the main clone's line-ending setting, so its working files
    have the same bytes. It is made under a temporary name and renamed when
    complete, so an interrupted clone is never taken for a worker copy.
    """
    partial = dest.with_name(dest.name + ".partial")
    if partial.exists():
        raise RuntimeError(f"{partial} is left from an interrupted clone; remove it and run again")
    autocrlf = subprocess.run([GIT, "-C", str(main), "config", "core.autocrlf"], capture_output=True,
                              text=True).stdout.strip()
    options = ["-c", f"core.autocrlf={autocrlf}"] if autocrlf else []
    _git("clone", "--quiet", "--local", "--no-hardlinks", "--no-checkout", *options, str(main), str(partial))
    _git("-C", str(partial), "checkout", "--quiet", "--detach", revision)
    os.replace(partial, dest)


def _prepare_one(cfg: dict, scopes_dir: Path, wdir: Path, expected: dict[str, str]) -> dict:
    repo = cfg["repository"]
    root = wdir / repo
    steps: list[dict] = []
    started = time.monotonic()
    if not root.exists():
        wdir.mkdir(parents=True, exist_ok=True)
        t = time.monotonic()
        _clone(scopes_dir / repo, root, cfg["revision"])
        steps.append({"step": "clone", "seconds": round(time.monotonic() - t, 2)})
    if h.revision(root) != cfg["revision"]:
        raise RuntimeError(f"{root} is not at the pinned revision {cfg['revision']}")
    for f, digest in expected.items():
        if h.sha256((root / f).read_bytes()) != digest:
            raise RuntimeError(f"{root / f} differs from the committed file (SHA-256 {digest})")
    marker = wdir / f".ready-{repo}.json"
    if not marker.exists():
        logs = wdir / f".prepare-{repo}"
        logs.mkdir(parents=True, exist_ok=True)
        env = h.scope_env(cfg, wdir)
        for i, argv in enumerate(cfg.get("worker_setup", []), 1):
            result = h.run("worker_setup", [h.expand(a, wdir) for a in argv], root, env, SETUP_TIMEOUT_S,
                           logs / f"worker-setup-{i}.log")
            steps.append({"step": "worker_setup", "command": argv, "exit_code": result.exit_code,
                          "seconds": result.seconds})
            if result.timed_out or result.exit_code != 0:
                return {"worker": wdir.name, "ready": False, "steps": steps, "reason": "worker_setup_failed"}
        if cfg.get("setup"):
            result = h.run("setup", [h.expand(a, wdir) for a in cfg["setup"]], root, env, 600, logs / "setup.log")
            steps.append({"step": "setup", "exit_code": result.exit_code, "seconds": result.seconds})
            if result.timed_out or result.exit_code != 0:
                return {"worker": wdir.name, "ready": False, "steps": steps, "reason": "setup_failed"}
        # One unmutated build and test, so the first batch's baseline (and with it
        # the scaled time limit) is not measured on a cold copy.
        t = time.monotonic()
        warm = h.check(cfg, root, wdir, logs, "warm-up", WARM_UP_LIMIT_S)
        steps.append({"step": "warm-up", "class": warm["class"], "seconds": round(time.monotonic() - t, 2)})
        if warm["class"] != "survived":
            return {"worker": wdir.name, "ready": False, "steps": steps, "reason": "warm_up_failed"}
        h.put_text(marker, json.dumps({"revision": cfg["revision"], "worker_setup": cfg.get("worker_setup", []),
                                       "steps": steps}, indent=1) + "\n")
    return {"worker": wdir.name, "ready": True, "steps": steps, "seconds": round(time.monotonic() - started, 2)}


def prepare_workers(scope_name: str, scopes_dir: Path, workers_dir: Path, n: int,
                    report: list | None = None) -> list[Path]:
    """Create or check ``workers_dir/wK/<repository>`` for K = 1..n; the ready worker directories.

    Idempotent: an existing copy is only checked (revision and scope-file
    hashes); ``worker_setup`` and the warm-up run once per copy, recorded by a
    marker file next to it. Workers are prepared concurrently; one that cannot
    be set up is left out and named in ``report``.
    """
    cfg = h.scope_config(scope_name)
    repo = cfg["repository"]
    if repo.startswith(".") or "/" in repo or "\\" in repo:
        raise ValueError(f"not a repository name: {repo}")  # dot directories (e.g. .target-mmhmem) are not repositories
    expected = _expected_digests(cfg, scopes_dir)
    dirs = [workers_dir / f"w{k}" for k in range(1, n + 1)]
    results: dict[str, dict] = {}

    def work(wdir: Path) -> None:
        try:
            results[wdir.name] = _prepare_one(cfg, scopes_dir, wdir, expected)
        except Exception as exc:  # reported; the worker is left out
            results[wdir.name] = {"worker": wdir.name, "ready": False, "reason": f"{type(exc).__name__}: {exc}"}

    threads = [threading.Thread(target=work, args=(d,), name=d.name) for d in dirs]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    if report is not None:
        report.extend(results[d.name] for d in dirs)
    return [d for d in dirs if results[d.name]["ready"]]


def run_pool(scope_name: str, scopes_dir: Path, workers_dir: Path, n: int, mutants: list[e.Mutant],
             out_dir: Path) -> dict:
    """Prepare the workers and run ``mutants`` over them; a batch record merged across workers.

    Mutants keep their numbers and order (``harness.run_batch`` numbering);
    each carries the worker that ran it. Logs are in ``out_dir/<worker>``.
    """
    cfg = h.scope_config(scope_name)
    out_dir.mkdir(parents=True, exist_ok=True)
    cache_dir = h.cache_root(scopes_dir)
    started = time.monotonic()
    prepared: list[dict] = []
    live = prepare_workers(scope_name, scopes_dir, workers_dir, n, prepared)
    prepare_seconds = round(time.monotonic() - started, 1)

    pending = deque(enumerate(mutants, 1))
    results: dict[int, dict] = {}
    workers = {d.name: {"worker": d.name, "scopes_dir": str(d), "void": None, "segments": []} for d in live}
    lock, stop, errors = threading.Lock(), threading.Event(), []
    requeued: list[int] = []
    rounds = 0
    while pending and live and not errors:
        rounds += 1
        segments: dict[str, dict] = {}

        def work(wdir: Path) -> None:
            scope = v.load_scopes(wdir)[scope_name]
            ran: list[int] = []
            batch = None
            try:
                batch = h.Batch(scope, cfg, wdir, out_dir / (wdir.name if rounds == 1 else f"{wdir.name}-r{rounds}"),
                                cache_dir)
                if batch.start():
                    try:
                        while not stop.is_set():
                            with lock:
                                if not pending:
                                    break
                                number, mutant = pending.popleft()
                            ran.append(number)
                            record = batch.mutant(number, mutant)
                            with lock:
                                results[number] = {**record, "worker": wdir.name}
                    finally:
                        batch.restore()
                    batch.finish()
            except BaseException as exc:  # a failed restore ends the whole pool, as it ends a serial batch
                with lock:
                    errors.append(exc)
                stop.set()
            record = dict(batch.record) if batch is not None else {"void": "not_started"}
            record.pop("mutants", None)
            segments[wdir.name] = {**record, "round": rounds, "mutants": ran}

        threads = [threading.Thread(target=work, args=(d,), name=d.name) for d in live]
        for t in threads:
            t.start()
        try:
            for t in threads:
                while t.is_alive():
                    t.join(0.5)
        except KeyboardInterrupt:
            stop.set()
            for t in threads:
                t.join()
            raise
        for d in list(live):
            segment = segments[d.name]
            workers[d.name]["segments"].append(segment)
            if segment["void"] is not None:
                workers[d.name]["void"] = segment["void"]
                live.remove(d)
                back = [m for m in segment["mutants"] if m in results]
                for m in back:
                    del results[m]
                requeued.extend(back)
                pending.extend((m, mutants[m - 1]) for m in back)
        pending = deque(sorted(pending, key=lambda item: item[0]))
    if errors:
        raise errors[0]

    def last(key: str) -> dict:
        return {w: info["segments"][-1].get(key) for w, info in workers.items() if info["segments"]}

    started_segments = [s for info in workers.values() for s in info["segments"] if s.get("baseline_before")]
    digests = next((s["scope_files_sha256"] for info in workers.values() for s in info["segments"]
                    if "scope_files_sha256" in s), {})
    not_run = sorted(m for m, _ in pending)
    return {
        "scope_files_sha256": digests,
        "void": "mutants_not_run" if not_run else None,
        "baseline_before": last("baseline_before"),
        "mutants": [results[m] for m in sorted(results)],
        "baseline_after": last("baseline_after"),
        "restored": bool(started_segments) and all(s["restored"] for s in started_segments
                                                   if s["void"] != "baseline_before_failed"),
        "limit_s": last("limit_s"),
        "workers": n,
        "pool": {
            "workers_dir": str(workers_dir),
            "cache_dir": str(cache_dir),
            "prepare_seconds": prepare_seconds,
            "prepare": prepared,
            "ready_workers": [w for w in workers],
            "rounds": rounds,
            "requeued": requeued,
            "not_run": not_run,
            "worker_batches": list(workers.values()),
        },
    }
