# Task group 8 handover (cgc: Rust port + U5 re-measurement — measured FAIL)

Date: 2026-11-08. Group: `constraint-guided-compression` tasks 8.1–8.7, run
from the task 7 handover next step under the R36 preregistration.

## What was done

1. **Preregistration and base receipt (8.1, `0667548`)**: R36 research entry +
   `prototype-gate.md` R36 amendment frozen before any port code;
   `rust-port-base.json` receipt (live canonical `admission.rs` 274 +
   `receipt.rs` 152 = 426, plus the R31-frozen 16 hooks = 442 base; new-module
   budget 2,270 − 442 = 1,828; `receipt.rs` later grew to 178 within the
   group and is re-measured live at 178 in the gate); the g1r `FROZEN_RUST_CANONICAL = 1,333`
   constant recorded as an unreceipted-provenance apparatus defect (verdict
   unchanged under every reading).
2. **Port (8.2–8.5, `7bb9653`, `5479c8e`, `7f5625f`, `d5e848c`, `bc9f34c`)**:
   `surface.rs` (Contract A/B on ast-grep tree-sitter-python), `measures.rs`
   (Tier-1 checks + canonical measure via pinned ruff), `recording.rs` (L1/L2
   record/replay + embedded shim `native/shim/recording_shim.py`),
   `regenerator.rs` (two-tier refine controller, BestOfN, receipts + bindings,
   openai-compatible adapter on pinned `ureq =3.5.1`, `regenerate` CLI arm).
   Parity gates: frozen E3/E4 LF-body digests reproduced; E4 three-turn
   replay at recorded scores; prompt byte-parity + `request_sha256` pinned to
   the Python reference digest; six Tier-1 controls fail closed with zero
   runtime calls; receipts admission-verified. All modules honestly
   registered in `native/complexity-budgets.json` (ceilings = measured).
3. **Re-discharge + measurement (8.6, `7657969`)**: native k=2 exhaustion
   test added (`loop_exhaustion_k2_halts_and_retains_best` — the exact G1-R
   item-3 protocol; the prior test used k=1); spike driver
   `rust_u5_gate.py` (uncounted stratum) runs the per-item native tests
   through `cargo nextest` (pinned 1.98.1) and measures U5 under the frozen
   canonical method. **Gate record `rust-u5-gate.json` (+`.sha256`):
   items 1, 2, 3, 5 PASS through the Rust substrate; U5 = 7,049 canonical
   lines vs the 2,270 ceiling → FAIL by 4,779** (new modules 6,382 =
   surface 1,154 + measures 1,428 + recording 1,002 + regenerator 2,798;
   base re-measured 452 = admission 274 + receipt 178; shim 178; hooks 16 +
   21 with hunks enumerated). The smaller-replaceable-representation
   hypothesis is falsified on the target substrate (port ≈ 2.9× the Python
   stage-1 canonical 2,450). Negative retained, nothing weakened.
4. **Records (8.7)**: R36 amendment 2 (executed results), `prototype-gate.md`
   outcome section (newest-first), research README index rotation, this
   handover. tasks.md 8.1–8.7 ticked (8.2/8.5 ticks were added retroactively
   with 8.6 — their evidence was already in-repo from their own commits).

## Evidence

- `cargo nextest` (toolchain 1.98.1): **236/236** (235 at 8.5 + the new k=2
  test). `cargo test --doc`: ok. clippy: clean. `cargo fmt --check`: clean.
- `lesscode-core budgets --config native/complexity-budgets.json --root
  native/src`: **exit 0** (all modules within measured ceilings).
- Python suites (pytest-xdist `-n 4 --dist loadfile`, venv
  `.runs/e2-facdrone`, from `D:\Source\lesscode`): `test_regenerator.py`
  59/59; `test_recording.py` **18/19 — defect 7 unchanged** (three-stratum
  canonical check: `receipt.rs` 176 > R30 ceiling 150; owner decision, not
  amended or skipped).
- Gate record + sidecar sha256 `84b7ccc2…4610`; frozen E3/E4 digests
  re-verified inside the same runs; no frozen file or receipt changed.

## Open defects and environment notes

- **Defect 7 (open, owner decision)**: Python stratum control red since
  `5479c8e` — `receipt.rs` 176 > R30 ceiling 150 (combined 448 > 430);
  observation #7 recorded. Not amended, not skipped.
- **PATH corruption defect (observation #8)**: this opencode harness inherits
  a corrupted PATH (literal merged `C:\PATHEXT=.COM;.EXE;…` segment) that
  breaks cmd.exe child-process resolution. All validation runs must REPLACE
  (not prepend) PATH with `C:\Windows\System32;C:\Windows;C:\Users\mariu\.cargo\bin;C:\Program Files\Git\cmd`
  **+ the venv Scripts dir for Python runs** (and the uv winget dir when
  `measure_canonical` runs). One intermediate validation run omitted the
  venv Scripts dir and failed three acceptance arms with parent exit
  0xE0434357 after their inner suites passed — environment, not code; the
  exact protocol PATH passes. Same defect class, recorded here for the next
  group.
- **Side effect on owner WIP**: the E3/E4 acceptance-arm reruns rewrote the
  machine-written `written` timestamp line in the two
  `sql-divergences-round21.json` files (both were already owner-WIP-modified
  at group start; the only delta is the timestamp). Owner WIP
  `src/lesscode_baseline/rules/evaluate.py` + `tests/test_rule_evaluation.py`
  remain unstaged by rule.

## Current state

- HEAD = 8.7 records commit (main), after `7657969` (8.6), `bc9f34c` (8.5),
  `d5e848c` (8.4), `7f5625f` (8.3), `5479c8e` (8.3 partial), `7bb9653`
  (8.2), `0667548` (8.1). tasks.md group 8 fully [x].
- **U5 outcome: FAIL as measured on both substrates** (Python 3,783 R34;
  Rust 7,049 R36 vs the frozen 2,270 ceiling). Slice B production/adoption
  stays withheld (`Req 7.10`). Python modules remain the stage-1 reference
  implementation; the Rust modules are the target-substrate apparatus.

## Next steps (owner decisions)

1. **U5 disposition after the Rust negative** (R36 outcome rule): slim the
   apparatus (the 2,798-line `regenerator.rs` is the largest single lever;
   `surface`+`measures`+`recording` share derivable structure), or amend the
   ceiling by dated decision, or accept the negative and close Slice B. No
   post-hoc weakening was performed.
2. Defect 7 disposition (`receipt.rs` 176 > 150): split, amend R30, or accept.
3. Open provenance question from R36 amendment 1: the unexplained g1r
   `FROZEN_RUST_CANONICAL = 1,333` constant.
4. If a task group 9 is opened (slim pass or otherwise), start it from this
   handover; clear the conversation context first per the owner rule.
