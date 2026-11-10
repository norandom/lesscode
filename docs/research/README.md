# Research log

> **Milestone Synthesis Report:** For the comprehensive cross-subsystem synthesis synthesizing Experiments E1 through E6, breaking the Density Trap, and certifying the dual-specimen maturity gates under COBESY principles, see [`docs/reports/state-boundary-deconvolution-report.md`](../reports/state-boundary-deconvolution-report.md).

## Latest: R57 Scopes admitted for the annotation-driver gate (2026-11-10)

[R57](2026-11-10/03-annotation-driver-scope-admission.md) completes checklist
step 1 of the gate. Setup only; no model call, no hypothesis test.
- **Admitted scopes, baselines healthy in isolated clones:** Rust `mmhmem` `src/setup/check.rs` (212 lines, 28 tests) and Python `mvibe` `mvibe_proxy/ratelimit.py` (478 lines, 230 tests) for guide development; TypeScript `OpenUltraCode` `src/state.ts` (188 lines, 8 tests) and Java `commons-csv` `Lexer.java` (319 lines, 37 tests) held out.
- **Control:** the fixed operators yield 0, 0, 1 and 0 mutants on these scopes. The gate expected 0 everywhere; Amendment 1 corrects the yield row before any run.
- **Proposed, pending the owner:** arm D, the generic operators as declarative syntax-tree rules with no model at run time, and rule packs for the held-out languages written by the model offline and admitted by checks.
- **Next:** owner confirmation of arm D, then the validator and its controls.

## Prior: R56 Direction: agnostic annotation-guided driver, gate before design (2026-11-10)

[R56](2026-11-10/02-agnostic-annotation-driver-direction.md) records the owner's
priorities after R55 and the plan that follows. Nothing has been run.
- **Priorities:** (1) a driver that is agnostic to code and language, so the process applies horizontally; (2) hardening against LLM inference drift. Both rest on an annotation-guidance process, later optimised with bounded and verified `MIPROv2` (`GEPA` if needed). Opus endpoint now, Mistral later.
- **Hypothesis H-AD1:** the model returns located annotations from a fixed taxonomy and writes no code; a deterministic validator and generic operators produce mutants; real tests classify them; one frozen guide works on languages it was not developed on.
- **Spec:** new [`agnostic-annotation-driver`](../../.kiro/specs/agnostic-annotation-driver/brief.md) at phase `initialized`, with a [prototype gate](../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md) (status not-run): three arms (annotation-guided, fixed operators as control, an existing mutation tool per language), four scopes outside the specimens, guide developed on Rust and Python and frozen for TypeScript and Java, targets fixed before execution, stop in favour of existing tools as a valid outcome.
- **Next gate:** the owner's go to screen scopes and run it.

## Prior: R55 The released binary used as an adopter would use it (2026-11-10)

[R55](2026-11-10/01-released-binary-adopter-dogfood.md) runs the 1.0.1 binary's
documented commands against this repository from the outside:
- **Disposition:** completed, exploratory; the hypothesis that every documented command gives a meaningful receipt on ordinary code was rejected.
- **Works:** `budgets` (38 files, 0 violations with `--root native/src`), `deconvolute density` (reproduces R52: 248 to 148 lines, 83 to 70 statements), `deconvolute analyze` for Rust, Java and TypeScript, and the pre-push hook's budget gate.
- **Does not work for an adopter:** `analyze --lang python` returns an empty receipt with `"passed": true` without analysis; `mutate` produced no mutants for 36 of 38 Rust files and 0 of 40 Python files because its operators replace specimen-specific substrings; the hook's `refactor/*` handling prints a note and checks nothing.
- **Breaks:** with the hook installed and the binary not on `PATH`, every push fails. Three README commands failed as written (`--file`, missing `--out`, wrong budgets file).
- **Consequence:** README, docs index and agent skill corrected; R54's tool consequence is narrowed to budget enforcement. No code changed. Owner decision open: implement, fail loudly, or drop the three non-working paths.

## Prior: R54 Self-application dogfooding, clean-clone validation & intelligent pre-push hook (2026-11-09)

> Amended 2026-11-10 by [R55](2026-11-10/01-released-binary-adopter-dogfood.md): the hook enforces complexity budgets only. The state-boundary check on `refactor/*` and `deconvolute/*` branches described below is not implemented.

[R54](2026-11-09/12-dogfood-self-application-synthesis.md) records the execution,
clean-clone verification, and tooling integration for the `dogfood-self-application`
milestone:
- **Disposition:** COMPLETED (all 7 spec tasks certified GO).
- **Clean-Clone Test Pass Rate:** 352/352 native tests (100%) passed via `cargo nextest` across 33 binaries on a pristine repository clone (`..\lesscode-dogfood-copy`) with zero manual interventions or warm caches.
- **Fixture Integrity & CRLF Normalization:** Tracked missing regenerator turn fixtures and historical `.runs/` fixtures with `text eol=lf` in `.gitattributes`, permanently eliminating byte-level hash divergences in `historical_s2_bindings_drift_reproduces_counts`.
- **Porcelain Commands:** Implemented `analyze`, `mutate`, and `density` subcommands in `native/src/cli_deconvolute.rs` with isolated specimen build targets (`target/isolated-specimen`).
- **Intelligent Git Pre-Push Hook:** Implemented `pre-push` and `install-hook` in `native/src/cli_hook.rs`, enforcing complexity budgets and AST state-boundary cleanliness on `refactor/*` and `deconvolute/*` branches while cleanly bypassing branch deletions and `LESSCODE_SKIP_HOOK=1`.
- **Complexity Budgets:** 10/10 budget checks passed across 38 governed files; 0 exceeded, 0 unlisted, 0 missing.

## Prior: R53 Multi-language deconvolution unification synthesis and four-substrate certification (2026-11-09)

[R53](2026-11-09/11-multi-language-deconvolution-synthesis.md) records the comprehensive
retrospective synthesis and full certification of the unified multi-language deconvolution
architecture across all four production language profiles (**Python**, **TypeScript**, **Java**, and **Rust**):
- **Disposition:** UNIFIED & COMPLETED (all 4 specs certified GO).
- **Substrate Unification:** Hosted natively under `lesscode-core` with 4 normative carrier contracts (`StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, `SpecimenTraceRunner`).
- **Cross-Language Compression:** Verified substantial whole-responsibility compression across 4 distinct production specimens:
  - Python (Facdrone): −23.80% lines, −11.20% stmts (Breaking Density Trap, 100 cases, 0 divergences).
  - TypeScript (`OpenUltraCode`): −43.78% lines, −40.38% stmts (100 cases, 0 divergences).
  - Java (`apache/commons-csv`): −38.46% lines, −27.68% stmts (100 cases, 0 divergences).
  - Rust (`lesscode-core` lifecycle): −40.32% lines, −15.66% stmts (100 cases, 0 divergences).
- **Adverse Sensitivity:** 100% kill rate across 20/20 mandatory adverse mutation controls (Python 8/8, TypeScript 4/4, Java 4/4, Rust 4/4).
- **Apparatus Budget:** 7,173 canonical lines held (0 lines overage against R41 ceiling); 36/36 modules within complexity budgets.
- **Substrate Health:** 338/338 `cargo-nextest` tests passed across 33 binaries; 32/32 `pytest` tests passed in parallel.
- **Reference Published:** Architecture and extension guide published in `docs/multi-language-deconvolution.md`.

## Prior: R52 Rust deconvolution adapter prototype feasibility gate (2026-11-09)

[R52](2026-11-09/10-rust-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `rust-deconvolution-unification`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified: `bcd823bc7edc79141d4dcbe5951eb8b2ec39773be5471fe7964d31a5248ff4a6`).
- **Candidate Admission:** Admitted native process lifecycle supervisor responsibility (`native/src/recording.rs`, lines 712–969) in isolated copies under `native/adapters/rust/baseline_lifecycle.rs` and `native/adapters/rust/candidate_lifecycle.rs` with frozen Rust 1.98.1 and cargo-nextest 0.9.143.
- **Compression:** 248 -> 148 canonical lines (−40.32%, 100 lines removed) and 83 -> 70 statements (−15.66%, 13 statements removed), meeting both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`rust-nextest-v1`) and IEEE-754 value transports.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across process lifecycle, exit codes, and environment configs with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (deadline kill omission, inverted verdict condition, inverted error classification, dropped environment isolation).
- **Specimen Regression:** Full Nextest execution across 28 binaries passed with 314 tests run, 314 passed, 0 failures.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R51 Java deconvolution adapter prototype feasibility gate (2026-11-09)

[R51](2026-11-09/09-java-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `java-deconvolution-unification`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified: `207765c042ef02e263c6d19648c19532675dc9c52b95e6044a179d409e2b3410`).
- **Candidate Admission:** Admitted real licensed specimen `apache/commons-csv` at commit `2bd66bf181d44ffc92806f585d3955e915666f7f` (Apache License 2.0) with Microsoft OpenJDK 21.0.12.1 LTS, Apache Maven 3.9.9, and JUnit 5.
- **Compression:** 221 -> 136 canonical lines (−38.46%, 85 lines removed) and 112 -> 81 statements (−27.68%, 31 statements removed), substantially exceeding both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`java-junit-v1`) and buffered stream flushing via `ProbeMain.java`.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across 5 CSV formats and 10 field data structures with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (state mutation omission, lock cleanup omission, delimiter sequencing inversion, dropped stream flush).
- **Specimen Regression:** Full Maven surefire execution of `CSVPrinterTest` passed with 168 tests run, 0 failures, 0 errors, 7 skipped.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held; adapter probe bounded at 212 canonical Java lines.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R50 TypeScript deconvolution adapter prototype feasibility gate (2026-11-09)

[R50](2026-11-09/08-typescript-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `typescript-deconvolution-adapter`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified).
- **Candidate Admission:** Admitted real licensed specimen `OpenUltraCode` with pinned Node.js 24.22.1, TypeScript 6.0.3, and tsx 4.22.4.
- **Compression:** 201 -> 113 canonical lines (−43.78%, 88 lines removed) and 52 -> 31 statements (−40.38%, 21 statements removed), exceeding both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`ts-node-v1`) and IEEE-754 bit representations for floats via `runner.mjs`.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across notice factories and degradation summary with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (severity inversion, slugify drop, stream buffer drop, summary error order).
- **Specimen Regression:** `pnpm exec tsc --noEmit` passed exit 0; `tests/degradation.test.ts` passed 3/3.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held; adapter probe bounded at 175 JavaScript lines.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R49 Native deconvolution closure prototype feasibility gate (2026-11-09)

[R49](2026-11-09/07-native-closure-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `native-deconvolution-closure`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified).
- **Parity:** Exact match on frozen E3/E4 replay digests with 0 Level 1 / Level 2 divergences.
- **Fidelity:** 0 counterexamples across bounded Cartesian domain `[-10, 10]` + edge cases.
- **Adverse Controls:** 100% kill rate across mandatory controls (E3 AuditedAbsence, E4 F1, F3, F7).
- **Compression:** Reproduced E5 Lifecycle (−24.82% lines, −12.17% stmts) and E6 Report Envelope (−23.01% lines, −10.49% stmts).
- **Apparatus Accounting:** 7,173 canonical lines held (0 overage against R41 ceiling); 10/10 budget tests passed.
- **Outcome:** Technical design and foreground TDD implementation tasks formally unblocked and approved.

## Prior: R48 aligned provisional adapter designs (2026-11-09)

[R48](2026-11-09/06-aligned-language-adapter-designs.md) records the shared native
contract and TypeScript/Java/Rust provisional designs, per-language research and
bounded prototype gates. Requirements approved by the owner's design request;
designs unapproved, gates not-run, production tasks absent.

## Prior: R47 specification review and hypothesis amendment (2026-11-09)

[R47](2026-11-09/05-deconvolution-spec-review-amendment.md) revises native closure
and the TypeScript/Java/Rust requirements after cross-spec review. Java is an
authorized target; runtime adapters remain counted; bounded claims and unknown
outcomes are explicit; native production planning awaits prototype evidence.
Documentation review only: no new experiment or runtime qualification.

## Prior: R46 Report envelope adoption & generalization maturity gate certification (2026-11-09)

[R46](2026-11-09/04-e6-report-envelope-adoption-maturity-gate.md) records the formal adoption of the
deconvoluted **Report Envelope & Snapshot Store** group (6 files) into `.runs/e6-adopted-report-envelope/`
and certifies the dual-specimen **Generalization Adoption Maturity Gate**:
- **Dual-Specimen Replication:** Confirmed cross-subsystem deconvolution across two distinct Facdrone
  domains: Signal Lifecycle Monitors (E5, 7 files, −24.82% lines, −12.12% statements) and Report Envelope
  & Snapshot Store (E6, 6 files, −23.01% lines, −10.49% statements).
- **Combined Impact:** 13 files, 3,433 baseline lines compressed to 2,616 adopted lines (−23.80%),
  with 11.20% statement reduction, 33.92% function reduction, and 30.93% call edge reduction.
- **State Confinement:** 15 total write sites confined strictly to designated imperative shells (11 in
  lifecycle store, 4 in reporting request store).
- **Behavioral Soundness:** 100% adverse mutant kill rate, 0 bounded pure-core counterexamples, 0 replay
  divergences, and 100% test pass on all unit and acceptance suites.
- **Rust Apparatus Volume:** Counted Rust substrate held at 7,173 canonical lines (0 overage against R41 ceiling).
- Full adoption artifacts verified: `e6-adopted-report-envelope.diff` and `e6-adoption-report.json`.

## Prior: R45 E6 generalization trial qualification — report envelope certified (2026-11-09)

[R45](2026-11-09/03-e6-generalization-trial-qualification.md) records the empirical qualification
of **Experiment E6: State-Boundary Deconvolution Generalization Trial** on an unseen
Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 baseline lines):
- **Canonical lines reduced from 1,938 to 1,492** (−23.01% reduction vs $\ge 20\%$ target).
- **AST statements reduced from 925 to 828** (−10.49% reduction vs $\ge 10\%$ target; Density Trap defeated).
- **Functions de-fragmented from 102 to 63** (−38.24% reduction vs $\ge 20\%$ target).
- **Call edges de-fragmented from 145 to 101** (−30.34% reduction vs $\ge 25\%$ target).
- **Write sites confined strictly to 4 imperative shell locations** in `reporting_request_store.py`.
- **Pure module share held at 5/6 (83.3%)** (target $\ge 50\%$).
- **Adverse mutation sensitivity:** 4/4 adverse controls killed (100% kill rate, E6-C1 to E6-C4).
- **Pure-core bounded equivalence:** 0 counterexamples across bounded domain $[-10, 10]$ and edge cases.
- **Rust apparatus stability:** 7,173 canonical lines held (0 overage against R41 ceiling).
- **Acceptance test battery:** 46/46 passed with zero regressions across `facdrone-reporting`, `facdrone-kernel`, and root suites.
- Certified as a generalizable, subsystem-agnostic architectural compression loop.

## Prior: R44 E6 state-boundary deconvolution generalization trial setup (2026-11-09)

[R44](2026-11-09/02-e6-generalization-trial-setup.md) records the preregistration and baseline
census for **Experiment E6: State-Boundary Deconvolution Generalization Trial** on an unseen
Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 canonical lines):
- Scope: 6 files across `facdrone-reporting` and `facdrone-kernel` (`reporting_request.py`,
  `reporting_request_store.py`, `snapshot_store.py`, `evidence_validation.py`,
  `composition_errors.py`, `canonical.py`).
- Baseline metrics: **1,938 canonical lines**, **925 AST statements**, **102 functions**,
  **145 intra-scope call edges**, and **4 write sites** (confined to `reporting_request_store.py`).
- Preregistered target bars: $\le 1,550$ canonical lines ($\ge 20\%$ reduction), $\le 832$ statements
  ($\ge 10\%$ reduction; Density Trap defense), $\le 81$ functions, $\le 108$ call edges.
- Adverse controls battery qualified: 4/4 passed (missing failure state, buffer truncation,
  out-of-domain purity, error order inversion).
- Rust substrate apparatus volume held at 7,173 canonical lines (0 overage against R41 ceiling).
- Acceptance test battery verified passing across reporting and root suites.

## Prior: R43 lifecycle deconvolution adoption & self-application maturity gate (2026-11-09)

[R43](2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md) records the formal adoption of the
deconvoluted Facdrone `lifecycle` monitor group (7 files) into `.runs/e5-adopted-lifecycle/` and
the certification of the **Self-Application Maturity Gate**:
- **Canonical lines reduced from 1,495 to 1,124** (−24.8% reduction, −371 lines).
- **AST statements reduced from 759 to 667** (−12.1% reduction, −92 statements).
- **Functions de-fragmented from 69 to 50** (−27.5% reduction, −19 functions).
- **Call edges de-fragmented from 89 to 61** (−31.5% reduction, −28 edges).
- **State write sites confined to 11 imperative shell locations** (target $\le 12$).
- Verified with exact pre/post SHA-256 hashes and 1,504-line unified diff.
- Full regression passed: specimen 108/108 tests, replay oracle 50/50 sequences with 0 Level 2 divergences.
- Verification suites green: Rust 241/241, Python state-boundary 32/32, apparatus volume 7,173 lines (0 overage).

## Prior: R42 E5 state-boundary deconvolution trial — breaking the density trap on lifecycle (2026-11-08)

[R42](2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md) records the empirical
execution and successful qualification of **Experiment E5: State-Boundary Deconvolution Trial**
on the Facdrone `lifecycle` monitor group (7 files, 1,495 baseline lines). The trial formally
**broke the Density Trap** observed in E4 without oracle blindness:
- **Canonical lines reduced by 24.8%** (1,124 lines vs $\le 1,196$ target / 1,495 baseline).
- **AST statements reduced by 12.1%** (667 stmts vs $\le 683$ target / 759 baseline).
- **Functions de-fragmented by 27.5%** (50 funcs vs $\le 55$ target / 69 baseline).
- **Call edges de-fragmented by 31.5%** (61 edges vs $\le 66$ target / 89 baseline).
- **100% kill rate on adverse controls** (E3 `AuditedAbsence`, E4 F1, F3, F7).
- **0 counterexamples on bounded pure-core equivalence** over $[-10, 10]$ and edge cases.
- **Rust apparatus volume held at 7,173 lines** (0 overage against R41 amended ceiling).
- Replay oracle green across 50/50 sequences and 317 operations with 0 Level 2 divergences.
- Unit suites fully passing: Rust 241/241, Python state-boundary 32/32, specimen 108/108.

## Prior: R41 state-boundary apparatus ceiling amendment — owner chose amend (2026-11-08)

[R41](2026-11-08/08-state-boundary-ceiling-amendment.md) records the owner disposition
on the prototype feasibility gate overage (option (a) of the preregistered outcome rule;
instruction: *"a) it is"*): **the Rust-substrate apparatus ceiling is amended from 7,004 to
7,173 canonical lines** — exactly the measured total on the extended counted set (7,079 on
R37 counted set + 94 for `native/src/state_contract.rs`), effective as a **no-growth cap**.
All 4 adverse controls (E3 `AuditedAbsence`, E4 F1 `Silent_None`, E4 F3 `Buffer_Drop`,
E4 F7 `Error_Order`) achieved 100% kill rates, the Facdrone `lifecycle` density trap was
defended, and the feasibility gate is marked **PASS** under the amended ceiling. Spec
`state-boundary-deconvolution` fully qualified.

## Prior: R40 structural consolidation without line reduction — the scattered state and density boundary (2026-11-08)

[R40](2026-11-08/07-structural-deconvolution-and-state-boundaries.md) synthesizes
the empirical results on structural consolidation across E1–E4, R11, R25, R26,
and R34–R39: an LLM can de-fragment scattered responsibilities (−24.6% functions,
−29.2% call edges, write sites confined to 11 shell functions, 51.9% pure core),
but canonical lines dropped only 6.0% (missing the 10–20% target; density replaced
fragmentation), while trace oracles missed exception-side state and partial buffer
divergences. Proposes a new specification (`state-boundary-deconvolution`) using
software verification techniques (frame conditions, bounded SMT pure-core equivalence,
adverse mutant killing) to soundly govern state-boundary deconvolution.

## Prior: R39 confirms refinement correctness defects; repaired and verified (2026-11-08)

[R39](2026-11-08/06-refinement-correctness.md) reproduced wrong-candidate
delivery, lost runtime feedback, erroneous obligation discharge, false oracle
passes after failure/timeout, and unsupported validity/compression outcomes.
Repaired and verified under specification
[refinement-evidence-integrity](../../.kiro/specs/refinement-evidence-integrity/spec.json)
(Task group 12; commit `37f21c4`, handover `f7c254f`). Adversarial probe battery
passes (Rust 9/9, Python 5/5) with regenerator suites green (Rust 50/50, Python 59/59).

## Prior: R38 U5 ceiling amendment — owner chose amend (2026-11-08)
[R38](2026-11-08/05-u5-ceiling-amendment.md) records the owner disposition on
the R37 negative (option (a) of the preregistered outcome rule; instruction:
"proceed with the amended ceiling"): **the Rust-substrate U5 ceiling is
amended from 2,270 to 7,004 canonical lines** — the measured R37 total on the
R37 counted set, effective as a **no-growth cap** (growth or baseline change
needs a further dated decision). The 20% whole-replacement reduction claim is
**retired as falsified**; the Python 3,783 and Rust 7,049/7,004 negatives
stay on record (`Req 7.8`) and the R36/R37 gate records remain FAIL under
the frozen 2,270. **Slice B adoption planning unlocked; production adoption
execution still gated** by `Req 7.9`/`7.11` and unscheduled until the owner
requests a plan. Task 10.1; no code, budget or frozen record changed.

## Prior: R37 slim-pass gate FAIL as measured — levers exhausted, disposition resolved by R38 (2026-11-08)
[R37](2026-11-08/04-rust-slim-pass-remeasure.md) executed under task 9.6
(gate record
[`rust-u5-gate-r37.json`](../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate-r37.json)):
G1-R items 1/2/3/5 **PASS** through the slimmed substrate (six Tier-1
controls zero-runtime, E4 three-turn replay at the recorded scores with
frozen digests, `k=2` exhaustion retains best, receipts
admission-verified), and **U5 = 7,004 canonical lines vs the unchanged
2,270 ceiling — FAIL by 4,734**. The preregistered levers delivered −45
net of 7,049 (defect-7 split: receipt 178 → 124 + jsonval 130; shared
kernels `j_get`/serde-bridge/escape consolidated into `jsonval.rs`;
writer merges abandoned on semantic grounds under the byte-parity
freezes): the bounded-consolidation hypothesis is falsified — the overage
is port expression volume, not consolidation overhead. Negative retained,
no ceiling or baseline change; Slice B stays withheld (`Req 7.10`); owner
chooses amend (dated decision) or close — *resolved 2026-11-08 by
[R38](2026-11-08/05-u5-ceiling-amendment.md): amend*. Task group 9 closed —
handover at
[.kiro/task_9_handover.md](../../.kiro/task_9_handover.md).

## Prior: Rust-apparatus slim pass preregistered — task group 9 opened (2026-11-08)
[R37](2026-11-08/04-rust-slim-pass-remeasure.md) freezes the slim-pass
accounting before any slim code exists, recording the owner dispositions on
the R36 negative: slim the Rust apparatus (`regenerator.rs` 2,798 the
largest lever; shared derivable structure across `surface`/`measures`/
`recording`), split `receipt.rs` for defect 7 without amending R30, and
investigate the g1r `FROZEN_RUST_CANONICAL = 1,333` provenance. The counted
set extends to every new module the split/slim adds (`Req 6.1`); the
canonical method, behavioral freezes and the unchanged 2,270 ceiling carry
over from R36; PASS requires the counted total ≤ 2,270 with budgets exit 0
(at least −4,779 of 7,049, −68%); fail returns to the owner with
amend/close options.

## Prior: Rust-port U5 re-measurement FAIL as measured — owner disposition open (2026-11-08)
[R36](2026-11-08/03-rust-port-u5-remeasure.md) executed under task 8.6
(commit `7657969`; gate record
[`rust-u5-gate.json`](../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate.json)):
G1-R items 1/2/3/5 re-discharged **PASS** through the Rust substrate (six
Tier-1 controls zero-runtime; E4 three-turn replay at the recorded scores
with frozen digests reproduced; k=2 exhaustion retains best; receipts
admission-verified), and **U5 = 7,049 canonical lines vs the frozen 2,270
ceiling — FAIL by 4,779** (new modules 6,382 + re-measured base 452 + shim
178 + hooks 16 + 21). The smaller-replaceable-representation hypothesis is
falsified on the target substrate (port ≈ 2.9× the Python stage-1 canonical);
no target weakened, negative retained; Slice B stays withheld (`Req 7.10`);
slim/amend returns to the owner. Task group 8 closed — handover at
[.kiro/task_8_handover.md](../../.kiro/task_8_handover.md).

## Prior: Rust-port U5 re-measurement preregistered — task group 8 opened (2026-11-08)
[R36](2026-11-08/03-rust-port-u5-remeasure.md) freezes the re-measurement
accounting before any port code exists: counted set = ported
`native/src/{surface,measures,recording,regenerator}.rs` + re-measured
`admission.rs`/`receipt.rs` + enumerated apparatus hooks; unchanged
rustfmt/ruff canonical method, unchanged 2,270 ceiling against
$B_{\text{app}}^{\text{expanded}} = 2{,}838$; Python modules stay as
superseded stage-1 reference; deterministic G1-R checklist items 1/2/3/5 must
re-discharge through the Rust substrate; pinned `ureq` recorded as a Req 3.6
dependency decision; pass unlocks Slice B adoption planning, fail returns to
the owner with slim/amend options.

## Prior: U5 disposition — Python stage-1 negative accepted; Rust re-measure gate before Slice B (2026-11-08)
[R35](2026-11-08/02-u5-disposition.md) records the owner disposition for the
failed whole-replacement apparatus ceiling: U5 stays failed on the Python
stage-1 prototype (3,783 vs 2,270 canonical lines; frozen ceiling unchanged,
`Req 7.8` negative retained). Slice B adoption remains withheld until a
Rust-port re-measurement of U5 under the same accounting passes the frozen
ceiling. The logged concise-output directive dispatch defect (emitted by
`next_rollout_params` but never sent to the model) is fixed in
`regenerator.py` with dated comments and a regression test.

## Prior: G1-R discharged — live weaker-model recovery reaches R=1.0; U5 fails as measured (2026-11-08)
[R34](2026-11-08/01-g1r-gate-and-live-recovery.md) executes the frozen G1-R
checklist for the DSPy-style refinement slice: all deterministic items green
(six Tier-1 controls fail closed with zero runtime, E4 replay sub-second,
k=2 exhaustion, every receipt admission-verified), and live run B-2 shows
`claude-haiku-4-5` recovering from a Tier-1 out-of-scope failure to full
acceptance R=1.0 inside the frozen budgets, with cost and negatives retained.
The live run also surfaced and fixed five apparatus defects (temperature and
effort dispatch, structure-grammar prompt bug, structure diagnostics, key
normalization). U5 is recorded as failed (3,783 vs 2,270 canonical lines);
owner disposition open, Slice B adoption still withheld. Python remains the
stage-1 prototype substrate; Rust is the target. Verification: pytest -n 4
(648 passed; only documented pre-existing failures), cargo nextest 123/123,
lesscode-core budgets exit 0, full receipt/sidecar hash sweep.

## Prior: constraint-guided-compression design reconciled with H8/H9 + DSPy-style probabilistic turn refinement (2026-11-07)
Owner reviewed the feasibility audit of `constraint-guided-compression` and directed
reconciling the spec with the completed detection/recording cores plus DSPy-style
probabilistic multi-turn prompt refinement (`dspy.Refine`/`BestOfN` with two-tier
static-assertion and runtime-oracle reward feedback) so weaker or non-deterministic
models recover from thinking-budget, syntax, surface or divergence failures across
bounded turns ([R33](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md),
[design](../../.kiro/specs/constraint-guided-compression/design.md)). Demonstrated
detection/loop apparatus (`surface`, `measures`, `editscope`, `testinfra`, single-turn
`regen_turn`) is separated from the provisional multi-rollout refinement slice (`H10`,
gated by spike `G1-R` in [prototype-gate.md](../../.kiro/specs/constraint-guided-compression/prototype-gate.md));
design approval remains owner-gated.

## Prior same-day milestone: attempt-2 descope passes the full gate — verdict proceed (2026-11-07)
Owner descope decision executed as attempt 2 of 2, preregistered before any code
change ([R31](2026-11-07/05-descoped-stratum-rerun.md), plan §9): the Rust digest
twin left the prototype (Rust keeps self-contained sidecar admission), Python owns
digests. Both scopes re-produce the frozen identities byte-identically (E3
`92a48f77…` ×2 + three goldens zero divergence; E4 `0b2e383d…` ×2 at exactly
5,479,067 bytes + golden-a zero divergence), both candidates accepted as
preregistered (level2_ops=0), 9/9 controls fail closed, and every frozen target
passes: **C_app = 1,193 ≤ 1,364 (ratio 0.6997), saved 512 ≥ 200, C_total 2,691 ≤
3,003, per-scope smaller, U7 1,193 ≤ 1,400** (attempt-1's 1,808 retained; like-
for-like sensitivity 272 ≥ 200). Evidence review approved inline with recorded
residual risks (`results/s10-evidence-review-2.json`); **gate verdict proceed** —
only the demonstrated slice may seek normal production design approval, which
stays owner-gated. Attempt-1 history: [R30](2026-11-07/04-s10-shared-core.md)
(reuse proven mechanically, U5/U7 failed as frozen, C_app 1,808); S9 in
[R29](2026-11-07/03-s9-surface-contracts.md) (contract B ratified); S2 in
[R28](2026-11-07/02-s2-artifact-admission-digest-parity.md) (admission + Rust
digest parity, U7 ceiling raised to 1,400 by owner mid-run).

## Proposal: solution-first native slice (2026-11-07)
[R27](2026-11-07/01-solution-first-native-slice.md): owner-requested spec amendment,
U1-U7 before production expansion, updated 2026-11-07 with empirical canonical baseline
audit (`2,222` written -> `3,963` canonical lines across 14 files; 3-stratum `U5`
accounting; `U7` brownfield/verbatim-extraction rules) and Rust library audit (`Cargo.lock`
pins kept; `serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`,
`toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added).
Prototype plan frozen and owner-approved 2026-11-07; execution started (see R28).
Historical H8 evidence below remains unchanged.

Current direction (2026-11-06): H7's static state models and family-duplication
selection did not verify or compress real scattered code. H8 proposes a recorded,
mutation-scored trace oracle, edge-based convolution measures and regeneration
behind the public API. Gates run so far (R20 to R24): oracle mechanics positive
on Rust and Python, gain predictions failed, edge selector rejected, solver rung
rejected as equivalence authority but useful as a verified counterexample
finder. Regeneration has run on two scopes. E3 (R25, 2026-11-04): one round met
the oracle with 16% fewer lines and 29% fewer edges, mutation sensitivity missed
its bar (0.643; the original scores 0.654 on the same arm) and one oracle-blind
difference on an exception path is real. E4 (R26, 2026-11-06), the first of the
three scopes the report's section 8 preregistered, run with the recorded
regenerator program on the lifecycle monitor group: accepted at turn 3, 25%
fewer functions and 29% fewer edges inside the predicted bands, the line
prediction failed (6% fewer canonical lines against 10 to 20), sensitivity 0.679
inside its band with the original at 0.414 on its own sample, seven oracle-blind
differences on error paths, zero divergence on 74 held-out sequences (26 of
them the acceptance seed sequences replayed again, 47 new; 1 generated sequence
duplicates an acceptance one), the differential void on a failed solver
control. H8(c) stands on two scopes with a lowered protection claim and one
failed prediction; the owner decides between a
third scope, adoption of the E4 candidate outside the loop, or prevention. Four
protocol amendments are proposed from E4 (the surface includes the recorder's
imports; turn parameters preregistered; hashes over bytes as stored; labels
distinct beyond letter case). The loop as prototyped is documented
algorithm-agnostically in the
[deconvolution-loop spec](../../.kiro/specs/deconvolution-loop/requirements.md)
with per-requirement feasibility status. The earlier CodeQL direction is history.

## Dated lab reports

Report for third parties (2026-11-04, no new evidence):
[Regenerate behind the surface](../reports/deconvolution-loop-applied.md),
self-contained: setting, the loop in eight steps with definitions, results
R20 to R25, lessons, limitations, a falsifiable prediction for the next
three scopes, the loop's place in spec-driven development, glossary and
references.

Latest proposal: [R19: dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md).
Owner review of 2026-11-03 recorded; static dataflow and LLM-authored SMT models
withdrawn as equivalence authority; H8 registered with kill criteria per stage.

Latest replacement: [R18: provider representation](2026-11-02/18-provider-representation-trial.md).
Rejected: 30 fewer named-argument bindings but 12 more lines and 10 more statements;
70 selected tests pass on each version and 32 provider observations match.

Latest screen: [R17: provider orchestration](2026-11-02/17-provider-orchestration-screen.md).
Common algorithms are already shared; the narrow orchestration rewrite fails
admission. Repeated constructor fields remain an unresolved representation lead.

Latest trial: [R16: evidence-publication binding](2026-11-02/16-evidence-publication-trial.md).
Rejected: 1,256 → 1,259 AST statements across the complete accounting scope.
128 differential observations matched; lifecycle integration remains untested.

Latest discovery: [R15: CodeQL cross-module cases in Facdrone](2026-11-02/15-codeql-cross-module-cases.md).
781 extracted files; three connected responsibility hypotheses, with state,
caller and behavior boundaries. Discovery evidence, not measured savings.

Latest modeling result: [R14: AALpy learns real snapshot behavior](2026-11-02/14-aalpy-state-learning.md).
Four states under copy-only observations, six with source reads; 2,144 bounded
sequences matched and six wrong merges were rejected. No compression claimed.

Latest deployment correction: [R13: Windows client, Cairn HTTP server](2026-11-02/13-windows-client-cairn-api.md).
Real cross-environment HTTP coordination passed; Windows remains the agent runtime.

Latest: [R12: direct Cairn adoption prototype](2026-11-02/12-cairn-direct-prototype.md).
Upstream API/storage and local execution tests pass; LLM search and compression
remain untested. R11 is a search limitation, not proof of uncompressible code.

Folders group the discussion or research day in Europe/Berlin. Each entry records
its actual writing date separately when backfilled. Execution dates, when known,
remain separate; file modification times are not treated as execution evidence.

| Discussion / record day | Record | Status |
| --- | --- | --- |
| 2026-11-07 | [33: H8/H9 + DSPy-style probabilistic turn refinement design](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md) | planned: `constraint-guided-compression` reconciled around completed `capability-compression-core` + `deconvolution-native-core` + proven loop apparatus (`surface` Contract B, 3-stratum `measures`, `editscope`, `testinfra`, `regen_turn`) and extended with H10 DSPy-style `Refine`/`BestOfN` two-tier reward/assertion iteration for weaker or non-deterministic models; H7 static-CodeQL/authored-SMT pipeline withdrawn from design; demonstrated slice (`proceed`) separated from H10 multi-rollout slice (`revise`, gated by `G1-R`); `design.md` generated, owner approval pending |
| 2026-11-07 | [32: Rust specimen detection boundary](2026-11-07/06-rust-specimen-testinfra-detection.md) | completed: deconvolution-loop task 2.3 (optional, outside the gate's Python scope, owner go) — `testinfra.py` unmodified on the E1 frozen mmhmem copy: exit 0, `runner: unknown`, 0 test files, 0 seeds, 0 parse errors; known runner is cargo test (ci.yml) with nextest via `.config/nextest.toml`; corpus 22 `tests/*.rs` + 18 inline modules + 128 `#[test]`; the nested `ci/` uv project (pytest ≥8.0.2, 9 `ci/scripts/test_*.py`) is invisible to root-config detection; one true positive: the `docker-compose.yml` store file fact; no Rust regeneration claim; last open loop task closed |
| 2026-11-07 | [31: Descoped replacement stratum — attempt 2](2026-11-07/05-descoped-stratum-rerun.md) | completed, **verdict proceed**: owner descope preregistered then executed (attempt 2 of 2) — Rust digest twin deleted, admission self-contained (415 canonical; it had imported pyjson); both scopes reproduce frozen identities (`92a48f77…`/`0b2e383d…` 5,479,067 bytes), candidates accepted as preregistered, 9/9 controls; C_app 1,193 ≤ 1,364 (0.6997), saved 512 (like-for-like 272), C_total 2,691 ≤ 3,003, U7 1,193 ≤ 1,400 (attempt-1 1,808 retained); U1-U7 pass; inline evidence review approved with residual risks; production still owner-gated |
| 2026-11-07 | [30: S10 shared core](2026-11-07/04-s10-shared-core.md) | completed with verdict **revise**: shared engine reproduced both frozen recorders byte-identically (E3 `92a48f77…`, E4 `0b2e383d…` 5,479,067 bytes), zero-divergence replays, both candidates accepted as preregistered, 9/9 adverse controls fail-closed, U4 freeze re-proven; U5/U7 FAILED as frozen (C_app 1,808 > 1,364, saved −103, U7 1,808 > 1,400); cost driver the Rust byte-parity core (1,046); owner decision required (descope ~1,072 / stop) |
| 2026-11-07 | [27: Solution-first native slice](2026-11-07/01-solution-first-native-slice.md) | planned (`solution_unlock=not-run`); spec amendment for U1-U7 before production expansion; Amendment 2 (2026-11-07): read-only canonical baseline audit (`2,222` written -> `3,963` canonical lines across 14 files; `e1b_oracle.py` 406->659, `e4_oracle.py` 534->1,258), 3-stratum `U5` accounting ($B_{\text{app}}, B_{\text{ops}}, B_{\text{seeds}}$), `U7` brownfield/verbatim-extraction rules, and Rust crate audit (`serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`, `toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added) |
| 2026-11-06 | [26: E4 lifecycle regeneration](2026-11-06/01-e4-lifecycle-regeneration.md) | second scope (seven lifecycle monitor files, 1,495 lines) with the recorded regenerator program; T1 held, accepted at turn 3 (turn 1 spent 128k tokens thinking, turn 2 failed on a surface gap: a name only the recorder imports); T2 failed (1,405 canonical lines against at most 1,345, 6.0% fewer); T3 held (52 functions for 69) and T4 held (63 edges for 89); T5 held (candidate 19/28 = 0.679 in the band 0.60 to 0.75; original 12/29 = 0.414 by its sample's composition, stated in advance; 0.70 unmet on both; M2 to M4 caught on both trees, M1 missed as predicted); T6 held on F1 and F3 from 128 review probes (26 disagreements, F1 to F6), 6.5 void under criterion 4 (the window solver control failed), compare and standing counterexamples kept as evidence (F7); T7 held, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one); S held (11 writes in shells, pure 0.519, patch check pass); residue corrected (`lifecycle_root.lifecycle_store` is reached, `bind_lifecycle` unreached and undeclared); missing-test report 28 entries; 771/900 lines; about $6.96 of model time (estimate); owner decision third scope, adopt or stop |
| 2026-11-04 | [25: E3 evidence regeneration](2026-11-04/02-e3-evidence-regeneration.md) | T1-T3 held in 1 round (1,274→1,074 lines, 660→546 statements, 84→60 edges, 11 writes all in shell, 36/51 pure); T4 failed on kill rate 18/28 = 0.643 (M1-M3 caught); T5 found one realizable oracle-blind difference (`capture_payload` absences after an exception) that reading and 300 random inputs had already exposed; reviewer caveat: 26 statements are packing; no kill criterion fired; 545/400 lines; owner decision E3b, E4 or stop. Amendment 2 (2026-11-06, task 3.2): the same arm on the original scores 17/26 = 0.654 (post hoc 0.773), one kill apart from the candidate, so T4 measures the arm's sensitivity on this module, not a loss from regeneration; five real corpus gaps shared by both trees |
| 2026-11-04 | [24: E1c-2 corrected solver rung](2026-11-04/01-e1c2-corrected-solver-rung.md) | S1-S4 failed, S5 held; kill criterion fired on a verified pipeline (4 verifier checks passed); 7 of 10 targets killed (4 by the reader-inclusive alphabet alone, 3 only by realised solver inputs), 3 unresolved, 0 proved equivalent; guided generation rejected as equivalence authority for DB-entangled predicates; owner decision E3 or stop |
| 2026-11-03 | [23: E1c concolic rung](2026-11-03/05-e1c-concolic-rung.md) | R1-R4 failed, R5 held; 1 of 10 targets solved, 1 of 8 killed; cause: driver encoding concretised strings and the oracle never calls the reader functions; corrected rerun pending owner decision |
| 2026-11-03 | [22: E1b Facdrone evidence-publication oracle](2026-11-03/04-e1b-evidence-publication-oracle.md) | Oracle byte-deterministic, catches M1-M3 incl. stored-representation change tests miss; kill rate 0.712 to 0.746 on 60 mutants, Q1 gap-halving failed; H8(a) rejected as formulated; random generation found 2 new behaviours; owner decision pending |
| 2026-11-03 | [21: E2 Facdrone edge census](2026-11-03/03-e2-facdrone-edge-census.md) | 2,275 responsibilities, 94.7% reached; C1/C3/C4 held, C2 and C5 failed; edge measures rejected as the E3 selector; recorder validity proven by diff; two Windows defects in specimen test support recorded |
| 2026-11-03 | [20: E1a session-bus trace oracle](2026-11-03/02-e1a-session-bus-oracle.md) | Oracle built, deterministic, catches all mutants; P2 gain prediction hit a 96% seed-test ceiling; Arm B partial; generated inputs found a real panic; owner decision pending |
| 2026-11-03 | [19: Dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md) | Proposal after owner review; H8 registered; E1 oracle feasibility and E2 edge census preregistered; not run |
| 2026-11-02 | [18: Provider representation trial](2026-11-02/18-provider-representation-trial.md) | Concrete replacement rejected for expansion; scoped behavior checks pass; no adopted savings |
| 2026-11-02 | [17: Provider-orchestration admission screen](2026-11-02/17-provider-orchestration-screen.md) | No rewrite admitted; existing shared helpers; constructor alternative deferred, metric limitation explicit |
| 2026-11-02 | [16: Evidence-publication binding trial](2026-11-02/16-evidence-publication-trial.md) | Candidate rejected: more statements, no adopted savings; full lifecycle not validated |
| 2026-11-01 | [Yesterday's seven experiment and decision entries](2026-11-01/README.md) | Retrospectively written 2026-11-02; original evidence linked |
| 2026-11-02 | [01: Prior experiments and hypothesis revisions](2026-11-02/01-retrospective.md) | Retrospective catalog of retained evidence; no new execution |
| 2026-11-02 | [02: CodeQL selection and setup](2026-11-02/02-codeql-direction.md) | Backend selected; CLI verified; extraction pending |
| 2026-11-02 | [03: Requirements and continuity anchor](2026-11-02/03-requirements.md) | Requirements generated for review; no new trial |
| 2026-11-02 | [04: Prototype-first Kiro gate](2026-11-02/04-prototype-first.md) | Real-code feasibility required before production tasks; gate not run |
| 2026-11-02 | [05: Provisional design and setup evidence](2026-11-02/05-provisional-design.md) | Requirements approved; query packs and real-code feasibility pending |
| 2026-11-02 | [06: CodeQL fixture extraction and replay](2026-11-02/06-codeql-extraction.md) | Revise: Python 13/13, TypeScript 12/13; callback omitted; no compression claim |
| 2026-11-02 | [07: Complete call census and state-flow checks](2026-11-02/07-codeql-call-census.md) | G0b: 9/9 checks each; unnamed-target omission fixed; full feasibility remains revise |
| 2026-11-02 | [08: Alias, overwrite and reachable exception boundary](2026-11-02/08-codeql-state-boundary.md) | G0c: runtime controls pass; both queries miss known alias flow; revise completeness assumption |
| 2026-11-02 | [09: Dependency checkpoints and controlled contrasts](2026-11-02/09-codeql-dependency-diagnostic.md) | Direct receiver restores flow but retains overwritten input; graph evidence is not an exact state model |
| 2026-11-02 | [10: Source-backed state model and adverse controls](2026-11-02/10-source-backed-state-model.md) | 16/16 observations per language; five wrong models rejected; real-code compression remains untested |
| 2026-11-02 | [11: Real-case admission screen](2026-11-02/11-real-case-admission-screen.md) | Fifteen files inventoried, thirteen overlapping leads; no qualifying replacement admitted or executed |

Older source, patches, metrics, charts and checks remain under `docs/experiments/`
and linked reports. Their paths and historical conclusions are preserved. New
narrative lab reports belong in `docs/research/YYYY-MM-DD/NN-topic.md`; place new
durable evidence beside the report or link an existing experiment directory.
Ignored `.runs/` output alone is not durable evidence.

## Hypothesis lineage

These IDs are retrospective navigation labels, not original preregistration IDs.
They do not replace trial identifiers in the original records.

| ID | Proposition | Current evidence / revision |
| --- | --- | --- |
| H1 | A shared synthesized kernel removes divergent flow | Works on verbose restricted fixtures; expands the compact control; no general source-equivalence claim |
| H2 | Local metrics and family factoring identify useful removal | Mixed and often marginal; evaluate the whole replacement |
| H3 | SMT feedback and complexity caps drive further compression | A bad proposal was caught; revised outcome classifier had no additional line/LLOC reduction |
| H4 | Shared framework knowledge and connected graphs improve proposals | Pilot benefit unestablished; graph rewrite increased LLOC; retain negative results |
| H5 | Complete capability scope exposes substantial redundancy | ATS/NPU positive manually selected cases; not automatic discovery efficacy |
| H6 | Native measurement plus agent discovery yields useful general compression | Native workflow exercised; local baselines and marginal trials do not establish this broader claim |
| H7 | Query-grounded, multi-pass constraint modeling drives substantial compression | Never ran end to end in 18 entries; static state models rejected on G0c/G0d; superseded by H8 as the equivalence approach |
| H8 | A recorded, mutation-scored trace oracle plus edge measures lets an LLM regenerate scattered responsibilities with fewer edges and equal behaviour | H8(a) oracle mechanics positive on mmhmem (R20) and Facdrone (R22) but the gain predictions failed on both, so H8(a) is rejected as formulated; H8(b) edge selector rejected as formulated (R21); H8(c) regeneration untested and not admitted under the preregistered gates; guided-generation rung failed as run (R23) and, corrected and verified, rejected for DB-entangled predicates while killing 7 of 10 residue targets (R24); H8(c) run as E3 (R25): one regeneration round met the oracle with 16% fewer lines and 29% fewer edges and a checked pure core, but mutation sensitivity missed its bar (0.643) and one oracle-blind difference is real, so H8(c) stands on one scope with a lowered protection claim; H8(c) run on a second scope as E4 (R26) with the recorded regenerator program: accepted at turn 3, 25% fewer functions and 29% fewer edges inside the predicted bands, canonical lines only 6% fewer (T2 failed), sensitivity 0.679 inside its band with the original at 0.414 on its own sample, seven oracle-blind differences on error paths, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one), the differential void on a failed solver control; H8(c) stands on two scopes with the line prediction failed on the second; owner decision pending: third scope, adoption of the E4 candidate outside the loop, or prevention |
| H9 | Shared recording, replay, acceptance and turn-input apparatus plus thin scope adapters reduces maintained apparatus across E3 and E4 while preserving observations | Registered in [R27](2026-11-07/01-solution-first-native-slice.md); Attempt 1 (`R30`) failed `U5`/`U7` (`C_app = 1,808`); Attempt 2 descope (`R31`) passed all `U1–U7` (`C_app = 1,193 <= 1,364`, saved 512) and production migration completed (`C_app = 1,203 <= 1,364`, 19/19 tests, 9/9 adverse controls fail closed) |
| H10 | A DSPy-style `Refine`/`BestOfN` probabilistic turn controller with a two-tier programmatic reward (fast static AST/surface/structure assertions before runtime trace replay) recovers weaker or non-deterministic model rollouts without weakening the frozen oracle | Registered in [R33](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md) and designed in [`constraint-guided-compression/design.md`](../../.kiro/specs/constraint-guided-compression/design.md); demonstrated single-turn/3-turn baseline apparatus (`proceed` from R25/R26/R29/R31/R32) separated from multi-rollout recovery (`revise`, gated by bounded spike `G1-R`) |

H7 extends scope-wide reasoning and reintroduces solver research for a concrete
purpose. It does not erase H1-H6 or restore previously dropped infrastructure.
Use the [retrospective](2026-11-02/01-retrospective.md) for evidence links and the
[current Kiro brief](../../.kiro/specs/constraint-guided-compression/brief.md)
for the proposed feature boundary.

## Record each expansion before calling it progress

1. Name the hypothesis, its predecessor and a disproof condition.
2. Freeze scope, source identity, evaluator, dependencies and usefulness target.
3. State which missing capability the proposed tool change addresses.
4. Run a bounded prototype; retain failures, counterexamples and intermediate models.
5. Report whole-replacement results and evidence limits. Distinguish candidate,
   verified candidate and adopted code; do not sum overlapping trials.
6. Link the result to the spec decision it informs. Approval of a backend or a
   license does not automatically approve requirements, design and tasks.

Use the [entry template](ENTRY-TEMPLATE.md). Each applicable spec links back here
and each lab entry links to its spec. Update this index in the same change as the
entry. New interpretations get a dated amendment; do not silently overwrite old
results or relabel an exploratory check as preregistered.

## COBESY handoff

Audience and anchor: the owner and agents continuing the research. Target action:
inspect the latest decision and its evidence before extending the tool. The
assumption to challenge is that more infrastructure, passes or lower complexity
must mean less maintained code. The owner welcomes early disproof; rejected
hypotheses are useful outcomes, not contributor failures.

Composition: outcome first, then hypothesis, method, evidence, limitations and
next decision. Keep findings and proposals separate. The date index supplies
chronology; individual reports lead with the decision. A future agent should be
able to reproduce the result without inheriting the conversation.

Validation: no pooled success rates across unlike scopes, no unsupported proof
claims, no bytes-as-benefit or inferred maintenance-hours savings. Quantitative
charts require actual data. Preserve test and apparatus accounting separately.
