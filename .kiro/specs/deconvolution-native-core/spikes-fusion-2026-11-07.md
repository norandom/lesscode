# Spikes: deconvolution-native-core

Written 2026-11-07 (owner request), from the arbitrated fusion plan. Prototype-first
steering applies: every reuse choice without evidence gets a bounded spike **before**
design approval; verdicts are `proceed` / `revise` / `stop`; `-y` cannot waive
evidence; every spike is preregistered and receipted (args, env, times, exit, content
hashes) with a dated verdict file under `docs/research/2026-11-DD/`. Spikes are
throwaway/verdict-bearing, not production tasks. S1 runs first; S2 second (it gates
nearly every byte-for-byte smoketest).

| # | Question | Method | Stop condition | Verdicts | Unblocks |
| --- | --- | --- | --- | --- | --- |
| S1 | Are two wire-level Level-2 recordings of the frozen E4 original byte-identical on native Windows (psycopg prepare thresholds, type lookups, pool resets)? | pgwire 0.42.1 proxy (TCP) in front of the E4 store; record the 50-sequence corpus twice; byte-compare; compare against the SQLAlchemy-hook record. Preregistered fallback ladder: driver hooks → sqlite3_trace_v2 → L1-only with gap declared. postgresql_embedded 0.22.0 Windows check folded in. | Byte-identical after documented normalisations, or disproof recorded and the fallback tested to the same bar | proceed / revise / stop | recorder design, L2 alphabet, `record`, `differential --query` |
| S2 | Does the Rust receipt layer (50-line serialiser, else serde_jcs 0.2.0) reproduce the Python prototype's canonical digest byte-for-byte on committed E4 receipts? | Re-emit all committed receipts with the Rust layer; insta-compare | All receipts byte-identical, or the chosen serialiser passes the same comparison | proceed / revise | receipt layer; every byte-for-byte smoketest |
| S3 | Does ast-grep-core 0.51.1 (+ Rust/JS grammar features) derive the E4 surface and measures matching committed values via YAML rule packs? | Run rule packs over frozen E4 trees; compare to committed surface/measures artefacts | Match on all readings, or the grammar gaps are recorded and the design states hand-written pattern support | proceed / revise | `surface`, `measure` |
| S4 | Do scip-python 0.6.8, scip-typescript 0.6.1 and rust-analyzer `scip` resolve cross-file names correctly per indexer? | Run each indexer on the specimen scopes; verify per indexer | Each indexer verified independently, or dropped from the design with the gap stated | proceed per indexer / revise | cross-file surface derivation |
| S5 | Are cargo-mutants 27.1.0 and StrykerJS 10.1.1 stable and sampleable on native Windows (isolated copies under `.runs/`)? | Run both on toy specimens; read composition; seeded sample | Stable JSON + composition read, or the rung is deferred with the gap stated | proceed / revise | `mutate` readers |
| S6 | Do sysinfo 0.40.1 + process-wrap 10.0.1 (Job Objects) enforce one-heavy-job, wait-for-idle and the memory floor on native Windows as a plain user? | Sandboxed guard tests: attempt a second heavy job, simulate load probes | Enforcement observed and the probe/scale/wait port behaves, or the guard design is revised | proceed / revise | `host guard` |
| S7 | Does a small reqwest 0.13.5 + sse-stream 0.3.0 client reproduce E4's req 6.8 receipt fields (hashed prompts/responses/model/seed)? | Stub-server replay of E4's recorded interaction shape | Receipt fields byte-compatible with committed E4 receipts | proceed / revise | turn-program HTTP client (owner-gated) |
| S8 | Does the full dependency tree build on all five release targets, tree-sitter C builds included? | `cargo build --release --locked --target` per target on native CI runners | All five targets build, or the failing dependency is swapped/flagged | proceed / revise | release matrix |
| S9 | What is the E4 surface delta with vs without recorder imports (the unadopted R26 amendment that run-report §7 already assumes)? | Run `surface derive` both ways on the frozen E4 copy | Delta quantified and reported to the owner | informs owner ratification | surface deriver finalisation (owner-gated) |

Adopted from panel A's round-1 proposal: a bounded symbolic-rung spike (Kani 0.68.0 /
CrossHair 0.0.113 / fast-check on planted mutants) slots after S5, with the package
constraints intact: Kani is Linux/macOS CI-only; the TS/JS symbolic gate is declared
unmet, not tested into existence.

Host rules during spikes: native Windows only, plain user; one heavy job at a time;
serial pytest for deterministic attribution; BLAS threads = 1; mutants only in
separate copies under `.runs/`; never modify `D:\Source\facdrone`, `D:\Source\mmhmem`
or frozen copies.
