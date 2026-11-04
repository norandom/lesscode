# E3 step 2: independent review of the round-1 candidate

- Reviewer: sonnet agent, read-only, native Windows. Date: 2026-11-04.
- Candidate: `.runs/e3-facdrone/cand`. Frozen copy: `.runs/e2-facdrone/source`.
- Helper scripts are in the session scratchpad (`diffscope.py`, `app.py`, `introspect.py`, `cmp.py`,
  `cosmetic.py`, `structcheck.py`, `ps.py`, `probe.py`, `probecmp.py`, `binding.py`), not in the repo.
  The only repo file written is this report.
- Side effect to note: importing the original in a second process may refresh `.pyc` files under the
  frozen copy's `__pycache__`. No source file was touched, and `.pyc` is excluded from check 1.

## Verdict

**Within rules.** No kill criterion fired. Checks 1 to 5 pass. Check 6 found one confirmed
behavioural difference on an out-of-contract input, one removed public method of a public class that
nothing in the repository calls, and one private-function difference. None is exercised by the
recorded corpus or the seed tests. Two caveats on how the T2 margin was reached are under
"Measure caveats" and are for the T2 reading, not rule violations.

## Check 1: diff scope (PASS)

Command: `uv.exe run --no-project --python 3.13 python <scratchpad>\diffscope.py`. It walks both trees,
skips `__pycache__` and `*.pyc`, and compares sha256 in both directions.

```
2798 2798
CHANGED src/facdrone/shells/decision_evidence_publication.py
CHANGED src/facdrone/shells/decision_evidence_reader.py
CHANGED src/facdrone/shells/decision_evidence_store.py
CHANGED src/facdrone/shells/decision_input_slice_store.py
CHANGED src/facdrone/shells/decision_slice_publisher.py
```

2,798 files each, 0 added, 0 removed, exactly the five permitted files changed. No test, golden or
satellite-package file differs.

## Check 2: public surface (PASS with one non-surface difference)

Method: `introspect.py` ran in two separate processes (venv python). `PYTHONPATH` was the candidate
`src` for one and the frozen `src` for the other, and `__file__` confirmed the right tree was loaded.
It iterated every name listed in `public-surface.json`: 58 names plus 4 `<module>` entries in 5 modules
(62 derived names). For each name it recorded presence, kind, defining module, `inspect.signature`,
and for classes the `__init__` signature, bases, dataclass fields, public and private class
attributes, and method signatures. `cmp.py` compared the two JSON files.

Result: 58 of 58 names present in both, same kind, same signature, same fields. **One difference:**

- `decision_slice_publisher.SlicePublication` lost the method `retained(self, value: InputSlice) ->
  tuple[SliceRow, int] | None`. Public attributes are `['retained','row']` in the original and
  `['row']` in the candidate. The class is a surface name, the method is not. A repository-wide grep
  for `.retained` finds only the original's own call at `decision_slice_publisher.py:141`. The
  regenerator already reported this removal.

Binding at call time (`binding.py`, candidate), confirmed by `co_names`, `__defaults__`,
`__kwdefaults__` and `__closure__`:

- `decision_evidence_reader.read_resolved_evidence` has no defaults and no closure, and its code looks
  up the global `read_decision_evidence` (source line 76). The test's
  `monkeypatch.setattr(reader, "read_decision_evidence", ...)` therefore takes effect.
- `decision_input_slice_store._resolve` has `__defaults__ == (False,)`, which is only the `count` flag.
  It looks up the global `_load` (line 144). `monkeypatch.setattr(slices, "_load", ...)` takes effect.
- `decision_slice_publisher.publish_slices` looks up its own imported `_load` global (line 105). That
  binding is import-time in the original as well; behaviour is unchanged.

## Check 3: apparatus integrity (PASS)

`app.py` compared the files with `results/receipt.json` facts, entry `step0-freeze` (the only facts
entry).

| File | Match |
| --- | --- |
| measures.py, qdiff.py, receipt3.py, run_e3.py, step3.py, surface.py, test_e3_replay.py, plan.md | all 8 sha256 identical |
| README.md | identical to `readme_sha256` |
| golden-trace.json `25c86305…`, golden-e1c.json `3c6a81ea…`, golden-e1c2.json `d42cceef…` | all 3 identical |
| the 8 `results_sha256` files frozen at step 0 | all identical |

Mismatches: 0. No test file under `cand` differs (check 1).

## Check 4: measures (PASS)

Command: `uv.exe run --no-project --python 3.13 python measures.py D:\Source\lesscode\.runs\e3-facdrone\cand --structure candidate-structure.json --out <scratchpad>\measures-review.json`.
Exit 0. My output is **text-identical** to `results/measures-round1.json`:

| Measure | Baseline | Candidate | T2/T3 target |
| --- | ---: | ---: | --- |
| lines | 1,274 | 1,074 | at most 1,083 |
| stmts | 660 | 546 | at most 561 |
| funcs | 67 | 51 | at most 67 |
| writes | 13 | 11 | at most 13 |
| edges | 84 | 60 | at most 67 |
| module_hops | 7 | 7 | none |
| structure_pass | n/a | true | all pure checks pass |

Pure share is 36 of 51 functions = 0.706 (target at least 0.50).

### Measure caveats (facts, for the T2 reading)

- `ast.stmt` counts a multi-name `import a, b, c` or `x, y = 1, 2` as one statement.
  `cosmetic.py` counts the following, per file, as statements that the original style would have
  written separately: multi-name `import` (counted per name), tuple assignments with a tuple value
  (per element), and merged `from X import Y as Y` re-export lines. The candidate has 26 of them
  (publication 6, store 5, reader 2, slice store 7, publisher 6) and the original has 2. The
  separated-form count is 662 for the original and 572 for the candidate: **−13.6%, and 572 exceeds
  the T2 limit of 561**. The literal preregistered measure (546) passes.
- The physical-line margin is thin (1,074 against 1,083). Maximum line length rose from 100 to 128,
  with 10 lines over 100 and none before. Total characters fell 46,032 to 43,868 (−4.7%).
- The effect shell holds predicate logic that was in the pure core. `_resolve` (32 own statements,
  declared shell) now holds the rebuild and digest checks that were `_rebuild` and `_refusal`.
  `publish_slices` (21) holds the retained-representation logic. By function-body statements the
  pure share is 214 of 343 = 0.62. By function count it is 0.706.

## Check 5: candidate-structure.json (PASS)

`structcheck.py` parsed all five files: 51 defs (nested and methods included), 51 declared (36 pure,
15 shell), 0 duplicates, 0 undeclared, 0 declared-but-missing.

Spot checks by reading the code:

Pure (5 checked):
- `decision_evidence_publication.py::_seal`: no connection, only payload decoding and `seal_decision_evidence`. OK.
- `decision_evidence_store.py::_validate`: takes a batch or consumed projection, no connection. OK.
- `decision_evidence_store.py::_read`: takes a `RowMapping`, so no connection and no execution. OK.
- `decision_input_slice_store.py::_row_digest`: pure hash over a row. OK.
- `decision_slice_publisher.py::referenced_slices`: reads batches only. OK.

Shell (3 checked):
- `decision_input_slice_store.py::_resolve`: takes a `Connection` and calls `_load`. OK.
- `decision_slice_publisher.py::publish_slices`: executes SELECT, calls `append_slice`. OK.
- `decision_evidence_reader.py::_resolve_batch`: takes a `Connection`, calls the resolver. OK.

Consistent with the static rule in `measures.py` (`structure_pass: true`). No function declared
pure touches a connection, and every shell function holds one or calls one.

## Check 6: adversarial semantic read, ranked

I read the full original and candidate of all five files, then ran 76 probes on both trees in
separate processes (`probe.py`, `probecmp.py`): `deduplicated_inputs`, `slice_bytes` (16 shapes),
`resolved_inputs`, `parse_run_slice` (7), `money_evidence_payload`, `compact_capture`,
`payload_batches`, `stored_evidence_payload`, `seal_evidence*` (no capture, bad capture, missing
`as_of`, orphan consumed reference, no routes), `bind_evidence`, and `decode_*` with garbage payloads.
**74 identical, 2 differ (H1).** The probes include no sealed batch with real routes; the oracle
covers that.

Found by reading and confirmed by the probe unless marked.

**H1. `deduplicated_inputs` iterates its argument twice (confirmed by probe).**
- File and function: `decision_evidence_publication.py`, `deduplicated_inputs`.
- Difference: the original is single-pass. The candidate builds `digests` with a comprehension over
  `inputs` and then does `zip(inputs, digests, strict=True)`.
- Input: any one-shot iterable, for example `deduplicated_inputs(i for i in items)` or `iter(items)`.
  Original returns the deduplicated tuple. Candidate raises `ValueError: zip() argument 2 is longer
  than argument 1`.
- In contract? The annotation is `Sequence`, and the only production caller passes a list
  (`capture.inputs`). Out of contract, but a solver differential over `Sequence` will not find it
  either. Severity low. Corpus: not exercised.

**H2. `SlicePublication.retained` removed (confirmed by reading and by the surface check).**
- File: `decision_slice_publisher.py`, class `SlicePublication`, function `publish_slices`.
- Difference: `publish_slices` inlines the retained logic and reads `publication.connection`,
  `.scope`, `.namespace`, `.knowledge_at` and `.resolver` directly.
- Inputs that expose it: `publication.retained(value)` raises `AttributeError`. A publication test
  double or subclass overriding `retained` (for example one returning a fixed `(row, depth)` or
  `None`) is no longer consulted by `publish_slices`. The SELECT text and call order are unchanged.
- Severity low to medium, as an API change. The repository contains no other caller. Corpus: not
  exercised.

**H3. `_resolve_batch` lost its own guard for batches without run-slice references (confirmed by reading).**
- File: `decision_evidence_reader.py`, `_resolve_batch`.
- Difference: the original returned the batch unchanged if `_has_run_slices` was false. The guard now
  lives only in `read_resolved_evidence` (`... if _has_run_slices(batch) else batch`).
- Input, on a direct call: a `DecisionEvidenceBatch` with `ConsumedInput(value=SliceReference("consumed:"+d, d))`
  and no `run-slice:` item. Original returns it as is. Candidate runs `resolved_inputs` and raises
  `EvidenceResolutionError(... namespace=<in-batch> ... unresolved consumed reference)` if `d` has no
  captured slice in the batch, or rewrites the inputs if it has.
- Reachable through the public `read_resolved_evidence`? No. Private function. Severity low.

**H4. `capture_payload` argument evaluation order changed (plausible, unlikely to matter).**
- File: `decision_evidence_publication.py`, `capture_payload`.
- Difference: the original evaluated `capture.route(minimum_assets)` first, then `traces()`,
  `consumed`, `deduplicated_inputs`. The candidate evaluates `traces()`, `consumed` and
  `deduplicated_inputs(capture.inputs)` and only then `route(...)`. `route()` appends a `"scoring"`
  `AuditedAbsence` to `capture.absences` when `scored is None`, and it still runs before the
  `capture.absences` read (argument 7), so the payload content is unchanged.
- Input: a capture where `deduplicated_inputs` raises (an `InputSlice` containing `float("nan")`) and
  `scored is None`. The original appends the `"scoring"` absence to `capture.absences` before raising.
  The candidate raises first and does not append it. It matters only if the capture object is
  reused after the exception. Likewise, if `traces()` and `route()` both raise, the exception type
  that wins differs. Severity low.

**H5. `deduplicated_inputs` no longer caches digests by `id()` (confirmed by reading; cost only).**
- Difference: the original hashed each distinct slice object once. The candidate re-serialises and
  hashes every occurrence. The docstring of the original says the same slice object is shared by many
  factors, and `ConsumedCapture` uses a `slice_cache`, so the work grows with the number of factors
  per slice. Output is identical (probes `dedup_list`, `dedup_tuple`, `dedup_bool_vs_int`).
- Input: one large `InputSlice` object referenced by N items. Original N-fold cheaper. No result
  difference. Not observable by an oracle. Severity low (performance).

**H6. `SliceResolver.resolve` replaces dicts instead of clearing in place (plausible, unlikely).**
- File: `decision_input_slice_store.py`, `SliceResolver.resolve`.
- Difference: after a namespace change, `self._slices` and `self._lengths` are new dict objects.
- Input: code holding a reference to the old `resolver._slices` sees the original's dict emptied and
  the candidate's dict untouched. No such code exists in the repository (grep for `._slices` and
  `._lengths` finds only the class). Severity very low.

**H7. `_seal` builds each route identity directly (unlikely).**
- File: `decision_evidence_publication.py`, `_seal`.
- Difference: the original built a placeholder `DecisionEvidenceIdentity` with `step_id=""` and
  `captured.signals`, then `replace`d `step_id` and `versions.signals` per route. The candidate
  constructs the final identity once per route. The dataclasses in `facdrone_kernel` have no
  `__post_init__`, so nothing is validated by the placeholder. With zero routes the original still
  built the placeholder and the candidate builds nothing; neither can raise (probe `seal_noroutes`,
  `seal_rs_noroutes` equal). Also, the plain path now ends with `replace(sealed, schema_version=1)`,
  which equals the field default. Severity very low.

**H8. Store validators merged into `_validate` (no difference found).**
- Read order of checks, messages and branches for each record type: identical. `_validate` for a
  consumed projection skips stage and route checks through `getattr(..., "stages", ())` and
  `getattr(..., "route", None)`; same as the original's separate `_validate_records`. A
  `SliceReference` that passes its check now falls through to the next `elif` branches, which
  cannot match it. `decode_consumed_evidence` still validates twice (JSON, then canonical text).
  Garbage-payload probes (`decode_dec_*`, `decode_con_*`) give identical exception types and messages.
  Hypotheses still worth feeding into T5: signal-identity checks in `_validate` on a `SignalVersion`
  with `version=0` or a blank name; `CombineEstimate._aligned` with empty `confidences`; the
  `ConsumedIC` window check with a `None` `knowledge_at`. These are checked by reading only.

**H9. SQL, ordering, transactions (no difference found).**
- Statement text is unchanged in `append_sealed_evidence`, `read_decision_evidence`
  (`ORDER BY step_id`), `append_slice`, `_load`, the `base_id` SELECT in `publish_slices`, and
  `require_referenced_slices` (`.in_(missing)` on the same set-typed `missing`; `available |= set(...)`
  equals `.update`). Statement order inside `publish_slices` is unchanged. No begin, commit or flush
  was added or removed.

**H10. Docstrings (certain, harmless).** The docstrings of the folded private helpers, the long
`money_evidence_payload` sentence about NewTypes, and the extra `deduplicated_inputs` sentence are
gone.

### Notes for T5 differential (concrete targets)

1. `deduplicated_inputs` against `deduplicated_inputs`: generate `Sequence` inputs mixing equal-but-distinct
   slices, `bool` versus `int` cells, `-0.0`, NaN and unserialisable cells. Expect equality for all
   sequences. A one-shot iterator is the known counterexample (H1) but is outside the annotation.
2. `slice_bytes`, `parse_run_slice`, `resolved_inputs`, `compact_capture` (with a counting `retain`),
   `money_evidence_payload`: the 16 `slice_bytes` shapes agreed; the solver adds unbounded `float`,
   `int` and `str` cells.
3. `_validate` (candidate) against `seal_decision_evidence` plus the original `_validate` for
   `SignalVersion`, `ConsumedIC` and `CombineEstimate` records.
4. `SlicePublication` doubles and `_resolve_batch` direct calls (H2, H3) for the replay, not for the solver.

## Facts versus hypotheses

Facts: checks 1 to 5, the 74 of 76 probe agreements, the two H1 probe differences, the removed
`retained` method, the `_resolve_batch` guard move (by reading), and the measure caveats.
Hypotheses (not executed against a database or a full batch): H4, H6, H7, H8 sub-items, and the
performance claim in H5.
