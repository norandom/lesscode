# Research notes: deconvolution-native-core

## Design discovery (2026-11-07, production design phase)

Owner approved requirements (opencode chat, "Approve & proceed") and selected
handover option 1: production design via normal gates. Discovery type:
extension (light) — productionizing a demonstrated prototype into existing
systems.

Consulted: kiro-spec-design skill rules (principles, discovery-light,
synthesis, review-gate), design/research templates, steering
(product/tech/structure), `src/lesscode_baseline/AGENTS.md`, spec files
(requirements, slice-proposal, prototype-gate, prototype-plan,
requirements-review), demonstrated code (`proto_core.py` full surface,
`scope_e4.py` full, `admission.rs` CLI/verify surface, `turncheck.rs` at
`f6bb6e2`: 207 lines + 72-line test, `main.rs` dispatch L50-51/L216-220,
`Cargo.toml`), upstream deconvolution-loop requirements (3.2, 6.7, 8.7, 9.5,
10.2), U1-U7 receipts.

Integration findings:
- `admission` is already a `lesscode-core` subcommand; its CLI contract is
  the compatibility baseline (5.1), preserved unchanged.
- `proto_core` importers: `scope_e3.py`, `scope_e4.py`, `s10_controls.py`,
  `s10_controls2.py`; measure drivers name it in frozen `PY_APP` lists —
  measure drivers stay untouched (frozen attempt evidence), so relocation
  updates only adapters/controls imports and deletes `proto_core.py` without
  a shim (no duplicated maintained lines, no pass-through layer).
- `complexity-budgets.json` is referenced by the requirements review but
  absent from the tree — it becomes a new committed artifact owned by this
  spec at `native/complexity-budgets.json`.
- `src/lesscode_baseline` is a flat module package with an evidence-contract
  AGENTS.md; a single `recording.py` module fits without a package dir.
- Adapter placement: specimen-bound wiring stays in the experiments tree next
  to its frozen scopes; moving it into the product package would couple the
  product to specimen paths (rejected alternative, recorded here).

Synthesis outcomes:
1. Generalization: descriptor→checks→LF receipt→exit-code is a shared
   contract shape across `admission`/`turncheck` (not a framework); the
   Python side generalizes the adapter interface (seeds/meta_of/record/
   replay/accept) already satisfied by both scopes.
2. Build vs adopt: adopt std fs locks, serde_json + custom ~40-line Formatter
   (research rows 2026-11-06/07), syn+proc-macro2 already in `Cargo.lock`
   for the checker (promote transitive to direct at locked versions); reject
   debuggers/proxies (Amendment 1 finding); no new external dependencies.
3. Simplification: one Python module, not a package; no shim on relocation;
   frozen drivers untouched; budget checker ~260 lines instead of a generic
   complexity platform; `receipt.rs` gives byte rules a single owner used by
   two commands.

Design decisions recorded in `design.md`; production size allocation proposed
there and to be frozen at task generation.

## Amendment 2: library and complexity audit (2026-11-07)
Empirical baseline audit (`ruff==0.17.3 format --line-length 100`) and inspection of
`native/Cargo.lock` and `native/src/` revised the reuse map and complexity estimates:
1. **Keep existing `Cargo.lock` pins for the first slice**: `ast-grep-core = 0.51.0`,
   `serde = 1.0.229`, `serde_json = 1.0.154`, `sha2 = 0.11.0`, `clap = 4.6.0`.
   Promote transitive crates already present in `native/Cargo.lock` (`ignore 0.4.34`,
   `walkdir 2.5.0`, `globset 0.4.21`, `regex 1.13.1`, `thiserror 2.0.21`, `indexmap 2.14.2`,
   `toml_edit 0.25.17`, `syn 2.0.119/3.0.9`, `proc-macro2 1.0.107`, `windows-sys 0.61.2`)
   before adding new external dependencies.
2. **Dropped / replaced crates from the 2026-11-06 survey**:
   - **Drop `serde_jcs 0.2.0`**: RFC 8785 serializes `-0.0` and `0.0` as `0` and emits
     raw UTF-8, breaking `S2` byte-for-byte digest parity with Python's
     `json.dumps(sort_keys=True)` (`ensure_ascii=True`, `-0.0`/`0.0` preserved, `indent=0/1`).
     Use `serde_json::ser::PrettyFormatter::with_indent` plus a ~40-line custom
     `serde_json::ser::Formatter`.
   - **Drop `fs4 1.1.0`**: Rust 1.89+ (`rust-version = "1.98.1"` in `native/Cargo.toml`)
     stabilized `std::fs::File::lock()` / `try_lock()` in `std`, and `receipt_atomic.py`
     uses an `OpenOptions::create_new(true)` `.lock` sidecar.
   - **Replace `quick-junit 0.8.0` with `junit-parser` (or `quick-xml` with `serde`)**:
     `quick-junit` is an XML serializer/writer only and has no XML parser or `Deserialize`.
   - **Drop `gix 0.90.0`, `pg_query 18.1.0`, and `postgresql_embedded 0.22.0`**: `gix`
     adds 150+ crates for unneeded git plumbing; `pg_query` compiles C `libpg_query`
     headers that break on `x86_64-pc-windows-msvc` (`sqlparser` is pure Rust); Facdrone
     already runs its own embedded PostgreSQL + Alembic chain via `e2_winshim.py`.
3. **High-impact additions to the reuse map (for deferred expansion and checker)**:
   - **`syn` (`visit`) + `proc-macro2` (`span-locations`)** (already in `Cargo.lock`):
     powers both the `rust-syntax-units-v1` foundation complexity checker and Rust
     specimen `surface`/`measure` AST walks without external indexers.
   - **`toml_edit 0.25.17`** (already in `Cargo.lock`): parses `breath.toml`,
     `.kiro/breadth.toml` and TOML scope descriptors with zero new dependencies.
   - **`ruff_python_parser` / `ruff_python_ast` (or `rustpython-parser`)**: pure-Rust
     Python `ast` parser matching Python's exact `ast.stmt` / `ast.Assign` / `ast.Compare`
     hierarchy (zero C build), preventing `tree-sitter-python` CST node-granularity
     mismatches on `surface.py`, `measures.py` and `measures3.py` (`S3`).
   - **`oxc` (`oxc_parser`, `oxc_ast`, `oxc_semantic`)**: pure-Rust TS/JS AST, symbol
     table and scope tree for the deferred TS/JS phase (replaces `scip-typescript`
     Node subprocesses).
   - **`rusqlite` (`bundled`, `trace`, `session`)**: for the deferred Rust/mmhmem SQLite
     scope; `trace_v2` captures L2 expanded SQL while the `session` extension (`Changeset`)
     captures exact L1 row-level table diffs automatically across transactions.
   - **`arbtest` / `bolero`**: `arbtest` provides a ~200-line deterministic property
     runner directly on `arbitrary`; `bolero` unifies `proptest` (Windows) and
     `libfuzzer`/`kani` (Linux CI) behind one harness macro.
4. **AST walking vs. YAML rule packs**: `surface.py` (per-file module-alias table for
   `alias.attr` and `monkeypatch.setattr`), `measures.py` (enclosing-function `pure_share`
   write-site check), and `measures3.py` (recursive paired-leaf tuple unpacking) cannot be
   expressed in declarative `ast-grep` YAML rules. Walk syntax trees in Rust code.

## Amendment 1: solution-first proposal (2026-11-07)
[R27](../../../docs/research/2026-11-07/01-solution-first-native-slice.md),
[requirements](requirements.md) and [gate](prototype-gate.md) supersede delivery
recommendations below: counted adapters allowed, hooks before proxy, broader
choices deferred. No new execution or dependency verification. Earlier surveys
retain their original dates and evidence limits.

Written 2026-11-06 from two web surveys run the same day (crates.io, PyPI, npm and GitHub APIs; project
docs). Versions and dates are as checked on 2026-11-06. Nothing here is approved; it feeds the brief and the
requirements. Evidence for the prototype itself: [run report](../../../docs/reports/deconvolution-loop-run-report.md)
section 7, [R25](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md),
[R26](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md), and the
[deconvolution-loop spec](../deconvolution-loop/design.md).

## 1. Finding that shapes the spec: instrumentation

The owner asked whether a debugger library can replace the hand-written recorder. Answer: technically yes for
Python, partly for JavaScript, weakly for Rust on native Windows, and it would add code rather than remove it.

The 534 lines of the E4 recorder (`e4_oracle.py`, 1,258 canonical lines under `ruff format`) are not interception
code. Level 2 capture is one SQLAlchemy `before_cursor_execute` listener of about 6 lines. Level 1 at the surface
needs no interception: the driver makes the call and sees the return value or the exception. Over 83% of
`e4_oracle.py` (1,048 of 1,258 canonical lines) is per-scope work that no interception mechanism removes: the
24-operation alphabet (`op_*`, 472 canonical lines), fixtures (160 canonical lines) and the 26 transcribed seed
sequences plus Hypothesis generator (470 canonical lines).

| Mechanism | Level 1 (calls, returns, exceptions) | Level 2 (SQL text and params) | Windows | Verdict |
| --- | --- | --- | --- | --- |
| Debug Adapter Protocol (debugpy, js-debug or CDP, lldb-dap) | Python good; JS needs CDP (js-debug has no function breakpoints); Rust on MSVC weak (PDB gaps, enum decoding, lldb-dap DLL issue) | only via breakpoints in driver internals | Python and JS yes, Rust weak | fallback only; 500 to 1,000 lines of client plus a serialiser per language; no Rust crate worth using (`dap` 0.4.1-alpha1 is 2023) |
| Direct debugger APIs (LLDB SB, GDB/MI, DbgEng), Detours/minhook | native frames only | at C boundaries (libpq, sqlite3) | partial | not recommended |
| Frida, DynamoRIO, Pin, TTD, rr | native only; no Python or V8 value bridge | C boundaries | rr Linux-only, TTD admin and Windows-only | not recommended |
| Runtime hooks: Python `sys.monitoring`, Node `module.registerHooks`, Rust in-process harness | yes, cheap, deterministic, no source change (Rust `tracing` would edit the specimen) | Python audit hook gives files, not SQL | all | recommended for calls inside the scope, if wanted |
| Store-level: PostgreSQL wire proxy in Rust (`pgwire` 0.42.1 codecs, hand-rolled passthrough) | n/a | every Parse/Bind/Execute with raw parameter bytes, in order, any language's driver | yes (TCP; the prototype's AF_UNIX socket directory must switch to TCP) | deferred; 200 to 400 lines; psycopg3 auto-prepare and `pg_type` traffic threaten byte determinism vs. 6-line hook |
| Store-level: `log_statement=all` with jsonlog | n/a | text exact, parameters re-rendered as text | yes | cross-check if proxy is ever built |
| Store-level: SQLite trace & session (`rusqlite` `trace_v2` + `session` changesets; Python `set_trace_callback`; `node:sqlite`) | `rusqlite::session` captures row diffs automatically | expanded SQL, same text in all three languages | yes | recommended for SQLite specimens, ~10-30 lines per language |
| OpenTelemetry auto-instrumentation | spans | parameters optional, stringified; nothing for Rust | yes | not recommended |

Recommended shape (as amended 2026-11-07):
- First slice (`S10`): reuse the Python subprocess recorder/replayer and 6-line SQLAlchemy hook with shared
  runner/arm orchestration and explicit counted scope adapters across the three strata ($B_{\text{app}}, B_{\text{ops}}, B_{\text{seeds}}$).
- Later languages: Rust in-process over `rusqlite` (`trace_v2` + `session`); Node via JSON-lines subprocess.
- Debuggers and TCP wire proxies stay out of the core unless a concrete scope demonstrates an unmet need.

## 2. Ecosystem reuse per component (amended 2026-11-07)

Flags: STALE = no release since 2025-05-06; UNIX = no native Windows; COPYLEFT = GPL, LGPL or MPL in the chain;
LOCKED = already present in `native/Cargo.lock`.

| Component | Reuse | Hand-written Rust | Per-language adapter | Notes |
| --- | --- | --- | --- | --- |
| Receipt & canonical digests | `serde_json` 1.0.154 (LOCKED), `std::fs::File::lock` / `OpenOptions::create_new` (Rust 1.98 std), `tempfile` 3.27.0 | ~150-250 lines | no | Drop `serde_jcs` (breaks `-0.0`/`0.0` and `ensure_ascii`) and `fs4` (in `std`); implement ~40-line `serde_json::ser::Formatter` matching Python `json.dumps` |
| Host guard, deadlines | `windows-sys` 0.61.2 (LOCKED: Job Objects + `GlobalMemoryStatusEx`), or `sysinfo` 0.40.1 + `process-wrap` 10.0.1 (deferred) | ~150-250 lines | no | `windows-sys` is already in `Cargo.lock` and covers Windows Job Objects + RAM probes in ~50 lines without pulling in `tokio` |
| Tree hash, edit scope, diffs | `ignore` 0.4.34 (LOCKED), `walkdir` 2.5.0 (LOCKED), `sha2` 0.11.0 (LOCKED), `similar` 3.2.1, `diffy` 0.5.2 | ~150-250 lines | no | Hash raw bytes as stored; drop `gix` (150+ crates) and `git2` (GPL-2.0 C build) |
| Surface and measures | Python: `ruff_python_parser`/`ruff_python_ast` (or `ast-grep-core` 0.51.0 LOCKED); Rust: `syn` 2.0/3.0 + `proc-macro2` (LOCKED); TS/JS: `oxc_parser` + `oxc_semantic`; pinned formatter CLIs (`ruff` 0.17.3/0.17.4, `rustfmt`, `prettier`/`oxfmt`) | ~600-850 lines | yes (AST walker per language) | AST walking in Rust code required for alias tracking (`surface.py`), `pure_share` (`measures.py`) and paired-leaf tuple unpacking (`measures3.py`); external SCIP indexers deferred |
| Complexity checker | `syn` (LOCKED, `visit` feature) + `proc-macro2` (`span-locations`) | ~200 lines | no | Powers `rust-syntax-units-v1` (300 logical units/file, 80 lines/fn) over `complexity-budgets.json` |
| Mutation | `cargo-mutants` 27.1.0 (Rust CLI), `cosmic-ray` 8.7.0 (Python, proven in E3/E4), `StrykerJS` 10.1.1 (TS/JS); `diffy` 0.5.2 to apply hunks | ~350-450 lines | yes (JSON/SQLite reader per engine) | `mutmut` is UNIX (needs `fork`) |
| Sequence generation | `proptest` 1.12.0 or `arbtest` / `bolero` over `arbitrary` 1.5.0; Python Hypothesis in first slice | ~200-350 lines | yes (alphabet/ops per scope) | `cargo-fuzz`, `afl.rs` are UNIX (Linux CI only) |
| Solver rung | `CrossHair` 0.0.113 subprocess (Python); `Kani` 0.68.0 Linux CI only (Rust); none for TS/JS (`fast-check` 4.10.2 property differential) | ~250-350 lines | yes | Behind the 4 solver verifier gates; failed control voids differential only (loop req 9.5) |
| Model client (deferred) | `reqwest` 0.13.5, `sse-stream` 0.3.0 | ~200 lines | no | Pure turn-input prompt/feedback transformation runs first without HTTP transport |
| CLI, config & descriptors | `clap` 4.6.0 (LOCKED, derive), `toml_edit` 0.25.17 (LOCKED) / `toml` | ~350-450 lines | no | `toml_edit` is already in `Cargo.lock` for `breath.toml`, `breadth.toml` and scope descriptors |
| Query differential & SQLite | `postgres` 0.19.14, `sqlparser` 0.63.0, `rusqlite` (`bundled`, `trace`, `session`) | ~300-400 lines | store-bound | Drop `pg_query` (MSVC C build issues) and `postgresql_embedded` (Facdrone owns its PG server) |
| Reports, regression | `tabled` 0.22.0, `comrak` 0.59.0, `insta` 1.49.0 | ~300-350 lines | no | Covers `residue` (req 10.2), `missing-tests` (req 8.7), `turn` (req 6.7), `review-pack`, `budgets` |
| Arm results across languages | `pytest --junitxml`, Jest/Vitest JUnit, `cargo-nextest` 0.9.151 JUnit; **`junit-parser`** (or `quick-xml` + `serde`) | ~100 lines | no | Replaces `quick-junit` (which is a writer only and cannot parse XML) |
| Artefact schemas | `schemars` 1.2.3, `jsonschema` 0.60.2 | ~100 lines | no | Validate frozen formats at boundaries |
