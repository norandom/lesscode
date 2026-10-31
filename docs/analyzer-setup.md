# Analyzer setup

Setup is explicit and network enabled. Evaluation and the probe never install
dependencies. First complete [package setup](runtime-setup.md) with CPython
3.12.10 and install Node 24.22.1 with npm.

Windows PowerShell:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv/Scripts/python.exe python tools/setup_toolchain.py
uv.exe run --no-project --no-python-downloads --python .venv/Scripts/python.exe python tools/probe_toolchain.py --output .tools/probe.json
```

Linux/macOS with the pinned runtimes installed:

```sh
uv run --no-project --no-python-downloads --python .venv/bin/python python tools/setup_toolchain.py --node /absolute/path/to/node --npm-cli /absolute/path/to/npm/bin/npm-cli.js
uv run --no-project --no-python-downloads --python .venv/bin/python python tools/probe_toolchain.py --output .tools/probe.json
```

The Linux/macOS command contains placeholder paths: replace both with actual
installed files. `command -v node` identifies the Node invocation; npm's resolved
entry point is often `npm/bin/npm-cli.js`, but the location depends on your
installation. Supply the JavaScript file, not a shell wrapper.

`--npm-cli` identifies npm's JavaScript entry point; by default setup looks below
the resolved Node directory at `node_modules/npm/bin/npm-cli.js`. This matches
the common Windows layout. Other layouts need the explicit argument above.
No shell wrapper is used. `--tool-root` selects an
alternative isolated environment, including a fresh directory for reproducing
installation. The default is `.tools/analyzers/`, excluded from source control.
Select a new disposable tool root: setup rejects existing roots and linked
ancestors without clearing their contents. Use a fresh `--tool-root` for each
installation. The setup records entry points in `entrypoints.json`, preserving
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

Native Windows, Linux and macOS setup, genuine probes and full installed pilots
passed in the [recorded platform validation](ci-validation.md). These results
apply to the recorded versions and fixture inputs, not every possible tool
installation layout.

After a successful default installation, use
`--tool-paths .tools/analyzers/entrypoints.json` when preparing a plan. A custom
tool root produces its metadata there instead. Keep that environment in place
while a prepared plan refers to it; rerunning setup requires a fresh tool root.
Next: run the [unchanged fixture](../README.md#try-an-unchanged-pair-first) or
configure a [project-specific profile](profiles.md).
