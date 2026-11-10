r"""Write the prototype's results for the gate scopes as reference files for the native driver's parity tests.

    uv run --no-sync python native-spike\make_reference.py --scopes-dir DIR --out-dir DIR

For each scope in scopes.json: the eligible sites, every mutant (site, operator, SHA-256 of the mutated text)
and the control cases with the annotations they are made of. Everything comes from the frozen prototype
(engine.py, validator.py, run_controls.py); nothing here decides an outcome. No model, no network.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import sys
from pathlib import Path

PROTOTYPE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROTOTYPE))

import engine as e  # noqa: E402
import run_controls as rc  # noqa: E402
import validator as v  # noqa: E402


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def reference(scope: v.Scope, entry: dict, spec: dict) -> dict:
    sites = e.eligible_sites(scope)
    mutants = [{"site": dataclasses.asdict(site), "operator": m.operator, "sha256": sha(m.text.encode("utf-8"))}
               for site in sites for m in e.mutants(scope, site)]
    controls = [{"label": label, "annotations": annotations, "expected": expected,
                 "got": [rc.outcome(d) for d in v.validate(scope, annotations)]}
                for label, annotations, expected in rc.annotation_cases(scope, spec)]
    original = scope.source(spec["seeds"][0]["file"])
    mutant_controls = [{"label": label, "sha256": sha(mutated.encode("utf-8")), "expected": expected,
                        "got": v.classify_mutant(scope.language, original, mutated)}
                       for label, mutated, expected in rc.mutant_cases(scope, spec)]
    frozen = ["engine.py", "validator.py", "run_controls.py", "controls.json", "scopes.json",
              f"tables/{scope.language}.json"]
    return {
        "reference": "annotation-driver-prototype-reference-v1",
        "scope": scope.name, "language": scope.language, "repository": entry["repository"],
        "revision": entry["revision"], "files": list(scope.files), "test_paths": list(scope.test_paths),
        "prototype_sha256": {name: sha((PROTOTYPE / name).read_bytes()) for name in frozen},
        "scope_files_sha256": {f: sha((scope.root / f).read_bytes()) for f in scope.files},
        "controls_spec": {"file": spec["seeds"][0]["file"], "real_change": spec["real_change"],
                          "comment_text": spec["comment_text"]},
        "sites": [dataclasses.asdict(s) for s in sites],
        "mutants": mutants,
        "controls": controls,
        "mutant_controls": mutant_controls,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    scopes = v.load_scopes(args.scopes_dir)
    entries = {s["name"]: s for s in json.loads((PROTOTYPE / "scopes.json").read_text(encoding="utf-8"))["scopes"]}
    controls = json.loads((PROTOTYPE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    args.out_dir.mkdir(parents=True, exist_ok=True)
    for name, scope in scopes.items():
        result = reference(scope, entries[name], controls[name])
        (args.out_dir / f"{name}.json").write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n",
                                                   encoding="utf-8", newline="\n")
        failed = [c["label"] for c in result["controls"] + result["mutant_controls"] if c["got"] != c["expected"]]
        print(f"{name}: sites {len(result['sites'])}, mutants {len(result['mutants'])}, "
              f"controls {len(result['controls']) + len(result['mutant_controls'])}, not as expected {failed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
