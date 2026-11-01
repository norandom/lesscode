# Fresh ATS native replay (task 6.2)

Run from the repository root after task 6.1 preparation, with a fresh output:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/ats-run/test_replay.py
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/ats-run/replay.py run --out .runs/native-replay/another-fresh-ats
```

The runner freezes its bytes, archived corpus/observation harness, oracle,
fragment hashes and explicit interpreter before behavioral execution. Native
prepare freezes the full-module snapshots, definitions and required check IDs.
Three isolated uv subprocesses execute the unchanged 8820-case corpus each.
Renderer ASTs must match the frozen fragments before execution. The archived
collector main block is never invoked. Exact returned text or exception type and
message, plus subclass dispatch overrides, retain the original semantics.

The separately named task 6.2 trial freezes a source-reduction target before
execution: at least one fewer production logical statement, with physical lines,
decisions and functions also requiring comparable available definitions. Internal
call layers remain unavailable contextual evidence. The task 6.1 preparation
target required call layers too; its initial probe was incomplete. This trial
does not satisfy or replace that older target. Preparation and original archived
inputs remain unchanged. Exact call expressions receive explicit built-in,
external API or whole-module owned relationship reviews. Unknown contracts stop
the runner. Review does not make unresolved call layers available.

[results.md](results.md) compares native whole-module counts with separately
labeled historical extracted-renderer counts. `recorded/` retains complete source
snapshots, frozen records, raw per-case digest observations, mismatch witnesses,
paired evidence and native JSON/Markdown reports. The receipt binds the observed
native executable and preserves before/after original artifact hashes. Hashes
establish consistency, not authenticated execution. Finite reporting checks do
not prove universal equivalence or validate unrelated external agent services.
No upstream repository is changed; corpus inputs are fictional.
