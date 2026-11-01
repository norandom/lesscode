# Task 6.1 validation

Requirements checked: 1.1, 2.3, 3.2, 4.1, 4.4.
Design checked: Scope contract; Evidence and decision contract; Data Models;
Testing Strategy integration and adverse fixtures; docs/experiments/native-replay
file ownership. This task prepares fixtures only.

Task brief: prepare isolated complete ATS/NPU replacements, freeze source/check
identities using the real native CLI, preserve original artifacts and historical
metric names, and withhold native behavioral success until fresh paired replay.

RED command:
`uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/test_prepare.py`

Raw RED output before implementation (preparation flag default OFF):
```text
E
======================================================================
ERROR: setUpClass (__main__.ReplayInputs)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "D:\Source\lesscode\docs\experiments\native-replay\test_prepare.py", line 20, in setUpClass
    cls.receipt = replay.prepare_inputs(cls.output)
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "D:\Source\lesscode\docs\experiments\native-replay\prepare.py", line 6, in prepare_inputs
    raise RuntimeError("native replay preparation disabled")
RuntimeError: native replay preparation disabled

----------------------------------------------------------------------
Ran 0 tests in 0.008s

FAILED (errors=1)
```

The flag was enabled to implement behavior and then removed. Final command above:
```text
....
----------------------------------------------------------------------
Ran 4 tests in 4.110s

OK
```

The tests invoke the actual CLI. They verify all six frozen identities/current
metric definitions, full NPU helper ownership, explicit missing historical native
provenance/results, historical adverse mismatches, original artifact immutability,
incomplete comparison with missing evidence, source-stale rejection, and changed
check-definition rejection. Both adverse binding tests return native exit 3;
missing evidence returns exit 2. No historical workload was executed.

Preparation command:
`uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/prepare.py --out .runs/native-replay/task-6.1`

Output: `Prepared ATS/NPU inputs; fresh behavioral evidence remains missing.`

`inputs/preparation-receipt.json` contains identical before/after SHA-256 maps
for every historical ATS/NPU artifact, all source identities and exact native
metric definition/check identities. Copies of the generated frozen JSON records
are under `inputs/ats/` and `inputs/npu/`. Tests and research remain separate from
production classification. No task 6.2/6.3 acceptance, native metric deltas, or
behavior preservation result is claimed. Historical finite outcomes and raw logs
remain reference evidence with missing native execution provenance.
