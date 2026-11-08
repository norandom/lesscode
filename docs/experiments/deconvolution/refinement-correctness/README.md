# R39 correctness probes

These probes assert required behavior and **fail on the uncorrected a755712
implementation**. They are research evidence outside the default production test
inventory, not a passing feature gate.

Run from the repository root in native Windows PowerShell:

```powershell
& docs/experiments/deconvolution/refinement-correctness/run.ps1
```

The runner uses the existing specimen Python environment and installed Rust
toolchain, makes a fresh `.runs/r39-*` directory, runs an isolated Rust test crate
against the actual native library using nextest, then Python probes through
pytest-xdist. It preserves prior runs and returns nonzero on probe failure.
No model service is called. `results/` contains the retained 2026-11-08 run;
new executions do not overwrite it.

- [Report](../../../research/2026-11-08/06-refinement-correctness.md)
- [Corrective requirements](../../../../.kiro/specs/refinement-evidence-integrity/requirements.md)
- [Adjusted concept](../../../../.kiro/specs/refinement-evidence-integrity/concept.md)

Existing-suite corroboration commands (same clean PATH and PYTHONPATH as runner):

```powershell
rustup run 1.98.1 cargo nextest run --locked --manifest-path native/Cargo.toml --test regenerator --test-threads 4 --no-fail-fast
uv.exe run --no-project --no-python-downloads --python .runs/e2-facdrone/venv/Scripts/python.exe python -m pytest tests/test_regenerator.py -n 4 --dist loadfile -q --tb=short
```

Do not infer full-repository or cross-platform validation from these commands.
Actual runtime versions and source hashes are in `results/provenance.json`.
