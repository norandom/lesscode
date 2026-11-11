"""Slice B annotation-layer baseline (preregistration R82): how much do a model's site annotations vary?

For each scope and run: one fresh annotation call (annotate.py, frozen guide), then the driver's own validator
(native example `annotation_sites`) says which annotations are accepted and which site each names. The summary
gives validity, the agreement between runs of one arm, and, with --compare, between two arms.

    uv run python slice-b/annotate_baseline.py call --out-dir results/slice-b/annot-opus --scopes-dir DIR --runs 5
    uv run python slice-b/annotate_baseline.py summarise --out-dir results/slice-b/annot-opus --scopes-dir DIR \
        --checker annotation_sites.exe [--compare results/slice-b/annot-mistral]

Research tooling. `call` makes real model calls unless --dry-run is given.
"""

from __future__ import annotations

import argparse
import itertools
import json
import statistics
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROTOTYPE = HERE.parent
SCOPES = json.loads((PROTOTYPE / "scopes.json").read_text(encoding="utf-8"))["scopes"]
KINDS = ("branch_condition", "failure_state", "state_write", "error_order")


def call(args) -> int:
    args.out_dir.mkdir(parents=True, exist_ok=True)
    jobs = [(scope["name"], run) for run in range(1, args.runs + 1) for scope in SCOPES]

    def one(job):
        name, run = job
        command = [sys.executable, str(PROTOTYPE / "annotate.py"), "--scope", name, "--scopes-dir", str(args.scopes_dir),
                   "--guide", str(args.guide), "--out-dir", str(args.out_dir / f"run{run}-{name}"),
                   "--max-output-tokens", str(args.max_output_tokens)]
        if args.endpoint_env:
            command += ["--endpoint-env", str(args.endpoint_env)]
        if args.dry_run:
            command.append("--dry-run")
        return name, run, subprocess.run(command, cwd=PROTOTYPE, capture_output=True, text=True).returncode

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        for name, run, code in pool.map(one, jobs):
            print(f"run{run} {name}: exit {code}")
    return 0


def checked(args, arm: Path, scope: dict, run: int) -> dict | None:
    """The validator's verdicts for one run, or None when the run left no annotations."""
    directory = arm / f"run{run}-{scope['name']}"
    annotations = directory / "annotations.json"
    if not annotations.exists():
        return None
    command = [str(args.checker), "--lang", scope["language"], "--source-root", str(args.scopes_dir / scope["repository"]),
               "--annotations", str(annotations)]
    for file in scope["files"]:
        command += ["--file", file]
    for path in scope["test_paths"]:
        command += ["--test-path", path]
    done = subprocess.run(command, capture_output=True, text=True)
    if done.returncode != 0:
        raise SystemExit(f"{directory.name}: checker failed: {done.stderr.strip()}")
    result = json.loads(done.stdout)
    rows = result["annotations"]
    return {"annotations": len(rows), "accepted": sum(r["accepted"] for r in rows),
            "sites": sorted({r["site"] for r in rows if r["site"]}),
            "rejections": sorted({r["outcome"] for r in rows if not r["accepted"]}),
            "eligible": [s for s in result["eligible"] if s.rsplit(":", 1)[1] in KINDS]}


def jaccard(a: list, b: list) -> float | None:
    union = set(a) | set(b)
    return round(len(set(a) & set(b)) / len(union), 3) if union else None


def arm_summary(args, arm: Path) -> dict:
    out = {}
    for scope in SCOPES:
        runs = {}
        for directory in sorted(arm.glob(f"run*-{scope['name']}")):
            run = int(directory.name.split("-")[0][3:])
            call_record = json.loads((directory / "call.json").read_text(encoding="utf-8"))
            runs[run] = {"stop_reason": call_record.get("stop_reason"), "exit_code": call_record.get("exit_code"),
                         "input_tokens": call_record.get("input_tokens"), "output_tokens": call_record.get("output_tokens"),
                         "checked": checked(args, arm, scope, run)}
        usable = {run: r["checked"] for run, r in runs.items() if r["checked"]}
        pairs = [jaccard(a["sites"], b["sites"]) for a, b in itertools.combinations(usable.values(), 2)]
        pairs = [p for p in pairs if p is not None]
        every = [set(c["sites"]) for c in usable.values()]
        eligible = next(iter(usable.values()))["eligible"] if usable else []
        out[scope["name"]] = {
            "runs": len(runs), "runs_with_annotations": len(usable),
            "refusals": sum(r["stop_reason"] == "refusal" for r in runs.values()),
            "annotations": [c["annotations"] for c in usable.values()], "accepted": [c["accepted"] for c in usable.values()],
            "validity": [round(c["accepted"] / c["annotations"], 3) if c["annotations"] else None for c in usable.values()],
            "rejection_reasons": sorted({reason for c in usable.values() for reason in c["rejections"]}),
            "pairwise_agreement": pairs, "median_agreement": round(statistics.median(pairs), 3) if pairs else None,
            "in_every_run": len(set.intersection(*every)) if every else 0, "in_any_run": len(set.union(*every)) if every else 0,
            "eligible_sites": len(eligible),
            "input_tokens": sum(r["input_tokens"] or 0 for r in runs.values()),
            "output_tokens": sum(r["output_tokens"] or 0 for r in runs.values()),
            "sites_by_run": {run: c["sites"] for run, c in usable.items()},
        }
    return out


def summarise(args) -> int:
    summary = {"receipt": "slice-b-annotation-baseline-v1", "preregistration": "R82", "arm": arm_summary(args, args.out_dir)}
    if args.compare:
        other = arm_summary(args, args.compare)
        cross = {}
        for name, mine in summary["arm"].items():
            pairs = [jaccard(a, b) for a in mine["sites_by_run"].values() for b in other[name]["sites_by_run"].values()]
            pairs = [p for p in pairs if p is not None]
            cross[name] = {"pairs": len(pairs), "median_agreement": round(statistics.median(pairs), 3) if pairs else None,
                           "min": min(pairs) if pairs else None, "max": max(pairs) if pairs else None}
        summary["compared_with"] = {"directory": args.compare.name, "cross_arm": cross}
    (args.out_dir / "summary.json").write_text(json.dumps(summary, indent=1) + "\n", encoding="utf-8", newline="\n")
    for name, row in summary["arm"].items():
        print(f"{name}: runs {row['runs_with_annotations']}/{row['runs']} validity {row['validity']} "
              f"median agreement {row['median_agreement']} core {row['in_every_run']}/{row['in_any_run']} of {row['eligible_sites']}")
    for name, row in summary.get("compared_with", {}).get("cross_arm", {}).items():
        print(f"cross {name}: median {row['median_agreement']} ({row['min']} to {row['max']}, {row['pairs']} pairs)")
    return 0


def ensemble(args) -> int:
    """R84: do rules over three runs select more stably than one run does?

    Every split of the runs into two disjoint groups of three is taken. In each group the majority rule keeps
    the sites at least two runs name, the union rule keeps the sites any run names. Agreement is measured
    between the two groups' selections, and between single runs for comparison.
    """
    arm = arm_summary(args, args.out_dir)
    out = {}
    for name, row in arm.items():
        runs = {run: set(sites) for run, sites in row["sites_by_run"].items()}
        keys = sorted(runs)
        single = [jaccard(sorted(runs[a]), sorted(runs[b])) for a, b in itertools.combinations(keys, 2)]
        majority, union, sizes = [], [], {"single": [len(s) for s in runs.values()], "majority": [], "union": []}
        for group in itertools.combinations(keys, 3):
            rest = tuple(k for k in keys if k not in group)
            if len(rest) != 3 or group > rest:
                continue
            chosen = []
            for members in (group, rest):
                count = {}
                for member in members:
                    for site in runs[member]:
                        count[site] = count.get(site, 0) + 1
                chosen.append(({s for s, n in count.items() if n >= 2}, set(count)))
            majority.append(jaccard(sorted(chosen[0][0]), sorted(chosen[1][0])))
            union.append(jaccard(sorted(chosen[0][1]), sorted(chosen[1][1])))
            sizes["majority"] += [len(chosen[0][0]), len(chosen[1][0])]
            sizes["union"] += [len(chosen[0][1]), len(chosen[1][1])]
        med = lambda values: round(statistics.median(values), 3) if values else None  # noqa: E731
        out[name] = {"runs": len(keys), "splits": len(majority), "eligible_sites": row["eligible_sites"],
                     "agreement": {"single": med(single), "majority": med(majority), "union": med(union)},
                     "agreement_range": {"single": [min(single), max(single)] if single else None,
                                         "majority": [min(majority), max(majority)] if majority else None,
                                         "union": [min(union), max(union)] if union else None},
                     "median_sites": {k: med(v) for k, v in sizes.items()}}
        print(f"{name}: agreement single {out[name]['agreement']['single']} majority {out[name]['agreement']['majority']} "
              f"union {out[name]['agreement']['union']}; sites {out[name]['median_sites']} of {row['eligible_sites']}")
    result = {"receipt": "slice-b-annotation-ensemble-v1", "preregistration": "R84", "scopes": out}
    (args.out_dir / "ensemble.json").write_text(json.dumps(result, indent=1) + "\n", encoding="utf-8", newline="\n")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["call", "summarise", "ensemble"])
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--guide", type=Path, default=PROTOTYPE / "guide-v2.md")
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--max-output-tokens", type=int, default=16000)
    parser.add_argument("--endpoint-env", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--checker", type=Path)
    parser.add_argument("--compare", type=Path)
    args = parser.parse_args()
    return {"call": call, "summarise": summarise, "ensemble": ensemble}[args.action](args)


if __name__ == "__main__":
    raise SystemExit(main())
