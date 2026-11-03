# Handover: deconvolution research, state at end of 2026-11-03

Written 2026-11-03 evening, Europe/Berlin, for the agent continuing on
2026-11-04. Read this first, then the files it points to. Nothing is running.

## What to do tomorrow

Execute [E1c-2](../docs/experiments/deconvolution/e1c2-concolic-rerun/plan.md),
the corrected solver rung, as preregistered. Owner decision of 2026-11-03.
Dispatch an opus implementer with the plan, the E1c README and the rules
below; then write the dated entry R24 under `docs/research/2026-11-03/` or a
new `2026-11-04/` folder (writing date 2026-11-04), update
`docs/research/README.md` and the H8 lineage row, checkpoint mmhmem, and bring
the result to the owner with the next options: E3 regeneration preregistration
with the measured oracle sensitivity, or stop.

## Rules that are not negotiable

- Native Windows only, plain user account. WSL is CI-only; nothing executes
  there. Never call a bare `bash` from Python or PowerShell: Windows resolves
  it to the System32 WSL launcher. Use native Python, PowerShell, or the full
  Git Bash path. Tell every subagent this explicitly.
- No symlink privilege, Developer Mode or elevation assumed, by the research
  harnesses or by the desired implementation. Specimen tests that need them
  stay recorded as baseline failures.
- Never modify `D:\Source\facdrone`, `D:\Source\mmhmem`, or the frozen copies.
  Mutants only in separate copies under `.runs/`.
- One heavy job at a time; serial pytest for anything whose attribution must
  be deterministic; BLAS thread variables set to 1. The host ran out of
  memory once when cargo-mutants, a Facdrone pytest and an unrelated rustc
  build overlapped.
- Preregister before executing; amendments are dated and never change
  predictions after results; failed predictions are reported as failed.
- Research entries use `docs/research/ENTRY-TEMPLATE.md`, conclusion first,
  COBESY check at the end; update the index in the same change.
- mmhmem: call `session_resume_or_handoff` for project `lesscode` at start,
  checkpoint after each unit of work. Last checkpoint 2026-11-03 19:24 UTC.

## Where things stand, by hypothesis

H8 (recorded trace oracle + edge measures + regeneration), registered in
[R19](../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md):

| Part | Status | Entry |
| --- | --- | --- |
| H8(a) oracle, Rust (mmhmem session bus) | mechanics positive; gain prediction hit a 96% seed-test ceiling | [R20](../docs/research/2026-11-03/02-e1a-session-bus-oracle.md) |
| H8(b) edge-based selector (Facdrone census) | rejected as formulated; C2 and C5 failed; recorder validity proven | [R21](../docs/research/2026-11-03/03-e2-facdrone-edge-census.md) |
| H8(a) oracle, Python (Facdrone evidence publication) | byte-deterministic, catches stored-representation change; kill rate 0.712 to 0.746; gap-halving failed | [R22](../docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md) |
| Guided-generation rung (CrossHair on lifted predicates) | failed as run because drivers concretised strings and the oracle never calls the readers; one target killed by solver input | [R23](../docs/research/2026-11-03/05-e1c-concolic-rung.md) |
| H8(c) regeneration | untested; not admitted under the preregistered gates | pending |

Tool assessment for symbolic execution per language:
[docs/symbolic-execution-tooling.md](../docs/symbolic-execution-tooling.md).

## Environment facts

- Facdrone frozen copy `.runs/e2-facdrone/source` at `d49f48bd…`, venv
  `.runs/e2-facdrone/venv` (CPython 3.13.16, from the copy's uv.lock plus
  coverage 7.13.0). Hypothesis extras `.runs/e1b-facdrone/extras`; CrossHair
  0.0.112 and z3 in `.runs/e1c-facdrone/extras`. Mutant copy
  `.runs/e1c-facdrone/mut`.
- Facdrone embedded PostgreSQL runs natively only with the two shim fixes in
  `docs/experiments/deconvolution/e2-facdrone-edge-census/e2_winshim.py`
  (unquoted `listen_addresses`, detached pipes on `pg_ctl start`); load with
  `-p e2_winshim` and the e2 directory on `PYTHONPATH`. Gate marker expression
  is `not port_baseline and not live_broker`; `not db` would deselect the
  database seeds.
- 236 Facdrone tests fail on this host at baseline (symlink privilege, file
  locks, POSIX-only modules); recorded, not repaired; identical with and
  without the recorder.
- mmhmem isolated copy `.runs/e1-mmhmem/original` at `8a9ce348`; cargo-mutants,
  cargo-llvm-cov, nextest installed natively; no nightly, so no Miri or
  cargo-fuzz.
- `python` is not on PATH; use `uv.exe run --no-project --python 3.13` or the
  venv interpreter.

## Specimen findings not yet reported upstream

- mmhmem: `checkpoint` panics when the 6,000-byte working-state cap falls
  inside a multi-byte character (`truncate_to_budget`, `session_bus.rs:355`).
- Facdrone test support on Windows: quoted empty `listen_addresses` and
  pipe inheritance on `pg_ctl start`; `os.geteuid` call in the fixture plugin.

## Open items after E1c-2

1. E3 preregistration on the evidence-publication scope: regeneration behind
   the public API with the pure-core / effect-shell split required, the
   oracle (reader-inclusive alphabet) as acceptance, CrossHair differential on
   the pure core, the measured sensitivity declared as the protection level.
2. A corrected edge census (E2b) if a measured selector is still wanted:
   distinct codec sites, size-normalised measures, minimum executed size,
   worker-independent attribution.
3. Prevention at generation time remains the fallback if the retrospective
   track stops.
