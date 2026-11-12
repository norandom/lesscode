# Research log: model-seam

Discovery and synthesis for the design, 2026-11-12. Dated evidence is under `docs/research/2026-11-12/`.

## What exists (read in the code on 2026-11-12)

- `native/src/regenerator.rs` holds the whole model path of the core: `load_endpoint` and `Endpoint`
  (style, address, model, key, optional reasoning level), the hand-written request (`openai_call`, `call`,
  `ureq_post` on pinned `ureq` 3.5.1), `request_kwargs` and `hash_set` (the request shape hashed into
  receipts), response parsing (`parse_result`, `prepare_candidate`), the receipt writer, and the
  Refine/BestOfN controller `run_refine_loop`, which already takes the model call as a function
  (`ModelCall`, `default_call_fn`).
- `ureq` has no other user in the core. `tools/guide-optimizer` imports `ureq_post` from the core and has
  its own request building, reply reading, wait on HTTP 429 and repair turns.
- The release workflow builds `lesscode-core` for five targets: Linux x86_64 and aarch64, Windows x86_64,
  macOS x86_64 and aarch64.
- The step flow (`native/src/step/`) and the driver (`native/src/adapters/driver/`) contain no network code.

## Findings that shape the design

| Finding | Source | Consequence |
| --- | --- | --- |
| The hand-written request fails on three vendors for four reasons | R102 | adopt a client library; no vendor rule of ours |
| `genai` 0.7.4 completes the same turn on GLM 5.3, Mistral Large, `gpt-6.1-sol`, and Mistral Large through the local endpoint | R103 and the run of 17:43 | candidate library; others still to compare |
| The library reports HTTP 429 with the endpoint's message and does not wait | run of 17:43 | waiting with an estimate is ours to build on top |
| 293 crates in the prototype's lock file against 115 in the core's; async runtime; TLS by `rustls` (C build) or by the platform's TLS (`native-tls` feature) | R103, the library's manifest | separate crate; TLS choice decided by the five-platform build |
| The core's controller takes the model call as a function | `run_refine_loop`, `ModelCall` | the core can call the seam without linking it |
| GLM 5.3 returns the JSON object inside a code fence | R103 | the caller's parsing must accept a fenced reply |
| A fresh repair call with the check's reasons brings GLM from 7 to 19 of 20 | R99 | the repair loop is part of the seam, with the check supplied by the caller |
| A failed turn prints nothing | R102 | reasons go to the terminal by the seam, not left to each caller |

## Synthesis

- **Build or adopt**: adopt for the request and reply (the part that differs between vendors and changes
  without notice). Build what no library tried here does: waiting with an estimate, the check command, the
  repair loop around a caller's check, the record of calls.
- **Generalisation**: the two existing clients (core, research tool) become callers of one call. The four
  failures of R102 are not four fixes but one rule: send only what is stated, read what comes back in any
  shape.
- **Simplification**: the core loses its HTTP code and its `ureq` dependency. The core reaches the seam
  through a process boundary (one request in, one reply out), so it links nothing of the library.
- **Not built**: streaming, tool calls by the model, prices, model ranking, a plugin system for clients.

## Options considered for how the core reaches the seam

| Option | For | Against | Decision |
| --- | --- | --- | --- |
| The core links the seam crate | one binary | the core takes on about 180 crates and an async runtime; contradicts the brief | no |
| `regenerate` moves into the seam's binary | core entirely free of model code | breaks the command line the receipts and docs name | no |
| The core starts the seam's binary for each call | core keeps its command line and receipts, links nothing | a second binary to install; one process start per call | chosen |

## Open before design approval

Listed with their falsifiers in [prototype-gate.md](prototype-gate.md): the library comparison, the build
on five platforms, recognising and estimating a wait, the process boundary on the six first cases.
