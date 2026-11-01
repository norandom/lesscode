# Bounded real TypeScript trial (task 6.5)

TS01 exercised OpenUltraCode's existing typed degradation-notice capability in
isolated copies. Native comparison accepted a modest replacement of its custom
slug-building loop with existing JavaScript string/RegExp operations. Both strict
project builds and the three existing capability tests passed; 5,542 frozen API
observations matched. A separator mutant produced 4,158 mismatches. No dependency
was installed and upstream remained unchanged. See [results.md](results.md), the
pre-edit [plan](plan.md), and [native report](recorded/report.md).

Fresh execution from the Lesscode root, using only existing installed tools:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/typescript-run/test_replay.py
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/typescript-run/replay.py --out .runs/native-replay/ts01-new
```

The runner refuses existing output. It reads `D:/Source/OpenUltraCode`, copies the
complete selected source/test/build closure, binds an installed-capability
inventory, freezes targets/checks and edits one helper in an isolated candidate.
It uses the existing Node/tsx test runner and TypeScript build, with absolute
installed-loader/typeRoots bindings so no node_modules copy, link or install is
needed. A fresh run on changed upstream source is a newly frozen observation,
not a reproduction of this recorded snapshot. Native analysis uses the compiled
Windows core; Linux/macOS and WSL were not exercised by this trial.

For exact native re-admission of the recorded supplied evidence, expand
[complete-run.zip](recorded/complete-run.zip) into a fresh ignored directory:

```powershell
$ts = '.runs/native-replay/ts01-archived'
Expand-Archive docs/experiments/native-replay/typescript-run/recorded/complete-run.zip $ts
native/target/debug/lesscode-core.exe compare --frozen "$ts/frozen.json" --baseline "$ts/baseline" --candidate "$ts/candidate" --evidence "$ts/candidate-evidence.json" --artifact-root $ts --out "$ts/recomparison"
```

Expect exit 0 and `accepted`. This consumes frozen observations; it does not
rerun the project checks. [readmission.json](recorded/readmission.json) records an
actual relocated re-admission with exact result equality. The archive includes
complete baseline/candidate/mutant snapshots, raw observations and logs, frozen
records, execution receipts and the complete source diff. Readable primary
records are also retained alongside it; [artifact hashes](recorded/artifact-sha256.json)
bind their bytes. Re-execution of Node checks requires the installed tools and
versions in [inventory.json](recorded/inventory.json).

This supports the exercised finite TypeScript workflow on one manually selected
capability. It does not establish broad TypeScript maturity, general discovery
efficacy, universal equivalence or reduced maintenance effort. No upstream
adoption was performed.
