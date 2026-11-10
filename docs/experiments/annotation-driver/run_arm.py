r"""Run one arm batch on one scope (gate Amendment 3b, 3d, 3e).

    uv run --no-sync python run_arm.py --scope NAME --scopes-dir DIR --out-dir DIR
        (--all-sites | --annotations FILE) [--limit N] [--workers N [--workers-dir DIR]]

``--all-sites`` builds every mutant of every eligible site and runs all of
them when there are at most 200, else 200 chosen by ``engine.select_mutants``
(gate Amendment 10d); ``--annotations`` validates a JSON list of annotations
and runs every mutant of each accepted one's site. The receipt records
``mutants_available`` and ``selection_made``; each mutant record carries its
operator label, so (site, operator) identifies a mutant. ``--limit`` caps the
number of mutants run. Writes
``receipt.json`` and one log per run into the output directory.
``--workers N`` (N > 1) runs the mutants concurrently in N worker copies
under ``--workers-dir`` (default ``<scopes-dir>/../workers``; see pool.py).

Exit code 0 when the batch protocol held, whatever the mutants' classes;
1 when the batch is void.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import time
from collections import Counter
from dataclasses import asdict
from pathlib import Path

import engine as e
import harness as h
import validator as v

HERE = Path(__file__).parent
CODE = ("engine.py", "harness.py", "validator.py", "run_arm.py", "scopes.json")
MUTANT_CAP = 200  # gate Amendment 10d


def file_sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def annotated_sites(scope: v.Scope, annotations: list[dict]) -> tuple[list[e.Site], list[dict], list[dict]]:
    """Accepted annotations as normalised sites, plus the validator decisions and skips."""
    decisions = v.validate(scope, annotations)
    sites: list[e.Site] = []
    skipped = []
    for decision in decisions:
        if not decision["accepted"]:
            continue
        annotation = decision["annotation"]
        site = e.site_from_annotation(scope, annotation)
        if site is None:
            reason = "no_operator_kind" if annotation["kind"] not in e.OPERATOR_KINDS else "excluded_kind"
            skipped.append({"annotation": annotation, "reason": reason})
        elif site in sites:
            skipped.append({"annotation": annotation, "reason": "same_site"})
        else:
            sites.append(site)
    return sites, decisions, skipped


def counts(records: list[dict]) -> tuple[dict, dict]:
    by_kind: dict[str, Counter] = {}
    for r in records:
        by_kind.setdefault(r["site"]["kind"], Counter())[r["class"]] += 1
    return dict(Counter(r["class"] for r in records)), {k: dict(c) for k, c in sorted(by_kind.items())}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scope", required=True)
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--all-sites", action="store_true")
    group.add_argument("--annotations", type=Path)
    parser.add_argument("--limit", type=int)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--workers-dir", type=Path)
    args = parser.parse_args()
    if args.workers < 1:
        parser.error("--workers must be at least 1")

    scope = v.load_scopes(args.scopes_dir)[args.scope]
    cfg = h.scope_config(args.scope)
    decisions, skipped = None, []
    eligible = e.eligible_sites(scope)
    mutants, no_operator = [], []
    if args.all_sites:
        # Gate Amendment 10d: every mutant of every eligible site, or MUTANT_CAP of them chosen
        # by the seeded, kind-balanced selection. Sites without an operator are not drawn (4b).
        every = [m for site in eligible for m in e.mutants(scope, site)]
        selection_made = len(every) > MUTANT_CAP
        mutants = e.select_mutants(every, MUTANT_CAP) if selection_made else every
        sites = list(dict.fromkeys(m.site for m in mutants))
        available = len(every)
    else:
        annotations = json.loads(args.annotations.read_text(encoding="utf-8"))
        sites, decisions, skipped = annotated_sites(scope, annotations)
        for site in sites:
            found = e.mutants(scope, site)
            if not found:
                no_operator.append({**asdict(site), "reason": "no_operator"})
            mutants += found
        selection_made, available = False, len(mutants)
    if args.limit is not None:
        mutants = mutants[: args.limit]

    started = time.monotonic()
    code = CODE
    args.workers = min(args.workers, cfg.get("max_workers", args.workers))  # e.g. builds that do not fit in memory side by side
    if args.workers > 1:
        import pool

        code = (*CODE, "pool.py")
        workers_dir = args.workers_dir or args.scopes_dir.parent / "workers"
        batch = pool.run_pool(args.scope, args.scopes_dir, workers_dir, args.workers, mutants, args.out_dir)
    else:
        batch = h.run_batch(scope, cfg, args.scopes_dir, mutants, args.out_dir)
    by_class, by_kind = counts(batch["mutants"])
    errors = [m["number"] for m in batch["mutants"] if m["class"] == "error"]
    receipt = {
        "receipt": "annotation-driver-arm-v0",
        "scope": scope.name,
        "language": scope.language,
        "sites_from": "all_sites" if args.all_sites else str(args.annotations),
        "revision": h.revision(scope.root),
        "expected_revision": cfg["revision"],
        "pack_sha256": file_sha(v.tables_dir() / f"{scope.language}.json"),
        "tables_dir": str(v.tables_dir().resolve()),
        "code_sha256": {name: file_sha(HERE / name) for name in code},
        "limit": args.limit,
        "eligible_sites": len(eligible),
        "sites": len(sites),
        "mutants_available": available,
        "selection_made": selection_made,
        "mutants_run": len(mutants),
        "counts_by_class": by_class,
        "counts_by_kind": by_kind,
        "error_mutants": errors,
        "sites_without_operator": no_operator,
        "annotation_decisions": decisions,
        "annotations_skipped": skipped,
        "wall_seconds": round(time.monotonic() - started, 1),
        **batch,
    }
    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "receipt.json").write_text(json.dumps(receipt, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(f"{scope.name}: void={batch['void']} {by_class} -> {args.out_dir / 'receipt.json'}")
    if errors:  # not void, but these mutants have no outcome
        print(f"INCOMPLETE: {len(errors)} mutant(s) are `error` (the test runner did not run, also on retry): "
              f"numbers {errors}")
    return 0 if batch["void"] is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
