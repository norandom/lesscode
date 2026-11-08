# Task group 9 handover — Rust-apparatus slim pass, defect-7 split, 1,333 provenance (cgc)

Written 2026-11-08 after task 9.7, per the owner rule (write the handover
when a task group finishes, before asking the user to clear context).
Start task group 10 from this file.

## What was done

Owner dispositions on the R36 negative (2026-11-08): SLIM the apparatus,
SPLIT `receipt.rs`, INVESTIGATE the 1,333 provenance. All three executed;
the R37 slim-pass gate then re-measured U5 under the preregistered counted
set (R36 set + every new module the split/slim adds, hooks net).

- **9.1** (`b02fac5`): `prototype-gate.md` R37 amendment (bounded levers,
  stop conditions, R36 behavioral freezes carried over, ceiling unchanged
  2,270), R37 entry shell, index rotation, task group 9 block, group-8
  tick fix (8/8.1 were `[ ]`); pre-slim baseline receipt
  `rust-slim-base.json` — 7,049, exactly reproducing the R36 gate.
- **9.2** (`1630d48`, mmhmem obs #10): the g1r
  `FROZEN_RUST_CANONICAL = 1,333` has **no receipted derivation anywhere**
  — unreceipted aggregation error authored at `1878947`
  (`g1r_spike.py:53` + `g1r-record.json`); the cited R27–R31 records
  receipt 311/1,046/431/442, none decomposing to 1,333. Recorded as R37
  amendment 1; g1r artifacts left unmodified (frozen stratum); verdicts
  unaffected (every reading exceeds the ceiling). **Closed.**
- **9.3** (`7a4d0c0`): defect 7 closed — `J` value type extracted to
  `jsonval.rs` (130 canonical) with a `pub use` re-export keeping every
  caller path; `receipt.rs` 178 → **124 canonical** (≤ 150, combined 274 +
  124 = 413 ≤ 430); honestly registered in `complexity-budgets.json`
  (measured ceilings); `test_recording.py` 19/19 without touching R30.
- **9.4/9.5** (`f1571d6`, one kernel wave): shared kernels consolidated in
  `jsonval.rs` — `j_get` ×4, serde bridge ×3 (`from_serde`/`to_serde`),
  ensure_ascii-optional escape ×3; regenerator 2,798 → 2,716, recording
  1,002 → 971, surface 1,154 → 1,151, measures 1,428 → 1,422, receipt
  139 → 124 canonical. Abandoned-and-recorded levers: `indent1_dumps` ↔
  `receipt::write_value` merge (float-repr + ascii semantics differ;
  prompt byte-parity freeze), `truncate_chars` unification (chars vs byte
  index), surface `value_to_j` (int-only by design).
- **9.6** (`9194909` receipt re-stamp, `1779944` gate): R37 gate executed
  — items 1/2/3/5 **PASS** through the slimmed substrate; **U5 = 7,004 vs
  2,270 — FAIL by 4,734**. Gate record `rust-u5-gate-r37.json` (new file;
  the R36 record is preserved). Driver amended for the R37 counted set.
- **9.7** (this commit): R37 amendment 2, gate outcome section, index
  rotation, this handover; tasks 9.1–9.7 all `[x]`.

## Bottom line

The preregistered slim levers, honestly exhausted, moved U5 by **−45 of
7,049 (−0.6%)**. The bounded-consolidation hypothesis is **falsified on
the target substrate**: the 4,734-line overage is the expression volume
of the ported apparatus itself (regenerator 2,716; measures 1,422;
surface 1,151; recording 971; jsonval 130; base 398; shim 178; hooks 38),
not consolidation overhead. Behavioral freezes all held (E3/E4 frozen
digests replay green through the slimmed modules — byte-parity proven).

## Validation evidence (all green at `f1571d6`/`9194909`/`1779944`)

nextest 236/236; `cargo test --doc` ok; clippy clean; `cargo fmt --check`
clean; `lesscode-core budgets` exit 0 (ledger re-measured: jsonval 128,
receipt 122, recording 969, regenerator 2,714, measures 1,420, surface
1,149 physical); `test_recording.py` 19/19; `test_regenerator.py` 59/59.
No frozen file, digest or receipt changed.

## Open items (owner decisions — do not start without an explicit answer)

1. **U5 disposition after R37**: the negative stands measured. Options per
   the preregistered outcome rule: (a) amend the 2,270 ceiling by dated
   owner decision (keeping the Python 3,783 / Rust 7,004 negatives on
   record), or (b) close Slice B permanently (adoption stays withheld per
   Req 7.10). There is no third preregistered lever set.
2. Owner WIP still uncommitted on main by rule: the two round21 jsons
   (timestamp-only drift from acceptance reruns), `src/lesscode_baseline/
   rules/evaluate.py`, `tests/test_rule_evaluation.py`.
3. Environment defect #8 (session PATH corruption) still applies: see the
   protocol below. The three-stratum control additionally needs `uv.exe`
   on PATH (winget package dir).

## Environment / protocol notes (unchanged, plus one addition)

- Work from `D:\Source\lesscode` (the `C:\Users\mariu\Source` junction
  path fails the integrity layer on reparse traversal).
- Validation PATH replace: `C:\Windows\System32;C:\Windows;C:\Users\
  mariu\.cargo\bin;C:\Program Files\Git\cmd` + `.runs/e2-facdrone/venv/
  Scripts` for Python + the uv winget dir `C:\Users\mariu\AppData\Local\
  Microsoft\WinGet\Packages\astral-sh.uv_Microsoft.Winget.Source_8wekyb3d8bbwe`
  for `test_three_stratum_canonical_line_counts` (uv.exe missing →
  FileNotFoundError, not a code regression).
- Rust through `rustup run 1.98.1 ...`; budgets CLI requires `--out`.
- Python suites: `pytest -n 4 --dist loadfile` (xdist), PYTHONPATH=src for
  module runs. ~10 min for the two suites combined.
- 6 pre-existing Windows process/timeout test failures elsewhere in the
  suite are unrelated (known before group 9).

## Next steps

1. Ask the owner for the U5 disposition (amend vs close). Nothing in
   Slice B may proceed until then (Req 7.10).
2. Before starting task group 10, clear the conversation context (owner
   rule) and resume from this file + the mmhmem checkpoint.
