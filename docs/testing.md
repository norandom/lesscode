# Parallel test runners

Use cargo-nextest for the native Rust core and pytest-xdist for the retained
Python reference package. These are development tools, not product dependencies.

## Rust

```powershell
cargo +1.98.1 install --locked cargo-nextest --version 0.9.143
cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --test-threads 4 --no-fail-fast
cargo +1.98.1 test --locked --manifest-path native/Cargo.toml --doc
```

Nextest runs tests in separate processes. Its worker count is independent of
Cargo build jobs. Keep the separate doctest command: nextest does not replace
that coverage. The native CI uses these commands and retains their output.
[Nextest usage](https://nexte.st/docs/running/)

## Python

Use the explicit existing uv interpreter described in [runtime setup](runtime-setup.md).
For this checkout:

```powershell
uv.exe pip install --python .venv-task11/Scripts/python.exe --require-hashes -r requirements-test.lock
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python -m pytest -n 4 --dist loadfile -ra --durations=10
```

The test dependency group pins pytest and pytest-xdist; the hash-locked file pins
their resolved dependencies across supported platforms. Refresh it intentionally:

```powershell
uv.exe pip compile --group test --python-version 3.12 --universal --generate-hashes --output-file requirements-test.lock
```

Pytest collects existing unittest classes and subtests. File-level distribution
keeps tests from each file in one worker, including their setup/teardown. Four
workers is an explicit starting point; use `-n 0` for serial diagnosis. No test
assertions or platform skips are removed. The configured collection is `tests/`
with `src/` on the import path. The original unittest command remains available
for historical replay and frozen trials.
[Pytest unittest support](https://docs.pytest.org/en/stable/how-to/unittest.html),
[xdist distribution](https://pytest-xdist.readthedocs.io/en/stable/distribution.html)

## Frozen experiments

For the shared native check script, local WSL Dagger commands and Actions
artifact semantics, see [Rust CI validation](ci-validation.md#rust-core-dagger-and-actions).

Do not change commands or dependencies halfway through an admitted trial.
Existing self-application runs retain their frozen unittest commands. A new
parallel experiment records its runner versions, worker count, collection,
command and outcome. Verify collected test identities and counts before adopting
a runner change. Timing reported here concerns test execution, not the product's
code-compression acceptance criteria.

For isolated copies beneath this checkout, explicitly select each copy's
configuration, root and source path. Otherwise pytest may discover the parent
checkout's configuration and test the wrong package:

```text
python -m pytest tests -c SIDE/pyproject.toml --rootdir SIDE -o pythonpath=SIDE/src -n 4 --dist loadfile
```

Use absolute side paths and record/assert actual imported module locations in
each worker when admitting a behavior-preservation trial. Equal collection counts
alone do not establish that the intended implementation was executed.
