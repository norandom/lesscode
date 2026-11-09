"""E6 frozen scope: the case-report-envelope files, SHA-256 digests, and entry points.

Executing `scope.py [ROOT]` emits the JSON record with byte hashes and canonical line counts.
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
LESSCODE = HERE.parents[3]
FROZEN = LESSCODE / ".runs/e2-facdrone/source"
CASE_JSON = LESSCODE / "docs/experiments/constraint-modeler/facdrone-discovery/run-02/case-report-envelope.json"

FILES = (
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py",
    "packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py",
    "packages/facdrone-kernel/facdrone_kernel/canonical.py",
)

FROZEN_DIGESTS = {
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py": "1c5dd687fbc87a2bdadc98600321e41eff8de5ab020a3adfa3ae98173b36a8a6",
    "packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py": "dccb32a179d1384974e60d2f8abf1919b744de40123ad9d543091ccc8772ac09",
    "packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py": "c69421000dc3e1c1f7ae9f22a7fa7710ff7d6c27072b286ab24dd09fce82d6fa",
    "packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py": "ac6a8dc79f9e1922e210d860c21058ab505c97e0616947196c636e1506af03c7",
    "packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py": "975539b6e446d698bd67c8cb49320eef278f65cee3ce13eb527d69e4b687b0be",
    "packages/facdrone-kernel/facdrone_kernel/canonical.py": "68166e605160fd829487ee953ed0838b70bc174a382b3e92f5838b6a12f1124e",
}


def sha256_of(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def record(root: Path = FROZEN) -> dict:
    file_records = {}
    changed = []
    total_physical = 0
    for rel in FILES:
        target = root / rel
        h = sha256_of(target)
        lines = len(target.read_text(encoding="utf-8").splitlines())
        total_physical += lines
        expected = FROZEN_DIGESTS.get(rel)
        if expected and h != expected:
            changed.append(rel)
        file_records[rel] = {
            "sha256": h,
            "expected_sha256": expected,
            "lines": lines,
        }
    return {
        "root": str(root),
        "files": file_records,
        "changed": changed,
        "total_physical_lines": total_physical,
    }


if __name__ == "__main__":
    target_root = Path(sys.argv[1]) if len(sys.argv) > 1 else FROZEN
    rec = record(target_root)
    print(json.dumps(rec, indent=2))
    if rec["changed"] and target_root == FROZEN:
        sys.exit(1)
