# R38: U5 ceiling amendment — owner disposition on the R37 negative (option a)

- Report date/time and timezone: 2026-11-08, ≈16:00Z (in-session record; the
  commit timestamp is authoritative)
- Execution interval: owner instruction received and records written in one
  opencode session on 2026-11-08 (≈15:40Z–16:00Z); no code, build or
  measurement executed
- Status: completed
- Spec: [constraint-guided-compression](../../../../.kiro/specs/constraint-guided-compression/requirements.md)
  (`Req 7.8`, `7.9`, `7.10`, `7.11`, `8.1`–`8.4`); task 10.1;
  [prototype-gate.md](../../../../.kiro/specs/constraint-guided-compression/prototype-gate.md) R38 amendment
- Hypothesis ID and predecessor: [R37](04-rust-slim-pass-remeasure.md)
  (measured FAIL, 7,004 vs 2,270; levers exhausted); [R36](03-rust-port-u5-remeasure.md);
  [R35](02-u5-disposition.md)
- Type: preregistered option exercised by dated owner decision (no new
  experiment; R37 outcome rule option (a))

## Decision

The owner chose **option (a) of the preregistered R37 outcome rule — amend
the ceiling** (owner instruction, 2026-11-08 opencode session: *"proceed with
the amended ceiling"*). Effective 2026-11-08:

1. **The Rust-substrate U5 ceiling is amended from 2,270 to 7,004 canonical
   lines** — exactly the R37 measured total on the R37 counted set
   (`native/src/{regenerator,measures,surface,recording,jsonval,admission,receipt}.rs`
   + shim + enumerated hooks; new apparatus modules count in full wherever
   they live, `Req 6.1`). The amended ceiling is a **no-growth cap**: any
   future change to the counted apparatus must keep the counted total ≤ 7,004
   under the unchanged R37 canonical method; growth beyond it, or any
   baseline change, requires a further dated owner decision.
2. **The 20% whole-replacement reduction claim is retired as falsified.**
   This amendment is not a reduction claim and creates no retroactive pass:
   the R36 and R37 gate records remain FAIL under the frozen 2,270 ceiling
   they were gated under, and the negatives stay on record (`Req 7.8`):
   Python stage-1 3,783 (R34/R35); Rust 7,049 (R36); Rust 7,004 (R37).
3. **Slice B adoption planning is unlocked; production adoption execution is
   not.** Symmetric with the R37 outcome rule ("pass unlocks Slice B adoption
   planning only"), the amend disposition unlocks planning work only.
   Production adoption execution remains gated by the `Req 7.9`
   production-design gate and `Req 7.11` revalidation of affected claims
   (the whole-replacement reduction target is invalidated and must not
   reappear as an assumption), and stays unscheduled until the owner
   requests an adoption plan.

U7 (`testinfra.py` verbatim-relocation accounting) is unchanged. The Python
stage-1 modules remain excluded as superseded per the R36 accounting; the
operative substrate is Rust.

## Hypothesis and falsification

No new hypothesis. The lineage that produced this decision: the
smaller-replaceable-representation hypothesis was falsified on the Python
stage-1 substrate (R34: 3,783 vs 2,270), re-tested as a port on the target
substrate (R36: 7,049 vs 2,270, ≈2.9× the Python stage-1 canonical), and its
consolidation-rescue form was falsified through the preregistered slim levers
(R37: −45 of 7,049, −0.6%; the overage is port expression volume, not
consolidation overhead). The preregistered R37 outcome rule left exactly two
dispositions: amend by dated decision or close Slice B. The falsification
record is closed by this decision, not reopened by it.

What would reject this amendment's usefulness: future measured growth of the
counted Rust apparatus beyond 7,004 without a dated decision, or any Slice B
production claim that cites a whole-replacement reduction this apparatus does
not deliver.

## Frozen inputs and method

- Disposition input: the R37 gate record
  [`rust-u5-gate-r37.json`](../../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate-r37.json)
  (head `9194909`; counted total 7,004 = new modules 6,390 + base 398 + shim
  178 + hooks 38) and the R37 outcome rule as preregistered in
  [prototype-gate.md](../../../../.kiro/specs/constraint-guided-compression/prototype-gate.md).
- Owner instruction (2026-11-08, opencode session, quoted verbatim):
  "proceed with the amended ceiling" — exercising option (a).
- Amended cap definition: the R37 counted set and canonical method, unchanged
  (rustfmt 1.98.1 formatted copies + `splitlines()`; shim under pinned
  `ruff==0.17.3 format --isolated --line-length 100 --target-version py313`;
  hooks by enumerated git-diff hunks counted net). The cap value 7,004 is the
  measured R37 total under exactly that method and set — no rounding,
  headroom or re-derivation.
- No code, budget ledger, frozen file, digest or receipt was changed by this
  record (diff scope: this entry, the research index, `prototype-gate.md`,
  `requirements.md`, `design.md`, `tasks.md`).

## Evidence and results

Negatives retained on record (all under the frozen 2,270 = 80% of
$B_{\text{app}}^{\text{expanded}} = 2{,}838$):

| Gate | Substrate | Counted total | Ceiling | Verdict |
|---|---|---|---|---|
| R34 (record `rust-u5-gate` era) | Python stage-1 | 3,783 | 2,270 | FAIL, accepted (R35) |
| R36 (`rust-u5-gate.json`) | Rust target | 7,049 | 2,270 | FAIL as measured |
| R37 (`rust-u5-gate-r37.json`) | Rust target, slimmed | 7,004 | 2,270 | FAIL as measured |

R37 composition (the amended cap's reference decomposition): regenerator
2,716 + measures 1,422 + surface 1,151 + recording 971 + jsonval 130 +
admission 274 + receipt 124 = 6,788 `.rs` canonical + shim 178 + hooks 38
(R31-frozen 16 + 22 net registration lines vs `f2fe982`).

## Limits and interpretation

- The amended ceiling is **descriptive of the measured apparatus, not a
  target met**: it converts U5 from a reduction target into a no-growth cap.
  Any presentation that reports U5 as "passing" must cite this amendment and
  the retained negatives, not the frozen 2,270 checklist value.
- The cap is substrate- and set-specific: it binds the R37 counted set under
  the R37 method on the Rust target substrate. It does not license
  reclassifying apparatus code into uncounted strata (relocation is not a
  lever, R36/R37 rule), and Python stage-1 numbers remain historical.
- `Req 7.11` applies to downstream work: claims that assumed the 20%
  whole-replacement reduction are invalid and require revalidation before
  dependent production work continues.
- No behavioral claim changes: the G1-R items 1/2/3/5 PASS evidence (R36,
  R37) is untouched by this accounting amendment.

## Tool/spec consequence and next gate

- `requirements.md` Amendment 4, a dated amendment section in
  `prototype-gate.md` (above R37), an inline dated note on the frozen
  checklist U5&U7 item, a dated note on `design.md` G1-R item 5, the
  `tasks.md` group-10 block (task 10.1) and the Deferred Slice B condition
  update carry this decision with reciprocal links; phase approvals are
  unchanged (Req 8.3).
- Next gate: owner-chosen. The unlocked path is a **Slice B adoption
  planning** proposal (multi-rollout default adoption, optional
  `DspyRefineAdapter --backend dspy`, offline `MIPROv2` compilation) which
  must pass the `Req 7.9` production-design gate before any production
  execution; a stop remains available without further infrastructure
  (`Req 7.8`).

## COBESY check

Outcome visible first (amend, 7,004, no-growth cap, negatives retained);
expert reader can replay (gate record + R37 method + owner quote); disproof
explicit (three falsifications on record); negative results safe to report
(negatives retained, no weakening of the recorded past); facts and
interpretation separated (measured table vs cap semantics); no inflated
claims (explicitly not a reduction, not a retroactive pass).
