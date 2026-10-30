"""Explicit network-enabled setup, separate from all evaluation/probe commands."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import venv


ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool-root", type=Path, default=ROOT / ".tools/analyzers")
    parser.add_argument("--node", type=Path, default=shutil.which("node"))
    parser.add_argument("--npm-cli", type=Path)
    args = parser.parse_args()
    if sys.version_info[:3] != (3, 12, 10):
        parser.error("setup requires CPython 3.12.10")
    if args.node is None or not args.node.is_file():
        parser.error("supply --node with the Node 24.22.1 executable")
    node = args.node.resolve()
    if subprocess.check_output([str(node), "--version"], text=True).strip() != "v24.22.1":
        parser.error("setup requires Node 24.22.1")
    npm_cli = args.npm_cli or node.parent / "node_modules/npm/bin/npm-cli.js"
    if not npm_cli.is_file():
        parser.error("supply --npm-cli with npm/bin/npm-cli.js (needed on some POSIX layouts)")
    target = args.tool_root.resolve()
    python_env = target / "python"
    node_env = target / "node"
    target.mkdir(parents=True, exist_ok=True)
    venv.EnvBuilder(with_pip=True).create(python_env)
    python = python_env / ("Scripts/python.exe" if sys.platform == "win32" else "bin/python")
    subprocess.run([str(python), "-m", "pip", "install", "--require-hashes", "--only-binary=:all:", "-r", str(ROOT / "requirements-tools.lock")], check=True)
    subprocess.run([str(python), "-m", "pip", "check"], check=True)
    node_env.mkdir(exist_ok=True)
    for name in ("package.json", "package-lock.json"):
        shutil.copyfile(ROOT / "tools" / name, node_env / name)
    subprocess.run([str(node), str(npm_cli.resolve()), "ci", "--ignore-scripts", "--no-audit", "--no-fund"], cwd=node_env, check=True)
    paths = {"python": str(python), "node": str(node), "jscpd_entry": str(node_env / "node_modules/jscpd/bin/jscpd"), "npm_cli": str(npm_cli.resolve())}
    (target / "entrypoints.json").write_text(json.dumps(paths, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(paths, indent=2))


if __name__ == "__main__":
    main()
