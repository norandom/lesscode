# Reproducible local validation

`corpus.json` declares the licensed boltons snapshot and a six-attempt example
schedule. Its `content_hash` is the canonical scoped Snapshot identity produced
under `profiles/python.json`; upstream raw-file hashes, revision and license
remain separate in `tests/fixtures/real/boltons_mathutils/provenance.json`.

Build a fresh wheel, create a clean CPython 3.12.10 environment outside the
checkout, install that wheel with `--no-deps`, and provision the pinned analyzers
as described in `docs/analyzer-setup.md`. From a directory outside the checkout,
with `PYTHONPATH` removed, run the clean environment's Python:

```powershell
Remove-Item Env:PYTHONPATH -ErrorAction SilentlyContinue
C:/pilot/env/Scripts/python.exe D:/Source/lesscode/tools/validate_pilot.py --checkout D:/Source/lesscode --tool-paths D:/Source/lesscode/.tools/analyzers/entrypoints.json --out C:/pilot/evidence
```

Use `env/bin/python` on Linux/macOS. Output must be a new directory outside the
checkout. The runner verifies the imported package is outside the checkout and
executes real console/module `plan`, `run` and console `summarize` commands.
It installs nothing and makes no model or network requests. Git is used only to
establish the disposable synthetic candidate's tracked/untracked inventory.

Before evaluation the runner writes an actual three-attempt experiment schedule:
two unchanged boltons controls and one supplied shorter, broken candidate. Each
attempt executes separately. These evaluate supplied snapshots; they are not
independent candidate generations. The two controls must match in every complete
structural MetricEvidence field except raw artifact paths. Timings, run IDs and
timestamps are outside that structural comparison. Lizard must report 23 functions,
maximum CCN 15, with the highest-complexity function at mathutils.py lines 140–167.

Three further evaluations exercise synthetic duplicate removal, a shorter broken
clamp, and the known jscpd aggregate pair-span fixture (236 total tokens, 118
duplicated tokens, ratio 0.5). The accepted synthetic candidate uses six meaningful
lines to remain within the profile's fixed five-line jscpd minimum. Selected added,
removed and Git-untracked paths and test/generated/vendor classifications must be
retained, with non-product classes excluded from product byte totals. Fixture
originals are never modified. Short broken sources may have unavailable structural
metrics; their genuine behavior failures still require rejection and retain those
availability diagnostics.

`validation.json` records assertions, fingerprints, environment and normalized
structural evidence. `commands.json` retains exact commands and exits; companion
stdout/stderr files contain command disclosures. `inputs/` retains the actual
manifest, strict producer records and synthetic sources/evaluators. Each `runs/`
directory retains a frozen plan, raw outputs, report, summary and hash index.
`summary/` contains the completed experiment report and all verified evaluations.
All run and experiment indexes must recover as complete. The summary command's
expected exit is 1 because the scheduled broken candidate is rejected.

`--resume` performs read-only verification of an already completed pilot. It
checks the current source/evaluator/tool inputs, canonical outer and indexed
frozen plan bindings, strict producer records and completed experiment summary.
It creates no directories, writes no bytes and issues no workloads. Repeating
it keeps the same attempts and summary. Partial pilots and changed inputs require
a new output directory; their existing diagnostics remain untouched.

Native execution establishes evidence for that operating system only. Linux and
macOS remain separate CI validation gates. Passing declared checks does not prove
equivalence, AI authorship or simplification effectiveness. No agent token usage,
provider cost or runtime workload is inferred from source size.
