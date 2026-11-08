# Prototype feasibility gate

## Rust-port U5 re-measurement outcome, 2026-11-08 ([R36](../../../docs/research/2026-11-08/03-rust-port-u5-remeasure.md))

Executed under task 8.6 (commit `7657969`; gate record
[`rust-u5-gate.json`](../../../docs/experiments/deconvolution/probabilistic-refine/results/rust-u5-gate.json)
+ `.sha256`): the deterministic G1-R checklist re-discharged **through the
Rust substrate — items 1, 2, 3 and 5 all PASS** (six Tier-1 controls fail
closed with zero runtime; E4 three-turn replay at the recorded scores with
the frozen E3/E4 digests and traces reproduced; k=2 exhaustion retains best;
every receipt admission-verified), and **U5 fails as measured: 7,049
canonical lines vs the unchanged 2,270 ceiling** (new modules 6,382; base
re-measured 452; shim 178; hooks 16 + 21). The smaller-replaceable-
representation hypothesis is falsified on the target substrate — the port is
≈2.9× the Python stage-1 canonical, not smaller. Per the preregistered
outcome rule no target is weakened and the negative is retained: Slice B
production/adoption stays withheld (`Req 7.10`) and the disposition (slim the
apparatus or amend the ceiling) returns to the owner as a dated decision.
Validation at the measurement commit: nextest 236/236, doc tests, clippy,
fmt, budgets exit 0, Python suites 77/78 (defect 7 unchanged).

## Rust-port U5 re-measurement preregistration, 2026-11-08 ([R36](../../../docs/research/2026-11-08/03-rust-port-u5-remeasure.md))

Frozen before any port code is written (owner go on the task_7_handover next
step; disposition R35 above). This gate re-measures U5 on the Rust target
substrate under the same expanded-baseline accounting:

- **Counted replaceable apparatus (Rust substrate)**: `native/src/{surface,
  measures,recording,regenerator}.rs` (the ported modules, full canonical
  lines) plus `native/src/{admission,receipt}.rs` re-measured live, plus
  apparatus registration hooks = the R31-frozen 16 hook lines plus the lines
  this task group adds to `native/src/{lib,main}.rs` / `native/Cargo.toml`
  (enumerated hunks in the receipt; product-only changes between R31 and now
  are excluded — they are not apparatus). The Python modules
  `src/lesscode_baseline/{recording,surface,measures,regenerator}.py` remain
  in-tree as the stage-1 reference implementation and are excluded as
  superseded, symmetric to the g1r recording-era-Python exclusion. Baseline
  $B_{\text{app}}^{\text{expanded}} = 2,838$ and ceiling $\le 2,270$ canonical
  lines are unchanged; `testinfra.py` stays outside U5 under U7 verbatim
  relocation.
- **Pre-port base (dated correction, 2026-11-08, before any port code)**: the
  g1r `FROZEN_RUST_CANONICAL = 1,333` constant cites "R27/R31 records" but no
  per-file receipt decomposes it; the live receipted count of
  `admission.rs` + `receipt.rs` is 426 canonical (rustfmt method) plus the
  R31-frozen 16 hook lines = 442 (`results/rust-port-base.json`), and R31's
  own receipt recorded 431 before `receipt.rs` existed. The R34 verdict is
  unchanged under every reading (2,450 + 442/583/1,333 all exceed 2,270); the
  re-measurement uses the live receipted counts per the method above, and the
  new-module budget is accordingly $2{,}270 - 442 = 1{,}828$ canonical lines,
  replacing the earlier $\approx 937$ estimate recorded before the base
  receipt existed. Recorded openly as an apparatus-accounting defect note in
  R36; no ceiling or baseline changes.
- **Canonical method (unchanged)**: Rust files rustfmt-formatted, counted by
  `splitlines()`; embedded Python shim code counted under the Python rule
  (`ruff==0.17.3 format --isolated --line-length 100 --target-version py313`);
  hooks by enumerated git-diff hunks. Receipt retains per-file counts and the
  exact commands.
- **Reuse boundary**: pre-existing product modules (`scope`, `metrics`,
  `compare`, `model`, `budgets` and their lines) are uncounted product, exactly
  as the Python substrate's use of `lesscode-core` bindings was uncounted; the
  port may call them. New shared code written for the apparatus counts in full
  wherever it lives (Req 6.1). Experiment drivers and tests stay uncounted per
  the g1r convention (separate strata).
- **Behavioral boundary (a port, not a sketch)**: the deterministic G1-R
  checklist items 1, 2, 3 and 5 must re-discharge **through the Rust
  substrate** on the same frozen artifacts (six Tier-1 controls fail closed
  with zero runtime invocations; the E4 three-turn replay caught/accepted at
  the recorded scores; `k=2` exhaustion retains best; every emitted receipt
  admission-verified via `bindings.json`), with Tier-2 replay reproducing the
  frozen identities. Live weaker-model recovery is not re-run by this gate: it
  remains demonstrated on the Python substrate (R34 run B-2), and no live
  transfer is claimed; if U5 passes, live runs on the Rust substrate belong to
  Slice B adoption planning.
- **Dependency decision (Req 3.6)**: the port adds pinned `ureq` for the
  openai-compatible HTTP adapter, because the replaced baseline scope includes
  the `regen_turn.py` HTTP/driver portion (262 canonical lines in
  $B_{\text{app}}^{\text{expanded}}$); omitting the adapter would silently
  narrow the claim. Pin exact version, `--locked`, record in the receipt.
- **Outcome rule**: report the measured total as pass/fail without weakening.
  Fail returns to the owner (slim or amend options); pass unlocks Slice B
  adoption planning only. Preregistered expectation: the four Rust modules
  must fit within the receipted new-module budget of $2{,}270 - 442 =
  1{,}828$ canonical lines (pre-port base receipt); both outcomes are
  credible and neither is assumed (R35: unknown is not pass).

## U5 disposition amendment, 2026-11-08 ([R35](../../../docs/research/2026-11-08/02-u5-disposition.md))

Owner decision (2026-11-08): U5 is **accepted as failing on the Python stage-1
prototype substrate** — 3,783 canonical lines vs the frozen ceiling 2,270
(L29 below, unchanged; `Req 7.8` negative retained, no target weakened). No
slimming pass and no ceiling amendment is authorized at this stage. Python
remains the stage-1 prototype; Rust is the target substrate — re-measure on
port, never assume transfer. Slice B production/adoption tasks stay withheld
(`Req 7.10`) until a **Rust-port re-measurement of U5** under the same
expanded-baseline accounting passes the frozen ceiling, or a further dated
owner decision records a different disposition.

## G1-R outcome amendment, 2026-11-08 ([R34](../../../docs/research/2026-11-08/01-g1r-gate-and-live-recovery.md))

Executed under tasks 6.1/6.2/7.1 (`constraint-guided-compression`, commits
`1878947`, `9d9cf72`): every deterministic G1-R checklist item passed (six
controls fail closed with zero runtime; E4 replay Tier-1 checks at
0.001–0.06 s; `k=2` exhaustion retains best; all receipts admission-verified),
and the live/replay recovery check is satisfied by weaker-model run `B-2`
(`claude-haiku-4-5`: Tier-1 `out_of_scope` 0.20 → same-turn acceptance
R=1.0, oracle 7/7, within `max_turns<=4` / `max_static_retries<=3`; 11 runs
retained incl. negatives). **U5 fails as measured**: 3,783 canonical lines vs
the frozen ceiling 2,270 (L29 below, unchanged). Per Req 7.8 the negative is
retained without weakening targets; the U5 disposition (accept as Python
stage-1 prototype with a Rust re-measure gate, slim, or amend) is an open
owner decision, and Slice B production/adoption tasks remain withheld
(Req 7.10). Phase approvals are unchanged by this amendment.

## DSPy-style probabilistic turn refinement & H8/H9 reconciliation amendment, 2026-11-07 ([R33](../../../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md))

Following `R19`–`R32` and the owner's 2026-11-07 review, this spec's feasibility gate is partitioned into two explicit slices matching [design.md](design.md):

1. **Slice A — Demonstrated Detection, Measurement & Single-Turn Loop Apparatus (`status: proceed`)**:
   - **Evidence established in real-code runs**:
     - [R25](../../../docs/research/2026-11-04/02-e3-evidence-regeneration.md) (E3, 5 Facdrone modules, 1,274 lines $\to$ 1,074 lines, $-16\%$, edges $84 \to 60$, $-29\%$, 36/51 pure functions) and [R26](../../../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md) (E4, 7 Facdrone modules, 1,495 lines $\to$ 1,405 canonical lines, functions $69 \to 52$, $-25\%$, edges $89 \to 63$, $-29\%$, accepted at Turn 3 with `regen_turn.py` and zero divergence on 74 held-out sequences).
     - [R28](../../../docs/research/2026-11-07/02-s2-artifact-admission-digest-parity.md), [R29](../../../docs/research/2026-11-07/03-s9-surface-contracts.md), and [R31](../../../docs/research/2026-11-07/05-descoped-stratum-rerun.md) + Task Group 3 (`815eade`): Contract B surface derivation (`surface.py` + recorder imports resolves E4 Turn 2's `labeled_reason` failure), 3-stratum canonical and separated-form measures (`measures.py`, `measures2.py`, `measures3.py`, `pureshare.py`), edit-scope and patch-sentinel checks (`editscope.py`), and shared recording/acceptance core (`src/lesscode_baseline/recording.py`, $C_{\text{app}} = 1,203 \le 1,364$, 502 canonical lines saved, 9/9 adverse controls fail closed).
     - [R32](../../../docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md): static test-infrastructure detector (`testinfra.py`) validated on Facdrone (task 2.2) and boundary-verified on Rust `mmhmem` (task 2.3).
   - **Eligibility**: Consolidating `surface.py`, `measures.py`, `testinfra.py`, and the deterministic `TurnRequest`/`Feedback` protocol into `src/lesscode_baseline/` has passed real-code feasibility (`proceed`) and is eligible for production task generation once the owner approves `design.md`.

2. **Slice B — DSPy-Style Probabilistic Multi-Turn Refinement (`H10`, `status: revise` pending bounded spike `G1-R`)**:
   - **Mechanism under test**: Wrapping `RegenerateScopeSignature` in a DSPy-style `Refine` / `BestOfN` loop (`src/lesscode_baseline/regenerator.py` + optional `DspyRefineAdapter`) with a two-tier programmatic reward function $R(\text{req}, \text{cand}) \in [0.0, 1.0]$:
     - **Tier 1 (`StaticAssertionGate`, $R \in [0.0, 0.49]$)**: Fast ($< 2\text{ s}$, zero DB/pytest spin-up) checks for `stop_reason == "end_turn"`, JSON schema validity, `editscope` path confinement, Python `ast.parse` syntax, Contract-B `surface` symbol + `patched`-global preservation, and `measures` `pure`/`shell` write-site confinement. Failed rollouts trigger immediate inner backtracking (`max_static_retries <= 3`) with parameter recovery (thinking-budget cap on `max_tokens` stop; `rollout_id` / temperature jitter on syntax or surface errors) and structured violation feedback.
     - **Tier 2 (`RuntimeOracleGate`, $R \in [0.50, 1.0]$)**: Executed only when $R \ge 0.50$. Runs `lesscode_baseline.recording` (`probe`, `run_groups`, `verdict`) and 3-stratum canonical/separated-form `measures`. Outer turns (`max_turns <= 4`) feed `first_divergence` and `measure_gaps` into the next turn and track the best candidate via `BestOfN`.
   - **Frozen `G1-R` spike requirements before unlocking Slice B production claims**:
     - **Inputs**: Frozen E3 (`92a48f77…`) and E4 (`0b2e383d…`) scopes, Contract B `public-surface.json`, and retained E4 Turn 1 (`stop_reason == "max_tokens"`), Turn 2 (missing `labeled_reason` surface symbol), and Turn 3 (accepted) artifacts.
     - **Adverse controls (must all fail closed in Tier 1 with $R < 0.50$ and zero runtime pytest/PG invocations)**:
       1. `ctrl_max_tokens`: truncated thinking response (`stop_reason == "max_tokens"`).
       2. `ctrl_out_of_scope`: candidate writes a file outside `scope_files`.
       3. `ctrl_syntax_error`: candidate introduces a Python `SyntaxError`.
       4. `ctrl_missing_surface_b`: candidate omits `labeled_reason` (Contract B recorder-imported symbol) or locally rebinds a `patched` global.
       5. `ctrl_write_in_pure`: candidate places an `.execute(` or `.commit(` call inside a function declared in `structure.pure`.
        6. `ctrl_feedback_leak` (added by dated amendment, 2026-11-07, owner-approved via the independent fusion design-review gates): a feedback payload containing a held-out sequence name is rejected before dispatch (`Req 7.5`).
     - **Live / replay recovery check**: Verify that `Refine` transitions deterministically from `ctrl_max_tokens` (Turn 1) $\to$ `ctrl_missing_surface_b` (Turn 2, caught in Tier 1 without running PostgreSQL) $\to$ accepted Turn 3 ($R = 1.0$), and in at least one non-deterministic or weaker-model multi-turn run recovers from a Tier-1 or Tier-2 failure to reach $R = 1.0$ within `max_turns <= 4`, `max_static_retries <= 3`.
     - **Budget & no-progress stop**: Confirm that `k = 2` consecutive turns without strict improvement in $\max(R)$ or a newly discharged obligation halts with `status = "exhausted"` (`Req 4.5, 4.6`).
     - **Whole-replacement apparatus target (`U5` & `U7` expanded baseline)**: Consolidated `src/lesscode_baseline/{recording,surface,measures,regenerator}.py` plus `native/src/{admission,receipt}.rs` must keep replaceable apparatus within the 20% reduction ceiling against the expanded baseline apparatus stratum $B_{\text{app}}^{\text{expanded}} = 2,838$ canonical lines ($20\%$ ceiling $\le 2,270$ canonical lines; composed of the R27/R31 baseline $B_{\text{app}} = 1,705$ plus the newly consolidated pre-consolidation baseline scripts: `surface.py` $117 +$ `surface4.py` $119 +$ `s9_surface_contracts.py` $117 = 353$, `measures.py` $129 +$ `measures4.py` $93 +$ `measures2.py` $99 +$ `measures3.py` $117 +$ `pureshare.py` $57 +$ `pureshare4.py` $23 = 518$, and the previously excluded `regen_turn.py` HTTP/driver portion $502 - 240 = 262$; while `testinfra.py` at $1,081$ canonical lines is relocated verbatim into `src/lesscode_baseline/testinfra.py` under `U7` verbatim-relocation accounting, and irreducible `op_*` / `SEEDS` strata remain excluded).

---

## Representation trial amendment, 2026-11-02

[R18](../../../docs/research/2026-11-02/18-provider-representation-trial.md) follows
PO01 with an actual shared snapshot-projection candidate. Named keyword bindings
fall by 30, but total scope grows by 12 lines and 10 statements. Both versions pass
70 selected original tests and match 32 provider-load observations; a wrong policy
mutant is detected. Reject for usefulness; no adoption. This manually constructed
candidate does not establish the full constraint-modeling loop. Keep `revise`.
Facdrone is now authorized as an editable experimental specimen; preserve before/
after evidence and do not commit/push upstream without instruction.

## Provider admission amendment, 2026-11-02

[R17](../../../docs/research/2026-11-02/17-provider-orchestration-screen.md) screens
the provider orchestration case without rewriting it. Its three load methods
contain 37 AST statements; including series builders gives 60, below a 70-statement
net reduction target even with zero replacement cost. Core operations already
share helpers. Reject the narrow orchestration proposal under this gate, not the
whole capability. Repeated constructor field plumbing remains deferred: statement
counts underrepresent it, requiring a separately declared representation measure
before another trial. No behavior tests or new model ran; feasibility is `revise`.

## Binding trial amendment, 2026-11-02

[R16](../../../docs/research/2026-11-02/16-evidence-publication-trial.md) executes
one isolated candidate within R15's evidence-publication scope. Whole-scope AST
statements increase from 1,256 to 1,259; 128 differential binding observations
match and a wrong-schema mutant differs in 20. Reject for usefulness; no adoption.
Database lifecycle and complete caller integration were not validated because
the preregistered reduction gate failed first. This is not the proposed stateful
constraint-modeling loop. Gate remains `revise`. Shared-callee connectivity is
insufficient admission evidence: identify the maintained operation that can
actually disappear, including the support needed to preserve distinct contracts.

## Cross-module discovery amendment, 2026-11-02

[R15](../../../docs/research/2026-11-02/15-codeql-cross-module-cases.md) uses CodeQL
on 781 frozen Facdrone source files and develops three real responsibility-level
cases. Prefer evidence-publication lifecycle for the stateful modeling trial;
provider orchestration is a lower-cost alternative. No synthetic detour is needed
to locate candidates. Candidate scopes include incoming/outgoing frontiers and
must be closed before replacement measurement. Partial call resolution and the
empty targeted API-graph query remain explicit limits. No replacement ran;
feasibility remains `revise`.

## Model-learning amendment, 2026-11-02

[R14](../../../docs/research/2026-11-02/14-aalpy-state-learning.md) records direct
AALpy adoption on native Windows against a restricted real Lesscode snapshot
lifecycle. Four learned states became six when source reads were added; all
2,144 bounded validation sequences matched and all six baseline pair-merge
controls were rejected by concrete replay. Operations/observations were manually
authored; no replacement or compression was attempted. Gate remains `revise`.
Freeze observers independently of candidate simplification. A smaller learned
graph under fewer observations is not evidence of removable production state.

## Direct-adoption amendment, 2026-11-02

Deployment clarification: [R13](../../../docs/research/2026-11-02/13-windows-client-cairn-api.md)
passes native Windows-to-WSL HTTP coordination. WSL hosts the deployment-test
server only. Windows agents may drive existing claim/evidence APIs directly;
server reuse alone does not exercise upstream autonomous dispatcher search.
No WSL model runtime or native worker-launcher port is required for the next
manually scheduled API trial. All compression acceptance conditions remain.

Owner prefers prototyping with Cairn itself rather than implementing a similar
orchestrator. [R12](../../../docs/research/2026-11-02/12-cairn-direct-prototype.md)
records the pinned upstream integration trial and bounded model-search follow-up.
The protocol works; autonomous search and compression have not run. Gate remains
`revise`. R11's early admission screen is not evidence of uncompressibility.
Apply usefulness thresholds to complete candidate replacements, while allowing
bounded exploration of uncertain alternatives before savings are established.
Reuse Cairn's exploration graph; preserve independent Lesscode acceptance checks.

## Prior gate evidence

Status: `revise`. G0e passes the bounded synthetic model check, but the subsequent
G1 admission screen did not establish an untried, qualifying replacement.
Fifteen files were inventoried after selective source reading; no real-code
candidate was constructed or executed. No savings were measured.
[R11](../../../docs/research/2026-11-02/11-real-case-admission-screen.md) records the limitation.
Established 2026-11-02 by owner direction. This is a design
entry constraint, not a claim of feasibility and not a production task plan.
Policy: [prototype-first](../../steering/prototype-first.md).
Requirements: [7.9-7.11](requirements.md).
Decision history: [R04](../../../docs/research/2026-11-02/04-prototype-first.md).

## Proposal to establish during design

Before the first run, identify and freeze:

- A representative real-code capability and its source revision/snapshot,
  complete owned scope, relevant callers, state, effects and tests.
- The custom implementation expected to disappear, the proposed smaller
  representation and an estimated net reduction including integration overhead.
- Absolute and relative usefulness thresholds, justified for that scope. Do not
  reuse three-line reductions as the target for a large responsibility.
- Evidence needed from CodeQL queries, the LLM's model and revision passes,
  constraint checking and original/candidate executions. Exercise the proposed
  interaction rather than manually selecting an unrelated easy edit.
- Semantic exclusions, adverse controls, baseline check handling, bounded
  pass/query/solver budget and a no-progress stop condition.

These fields are deliberately unset until design identifies the actual specimen.
No candidate generation can count as a preregistered trial before they are fixed.

## Gate progression

First challenge extraction on small labeled fixtures. If the needed relationships
are absent or silently wrong, revise or stop before modeling the real scope.
Then exercise a minimal end-to-end real-code prototype. Hand-operated orchestration
is acceptable if all interventions are recorded and the model/query/verification
steps being claimed actually run. A mature autonomous service is not required.

For `proceed`, link a concrete, behavior-checked replacement meeting the frozen
net-compression target and a report explaining why the proposed mechanism can
produce it. Keep modeling fidelity distinct from solver satisfiability and tests.
Do not claim full source equivalence from a restricted arithmetic model.

For `revise` or `stop`, retain the failed attempt, defect/counterexample or missing
evidence. Do not generate or execute downstream production tasks. A new bounded
proposal may change scope with an explicit dated rationale and fresh targets.

## Current evidence

G1 admission amendment, 2026-11-02: manual screening covered thirteen overlapping
responsibility leads in fifteen files across Lesscode and authorized examples.
No concrete complete-scope replacement met the retained discovery screen of
25% and 30 net statements. This is not proof of minimality and not a failed
candidate comparison; admission did not occur. Do not expand graph/solver/agent
infrastructure to substitute for the missing deletion hypothesis. See R11.

G0e amendment, 2026-11-02: agent-authored explicit identities and ordered writes
match the frozen six plus ten supplemental observations per language. Five
deliberately wrong models are rejected; three match retained source mutants.
Replay is identical. The author saw the oracle; there was no independent or
blind model discovery, solver proof or code reduction. Next is a bounded real
responsibility with a frozen replacement hypothesis and usefulness target,
not expansion of this fixture model into a general analyzer. See R10 above.

G0d amendment, 2026-11-02: original and inline-alias variants lose input flow
after the field write. Changing only the first write receiver to the original
object restores both the first observation and an imprecise post-overwrite
flow, in Python and TS. Both source variants pass the frozen six runtime cases
per language. Unchanged focused queries confirm the contrast; original/direct
replay matches. Stop after this informative contrast; no third variant ran.
Next: challenge an explicit source-backed ordered-state model with the frozen
controls. No general analyzer expansion, real-code reduction or SMT proof is
established by this diagnostic. See R09 above.

G0c amendment, 2026-11-02: six original runtime cases pass per language,
including reachable exceptional cleanup. All three adverse variants per language
are detected. Both static assessments report 9/10 but fail the essential alias
flow positive control; zero flow rows make the passing negative assertions
insufficient evidence of precision. All query commands succeeded without repair.
Next: bounded first-missing-edge diagnosis, with source/runtime evidence kept
separate from the graph. Do not advance production design or tasks on this score.

G0b amendment, 2026-11-02: complete syntactic invocation census plus optional
target rows retains the unnamed TypeScript callback. Both languages pass nine
checks (three output-presence checks, six fixture assertions). Input propagates
through state helpers to the callback; unrelated constant flow is excluded.
CFG connectivity does not establish executable exception behavior: this
fixture's throw condition is false. Full source-model fidelity and useful
real-code compression remain unestablished. See R07 above.

Earlier G0 amendment, 2026-11-02: query libraries are pinned and compile; both
databases extract. Python passes 13/13 fixture assertions, TypeScript 12/13.
Replay matches all query outputs, including the TS omission. State-value/alias
and exception-path checks remain unestablished. No real-code modeling or
compression result exists. Preserve the earlier discovery state below as history.

Design discovery, 2026-11-02: [provisional proposal](design-proposal.md) recorded.
Python/JavaScript extractors resolve, but analysis query packs do not yet resolve.
Read-only inspection did not establish a threshold-qualified untried Lesscode
target. Analyzer orchestration is a lead, not a compression forecast. The complete
mechanism is untested; gate remains not-run. See
[R05](../../../docs/research/2026-11-02/05-provisional-design.md).

CodeQL 2.26.3 runs locally. MIT licensing is recorded. Earlier manual scoped
compression trials establish examples worth studying, but they do not exercise
this proposed graph-grounded LLM modeling mechanism. No feasibility pass is claimed.

## Gate decision receipt

After the trial, record outcome, evidence links, actual versus expected reduction,
behavioral checks/adverse controls, manual interventions, unsupported scope,
remaining risks, and the next decision. Only `proceed` enables final design
approval and a production task plan. Formal phase approvals remain separate.
