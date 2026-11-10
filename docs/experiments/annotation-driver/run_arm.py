r"""Run one arm batch on one scope (gate Amendment 3b, 3d, 3e).

    uv run --no-sync python run_arm.py --scope NAME --scopes-dir DIR --out-dir DIR
        (--all-sites | --annotations FILE) [--limit N]

``--all-sites`` takes every eligible site through ``engine.select``;
``--annotations`` validates a JSON list of annotations and mutates the
accepted ones. ``--limit`` caps the number of mutants run. Writes
``receipt.json`` and one log per run into the output directory.

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
    args = parser.parse_args()

    scope = v.load_scopes(args.scopes_dir)[args.scope]
    cfg = h.scope_config(args.scope)
    decisions, skipped = None, []
    eligible = e.eligible_sites(scope)
    if args.all_sites:
        # Gate Amendment 4b: the cap applies to mutants, so choose among sites that have an operator.
        sites = e.select([s for s in eligible if e.mutate(scope, s) is not None])
    else:
        annotations = json.loads(args.annotations.read_text(encoding="utf-8"))
        sites, decisions, skipped = annotated_sites(scope, annotations)
    mutants, no_operator = [], []
    for site in sites:
        mutant = e.mutate(scope, site)
        if mutant is None:
            no_operator.append({**asdict(site), "reason": "no_operator"})
        else:
            mutants.append(mutant)
    if args.limit is not None:
        mutants = mutants[: args.limit]

    started = time.monotonic()
    batch = h.run_batch(scope, cfg, args.scopes_dir, mutants, args.out_dir)
    by_class, by_kind = counts(batch["mutants"])
    receipt = {
        "receipt": "annotation-driver-arm-v0",
        "scope": scope.name,
        "language": scope.language,
        "sites_from": "all_sites" if args.all_sites else str(args.annotations),
        "revision": h.revision(scope.root),
        "expected_revision": cfg["revision"],
        "pack_sha256": file_sha(v.tables_dir() / f"{scope.language}.json"),
        "tables_dir": str(v.tables_dir().resolve()),
        "code_sha256": {name: file_sha(HERE / name) for name in CODE},
        "limit": args.limit,
        "eligible_sites": len(eligible),
        "sites": len(sites),
        "mutants_run": len(mutants),
        "counts_by_class": by_class,
        "counts_by_kind": by_kind,
        "sites_without_operator": no_operator,
        "annotation_decisions": decisions,
        "annotations_skipped": skipped,
        "wall_seconds": round(time.monotonic() - started, 1),
        **batch,
    }
    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "receipt.json").write_text(json.dumps(receipt, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(f"{scope.name}: void={batch['void']} {by_class} -> {args.out_dir / 'receipt.json'}")
    return 0 if batch["void"] is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
