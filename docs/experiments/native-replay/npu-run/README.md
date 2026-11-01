# Fresh NPU native replay (task 6.3)

This bounded adapter replays the complete CPU/VitisAI provider modules and the
candidate's entire typed `checked_run.py` helper in isolated copies. It uses the
existing NPU virtual environment through uv without syncing dependencies or
accessing accelerator hardware. The archived collectors are read as frozen
check definitions; their historical execution blocks are never executed.

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python -m unittest discover -s docs/experiments/native-replay/npu-run -p test_replay.py -v
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/npu-run/replay.py run --out .runs/native-replay/another-fresh-npu
```

Run task 6.1 preparation first. The runner uses its `.runs/native-replay/task-6.1/npu`
snapshots and the existing `../npu_rag_from_folder/.venv/Scripts/python.exe`.
It freezes the exact runner, archived corpus/oracle, unchanged test hashes,
interpreter, test/type commands and safety policy before fresh checks. Per-side
source identities, the frozen trial digest, check definition digest and actual
artifact hashes bind evidence submitted to native compare. The generated
3,168 cases retain feed/output identity, exception details, input mutation and
shape/dtype guards. The 339 unchanged software tests cover provider construction,
public APIs, interaction contracts, service and postprocessing. Selected mypy
checks preserve the original provider modules and add the typed candidate helper.
The shared-safety audit retains the archived member/forbidden-call policy.

The separately named `NPU01-fresh-native-source-target-6.3` freezes a positive
production logical-statement reduction target before observations. Physical lines,
logical statements, aggregate decisions and functions require their native
comparable definitions. Internal call layers remain unavailable contextual
measurements. This deliberately differs from task 6.1's stricter all-metric target;
that preparation remains incomplete and unchanged. Located calls in the selected
provider/helper modules receive explicit owned, builtin, dependency or injected
callable contract reviews. Unknown selected contracts stop replay. Unchanged
wider-project unknown calls remain visible contextual evidence, without invented
reviews or availability.

Integrated variants submit actual candidate/evidence changes through the native
workflow: helper relocation outside ownership, attempted reclassification by
moving maintained implementation into an undeclared test path, omitted ownership,
missing required type evidence, and stale source-bound differential evidence.
These are adverse admission fixtures, not runnable product replacements. The
known dtype mutant has fresh differential evidence; its other candidate checks
are explicitly missing because observed behavior rejection already applies.

[results.md](results.md) records actual outcomes and separates whole-module native
metrics from the archived selected-family Radon measurements. [validation.md](validation.md)
records verification. `recorded/` retains compact raw per-case digests, mismatch
witnesses, test XML/logs, mypy logs, frozen records and source hash inventories.
`native-reports.zip` retains full native JSON/Markdown reports and comparison
inputs; compression is artifact storage only, never a benefit metric. Source
snapshots are reproduced byte-for-byte by task 6.1 preparation and verified against
the retained source identity inventories. No upstream or archived artifact is
changed. Evidence remains finite software observation, not universal equivalence,
hardware validation or authenticated execution.
