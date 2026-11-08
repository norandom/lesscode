# Task group 10 handover — U5 ceiling amendment (R38, owner option a) (cgc)

Written 2026-11-08 after task 10.1, per the owner rule (write the handover
when a task group finishes, before asking the user to clear context).
Start task group 11 from this file.

## What was done

Owner disposition on the R37 negative (2026-11-08, instruction: "proceed
with the amended ceiling" = preregistered option (a)). Executed as task
10.1, commit `479b082` on main — **docs-only** (6 files, 224 insertions,
9 deletions; no code, frozen receipt, gate record or budget ledger
touched; owner WIP left uncommitted by rule).

- New R38 entry `docs/research/2026-11-08/05-u5-ceiling-amendment.md`
  (full template, COBESY check); research index rotated (`## Latest:` R38,
  R37 demoted to `## Prior:` with `*resolved 2026-11-08 by R38: amend*`).
- `prototype-gate.md`: dated `## U5 ceiling amendment (owner decision),
  2026-11-08 (R38)` section above the R37 outcome section + inline dated
  note on the frozen-checklist U5&U7 item (the `ctrl_feedback_leak`
  inline-amendment precedent).
- `requirements.md`: Amendment 4 after Amendment 3.
- `design.md`: dated note on the G1-R item-5 (U5&U7 accounting) paragraph.
- `tasks.md`: group-10 block (`10.1 [x]`), Deferred Slice B condition
  updated, group-9 parent tick fixed openly (9.1 precedent).

## Amended semantics (authoritative)

- Ceiling: Rust-substrate U5 **2,270 → 7,004** canonical lines (R37
  measured total; R37 counted set + canonical method unchanged:
  six .rs modules + `admission`/`receipt` base + jsonval + shim 178 +
  hooks 38 + every new module, Req 6.1).
- Semantics: **no-growth cap** — growth or a baseline/count change needs a
  further dated owner decision.
- The 20 % whole-replacement reduction claim (2,838 → 2,270) is **retired
  falsified**. Negatives retained: Python 3,783 (R34/R35), Rust 7,049
  (R36), Rust 7,004 (R37). R36/R37 gate records remain **FAIL** under the
  frozen 2,270 — no retroactive pass.
- Slice B adoption **planning** unlocked; production adoption **execution**
  gated by Req 7.9/7.11, unscheduled until the owner requests a plan.
- U7 unchanged. Phase approvals unchanged (Req 8.3).

## Fresh verification gathered this session (at HEAD `479b082`)

- `git log`/`git status`: HEAD `479b082`; owner WIP intact (two round21
  jsons, `src/lesscode_baseline/rules/evaluate.py`,
  `tests/test_rule_evaluation.py`); group-10 commit contains exactly the
  six expected files.
- Frozen gate receipts sha256-verified intact in the worktree:
  `rust-u5-gate.json` and `rust-u5-gate-r37.json` both match their
  `sha256sum`-format sidecars (first comparison mismatched only because
  the sidecar embeds `<hash>  <filename>`; corrected). No frozen record
  was altered by the amendment.

## Open items

1. **Owner doubt on implementation correctness (2026-11-08, verbatim:
   "I seriously doubt the implementation is correct")** — recorded per
   Req 8.4; NOT resolved. What is and is not verified:
   - Last full-suite green evidence is from group 9 (`f1571d6`/
     `9194909`/`1779944`): nextest 236/236 incl. E3/E4 frozen-digest
     replays, `test_recording.py` 19/19 + `test_regenerator.py` 59/59,
     budgets exit 0. Since then only docs changed (`825e526`, `479b082`),
     so the binaries under test are identical — but **no full suite has
     been re-run at `479b082` in this session** (owner redirected to
     "just produce a handover" mid-battery; a pytest background run may
     have been abandoned incomplete).
   - Fresh re-verification procedure, if wanted: clean-PATH full nextest
     (236 expected) + the two pytest suites; optionally re-run
     `rust_u5_gate.py`. **Caveat:** the driver unconditionally rewrites
     `rust-u5-gate-r37.json` + sidecar (new `written_utc`/`head`) —
     back up or `git restore` the receipt afterwards, then re-verify the
     sidecar hash.
   - Residual risk the tests cannot retire: behavior preservation beyond
     the frozen-digest replays (e.g. regenerator semantics on inputs
     outside the frozen corpora). If the doubt is about that, an
     adversarial-review pass (fresh eyes on `native/src/regenerator.rs`
     vs the Python reference) is the proportionate next gate, not more
     line-counting.
2. Slice B adoption-plan proposal is unlocked but **unscheduled** — do
   not start without an explicit owner request (Req 7.9/7.10/7.11).
3. Owner WIP still uncommitted on main by rule (unchanged from group 9).
4. Environment defect #8 (session PATH corruption) still applies — see
   the protocol below.

## Environment / protocol notes (unchanged from task_9_handover.md)

- Work from `D:\Source\lesscode` (the `C:\Users\mariu\Source` junction
  path fails the integrity layer on reparse traversal).
- Validation PATH replace: `C:\Windows\System32;C:\Windows;C:\Users\
  mariu\.cargo\bin;C:\Program Files\Git\cmd` + `.runs/e2-facdrone/venv/
  Scripts` for Python + the uv winget dir `C:\Users\mariu\AppData\Local\
  Microsoft\WinGet\Packages\astral-sh.uv_Microsoft.Winget.Source_8wekyb3d8bbwe`
  (uv.exe missing → FileNotFoundError, not a code regression).
- Rust through `rustup run 1.98.1 ...`; budgets CLI requires `--out`.
- Python suites: `pytest -n 4 --dist loadfile` (xdist), PYTHONPATH=src.
- 6 pre-existing Windows process/timeout test failures elsewhere in the
  suite are unrelated (known before group 9).

## Next steps

1. Ask the owner what evidence would resolve the correctness doubt
   (fresh full-suite re-run at `479b082`, and/or adversarial review of
   the ported modules) — or whether the doubt stands as recorded and
   work stops.
2. Nothing in Slice B may start without an explicit owner request
   (planning unlocked, execution gated).
3. Before starting the next task group, clear the conversation context
   (owner rule) and resume from this file + the mmhmem checkpoint.
