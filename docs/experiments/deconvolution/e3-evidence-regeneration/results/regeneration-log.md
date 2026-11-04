# E3 regeneration log

Regenerator: opus agent (step 1). Candidate root `.runs/e3-facdrone/cand`; only the five files
under `src/facdrone/shells/` and `candidate-structure.json` were edited.

## Round 1

- Started: 2026-11-04 (see wall times below), after the frozen-copy hash check (all five
  candidate files were byte-identical to `.runs/e2-facdrone/source` before editing).
- Changes (one rewrite of all five files):
  - `decision_evidence_store.py`: `_json` folded into `_dump(record, separators)`;
    `_check_record`, `_check_source_identity`, `_check_transform`, `_check_ic`,
    `_check_input` and `_validate_records` folded into one `_validate(value, adapter)` that
    also serves `decode_consumed_evidence` (stage and route checks apply only when the record
    has them). Check order and messages unchanged; SQL expressions unchanged.
  - `decision_evidence_publication.py`: `seal_evidence` and `seal_evidence_with_run_slices`
    share one private `_seal` (inlines `_captured_payload`, `_seal_captured`,
    `_run_slice_inputs`; same evaluation order: decode, convert, `as_of`, seal per route);
    `_valid_slice_scalar` inlined into `slice_bytes`; `deduplicated_inputs` rewritten as a
    pure first-occurrence map (no `set.add`, no id cache).
  - `decision_input_slice_store.py`: `_walk`, `_rebuild`, `_refusal` folded into `_resolve`
    (`count=True` serves `chain_length`); `_load` still looked up as a module global.
  - `decision_evidence_reader.py`: `_resolve_input` inlined into `_resolve_batch`;
    `read_decision_evidence` still a module global.
  - `decision_slice_publisher.py`: `_referenced_id` inlined into `referenced_slices`;
    `SlicePublication.retained` inlined into `publish_slices` (same SELECT text).
- Measures (`results/measures-round1.json`): lines 1,074, statements 546, functions 51,
  write sites 11 (all in shell functions), intra-set edges 60, module hops 7, pure share
  0.706 (36 of 51), structure check pass (0 failures, 0 undeclared).
- Pre-round sanity: all five modules import with `ROOT/src` first; every name in
  `public-surface.json` resolves (0 missing).
- Acceptance: `step3.py round1 2569.2 -- run_e3.py ... --round 1`, started
  2026-11-04T09:59:46+01:00, ended 10:12:43+01:00; `round1: exit 0 in 768.8s`, adaptive
  factor 1.79, no timeout.
  - Seed tests: root group 1403 passed, 2 skipped, 10 deselected (639.8 s);
    facdrone-research 39 passed, 3 skipped, 6 deselected; facdrone-reporting 92 passed,
    8 deselected, 1 xfailed.
  - Replay: `replay file fresh=True sequences=64 operations=311 level2_ops=0`.
  - First Level 1 divergence: none.
  - `results/sql-divergences-round1.json`: `divergences: []` (no Level 2 pairs), so
    `qdiff.py` was not run (step 3 of the protocol applies only to a non-empty file).
- Conclusion: round 1 passes acceptance (T1 within 1 of 4 rounds). Measures meet the T2
  and T3 targets. No further rounds were run. The candidate is left in place.
- Edit scope check after the round: a hash comparison of every `.py` under `src`, `tests`
  and `packages` of the candidate against `.runs/e2-facdrone/source` shows only the five
  regenerated files differ, and none were added.

## Not preserved / deliberate differences (not observed by the oracle)

- `deduplicated_inputs` no longer caches digests by `id()`. It hashes each slice item, so
  the output is the same and the cost is repeated hashing of identical objects.
- `SlicePublication.retained` (not in the public surface) is removed; its logic and SELECT
  are inlined in `publish_slices`.
- `SliceResolver.resolve` replaces its cache dicts on a namespace change instead of clearing
  them in place (private attributes).
- Docstrings on the private helpers that were folded away are gone. Short comments on the new
  private helpers (`_dump`, `_validate`, `_seal`, `_resolve`, `_resolve_batch`) replace them.
