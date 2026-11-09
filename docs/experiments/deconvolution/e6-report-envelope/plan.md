# E6 Plan: State-Boundary Deconvolution Generalization Trial on Report Envelope & Snapshot Store

- Written: 2026-11-09, Europe/Berlin (UTC+1), following certification of the Self-Application Maturity Gate (R43) and owner approval of the Generalization Trial.
- Specification: [.kiro/specs/state-boundary-deconvolution/spec.json](../../../.kiro/specs/state-boundary-deconvolution/spec.json) (phase completed, maturity gate passed).
- Predecessors: E5 ([R42](../../research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md), [R43](../../research/2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md)), R41 ([08-state-boundary-ceiling-amendment.md](../../research/2026-11-08/08-state-boundary-ceiling-amendment.md)), R40 ([07-structural-deconvolution-and-state-boundaries.md](../../research/2026-11-08/07-structural-deconvolution-and-state-boundaries.md)).
- Type: Preregistered trial plan with COBESY falsifiable framing. Absolute numerical prediction bars are fixed before trial execution; failing observations are recorded as falsified.
- Runtime Environment: Native Windows 11, PowerShell only (no `cmd.exe`, no WSL for development execution), CPython 3.13.16 pinned specimen venv at `.runs/e2-facdrone/venv/Scripts/python.exe` with `PYTHONPATH=src`. Preserved owner WIP files untouched and unstaged.

---

## 1. Research Question

Does the **State-Boundary Deconvolution** loop—which incorporates explicit state frame contracts, stream buffer preservation, bounded pure-core equivalence, and adverse mutation sensitivity gates—generalize beyond the `lifecycle` monitor group to an unseen, state-scattered Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 canonical lines measured via `measures.measure_written`)? Specifically, can it:
1. Break the **Density Trap** on an unseen architecture, achieving $\ge 20\%$ canonical line compression ($\le 1,550$ canonical lines) and $\ge 10\%$ AST statement compression ($\le 832$ statements)?
2. De-fragment call graphs by $\ge 20\%$ functions ($\le 81$ functions) and $\ge 25\%$ call edges ($\le 108$ edges)?
3. Defend against oracle blindness by achieving a **100% kill rate** on adverse state mutants?
4. Maintain bounded pure-core equivalence with **0 counterexamples**?
5. Strictly honor the **7,173 canonical line ceiling** on the Rust apparatus without line growth?

---

## 2. Definitions

- **Imperative Shell:** The outer coordination and persistence layer where mutable database operations, transactional commits, and I/O sinks reside (`reporting_request_store.py`, `snapshot_store.py`).
- **Pure Core:** The inner domain logic (`canonical.py` and stateless validation/formatting) containing zero mutable state writes, zero global lookups, and zero external I/O invocations, formally verified via static AST analysis and bounded equivalence checking.
- **StateFrame:** Explicit declaration of permitted write targets (`permitted_write_functions`) and exception-side state transitions (`required_failure_invariants`).
- **StreamContract:** Explicit monitoring of output streams to guarantee intermediate logs and error messages are flushed prior to abort points.
- **Density Gate:** Scope-wide evaluation separating structural consolidation (`structurally_consolidated`) from genuine compression (`compression_met`), requiring concurrent reductions in functions, call edges, AST statements, and canonical lines.
- **Kill Rate:** `killed / (killed + survived)` across planted adverse state mutants, requiring 100% kill rate on adverse controls.

---

## 3. Frozen Inputs

### Scope Files (Facdrone `report-envelope` group, 6 files, 1,938 lines)

| File | SHA-256 | Canonical Lines | AST Stmts | Functions | Write Sites |
|---|---|---:|---:|---:|---:|
| `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request.py` | `1c5dd687fbc87a2bdadc98600321e41eff8de5ab020a3adfa3ae98173b36a8a6` | 553 | 256 | 23 | 0 |
| `packages/facdrone-reporting/facdrone_reporting/shells/reporting_request_store.py` | `dccb32a179d1384974e60d2f8abf1919b744de40123ad9d543091ccc8772ac09` | 215 | 92 | 9 | 4 |
| `packages/facdrone-reporting/facdrone_reporting/shells/snapshot_store.py` | `c69421000dc3e1c1f7ae9f22a7fa7710ff7d6c27072b286ab24dd09fce82d6fa` | 453 | 154 | 11 | 0 |
| `packages/facdrone-reporting/facdrone_reporting/shells/evidence_validation.py` | `ac6a8dc79f9e1922e210d860c21058ab505c97e0616947196c636e1506af03c7` | 249 | 148 | 21 | 0 |
| `packages/facdrone-reporting/facdrone_reporting/shells/composition_errors.py` | `975539b6e446d698bd67c8cb49320eef278f65cee3ce13eb527d69e4b687b0be` | 66 | 34 | 3 | 0 |
| `packages/facdrone-kernel/facdrone_kernel/canonical.py` | `68166e605160fd829487ee953ed0838b70bc174a382b3e92f5838b6a12f1124e` | 402 | 241 | 35 | 0 |
| **Total Baseline** | — | **1,938** | **925** | **102** | **4** |

- Scope intra-scope call edges: **145** (module hops: 10).
- Specimen baseline root: `.runs/e2-facdrone/source` (read-only snapshot).
- Primary test suites: `test_store_faults.py`, `test_record_simulation_request.py`, `test_export_on_request.py`, `test_factor_basket_request.py`.

### Adverse Controls Battery for E6
1. **Control 1 (E6-C1 Missing Failure State):** Omission of exception-side error state recording / rollback recording in `reporting_request_store.py` on database transaction abort.
2. **Control 2 (E6-C2 Stream Buffer Truncation):** Truncation or suppression of intermediate diagnostic logs prior to an unexpected pipeline error in `evidence_validation.py`.
3. **Control 3 (E6-C3 Out-of-Domain Purity Violation):** Unsound pure-core substitution in `canonical.py` accepting corrupted sequence identifiers or returning fallback defaults.
4. **Control 4 (E6-C4 Error Order Inversion):** Inversion of sequential error checking order in `composition_errors.py` causing masked exception types.

---

## 4. Predictions (Fixed Prior to Execution)

| ID | Prediction | Target / Absolute Bar | Held When | Falsified When |
|---|---|---|---|---|
| **T1** | Refinement convergence | Accepted candidate in **$\le 3$ turns** | Turn $k \le 3$ accepted by verification harness | 4 turns exhausted or candidate rejected |
| **T2** | Whole-responsibility canonical lines | **$\le 1,550$ lines** ($\ge 20\%$ reduction from 1,938) | Measured canonical lines $\le 1,550$ | Canonical lines $\ge 1,551$ ($<20\%$ reduction; Density Trap) |
| **T3** | AST Statement reduction | **$\le 832$ stmts** ($\ge 10\%$ reduction from 925) | Measured statements $\le 832$ | Statements $\ge 833$ (Density Trap: statements invariant) |
| **T4** | Structural de-fragmentation | Functions $\le 81$ ($-20\%$), Call edges $\le 108$ ($-25\%$) | Both function and edge targets met | Functions $>81$ or Call edges $>108$ |
| **T5** | Adverse state mutation sensitivity | **100% kill rate** on non-equivalent adverse state mutants | All 4 adverse controls detected and killed | Any adverse mutant survives without detection |
| **T6** | Bounded pure-core equivalence | **0 counterexamples** across bounded domain $[-10, 10]$ and edge cases | Verified equivalence with 0 witnesses | Any input yields a divergence between orig and cand core |
| **T7** | Apparatus line stability | Counted Rust apparatus **$\le 7,173$ canonical lines** | Measured lines $\le 7,173$ (R41 amended ceiling) | Lines $>7,173$ (violation of no-growth cap) |
| **S** | Structural & State Contract | Confined shell write sites ($\le 4$), Pure share $\ge 50\%$, StateFrame verified, StreamContract verified | Full structural contract verified | Any write site leak, pure violation, or dropped buffer |

---

## 5. Kill Criteria

1. **Density Trap Trigger:** Candidate consolidates functions or edges but fails T2 ($\le 1,550$ lines) or T3 ($\le 832$ stmts). Outcome marked `REVISE / FAILED_DENSITY_TRAP`.
2. **Oracle Blindness / Adverse Survivor:** Any candidate that passes standard tests but allows an adverse state mutant (E6-C1 through E6-C4) to survive.
3. **State Frame Violation:** Any state mutation site discovered outside declared imperative shell functions.
4. **Stream Divergence:** Intermediate output buffers truncated or dropped prior to process abort.
5. **Pure Core Non-Equivalence:** Any counterexample discovered during bounded domain evaluation.
6. **Apparatus Line Expansion:** Any growth of the Rust substrate apparatus beyond 7,173 canonical lines.
