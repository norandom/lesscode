# R33: Reconcile constraint-guided-compression around H8/H9 apparatus and DSPy-style probabilistic turn refinement

- Report date/time and timezone: 2026-11-07, Europe/Berlin
- Execution interval: not run; design discovery and specification amendment only
- Status: planned (`demonstrated_apparatus = proceed`; `probabilistic_dspy_refinement = revise` pending bounded spike `G1-R`)
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md),
  [requirements](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  [design](../../../.kiro/specs/constraint-guided-compression/design.md),
  [prototype gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md),
  [research notes](../../../.kiro/specs/constraint-guided-compression/research.md)
- Hypothesis ID and predecessor: H10 (probabilistic multi-turn assertion/reward refinement over the H8/H9 dynamic deconvolution substrate), following H8 ([R19](../2026-11-03/01-dynamic-deconvolution-direction.md), [R25](../2026-11-04/02-e3-evidence-regeneration.md), [R26](../2026-11-06/01-e4-lifecycle-regeneration.md)), H9 ([R27](01-solution-first-native-slice.md), [R31](05-descoped-stratum-rerun.md)) and [R32](06-rust-specimen-testinfra-detection.md)
- Type: exploratory design synthesis + preregistered prototype gate (`G1-R`) for probabilistic multi-turn recovery

## Decision

Owner reviewed the feasibility audit of `constraint-guided-compression` on 2026-11-07 and directed two actions:

1. **Reconcile `constraint-guided-compression` with the completed detection and verification cores** ([`capability-compression-core`](../../../.kiro/specs/capability-compression-core/spec.json) and [`deconvolution-native-core`](../../../.kiro/specs/deconvolution-native-core/spec.json)) and the demonstrated [`deconvolution-loop`](../../../.kiro/specs/deconvolution-loop/spec.json) apparatus (`surface.py` with Contract B from [R29](03-s9-surface-contracts.md), `measures.py`/`measures2.py`/`measures3.py`/`pureshare.py`, `editscope.py`, `testinfra.py` from [R32](06-rust-specimen-testinfra-detection.md), and `regen_turn.py` with the four [R26](../2026-11-06/01-e4-lifecycle-regeneration.md) amendments). Withdraw the falsified 2026-11-02 H7 proposal (static CodeQL state-flow models + LLM-authored SMT equivalence models + in-place family refactoring) from the active design surface while preserving its historical records.
2. **Adopt DSPy-style probabilistic prompt iterations (`dspy.Refine` / `dspy.BestOfN` with two-tier programmatic reward and assertion feedback)** in the regenerator design so that weaker models or non-deterministic sampling errors (such as E4 Turn 1 exhausting its output budget on thinking tokens or E4 Turn 2 omitting a recorder-imported surface symbol) are recovered automatically through bounded inner static-assertion retries and outer trace-oracle feedback turns rather than aborting the trial.

In accordance with [prototype-first policy](../../../.kiro/steering/prototype-first.md):
- The **Demonstrated Detection & Loop Apparatus Slice** (`surface`, `measures`, `testinfra`, `editscope`, and the deterministic single-turn `TurnRequest`/`Feedback` protocol wired to `lesscode_baseline.recording` and `lesscode-core`) carries `proceed` status from [R25](../2026-11-04/02-e3-evidence-regeneration.md), [R26](../2026-11-06/01-e4-lifecycle-regeneration.md), [R29](03-s9-surface-contracts.md), [R31](05-descoped-stratum-rerun.md), and [R32](06-rust-specimen-testinfra-detection.md).
- The **Probabilistic DSPy Refinement & Multi-Turn Recovery Slice (`H10`)** is labeled `provisional` (`feasibility = revise`) in [`design.md`](../../../.kiro/specs/constraint-guided-compression/design.md) and gated behind bounded prototype spike `G1-R` in [`prototype-gate.md`](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md). No production tasks for `H10` optimizer or multi-rollout claims may be generated or executed before `G1-R` passes, and design approval remains owner-gated.

## Hypothesis and falsification

**H10 (Probabilistic Two-Tier Turn Refinement):** Wrapping the typed regeneration signature (`RegenerateScopeSignature`) in a DSPy-style `Refine` / `BestOfN` controller governed by a deterministic two-tier programmatic reward function $R(\text{req}, \text{cand}) \in [0.0, 1.0]$:
- **Tier 1 (Fast Static Assertion Gate, $R \in [0.0, 0.49]$)**: Checks response completion (`stop_reason == "end_turn"`), JSON schema validity, `editscope` path confinement, Python `ast.parse` syntax validity, Contract-B `surface` symbol and `patched`-global preservation, and static `structure` (`pure`/`shell` write-site confinement) in milliseconds without invoking PostgreSQL or `pytest`. On failure, immediately backtracks within a bounded inner retry budget (`max_static_retries <= 3`), adjusting thinking/temperature parameters and injecting the exact static violation into the prompt.
- **Tier 2 (Runtime Trace Oracle & Canonical Measure Gate, $R \in [0.50, 1.0]$)**: Executes only when Tier 1 passes ($R \ge 0.50$). Runs `lesscode_baseline.recording` (`probe`, `run_groups`, `verdict`) and 3-stratum canonical/separated-form `measures`, feeding structured counterexample and measure-gap diagnostics (`first_divergence`, `measure_gaps`, `surviving_faults`, `counterexamples`) into the next outer turn (`max_turns <= 4`) and retaining the best trajectory (`BestOfN`) across rollouts.

**Predictions and falsification (`G1-R` kill criteria):**
1. **Static recovery without oracle spin-up**: Planted or natural Tier-1 failures (including E4 Turn 1's `stop_reason == "max_tokens"` and E4 Turn 2's missing `labeled_reason` surface symbol) are caught by Tier 1 in $< 2\text{ s}$ with zero specimen pytest/PostgreSQL invocations and formatted into deterministic Tier-1 feedback.
2. **Bounded recovery on weaker or non-deterministic model runs**: Across a bounded multi-turn run (`max_turns <= 4`, `max_static_retries <= 3`) on a frozen scope (E3 or E4), the `Refine` loop recovers from at least one failed rollout/turn to reach $R = 1.0$ (zero L1/L2 divergence, `structure.pass == True`, and preregistered edge/function/canonical-line targets met) without weakening any frozen oracle check.
3. **Fail-closed budget and no-progress stop**: When the token/turn budget is exhausted or `k = 2` consecutive outer turns fail to strictly improve $\max(R)$ or discharge a distinct obligation, the loop halts and reports `exhausted` separately from `accepted` or `rejected` (Req 4.5, 4.6).
4. **Apparatus footprint discipline**: Adding the unified detection/measures/probabilistic-regenerator modules to `src/lesscode_baseline/` keeps replaceable apparatus within the 3-stratum `U5` ceiling ($C_{\text{app}} \le 1,364$ canonical lines when replacing the corresponding baseline apparatus files) and adds zero new direct dependencies to `native/Cargo.lock` or `src/lesscode_baseline/` top-level imports.

**Disproof:** H10 is rejected if Tier-1 static checks permit an invalid-surface or out-of-scope candidate to reach the runtime arm, if additional turns degrade mutation sensitivity below the original's arm baseline on the same sample, if prompt/parameter adjustments alter or bypass the frozen acceptance oracle, or if `Refine` retries exhaust the budget without improving $\max(R)$ over single-turn `regen_turn.py`.

## Frozen inputs and method

- **Completed production baselines reused unchanged**:
  - Native CLI `lesscode-core` (`prepare`, `compare`, `admission`, `budgets`; `native/src/{scope,metrics,compare,model,admission,receipt,budgets}.rs`, verified at commit `815eade`, nextest 123/123, `native/complexity-budgets.json` exit 0).
  - Shared Python trace recording/replay/acceptance engine [`src/lesscode_baseline/recording.py`](../../../src/lesscode_baseline/recording.py) (verified 19/19 in `tests/test_recording.py`, `C_app = 1,203 <= 1,364`).
- **Demonstrated prototype modules to consolidate into `src/lesscode_baseline/`**:
  - `surface.py` / `surface4.py` + `s9_surface_contracts.py` (Contract B: tree + recorder imports, 60–79 written lines).
  - `measures.py` (54 lines), `measures2.py` (46 lines), `measures3.py` (69 lines), `pureshare.py` (37 lines), `editscope.py` (65 lines).
  - `testinfra.py` (626 written lines, static pytest/fixture/marker/store/seed detector, validated in [R32](06-rust-specimen-testinfra-detection.md)).
  - `regen_turn.py` (337 written / 502 canonical lines, single-turn recorded regenerator).
- **DSPy v2.6+ pattern alignment**:
  - Modern DSPy (`>= 2.6`) replaces deprecated `dspy.Assert`/`dspy.Suggest` with `dspy.BestOfN(module, N, reward_fn, threshold, fail_count)` and `dspy.Refine(module, N, reward_fn, threshold, fail_count)`.
  - Our design implements the exact `(module, N, reward_fn, threshold, fail_count)` contract natively in `src/lesscode_baseline/regenerator.py` using Python standard library types at top level (supporting both `anthropic` and `openai`-compatible endpoints for weaker/local models), with an optional lazy-loaded `dspy` adapter for offline prompt optimization on training scopes strictly disjoint from held-out evaluation scopes (Req 7.5).

## Evidence and results

No new specimen execution was performed for this design amendment. Supporting empirical evidence from prior runs:
- **H7 disproof (why the 2026-11-02 design proposal had to be replaced)**: [R08](../2026-11-02/08-codeql-state-boundary.md) (`G0c` alias flow 0 rows), [R09](../2026-11-02/09-codeql-dependency-diagnostic.md) (`G0d` stale flow after overwrite), [R16](../2026-11-02/16-evidence-publication-trial.md) (1,256 -> 1,259 statements), [R18](../2026-11-02/18-provider-representation-trial.md) (+12 lines, +10 statements), [R19](../2026-11-03/01-dynamic-deconvolution-direction.md) (H7 withdrawn).
- **H8 regeneration & multi-turn failure modes motivating H10**:
  - [R25](../2026-11-04/02-e3-evidence-regeneration.md) (E3): 1 turn, 1,274 -> 1,074 lines (-16%), 84 -> 60 edges (-29%), 36/51 pure functions.
  - [R26](../2026-11-06/01-e4-lifecycle-regeneration.md) (E4): 3 turns with `regen_turn.py` — Turn 1 failed on `stop_reason == "max_tokens"` (128k thinking tokens, 0 text output); Turn 2 failed on missing recorder-imported surface name `labeled_reason` (motivating Contract B in [R29](03-s9-surface-contracts.md) and Tier-1 static surface assertion); Turn 3 accepted with 69 -> 52 functions (-25%) and 89 -> 63 edges (-29%).
- **H9 shared apparatus unlock**: [R31](05-descoped-stratum-rerun.md) & Task Group 3 (`815eade`): `C_app = 1,203 <= 1,364`, 502 canonical lines saved, 9/9 adverse controls fail closed.

## Limits and interpretation

- Consolidating the proven detection/measures/turn modules and adding a DSPy-style `Refine`/`BestOfN` loop addresses **generation fragility and wasted runtime evaluations** on weaker or non-deterministic models; it does **not** by itself increase the recorded trace oracle's coverage of unexercised error paths (which remains governed by the seed corpus and mutation/property amplification).
- A candidate accepted by `Refine` at $R = 1.0$ still carries only "measured protection on corpus $C$ with sensitivity $m$ and declared residue", never unbounded equivalence.
- Offline DSPy prompt optimizers (`MIPROv2`, `BootstrapFewShotWithRandomSearch`) require separate training scopes and must never tune against held-out acceptance scopes (Req 7.5).

## Tool/spec consequence and next gate

- `.kiro/specs/constraint-guided-compression/` is updated with [`design.md`](../../../.kiro/specs/constraint-guided-compression/design.md), amended [`brief.md`](../../../.kiro/specs/constraint-guided-compression/brief.md), [`requirements.md`](../../../.kiro/specs/constraint-guided-compression/requirements.md), [`prototype-gate.md`](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md), [`research.md`](../../../.kiro/specs/constraint-guided-compression/research.md), and [`design-review.md`](../../../.kiro/specs/constraint-guided-compression/design-review.md).
- `spec.json` advances to `phase: "design-generated"` with `approvals.design.approved: false` (awaiting owner review).
- Next gate: owner review of `design.md`; bounded execution of spike `G1-R` (Tier-1 static assertion recovery + multi-rollout `Refine` on frozen E3/E4 fixtures) before unlocking production tasks for the probabilistic optimizer slice.

## COBESY check

Outcome first: H7 design proposal replaced by H8/H9 + H10 DSPy-style two-tier probabilistic refinement design; demonstrated vs. provisional slices explicitly separated. Disproof conditions and `G1-R` kill criteria are preregistered before any new model run.
