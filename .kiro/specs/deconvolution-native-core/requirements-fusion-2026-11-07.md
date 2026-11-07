# Requirements Document

## Project Description (Input)

Who has the problem: the Lesscode owner, who wants to run the deconvolution loop (regenerate a scattered
responsibility behind its public surface and accept it on a recorded trace oracle) on real codebases written in
Python, TypeScript/JavaScript and Rust, on native Windows first and on Linux and macOS.

Current situation: the loop exists as research scripts in Python, assembled per experiment (about 1,400 lines of
apparatus per scope, the recorder rewritten per scope), proven on two Python scopes (R25, R26). Its agnostic
contract is stable; its language-bound parts exist for Python only, plus one Rust recorder (R20). See
[brief.md](brief.md) and [research.md](research.md).

What should change: a cross-platform Rust tool suite, one library with a reusable core plus CLIs, built from
existing crates and tools so that little code is written by hand; per-language and per-scope knowledge held as
data; instrumentation at the driver and store level rather than through debuggers; the effort and porting cost
per language visible in the specification before implementation, with spikes for every reuse choice that has no
evidence.

## Requirements

### Amendment 2026-11-07 (owner request) — draft from the arbitrated fusion plan

The OpenUltraCode fusion run of 2026-11-07 (strategy critique-revise-vote; panels
`ultracode-fusion-panel-a` and `ultracode-fusion-panel-b`; decider = the selected
opencode model `vibe-oidc/mistral-large-4` as coordinator and final arbiter) produced
and arbitrated the implementation plan. It is drafted here as requirements N1-N10.
**Status: generated, NOT approved.** No design, no tasks, no code. Supporting files:
[gates.md](gates.md) (1:1 gate map), [verification.md](verification.md) (E2E verification
of the .kiro workflow), [spikes.md](spikes.md) (S1-S9), [breath.toml](breath.toml)
(scope guard), plus project-level [../../breadth.toml](../../breadth.toml) (capability/
evidence map), [../../steering/complexity.md](../../steering/complexity.md) and
[../../complexity-budgets.json](../../complexity-budgets.json) (complexity caps).

#### N1 — 1:1 gate coverage

Every artefact and gate of the `deconvolution-loop` spec (requirements 1-12, T1-T7-style
predictions, kill criteria in three groups, the four solver verifier gates, the turn
rule) shall map 1:1 to a native component and a named, fixture-replaying smoketest
([gates.md](gates.md)); no silent gaps; do-not-build items stay out. The mapping is
enforced mechanically by the `requirements_traceability` check. Smoketests replay
committed E3/E4 artefacts (insta 1.49.0) and run in seconds-to-minutes; live stores,
model endpoints and heavy jobs live only in spikes, gated on their verdicts.

#### N2 — CLI suite shape

Extend the existing crate `native/lesscode-core` as one crate with a lib target plus a
`lesscode` binary (tech.md "one library/CLI package"; the brief requires a reusable
core, not a bare binary). Twelve top-level subcommands — `surface`, `record`, `replay`,
`accept`, `measure`, `mutate`, `score`, `differential`, `residue`, `report`, `scope`,
`host` — with verbs/flags carrying gate coverage. Exit codes keep the recorded 0/1/2/3
convention with the preregistered decision table in [gates.md](gates.md) (stale result
file -> 1; solver gate refusal from insufficient evidence -> 2; completed-but-
inconclusive -> 1; never 0 outside acceptance). Legacy `Prepare`/`Compare` stay as
deprecated aliases. Scope knowledge is data (TOML scope descriptor + versioned JSON
artefact schemas via schemars 1.2.3 / jsonschema 0.60.2), byte-compatible with the
prototype where committed E3/E4 artefacts are regression fixtures. Cross-compile
matrix: x86_64-pc-windows-msvc (first-class; MSVC over GNU), x86_64-unknown-linux-gnu,
aarch64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin, released via
native-runner `cargo build --release --locked --target` + SHA256SUMS verifiable with
`scope verify`. cargo-zigbuild rejected (cannot produce the MSVC target);
cargo-dist rejected (CI/installer surface exceeds "simplest reliable").

#### N3 — Reuse-first library table

Every component shall name its crate/tool with the exact pinned version from
[research.md](research.md) (2026-11-06 surveys) and its STALE/UNIX/COPYLEFT flags:
serde 1.0.229, serde_json 1.0.154, sha2 0.11.1 (bump from pin 0.11.0, stated), fs4 1.1.0,
tempfile 3.27.0; sysinfo 0.40.1, process-wrap 10.0.1; ignore 0.4.34, similar 3.2.1,
diffy 0.5.2 (git2 avoided: GPL-2.0 C build); ast-grep-core 0.51.1 (bump from pin 0.51.0),
scip 0.10.0 + scip-python 0.6.8 / scip-typescript 0.6.1 / rust-analyzer scip
(stack-graphs archived: avoided); pinned formatter CLIs ruff 0.17.4, prettier 3.9.10,
rustfmt; cargo-mutants 27.1.0, cosmic-ray 8.7.0, StrykerJS 10.1.1 (mutmut/cargo-fuzz/AFL
UNIX-only -> Linux CI); proptest 1.12.0; CrossHair 0.0.113, Kani 0.68.0 (CI-only),
fast-check; reqwest 0.13.5 + sse-stream 0.3.0; clap 4.6.7 (bump from pin 4.6.0, stated),
figment/dotenvy STALE -> avoided; postgresql_embedded 0.22.0, postgres 0.19.14,
sqlparser 0.63.0 (pgtemp Linux-only: avoided); tabled 0.22.0, comrak 0.59.0, insta 1.49.0,
quick-junit 0.8.0, schemars 1.2.3, jsonschema 0.60.2; pgwire 0.42.1 for the L2 wire
proxy. Permissive licences only; licence audit recorded. Hand-written code is limited
to the agnostic decision logic, the driver shims, the rule packs (data), the engine
readers and the per-scope descriptors (data).

#### N4 — Instrumentation: driver-level L1, store-level L2, no debuggers in the core

Level 1: one generic JSON-lines driver shim per language (Python subprocess, Node
subprocess, Rust in-process), ~150-300 lines each, written once; returns and exceptions
come back to the driver. Level 2: store-level and language-agnostic — pgwire 0.42.1
wire proxy for PostgreSQL (TCP, works on native Windows) or sqlite3_trace_v2 (~10 lines
per language); the 6-line driver hooks remain the fallback. Debuggers (DAP/CDP/lldb-dap)
stay out of the core. The same corpus recorded twice shall be byte-identical without
masking, or the tool refuses to proceed (req 2.3). sqlx/rusqlite hooks have NO evidence
and are the disproof condition, not the plan (spike S1 tests the pgwire proxy).

#### N5 — Spikes before design approval

Every reuse choice without evidence gets a bounded, preregistered, receipted spike
([spikes.md](spikes.md), S1-S9) **before** design approval, per
[../../steering/prototype-first.md](../../steering/prototype-first.md): verdicts
proceed/revise/stop, dated verdict files under `docs/research/2026-11-DD/`; `-y` cannot
waive evidence. S1 (wire-level L2 byte-determinism on native Windows) runs first; S2
(receipt digest compatibility) second, since nearly every byte-for-byte smoketest
depends on it.

#### N6 — Complexity caps (machine-enforced)

Caps per [../../steering/complexity.md](../../steering/complexity.md) and
[../../complexity-budgets.json](../../complexity-budgets.json): 400 physical lines per
maintained Rust file, 300 logical units per file, 80 physical lines per function;
spec-level budget <= 3,500 hand-written lines for the first release (Python+Rust) and
<= 4,600 for all three languages; <= 12 top-level subcommands; per-component ceilings as
recorded. An executable checker over the inventory is a foundation task accepted before
any capped feature work (policy text alone is not enforcement); missing scope input
fails, never skips. A breach defers work; it never pads or compresses code to pass a
counter. Exceptions require a reviewed, dated, expiring entry with measured baseline.

#### N7 — Scope guard (breath discipline)

[breath.toml](breath.toml) in this directory is the only way to change scope:
[features.in] lists what is built, [features.out] records every rejection with its
reason and date. Staying in scope is a deliberate decision, not an accident. The
project-level capability/evidence ledger is [../../breadth.toml](../../breadth.toml).

#### N8 — End-to-end verification of the .kiro workflow

The workflow itself is verified end-to-end by a CLI-driven runner
([verification.md](verification.md)): one phase at a time, serial, stop-on-first-failure,
fail-closed gate checks (`requirements_traceability`, `spike_verdicts_complete`,
`design_evidence_coverage`, `task_smoketest_coverage` over a committed machine-readable
evidence registry), per-task independent review with named smoketests on both trees,
receipt/hash verification at every step, `evidence.json` per step, final validation =
full fixture-replay suite green on native Windows + five-target CI matrix + caps pass.
No browser, no Playwright. AI testing capabilities: model-generated probe suites from
plan+diff+two trees only (the E4 128-probe review pattern), every probe receipted;
the missing-test report drives test writing in the TDD workflow (the loop reports, it
does not write specimen tests); held-out corpora with fresh seeds are the trust row.

#### N9 — Amendments pending owner ratification

The four R26 amendments (surface = tree imports + recorder imports; turn parameters
preregistered with thinking-only turn = failed; hashes over bytes as stored; receipt
labels distinct beyond letter case) are **proposed, not adopted**; the handover forbids
starting them until the owner asks. They enter these requirements only as dated,
owner-approved items; spike S9 quantifies the surface delta to inform the decision.
Implement the approved req 1 as written until then.

#### N10 — Host and isolation rules

Native Windows first, plain user account, no symlink privilege or elevation, WSL CI-only;
never call bare `bash` from Python/PowerShell; never modify `D:\Source\facdrone`,
`D:\Source\mmhmem` or frozen copies; mutants only in separate copies under `.runs/`;
one heavy job at a time; serial runs for deterministic attribution with numeric thread
counts pinned to 1; adaptive deadlines scaled by a receipted host-load probe, never a
fixed wall-clock cap; receipts for every command (args, env, times, exit, load probe,
content hashes); failed gate iterations kept, never relabelled.

<!-- Further requirements will be generated/refined in the /kiro-spec-requirements phase -->
