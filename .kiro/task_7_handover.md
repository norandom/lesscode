# Task group 7 handover (cgc: U5 disposition + apparatus defect 6 fix)

Date: 2026-11-08. Owner-directed post-7.1 work; no tasks.md group consumed.

## What was done

1. **U5 disposition recorded** (commit `b46db97`, [R35](../docs/research/2026-11-08/02-u5-disposition.md)):
   owner accepted the Python stage-1 negative (3,783 vs 2,270 canonical lines;
   frozen ceiling unchanged, Req 7.8 negative retained). Slice B adoption
   (Req 7.10) now additionally requires the Rust-port U5 re-measurement gate
   to pass the frozen ceiling first. Records: requirements.md Amendment 3;
   prototype-gate.md U5 disposition amendment (newest-first); tasks.md
   Deferred section updated; R35 research entry + research README index.
2. **Concise-output directive dispatch fix** (commit `354a472`): the
   score-0.00 recovery directive from `next_rollout_params` was filtered out
   by `run_refine_loop` (`k in base_params`) and never sent to the model.
   Now dispatched in the retry user prompt (prompt-level; API adapters
   consume only `base_params` keys, so default and live weaker-model
   adapters are unaffected) and preregistered in `attempt_params`;
   per-attempt `preregistered_params` views frozen in receipts (previously
   aliased to the final adapted params of the rollout). Regression test added.

## Evidence

- `tests/test_regenerator.py`: 59 passed (58 prior + 1 new dispatch test).
- Full `pytest -n 4` (venv `.runs/e2-facdrone`, from `D:\Source\lesscode`):
  649 passed, 11 skipped, 473 subtests passed; 4 failed = documented
  pre-existing xdist execnet subTest flakes (`test_config` PreparationTests,
  `test_metrics` DuplicationMetricTests, `test_snapshot` DisposableTests x2),
  each re-confirmed passing serially in the same run cycle.
- No Rust/native code touched: cargo nextest and lesscode-core budgets not
  re-run (last verified 2026-11-08 in 7.1, commit `227e12b`).

## Current state

- All scheduled tasks.md groups 1-7 [x]. Deferred (owner-gated): TypeScript
  extraction evaluation (Req 7.1 partial); Slice B production adoption now
  double-gated: G1-R outcome recorded + Rust-port U5 re-measure gate.
- Owner WIP `src/lesscode_baseline/rules/evaluate.py` +
  `tests/test_rule_evaluation.py` remain unstaged by rule.
- HEAD = `354a472` (main).

## Next steps

1. Rust port planning: re-measure U5 under the same expanded-baseline
   canonical-line accounting on the Rust substrate; pass unlocks Slice B
   adoption planning, fail returns to the owner (slim/amend options, no
   silent weakening).
2. If Rust-port work is started, run it as a new task group from this
   handover.
3. No other known gaps logged: directive defect 6 is fixed.
