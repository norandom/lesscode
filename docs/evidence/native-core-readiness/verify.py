"""Task 7.3 bounded readmission and byte-identity receipt; research support only."""
from pathlib import Path
import datetime
import hashlib
import json
import platform
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()


def run(name, args, expected=0):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, capture_output=True)
    (OUT / (name + ".log")).write_bytes(result.stdout + result.stderr)
    receipt = {"command": [str(a) for a in args], "exit_code": result.returncode}
    assert result.returncode == expected, receipt
    return receipt


if __name__ == "__main__":
    binary = ROOT / "native/target/debug/lesscode-core.exe"
    source = ROOT / "docs/experiments/native-replay/ats-run/recorded"
    receipts = {"help": run("help", [binary, "--help"])}
    before = {str(p.relative_to(source)): sha(p.read_bytes())
              for side in ("baseline", "candidate") for p in (source / side).rglob("*") if p.is_file()}
    fresh = ROOT.parent / ("lesscode-readiness-" + datetime.datetime.now().strftime("%Y%m%d-%H%M%S"))
    receipts["readmission"] = run("readmission", [binary, "compare", "--frozen", source / "frozen.json",
        "--baseline", source / "baseline", "--candidate", source / "candidate",
        "--evidence", source / "candidate-evidence.json", "--artifact-root", source, "--out", fresh])
    for filename in ("result.json", "report.md"):
        (OUT / ("ats-" + filename)).write_bytes((fresh / filename).read_bytes())
    after = {str(p.relative_to(source)): sha(p.read_bytes())
             for side in ("baseline", "candidate") for p in (source / side).rglob("*") if p.is_file()}
    assert before == after
    expected = {}
    for archive in (ROOT / "docs/experiments/native-self-apply/baseline.zip",
                    ROOT / "docs/experiments/native-self-apply/trial/candidate-source.zip"):
        with zipfile.ZipFile(archive) as zipped:
            for name in zipped.namelist():
                if name.startswith(("src/", "tests/")) and not name.endswith("/"):
                    expected[name] = sha(zipped.read(name))
    actual = {str(p.relative_to(ROOT)).replace("\\", "/"): sha(p.read_bytes())
              for folder in ("src", "tests") for p in (ROOT / folder).rglob("*")
              if p.is_file() and "__pycache__" not in p.parts and p.suffix != ".pyc"}
    generated_metadata = {p: actual.pop(p) for p in list(actual) if ".egg-info/" in p}
    deltas = {p: {"expected": expected.get(p), "actual": actual.get(p)}
              for p in expected.keys() | actual.keys() if expected.get(p) != actual.get(p)}
    assert not deltas, deltas
    footprint = {}
    for category, paths in {
        "rust_src_including_inline_tests": sorted((ROOT / "native/src").glob("*.rs")),
        "rust_integration_tests": sorted((ROOT / "native/tests").glob("*.rs")),
        "readiness_research_support": [Path(__file__).resolve()],
    }.items():
        footprint[category] = {str(p.relative_to(ROOT)).replace("\\", "/"): len(p.read_bytes().splitlines()) for p in paths}
    record = {"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "platform": platform.platform(),
        "native_binary_sha256": sha(binary.read_bytes()), "commands": receipts,
        "specimen_before": before, "specimen_after": after, "python_src_tests_files": len(expected),
        "python_src_tests_deltas": deltas, "python_src_tests_sha256": actual,
        "excluded_generated_egg_info": generated_metadata,
        "physical_lines_by_category": footprint,
        "limitations": ["Readmission reuses bound finite behavioral evidence; does not rerun ATS checks.",
            "Rust src includes inline tests; this is not a production-only Rust count.",
            "Full Python regression was executed on guarded identical isolated candidate; no new live full-suite run."]}
    (OUT / "receipt.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print("Verified ATS readmission, unchanged specimens and", len(expected), "matching live Python source/test files")
