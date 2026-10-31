# Install the baseline package

Use a checkout of this repository, uv, and CPython 3.12.10. Validation used uv
0.13.5. The duplication analyzer also requires Node 24.22.1 with npm;
[analyzer setup](analyzer-setup.md)
provisions the analysis packages separately. The package's help and version
commands need neither Node nor an LLM service.

Installation can download the interpreter and the declared setuptools build
dependency. Evaluation does not install dependencies. Use the environment's
explicit executable paths below; no shell activation or system Python alias is
needed. On Windows use `uv.exe` directly, including when a shell function named
`uv` exists. Do not use Windows Store Python.

## Windows PowerShell

From a fresh checkout's root:

```powershell
uv.exe python install 3.12.10
uv.exe venv --managed-python --python 3.12.10 .venv
uv.exe pip install --python .venv/Scripts/python.exe .
.\.venv\Scripts\lesscode-baseline.exe --help
.\.venv\Scripts\lesscode-baseline.exe --version
node --version
```

The Node output must be `v24.22.1` before analyzer setup. The package version is
`0.1.0`. Choose a new environment path if `.venv` is already used for something
else, and substitute it consistently in subsequent commands.

## Linux and macOS

```sh
uv python install 3.12.10
uv venv --managed-python --python 3.12.10 .venv
uv pip install --python .venv/bin/python .
.venv/bin/lesscode-baseline --help
.venv/bin/lesscode-baseline --version
node --version
```

Use the same pinned versions on each host. Native Windows, Linux and macOS
validation is recorded in the [CI guide](ci-validation.md). WSL is optional for
Windows development.

## Build a wheel or work on source

On Windows:

```powershell
uv.exe build --python .venv/Scripts/python.exe --wheel --out-dir dist
$env:PYTHONPATH = (Resolve-Path src).Path
uv.exe run --no-project --no-python-downloads --python .venv/Scripts/python.exe python -m unittest discover -v
```

On Linux or macOS:

```sh
uv build --python .venv/bin/python --wheel --out-dir dist
PYTHONPATH="$PWD/src" uv run --no-project --no-python-downloads --python .venv/bin/python python -m unittest discover -v
```

Source testing and installed-package testing serve different purposes. Remove
`PYTHONPATH` when checking a clean installed wheel outside the checkout. See
[installed-wheel usage](cli.md#using-an-installed-wheel-outside-the-checkout).

Environments, local results and build outputs are ignored by Git. Measurement
scope is configured separately; Git ignore rules do not define which source
bytes the evaluator measures.

Next: [install and probe the analyzers](analyzer-setup.md), then run the
[unchanged fixture](../README.md#try-an-unchanged-pair-first).
