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
py -3.12 -c "import sys; print(sys.version); print(sys.executable)"
node --version
node -p "process.execPath"
```

## Build and verify

From the repository root on Windows:

```powershell
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip wheel . --no-deps --wheel-dir dist
.\.venv\Scripts\python.exe -m pip install --no-deps dist/lesscode_baseline-0.1.0-py3-none-any.whl
.\.venv\Scripts\python.exe -m unittest discover -v
.\.venv\Scripts\lesscode-baseline.exe --help
.\.venv\Scripts\lesscode-baseline.exe --version
.\.venv\Scripts\python.exe -m lesscode_baseline --version
```

On Linux or macOS, select a CPython 3.12.10 executable, use it for
`-m venv .venv`, and substitute `.venv/bin/python` and
`.venv/bin/lesscode-baseline` for the Windows environment commands.
Cross-platform integration remains a separate validation task.

Standard-library discovery also supports source development before installation:
set `PYTHONPATH` to the absolute `src` directory, then run
`python -m unittest discover -v` with the selected interpreter.

Only help and version are available at this foundation stage. Evaluation
commands will be integrated after their components are implemented.

Environments, bytecode, build outputs, local `runs/` and `.workspaces/` are
ignored by Git. Later source-scope implementation must also exclude generated
material explicitly; Git ignore rules alone do not define measurement scope.
