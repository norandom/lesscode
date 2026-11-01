# Native external replay inputs (task 6.1)

These are preparation records for ATS01 and NPU01. Behavioral replay remains
pending tasks 6.2 and 6.3. Historical success does not supply valid native check
results: every paired result in the evidence templates is explicitly missing.

Run from the Lesscode repository root, with the existing environments and built
native executable. No environment sync, model download, upstream write, or old
collector invocation is performed:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/test_prepare.py
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/prepare.py --out .runs/native-replay/another-fresh-run
```

`prepare.py` refuses existing output. The generated source roots are disposable
and ignored under `.runs/`. The durable `inputs/` directory retains manifests,
complete frozen records, check definitions, ownership/evidence templates and a
preparation receipt from `.runs/native-replay/task-6.1`. Native `prepare` computes
each source identity and the current metric definitions; no historical identity
is invented. Every NPU supporting source/test/configuration file must still match
the original freeze before copying. Changed/missing upstream inputs stop preparation.
ATS archived complete modules supply the baseline and candidate; the mutant is
assembled from the archived reporting replacement without executing its code.

The ATS capability uses the entire agent module, including all report tables,
helpers and unchanged surrounding members. NPU owns the entire CPU and VitisAI
provider modules and maps the entire new `checked_run.py` helper into the
replacement. Its wider boundary contains the original frozen source, selected
unchanged tests, pyproject and lock file. These conservative whole-module scopes
differ from historical extracted-method scopes. Later native counts must be
reported separately. No native comparison/benefit result is claimed here.

The receipt copies historical observations under their original labels: Radon
`lloc`, collector AST `decisions`, Lizard `ccn_sum` and `physical`. They are not
native metric definitions. `current_artifact_sha256` records bytes observed now;
it does not authenticate historical execution. Historical collector exit zero
is never converted to a passed native result. Raw originals remain at
`../ats-report/` and `../npu-backends/`, with before/after artifact hashes in the
receipt. Historical execution lacks native trial/check/source bindings, so its
native provenance is unavailable. New bound evidence must come from fresh runs.

Check definitions hash the current corpus/oracle/collector definitions, selected
NPU test bytes and configuration. The old collectors write historical folders
and must not be invoked directly. Tasks 6.2/6.3 must retain equivalent observations
with output redirected to isolated runs, freeze the actual replay runner and
supply its raw outputs and source/check/trial bindings. A runner or check change
requires newly prepared records and paired runs. The placeholder-free evidence
templates intentionally contain JSON null results, rather than claimed outcomes.
All existing contract checks are required; missing artifacts remain incomplete.
Internal call depth may remain unavailable because full modules retain external
imports/dynamic calls. No hardware, throughput, universal-equivalence or project
maturity claim follows from these preparation records.

Prerequisites verified through `uv.exe` without syncing: both explicit Python
executables exist and report CPython 3.12.10. The NPU environment reports NumPy
1.26.4, pytest 9.1.1, ONNX 1.22.0 and mypy 2.3.1. The native executable accepts
both manifests and prepares all six source roots. Available-platform evidence is
Windows only for this task; Linux/macOS CI and WSL were not run here.
Task 6.2 fresh ATS replay: [ats-run/README.md](ats-run/README.md).
