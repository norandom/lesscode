# R36: Rust-port U5 re-measurement — preregistration (task group 8)

- Report date/time and timezone: 2026-11-08, ~08:50Z (preregistration; no
  execution yet); amendment 2 executed 2026-11-08 ~13:50Z
- Execution interval: port built 2026-11-08 (commits `7bb9653`…`bc9f34c`);
  gate measured 2026-11-08 at `7657969`
- Status: executed — measured FAIL (7,049 vs 2,270 canonical lines); items
  1/2/3/5 PASS; results in amendment 2 below
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md)
  `Req 7.8`, `7.10`; gate frozen as the
  [Rust-port U5 re-measurement preregistration amendment](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md);
  task group 8 in [tasks.md](../../../.kiro/specs/constraint-guided-compression/tasks.md)
- Hypothesis ID and predecessor: U5 re-measure gate; predecessor
  [R35](02-u5-disposition.md) (accept Python negative; re-measure on port)
- Type: preregistered

## Decision

Nothing is decided yet. This entry freezes the accounting, boundaries and
outcome rule before the first line of port code exists, so the re-measurement
cannot be tuned toward a pass. Per R35: pass unlocks Slice B adoption planning
(`Req 7.10`) only; fail returns to the owner with slim/amend options; no
target is weakened and no transfer of the Python 3,783-line result to Rust is
assumed in either direction.

## Hypothesis and falsification

Working hypothesis (owner framing, R35): the Python overage is partly a
prototype-substrate property — the loop apparatus is expressed in the same
Python it measures — so a Rust port consolidating onto the existing native
substrate (`ast-grep` tree-sitter-python, `receipt.rs` writer, `admission`,
`sha2` digests) may come in materially smaller than the 2,450 canonical
Python lines.

Falsifiable prediction: if the ported Rust replaceable apparatus —
`native/src/{surface,measures,recording,regenerator}.rs` full canonical +
`native/src/{admission,receipt}.rs` re-measured + apparatus hooks (R31-frozen
16 + this group's enumerated registration hunks) — still exceeds 2,270
canonical lines against the unchanged $B_{\text{app}}^{\text{expanded}} =
2{,}838$, then the smaller-replaceable-representation claim fails on the
target substrate too and Slice B adoption requires a new dated owner decision.
Preregistered arithmetic (pre-port base receipt): the four Rust modules must
fit within $2{,}270 - 442 = 1{,}828$ canonical lines; both outcomes remain
credible. Any parity failure (frozen digests/symbols/verdicts not reproduced),
any Tier-1 control passing, or any receipt failing admission also fails the
gate.

## Frozen inputs and method

- Gate definition: `prototype-gate.md` R36 amendment (counted set, canonical
  method, hooks enumeration, reuse and behavioral boundaries, dependency
  decision), frozen in the same commit as this entry.
- Canonical method (unchanged from R27–R31/R34): Rust files rustfmt-formatted,
  `splitlines()` counts; embedded Python shim under
  `ruff==0.17.3 format --isolated --line-length 100 --target-version py313`;
  hooks by enumerated git-diff hunks; per-file counts and exact commands
  retained in the receipt; unavailable readings reported unavailable, never 0.
- Reuse boundary: pre-existing product modules (`scope`, `metrics`, `compare`,
  `model`, `budgets`) are uncounted product, symmetric to the Python
  substrate's uncounted use of `lesscode-core` bindings; new shared code
  written for the apparatus counts in full wherever it lives (Req 6.1);
  drivers and tests stay uncounted per the g1r convention.
- Behavioral boundary: deterministic G1-R checklist items 1, 2, 3, 5 must
  re-discharge through the Rust substrate on the same frozen artifacts (E4
  turns `e4-turn1..3` with sha256 sidecars, `public-surface.json`, `e4_oracle.py`,
  frozen facdrone copy, frozen E3/E4 digests `92a48f77…` / `0b2e383d…`);
  Tier-2 replay reproduces frozen identities; receipts verify under
  `lesscode-core admission`. Live weaker-model recovery is not re-run by this
  gate (R34 run B-2 stands on Python; no live transfer claimed).
- Dependency decision (Req 3.6): add pinned `ureq` (`--locked`) for the
  openai-compatible HTTP adapter — the replaced baseline scope includes the
  `regen_turn.py` HTTP/driver portion (262 canonical lines), so omitting it
  would silently narrow the claim. Recorded here before introduction.
- Validation: `cargo nextest` + `cargo test --doc` (toolchain 1.98.1),
  `lesscode-core budgets` with the new modules honestly registered in
  `native/complexity-budgets.json`, Python suites unchanged (pytest-xdist
  from `D:\Source\lesscode`, venv `.runs/e2-facdrone`), PowerShell only.

## Evidence and results

**Preregistration amendment 1 (2026-11-08, pre-port): 1,333 constant
discrepancy.** The g1r spike's `FROZEN_RUST_CANONICAL = 1,333` cites
"R27/R31 records", but no per-file receipt decomposes it and no reconstructible
file set yields it: the live receipted count of `admission.rs` (274) +
`receipt.rs` (152) is 426 canonical (rustfmt, toolchain 1.98.1), plus the
R31-frozen 16 hook lines = 442 (`results/rust-port-base.json`, pre-port commit
`f2fe982`); R31's own receipt recorded Rust 431 + hooks before `receipt.rs`
existed (415 + 16), i.e. 415 + 152 + 16 = 583 at most under the widest
defensible reading. The R34 Python-substrate FAIL verdict is unchanged under
every reading (2,450 + 442/583/1,333 = 2,892/3,033/3,783, all > 2,270). The
re-measurement therefore uses live receipted counts per the frozen method;
the new-module budget is $2{,}270 - 442 = 1{,}828$. This is recorded as an
apparatus-accounting defect (class: unreceipted frozen constant), not
silently absorbed; the ceiling and baseline are unchanged. The unexplained
1,333 is retained as an open provenance question for the owner.

**Amendment 2 (2026-11-08, executed at `7657969`): U5 FAIL on the Rust
substrate as measured — 7,049 canonical lines vs the unchanged 2,270
ceiling.** Gate record:
[`results/rust-u5-gate.json`](../../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate.json)
(+ `.sha256`, receipt `rust-u5-gate-1`, sha256 `84b7ccc2…4610`), driver
`rust_u5_gate.py` (uncounted stratum), pre-port base `f2fe982`.

- **Deterministic re-discharge through the Rust substrate (items 1, 2, 3, 5):
  PASS.** Six Tier-1 controls fail closed with zero runtime calls
  (`controls_fail_closed_without_runtime`,
  `control_feedback_leak_rejected_before_dispatch`); the E4 three-turn replay
  reproduces the recorded transitions (t1 stop_reason 0.00, t2 surface 0.40
  missing `labeled_reason`, t3 Tier-1 0.50 + Tier-2 replayed 1.00, each
  < 2 s), corroborated by the frozen E3/E4 LF-body digest reproduction and
  zero-divergence trace replay through the Rust recording driver; k=2
  no-progress exhaustion halts `exhausted` with the best trajectory retained
  (new native test `loop_exhaustion_k2_halts_and_retains_best` covers the
  exact k=2 protocol; the prior test used k=1); every emitted receipt
  verifies under `lesscode-core admission` (exit 0). Live weaker-model
  recovery was not re-run: R34 run B-2 stands on Python, no live transfer
  claimed.
- **U5 measured under the frozen counted set**: `surface.rs` 1,154 +
  `measures.rs` 1,428 + `recording.rs` 1,002 + `regenerator.rs` 2,798 =
  6,382 new-module canonical; `admission.rs` 274 + `receipt.rs` 178 = 452
  base re-measured live; `recording_shim.py` 178 under the Python rule
  (ruff==0.17.3); R31-frozen hooks 16; task-group-8 registration hooks 21
  (`Cargo.toml` 2, `lib.rs` 4, `main.rs` 15; hunks enumerated in the record).
  **Total 7,049 vs ceiling 2,270 → FAIL by 4,779.** The preregistered
  new-module budget 1,828 was exceeded by 4,554 before base and hooks.
- **Interpretation (falsification confirmed):** the working hypothesis — a
  Rust consolidation materially smaller than the 2,450-line Python apparatus —
  fails on the target substrate: the port is ≈2.9× the Python stage-1
  canonical, larger in every component, not smaller. Per the frozen outcome
  rule: no target weakened, negative retained, unknown is not pass; Slice B
  production/adoption stays withheld (`Req 7.10`); the disposition (slim the
  apparatus or amend the ceiling) returns to the owner as a dated decision.
- **Validation at the measurement commit:** `cargo nextest` 236/236 (incl.
  the new k=2 test), `cargo test --doc`, clippy, `cargo fmt --check`,
  `lesscode-core budgets` exit 0; Python suites unchanged: pytest-xdist
  `test_regenerator.py` 59/59, `test_recording.py` 18/19 (defect 7 unchanged:
  `receipt.rs` 176 > 150). No frozen file or receipt changed; the only
  tracked-file side effect is the machine-written `written` timestamp line in
  the two `sql-divergences-round21.json` files (rewritten by the
  acceptance-arm reruns; both were already owner-WIP-modified). One transient
  intermediate run failed three acceptance arms with parent-process exit
  0xE0434357 after their inner suites passed — traced to the harness PATH
  deviation (venv Scripts omitted; defect #8 class), not code: the exact
  validation PATH passes.

## Limits and interpretation

The gate measures the replaceable apparatus on the Rust substrate; it does
not claim live-recovery transfer (Python B-2 stands), repository-scale
generality, or any adoption. A pass is a line-count plus behavior-parity
result on the frozen E3/E4 material only. Unknown is not pass.

## Tool/spec consequence and next gate

Next gate: the measured U5 outcome under this preregistration. Pass → Slice B
adoption planning (still owner-gated per `Req 7.10`); fail → owner decision
(slim the apparatus or amend the ceiling; no post-hoc weakening). The Python
stage-1 modules remain the reference implementation; the Rust modules become
the target-substrate apparatus.

## COBESY check

Outcome-first framing fixed in advance (report as measured, negatives safe);
disproof explicit (overage, parity failure, control leak or unverifiable
receipt fails the gate); preregistration precedes execution; facts and
interpretation separated; no adoption claimed from this gate alone.
