# Task 6.3 validation

Task brief: exercise complete CPU/VitisAI/helper replacement with frozen software
checks and dtype negative control, count the entire replacement, and prevent
acceptance of moved/hidden implementation, narrowed ownership, missing checks and
stale evidence. Requirements checked: 1.2, 1.3, 2.1, 2.5, 3.2, 3.3, 3.5, 4.4.
Design checked: Scope contract, Metrics contract, Evidence and comparison
contract, FrozenTrial/EvidenceBundle data models and External trial integration.

The tests were written before `replay.py`; the initial discovery exited 1 with
`FileNotFoundError` importing the absent runner. [red-phase.txt](red-phase.txt)
retains the complete command and raw failing output. This is research integration
with standard RED/GREEN validation, without a product feature flag.

Fresh execution completed with exit 0:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/npu-run/replay.py run --out .runs/native-replay/task-6.3-verified
```

Both sides passed all 339 unchanged software tests, fresh 3,168-case observations,
selected mypy checks and the retained-member/forbidden-call safety policy. The
mutant produced 572 mismatches and native rejected-behavior. All five integrated
adverse variants were incomplete or invalid-input; none was accepted. The first
incomplete adapter outcome is retained separately in `first-incomplete.json`.

The completed fresh run was asserted without repeating its expensive project
checks; the default test command creates and executes a new fresh trial:

```powershell
$env:NPU_REPLAY_VALIDATION_OUT='.runs/native-replay/task-6.3-verified'
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python -m unittest discover -s docs/experiments/native-replay/npu-run -p test_replay.py -v
Remove-Item Env:NPU_REPLAY_VALIDATION_OUT
```

Result: 2 tests passed. Assertions cover native verdicts, raw differential counts,
339 tests per side, type exits, invalid-input stderr and exit codes, the actual
runner hash, and preserved historical/prepared-input hash inventories. Unknown
selected call contracts explicitly raise an error rather than receiving a blanket
external classification. An initial test assertion expected helper-path text for
all invalid variants; it was corrected to require source identity mismatch for
the stale-evidence variant, matching the actual native rejection.

The retained raw artifact hashes were independently re-read and checked against
all baseline/candidate/mutant supplied evidence references. The full native
reports and isolated source copies are in `recorded/native-reports.zip`; raw
check artifacts and frozen/evidence records are directly accessible under
`recorded/`. `artifact-sha256.json` binds the retained artifacts, including the
archive. The archive contains the exact executed runner as `check-runner.py`.
Storage compression is unrelated to measured source benefit.

Historical ATS/NPU artifact hashes and all task 6.1 prepared input bytes were
unchanged before/after execution. `git -C ../npu_rag_from_folder status --porcelain`
returned no entries after the run. No upstream source, original artifact, task
state or commit was changed. All executable work occurred in isolated local
copies. The existing NPU interpreter was selected through uv with
`--no-project --no-python-downloads`; no environment sync, installation or hardware
access was performed. The native executable hash is recorded in the receipt.

Limitations: finite software observations and local consistency checks; no claim
of universal equivalence, authenticated execution, hardware validation or
performance improvement. Unresolved call layers remain unavailable, and unchanged
wider-project relationships remain contextual. Task 6.1's stricter target is still
incomplete. The candidate is accepted only for the explicitly separate task 6.3
source-metric target, with no upstream adoption.

The same two assertions also passed against the durable `recorded/` directory
using `NPU_REPLAY_VALIDATION_OUT=docs/experiments/native-replay/npu-run/recorded`;
this checks the delivered evidence without repeating project execution.
