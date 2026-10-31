# Lesscode baseline

Compare a supplied Python baseline and candidate with local, pinned open-source
measurements and declared behavior checks. No LLM service is required.

Use uv to manage the pinned CPython 3.12.10 environment; see
[runtime setup](docs/runtime-setup.md). Evaluation uses explicit recorded runtime
paths after setup.

See the [CLI reference](docs/cli.md) for explicit installation, complete plan/run
examples, installed-wheel usage, reports and exit codes.

The Python evaluator has completed all 33 reviewed implementation tasks and
passed its [final validation gate](.kiro/specs/efficiency-baseline/validation.md). See the
[readiness evidence](.kiro/specs/efficiency-baseline/readiness.md),
[all 56 acceptance criteria](.kiro/specs/efficiency-baseline/requirement-coverage.md)
and [native Windows/Linux/macOS validation](docs/ci-validation.md).
The recorded [three-platform run](https://github.com/norandom/lesscode/actions/runs/41198054658)
passed the declared fixture checks; it does not establish general simplification
effectiveness. Configurable preventive agent rules, the optional simplifier
skill, rewrite engines and MCP integration are later features.
