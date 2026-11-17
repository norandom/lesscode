# Prototype preparation

2026-11-17. Owner approved the bounded protocol after commit 09959f6. This
authorizes preparation and execution within its stated limits, without another
permission request for those same actions. Production eligibility remains not-run.

Initial host checks: Cargo 1.98.1, uv 0.13.5, Git 2.56.0.windows.3; native Windows
Podman client 5.8.3 reaches its container service 5.8.9. No development code was
executed directly in WSL. CARGO_TARGET_DIR, UV_CACHE_DIR and TEMP resolve below
D:\BuildArtifacts. These checks establish tool availability, not a working specimen
runner, admitted measurement profile or behavioral baseline.

Preparation artifacts: `D:\BuildArtifacts\lesscode\model-simplification\ms1\preflight\`:
`source-inventory.json` records tracked specimen file paths, sizes and raw SHA-256
hashes; `podman-info.json` records the host/container service probe. This broad source
inventory is not the selected model context or complete experiment freeze. No source
content has been sent to a model.

Still required: isolated spike, qualified measurement profile, observation pins,
runner overlay and image identity, exact commands/model settings, scope/control
inventories and complete freeze. Then execute the already-authorized protocol.
No tests, builds, candidate calls or prototype trials ran during this initial preflight.

## Runtime preparation, later on 2026-11-17

[R116](../../../docs/research/2026-11-17/03-model-simplification-runtime-preparation.md)
records the prepared container and 47-test offline collection. Source lock and
pyproject hashes remain unchanged. P0 is still incomplete: the Rust spike, metric
profile, observation pins and complete freeze remain outstanding. No task group
completion or behavioral qualification is claimed.

## Census and initial observation pins

[R117](../../../docs/research/2026-11-17/04-model-simplification-census-pins.md)
records the research Rust census (four nextest checks), 26 passing proposed pins
on unchanged source and two detected known-bad controls. Measurements remain
not measurable: binding/alias resolution and atomic predicates are not implemented.
Remaining P0 work includes profile qualification, broader observations/controls,
private Git step integration and the complete manifest. No model calls ran.

## Predicate and binding increment

[R118](../../../docs/research/2026-11-17/05-model-simplification-predicates-bindings.md)
records twelve passing nextest checks, lexical owner identities, predicate work and
conservative L/W/U observations. Aliases and dispersion remain unresolved; real
reports still say not measurable. Apparatus is 542 cumulative maintained lines.
P0 remains open; no candidate calls or production task completion are claimed.

## P0 completion amendment, 2026-11-17

[R119](../../../docs/research/2026-11-17/06-model-simplification-p0-freeze.md)
closes preparation and input freezing. The owner added 60 minutes, giving 120 total
and a 14:57 Europe/Berlin preparation deadline. No other cap changed.

The [receipt](../../../docs/experiments/model-simplification/freeze-receipt.json)
identifies the verified `ms1/freeze-1` manifest: 278 inputs and 5,514 indexed evidence
files. Apparatus is 1,344 canonical maintained lines and pins are 336 lines.
Final nextest: 21 passed. The sealed, offline, unprivileged container passes all
96 selected tests twice and rejects five injected behavioral faults. Boundary,
integrity and stub lifecycle controls have retained evidence. Earlier failed
preparation attempts remain recorded; their effort is not refunded.

P0 is complete; P1 is pending. The structural facts remain conditional on explicit
contracts, not admitted measurement evidence. P1 must run the four frozen native
step baselines, complete control qualification and retain numeric ceilings before
any live call. No model endpoint was called. Design/tasks/production approval and
the overall solution-unlock result remain unchanged and pending.
