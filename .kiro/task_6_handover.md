# Task Group 6 Handover — cgc 6.2 + 7.1 (2026-11-08)

Continue from this file after clearing context. mmhmem checkpoint: 2026-11-08 (post-7.1).

## What was done

| Task | Commit | Evidence |
|---|---|---|
| 6.2 Live weaker-model recovery (owner go) | `9d9cf72` | `docs/experiments/deconvolution/probabilistic-refine/results/g1r-live-record.json` + `results/live/{A-1..A-8, B-1..B-3}` — criterion run **B-2**: haiku attempt 1 Tier-1 `out_of_scope` 0.20 → same-turn acceptance R=1.0 (oracle 7/7) within frozen budgets; 11 runs retained incl. negatives; all receipts admission-verified |
| 7.1 Full verification + records | `227e12b` | pytest -n 4: 648 passed / 4 pre-existing xdist flakes (root cause surfaced: execnet `DumpError` serializing `WindowsPath`; proven by stash A/B in 6.2); cargo nextest 123/123; `lesscode-core budgets` exit 0 (receipt `results/budgets-selfcheck/`); admission sweep 11/11 (`admission-7_1.json` + sidecars); hash check: 0 tracked modifications, 53 sidecars verify, 14+4 pre-existing mismatches enumerated unrepaired; research entry **R34** `docs/research/2026-11-08/01-g1r-gate-and-live-recovery.md` + index + `prototype-gate.md` G1-R outcome amendment (phase approvals unchanged) |

Earlier group: 6.1 driver = commits `1878947`/`c998d25` (see task_5_handover.md).

## Apparatus defects found live and fixed (commit `9d9cf72`, dated comments in code)

1. anthropic SDK 2.6.0 `Messages.stream()` has no `temperature` and API rejects it beside `output_config` → temperature dispatchable only on openai-compatible endpoints
2. `claude-haiku-4-5` rejects `output_config.effort` → `request_kwargs(effort=None)` omits it; runner adapter maps effort→None
3. System prompt rule 4 taught wrong structure grammar → `<basename>::<qualname>` (matches frozen E4 Turn 3 style)
4. `_static_feedback` now forwards `why`/`write_sites`
5. `measures._check_structure` normalizes scope-path-qualified declared keys

Known gap logged (not fixed): `next_rollout_params` concise-output directive filtered out by `run_refine_loop`, never dispatched.

## Current state

- All scheduled tasks in `.kiro/specs/constraint-guided-compression/tasks.md` are `[x]` (groups 1–7). Deferred section remains: TypeScript extraction eval + Slice B production adoption — both owner-gated (Req 7.10, prototype-gate.md).
- U5 **fails as measured**: 3,783 vs 2,270 canonical lines (gate L29 unchanged; Req 7.8 negative retained).
- Framing per owner: Python = stage-1 prototype substrate; **Rust = the target substrate** (re-measure on port, never assume).
- Suites: pytest 648 pass + 4 xdist flakes + 11 skipped; the 6 documented process/timeout failures did NOT reproduce in either of today's full runs; native 123/123; budgets exit 0.

## Owner decisions pending

1. **U5 disposition**: accept as Python stage-1 prototype with a Rust re-measure gate / slim the apparatus / amend the ceiling. Never weaken silently (Req 7.8).
2. Slice B adoption tasks stay withheld until U5 resolved (Req 7.10).

## Environment reminders

- Run from `D:\Source\lesscode` (junction rule). venv `.runs\e2-facdrone` (pytest 9.0.3, xdist 3.8.0); `PYTHONPATH=src` for direct module runs; live runs additionally need `PYTHONPATH=.runs\regen-extras` (anthropic 2.6.0).
- Owner WIP `rules/evaluate.py` + `tests/test_rule_evaluation.py` modified — never stage.
- `lesscode-core.exe` exists debug-only under `native\target\debug\`; budgets root = `native\src`, config `native\complexity-budgets.json`.
- Untracked response bodies/goldens by convention; sidecars + result.json tracked.
- pwsh here-strings fail through the agent harness — use write-tool temp files for scripts/commit messages.

## Next steps

1. Owner: U5 disposition decision (above).
2. If accept: Rust-port re-measurement gate for U5 before any adoption task.
3. Optional small fix: dispatch or remove the concise-output directive in `next_rollout_params`.
