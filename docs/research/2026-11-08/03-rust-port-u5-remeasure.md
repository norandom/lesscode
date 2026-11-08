# R36: Rust-port U5 re-measurement — preregistration (task group 8)

- Report date/time and timezone: 2026-11-08, ~08:50Z (preregistration; no
  execution yet)
- Execution interval: not run — preregistration precedes all port code
- Status: planned (preregistered; results amend below with dated entries)
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

No other results yet. Port results will amend this entry with the per-file
canonical receipt
(`docs/experiments/deconvolution/probabilistic-refine/results/`), the gate
record and the measured pass/fail.

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
