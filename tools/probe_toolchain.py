"""Run real installation probes without downloading or installing anything.

This is setup verification, not the future production metric adapter.
"""

import argparse
import json
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
SAMPLE = """def classify(value):
    result = 0
    if value > 0:
        result += value
    for item in range(value):
        result += item
    result += 1
    return result
"""


def run(argv, **kwargs):
    return subprocess.check_output(argv, text=True, timeout=60, **kwargs).strip()


def probe(tool_root):
    paths = json.loads((tool_root / "entrypoints.json").read_text(encoding="utf-8"))
    python, node, entry = (str(Path(paths[key]).resolve(strict=True)) for key in ("python", "node", "jscpd_entry"))
    python_version = run([python, "-c", "import platform; print(platform.python_version())"])
    node_version = run([node, "--version"])
    version = run([node, entry, "--version"])
    if (python_version, node_version, version) != ("3.12.10", "v24.22.1", "4.0.5"):
        raise ValueError("installed runtime/analyzer version differs from pinned contract")
    with tempfile.TemporaryDirectory(prefix="lesscode-tool-probe-") as temporary:
        workspace = Path(temporary)
        source = workspace / "input"
        source.mkdir()
        for name in ("one.py", "two.py"):
            (source / name).write_text(SAMPLE, encoding="utf-8", newline="\n")
        worker = """import importlib.metadata as m, json, lizard, sys
data = lizard.analyze_file(sys.argv[1])
print(json.dumps({'version':m.version('lizard'), 'entrypoint':lizard.__file__, 'nloc':data.nloc, 'functions':[{'name':f.name, 'ccn':f.cyclomatic_complexity, 'nloc':f.nloc, 'tokens':f.token_count, 'start':f.start_line, 'end':f.end_line} for f in data.function_list]}))
"""
        complexity = json.loads(run([python, "-c", worker, str(source / "one.py")]))
        expected = [{"name": "classify", "ccn": 3, "nloc": 8, "tokens": 32, "start": 1, "end": 8}]
        if complexity["version"] != "1.17.31" or complexity["functions"] != expected or complexity["nloc"] != 8:
            raise ValueError(f"Lizard probe contract mismatch: {complexity}")
        output = workspace / "output"
        config = workspace / "jscpd.json"
        config.write_text("{}", encoding="utf-8")
        # The small installation probe uses 20 tokens; production profiles use 50.
        argv = [node, entry, source.as_posix(), "--config", config.as_posix(), "--format", "python", "--mode", "weak", "--min-lines", "5", "--min-tokens", "20", "--max-lines", "1000", "--max-size", "1mb", "--reporters", "json", "--output", output.as_posix(), "--silent"]
        stdout = run(argv, cwd=workspace)
        # New temporary directory: a successful exit without a new report fails here.
        report = json.loads((output / "jscpd-report.json").read_text(encoding="utf-8"))
        total = report["statistics"]["total"]
        if (total["tokens"], total["duplicatedTokens"], total["sources"], total["clones"]) != (124, 62, 2, 1):
            raise ValueError(f"jscpd probe did not detect the known duplication: {total}")
        scanned = report["statistics"]["formats"]["python"]["sources"]
        if {Path(path).name for path in scanned} != {"one.py", "two.py"}:
            raise ValueError("jscpd probe omitted expected source files")
        if len(report["duplicates"]) != 1 or report["duplicates"][0]["tokens"] != 0:
            raise ValueError("jscpd per-clone contract differs from 4.0.5")
        return {"probe_format_version": 1, "scope": ["one.py", "two.py"], "entrypoints": paths, "python": python_version, "node": node_version, "lizard": complexity, "jscpd": {"version": version, "argv": argv, "stdout": stdout, "statistics": report["statistics"], "duplicates": report["duplicates"]}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool-root", type=Path, default=ROOT / ".tools/analyzers")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    evidence = json.dumps(probe(args.tool_root.resolve()), indent=2) + "\n"
    if args.output:
        args.output.write_text(evidence, encoding="utf-8")
    print(evidence)


if __name__ == "__main__":
    main()
