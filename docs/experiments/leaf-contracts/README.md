# Library replacement trial: five leaf records

Date: 2026-11-01. Decision: **retain as an experiment; do not migrate the runtime on this evidence**.

Hypothesis: declarative validation can remove handwritten readers while preserving their supported behavior. This trial replaces Limits, FileComplexity, SourceSpan, FunctionComplexity and ClonePair with Pydantic 2.12.3 models. The candidate includes its own path/name validators and shared configuration; it does not borrow the existing validation helpers.

The existing readers and candidate agreed on acceptance and returned values for 5,404 deterministic cases, including missing/extra fields, wrong types, invalid paths, bounds and nested mutations. Exact diagnostics, direct constructor behavior and the remaining record families are outside this trial. This is differential evidence, not proof of feature equivalence.

| Implementation counted | Physical lines | AST statements | UTF-8 bytes |
| --- | ---: | ---: | ---: |
| Five existing classes, shared helpers excluded | 65 | 43 | 2,368 |
| Candidate including its support code | 61 | 40 | 1,588 |

The reduction is four physical lines (6.2%), three statements (7.0%) and 780 bytes (32.9%). Bytes alone overstate the apparent simplification. The existing shared helpers remain necessary for other records, so their deletion is not claimed. Integrating the existing API and diagnostic compatibility could require additional code. Five packages were installed in the disposable uv environment; the project's dependency declaration and runtime were unchanged.

This is insufficient evidence of substantial simplification. A broader validation replacement may amortize shared support, but must earn adoption through a new measured trial; savings cannot be extrapolated from these five classes.

`result.json` records the exact baseline module hash, corpus size, library version, counts and separate production/test footprint. Physical lines count all source lines; AST statements are a separate syntactic count, not LLOC. Byte counts use UTF-8 text with normalized newlines, rather than raw checkout bytes. The trial driver and normal automated tests are test code, separate from the hypothetical production candidate. Fixtures, third-party dependencies and documentation are not counted as production source. Installed dependency size and maintenance effort were not measured.

From the repository root in PowerShell, reproduce with:

```powershell
$env:PYTHONPATH=(Resolve-Path src).Path
uv.exe run --no-project --python .venv-task11/Scripts/python.exe --with pydantic==2.12.3 python docs/experiments/leaf-contracts/trial.py
```

Use an available Python 3.12 interpreter path in place of the local venv on other systems. Only Windows was exercised here. The script overwrites its own result.json. Pydantic's [validation documentation](https://pydantic.dev/docs/validation/latest/api/pydantic/type_adapter/) describes strictness and extra-field handling; defaults must not be assumed equivalent to Lesscode's contract.
