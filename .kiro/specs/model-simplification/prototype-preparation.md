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
