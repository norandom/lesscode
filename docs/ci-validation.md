# Native platform evidence

`.github/workflows/baseline-platforms.yml` runs on GitHub-hosted
`ubuntu-24.04`, `windows-2025` and `macos-15-intel`. Each job explicitly installs
CPython 3.12.10 through uv 0.13.5 and Node 24.22.1, then performs the locked
Lizard 1.17.31 / jscpd 4.0.5 setup and genuine analyzer probe. No Python Store
alias, shell launcher for npm, analysis service or LLM account is used.
Both bootstrap and clean installation create environments with explicit
`uv venv --managed-python --python 3.12.10`. The workflow deliberately omits
`UV_PYTHON_PREFERENCE`: uv 0.13.5 rejects that preference when combined with
`--managed-python`. Subsequent commands select the recorded environment's
absolute Python path.
Analyzer setup also uses uv to create a fresh tool environment and install its
hash-locked Python packages. Interpreter bindings retain the venv invocation
path, fingerprint the resolved binary and `pyvenv.cfg`, and reject changed
runtime targets or environment configuration during integrity verification.

`tools/ci_validate.py` retains commands, working directories, exit outcomes,
stdout/stderr, timings and runtime identities.
The helper supplies a canonical dedicated scratch directory through `TMPDIR`,
`TEMP` and `TMP`; its path is recorded in `runtime.json`. This avoids macOS's
default `/var` alias while preserving strict source and artifact symlink guards.
Metric staging independently canonicalizes its owned temporary root before
launching analyzers, keeping Windows 8.3 paths and actual process cwd consistent.
It runs canonical verbose unittest discovery with absolute source `PYTHONPATH`,
followed by the native parent/descendant timeout test. Host-inapplicable unittest skips remain visible
in the canonical log. Other failures remain failures. A failed test stage does
not prevent independent build and installed-package evidence from being gathered.

The same job builds the wheel, creates a fresh uv environment outside the
checkout, installs the wheel without dependencies, and checks both console and
module help/version from that external directory with `PYTHONPATH` absent.
It then runs `tools/validate_pilot.py`: six genuine evaluations, repeated
structural metrics, known complexity/duplication values, preserved originals,
behavior rejection and experiment aggregation. Completed pilot recovery uses
`--resume` and verifies the retained artifacts read-only. Partial pilots require
a fresh output directory and cannot become a resumed success.

The `baseline-<runner>-<attempt>` artifact is uploaded even after a failed stage.
It retains bootstrap logs, `validation/commands.json`, `runtime.json`, native
test diagnostics, probe results, the wheel and pilot artifacts for 14 days.
The disposable installed environment is excluded. GitHub job logs also retain
action setup failures. Analyzer dependencies are locked; only uv's dependency
cache is enabled. Evaluations and result artifacts are never cached. Uploaded
analysis inputs are exclusively the checked-in public fixtures; there are no
model calls or uploads of user projects.

Notice files use targeted Git `-text` attributes because their names identify
exact distribution bytes, including upstream CRLF. The toolchain tests verify
all retained notice hashes and Git's effective attribute. Preserve those bytes
when changing the inventory; never normalize an upstream notice to repair its
filename hash.

A green workflow configuration or a local Windows check is insufficient
cross-platform evidence. Read the native job outcomes, canonical skips,
`validation/result.json` and pilot `validation.json` for the tested revision.
An unexecuted platform remains outstanding. This validates the harness on the
declared fixtures; it does not establish general simplification effectiveness.

Action SHA pins were resolved from official upstream tags on 2026-10-31:

- [checkout v6](https://github.com/actions/checkout/tree/d23441a48e516b6c34aea4fa41551a30e30af803)
- [setup-uv v10.3.3](https://github.com/astral-sh/setup-uv/tree/68559e99cc807c1eb7e9238fd43a15bba460eec2)
- [setup-node v7](https://github.com/actions/setup-node/tree/949feb2413d6458794dcd2491c4babbbce0c15c1)
- [upload-artifact v7](https://github.com/actions/upload-artifact/tree/cf430e030ddbb5b0abf93d22962f4752f3646cd9)

Runner labels are documented in the
[GitHub-hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
