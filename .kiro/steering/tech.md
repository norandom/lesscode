# Technology Direction

Start with a local measurement CLI and a skill. Add MCP and transactional transformations after controlled evaluation demonstrates benefit. The approved baseline design uses Python for the harness and first fixture language; JS/TS and Java are later scope.

Use uv/uvx for Python development commands and setup. On Windows invoke uv.exe/uvx.exe directly to avoid local PowerShell function wrappers. Select the pinned interpreter explicitly; never invoke or install Windows Store Python. Existing evaluator subprocesses use recorded, absolute pinned runtime bindings. For the current validation environment: uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python -m unittest discover -q, with PYTHONPATH set to resolved src. Save multiline Python as a script instead of piping it through a shell wrapper.

The approved baseline pins CPython 3.12.10, Node 24.22.1, Lizard 1.17.31 and jscpd 4.0.5. Exact transitive locks, real analyzer probes and native Windows/Linux/macOS validation passed for runtime revision 778e46258dbca42d5b07191a4f011b2cb858fb61 (run 41198054658). Task 7.4 readiness review and final feature validation remain pending. ast-grep belongs to later structural rewrites; Radon and mutation engines are not baseline runtime dependencies. Use declared repository-native and independent checks, with optional supplied coverage/mutation evidence. Verify dependency licenses and retain applicable source notices, including original CRLF bytes where hash-named.

Unavailable metrics must remain unavailable, never zero. Different analyzers' complexity definitions are not interchangeable. AST similarity does not establish behavioral equivalence. Fix the measurement scope, exclusions and configuration within each experiment.

Keep evaluation tests and configuration outside the rewrite workspace. Record their hashes. A separate process and hashes detect accidental tampering; adversarial enforcement requires externally controlled CI or OS permissions. The rewriting agent must not weaken tests, thresholds or exclusions.

Preserve user changes. Inspect commands before executing untrusted repository workloads, use isolated workspaces and timeouts, and avoid blanket reset/clean operations.
