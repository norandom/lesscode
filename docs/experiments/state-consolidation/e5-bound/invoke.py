"""Invoke the selected E5 specimen function on the cases supplied by Rust."""

import hashlib
import importlib.util
import json
import sys
from datetime import datetime, timedelta
from pathlib import Path

root, source, inputs, output = map(Path, sys.argv[1:])
kernel_root = root / "packages" / "facdrone-kernel"
sys.path.insert(0, str(kernel_root))
from facdrone_kernel.domain import signal_lifecycle as kernel

spec = importlib.util.spec_from_file_location("e5_subject", source)
subject = importlib.util.module_from_spec(spec)
spec.loader.exec_module(subject)
reports = []
for case in json.loads(inputs.read_text(encoding="utf-8")):
    moment = datetime.fromisoformat(case["now"]), timedelta(days=case["days"])
    for row in case["rows"]:
        row["known_at"] = datetime.fromisoformat(row["known_at"])
    standing = kernel.UnrecordedStanding(**case["standing"])
    value = subject.signal_standing(standing, None, case["rows"], None, moment)
    reports.append({"id": case["id"], "value": value})


def identity(path):
    path = Path(path).resolve(strict=True)
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


receipt = {
    "source": identity(subject.signal_standing.__code__.co_filename),
    "kernel": identity(kernel.__file__),
    "python": sys.version,
    "interpreter": identity(sys._base_executable),
    "runtime": identity(Path(sys.base_prefix) / "python313.dll"),
    "reports": reports,
}
output.write_text(json.dumps(receipt, allow_nan=False), encoding="utf-8")
