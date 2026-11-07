# First slice: reusable recording and acceptance

Written 2026-11-07; amended 2026-11-07 after library and complexity audit.
Provisional design proposal; documentation authorization only.

## Hypothesis
Shared recording/acceptance apparatus plus explicit scope adapters can cover E3
and E4 with less complete maintained implementation (`>= 20%` and `>= 200` fewer
canonical lines on the replaceable apparatus/wiring stratum and `>= 200` fewer
canonical lines overall) and unchanged observations.

## Proposed boundary
Reuse the library in `native/` and preserve `lesscode-core` prepare/compare and
existing `Cargo.lock` pins (`ast-grep-core = 0.51.0`, `serde = 1.0.229`,
`serde_json = 1.0.154`, `sha2 = 0.11.0`, `clap = 4.6.0`, plus zero-cost transitive
crates already in `Cargo.lock` such as `ignore = 0.4.34`, `walkdir = 2.5.0`,
`regex = 1.13.1`, `toml_edit = 0.25.17`, `windows-sys = 0.61.2`).
Do not add `serde_jcs` (breaks Python `-0.0`/`0.0` float and `ensure_ascii` digest
parity), `fs4` (redundant with Rust 1.98 `std::fs::File::lock` and `.lock` sidecars)
or `quick-junit` (writer-only, cannot parse XML).

Add only artifact admission (with explicit `--artifact-root` binding for untracked
`.runs/` goldens verified against committed `.sha256` sidecars), receipt verification
(with a Python-compatible `serde_json` formatter), acceptance/residue and the pure
turn-input transformation (`build_prompt` / feedback validation). Exercise one shared
Python subprocess recorder/replayer and arm runner with the specimen's pinned
interpreter and existing SQLAlchemy `before_cursor_execute` hook. Count descriptors
and thin adapters alike across the three strata in [prototype-gate.md](prototype-gate.md);
do not promise a data-only language for arbitrary specimen `op_*` methods.

Use frozen E3/E4 originals, existing candidates and planted controls. Do not
regenerate or adopt candidates. Imported sensitivity must match candidate/protocol
identities; missing evidence stays missing. Preserve query-differential semantics;
unsupported L2 differences refuse acceptance, never silently fall back to L1.

## Sequence
1. Prepare a bounded plan; S9 compares surfaces using existing apparatus before
   owner ratification. Evidence gathering and contract adoption are separate.
2. S2 establishes available artifact bytes (including `.runs/` goldens vs. committed
   `.sha256` sidecars) and Python-compatible JSON digest parity in Rust.
3. S10 builds bounded shared support on E3, freezes it, and onboards E4.
4. Independently review U1-U7. Only proceed permits normal production design review.
5. Approved production tasks begin with concrete inventory/checker foundation
   (using `syn` + `proc-macro2` already in `Cargo.lock`) and explicit disposition
   of pre-existing >400-line files in `native/src/`.

## Deferred expansion
Rust/mmhmem SQLite (`rusqlite` with `bundled`, `trace`, `session`) is a later
language/store proposal with L2 unproven. TS/JS (`oxc_parser`/`oxc_semantic`),
wire proxy, indexers, new engines, model HTTP client, twelve-command CLI and
five-target release expansion require separate need/evidence decisions.
Existing delivered compatibility and platform contracts remain protected.

## Cost and outcome
[prototype-gate.md](prototype-gate.md) proposes `<= 600` added/modified canonical
maintained lines of shared + adapter-wiring code across `<= 2` attempts (with
dedicated entrypoint or registration-only diff charging so `native/src/lib.rs`'s
618 existing lines and verbatim-migrated E3/E4 `op_*`/`SEEDS` blocks do not
deadlock the gate). These are limits to approve before running, not measured
results. Whole-replacement savings include retained/reused support across all
three strata.
[Research record](../../../docs/research/2026-11-07/01-solution-first-native-slice.md).
