# R136 · E6 acceptance rerun collected 57 cases and passed 56

Execution 18 November 2026, 07:54–07:58 Europe/Berlin. Reported the same morning. Retrospective verification for [corrective task 1.1](../../../.kiro/corrective-tasks.md), following [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md). [R45](../2026-11-09/03-e6-generalization-trial-qualification.md) recorded 46 of 46 passes.

| Suite | Collected | Passed | Skipped | Failures or errors |
| --- | ---: | ---: | ---: | ---: |
| Reporting | 33 | 32 | 1 | 0 |
| Kernel | 5 | 5 | 0 | 0 |
| Root | 19 | 19 | 0 | 0 |
| Total | 57 | 56 | 1 | 0 |

The source-import guard passed. The only skip was the reporting test for authenticating persisted envelopes. Its marker requires a populated operator database via `FACDRONE_TEST_PG_DSN`. The embedded test server supplies the schema.

## Method and limits

The test was whether the current runner would import from the frozen E6 candidate, exit successfully and reproduce the historical count. Run the original from the lesscode root using the existing specimen interpreter and four pytest-xdist workers. It exited 0 for all three suites. Repeat with a copy that keeps the original harness directory and adds a distinct JUnit output file per suite. That invocation also exited 0. Only the second invocation supplies the counts.

```powershell
$env:PYTEST_ADDOPTS = '-n 4 --dist loadfile'
$env:PYTHONDONTWRITEBYTECODE = '1'
& .runs/e2-facdrone/venv/Scripts/python.exe docs/experiments/deconvolution/e6-report-envelope/run_e6.py .runs/e6-report-envelope/cand-e6
```

The two populations differ in size. The historical report supplies no case list in its acceptance row, so this rerun does not explain the difference. It establishes present behavior on this specimen and test environment. [Runner identities, versions, results and skip reason](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118/e6-count-receipt.json) · [Original and counted logs plus JUnit files](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118)
