# R94: The first step in a Python project; an unpinned change that alters output is accepted, the same change is refused once pinned

- Report date/time and timezone: 2026-11-12, 08:45, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 08:19 to 08:38 local (times from file and commit timestamps)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/requirements.md); all
  tasks of that spec are ticked. This entry is open work from handover 43, taken up by owner decision of
  2026-11-12 ("use our own tool and record the results") before a new onboarding spec is started
- Hypothesis ID and predecessor: [R93](../2026-11-11/19-refused-pinned-accepted.md), which left open a step
  in a Python project, an override and a push through the installed hook
- Type: one real step in a foreign project (the owner's `x_likes_scraper`), with predictions stated before
  each check; exploratory for the hook and the override

## Decision

1. The step commands work on a Python project from plan to guard. Step `escape-html` replaces five chained
   `str.replace` calls in `HTMLFormatter._escape_html` by `html.escape`. It was accepted, recorded and
   passed by the guard: 0 noticed / 15 unnoticed before, 3 / 0 after; 7 lines removed, 2 added.
2. Without a pin, the check accepted a change that alters the output. `html.escape` writes `&#x27;` for an
   apostrophe, the old code wrote `&#39;`. No test of the project called `_escape_html` with an apostrophe,
   so the tests passed, no fault was planted in the one remaining call, and the verdict was "accepted".
   The plan had said so beforehand: "no fault noticed". The same change on top of two pinning tests was
   refused. An onboarding flow must not let a developer change a function that the plan marks "no fault
   noticed" without pins.
3. The push through the installed hook and an override are shown on a real repository. The hook passed the
   accepted branch, refused a commit made after the verdict in 0.6 seconds, and passed it after an
   override, which shows as "overridden". The override was made on a throwaway commit that was removed
   again; it shows the mechanism, not a real use.
4. Still not shown on a real step: a release entry on the ledger, a frame specification, an out-of-reach
   mark.

Separate outcomes: mechanism (plan, pin, check, record, guard, hook, override: shown on Python); behaviour
of the changed code (tests pass; old and new agree on 402,234 short inputs; not a proof); size (the file
shrinks); adoption (none: the branch is on a local clone). No model was called.

## Hypothesis and falsification

Stated before each check:

- Check A, naive change, no pin: accepted, although the output changes. Refuted by a refusal. It was
  accepted.
- Pin: the two tests are admitted against fault #1. Refuted by a rejection. Admitted; no unnoticed fault
  remains in the function.
- Check B, naive change on top of the pins: refused because the tests fail. Refuted by an acceptance. It
  was refused.
- Check C, `escape(text).replace("&#x27;", "&#39;")` on top of the pins: accepted. Refuted by a refusal.
  It was accepted.

## Frozen inputs and method

- Project: `x_likes_scraper` (MIT, the owner's), commit `3a214ff`, cloned to
  `D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper`; branch `step/escape-html`, head `1ddf17c`.
  The owner's repository was only read.
- Binary: `lesscode-core 1.2.0`, built with `--release --bins` from `e0e6b14` (code as in the prepared
  release) and copied before the runs.
- Declaration: [step.json](../../experiments/gradual-steps/steps/escape-html/step.json). Test command
  `uv run --no-sync python -m pytest tests/test_formatters.py -q -x`, setup `uv sync --quiet`,
  `max_token_growth` 0, focus `_escape_html`.
- Plan: [plan-config.json](../../experiments/gradual-steps/steps/escape-html/plan-config.json), four files
  of `x_likes_exporter`, four test files, 8 workers.
- A fresh work directory for every run. Each was deleted after its run.

## Evidence and results

All files are in `docs/experiments/gradual-steps/steps/escape-html/`.

| Run | Result | Time | File |
| --- | --- | --- | --- |
| `step plan`, 4 files, 27 functions | 249 faults noticed, 223 not; 5 functions with no fault noticed | 211 s | `plan.txt`, `plan.json` |
| Check A: naive change, no pin | accepted; `_escape_html` 0 / 15 before, 0 / 0 after | 139 s | `verdict-a-accepted-unpinned.json` |
| `step pin --fault 1` | admitted; none remains unnoticed | 65 s | `pin.txt`, `pins.patch` |
| Check on the pins alone (run by mistake, see below) | accepted | 67 s | `verdict-pins-only-accepted.json` |
| Check B: naive change, pinned | refused: "the project's tests do not pass on the step" | 97 s | `verdict-b-refused-pinned.json` |
| Check C: output kept | accepted; 0 / 15 before, 3 / 0 after | 206 s | `verdict.json`, `evidence.json`, `change.patch` |
| `step record`, `step guard` | recorded as accepted; guard exit 0 | under 1 s | `ledger.jsonl` |
| Push through the hook, accepted branch | passed | under 1 s | |
| Push of a commit made after the verdict | refused: "formatters.py changed after the verdict was written" | 0.6 s | |
| `step record --override`, push again | "overridden"; passed | under 1 s | `ledger-with-throwaway-override.jsonl` |

Own comparison, outside the tooling: old and new `_escape_html` on every string of up to five characters
from `& < > " ' # x 2 7 9 ; a 3` (402,234 inputs): none differs (`compare.py`).

Disk: the work directory held 3.8 GiB after the plan and one check, because every copy gets its own
virtual environment (the project depends on pandas and DSPy). One check alone left 0.8 GiB.

Mistakes and oddities of this run:

- The first attempt at check B picked the revert commit instead of the change (`git log --grep` matched
  both), so the check ran on the pins alone. It is in the table as what it was.
- On a refusal the hook prints "[lesscode-hook] Pre-push verification passed." and then the guard's
  refusal. The first line refers to the budgets only and reads as if the push were fine.
- `user.name` and `screen_name` go into the HTML export unescaped (`_format_tweet_html`, line 312). Not
  part of this step; reported to the owner.

## Limits and interpretation

- Check A is the tool working as documented ("only unnoticed faults refuse a step"; "an accepted step is
  not shown to behave the same"). For a user it is still the wrong answer to "did I break something".
  The plan carries the warning, the check does not repeat it.
- One function, one project. The times are for a test file that runs in a few seconds.
- The pins call a private method directly. That fixes the output of the method, not of the export.
- The override was demonstrated on a commit made for the purpose.

## Tool/spec consequence and next gate

- For the onboarding spec: a function with no noticed fault needs pins before a change, and the flow
  should say so at the check, not only in the plan. Candidates: `step check` warns or refuses when a
  changed function had no noticed fault on the base; or the flow runs `step plan` first and blocks.
  This is a decision for that spec, not made here.
- The guard is fast enough for a pre-push hook (it runs nothing). The check is not: 1.5 to 3.5 minutes
  here for one small function. That supports "guard in the hook and in the pipeline, check on request".
- Disk use per work directory has to be bounded or cleaned up by the tool for levels 1 to 3 (owner,
  2026-11-12).
- The hook message on a refusal should be corrected.
- Next: the release entry, a frame specification and an out-of-reach mark on a real step; `step guard`
  in this repository's pipeline; `tools/adopter_check.sh` after the tag.

## COBESY check

Outcome first; every run has its file and can be repeated from the declaration; each prediction named what
would refute it; the accepted unpinned change and the mistaken run are reported; the comparison on inputs
is labelled as outside the tooling; nothing is claimed about equivalence or adoption.
