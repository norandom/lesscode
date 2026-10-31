# Local evaluation CLI

The evaluator compares supplied Python sources locally. It requires no Codex,
LLM service, hosted account, source upload or commercial analyzer. It does not
create a candidate. Passing declared checks is evidence, not proof of equivalence.

## Explicit installation

Use CPython 3.12.10 and Node 24.22.1. From the repository root, install the
package and explicitly provision the pinned open-source analyzers:

```powershell
py -3.12 -m pip install .
py -3.12 tools/setup_toolchain.py --node 'C:/Program Files/nodejs/node.exe'
```

On Linux/macOS use `python3.12` in place of `py -3.12` and supply the absolute
Node executable path. If npm is elsewhere, pass `--npm-cli /absolute/path/to/npm/bin/npm-cli.js`.
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
lesscode-baseline plan --profile benchmarks/profiles/python.json --baseline tests/fixtures/real/boltons_mathutils/source --candidate tests/fixtures/real/boltons_mathutils/source --tool-paths .tools/analyzers/entrypoints.json --out ../comparison-plan.json
lesscode-baseline run --plan ../comparison-plan.json --out ../comparison-run
```

`python -m lesscode_baseline` accepts the same arguments. `--help` and `--version`
work without any analyzers. Planning probes fixed tool versions but executes no
project checks. Both commands print source roots and resolved argv, environment,
working directory and limits. Run flushes this disclosure before executing the
checks. The working directory is relative to each fresh disposable source copy.
Invoking run authorizes those declared commands; inspect the plan first.
Ordinary disposable copies do not isolate hostile code: use an external sandbox
for untrusted workloads.

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
py -3.12 -m pip wheel . --no-deps --wheel-dir dist
py -3.12 -m venv ../baseline-cli-env
../baseline-cli-env/Scripts/python.exe -m pip install dist/lesscode_baseline-0.1.0-py3-none-any.whl
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

## Optional supplied evidence

Append `--baseline-evidence /path/envelope.json` or
`--candidate-evidence /path/envelope.json` to run, repeating flags for multiple
envelopes. These import the existing versioned optional-evidence contract as
`producer_supplied`. The producer's source hash, canonical prepared-plan hash,
scope, tool metadata and provenance remain authoritative and validated. Imports
do not recertify freshness. The CLI does not generate coverage/mutation results;
generated evidence with explicit freshness bounds remains a Python integration API.
