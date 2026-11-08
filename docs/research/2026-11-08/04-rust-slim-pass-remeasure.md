# R37: Rust-apparatus slim pass and U5 re-measurement — preregistration (task group 9)

- Report date/time and timezone: 2026-11-08, ~14:35Z (preregistration; no
  execution yet)
- Execution interval: not run; no slim code exists yet
- Status: planned (preregistered)
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md)
  `Req 6.1`, `6.4`, `6.5`, `7.8`, `7.10`; gate frozen as the
  [Rust-apparatus slim-pass preregistration amendment](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md);
  task group 9 in [tasks.md](../../../.kiro/specs/constraint-guided-compression/tasks.md)
- Hypothesis ID and predecessor: U5 slim-pass gate; predecessor
  [R36](03-rust-port-u5-remeasure.md) (measured FAIL, 7,049 vs 2,270)
- Type: preregistered

## Decision

Nothing is measured yet. This entry freezes the slim-pass accounting, bounded
levers and outcome rule before any slim code exists, so the re-measurement
cannot be tuned toward a pass. Owner dispositions recorded 2026-11-08,
following the R36 outcome rule: (1) slim the Rust apparatus, then re-measure
U5 against the **unchanged** 2,270 ceiling; (2) resolve defect 7 by splitting
`receipt.rs` — no R30 amendment; (3) investigate the g1r
`FROZEN_RUST_CANONICAL = 1,333` provenance now (task 9.2, documentation-only,
no measurement effect: R36 already showed every reading of the constant
exceeds the ceiling). Per R36: pass unlocks Slice B adoption planning only;
fail returns to the owner with amend/close options; no target is weakened.

## Hypothesis and falsification

Working hypothesis (owner framing on the R36 outcome): the 4,779-line overage
is partly apparatus-expression overhead — `regenerator.rs` (2,798 canonical
lines) contains consolidation levers, and `surface.rs` (1,154),
`measures.rs` (1,428) and `recording.rs` (1,002) share derivable structure —
so a behavior-preserving slim pass may bring the counted total under the
unchanged 2,270 ceiling.

Falsifiable prediction: if the counted total — the R36 counted set (six Rust
modules + shim + enumerated hooks) **plus every module the split/slim adds**
(counted in full wherever it lives, `Req 6.1`) — still exceeds 2,270
canonical lines after the preregistered levers are exhausted, the
slim-expression hypothesis fails on the target substrate, the negative is
retained, and Slice B adoption requires a new dated owner decision (amend the
ceiling or close Slice B). Preregistered arithmetic (pre-slim baseline
receipt, task 9.1): the counted apparatus must fall by at least
7,049 − 2,270 = 4,779 canonical lines (−68%); both outcomes remain credible
and neither is assumed (R35: unknown is not pass). Any parity failure (frozen
digests/symbols/verdicts not reproduced), any Tier-1 control passing, or any
receipt failing admission also fails the gate regardless of size (`Req 6.5`).

## Frozen inputs and method

- Gate definition: `prototype-gate.md` R37 amendment, frozen in the same
  commit as this entry.
- Canonical method (unchanged from R27–R31/R34/R36): Rust files
  rustfmt-formatted (toolchain 1.98.1, `--edition 2021`), `splitlines()`
  counts; the embedded Python shim under `ruff==0.17.3 format --isolated
  --line-length 100 --target-version py313`; hooks by enumerated git-diff
  hunks (R31-frozen 16 + group-8 21 at baseline; slim-pass hook changes
  counted **net** — additions and removals — in the gate receipt); per-file
  counts and exact commands retained in the receipt; unavailable readings
  reported unavailable, never 0.
- Counted set (R36 set, extended): `native/src/{surface,measures,recording,
  regenerator,admission,receipt}.rs` + shim + hooks, plus every new module
  the defect-7 split or slim pass introduces (shared kernels included),
  counted in full wherever it lives. Pre-existing product modules (`scope`,
  `metrics`, `compare`, `model`, `budgets`) stay uncounted product; the
  Python stage-1 modules stay excluded as superseded (R36 boundary);
  experiment drivers and tests stay uncounted (separate strata). Relocating
  apparatus code into an uncounted stratum is not a lever.
- Defect-7 split rule: the retained `receipt.rs` must measure ≤ 150 canonical
  lines (R30 ceiling unchanged), and any extracted module is honestly
  registered in `native/complexity-budgets.json` with measured ceilings and
  counted in U5. The Python three-stratum control (`test_recording.py`) must
  pass 19/19 without amending R30.
- Slim levers (bounded, preregistered): (a) `regenerator.rs` consolidation;
  (b) shared derivable structure across `surface`/`measures`/`recording`
  extracted into counted shared kernels; (c) the defect-7 receipt split
  (base re-measured live). A lever that cannot preserve a frozen behavior is
  abandoned and recorded, not forced. Slim attempts stop when the levers are
  exhausted — no unbounded iteration.
- Behavioral boundary (a port stays a port; unchanged from R36): six Tier-1
  controls fail closed with zero runtime calls; the E4 three-turn replay at
  the recorded scores with the frozen E3/E4 LF-body digests (`92a48f77…`,
  `0b2e383d…`) and traces reproduced; `k=2` exhaustion retains best; every
  emitted receipt admission-verified via `bindings.json`; prompt byte-parity
  and `request_sha256` unchanged; report envelope field names and CLI exit
  codes stable. Live weaker-model recovery is not re-run by this gate (R36
  boundary unchanged).
- Outcome rule: report the measured total as pass/fail without weakening.
  PASS requires the counted total ≤ 2,270 with every behavioral freeze green
  and `lesscode-core budgets` exit 0. Fail returns to the owner (amend/close
  options); pass unlocks Slice B adoption planning only.

## Evidence and results

Not run. Pre-slim baseline receipt captured in task 9.1
([`rust-slim-base.json`](../../experiments/deconvolution/probabilistic-refine/results/rust-slim-base.json));
R36 measured baseline at `7657969`: total 7,049 = surface 1,154 + measures
1,428 + recording 1,002 + regenerator 2,798 + admission 274 + receipt 178 +
shim 178 + hooks 16 + 21 (gate record
[`rust-u5-gate.json`](../../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate.json)).

## Limits and interpretation

The −68% target is aggressive; a slim-pass failure would not falsify the
port's correctness (R36 items 1/2/3/5 PASS stands) — only the claim that the
same apparatus can be expressed within the frozen ceiling by bounded
consolidation. The rustfmt/ruff canonical-line rules are method choices
frozen since R27; they are not renegotiated here. The 1,333 provenance
investigation (task 9.2) cannot change any verdict, only the audit trail.

## Tool/spec consequence and next gate

Slice B production/adoption stays withheld (`Req 7.10`) until the R37 gate
measures ≤ 2,270. Next gate: R37 re-measurement (task 9.6) after the slim
pass and defect-7 split; on fail, owner disposition (amend the ceiling by
dated decision or close Slice B). Spec reciprocal links updated in the same
commit.

## COBESY check

Outcome rule visible first; a reader can replay every count from the receipt
commands; disproof explicit (any freeze break fails the gate regardless of
size); negative results safe to report; facts (R36 measurements) separated
from interpretation (slim levers); no inflated claims.
