# R90: Complexity budgets count tokens; the change refused in R89 would now be within budget

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, afternoon, in one session after R89 (no clock times were recorded)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/requirements.md)
  (requirements approved 2026-11-11); the `budgets` command belongs to deconvolution-native-core
- Hypothesis ID and predecessor: follows from [R89](15-first-tracked-step.md) and
  [R87](13-density-by-syntax-specimens-restated.md)
- Type: change to a released check, with a migration of this repository's own ceilings

## Decision

The owner agreed that complexity budgets should count tokens instead of lines. `budgets` now does.

- Size of a file: tokens outside comments and outside `#[cfg(test)]` code. Size of a function: its tokens.
  Logical units are unchanged. Lines are still reported, for information.
- Defaults: 2,800 tokens per file and 560 per function, in place of 400 and 80 lines. That is seven tokens
  a line; this repository has 6.96 over all production code (115,199 tokens on 16,551 lines).
- An entry that still gives `max_physical_lines` or `max_fn_lines` is held to it, in lines. Budgets files
  written for release 1.1.1 or earlier therefore keep working with the same result.
- The receipt schema is `rust-syntax-units-v2`.

Separate outcomes: the check and the migration of one repository. No model was called.

## Hypothesis and falsification

Expectation: the change to `verdict()` that R89 refused for three more lines is within budget when size is
counted in tokens, because it removes one token. It would be refuted if the check still refused it.
Result: `recording.rs` with that change has 7,306 tokens against a ceiling of 7,307, 338 of 338 units and a
longest function of 1,010 of 1,010 tokens: within budget.

That does not accept the change. It was refused by the rule in force when it was made, and whether it goes
in now is the owner's decision.

## Method

- Tokens come from the driver's syntax tree with the shipped Rust pack (`measure::token_starts`), the same
  count as `deconvolute density`. Which tokens belong to test code and to which function is decided by the
  spans `syn` gives, as before.
- Migration of `native/complexity-budgets.json`, by script, from a receipt of the tree before the change:
  - a default ceiling becomes the default in tokens and is left out of the entry;
  - a ceiling of its own becomes today's tokens times ceiling over today's lines, rounded down, so each file
    keeps the headroom it had and no more;
  - a file or function that is over the new default is frozen at today's count.

## Results

| Item | Value |
| --- | --- |
| Files listed | 52 (51 before; `budgets_report.rs` is new) |
| Entries with a file ceiling of their own | 16 |
| Entries with a function ceiling of their own | 16 |
| Frozen because over the new default | `adapters/driver/operators.rs` (3,024 tokens on 348 lines); `run_merge` (655 tokens), `run_run` (719 tokens) |
| `recording.rs` | 7,307 of 7,307 tokens (was 969 of 969 lines) |
| Tests | 361 pass (359 before; two new for token ceilings); format and lints clean |

Two things became stricter, none looser:

- `operators.rs` had 52 lines of headroom and now has none.
- `run_merge` and `run_run` were under 80 lines and are over 560 tokens; they are frozen where they are.

## What went wrong on the way

- The first defaults were 3,200 and 640 (eight tokens a line). Measured, the code has seven. Eight would
  have loosened every default ceiling by about 15%. Corrected before anything was committed.
- Adding the token count put the checker itself over its own ceiling (411 lines against 330). The first
  migration run would have set its ceiling to its new size. Instead the receipt-writing functions moved to
  `budgets_report.rs` and `budgets.rs` is held to the 2,525 tokens it had before (now 2,429).

## Limits

- Tokens are those of the tree-sitter Rust grammar. The contents of a macro call count token by token; a
  string literal counts as a few tokens whatever its length.
- The defaults come from one repository's ratio of tokens to lines.
- `.kiro/complexity-budgets.json` and the design of deconvolution-native-core still describe the line
  metric; they are records of that time and were not rewritten.

## COBESY check

- Claim that can fail: stated above, and it held.
- What a reader may not conclude: that the refused change is accepted, or that token ceilings measure
  complexity better than lines in general. They only stop layout from deciding.
