# Local evaluation CLI

> Archived (2026-11-10). This guide describes the Python prototype
> `lesscode-baseline`, retired at commit `fd0b5b9`. Its package, tests and some
> files linked below are no longer in the repository; the
> [preserved baseline index](archive/preserved-python-baseline.md) explains how to
> recover them. For the current tool, start at the [README](../README.md).

The evaluator compares supplied Python sources locally. It requires no Codex,
LLM service, hosted account, source upload or commercial analyzer. It does not
create a candidate. Passing declared checks is evidence, not proof of equivalence.

## Explicit installation

Follow [runtime setup](runtime-setup.md) for CPython 3.12.10 and package
installation, then [analyzer setup](analyzer-setup.md) for Node 24.22.1 and
the pinned tools. The Windows commands, from the repository root, are:

```powershell
uv.exe python install 3.12.10
uv.exe venv --managed-python --python 3.12.10 .venv
uv.exe pip install --python .venv/Scripts/python.exe .
uv.exe run --no-project --no-python-downloads --python .venv/Scripts/python.exe python tools/setup_toolchain.py
```

On Linux/macOS use `.venv/bin/python` and supply the absolute Node executable
path. Use uv to select the pinned Python; no Windows Store Python is needed.
If npm is elsewhere, pass `--npm-cli /absolute/path/to/npm/bin/npm-cli.js`.
Setup is a separate, network-enabled operation. Plan/run never install tools.
Setup records absolute interpreter, Node and jscpd paths in
`.tools/analyzers/entrypoints.json`. Keep those environments in place for the
lifetime of prepared plans. `--tool-paths` accepts that JSON object; relative
values resolve from the metadata file's directory. An explicit empty object
records unavailable tools and never discovers project executables.

## Prepare, inspect and run

From the repository root, this licensed unchanged pair demonstrates the complete
workflow. Choose new output names for every invocation:

```powershell
.\.venv\Scripts\lesscode-baseline.exe plan --profile benchmarks/profiles/python.json --baseline tests/fixtures/real/boltons_mathutils/source --candidate tests/fixtures/real/boltons_mathutils/source --tool-paths .tools/analyzers/entrypoints.json --out ../comparison-plan.json
.\.venv\Scripts\lesscode-baseline.exe run --plan ../comparison-plan.json --out ../comparison-run
```

On Linux/macOS use `.venv/bin/lesscode-baseline`; the [README](../README.md#try-an-unchanged-pair-first)
has the complete commands. The environment's module entry point accepts the same
arguments, for example `uv.exe run --no-project --no-python-downloads --python .venv/Scripts/python.exe python -m lesscode_baseline --help`.
`--help` and `--version`
work without any analyzers. Planning probes fixed tool versions but executes no
project checks. Both commands print source roots and resolved argv, environment,
working directory and limits. Run flushes this disclosure before executing the
checks. The working directory is relative to each fresh disposable source copy.
Invoking run authorizes those declared commands; inspect the plan first.
Ordinary disposable copies do not isolate hostile code: use an external sandbox
for untrusted workloads.

The boltons profile is fixture-specific. Read [profile configuration](profiles.md)
before substituting your own source roots. The
[technical overview](technical-overview.md) explains measurement definitions,
decision precedence and recovery contracts.

The profile and independent evaluator resources must be outside both sources.
Plan and run outputs must be new and outside sources and the evaluator directory.
Sources and evaluator files are fingerprinted; changes after preparation invalidate
the run. Prepare a new plan after edits or reinstalling the harness. Originals
are preserved. `run --retain-workspaces` retains disposable copies for diagnosis.

The run directory contains schema-versioned `report.json`, human-readable
`summary.md`, raw command/analyzer outputs and an artifact index. Partial failures
retain available diagnostics; errors print their surviving location. Read the
report's decision, reasons, unavailable evidence and limitations before interpreting
metric improvements. Missing tools produce unavailable metrics, never zeros.

| Exit | Meaning |
| --- | --- |
| 0 | Accepted, or successfully prepared plan |
| 1 | Rejected |
| 2 | Invalid input/contract/integrity |
| 3 | Incomplete, baseline ineligible, or interrupted |
| 4 | Internal or persistence failure; inspect retained diagnostics |

## Using an installed wheel outside the checkout

Build and install a wheel in a clean CPython 3.12 environment:

```powershell
uv.exe build --python 3.12.10 --wheel --out-dir dist
uv.exe venv --managed-python --python 3.12.10 ../baseline-cli-env
uv.exe pip install --python ../baseline-cli-env/Scripts/python.exe dist/lesscode_baseline-0.1.0-py3-none-any.whl
```

On Linux/macOS use `../baseline-cli-env/bin/python`. Change to another directory
and invoke the installed console executable (in `Scripts` on Windows, `bin` on
Linux/macOS), or that environment's `python -m lesscode_baseline`. Pass absolute
profile, baseline, candidate, tool metadata and output paths. There is no checkout
working-directory dependency: the wheel bundles exact analyzer locks, while the
caller supplies evaluator resources and explicitly provisioned external tools.
An installed package's default tool bindings may be unavailable; supply
`--tool-paths /absolute/path/to/entrypoints.json` for reproducible operation.
No `PYTHONPATH` is needed.

## Fixed analyzer settings and audit files

The Python adapters implement Lizard `language="python"`, `extensions=false`
and jscpd `format="python"`, `mode="weak"`, `minTokens=50`, `minLines=5`,
`ignoreCase=false`. Omitted option keys mean these defaults. Preparation rejects
unknown options and incompatible values before probing tools. jscpd weak mode
excludes comment tokens; `skipComments` is not an option of the pinned vendor.
`maxLines` and `maxSize` are derived from the actual selected input manifest and
cannot be overridden in the profile.

Metric log directories retain unique input inventories, the effective jscpd
configuration and versioned `*.execution.json` audit sidecars. Each sidecar keeps
the typed execution result (actual argv, limits, tool version, status, exit code,
elapsed time, timestamp, environment identity and associated stdout/stderr),
plus its absolute disposable workspace and actual working directory. The actual
argv inputs remain in the artifact index after disposable workspaces are removed.
Analyzer audits are raw metric evidence; observation executions describe declared
behavior checks. Historical Profile/Plan records keep their existing format.

## Optional supplied evidence

Append `--baseline-evidence /path/envelope.json` or
`--candidate-evidence /path/envelope.json` to run, repeating flags for multiple
envelopes. These import the existing versioned optional-evidence contract as
`producer_supplied`. The producer's source hash, canonical prepared-plan hash,
scope, tool metadata and provenance remain authoritative and validated. Imports
do not recertify freshness. The CLI does not generate coverage/mutation results;
generated evidence with explicit freshness bounds remains a Python integration API.

> Archived (2026-11-10). This guide describes the Python prototype
> `lesscode-baseline`, retired at commit `fd0b5b9`. Its package, tests and some
> files linked below are no longer in the repository; the
> [preserved baseline index](archive/preserved-python-baseline.md) explains how to
> recover them. For the current tool, start at the [README](../README.md).

## Summarizing recorded experiments

```powershell
.\.venv\Scripts\lesscode-baseline.exe summarize --experiment C:/pilot/inputs/experiment.json --trials C:/pilot/inputs/control.json --trials C:/pilot/inputs/skill.json --runs C:/pilot/evidence --out C:/pilot/summary
```

`--experiment` is a strict schema-version-1 `ExperimentManifest`. Each repeatable
`--trials FILE` imports exactly one schema-version-1 `TrialRecord` JSON object.
The manifest does not name producer files: there is no implicit file discovery.
Omit `--trials` to report all scheduled trials as missing. Unknown fields,
duplicate JSON keys and duplicate trial IDs are input errors. Use the shared
`contracts` serializers to produce these records; all controls, budgets,
provenance and explicit unavailable telemetry fields are required.

`--runs` is the artifact root for each record's relative `run_artifact`, which
may identify a run directory or its `report.json`. For example,
`runs/control/report.json` resolves below `C:/pilot/evidence`. Only completed,
hash-indexed run artifacts establish evaluated evidence; bare, interrupted or
tampered reports remain incomplete with diagnostics. Summarization executes no
project checks, analyzers or LLM requests. Recorded source and evaluation inputs
remain unchanged. Use a new output directory separate from run artifacts,
source roots and evaluator code. It may be a sibling of the manifest and trial
files; input files themselves cannot be overwritten or contain the output.

The output includes the public schema-version-1 `ExperimentReport` in
`report.json`, escaped human evidence in `summary.md`, and complete verified
`RunReport` records with trial references and origins in `evaluations.json`.
`manifest.json` records storage state; `artifacts.json` is published last and
hashes the final files. A missing/mismatched index means incomplete storage even
if the manifest says finalized. Python callers can inspect this with
`report.recover_experiment(Path(...))`. Failed writes retain diagnostics and
temporary files; reuse requires a new output directory.

Summary exit precedence is **2** for invalid input or an invalid planned outcome,
then **3** for any scheduled missing/incomplete attempt, then **1** for any valid
rejection, otherwise **0** for successful report production. Persistence/internal
failures return **4**. Baseline-ineligible outcomes alone permit summary exit 0;
the pair `run` command returns 3 for that state. Summary exit 0 does not claim
acceptance: every state remains separately counted. Extra records are listed
and excluded from scheduled denominators.

Correctness uses its disclosed eligible-attempt denominator, independently of
acceptance among planned attempts. All-attempt and accepted-only structural
distributions remain separate; paired statistics exclude confounded comparisons.
Statistics are descriptive with sample counts and no inferential confidence
interval. Observed tokens, provider cost and agent time retain individual source,
unit and timestamp; currencies are not pooled and absent observations remain
unavailable. Evaluator elapsed time is separate, as are runtime workloads and
environment identities. Corpus source/license and generation provenance remain
producer declarations; matching metadata does not prove causal effectiveness.
