# Local Python pilot

This profile measures the complete, unmodified `boltons/mathutils.py` module
from boltons 25.0.0, commit `c23dbdadb6fecdf505eb4231559561913b84c13f`.
The fixture and retained BSD-3-Clause license are under
`tests/fixtures/real/boltons_mathutils/`. Exact upstream URLs, byte lengths and
SHA-256 hashes are in its `provenance.json`. This is a single-module pilot,
not validation of the whole upstream project. Generation provenance is unknown.

`benchmarks/corpus.json` is an `ExperimentManifest` and `python.json` is a
`Profile`, both schema version 1 from `lesscode_baseline.contracts`. The corpus
license artifact is repository relative. The corpus content hash identifies the
single upstream product file; it is not the future canonical multi-file snapshot
identity. No candidate, completed trial, observed cost or success is fabricated.
Three paired repetitions are scheduled for unchanged and supplied-simplification
arms. These are local supplied-snapshot controls with no model invocation. A
future model experiment must declare its real model/settings, prompt, permissions
and budget in a new manifest before producing trials.

## Explicit setup and verification

Use CPython 3.12.10 and Node 24.22.1. See `docs/runtime-setup.md` and
`docs/analyzer-setup.md` for runtime setup and hash-locked analyzer installation.
The fixture and check runner need only the Python standard library; no boltons
package install or network access is needed to run checks. Install the local
package and analyzers explicitly from the repository root:

```powershell
py -3.12 -m pip install .
py -3.12 tools/setup_toolchain.py
py -3.12 tools/probe_toolchain.py --output .tools/probe.json
$env:PYTHONPATH=(Resolve-Path src).Path
.venv-task11/Scripts/python.exe -m unittest tests.test_pilot -v
```

On Linux/macOS, use the pinned `python3.12`, the documented Node/npm arguments
for `setup_toolchain.py`, and `PYTHONPATH=src python3.12 -m unittest tests.test_pilot -v`.
The tests copy the source to a disposable directory, resolve the trusted runner
against the profile directory, and run both declared commands with positive
timeouts. All report outputs go into temporary directories.

For direct checks, copy the fixture `source` directory to a new disposable
directory outside the repository; use absolute paths for SOURCE, RUNNER and
OUTPUT below, with RUNNER pointing at this directory's `check.py`:

```text
python RUNNER --source SOURCE --output OUTPUT/native.xml --mode native
python RUNNER --source SOURCE --output OUTPUT/independent.xml --mode independent
```

Profile resources and the runner argument resolve relative to the profile
directory during plan preparation. The source argument `.` and command cwd are
the disposable check root, and `.checks/*.xml` are fresh source-relative result
paths. Prepared commands must resolve the runner to an absolute path before
changing cwd. No placeholder syntax or runtime installation is involved.

## Recorded evidence and limits

The native command executes exactly ten original upstream doctest examples and
fails if examples disappear. The independent command checks fourteen external
golden cases (clamp, ceiling, floor and invalid ranges/options), clamp bounds and
identity, and 256 byte/Bits round trips. Both commands emit JUnit with fixed
inventories and nonzero exit on failure. Missing APIs/import errors also fail.
They do not prove behavior for every Bits operation, numeric type or edge case.

Required measurements are Lizard complexity, jscpd duplication, raw source bytes,
physical lines and compressed source bytes. Tool versions, language, weak-token
mode, 50 minimum tokens, five minimum lines, explicit file limits and command
budgets are frozen in the profile. Native analyzer defaults/config files must
not override those settings. Commands have 30-second and 1 MiB log budgets;
producer attempts have a separate 60-second budget. Source bytes may not increase
(delta maximum zero); there are no other structural threshold rules. Coverage,
mutation, architecture, runtime and provider cost are not required in this pilot.

The original source root contains only `mathutils.py`; exclude rules keep check
reports, caches, tools, virtual environments and runs out of measurements. Keep
the baseline, profile, independent runner/goldens/prompt and retained provenance
outside the candidate rewrite root. The later evaluator freezes their identities
before execution. Local integrity detection is not an adversarial sandbox.
