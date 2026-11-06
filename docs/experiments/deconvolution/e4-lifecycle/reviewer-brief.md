# Reviewer brief for E4 task 6.3 (independent review of the accepted candidate)

Written 2026-11-06 with the plan (task 6.1), before any candidate exists. Requirements 7.1 to 7.5 of
`.kiro/specs/deconvolution-loop/requirements.md`. The reviewer is a fresh agent that did not write the
candidate and has not seen the regenerator's output beyond the files named here.

## What the reviewer receives (requirement 7.1: plan, diff, the two trees, nothing else)

1. `docs/experiments/deconvolution/e4-lifecycle/plan-for-reviewer.md`: the preregistration with its
   frozen hashes, in which the sensitivity sample's composition, the hand-written mutants and the
   differential pair list are replaced by "[redacted for the 6.3 reviewer]" (edited 2026-11-06 after
   review, before go: the reviewer does not receive `plan.md` itself, whose sections 6.4 and 6.5 disclose
   them; the redacted copy's hash is in receipt fact `task-6.1-apparatus-2`).
2. The candidate diff: `git diff --no-index` of the seven scope files between the frozen original
   `.runs/e2-facdrone/source` and the accepted candidate root `.runs/e4-lifecycle/cand-turn<k>` (the
   round controller names k in the receipt note `e4-accepted`).
3. Both tree roots, read only: the frozen original and the accepted candidate root.
4. The recorded measures of both trees (`results/measures4-baseline*.json`, `results/measures4-e4-t<k>*.json`,
   `results/pureshare-e4-t<k>.json`), the structure declaration the turn returned (`results/structure-e4-t<k>.json`),
   the edit-scope result (`results/editscope-e4-t<k>.json`), `results/public-surface.json`, `results/seeds.json`.
5. `results/receipt.json` (append-only; the reviewer appends through `receipt_atomic.py` only).
6. The apparatus it needs to rerun checks: `measures4.py`, `../t51-integration/editscope.py` and
   `pureshare.py`, `surface4.py`, `run_e4.py` (read, not run: no heavy step in the review).

## What the reviewer must not receive

- The regenerator's change log, its summarised thinking, its response text or the prompts
  (`../regenerator/results/turns/e4-turn*/`): no rationale from the writer of the candidate (7.1). The
  round controller hands over the `structure` object extracted from `result.json` and nothing else from
  that file.
- The sensitivity sample, the hand-written mutants' predictions and the differential pair list
  (`results/e4-orig-mutant-sample.json`, `mutants/`, `plan.md` sections 6.4 and 6.5): the review's
  hypotheses must not be steered by them; they are compared afterwards (7.5). For the same reason this
  brief names no candidate hypotheses: the reviewer forms its own from the diff (edited 2026-11-06 after
  review, before go).
- Any owner conversation or handover text about what the candidate should look like.

## Probe protocol

1. **Independent verification (7.2).** Recompute and compare with the receipt: the sha256 of every file
   outside the seven scope files between the two trees (`editscope.py`, whole-tree walk with `--dirs`, not
   only src/tests/packages); the seven scope files' hashes against the turn's receipt note; the apparatus
   hashes of plan.md's frozen-input table against the files on disk; the surface: every name in
   `public-surface.json` importable from the candidate with the same kind (function, class, constant,
   module, reexport) and the same signature (`inspect.signature`), and every `patched` name a module
   global looked up at call time (read the candidate's call sites); the measures (rerun `measures4.py`
   on both trees into a scratch directory and diff against the recorded files, byte for byte).
2. **Hypotheses (7.3).** Read the diff and state, before probing, numbered hypotheses H1, H2, ... of the
   form "on input class X the candidate behaves differently from the original", each naming the function
   pair and the input class. Error and malformed-input paths come first (the E4 prediction T6 expects at
   least one oracle-blind difference there); which paths those are is for the reviewer to find in the
   diff, and this brief deliberately lists none.
3. **Probes on both trees (7.3).** For each hypothesis, a concrete probe: import the pair from both trees
   (original from the frozen copy, candidate from the candidate root, with `PYTHONDONTWRITEBYTECODE=1`,
   no `.pyc` in either tree) and call both with the same inputs; or a deterministic walk through a store
   operation on an embedded database only if the probe needs none of the heavy fixtures. Record for every
   probe: inputs, both outputs (or exception type and message), agreement or disagreement. A probe that
   cannot be run is reported as not run with the reason, never dropped.
4. **Formatting effects (7.4).** Report the canonical and separated readings beside the as-written counts,
   the per-file maximum line length in both trees, the count of packed statements the separated rule
   expands (`measures4-*-separated.json`, `extra`), and any statement packing or line joining the
   formatter undoes.
5. **Patching checks.** For each name the tests patch (plan.md, structural contract): show the candidate
   line where the module's own function looks the name up, and say whether a `monkeypatch.setattr` on the
   module global would take effect. For the instance attributes (`_rules`, `_append`, `_evidence`,
   `_monitored`, `_engine`, `standing`): show they exist with the same role. Report the lazy imports the
   candidate kept or hoisted.
6. **Repository gates (information, not a verdict).** First-party fan-out per file against the
   `.sentrux` limit (16) and the import-linter contract in `pyproject.toml` (the store is private to the
   listed roots; `lifecycle_projection` must not import it); report violations.

## Output format

One file `docs/experiments/deconvolution/e4-lifecycle/review.md`, dated, with:

- a verification table (what was recomputed, recorded value, recomputed value, agreement);
- the hypothesis list H1..Hn, each with: pair, input class, probe, inputs, original output, candidate
  output, agreement (`agree` / `disagree` / `not run: reason`);
- the formatting-effects table;
- the patching table;
- a verdict line: `APPROVED` (every verification agrees, every probe recorded) or `NEEDS_FIXES`
  (something the round controller must do before 6.4 starts), with no claim about equivalence;
- a "carried forward" list: every hypothesis by name with the fault class it names, so that 6.4 addresses
  each named fault class in its survivor classification and 6.5 ties each differential pair to a
  hypothesis by name (7.5). A hypothesis whose probe disagreed is reported as an oracle-blind difference
  candidate for T6 and is replayed through the oracle in 6.5/6.7 if the alphabet can express it.

The reviewer writes nothing into either tree, runs no test suite, no recording, no mutant and no model
call, and appends its own commands to the receipt with `receipt_atomic.py ... run review-* --no-probe`.
