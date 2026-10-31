# Baseline package setup

The baseline harness targets CPython 3.12.10 and Node 24.22.1. Startup uses
Python's standard library and needs no analyzer, LLM account or network access.
Node is needed by the later duplication analyzer setup, not by help or version.
Installing the package requires the declared setuptools build dependency.

## Verified local runtimes

Verified on Windows on 2026-10-30:

| Runtime | Version | Resolved executable |
| --- | --- | --- |
| CPython | 3.12.10 | `C:\Users\mariu\AppData\Local\Programs\Python\Python312\python.exe` |
| Node | 24.22.1 | `C:\Program Files\nodejs\node.exe` |

The checkout path `C:\Users\mariu\Source\lesscode` resolves to
`D:\Source\lesscode` on this machine. Virtual environments therefore report
their executable under that resolved root. These paths describe this machine;
they are not embedded in the package.

Verify the selected runtimes before setup:

```powershell
uv python install 3.12.10
uv run --no-project --python 3.12.10 python -c "import sys; print(sys.version); print(sys.executable)"
node --version
node -p "process.execPath"
```

## Build and verify

From the repository root on Windows:

```powershell
uv venv --python 3.12.10 .venv
uv build --python .venv/Scripts/python.exe --wheel --out-dir dist
uv pip install --python .venv/Scripts/python.exe --no-deps dist/lesscode_baseline-0.1.0-py3-none-any.whl
uv run --no-project --no-python-downloads --python .venv/Scripts/python.exe python -m unittest discover -v
.\.venv\Scripts\lesscode-baseline.exe --help
.\.venv\Scripts\lesscode-baseline.exe --version
uv run --no-project --no-python-downloads --python .venv/Scripts/python.exe python -m lesscode_baseline --version
```

On Linux or macOS, use the same uv-managed CPython 3.12.10 setup and substitute
`.venv/bin/python` and `.venv/bin/lesscode-baseline` for Windows environment paths.
Cross-platform integration remains a separate validation task.

Standard-library discovery also supports source development before installation:
set `PYTHONPATH` to the absolute `src` directory, then run
`uv run --no-project --no-python-downloads --python .venv/Scripts/python.exe python -m unittest discover -v`.
The explicit environment path prevents ambient Python aliases from selecting
another runtime.

The CLI reference documents the integrated plan, run and summarize commands.

Environments, bytecode, build outputs, local `runs/` and `.workspaces/` are
ignored by Git. Later source-scope implementation must also exclude generated
material explicitly; Git ignore rules alone do not define measurement scope.
