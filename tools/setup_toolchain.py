"""Explicit network-enabled setup, separate from all evaluation/probe commands."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def fresh_tool_root(path):
    path = Path(os.path.abspath(path))
    for part in (path, *path.parents):
        if part.is_symlink() or part.is_junction():
            raise ValueError("tool root cannot traverse a symlink/reparse point")
    if path.exists():
        raise ValueError("tool root must be a new directory; select a fresh --tool-root")
    path.mkdir(parents=True, exist_ok=False)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool-root", type=Path, default=ROOT / ".tools/analyzers")
    parser.add_argument("--node", type=Path, default=shutil.which("node"))
    parser.add_argument("--npm-cli", type=Path)
    parser.add_argument("--uv", type=Path, default=shutil.which("uv.exe" if os.name == "nt" else "uv"))
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
    if args.uv is None or not args.uv.is_file():
        parser.error("setup requires an explicitly installed uv executable")
    uv = str(args.uv.resolve(strict=True))
    try:
        target = fresh_tool_root(args.tool_root)
    except ValueError as error:
        parser.error(str(error))
    python_env = target / "python"
    node_env = target / "node"
    # uv preserves platform runtime layout without copied macOS executables or
    # ensurepip. sys.executable is the already version-checked absolute runtime.
    subprocess.run([uv, "venv", "--python", os.path.abspath(sys.executable), str(python_env)], check=True)
    python = python_env / ("Scripts/python.exe" if sys.platform == "win32" else "bin/python")
    subprocess.run([uv, "pip", "install", "--python", str(python), "--require-hashes", "--only-binary=:all:", "-r", str(ROOT / "requirements-tools.lock")], check=True)
    subprocess.run([uv, "pip", "check", "--python", str(python)], check=True)
    node_env.mkdir(exist_ok=True)
    for name in ("package.json", "package-lock.json"):
        shutil.copyfile(ROOT / "tools" / name, node_env / name)
    subprocess.run([str(node), str(npm_cli.resolve()), "ci", "--ignore-scripts", "--no-audit", "--no-fund"], cwd=node_env, check=True)
    paths = {"python": str(python), "node": str(node), "jscpd_entry": str(node_env / "node_modules/jscpd/bin/jscpd"), "npm_cli": str(npm_cli.resolve())}
    (target / "entrypoints.json").write_text(json.dumps(paths, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(paths, indent=2))


if __name__ == "__main__":
    main()
