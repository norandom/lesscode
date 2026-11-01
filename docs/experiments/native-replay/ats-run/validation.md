# Task 6.2 validation

Task brief: count whole replacement helpers, tables and callers (1.2); report
native source metrics separately from historical measurements (2.2); preserve
required checks on both sides and reject the behavioral mutant (3.3, 3.5); retain
machine and human reports with hypothesis and rationale (4.1, 4.2). Design:
Evidence and comparison contract; Data Models; System Flows.

Tests were written before implementation. RED command:

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python docs/experiments/native-replay/ats-run/test_replay.py
```

Full raw RED output (exit 1):

```text
Traceback (most recent call last):
  File "D:\Source\lesscode\docs\experiments\native-replay\ats-run\test_replay.py", line 9, in <module>
    spec.loader.exec_module(replay)
  File "<frozen importlib._bootstrap_external>", line 995, in exec_module
  File "<frozen importlib._bootstrap_external>", line 1132, in get_code
  File "<frozen importlib._bootstrap_external>", line 1190, in get_data
FileNotFoundError: [Errno 2] No such file or directory: 'D:\\Source\\lesscode\\docs\\experiments\\native-replay\\ats-run\\replay.py'
```

GREEN: all three tests passed. Tests execute real mutant detection across 8820
cases (2251 mismatches), reject changed full-module renderer fragments, and bind
raw mismatch summaries to source, trial, definition and artifact identities.
Native regression checks are parent-owned. This research integration adds no
product runtime behavior or feature flag. The fresh integrated run exercised
three complete real observation subprocesses and native prepare/compare; the
candidate was accepted and the mutant rejected for behavioral failures.
