# Analyzer setup

Setup is explicit and network enabled. Evaluation and the probe never install
dependencies. Use CPython 3.12.10 and Node 24.22.1 (see runtime-setup.md).

Windows PowerShell:

```powershell
uv run --no-project --no-python-downloads --python .venv/Scripts/python.exe python tools/setup_toolchain.py
uv run --no-project --no-python-downloads --python .venv/Scripts/python.exe python tools/probe_toolchain.py --output .tools/probe.json
```

Linux/macOS with the pinned runtimes installed:

```sh
uv run --no-project --no-python-downloads --python .venv/bin/python python tools/setup_toolchain.py --node /absolute/path/to/node --npm-cli /absolute/path/to/npm/bin/npm-cli.js
uv run --no-project --no-python-downloads --python .venv/bin/python python tools/probe_toolchain.py --output .tools/probe.json
```

`--npm-cli` identifies npm's JavaScript entry point; its default is the Windows
Node installation layout. No shell wrapper is used. `--tool-root` selects an
alternative isolated environment, including a fresh directory for reproducing
installation. The default is `.tools/analyzers/`, excluded from source control.
Select a new disposable tool root: setup rejects existing roots and linked
ancestors without clearing their contents. Use a fresh `--tool-root` for each
installation. The setup records entry points in entrypoints.json, preserving
Python's virtual-environment invocation path on POSIX. Python environment creation
and locked package installation use uv, avoiding copied macOS executables and
the ensurepip bootstrap. `--uv` may select its explicitly installed executable.

requirements-tools.lock pins all three Python analyzer packages with wheel SHA256
hashes; uv pip enforces them and rejects unlocked transitive requirements. The npm
lock pins all 119 packages with registry integrity values. npm ci uses the lock,
disables lifecycle scripts and does not update dependencies. Bootstrap uv/npm
are installation tools, outside the analyzer dependency inventory.

tools/licenses.json maps every locked package to retained notices in
tools/notices/. These are copied from the resolved package distributions, with
duplicate notice contents stored once by SHA256. Lizard declares MIT in package
metadata and carries Apache-2.0 source headers; both notices and the Apache text
are retained. Pathspec's bundled license is MPL-2.0. License retention is not a
claim that every dependency has the same license.

The real probe checks interpreter/analyzer versions, Lizard's function location,
CCN 3 and NLOC 8, and fresh jscpd JSON with both LF-normalized source files, 124
aggregate tokens, 62 duplicated tokens and one clone pair. Per-clone tokens remain the
upstream zero field. This tiny probe uses 20 minimum tokens and 5 minimum lines;
production evaluation profiles use 50 minimum tokens. Windows inputs/config/
output paths use forward slashes. The JSON evidence includes resolved entry
points, command, statistics and clone locations. Temporary inputs are removed
after verification. Missing tools, unsuccessful commands and missing/incorrect
JSON fail visibly; they never become successful zero metrics.

Two fresh isolated installs and probes were verified on Windows with the pinned
runtimes. Linux/macOS commands are provided for the portable setup; they have not
been executed here. Production metrics and three-platform CI remain later tasks.
