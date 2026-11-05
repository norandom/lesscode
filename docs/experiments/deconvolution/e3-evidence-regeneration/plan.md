# E3 plan: regeneration of the evidence-publication core behind its public API

- Written: 2026-11-04 morning, Europe/Berlin, after R24 and the owner's
  "proceed" (option 1), before any E3 code, candidate or execution.
- Hypothesis: H8(c) in [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md):
  given a recorded, mutation-scored trace oracle as acceptance, an LLM can
  regenerate a scattered responsibility behind its public API with fewer
  edges and statements and unchanged observable behaviour. Predecessors:
  [R22](../../../research/2026-11-03/04-e1b-evidence-publication-oracle.md)
  (the oracle), [R24](../../../research/2026-11-04/01-e1c2-corrected-solver-rung.md)
  (reader-inclusive alphabet, verifier, declared residue),
  [R16](../../../research/2026-11-02/16-evidence-publication-trial.md) (the
  earlier local candidate on this scope: +3 statements, rejected).
- Type: preregistered. Predictions fixed below; amendments dated; failed
  predictions reported as failed.
- Runtime: native Windows, plain user account, no WSL, never a bare `bash`
  from scripts, no symlink privilege. One heavy job at a time. Deadlines
  adaptive via [adaptive.py](../adaptive.py) (reference recalibrated
  2026-11-04 09:15 on a quiet host: `ref_bench_s` 0.212; the earlier 0.311
  file is kept as `adaptive-reference-2026-11-04-0741.json`).

## Question

Can a regenerated implementation of the five evidence modules, written by an
LLM against the oracle as the only acceptance authority, keep every
observable behaviour the oracle and the seed tests record while reducing
statements and intra-scope call edges by a preregistered margin, and does the
oracle alone suffice, or does a verifier-gated solver differential on the
pure core find differences the oracle misses?

## Definitions

- **Public API** (one sentence): every name that code outside the
  regenerated files imports from them, with its signature and observable
  behaviour, including private names that tests import.
- **Pure core**: functions that take no engine, connection or session, open
  none, and execute no SQL; they compute payloads, digests, codecs,
  bindings and predicates from values.
- **Effect shell**: functions that hold a connection or engine and perform
  reads or writes; they call the pure core and contain no predicate logic
  beyond dispatch.
- **Level 1 observation**: return value or exception type and message,
  ordered table dumps after each operation, files written. **Level 2
  observation**: SQL statement text and parameters.

## Frozen inputs

- Specimen Facdrone at `d49f48bd…`, frozen copy `.runs/e2-facdrone/source`
  (hash-matched to the R15 manifest), venv `.runs/e2-facdrone/venv`
  (CPython 3.13.16), extras `.runs/e1b-facdrone/extras` (Hypothesis,
  cosmic-ray 8.7.0), `.runs/e1c-facdrone/extras` (CrossHair 0.0.112, z3).
- **Regenerated set** (five files, measured on the frozen copy with the E3
  measures script, baseline to be re-recorded by it before regeneration):

  | File | Lines | AST statements | Functions | Write sites |
  | --- | ---: | ---: | ---: | ---: |
  | `shells/decision_evidence_publication.py` | 442 | 207 | 22 | 1 |
  | `shells/decision_evidence_store.py` | 313 | 171 | 20 | 4 |
  | `shells/decision_evidence_reader.py` | 102 | 42 | 5 | 0 |
  | `shells/decision_input_slice_store.py` | 239 | 143 | 14 | 5 |
  | `shells/decision_slice_publisher.py` | 178 | 97 | 6 | 3 |
  | total | 1,274 | 660 | 67 | 13 |

  Intra-set call edges (distinct caller function to callee function pairs,
  static, by name, both inside the set): 84. Write sites are static matches of
  `execute|add|add_all|commit|flush|delete|merge(` and `insert|update|delete(`.
- **Unchanged but in the oracle scope**: `cppi/state_repository.py`,
  `wiring/run_evidence_store.py`, `facdrone_kernel/domain/decision_evidence.py`.
  They are callers of the set and stay byte-identical.
- **Public API to preserve** (names imported from outside the set; `prod`,
  `test` or both):
  - `decision_evidence_publication`: module import; `BoundEvidence`,
    `EvidenceBinding`, `RUN_SLICE_URI`, `bind_evidence`,
    `bind_evidence_with_run_slices`, `capture_payload`, `compact_capture`,
    `money_evidence_payload`, `payload_batches`, `seal_evidence`,
    `seal_evidence_with_run_slices`, `slice_bytes`, `stored_evidence_payload`;
    plus every name `state_repository.py`, `run_evidence_store.py` and the
    other set files import from it.
  - `decision_evidence_store`: module import; `DecisionEvidenceBatch`,
    `EvidenceConflictError`, `EvidenceReceipt`, `SealedEvidence`, `_read`,
    `append_decision_evidence`, `append_sealed_evidence`,
    `decode_consumed_evidence`, `decode_decision_evidence`,
    `encode_decision_evidence`, `read_decision_evidence`,
    `seal_decision_evidence`.
  - `decision_evidence_reader`: module import; `EvidenceResolutionError`,
    `SliceResolver`, `read_resolved_evidence`.
  - `decision_input_slice_store`: module import; `SliceConflictError`,
    `SliceReceipt`, `SliceResolutionError`, `SliceResolver`, `SliceRow`,
    `append_slice`, `chain_length`, `delta_row`, `full_row`, `resolve_slice`,
    `slice_identity`.
  - `decision_slice_publisher`: `SliceLedger`, `SlicePublication`,
    `publish_slices`, `require_referenced_slices`.
  The apparatus derives this list mechanically (`surface.py`) and the
  derived list is authoritative; this transcription is the preregistered
  expectation.
- **Acceptance corpus**: E1b golden trace `92a48f77…` (51 sequences, 290
  operations), `golden-e1c.json` `c69cb957…` (2), `golden-e1c2.json`
  `034f0f31…` (11): 64 sequences. R16 codec harness, 128 observations,
  baseline `r16-baseline.json`. Seed tests: the 191 E1b seed files with the
  236 baseline deselections (`selection-map.json`), measured Arm B time
  513.8 s idle (deadline base 2,569.2 s).
- **Declared residue** (unprotected by the acceptance): targets 166, 169,
  200 of R24 and every function the corpus does not reach; they are in
  `state_repository.py`, outside the regenerated set, and are listed so
  nobody reads E3 as protecting them.

## Method

### Step 0, apparatus (opus agent, before any candidate exists)

Research code in this directory, counted, frozen by hash before step 1:

1. `surface.py`: derives the public API from imports in `src`, `packages`
   and `tests` of the frozen copy; writes `results/public-surface.json`.
2. `measures.py`: lines, AST statements, functions, write sites, intra-set
   call edges, module-hop pairs, and the pure-core / effect-shell check
   (reads `candidate-structure.json`, verifies each declared pure function
   has no engine/connection/session parameter and no SQL execution call,
   each write site is inside a declared shell function). Writes
   `results/measures-<label>.json`. Baseline on the frozen copy first.
3. `run_e3.py`: Arm B against a candidate root with **all** 191 seed files
   and the E1b deselections (E1b's `run_arm.py` selects nothing for an
   unknown change, so E3 forces the full set), the R16 check, then replay
   of the three golden files with Level 1 and Level 2 separated: Level 1
   divergence fails; Level 2 divergences are collected to
   `results/sql-divergences-<round>.json` instead of failing.
4. `qdiff.py`: for each distinct Level 2 pair (recorded statement, replayed
   statement, parameters), builds at least 100 perturbed states from the
   recorded table dumps (row dropped, row duplicated with a changed key,
   string field flipped, version bumped; seed recorded), loads each into the
   embedded database, executes both statements and compares result sets. A
   pair passes if all perturbed states agree.
5. Verifier harness for T5 (`t5_drivers.py`): CrossHair `diffbehavior`
   between an original pure function and its candidate counterpart, with
   the four checks of the
   [solver-encoding verifier](../../../symbolic-execution-tooling.md):
   symbolic-ness smoke, soundness on recorded inputs (the candidate must
   agree with the original on every recorded call before solving),
   controls (a planted mutant in the candidate must yield a counterexample;
   an equivalent renaming must exhaust), realisability by concrete replay.
6. `receipt.py` as before; every heavy step through the adaptive wrapper.

### Step 1, regeneration (opus agent, the regenerator)

The regenerator receives: the five files, `public-surface.json`, the
definitions above, the measures targets, and the acceptance command. It
writes the candidate into `.runs/e3-facdrone/cand` (a copy of the frozen
copy; only the five files may differ) and a `candidate-structure.json`
declaring every function as pure core or effect shell. It may not read or
edit the apparatus, the tests, the golden files or any file outside the
five. A round is: write or fix the candidate, run `run_e3.py`, read the
first Level 1 divergence. At most **4 rounds**. Every round's divergence,
measures and wall time are recorded. The regenerator is told the measures
targets but not the T4 or T5 procedure.

### Step 2, independent review (sonnet agent)

Diff restricted to the five files; `public-surface.json` import check on
the candidate (every name importable, same kind: function, class, constant);
apparatus hashes unchanged since step 0; measures recomputed; no test file
touched; `candidate-structure.json` consistent with `measures.py`.

### Step 3, T4 and T5 (apparatus agent)

- T4: cosmic-ray sample of 30 mutants (seed 20261104) from the candidate's
  `decision_evidence_publication.py`; arm = R16 check + replay of the 64
  sequences + the seed files that import that module directly (from
  `public-surface.json`), per mutant, adaptive deadline. Plus M3 of E1b
  ported to the candidate's compression call (documented), M1 and M2
  unchanged (their file is unchanged).
- T5: at least three pure-core pairs chosen by the apparatus agent from the
  original's codec and payload functions that have a candidate counterpart
  with the same signature; `diffbehavior`, 300 s base per pair, verifier
  gates first. Every realizable counterexample is replayed through the
  oracle as a new sequence to see whether the oracle would have caught it.

## Predictions, fixed before execution

- T1 (behaviour): within at most 4 rounds the candidate reaches zero Level 1
  divergence on all 64 sequences, identical R16 output, every seed test
  passing with the baseline deselections, and every Level 2 pair passing
  `qdiff.py`.
- T2 (size): on the five files, AST statements at most 561 (15% below 660)
  and physical lines at most 1,083 (15% below 1,274); functions not more
  than 67.
- T3 (edges and structure): intra-set call edges at most 67 (20% below 84);
  write sites at most 13 and all inside declared shell functions; the
  declared pure core holds at least 50% of the candidate's functions and the
  static check passes for every one.
- T4 (sensitivity): kill rate on the 30-mutant sample at least 0.70; M1, M2
  and the ported M3 caught.
- T5 (oracle blindness): on the first candidate that passes T1, the solver
  differential finds at least one realizable counterexample that the oracle
  corpus did not catch. This prediction says the oracle alone is not
  sufficient acceptance; its failure would be good news for the oracle.

## Kill criteria

- T1 fails after 4 rounds: H8(c) is rejected on this scope. LLM regeneration
  did not reach behavioural equivalence under the oracle within budget.
- T1 holds but T2 and T3 both fail: H8(c) rejected as formulated; equivalence
  is reachable, compression is not.
- Any edit outside the five files, to a test, to a golden file or to the
  apparatus after step 0: the candidate is void, reported as void.
- A T5 verifier control fails: T5 is void, reported as void; T1 to T4 stand.
- Level 1 divergence on the frozen copy itself (apparatus defect): stop.

## Budget

Research code at most 400 new lines (surface, measures, run_e3, qdiff, T5
drivers, receipt), counted. Regeneration at most 4 rounds; each round's
acceptance base deadline 2,569 s (E1b's measured full Arm B), scaled. T4 at
most 30 mutants. T5 at most 3 pairs × 300 s base plus controls. All deadlines
scaled by the adaptive factor; a timeout under factor above 1.5 reruns once
when idle.

## Not claimed

No adoption. A candidate that passes T1 to T3 is a research result on one
scope, not a patch for Facdrone. No whole-scope equivalence: the protection
level is the acceptance corpus plus T4's measured sensitivity plus T5's
differential within bounds, with the declared residue. Nothing about
TypeScript, JavaScript or Rust.

## Amendment 1 (2026-11-04, step 0)

Written after the apparatus was built and validated on the frozen copy, before any candidate.
These are apparatus decisions the plan left open. Predictions T1 to T5 and the kill criteria are
unchanged. Details and the validation results are in [README.md](README.md).

1. **Deselections.** The 236 baseline deselections are E1b's failures over the whole gate tier.
   Only 24 of them name nodes in the 191 seed files, and `run_e3.py` passes those 24, as
   `run_arm.seeds` does for the full set.
2. **Replay fails closed.** `run_e3.py` exits 0 only if the replay wrote a fresh
   `results/sql-divergences-round<N>.json` with all 64 sequences. Round 0 is the frozen-copy
   validation.
3. **Level 2 pairs.** The recorded and replayed statement lists of an operation are aligned with
   difflib, and each differing pair runs with its own side's parameters. An unmatched statement
   pairs with `absent` and fails. Base states are the recorded tables before and after the
   operation. Normalised values are restored as follows: time ordinals become fixed synthetic
   times in ordinal order, byte digests become bytes, JSON text is parsed. The outcome is the
   sorted canonical rows, or for writes the rowcount plus all tables, or the error type. States
   the schema refuses are skipped and counted, with at least 100 loaded states required.
4. **qdiff strengthening.** Control (ii) passed under the plan's four perturbations, so they were
   too weak. Each perturbed state now starts with a boundary row: a new-key copy of a row of a
   table the statements name, with every column that a parameter binds set to the parameter or a
   neighbour. With it, control (ii) fails at state 20 and control (i) passes.
5. **Public surface.** `surface.py` also counts attribute reads on module aliases and monkeypatch
   targets, and tags `facdrone_testing` as test. Imports among the five files are listed
   separately and are not binding. The derived list adds 15 names to the transcription and drops
   none.
6. **Structure file.** The structure file is `candidate-structure.json`:
   `{"pure": ["<file>.py::<qualname>", ...], "shell": [...]}`. The SQL-call rule includes `.add`,
   which also flags `set.add` (in the original, `deduplicated_inputs`). The regenerator is told
   this rule as written.
7. **Satellite packages.** They resolve from the frozen copy through the venv's editable installs.
   Only `ROOT/src` comes from the candidate, as in E1b.

## Amendment 2 (2026-11-04, step 3)

Written during step 3, after the T4 results and before the T5 solver runs were read. It records
step-3 decisions the plan left open. Predictions T1 to T5 and the kill criteria are unchanged.
Results are in [README.md](README.md), section "Step 3".

1. **T4 arm.** The arm is `t4_arm.py arm`. It runs `run_e3.py`'s patched `run_arm.run("B")` on
   `.runs/e3-facdrone/mut`, a sha256-verified native copy of the candidate, with the seed
   selection replaced. The arm has three stages: the scope-import probe, the R16 check, then
   pytest over the seed files plus `test_e3_replay.py` (Level 1 only, with
   `E3_ROUND=t4`). Kill means any nonzero exit. An exit of 2 with `INVALID scope` means the
   mutant is incompetent and is excluded. The deadline base is max(120, 5 × 199.9 s) = 999.5 s.
   That is 5 × the unmutated arm time (`t4-base`), scaled by `step3.py`. One mutant runs at a
   time. The mutated file is restored by copying the candidate file back and verifying it by
   hash.
2. **T4 sample.** `cosmic-ray init` (8.7.0, through `uv tool run`, config `cosmic-ray-t4.toml`)
   enumerates the candidate's `decision_evidence_publication.py`: 317 specs. `t4_arm.py sample`
   draws 30 of them with `random.Random(20261104)` from the sorted keys, following E1b's sampling.
   It applies each mutant as its exact diff hunks.
3. **T4 seed files.** These are the 19 test files under `tests/` and `packages/*/tests/` that
   `public-surface.json` lists as importers of `decision_evidence_publication` or its names.
   Helpers in `facdrone_testing` are excluded. One of the 19, `test_weight_study.py`, is outside
   the 191 E1b seeds. Six baseline deselections name nodes in these files and are passed.
4. **M1 to M3.** E1b's `m1.patch` and `m2.patch` are applied verbatim with `git apply` in the
   mut copy. The M3 port is `t4-m3-port.patch`: candidate line 320 in `compact_capture` changes
   from `zlib.compress(..., 1)` to `9`. Line 179 (`capture_payload`, `level=1`) is not changed,
   as E1b's M3 changed only `compact_capture`. Each runs the full acceptance
   `run_e3.py <mut> --round t4-<m>` through `step3.py` with base 2,569.2 s.
5. **T4 reporting.** The kill rate follows the R22 convention:
   killed / (killed + survived), with incompetent mutants excluded. A reading that also excludes
   mutants judged equivalent by inspection is reported only as a labelled post hoc figure.
6. **T5 pairs.** The step-2 review hypotheses were taken up as follows:
   - `deduplicated_inputs`: rewritten. The driver passes a list, so review H1 (a one-shot
     iterator) is outside the domain by construction. The differential is blind to H1 by design,
     and H1 is recorded as the reviewer's probe fact.
   - `capture_payload`, with `capture.absences` observed after the call: review H4.
   - The folded sealing path, `seal_evidence` and `seal_evidence_with_run_slices` with a
     symbolic namespace switch: review H7. The symbolic `SignalVersion` list and premise also
     exercise the H8 signal-identity checks through `seal_decision_evidence`.
   - H3 (`_resolve_batch`) is not a pair: it is declared shell. H2 and H6 are not pure.
     `slice_bytes` and `parse_run_slice` were left out because the reviewer's probes already
     agreed and the plan allows at most 3 pairs.
   - The original is loaded from the frozen copy as `orig_pub`, with its own store as
     `orig_store`. The candidate is loaded from `cand/src`. Bytecode writing is disabled.
7. **T5 domains.** The bounds follow E1c-2. Strings come from `abc`, length at most 8 via zip.
   Lists hold at most 3 elements. Enumerations are integer indexes. Cells are str, int,
   `-n/4` (includes -0.0), nan or inf, bool, None, or bytes (bytes not in the seal domain).
   Items are a new slice, the previous slice object again (identity), or a scalar. There is no
   rejection guard and no slicing of symbolic strings.
8. **T5 verifier.**
   - (1) Smoke: `crosshair check` on a `smoke` function with entry markers, 15 s per
     condition. The seal pair takes about 15 s per path, so its gate was rerun at 60 s.
   - (2) Soundness corpus: 300 Hypothesis inputs per pair (`st.from_type` of the driver
     signatures, seed 20261104), written to `results/t5-soundness.json`. There are no recorded
     inputs for these drivers.
   - (3a) Planted mutant: candidate line 135 `!=` → `>` (`k_dedup` against `c_dedup`), over
     the full driver domain.
   - (3b) Equivalent variant: the candidate's `deduplicated_inputs` with its locals renamed
     (`e_tiny` against `c_tiny`). The domain is shrunk to three booleans choosing between two
     fixed slices, so that exhaustion is reachable.
   - (4) Realisability: `t5_drivers.py replay` parses each `Given:` and calls both drivers
     concretely.
9. **Launcher.** CrossHair runs with E1c-2's launcher (Amendment 4 there): `python -c "import
   t5_drivers; crosshair.main..."`. The reason is that CrossHair's audit wall rejected the
   `socket.gethostname` call made while sqlalchemy is imported (`t5-smoke-*`, exit 2).
10. **Gate iterations kept.** The first `t5-sound` failed on a driver defect: Hypothesis refuses
    `@given` on a function with default arguments. It was fixed by a factory function and rerun
    as `t5-sound-2`. The first `t5-ctl-*` runs never started CrossHair: in PowerShell, `$l`
    shadowed `$L`. They were rerun as `t5-ctl-*-2`. All failed entries stay in the receipt.
11. **Memory guard (coordinator, after a host-memory kill).** Before every heavy step,
    `adaptive.wait_for_idle(1800, cpu 0.35, ram floor 8 GB)` runs and is recorded as the note
    `<label>-guard`. The step runs if the host is idle, or else if free RAM is above 6 GB.
    Otherwise there is one more wait, then a stop. Mutant 16's killed partial run is kept as
    `logs/t4-m16-partial-killed.log` and was rerun from scratch as `t4-m16`.

## Amendment 3 (2026-11-05, after results)

A separate canonical-format reading of T2 was added after the results; no target or verdict changes.
`measures2.py` formats copies of both trees with `ruff format` 0.17.3 (`--isolated --line-length 100
--target-version py313`) and reports the candidate at 546 statements and 1,188 lines against the
original's 660 and 1,274 (both unchanged by the formatter): statements below the 561 bar, lines above
the 1,083 bar. Results: `results/measures-canonical-baseline.json`,
`results/measures-canonical-round1.json`; receipt labels `t11-*`.
