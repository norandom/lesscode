# E5 Plan: State-Boundary Deconvolution Trial on the Lifecycle Monitor Group

- Written: 2026-11-08, Europe/Berlin (UTC+1), Option 1 disposition following R40 synthesis and R41 apparatus ceiling amendment (`3387902`).
- Specification: [.kiro/specs/state-boundary-deconvolution/spec.json](../../../.kiro/specs/state-boundary-deconvolution/spec.json) (phase completed, prototype gate PASS under R41 amended ceiling).
- Predecessors: E3 ([R25](../../research/2026-11-04/02-e3-evidence-regeneration.md)), E4 ([R26](../../research/2026-11-06/01-e4-lifecycle-regeneration.md)), R38 ([05-u5-ceiling-amendment.md](../../research/2026-11-08/05-u5-ceiling-amendment.md)), R39 ([06-refinement-correctness.md](../../research/2026-11-08/06-refinement-correctness.md)), R40 ([07-structural-deconvolution-and-state-boundaries.md](../../research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md)), R41 ([08-state-boundary-ceiling-amendment.md](../../research/2026-11-08/08-state-boundary-ceiling-amendment.md)).
- Type: Preregistered trial plan with COBESY falsifiable framing. Absolute numeric prediction bars are fixed before trial execution; failing observations are recorded as falsified.
- Runtime Environment: Native Windows 11, PowerShell only (no `cmd.exe`, no WSL for development execution), CPython 3.13.16 pinned specimen venv at `.runs/e2-facdrone/venv/Scripts/python.exe` with `PYTHONPATH=src`. Preserved owner WIP files untouched and unstaged.

---

## 1. Research Question

Can state-boundary deconvolution—enforcing explicit state frames, stream buffer preservation, bounded pure-core equivalence, and adverse mutation sensitivity—break out of the **Density Trap** observed in E4 (where function consolidation achieved $-24.6\%$ functions and $-29.2\%$ call edges, but only $-6.0\%$ canonical lines) and achieve $\ge 20\%$ whole-responsibility canonical line reduction ($\le 1,196$ lines) without oracle blindness on edge cases and without expanding the Rust apparatus beyond the R41 amended ceiling ($\le 7,173$ lines)?

---

## 2. Definitions

- **Imperative Shell:** The outer coordination layer containing all permitted mutable state locations, database transactions, and I/O sinks.
- **Pure Core:** The inner domain logic containing zero mutable state writes, zero global lookups, and zero external I/O invocations, formally verified via static AST analysis and bounded equivalence checking.
- **StateFrame:** Explicit declaration of permitted mutable locations and post-condition invariants across both normal returns and exception exits (`required_failure_invariants`).
- **StreamContract:** Explicit monitoring of stdout, stderr, and logging streams to ensure output emitted prior to abort/crash points is preserved in correct order.
- **Density Gate:** Scope-wide metric evaluation distinguishing `structurally_consolidated` (merely packing code into fewer functions) from `compression_met` (true net reduction in AST statements and canonical lines).
- **Kill Rate:** `killed / (killed + survived)` on planted adverse state mutants, requiring 100% kill rate on adverse controls (E3 `AuditedAbsence`, E4 F1 `Silent_None`, E4 F3 `Buffer_Drop`, E4 F7 `Error_Order`).

---

## 3. Frozen Inputs

### Scope Files (Facdrone `lifecycle` monitor group, 7 files, 1,495 canonical lines)

| File | SHA-256 | Canonical Lines | AST Stmts | Functions | Write Sites |
|---|---|---:|---:|---:|---:|
| `src/facdrone/shells/lifecycle_monitor_runner.py` | `fdd35e1c42caabe83c5fcb8136924355a707f9a684fc828a84a454674657b2f5` | 397 | 189 | 14 | 3 |
| `src/facdrone/shells/lifecycle_store.py` | `35d9f27a7727c49cb49086254c2513b897a89621f3787ef69c6d65c09f38d974` | 289 | 142 | 17 | 8 |
| `src/facdrone/shells/cli/lifecycle_group.py` | `03bfef33c14209407d8fcbac9911ce0f3cb51601a68c64cc1e96a7d607d8b6cb` | 278 | 158 | 9 | 0 |
| `src/facdrone/shells/lifecycle_evidence.py` | `5ae9a370f74516249ca5663b6e9907069d97da8ba04ec0af23d28fda884fe354` | 215 | 114 | 11 | 1 |
| `src/facdrone/shells/lifecycle_projection.py` | `adb6075c061d48058a56b69655d0486bc6ea1f6a98e927774e0c3e596d94cea7` | 161 | 61 | 9 | 0 |
| `src/facdrone/shells/cppi/lifecycle_seams.py` | `10e5605f51589f89f5a9fbe83c7c3e5838e9a5baea696733244e93ff5ac0bf41` | 97 | 56 | 5 | 0 |
| `src/facdrone/shells/wiring/lifecycle_root.py` | `ffe5ac0b1ead257e781b65683d28160c2d3ad3232e47e008c1b36bfec6304f7f` | 58 | 39 | 4 | 0 |
| **Total Baseline** | — | **1,495** | **759** | **69** | **12** |

- Baseline intra-scope call edges: **89**.
- Specimen root: `.runs/e2-facdrone/source` (read-only, SHA-256 tree snapshot).
- Golden acceptance corpus: `.runs/e4-lifecycle/golden-a.json` (50 sequences, 317 operations).

### Adverse Controls Battery
1. **Control 1 (E3 `AuditedAbsence`):** Omission of audit log recording on serialization failure / exception path.
2. **Control 2 (E4 F1 `Silent_None`):** Silent acceptance of `None` evidence values on typed inputs instead of raising ValueError.
3. **Control 3 (E4 F3 `Buffer_Drop`):** Loss of intermediate stdout bytes prior to an unexpected pipeline abort.
4. **Control 4 (E4 F7 `Error_Order`):** Inversion of sequential input validation order causing wrong exception type to propagate.

---

## 4. Predictions (Fixed Prior to Execution)

| ID | Prediction | Target / Absolute Bar | Held When | Falsified When |
|---|---|---|---|---|
| **T1** | Refinement convergence | Accepted candidate in **$\le 3$ turns** | Turn $k \le 3$ accepted by verification harness | 4 turns exhausted or candidate rejected |
| **T2** | Whole-responsibility canonical lines | **$\le 1,196$ lines** ($\ge 20\%$ reduction from 1,495) | Measured canonical lines $\le 1,196$ | Canonical lines $\ge 1,197$ ($<20\%$ reduction; Density Trap) |
| **T3** | AST Statement reduction | **$\le 683$ stmts** ($\ge 10\%$ reduction from 759) | Measured statements $\le 683$ | Statements $\ge 684$ (Density Trap: statements invariant) |
| **T4** | Structural de-fragmentation | Functions $\le 55$ ($-20\%$), Call edges $\le 66$ ($-25\%$) | Both function and edge targets met | Functions $>55$ or Call edges $>66$ |
| **T5** | Adverse state mutation sensitivity | **100% kill rate** on non-equivalent adverse state mutants | All 4 adverse controls detected and killed | Any adverse mutant survives without detection |
| **T6** | Bounded pure-core equivalence | **0 counterexamples** across bounded domain $[-10, 10]$ and edge cases | Verified equivalence with 0 witnesses | Any input yields a divergence between orig and cand core |
| **T7** | Apparatus line stability | Counted Rust apparatus **$\le 7,173$ canonical lines** | Measured lines $\le 7,173$ (R41 amended ceiling) | Lines $>7,173$ (violation of no-growth cap) |
| **S** | Structural & State Contract | Write sites $\le 12$, Pure share $\ge 50\%$, StateFrame verified, StreamContract verified | Full structural contract verified | Any write site leak, pure violation, or dropped buffer |

---

## 5. Kill Criteria

1. **Density Trap Trigger:** Candidate achieves structural consolidation (fewer functions/edges) but fails T2 ($\le 1,196$ lines) or T3 ($\le 683$ stmts). Outcome marked `REVISE / FAILED_DENSITY_TRAP`.
2. **Oracle Blindness / Adverse Survivor:** Any candidate that passes trace replay but allows an adverse state mutant (such as E3 `AuditedAbsence` or E4 F1/F3/F7) to survive.
3. **State Frame Violation:** Any state mutation site discovered outside declared imperative shell functions.
4. **Stream Divergence:** Intermediate output buffers truncated or dropped prior to process abort.
5. **Pure Core Non-Equivalence:** Any counterexample discovered during bounded domain evaluation.
6. **Apparatus Line Expansion:** Any growth of the Rust substrate apparatus beyond 7,173 canonical lines.

---

## 6. Execution Protocol

1. **Step 1: Baseline Round 0 Verification**
   - Execute baseline measurement and verification harness against `.runs/e2-facdrone/source` to confirm baseline numbers and verify that adverse controls fail closed.
2. **Step 2: Candidate Construction & Refinement Loop**
   - Execute the deconvolution driver with explicit pure-core partitioning and imperative shell extraction.
   - Inject state-boundary verification diagnostics (frame condition leaks, stream buffer drops, bounded counterexamples) into refinement feedback.
3. **Step 3: Verification Harness Gate Evaluation**
   - Run acceptance suite (50 sequences, 317 operations).
   - Run StateFrame static and dynamic invariant checkers.
   - Run StreamContract buffer preservation verifier.
   - Run PureVerifier bounded equivalence on extracted pure functions.
   - Run StateMutantHarness sensitivity gate across adverse mutants.
   - Run DensityGate measuring functions, edges, write sites, AST statements, and canonical lines.
   - Run apparatus line accounting against R41 ceiling (7,173 lines).
4. **Step 4: Evidence Recording & Research Log**
   - Record results in `docs/experiments/deconvolution/e5-state-boundary-lifecycle/results/`.
   - Author formal research log entry `docs/research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md`.
