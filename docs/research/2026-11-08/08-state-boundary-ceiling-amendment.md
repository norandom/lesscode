# R41: State-boundary apparatus ceiling amendment — owner disposition on prototype feasibility gate (option a)

- Report date/time and timezone: 2026-11-08, ≈19:00Z (commit timestamp authoritative)
- Execution interval: owner instruction ("a) it is") received following prototype gate qualification in opencode session on 2026-11-08; measurements performed live
- Status: completed
- Spec: [state-boundary-deconvolution](../../../../.kiro/specs/state-boundary-deconvolution/requirements.md) (`Req 3.5`, `5.5`, `6.1`, `6.2`, `6.5`); task 4.2; [prototype-gate.md](../../../../.kiro/specs/state-boundary-deconvolution/prototype-gate.md)
- Hypothesis ID and predecessor: [R40](07-structural-deconvolution-and-state-boundaries.md) (structural deconvolution & state boundary synthesis); [R39](06-refinement-correctness.md) (refinement evidence integrity); [R38](05-u5-ceiling-amendment.md) (U5 ceiling amendment to 7,004)
- Type: preregistered option exercised by dated owner decision (R38 outcome rule option (a))

## Decision

The owner chose **option (a) of the preregistered prototype gate outcome rule — amend the apparatus ceiling** (owner instruction, 2026-11-08 session: *"a) it is"*). Effective 2026-11-08:

1. **The Rust-substrate apparatus ceiling is amended from 7,004 to 7,173 canonical lines** — exactly the measured total on the extended counted set (`native/src/{regenerator,measures,surface,recording,jsonval,admission,receipt,state_contract}.rs` + shim + enumerated net hooks vs `f2fe982`). On the original R37 counted set without `state_contract.rs`, the measured total is 7,079 canonical lines (+75 lines). The amended ceiling of 7,173 canonical lines is a **no-growth cap**: any future modification must keep the apparatus total $\le 7,173$ under the unchanged R37 canonical method.
2. **Prototype feasibility gate is marked PASSED under the amended ceiling.** All 4 adverse controls (E3 `AuditedAbsence`, E4 F1 `Silent_None`, E4 F3 `Buffer_Drop`, E4 F7 `Error_Order`) achieved 100% kill rates, the density trap on Facdrone `lifecycle` was detected and defended, and the apparatus volume complies with the amended ceiling.
3. **Production rollout and downstream task execution remain gated.** Symmetric with prior ceiling amendments, this decision ratifies the necessary volume for causal state verification and R39 oracle fixes; it does not authorize unmeasured code expansion.

## Hypothesis and falsification

- **Hypothesis:** Software verification techniques (state frames, stream monitoring, bounded pure-core equivalence, and adverse mutation gates) eliminate oracle blindness on scattered state at a marginal apparatus cost of +169 canonical lines (+2.41%), comprising +74 lines in `regenerator.rs` (R39 correctness repairs and state obligation bridging), +1 line in registration hooks, and +94 lines in `state_contract.rs`.
- **Falsification Observed & Resolved:** The initial prototype run strictly breached the frozen 7,004 line ceiling (7,079 / 7,173 vs 7,004), triggering the preregistered `REVISE` stop condition. The overage was evaluated as necessary verification and correctness logic rather than unbounded apparatus bloat. The dated owner decision resolves the stop condition without lowering detection bars or falsifying test evidence.

## Frozen inputs and method

- **Specimen:** Facdrone `lifecycle` monitor (7 files, 1,495 canonical lines, 69 functions, 89 intra-scope call edges).
- **Canonical Method:** `rustfmt 1.98.1` (`--edition 2021`) + `splitlines()`; Python shim formatted via `ruff==0.17.3 format --isolated --line-length 100 --target-version py313`; hooks by enumerated net git-diff hunks vs `f2fe982`.
- **Evaluation Runner:** `.kiro/specs/state-boundary-deconvolution/run_gate.py` emitting `.kiro/specs/state-boundary-deconvolution/results/prototype-gate-report.json`.

## Evidence and results

### 1. Counted Set Breakdown

| Module | R37 Canonical | Post-Task 13 Canonical | Delta | Allocation |
|---|---|---|---|---|
| `native/src/regenerator.rs` | 2,716 | 2,790 | +74 | R39 oracle repairs (exit codes, delivery, feedback, obligations) + Task 3.2 state obligation bridge |
| `native/src/measures.rs` | 1,422 | 1,422 | 0 | Unchanged |
| `native/src/surface.rs` | 1,151 | 1,151 | 0 | Unchanged |
| `native/src/recording.rs` | 971 | 971 | 0 | Unchanged |
| `native/src/jsonval.rs` | 130 | 130 | 0 | Unchanged |
| `native/src/admission.rs` | 274 | 274 | 0 | Unchanged |
| `native/src/receipt.rs` | 124 | 124 | 0 | Unchanged |
| `native/shim/recording_shim.py` | 178 | 178 | 0 | Unchanged |
| Hooks (`lib.rs`, `main.rs`, `Cargo.toml`) | 38 | 39 | +1 | Registration of `state_contract` in `lib.rs` |
| **Subtotal (R37 counted set)** | **7,004** | **7,079** | **+75** | **+1.07% overage** |
| `native/src/state_contract.rs` | — | 94 | +94 | Lean native state contract and scope density evaluation structures |
| **Total Counted Apparatus** | **7,004** | **7,173** | **+169** | **+2.41% overage** |

### 2. Adverse Mutation and Verification Kill Evidence

- **E3 Serialization Exception (`AuditedAbsence`):** 100% verified. Missing audit post-condition flagged as `MISSING_FAILURE_INVARIANT`.
- **E4 F3 Stream Preservation:** 100% verified. Truncated pre-abort output (31 bytes dropped on `stdout`) caught and reported as `TRUNCATED`.
- **E4 F1 Pure Core Equivalence:** 100% verified. Bounded equivalence generated minimal counterexample `{"x": None}`.
- **E4 F7 Error Ordering:** 100% verified. All planted error-order-swap mutants killed by candidate test battery (0 survived).
- **Density Trap Defense:** Successfully caught. Structural consolidation (functions −24.64%, edges −29.21%, pure functions 51.92%) decoupled from compression (lines −6.02%, statements −1.67%), failing `compression_met` and preventing false compression claims.

## Limits and interpretation

1. The ceiling amendment absorbs only the minimal necessary logic for truthfulness (R39) and state boundary verification (R40/R41). It does not justify general feature creep.
2. The density trap defense confirms that structural de-fragmentation alone does not yield proportional line compression. True compression requires deleting redundant requirements or whole responsibilities, not merely repacking AST nodes into denser functions.

## Tool/spec consequence and next gate

- Specification `state-boundary-deconvolution` is fully implemented and qualified.
- The prototype feasibility gate transitions from `REVISE` to `PASS` under the amended 7,173 ceiling.
- With the state boundary verification apparatus proven and capped, future deconvolution trials can operate safely without oracle blindness on scattered state.

## COBESY check

- **Outcome Visible First:** The amended ceiling (7,173 canonical lines) and exact delta (+169 lines) stated upfront.
- **Replayability:** Fully reproducible via `.kiro/specs/state-boundary-deconvolution/run_gate.py` and cargo nextest.
- **Disproof Explicit:** Acknowledged the initial gate breach of 7,004 lines before applying the dated owner decision.
- **Negative Results Safe:** Falsified assumptions and exact overages documented transparently without distortion.
