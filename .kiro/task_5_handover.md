# Task Group 5 Handover — cgc group 6 in progress (6.1 done, 6.2/7.1 pending)

Date: 2026-11-07 (night, second session). Scope: resumed from
`.kiro/task_4_handover.md` per the owner's clear-&-resume rule; executed task
6.1 (G1-R deterministic driver) in foreground manual mode. Owner framing
reaffirmed at session start: **the Python modules are the prototype stage;
Rust remains the target** (verification/admission substrate now, native loop
port behind its own later gate per the R33 design's language-extractor seam).

## What was done

- Reviewed the uncommitted 6.1 draft (`g1r_spike.py`) against the real
  `regenerator.py` / `surface.py` / `measures.py` API and fixed seven draft
  defects before the first run: undefined `_e4_scope_files_parsed`;
  `HERE.parents` off-by-one for both `LESSCODE` and `DC`; Contract-B
  `recorder_files` must resolve as `root/rel` (the driver now makes one
  byte-copy of the frozen tree and places `e4_oracle.py` at its repo-relative
  path inside it — frozen specimen never modified); `lesscode-core admission`
  takes `--root/--bindings/--out` flags, not positionals; admission now covers
  **every** emitted receipt (all six run dirs); controls budget corrected to
  `max_static_retries=3` per the frozen gate; frozen `response.json` is the
  full Anthropic envelope — the TurnResult JSON is extracted from its first
  `text` content block.
- Ran the driver end-to-end from `D:\Source\lesscode` (venv
  `.runs/e2-facdrone`). Result: **4/5 PASS, U5 FAIL as measured** (honest
  negative retained, Req 7.8): controls all fail closed with zero runtime
  calls (incl. `ctrl_feedback_leak`); E4 replay t1=0.001 s (stop_reason),
  t2=0.032 s (`labeled_reason` caught via Contract-B recorder tags),
  t3=0.06 s accepted with Tier-2 replayed 1.00 from the frozen R31 receipt;
  k=2 no-progress exhaustion retains best; admission verifies all six
  receipts via `bindings.json` (exit 0 each); **U5: 2,450 Python + 1,333
  frozen rust = 3,783 canonical vs ceiling 2,270** (baseline_expanded 2,838;
  `regenerator.py` 1,239 vs the prototype-era 262 budget line).
- Commit `1878947`: driver + `results/g1r-record.json` + tasks.md tick (6.1).
  Owner WIP (`rules/evaluate.py`, `tests/test_rule_evaluation.py`) untouched.

## Current state / open items

- **6.2 (live weaker-model recovery run)** — owner-go, NOT started (spends
  model budget; not run tonight). Run it via the driver's
  `run_refine_loop`/`_default_call_fn` path or a small live runner appending
  receipts; done-criterion: receipts show a recorded failure then acceptance
  R=1.0 within `max_turns<=4`, `max_static_retries<=3`, gate record cites it.
- **7.1 full verification** pending: `pytest -n 4` (expect 129/129 spec
  suites green; 6 pre-existing Windows process failures are NOT regressions),
  `cargo nextest`, `lesscode-core budgets`, frozen-artifact hash check, spec
  record updates + dated `docs/research/` entry (8.1–8.6) covering the U5
  outcome and the step-1-prototype/Rust-target framing.
- **U5 owner decision still open** (blocks Slice B): accept honest FAIL +
  Slice B locked / slim Python modules / dated ceiling amendment. Do not
  weaken the check silently.

## Environment reminders

Run everything from `D:\Source\lesscode` (junction rule). Direct module runs:
`.runs\e2-facdrone\venv\Scripts\python.exe` (spike self-inserts `src` into
`sys.path`). `lesscode-core.exe` exists in `native\target\debug` only.
Frozen E4 turn artifacts: response bodies untracked, sidecars tracked — do
not stage the untracked response bodies.

## Next steps

1. 6.2 live weaker-model run (owner go; spends model budget).
2. 7.1 verification + records (includes research-log link back to
   `g1r-record.json`, receipt `g1-r-spike-1`).
3. Clear context before the next task group; this handover is the resume
   point.
