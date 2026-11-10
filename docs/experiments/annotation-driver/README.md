# Annotation-driver prototype

Disposable prototype for the
[agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md)
gate. It is not a product package and nothing here is used by `lesscode-core`.
Research entries: [R56](../../research/2026-11-10/02-agnostic-annotation-driver-direction.md),
[R57](../../research/2026-11-10/03-annotation-driver-scope-admission.md),
[R58](../../research/2026-11-10/04-annotation-driver-validator-controls.md).

| File | Purpose |
| --- | --- |
| `scopes.json` | The four admitted scopes and their revisions |
| `tables/<language>.json` | Everything language-specific: comment and string kinds, which syntax nodes each annotation kind may sit on |
| `validator.py` | Accepts or rejects annotations; classifies a mutant as changed, no-op or unparseable. No model |
| `controls.json`, `run_controls.py` | Hand-made control annotations and their runner (gate checklist step 2) |
| `results/` | Retained receipts |
| `compose.yaml` | Linux-side tools (Maven for the Java scope) |

Scopes are isolated clones outside this repository. To reproduce the
controls from the repository root on Windows:

```powershell
$env:UV_PROJECT_ENVIRONMENT = 'D:\BuildArtifacts\lesscode\annotation-driver\venv'
cd docs\experiments\annotation-driver
uv sync
uv run python run_controls.py --scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes --out results\controls-receipt.json
```

The clones are made from the revisions in `scopes.json`; R57 lists the
baseline test command for each. The files in this directory are frozen by
hash in R58 and stored with LF line endings. Changing them after the first
model call needs a dated amendment to the gate.
