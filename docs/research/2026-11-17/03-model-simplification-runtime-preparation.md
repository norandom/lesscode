# R116: The frozen specimen environment can be prepared and collected offline

- Report date: 2026-11-17, Europe/Berlin.
- Execution interval: 12:57 onward; exact command receipts remain in the session.
- Status: completed for runtime preparation; P0 remains incomplete.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Hypothesis: H-MS1 setup prerequisite; predecessor [R115](02-model-simplification-design-gate.md).
- Type: exploratory setup verification, not a simplification trial.

## Decision

The isolated Python runtime is usable for test collection. Continue P0 with the
Rust spike, measurement profile and observation pins. This result does not qualify
behavior, metrics, useful simplification or adoption.

## Hypothesis and falsification

Check whether the frozen source lock installs in the available Python container
and the two selected test modules collect with networking disabled. A failed install,
changed source lock or collection error would reject this setup configuration.

## Frozen inputs and method

Source revision remains `3a214ff13ee768535481f71258ac2b3529306870`; created a Git
archive without modifying the original checkout. Base image digest:
`sha256:60936acc2dfbd8db1ebdc2e841000aeee1a86394df6a09f96eb99d45a36038c1`.
Python 3.12.12, uv 0.9.7; `uv sync --frozen --group dev` installed 122 packages.
Added only runner overlay pytest-xdist 3.8.0 and execnet 2.1.2; pytest is 9.1.1.
The upstream pyproject/lock hashes match the protocol after installation.

Prepared image ID:
`c2d586c24d16ae0e9d614d53be8bdb50fab61ad83a0a4af40abf63c510473a64`.
Artifact directory: `D:\BuildArtifacts\lesscode\model-simplification\ms1\runtime`.
It contains source.tar, installed.txt, image-id.txt and collection.log. The initial
container remains stopped as `lesscode-ms1-prep`; the prepared image is retained.
[Preparation script](../../experiments/model-simplification/prepare-runtime.ps1)
records the commands with explicit exit checks; it has not been replayed end to end.

## Evidence and results

The first container start failed because `/work` did not exist. Created the directory
by copying an empty directory before start, then extracted the archived source.
No gate or dependency constraint was weakened to fix this setup error.

Offline `pytest --collect-only -q tests/test_exporter_resume.py tests/mcp/test_tools.py`
returned exit 0 and collected 47 tests. No test bodies or model calls were run.
No original tracked files changed. Native Windows Podman commands performed all
container operations; no development command ran directly inside WSL.

## Limits and interpretation

This is neither the full suite inventory nor the full experiment freeze. The newly
required alias/callback/error-order observations, mutation controls, qualified metric
profile and Rust orchestration are absent. The local image is retained evidence;
availability must be rechecked before later use. Package listing is an environment
receipt, not a new upstream dependency lock.

## Tool/spec consequence and next gate

P0 remains in progress. No production task group is declared completed. Continue
with executable measurement and observation fixtures, then complete and verify the
approved protocol's freeze before model trials. Keep the 1,400-line apparatus budget;
the preparation script contributes to it.

## COBESY check

Setup success is separate from behavioral and structural outcomes. The failed start
and missing evidence are explicit. No effectiveness or adoption claim follows.
