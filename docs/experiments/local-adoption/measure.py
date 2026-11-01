"""Read-only research baseline; Lizard metrics, not the native acceptance engine."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from importlib.metadata import version

import lizard


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args]).decode().strip()


def category(path, manifest):
    parts = Path(path).parts
    name = Path(path).name
    if any(p in {"fixtures", "fixture", "__fixtures__"} for p in parts):
        return "fixtures"
    if any(p in {"tests", "test", "__tests__"} for p in parts) or name.startswith("test_") or ".test." in name or ".spec." in name:
        return "tests"
    if any(path == prefix or path.startswith(prefix + "/") for prefix in manifest["runtime_roots"]):
        if "/scripts/" not in path:
            return "production"
    return "support"


def measure(manifest):
    root = Path(manifest["root"]).resolve()
    files = git(root, "ls-files", "-z").split("\0")
    result = {"revision": git(root, "rev-parse", "HEAD"), "root": str(root),
              "lizard": version("lizard"), "files": [], "totals": {},
              "logical_statements": None, "native_acceptance": "not run"}
    for relative in sorted(files):
        path = root / relative
        if path.suffix not in manifest["extensions"]:
            continue
        if any(part in {"dist", "node_modules", "target", ".dagger"} for part in Path(relative).parts):
            continue
        if path.is_symlink():
            raise ValueError(f"Symlink excluded explicitly before measurement: {relative}")
        raw = path.read_bytes()
        if relative in manifest.get("inline_test_ranges", {}):
            if digest(raw) != manifest["inline_test_source_sha256"][relative]:
                raise ValueError(f"Re-review inline test ranges after source change: {relative}")
        lines = raw.decode("utf-8-sig").splitlines(keepends=True)
        default = category(relative, manifest)
        labels = [default] * len(lines)
        for start, end in manifest.get("inline_test_ranges", {}).get(relative, []):
            end = len(lines) if end is None else end
            if not 1 <= start <= end <= len(lines):
                raise ValueError(f"Invalid reviewed line range: {relative}:{start}-{end}")
            labels[start - 1:end] = ["tests"] * (end - start + 1)
        entries = []
        for label in sorted(set(labels)):
            text = "".join(line if assigned == label else "\n" for line, assigned in zip(lines, labels))
            # The admitted Rust lifetime probe fails Lizard's routine parser.
            # Keep raw line counts; use the separate RCA measurements for Rust.
            analysis = None if path.suffix == ".rs" else lizard.analyze_file.analyze_source_code(relative, text)
            functions = [{"name": f.long_name, "start": f.start_line, "end": f.end_line,
                          "nloc": f.nloc, "ccn": f.cyclomatic_complexity}
                         for f in analysis.function_list] if analysis else None
            entry = {"category": label, "physical_lines": labels.count(label),
                     "nloc": analysis.nloc if analysis else None, "functions": functions}
            entries.append(entry)
            total = result["totals"].setdefault(label, {"physical_lines": 0, "nloc": 0,
                "functions": 0, "ccn_sum": 0, "max_ccn": 0})
            total["physical_lines"] += entry["physical_lines"]
            if functions is None or total["nloc"] is None:
                for field in ("nloc", "functions", "ccn_sum", "max_ccn"):
                    total[field] = None
            else:
                total["nloc"] += entry["nloc"]
                total["functions"] += len(functions)
                total["ccn_sum"] += sum(f["ccn"] for f in functions)
                total["max_ccn"] = max(total["max_ccn"], max((f["ccn"] for f in functions), default=0))
        if digest(path.read_bytes()) != digest(raw):
            raise ValueError(f"Source changed during measurement: {relative}")
        result["files"].append({"path": relative, "sha256": digest(raw), "parts": entries})
    identity = [(f["path"], f["sha256"]) for f in result["files"]]
    result["source_identity"] = digest(json.dumps(identity, separators=(",", ":")).encode())
    return result


def copy_production(manifest, result, destination):
    """Stage only the measured production segments for an external clone scan."""
    destination.mkdir(parents=True, exist_ok=False)
    for entry in result["files"]:
        if not any(p["category"] == "production" for p in entry["parts"]):
            continue
        raw = (Path(manifest["root"]) / entry["path"]).read_bytes()
        if digest(raw) != entry["sha256"]:
            raise ValueError(f"Source changed before clone staging: {entry['path']}")
        lines = raw.decode("utf-8-sig").splitlines(keepends=True)
        for start, end in manifest.get("inline_test_ranges", {}).get(entry["path"], []):
            end = len(lines) if end is None else end
            lines[start - 1:end] = ["\n"] * (end - start + 1)
        target = destination / entry["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("".join(lines), encoding="utf-8", newline="\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--production-copy", type=Path)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8-sig"))
    result = measure(manifest)
    if args.production_copy:
        copy_production(manifest, result, args.production_copy)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps(result["totals"], indent=2))
