# E4 apparatus: the lifecycle monitor group (second scope), round 0 on an identical copy

Deconvolution-loop task 5.2. Scope and owner conditions: [scope-rationale.md](scope-rationale.md)
(candidate 1, approved 2026-11-06). This directory holds the apparatus for the second scope and the
round-0 validation on a byte-identical copy of the frozen original. No candidate exists; nothing was
regenerated; nothing was preregistered here (5.2 is apparatus, not an experiment).

**Status (2026-11-06): round 0 passes on the identical copy, native Windows, plain user account.** Every
command, probe, decision and hash is in `results/receipt.json` (labels below), logs in `results/logs/`.

## Result table (round 0, identical copy `.runs/e4-lifecycle/ident`)

| Step | Receipt label | Outcome |
| --- | --- | --- |
| Copy frozen original, hash the seven files in both trees | `copy-ident`, `hash7-before` | identical (1,495 lines; hashes in the receipt) |
| Public surface (deriver) | `surface4-frozen` | `results/public-surface.json`, 1,754 files scanned, 0 unresolved |
| Test-infrastructure detection | `testinfra-e4-scope` | `results/testinfra.{json,md}`: 15 direct test files, marker `not port_baseline and not live_broker`, 236 node failures resolved + 8 collection errors |
| Apparatus tests, red then green | `test-e4-oracle-red`, `test-e4-oracle-green` | ImportError before the oracle existed; 7 passed after |
| Recording 1 and 2 from the frozen original | `record-a`, `record-b` | 50 sequences, 317 operations; both sha256 `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584` (5,479,067 bytes), byte-identical, no masking beyond the declared normalisations |
| Replay on the frozen original (driver) | `replay-frozen` | Level 1: no divergence; Level 2: 0 differences |
| Acceptance arm, round 0 | `round0-ident` | **exit 0** in 75.7 s (deadline 1200 s base x factor 2.0 = 2400 s from an `adaptive.scaled` probe whose output was printed, not receipted; the guard's receipted probe a moment later read 1.47; see note `round0-ident-deadline-provenance`; not reached); root group 108 passed, 1 skipped, 1 deselected (47.8 s); research group 1 passed (26.7 s); replay file fresh, 50/50 sequences, 317 operations, `level2_ops` 0 |
| Seven files re-hashed after the run | `hash7-after` | frozen original and copy unchanged and identical |
| Host facts | `host-facts` | Windows 11 10.0.26200, CPython 3.13.16, user `mini\mariu`, no admin token, medium integrity, no symlink privilege, WSL unused, threads pinned to 1, serial |

The one skipped test is the fixture plugin's operator-tier rule (`facdrone_testing/fixtures.py`: a
`db`-marked test whose fixture closure lacks `pg_engine` skips under the embedded server, which holds the
schema only). The rule applies to original and candidate alike; the run used `-q`, so the node id was
not captured. The one deselection is the recorded baseline failure
`test_lifecycle_store_db.py::TestTheTableOnTheDatabase::test_the_event_id_is_the_stored_one_whatever_the_session_time_zone`
(cause `other` in E1b's census). Neither was repaired.

## Files

| File | Lines | Role |
| --- | ---: | --- |
| `surface4.py` | 60 | E3's `surface.py` with the seven files as the set; `results/public-surface.json` |
| `e4_oracle.py` | 534 | recorder and replayer for the lifecycle alphabet; imports `e1b_oracle`'s generic parts (`Norm`, `canon`, `first_diff`, `embedded_engine`) unchanged |
| `run_e4.py` | 77 | the acceptance arm on a root: import probe, seed groups with deselections, replay target, fresh-file check; `seeds` freezes `results/seeds.json` from `results/testinfra.json` |
| counted total | **671** | budget 600: **overrun of 71 lines, reported, not compacted** |
| `test_e4_oracle.py` | 69 | apparatus tests (no database): canonicalisation, orderings, clock injection; written first |
| `test_e4_replay.py` | 32 | pytest target: replays the golden on `pg_engine`, Level 1 fails, Level 2 collected into `results/sql-divergences-round<N>.json` |
| `results/golden-e4.json.sha256` | | sidecar of the golden, which stays on disk at `.runs/e4-lifecycle/golden-a.json` (5.48 MB > 1 MB) |
| `results/seeds.json` | | the frozen seed list, dropped files with reasons, deselections, marker expression, testinfra hash |
| `results/receipt.json` | | commands with argv, cwd, env, UTC times, exit, adaptive probe; facts (hashes, line counts, host); notes (guards, decisions) |

E1b, E2 and E3 modules (`run_arm`, `e1b_oracle`, `e2_winshim`, `adaptive`, `guard`, `receipt_atomic`)
are imported unchanged. The frozen original `.runs/e2-facdrone/source` was only read.

## The alphabet (scope-rationale section 4 minus `lifecycle_review`)

Every operation is a deterministic call with recorded inputs. Time comes from the driver's clock
(`tick` moves it; the store is built with `now=clock_now`; the CLI's own stores get it because the driver
patches `lifecycle_store.LifecycleStore` with a subclass whose default clock is the driver's). Ids are the
specimen's content digests, stable once the clock is.

- Store writers (`lifecycle_store`): `enroll`, `admit` (evidence `pass`, `renewed`, `incomplete`,
  `failing`), `flag` (plain, or bound into a caller transaction with `rollback` true or false),
  `retire`, `transition` (the raw request: a wrong authority or evidence kind). `expected` is a number or
  `"standing"` (read at call time).
- Store readers: `events`, `standing`, `census_references`, `admitted`, `open_flags`, `overdue_flags`,
  `readings`.
- `tick` (days from the fixed `AS_OF`, 2026-09-07 12:00 UTC).
- `run` (`lifecycle_monitor_runner.MonitorRunner.run` over scripted sources): targets (signal, version,
  digest, expected revision, anchor) or `"raise"` (the monitored source fails); evidence per window or
  per signal (`breach`, `within`, `thin`, `none` = not run, `raise`); flag writer `bound`, `never`,
  `broken`, `racing` (another writer lands the flag first, the runner's own clashes inside its
  transaction); `rules: old` (an earlier method revision, as the tests build it); `append: twice`
  (ON CONFLICT for every reading id). Level 1 observation: the `MonitorSummary` and every emitted record.
- Evidence (`lifecycle_evidence`): `seed_evidence` (Facdrone's `evidence_batch()` appended to
  `decision_evidence`), `live_evidence` over the seeded rows with a scripted readings source,
  `baseline_view` (scored, rule_book).
- Pure projection (`lifecycle_projection`): `standing_view` (`signal_standing` over standing, baseline,
  readings and flag variants, plus `monitor_label` and `labeled_reason`).
- Seams (`cppi/lifecycle_seams`): `ledger` (writes a verdict ledger under the sequence's temporary project
  root), `admit_durable` (recensus puts the ledger row to the bound admission writer), `seams`
  (`bind_lifecycle_monitor` over that root and a composed test book, then `_monitored()` and `run`; the
  real retained-evidence path, or `LiveEvidence.for_window` patched to a breach as the binding test does).
- CLI (`cli/lifecycle_group` through `CliRunner` on the core app): `cli_enroll`, `cli_retire`,
  `cli_readings` (exit code and output verbatim). `cli_readings` patches the same calculator inputs the
  runner test's `method_revisions` patches (`_lifecycle_store`, `read_parameters`, `_cited_baseline`,
  `_pod_of`, `configured_pods`, `lifecycle_evidence.baseline_view`, `lifecycle_evidence.live_evidence`).

Observations per operation: result or error (type, message, cause type); every SQL statement with
parameters (`before_cursor_execute`, listened on the `Engine` class so the CLI's own engines are seen);
an ordered dump of `signal_lifecycle_event`, `signal_monitor_reading` and `decision_evidence` with JSON
columns as stored text; the files under the sequence's project root.

Declared normalisations (every normalised field): datetimes become first-seen ordinals per sequence
(E1b's `Norm`); table dumps are sorted by canonical JSON; `events` by (moment, revision, event_id);
`admitted`, `open_flags`, `overdue_flags` and a seams run's monitored targets by (signal, version), because
the specimen leaves those orders to PostgreSQL (`select(...).distinct()` without `ORDER BY`,
`lifecycle_store.py:121,134`; ties on `known_at, revision` at `:80`); emitted records are grouped per
`signal@version` in emission order; a summary's `flags` and `unavailable` tuples are sorted. Nothing else
is masked: digests, ids, output text and SQL parameters are verbatim. The two recordings agreed without
any further intervention, so none of the rationale's determinism candidates had to be fixed after the fact;
the clock injection and the orderings were designed in before the first recording.

## Corpus identity

- `.runs/e4-lifecycle/golden-a.json` and `golden-b.json`, two figures each (review correction, 2026-11-06):
  the **identity** is the LF body `record()` hashes, sha256
  `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584`, 5,479,067 bytes; the files **as
  stored** are CRLF (`write_text` on Windows, the mechanism E1b's goldens share), 5,643,360 bytes, raw sha256
  `62791b3269b843586300f394ce53c88193269ac0d509f0eb0443a5140c805bd3`. Both files agree on both figures. The
  sidecars carry the raw hash on line 1 (it verifies against the file) and the body hash on a `#` line.
  Replay is unaffected (`read_text` normalises newlines). `record()` now writes with `newline="\n"`, so a
  future recording's raw bytes equal its hashed body; the round-0 recordings were not redone.
- 26 seed sequences (S01 to S26, transcribed from `test_lifecycle_store_db.py`,
  `test_lifecycle_monitor_runner_db.py`, `test_lifecycle_window_commits.py`,
  `test_lifecycle_monitor_binding_db.py`, `test_lifecycle_evidence.py`, `test_lifecycle_projection.py`) and
  24 generated sequences (Hypothesis 6.170.14, seed 20261106, 3 to 8 operations, optionally starting from
  the recorded end state of S01, S08 or S24). 317 operations. Generated-only outcome signatures:
  `enroll:TransitionRefused`, `retire:TransitionRefused`; `harness_suspects` empty.
- The scope files' hashes at recording time are in the golden's `meta.scope_sha256` and in the receipt.

## Seeds and deselections

`results/seeds.json` (from `results/testinfra.json`, sha256 `fcefcbf1…f62c`):

- 12 seed files: the 15 test files that reference the seven modules directly, minus
  `packages/facdrone-research/tests/test_lifecycle_full_loop_db.py` (gate `--ignore`; it also imports
  `fcntl` through the research app) and `test_controlled_study_producer.py`, `test_recensus_db.py`
  (collection errors on this host, `fcntl`). The rationale's 18 importing test-tree files are these 15
  plus the three helpers in `packages/facdrone-research/tests/` the detector classifies as support
  (`lifecycle_loop_daemon.py`, `lifecycle_loop_support.py`, `study_prior_fixture.py`); a fourth support
  file, `packages/facdrone-testing/facdrone_testing/core/lifecycle.py`, lies outside the test tree (the
  first version of this README named it in place of `study_prior_fixture.py`; corrected after review).
- The detector's four `via_fixture` candidates (`test_intraday_panel_cache.py`, `test_intraday_read_note.py`,
  `test_intraday_upsert_concurrency_db.py`, `test_release_receipt.py`) are false positives: they matched the
  fixture name `store` of `facdrone_testing/core/lifecycle.py` while defining or requesting their own.
  Excluded, recorded in the receipt note `red-phase`.
- Deselections: one node (above). Baseline list used: E1b's `baseline-failures-by-cause.json` (244
  entries = the 236 node ids of the E1b/E3 deselect list, set-equal per the task 2.2 validation, + 8
  `fcntl` collection-error modules; the spec prose's 236 counts only nodes).
- Marker expression: `not port_baseline and not live_broker` (detector and both gate scripts agree).

## Surface: deriver versus the rationale's section 3

The deriver is authoritative (requirement 1.5). Differences:

- Adds: the five `lifecycle_group` globals the tests patch as explicit `patched` names
  (`_cited_baseline`, `_lifecycle_store`, `_pod_of`, `configured_pods`, `read_parameters`); `attribute`
  kinds for `_consumed`, `live_evidence` and `study_baseline`.
- Drops: instance attributes read or patched on objects, not modules (`MonitorRunner._rules`, `_append`,
  `_evidence`, `_monitored`; `LifecycleStore._engine`, `standing`), and `LiveEvidence`, patched by the
  string target `facdrone.shells.lifecycle_evidence.LiveEvidence.for_window`, which the deriver resolves
  only to module.name. These are constraints on a candidate all the same (requirement 6.6) and are
  exercised by the seed tests and by the oracle (`rules: old`, `append: twice`, the seams `breach` patch).
  `labeled_reason` is listed under `deriver_drops` in the receipt fact for completeness only: the
  rationale's section 3 table does not list it either (its section 4 names it as an alphabet operation).
- `_rules_revision`, `MonitorRunner` and the other names the CLI, root and seams import from one another
  are intra-set (requirement 1.4), not surface. `labeled_reason` is referenced by no outside file.

## Residue and what is not claimed

- Residue (owner condition): `lifecycle_group.lifecycle_admit`, `lifecycle_group.lifecycle_review`,
  `lifecycle_root.pod_lifecycle_standing`, `lifecycle_root.lifecycle_store` are reached by no seed test on
  this host and by no oracle operation. `lifecycle_review` reads the wall clock and is excluded from the
  alphabet.
- The behaviour behind the deselected node (the event id under a non-UTC session time zone) and behind the
  skipped operator-tier test is unprotected on this host.
- The seams operation's retained-evidence join is exercised on an empty and on one seeded
  `decision_evidence` state only; a breaching window through the real join is not in the corpus (the
  binding test patches it too).
- Round 0 shows that the apparatus reproduces the original on an identical copy. It says nothing about
  any candidate, about sensitivity, or about equivalence.
- Threaded races (`test_two_writers_at_one_revision_cannot_both_apply`) are not in the alphabet; the
  `racing` flag writer reproduces the clash deterministically on one thread.

## Commands as executed (Git Bash; full argv in the receipt)

```bash
PY=D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe
D=D:/Source/lesscode/docs/experiments/deconvolution; E4=$D/e4-lifecycle; R=$E4/results/receipt.json
PP="D:/Source/lesscode/.runs/e2-facdrone/source/src;$D/e1b-facdrone-evidence-publication;D:/Source/lesscode/.runs/e1b-facdrone/extras;$D/e2-facdrone-edge-census"
EV="--env PYTHONPATH=$PP --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env PYTHONWARNINGS=ignore"
$PY $D/receipt_atomic.py $R run copy-ident -- $PY -c "import shutil; shutil.copytree(<frozen>, <ident>, symlinks=False, ignore=shutil.ignore_patterns('.git', '__pycache__', '.pytest_cache'))"
$PY $D/receipt_atomic.py $R run surface4-frozen --cwd $E4 -- $PY surface4.py <frozen> --out $E4/results/public-surface.json
$PY $D/receipt_atomic.py $R run testinfra-e4-scope -- $PY $D/testinfra/testinfra.py <frozen> --scope <7 files> --baseline-failures <e1b>/results/baseline-failures-by-cause.json --site-packages <venv>/Lib/site-packages --out $E4/results --stem testinfra
$PY $D/guard.py record-a --receipt $R && $PY $D/receipt_atomic.py $R run record-a --cwd $E4 $EV --timeout $(scaled 600) -- $PY e4_oracle.py record --out D:/Source/lesscode/.runs/e4-lifecycle/golden-a.json   # then record-b
$PY $D/guard.py replay-frozen --receipt $R && $PY $D/receipt_atomic.py $R run replay-frozen --cwd $E4 $EV --timeout $(scaled 600) -- $PY e4_oracle.py replay --golden D:/Source/lesscode/.runs/e4-lifecycle/golden-a.json
$PY $D/receipt_atomic.py $R run seeds-freeze --cwd $E4 -- $PY run_e4.py seeds
$PY $D/guard.py round0-ident --receipt $R && $PY $D/receipt_atomic.py $R run round0-ident --cwd $E4 --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env PYTHONWARNINGS=ignore --timeout 2520 -- $PY run_e4.py D:/Source/lesscode/.runs/e4-lifecycle/ident --round 0 --deadline 2400.0
```

`run_e4.py` exits 0 only if the import probe resolves `facdrone.shells.lifecycle_store` under the root, every
pytest group passes (serial, `-x`, `-p e2_winshim`, `--timeout=600`, the marker expression, `ROOT/src` first
on `PYTHONPATH`, the deselections applied), and the replay wrote this run's
`results/sql-divergences-round<N>.json` after the arm started, covering every golden sequence. No R16
harness: it belongs to the E3 scope. In the command lines above, `--timeout` is `receipt_atomic.py`'s own
subprocess timeout (deadline + 120 s), not part of `run_e4.py`'s argv; `run_e4.py` enforces `--deadline`.

## Deviations from the task brief

- Line budget: 671 counted lines against 600 (71 over). The overrun is in `e4_oracle.py`'s 26 transcribed
  seed sequences and the CLI and seams operations; reported, not compacted (requirement 11.6).
- The baseline-failure list used is the 244-entry by-cause file (the only one testinfra reads); its 236
  node ids are the E1b/E3 list. Two importing modules with collection errors are dropped at file level
  rather than deselected by node, because a module that does not collect has no node.
- `wiring/lifecycle_root.lifecycle_store(engine)` could be reached trivially by the driver but is kept as
  residue, as the owner's condition names it.

## 6.2 Regeneration and acceptance (2026-11-06)

Deconvolution-loop task 6.2 under the frozen [plan.md](plan.md) (sha256 `baf5ea1c…7520c`, fact
`plan-frozen`; owner go 2026-11-06). Regenerator `regenerator/regen_turn.py` (`038cbad2…727ff`) at the fixed
parameters `--max-tokens 128000 --effort medium --thinking-display summarized`, model `claude-opus-5-5` sent and
returned, one fresh root per turn (`.runs/e4-lifecycle/cand-turn<k>`, copied and hash-verified by the program).
Every command, probe, hash and verdict is in `results/receipt.json` (labels below); the turn texts under
`regenerator/results/turns/e4-turn<k>/` (sidecars committed, `user.txt` and `response.json` not).

**Result: accepted at turn 3 (T1 held, ≤ 3 turns).** The accepted candidate is `.runs/e4-lifecycle/cand-turn3`
(seven-file hashes in note `e4-turn3-candidate`, re-hashed after the arm in fact `e4-t3-hash7-after-cand`; the
frozen original unchanged, fact `e4-t3-hash7-after-frozen`).

### Per turn

| Turn | Receipt labels | stop_reason | usage in / out (thinking) | elapsed s | files | lines as written (canonical) | editscope | structure (pure share) | patchcheck | arm exit (s) | first divergence |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- | --- | --- |
| 1 | `e4-turn1`, `editscope-e4-t1` | `max_tokens` | 36,347 / 128,000 (128,000) | 1,116.9 | 0 | not read | no file differs (information) | not read | not run | not run | none: no candidate (failed turn, counts) |
| 2 | `e4-turn2`, `editscope-e4-t2`, `structure-e4-t2`, `measures4-e4-t2`, `pureshare-e4-t2`, `patchcheck-e4-t2`, `pycompile-e4-t2`, `e4-t2-arm` | `end_turn` | 36,347 / 95,292 (64,819) | 781.5 | 7 | 1,447 (1,449) | pass | pass (0.50) | pass | 1 (19.2) | `test_e4_replay.py` collection error: `ImportError: cannot import name 'labeled_reason' from 'facdrone.shells.lifecycle_projection'`; 0 seed tests ran, replay not run |
| 3 | `e4-turn3`, `editscope-e4-t3`, `structure-e4-t3`, `measures4-e4-t3`, `pureshare-e4-t3`, `patchcheck-e4-t3`, `pycompile-e4-t3`, `e4-t3-arm` | `end_turn` | 36,805 / 102,681 (73,922) | 836.0 | 7 | 1,397 (1,405) | pass | pass (0.519) | pass | **0** (49.6) | none: 108 passed, 1 skipped, 1 deselected; research 1 passed; replay fresh 50/50, 317 operations, `level2_ops` 0 |

Turn 1 spent the whole output budget thinking and returned no text block (plan 6.2 step 1: a `stop_reason`
other than `end_turn` is a failed turn that counts; step 4: no feedback section, so `request-e4-turn2.json` is a
byte-identical copy of turn 1's, fact `request-e4-turn2`). Turn 2's request was therefore turn 1's; turn 3's
(`request-e4-turn3.json`, `7b5394c9…93804`) is turn 1's plus `feedback = {"kind": "failed", "first_divergence":
<the node above as one text>, "measure_gaps": {lines_canonical 1449 vs ≤ 1345: +104; functions 52 vs ≤ 55: −3;
intra_set_call_edges 61 vs ≤ 66: −5; write_sites 12 vs ≤ 12: 0; pure_core_share 0.5 vs ≥ 0.5: 0; level1 not
reached}}` and nothing else (requirement 6.7). Arm deadlines came from receipted `adaptive.scaled(1200)` probes
(notes `e4-t2-arm-deadline` factor 1.37 → 1,644 s; `e4-t3-arm-deadline` factor 1.26 → 1,512 s; guards
`e4-t<k>-arm-guard`), neither reached.

### Readings on the accepted candidate (T2 to T4 and S are information read on the accepted turn, not acceptance conditions)

| Id | Bar | Candidate (turn 3) | Original | Reading |
| --- | --- | ---: | ---: | --- |
| T1 | acceptance within ≤ 3 turns | turn 3 | — | **held** |
| T2 | canonical lines ≤ 1,345 | 1,405 | 1,495 | **failed** (6.0 % fewer; ≥ 1,346) |
| T3 | functions ≤ 55 (falsifier ≥ 59) | 52 | 69 | **held** (in the band 49 to 55) |
| T4 | intra-scope call edges ≤ 66 (falsifier ≥ 72) | 63 | 89 | **held** (in the band 58 to 66) |
| S | write sites ≤ 12 in declared shells; pure share by functions ≥ 0.50; every function declared and every pure check passing; patched names and instance attributes | 11 sites, all in shells; 0.519; 52 of 52 declared, 0 failures; patchcheck pass | 12; — | **held** |
| beside S | pure share by statements inside defs (no bar) | 0.40 | — | reported |
| statements | as written / canonical / separated (no target) | 720 / 720 / 803 | 759 / 759 / 844 | reported |

Module hops 10 (original 12). Level 2: no pair occurred on any turn that reached the replay, so the query
differential was not run. Per-file lines of the accepted candidate (as written): runner 356, store 273, group
271, evidence 206, projection 139, seams 95, root 57.

### Patching sentinel check and apparatus added in 6.2

| File | Lines | Receipt | Role |
| --- | ---: | --- | --- |
| `patchcheck.py` | 60 | facts `patchcheck-apparatus` (first version, 89 lines), `patchcheck-apparatus-2` (final); runs `patchcheck-frozen` (exit 1: my control misread typer's `SystemExit` as an exception; every tier had passed), `patchcheck-frozen-2` (exit 0), `patchcheck-e4-t2`, `patchcheck-e4-t3` (exit 0) | plan 6.2 step 2e: tier 1 (12 names bound, no local rebinding; 7 instance attributes) and tier 2 (11 sentinel lookups through store-free triggers) on each tree; `results/patchcheck-*.json` |
| `pureshare4.py` | 15 | fact `pureshare4-apparatus`; runs `pureshare-e4-t2`, `-t3` | the plan names `pureshare.py`, whose file list is E3's `surface.SET`; the shim rebinds it to `surface4.SET` as `measures4.py` does |
| counted in 6.2 | **75** | | running total 97 (6.1) + 75 = 172 of 900 |

### Findings and deviations

- `labeled_reason` of `lifecycle_projection` is not a name of `results/public-surface.json` (the deriver scanned
  the specimen tree, where only the module itself calls it), but `e4_oracle.py` imports it at module level for
  `op_standing_view`. Turn 2 kept every surface name and was still rejected at collection; the arm is the
  authority under the plan, so the turn is a failed turn with that node as the divergence. Neither the surface nor
  the oracle was edited (kill criterion 5); the gap is carried into 6.3 to 6.7 and the research entry.
- Turn 1 exhausted 128,000 output tokens on thinking at effort `medium` (task 4 saw the same at `high` with
  64,000); the preregistered parameters were kept and the turn counted.
- The first launch of the `e4-t2` arm never executed (a PowerShell parse error in the launch command before
  any guard, probe or command ran); the receipt holds one `e4-t2-arm` entry only.
- Turn 1's unmodified root was edit-scope checked as information (`editscope-e4-t1`: 2,798 files compared,
  none differ; `verdict` fail only because no scope file changed).
- Nothing was committed; 6.3 to 6.7 are not started here.

### Commands as executed (PowerShell 7; full argv, cwd and env in the receipt)

```
PY=D:\Source\lesscode\.runs\e2-facdrone\venv\Scripts\python.exe   D=D:\Source\lesscode\docs\experiments\deconvolution   E4=$D\e4-lifecycle   R=$E4\results\receipt.json
EV=--env PYTHONDONTWRITEBYTECODE=1 --env PYTHONIOENCODING=utf-8 --env PYTHONWARNINGS=ignore
$PY $D\receipt_atomic.py $R run patchcheck-frozen-2 --no-probe --cwd $E4 $EV --timeout 600 -- $PY patchcheck.py <frozen> --out results\patchcheck-frozen.json
$PY $D\receipt_atomic.py $R run e4-turn<k> --no-probe --cwd $D\regenerator --env PYTHONDONTWRITEBYTECODE=1 --env PYTHONIOENCODING=utf-8 -- $PY regen_turn.py run --request $E4\results\request-e4-turn<k>.json --cand D:\Source\lesscode\.runs\e4-lifecycle\cand-turn<k> --receipt $R --label e4-turn<k> --max-tokens 128000 --effort medium --thinking-display summarized
$PY $D\receipt_atomic.py $R run editscope-e4-t<k> --no-probe --cwd $D\t51-integration $EV -- $PY editscope.py <frozen> <cand> --scope <the seven> --dirs <the 22 top-level directories> --mentions <turn>\result.json <turn>\response.json --out $E4\results\editscope-e4-t<k>.json
$PY $D\receipt_atomic.py $R run measures4-e4-t<k> --no-probe --cwd $E4 $EV -- $PY measures4.py --root <cand> --structure results\structure-e4-t<k>.json --fmt-dir D:\Source\lesscode\.runs\e4-lifecycle\fmt-t<k> --label e4-t<k>
$PY $D\receipt_atomic.py $R run pureshare-e4-t<k> --no-probe --cwd $E4 $EV -- $PY pureshare4.py <cand> --structure results\structure-e4-t<k>.json --out results\pureshare-e4-t<k>.json
$PY $D\receipt_atomic.py $R run patchcheck-e4-t<k> --no-probe --cwd $E4 $EV -- $PY patchcheck.py <cand> --out results\patchcheck-e4-t<k>.json
$PY $D\guard.py e4-t<k>-arm --receipt $R; <adaptive.scaled(1200) probe -> note e4-t<k>-arm-deadline>; $PY $D\receipt_atomic.py $R run e4-t<k>-arm --cwd $E4 --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env PYTHONWARNINGS=ignore --env PYTHONDONTWRITEBYTECODE=1 --env PYTHONIOENCODING=utf-8 --timeout <scaled+120> -- $PY run_e4.py <cand> --round e4-t<k> --deadline <scaled>
```

## 6.4 Sensitivity on both trees (2026-11-06)

Deconvolution-loop task 6.4 under the frozen [plan.md](plan.md) section 6.4: the same seed (20261106), draw rule
(`t31_arm.draw` over the sorted spec keys), arm (`run_e4.py`: import probe, the 12 seed files with the one deselection,
the 50-sequence replay), deadline rule (`scaled(max(120, 5 × unmutated seconds of that tree))`, hand-written mutants at
base 1,200 s), classification and restore-and-verify on both trees; different populations over different code
(requirement 8.6). Original: reference `.runs/e2-facdrone/source`, mutable copy `.runs/e4-lifecycle/orig-mut`, labels
`e4o-*`. Candidate: reference `.runs/e4-lifecycle/cand-turn3` (accepted at turn 3), mutable copy `.runs/e4-lifecycle/cand-mut`,
labels `e4c-*`. Every arm: `guard.py` first, then `receipt_atomic` with `OPENBLAS/OMP/MKL_NUM_THREADS=1`, serial pytest,
the deadline from a receipted probe (note `<label>-deadline`), the seven files hash-verified before (`-pre`) and after
(`-restore`) every mutant. **No guard stop, no timeout, no rerun, no `-wait` note; all 70 `-pre` and 68 `-restore` notes
`identical=true`** (the references were only read; CRLF kept byte for byte: every post-apply hash in the `-class` notes
equals the hash `e4_sample.py` or `mutants/hand-mutants.json` recorded). The 68 arms ran from 11:58 to 13:50 UTC
(arm seconds 1,372 original, 1,354 candidate); deadline factors 1.08 to 4.0 (`e4o-m6` at the 4.0 clamp, `e4c-m8` at
3.39, while the 6.3 reviewer's probes ran), none reached.

### Apparatus (registered by facts `task-6.4-apparatus`, `task-6.4-apparatus-2` before first use)

| File | Lines | sha256 | Role |
| --- | ---: | --- | --- |
| `e4_arm.py` | 253 | `7b7d0cbb…f2dc` | `t31_arm.py`'s procedure with roots, the seven files and the arm parameterised (`arm`, `mutant`, `hand` inner; `prepare`, `base`, `dry`, `hand-run` drivers); imports `draw`, `hunks_of`, `apply_hunks`, `restore`, `sha` from `t31_arm` |
| `e4_rates.py` | 55 | `2b13e4f4…5abf` | rates from the receipt's last `-class` note per label; `results/e4-rates.json` (`21428651…9c42` after the review correction; `24f0ec54…dcfb` before), `--check` identical (`e4-rates`, `e4-rates-check`; `e4-rates-2`, `e4-rates-check-2`) |
| `cosmic-ray-e4-cand.toml` | 21 | `d67fb85d…d551` | the candidate's mutation configuration, same seven module paths and operator set |
| `test_e4_arm.py` | 32 | `95084096…370e` | the new logic only (classification, status-line parse); `test-e4-arm` 2 passed |
| `mutants/M1-cand.patch` … `M4-cand.patch`, `mutants/hand-mutants.json` | 11, 11, 24, 11; 122 | `bc69d255…`, `a007627a…`, `a5f0f936…`, `ac32cb79…`; `fde02dea…9d4a` | the ports (below) and the recorded patch, target and mutated hashes the arm checks (`mutants-cand-generate`) |
| `results/e4-cand-mutant-sample.json` | 737 | `7d709763…30f0` | 30 of the candidate's 1,488 specs; redraw byte-identical (`e4-sample-cand`, `e4-sample-cand-redraw`) |
| `results/e4-survivor-readings.json` | 64 | `273fc174…7027` (`5193d239…ea4e` before the review correction) | hand-written readings of every survivor, classes `equivalent` / `closed_assumption` / `real`, reviewer hypotheses by name |
| counted in 6.4 | **308** | | budget 260: **overrun of 48 lines, reported, not compacted**; running total 97 (6.1) + 75 (6.2) + 308 = **480 of 900** |

`e4_sample_cand.py` was not needed (`e4_sample.py --root`). `cosmic-ray init cosmic-ray-e4-cand.toml .runs/e4-lifecycle/e4-cand.sqlite`
ran with cwd `cand-turn3` (`e4-cr-init-cand`, 6.1 s); tree snapshots before and after (2,798 files: path, size, mtime)
both hash `40e868d7…071e`, so nothing was written into the candidate. Population 1,488 (original 1,468):
runner 387, group 344, evidence 265, store 221, projection 107, root 103, seams 61. Sample per module: group 11,
evidence 7, store 5, runner 3, root 3, seams 1, projection 0. The 11 `lifecycle_group` draws are the same specs on
both trees (the module sorts first and its population is 343 against 344), so draws 0 to 10 compare like for like;
draws 11 to 29 differ. Composition recorded before any result (note `cand-sample-composition`): 2 syntax errors (#20,
#24), 0 annotation-only, 5 in the declared residue (#5, #6, #8 `lifecycle_review`; #28, #29 root `lifecycle_store`),
1 in `study_baseline` (#27); expected ceiling 22 of 28 = 0.786.

### Unmutated arms

| Label | Exit | Seconds | Factor | Deadline | Groups |
| --- | ---: | ---: | ---: | ---: | --- |
| `e4o-base` (orig-mut) | 0 | 138.6 | 2.94 | 3,528 | root 108 passed, 1 skipped, 1 deselected (91.2 s); research 1 passed (35.7 s); replay fresh 50/50, 317 ops, `level2_ops` 0 |
| `e4c-base` (cand-mut) | 0 | 81.1 | 1.82 | 2,184 | root 108 passed, 1 skipped, 1 deselected (57.1 s); research 1 passed (22.8 s); replay fresh 50/50, 317 ops, `level2_ops` 0 |

Both ran under load (round 0 took 75.7 s idle; the acceptance 49.6 s), so the mutant deadline bases are 692.8 s
(original) and 405.7 s (candidate), scaled per mutant.

### Per mutant, original (`e4o-m<k>`; line numbers of the frozen files)

| k | File:line | Function | Operator | Class | First failing reason | s |
| ---: | --- | --- | --- | --- | --- | ---: |
| 0 | `lifecycle_group:151` | `lifecycle_readings` | AddNot | killed | `test_lifecycle_monitor_runner_db.py::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 26.5 |
| 1 | `lifecycle_group:181` | `lifecycle_readings` | AddNot | killed | `test_lifecycle_monitor_runner_db.py::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 28.6 |
| 2 | `lifecycle_group:222` | `_pod_of` | AddNot | survived | | 50.9 |
| 3 | `lifecycle_group:70` | `lifecycle_retire` | NumberReplacer (`[:17]`) | killed | replay S07-cli-retire op 2 `/result/output` (event id 17 chars) | 24.7 |
| 4 | `lifecycle_group:214` | `_compare` | NumberReplacer (`else 1`) | killed | `…::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 25.6 |
| 5 | `lifecycle_group:266` | `lifecycle_review` | Div→Add | survived | | 66.9 |
| 6 | `lifecycle_group:266` | `lifecycle_review` | Div→FloorDiv | survived | | 93.1 |
| 7 | `lifecycle_group:160` | `lifecycle_readings` | Div→Mod | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 29.1 |
| 8 | `lifecycle_group:254` | `lifecycle_review` | Div→RShift | survived | | 44.9 |
| 9 | `lifecycle_group:209` | `_compare` | Eq→GtE | survived | | 42.1 |
| 10 | `lifecycle_group:210` | `_compare` | Eq→Is | killed | `…::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 19.4 |
| 11 | `lifecycle_seams:66` | `evidence.for_window` (annotation) | BitOr→FloorDiv | survived | | 42.9 |
| 12 | `lifecycle_evidence:147` | `read_evidence` | NumberReplacer (`+ 0`) | killed | `test_lifecycle_run_evidence.py::test_three_signals_two_windows_load_once_and_next_run_refreshes` | 18.3 |
| 13 | `lifecycle_evidence:166` | `LiveEvidence.for_window` (annotation) | BitOr→FloorDiv | survived | | 43.0 |
| 14 | `lifecycle_evidence:126` | `_lookback` (annotation) | BitOr→Pow | survived | | 43.6 |
| 15 | `lifecycle_evidence:79` | `_consumed_query` | Eq→LtE | survived | | 44.1 |
| 16 | `lifecycle_evidence:130` | `_lookback` | GtE→Eq | killed | `test_lifecycle_evidence_intervals.py::test_earliest_value_endpoint_and_exact_recent_window` | 18.2 |
| 17 | `lifecycle_evidence:114` | `_daily` | LtE→IsNot | survived | | 56.2 |
| 18 | `lifecycle_evidence:118` | `_daily` | NotEq→GtE | killed | `test_lifecycle_evidence.py::TestTheLiveEvidence::test_consumed_ic_joined_to_the_days_state` (ValueError) | 22.8 |
| 19 | `lifecycle_evidence:81` | `_consumed_query` | True→False | killed | `test_lifecycle_evidence_projection.py::test_query_projects_only_identity_and_consumed` | 21.4 |
| 20 | `lifecycle_monitor_runner:333` | `MonitorRunner._row` (annotation) | BitOr→RShift | survived | | 49.4 |
| 21 | `lifecycle_projection:141` | `signal_standing` (annotation) | BitOr→LShift | survived | | 51.3 |
| 22 | `lifecycle_store:194` | `LifecycleStore.transition` (annotation) | BitOr→Add | survived | | 46.5 |
| 23 | `lifecycle_store:260` | `_bound` (annotation) | BitOr→BitAnd | survived | | 41.5 |
| 24 | `lifecycle_store:166` | `LifecycleStore._write` | Pow→LShift | incompetent | SyntaxError (`INVALID scope`) | 0.6 |
| 25 | `lifecycle_store:138` | `LifecycleStore.open_flags` | Eq→LtE | killed | `test_lifecycle_monitor_binding_db.py::test_same_bound_runner_uses_only_the_current_census[False]` | 16.7 |
| 26 | `lifecycle_root:34` | `study_baseline` | NumberReplacer (`[-2]`) | survived | | 41.3 |
| 27 | `lifecycle_root:16` | `lifecycle_store` (annotation) | BitOr→RShift | survived | | 43.6 |
| 28 | `lifecycle_root:23` | `lifecycle_store` | Div→LShift | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 19.5 |
| 29 | `lifecycle_root:34` | `study_baseline` | USub→Invert | survived | | 44.6 |

### Per mutant, candidate (`e4c-m<k>`; line numbers of `cand-turn3`)

| k | File:line | Function | Operator | Class | First failing reason | s |
| ---: | --- | --- | --- | --- | --- | ---: |
| 0 | `lifecycle_group:148` | `lifecycle_readings` | AddNot | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 22.8 |
| 1 | `lifecycle_group:179` | `lifecycle_readings` | AddNot | killed | `…::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 22.5 |
| 2 | `lifecycle_group:215` | `_pod_of` | AddNot | survived | | 47.3 |
| 3 | `lifecycle_group:67` | `lifecycle_retire` | NumberReplacer (`[:17]`) | killed | replay S07-cli-retire op 2 `/result/output` | 37.0 |
| 4 | `lifecycle_group:207` | `_compare` | NumberReplacer (`else 1`) | killed | `…::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 28.0 |
| 5 | `lifecycle_group:259` | `lifecycle_review` | Div→Add | survived | | 61.0 |
| 6 | `lifecycle_group:259` | `lifecycle_review` | Div→FloorDiv | survived | | 74.5 |
| 7 | `lifecycle_group:157` | `lifecycle_readings` | Div→Mod | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 28.3 |
| 8 | `lifecycle_group:247` | `lifecycle_review` | Div→RShift | survived | | 74.3 |
| 9 | `lifecycle_group:202` | `_compare` | Eq→GtE | survived | | 91.3 |
| 10 | `lifecycle_group:203` | `_compare` | Eq→Is | killed | `…::test_method_revision_keeps_old_rows_and_reproduces_only_current[True]` | 21.9 |
| 11 | `lifecycle_seams:89` | `bind_lifecycle` | Div→Add | survived | | 44.6 |
| 12 | `lifecycle_evidence:129` | `read_evidence` | Add→BitOr (`ic_days \| 1`) | survived | | 48.4 |
| 13 | `lifecycle_evidence:151` | `LiveEvidence.for_window` | Sub→Div | killed | `test_lifecycle_evidence.py::TestTheLiveEvidence::test_consumed_ic_joined_to_the_days_state` (TypeError) | 20.8 |
| 14 | `lifecycle_evidence:115` | `_daily` | Eq→Gt | killed | `test_lifecycle_evidence_intervals.py::test_earliest_value_endpoint_and_exact_recent_window` (ValueError) | 19.0 |
| 15 | `lifecycle_evidence:76` | `_consumed_query` | Gt→Lt | killed | `…::TestTheLiveEvidence::test_consumed_ic_joined_to_the_days_state` | 30.9 |
| 16 | `lifecycle_evidence:111` | `_daily` | LtE→Eq | killed | `…::TestTheLiveEvidence::test_consumed_ic_joined_to_the_days_state` | 17.7 |
| 17 | `lifecycle_evidence:67` | `baseline_view` | NotEq→Is | killed | `test_lifecycle_evidence.py::TestTheBaselineProjection::test_only_what_the_census_recorded` | 19.6 |
| 18 | `lifecycle_evidence:174` | `LiveEvidence.for_window` | Not deleted | killed | `…::TestTheLiveEvidence::test_no_consumed_ic_is_not_run_with_its_reason` | 25.7 |
| 19 | `lifecycle_monitor_runner:156` | `_verdict` | NumberReplacer (`[-0]`) | killed | `test_lifecycle_monitor_runner_db.py::TestTheRunnerAlone::test_a_later_cutoff_evaluates_only_the_new_windows` | 34.9 |
| 20 | `lifecycle_monitor_runner:179` | `_records` | Pow→BitXor (`^who`) | incompetent | SyntaxError at import (probe `e4c-m20-syntax-probe`; see findings) | 10.7 |
| 21 | `lifecycle_monitor_runner:226` | `MonitorRunner.run` | USub→Not (`lost[not 1]`) | survived | | 64.5 |
| 22 | `lifecycle_store:209` | `LifecycleStore.transition` | AddNot (`not why is None`) | killed | `test_lifecycle_monitor_binding_db.py::test_same_bound_runner_uses_only_the_current_census[False]` | 24.0 |
| 23 | `lifecycle_store:269` | `bind_writers` | NumberReplacer (enroll expected 1) | killed | `…::test_same_bound_runner_uses_only_the_current_census[False]` | 22.5 |
| 24 | `lifecycle_store:60` | `_event_id` | Pow→Mul (`{*asdict`) | incompetent | SyntaxError (`INVALID scope`) | 0.6 |
| 25 | `lifecycle_store:134` | `LifecycleStore.open_flags` | Eq→Lt | killed | `…::TestTheLiveRiskSurface::test_the_flag_is_on_the_status_surface_after_background_completion` (ValueError) | 22.4 |
| 26 | `lifecycle_store:126` | `LifecycleStore.admitted` | Is→NotEq | killed | `…::test_same_bound_runner_uses_only_the_current_census[False]` (ValueError) | 38.0 |
| 27 | `lifecycle_root:34` | `study_baseline` | NumberReplacer (`[-0]`) | survived | | 85.8 |
| 28 | `lifecycle_root:23` | `lifecycle_store` | Div→BitAnd | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 26.5 |
| 29 | `lifecycle_root:23` | `lifecycle_store` | Div→RShift | killed | `…::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce` | 25.2 |

### Rates (R22 convention, killed / (killed + survived), incompetent excluded; bar 0.70 unchanged; `results/e4-rates.json`)

| Tree | Killed | Survived | Incompetent | Rate | Bar 0.70 | Post hoc, equivalent excluded (labelled, not the result) | Post hoc, equivalent and residue-sited excluded (labelled) |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| original (`e4o`) | 12 | 17 | 1 (#24) | 12/29 = **0.414** | not met | 12/21 = 0.571 (8 annotation-only: #11, 13, 14, 20, 21, 22, 23, 27) | 12/15 = 0.800 (+ #2, 5, 6, 8, 26, 29) |
| candidate (`e4c`) | 19 | 9 | 2 (#20, #24) | 19/28 = **0.679** | not met | 19/28 = 0.679 (no equivalent survivor; the first reading excluded #12 and gave 19/27 = 0.704, corrected after the review) | 19/22 = 0.864 (#2, 5, 6, 8, 11, 27) |

**T5 (band 0.60 to 0.75 on the candidate's own sample, falsifier < 0.55 or > 0.85): 19/28 = 0.679, held.** Counting
#20 as the arm's raw exit would give 20/29 = 0.690, also in the band; the 0.70 bar is not met either way. The original's
0.414 against the bar is, as the plan states in advance, a result about the sample's composition (ceiling 15/29 =
0.517 with the 14 expected survivors: 13 of them survived; the fourteenth, #28, was killed, see findings), not about
the arm's sensitivity; the four competent survivors of exercised original code are #2, #9, #15, #17 (readings below).
On the candidate the expected survivors were 6 of 28 (ceiling 0.786); nine survived (#2, #5, #6, #8, #9, #11, #12,
#21, #27): four of the six expected (#28 and #29 were killed against expectation, see findings) plus #9, #11, #12 and
#21. No target is changed. The two rates are on different samples of different populations; the
11 shared `lifecycle_group` draws classify identically on both trees (killed 0, 1, 3, 4, 7, 10; survived 2, 5, 6, 8, 9).

### Hand-written mutants: outcome against the prediction (`mutants/README.md`, note `mutants-cand-predictions`)

| Id | Predicted | Original (`e4o-M<j>`) | Candidate (`e4c-M<j>`, port) | Read |
| --- | --- | --- | --- | --- |
| M1 overdue window one day early | survives (alphabet gap) | survived, 43.8 s | survived, 73.4 s (same construct at cand line 145) | as predicted on both; the gap is expressible by the alphabet (`flag`, `tick 1`, `overdue_flags days=1`), carried to 6.5/6.7 under 9.6 |
| M2 revision bump dropped | caught by seed tests (store test named) and by the corpus at S01 op 1 | caught: `test_lifecycle_monitor_binding_db.py::test_same_bound_runner_uses_only_the_current_census[False]` (`ValueError: a standing revision names … a revision >= 1`), 18.6 s | caught: the same node, 27.7 s (same construct at cand line 196) | caught as predicted; under `-x` the first failing node is the binding test, which runs before the store test the README named (the candidate-side prediction said so); the corpus was not reached |
| M3 not-run records before readings | seed tests pass; caught by the corpus at S09 op 2, `result.emitted.carry@3[0].event` | caught: replay S09-watch-unmeasured-not-run op 2 `/result/emitted/carry@3[0]/boundary (present on one side only)`, 26.1 s | caught: identical divergence, 43.8 s (port: `_emit` folded into `_records`, the `not_run` list built before the readings) | caught as predicted at the predicted sequence, operation and record index; `first_diff` names the first differing key of that record (`boundary`, a reading field absent from a not-run record) rather than `event` |
| M4 census from any authority | seed tests pass; caught by the corpus at S02 op 7 `result.census` | caught: replay S02-refusals-recorded op 7 `/result/census: recorded null vs replayed "enrollment:operator"`, 27.7 s | caught: identical divergence, 37.1 s (same construct at cand line 101) | caught exactly as predicted on both |

Every port is the same semantic change (`mutants/hand-mutants.json`, `port`); each applied cleanly to a scratch copy and
to the mutable copy with the recorded mutated hash (`git apply` exit 0, hashes in the `-class` notes); none was unportable.

### Survivor readings (`results/e4-survivor-readings.json`; every reviewer hypothesis of [review-e4.md](review-e4.md) section 8 addressed by name there)

- Original, 17 survivors: 8 **equivalent** (annotation-only under `from __future__ import annotations`: #11, 13, 14,
  20, 21, 22, 23, 27); 6 **closed_assumption** (#5, 6, 8 in the declared residue `lifecycle_review`; #26, 29 in
  `study_baseline`, reached only through patched names; #2 `_pod_of`, patched by every seed test and oracle
  operation that reaches `lifecycle readings`); 3 **real**: #9 (`_compare` `>=`: a re-measurement greater than the
  stored (value, boundary) is never exercised, H11), #15 (`_consumed_query` `scope <= "live"`: no seeded row has a
  scope sorting below `live`, so Level 1 agrees; **Level 2 distinguishes it**: `rounde4o-m15.json` holds 9 SQL
  divergences, `scope <=` against the recorded `scope =`, first S21 op 1; the arm scores Level 1 only, so it stays
  survived, and the observation goes to the 6.7 protection statement; note `e4o-m15-level2-observation`; H13), #17
  (`_daily` past-cutoff day admitted: no consumed IC observed after a window's cutoff in any exercised read, H7/H13).
- Candidate, 9 survivors: 0 **equivalent**; 6 **closed_assumption** (#2 `_pod_of` patched away; #5, 6, 8
  `lifecycle_review`; #27 `study_baseline`; #11 `bind_lifecycle` in the seams, a function no seed test and no oracle
  operation calls, residue the plan does not declare); 3 **real**: #9 (as the original's #9), #12 (`read_evidence`
  `ic_days | 1` for `+ 1`, the lower fetch margin of the consumed read: the corpus's one odd-`ic_days` run, S21 op 5
  with `ic_days` 5, fetches one day less under the mutant and still agrees because no row the lookback uses lies in
  that day; first read as `equivalent` on the even declarations 60/450/900, corrected after the 6.4 review, note
  `e4-rates-2`; H13) and #21 (`MonitorRunner.run` emits the first lost record's message with the latest record's
  fields: only two failing targets with different failure texts distinguish it; the corpus never loses two signals in
  one run; H6).
- Hypotheses: H1, H2, H3 and H9 show no survivor at the review's sites (no draw landed on `_cycle`'s `None` branch,
  `_compare`'s echo order, the `_t`/`_a` defaults, `from None`/`suppress`, or `_monitored`'s imports); the one seams
  survivor (#11) is unreached code, not a patch-timing effect. The candidate's draws at H2 (#23), H3 (#22), H4 (#25,
  #26, M2, M4), H5 (#19), H10 (#17) and H13 (#13 to #16, #18) are all killed. H6 (#21), H7 (M1, original #17),
  H11 (#9 on both trees) and H13 (candidate #12, original #15 and #17) are faults the corpus does not see at sites
  the review's probes had covered with agreeing inputs. H8 is not read on the candidate (0 of its 107 projection specs were drawn); H12 had no draw on either tree.

### Findings and deviations

- **`lifecycle_root.lifecycle_store` is reached by a seed test.** The plan declares it reached by no seed test;
  `e4o-m28`, `e4c-m28` and `e4c-m29` (its line 23) are killed by
  `test_lifecycle_monitor_runner_db.py::TestThePublishedReadings::test_the_reproduction_command_refuses_what_it_cannot_reproduce`
  (the CLI `readings` command's default store). Whether an oracle operation reaches it is not shown (`-x` stops at the
  seed test). The original's #27 (line 16) survived only because it is annotation-only. Conversely
  `cppi/lifecycle_seams.bind_lifecycle` (`e4c-m11`) is unreached and undeclared. Both go to the 6.7 residue report.
- **`e4c-m20` classification corrected in a second `-class` note** (the last note per label is authoritative, the E3b
  convention): the arm's pytest group exited 1 in 9.4 s with nothing collected and `e4_arm.classify` fell back to
  `exit 1` (killed) because `e4_arm.arm` prints `run_e4`'s stdout only and pytest reports a plugin/conftest load
  failure on stderr. The receipted light probe `e4c-m20-syntax-probe` (apply, `compile()`, import from `cand-mut/src`,
  restore, verify) shows `SyntaxError: invalid syntax (line 179, col 17): ^who,`; the plan classifies a SyntaxError at
  import as incompetent, and the draw was listed as a syntax error in `cand-sample-composition` before the run. Both
  numbers are reported above. `e4_arm.py` was not edited after the runs (kill criterion 5); the stderr loss affects
  log text only, never an exit code; every other killed row carries its `FAILED` node from stdout.
- **Label collision on a case-insensitive filesystem** (full inventory after the review, note `label-collision-full`):
  the brief's labels `e4{o,c}-m<k>` and `e4{o,c}-M<j>` share `logs/<label>.log` and
  `results/sql-divergences-round<label>.json` for k = j in 1..4, and each M run came after its m run. Logs: all eight
  (`e4o-m1..m4.log`, `e4c-m1..m4.log`) now hold the M run's content. Level 2 files: m1 and m4 wrote none on either
  tree (killed by a seed test before the replay), so `rounde4{o,c}-M1.json` and `-M4.json` are M1's and M4's;
  `rounde4{o,c}-m2.json` is the sampled survivor's (M2 was killed by a seed test and wrote none); only
  `rounde4{o,c}-m3.json` is overwritten, by M3 (both killed at the replay). No classification depends on an overwritten
  artefact: every `m1..m4` `-class` note was written from its own log before the M run started (original 12:00 to
  12:08 UTC against M starts 13:02 to 13:04; candidate 13:06 to 13:09 against 13:45 to 13:50), and the receipt's
  command entries of all sixteen runs are intact. Later tasks keep labels distinct beyond letter case.
- Line budget: 308 counted lines against 260 (48 over), in `e4_arm.py`'s two-tree, seven-file, hand-written-mutant
  handling and `e4_rates.py`'s second post hoc reading; reported, not compacted.
- The unmutated arms ran at factors 2.94 and 1.82 (the 6.3 reviewer's probes shared the host), so the mutant deadline
  bases (692.8 s, 405.7 s) are larger than an idle base would give; no deadline was approached (longest arm 93.1 s).
- 34 Level 2 files `results/sql-divergences-rounde4{o,c}-*.json` were written (every arm that reached the replay);
  33 have `level2_ops` 0 and `rounde4o-m15.json` has **9** (the widened `scope <=` predicate in the replayed SQL
  against the recorded `scope =`, first S21 op 1; note `e4o-m15-level2-observation`): the arm scores Level 1 only,
  so #15 stays survived, and the Level 2 view's ability to distinguish it is carried to the 6.7 protection statement.
  Nothing from earlier rounds was moved or deleted.
- **Review corrections (independent review of 6.4, 2026-11-06, NEEDS_FIXES on four points; every primary number
  reproduced):** candidate #12 reclassified `equivalent` → `real` (above; the post hoc candidate readings became
  19/28 = 0.679 and 19/22 = 0.864; `e4-rates-2`, `e4-rates-check-2` identical; R22 rate and T5 unchanged); the
  label-collision inventory completed; the false `level2_ops` sentence replaced by the m15 observation; the survivor
  counts in the text corrected (nine candidate survivors; three original `real` survivors with #2 as
  `closed_assumption`; `e4c-m8` at factor 3.39). No arm was rerun; `e4_arm.py` and `e4_rates.py` are unchanged.
- Nothing was committed; 6.5 to 6.7 are not started here.

### Not claimed

No equivalence between the trees: 0.679 on the candidate's sample and 0.414 on the original's are two readings on
different populations, each with the composition stated above, and thirty draws carry no significance. The hand-written
set shows the arm catches the three faults it was predicted to catch and misses the one it was predicted to miss; it
says nothing about faults outside those four classes. The survivor classes are readings of the code, not proofs; a
`closed_assumption` survivor is unprotected code, and the protection statement of 6.7 must list it.

### Commands as executed (PowerShell 7; full argv, cwd and env in the receipt; drivers `results/driver-6.4-stage-{a,b}.out`)

```
PY=D:\Source\lesscode\.runs\e2-facdrone\venv\Scripts\python.exe   D=D:\Source\lesscode\docs\experiments\deconvolution   E4=$D\e4-lifecycle   R=$E4\results\receipt.json   UV=<winget>\uv.exe
$PY $D\receipt_atomic.py $R run mutants-cand-generate --no-probe --cwd $E4 -- $PY <scratch>\gen_cand_mutants.py <scratch>\mcheck-cand     # M<j>-cand.patch, hand-mutants.json
<register fact task-6.4-apparatus; note mutants-cand-predictions>; $PY $D\receipt_atomic.py $R run test-e4-arm --no-probe --cwd $E4 -- $PY -m pytest test_e4_arm.py -q -p no:cacheprovider
$PY e4_arm.py prepare orig; $PY e4_arm.py prepare cand                       # e4o-copy, e4o-copy-verify, e4c-copy, e4c-copy-verify (guarded copytree, no .git/__pycache__/.pytest_cache)
$PY $D\receipt_atomic.py $R run cand-tree-snapshot-before --no-probe -- $PY <scratch>\snapshot.py <cand-turn3> <out>; $PY $D\guard.py e4-cr-init-cand --receipt $R
$PY $D\receipt_atomic.py $R run e4-cr-init-cand --cwd <cand-turn3> -- $UV tool run --python 3.13 --from cosmic-ray==8.7.0 cosmic-ray init $E4\cosmic-ray-e4-cand.toml D:\Source\lesscode\.runs\e4-lifecycle\e4-cand.sqlite; <cand-tree-snapshot-after>
$PY $D\receipt_atomic.py $R run e4-sample-cand --no-probe --cwd $E4 -- $UV tool run --python 3.13 --from cosmic-ray==8.7.0 python e4_sample.py <e4-cand.sqlite> --root <cand-turn3> --out results\e4-cand-mutant-sample.json   # and e4-sample-cand-redraw to scratch
$PY e4_arm.py base orig; $PY e4_arm.py base cand                             # e4o-base, e4c-base: guard, scaled(1200) probe -> -deadline note, receipt run of `e4_arm.py arm <tree> --round <label> --deadline S`
$PY e4_arm.py dry orig 0 1 ... 29; $PY e4_arm.py hand-run orig 1 2 3 4; $PY e4_arm.py dry cand 0 ... 29; $PY e4_arm.py hand-run cand 1 2 3 4
   # per mutant: -pre, guard, -deadline (scaled(max(120, 5 x base s)) or scaled(1200)), receipt run `e4_arm.py mutant|hand <tree> K --round <label> --deadline S` with the thread pins, -restore, -class
$PY $D\receipt_atomic.py $R run e4c-m20-syntax-probe --no-probe --cwd $E4 -- $PY <scratch>\m20_probe.py; <second e4c-m20-class note; fact task-6.4-apparatus-2>
$PY $D\receipt_atomic.py $R run e4-rates --no-probe --cwd $E4 -- $PY e4_rates.py; $PY $D\receipt_atomic.py $R run e4-rates-check --no-probe --cwd $E4 -- $PY e4_rates.py --check
```

## 6.6 Held-out corpus after acceptance (2026-11-06)

Deconvolution-loop task 6.6 under the frozen [plan.md](plan.md) section 6.6: a corpus recorded from the frozen
original **after** acceptance, with a Hypothesis seed that did not exist before (20261107), replayed on the accepted
candidate `.runs/e4-lifecycle/cand-turn3`. T7 is read on this replay. Every command, probe, hash and verdict is in
`results/receipt.json` (labels `e6-*`, facts `task-6.6-apparatus`, `-apparatus-2`, `-corpus`, `-replay`, `-summary`),
logs in `results/logs/e6-*.log`. Nothing was committed.

**Result: T7 held.** Zero Level 1 divergence on all 74 sequences (424 operations) and zero Level 2 pairs on the
accepted candidate; the same corpus replays with zero divergence at both levels on the frozen original.

### Result table

| Step | Receipt label | Outcome |
| --- | --- | --- |
| Seven files of both trees hashed before | `e6-hash7-before` | frozen original = fact `e4-t3-hash7-after-frozen`; candidate = fact `e4-t3-hash7-after-cand` (1,495 and 1,397 lines) |
| Recorder shim registered, syntax-checked | facts `task-6.6-apparatus` (21 lines), `task-6.6-apparatus-2` (19 lines, `567007a6…47fef`), `e6-shim-compile` | `e4_record2.py`: rebinds `e4_oracle.SEED` to 20261107 and `N_GENERATED` to 48, calls `e4_oracle.record` and `replay_engine` unchanged |
| Recording 1 and 2 from the frozen original | `e6-record-a`, `e6-record-b` | 74 sequences (26 seed + 48 generated), 424 operations; both raw sha256 `27a2cc9a9f3f063d87dc682af7a70565add20b05e39e728e6ab299f39578d8ed`, 5,633,158 bytes; byte-identical; 26.5 s and 23.6 s |
| Validation | `e6-validate`, fact `task-6.6-corpus` | raw bytes = LF body (no CRLF; raw hash = body hash); alphabet identical to the acceptance corpus (`lifecycle_review` absent); `scope_sha256` identical; the 26 seed sequences' operations identical to the acceptance corpus's; sidecars verify |
| Replay on the frozen original (driver) | `e6-replay-frozen` | `replay: level1 no divergence; level2 differences 0 []`, 22.6 s |
| Replay on the accepted candidate (driver, `E4_SOURCE_ROOT=cand-turn3`) | `e6-replay-cand` | `replay: level1 no divergence; level2 differences 0 []`, 23.3 s: **T7 held** |
| Second candidate replay writing the Level 2 file | `e6-replay-cand-l2` | `results/sql-divergences-e6-cand.json` (root `cand-turn3`, `level1` null, `level2_ops` 0, written inside the command window), 22.2 s |
| Overlap with the acceptance corpus (informational) | `e6-overlap` | 1 of 48 generated sequences is an exact duplicate of an acceptance generated sequence; 47 share no first operation under the receipt's identity (same start state and same operations; ignoring the start state, 11 of the 47 share a first operation name) |
| Seven files re-hashed after | `e6-hash7-after` | both trees unchanged |
| Host facts | `e6-host-facts` | Windows 11 10.0.26200, CPython 3.13.16, user `mini\mariu`, no admin token, medium integrity, no symlink privilege, WSL unused, threads pinned to 1, serial |

### Corpus identity

- `.runs/e4-lifecycle/held-out-a.json` and `held-out-b.json` (the plan's names; not committed, `.runs/` is ignored):
  sha256 `27a2cc9a9f3f063d87dc682af7a70565add20b05e39e728e6ab299f39578d8ed`, 5,633,158 bytes, **one figure**: since
  `record()` writes with `newline="\n"` (5.2 review fix) the raw bytes are the hashed LF body, so the raw hash and the
  body hash coincide (the acceptance corpus had two figures, `62791b32…` raw CRLF and `0b2e383d…` body). Both files
  agree. Sidecars: line 1 the raw hash (verifies against the file), line 2 a `#` line with the body hash and size.
  Committed: `results/held-out-e4.json.sha256` (sidecar of `-a`; the file is over 1 MB).
- 74 sequences: the 26 seed sequences S01 to S26 (operations identical to the acceptance corpus's by construction)
  plus 48 generated G00 to G47 (seed 20261107, 3 to 8 operations each, 18 of length 3 and 10 of length 8; start
  states: 17 fresh, 18 from S01, 13 from S08, none from S24), 424 operations (acceptance corpus: 50 and 317).
- `meta`: `hypothesis_seed` 20261107, `generated_requested` 48, Hypothesis 6.170.14, SQLAlchemy 2.0.50, root the
  frozen original, `scope_sha256` the seven frozen hashes, `harness_suspects` empty, `generated_only_signatures`
  `enroll:TransitionRefused<None` and `retire:TransitionRefused<None` (the same two as the acceptance corpus).

### T7 reading

| Id | Bar | Held-out corpus on `cand-turn3` | Reading |
| --- | --- | --- | --- |
| T7 | zero Level 1 divergence on every sequence; Level 2 pairs, if any, passing the query differential | 0 Level 1 divergences on 74/74 sequences; 0 Level 2 pairs (`qdiff.py` not run: nothing to compare) | **held** |

The 48 generated sequences are the ones the regenerator could not have been shaped by (recorded after acceptance
from a seed that did not exist before); the 26 seed sequences repeat the acceptance corpus and are reported
separately: they too replay without divergence, which the acceptance arm already showed. The overlap note says how
much of the 48 is new in operation terms: one exact duplicate of an acceptance generated sequence (a 3-operation
sequence), 47 with no shared first operation; `open_flags` appears in the held-out generated set and not in the
acceptance generated set; no operation name appears only in the acceptance set.

### Deviations from the task brief (the plan wins)

- File names `held-out-a/b.json` and `results/held-out-e4.json.sha256` as plan 6.6 names them, not `golden2-*`.
- Recording and replay deadlines from `adaptive.scaled(600)` (plan Budgets: recording base 600 s, replay base 600 s),
  not 1,200 s; every `scaled()` probe is receipted as `<label>-deadline` (factors 1.13, 1.14, 1.26, 1.14, 1.44;
  deadlines 678 to 864 s; longest run 26.5 s). Guards: five, all `run` at attempt 1; no stop, no timeout, no rerun,
  no reaped job.
- The T7 replays use `e4_oracle.py replay --golden <held-out>` exactly as plan 6.6 says; the Level 2 file the brief
  asks for (`results/sql-divergences-e6-cand.json`) comes from a second replay through the shim's `replay` mode, which
  calls `e4_oracle.replay_engine` unchanged (label `e6-replay-cand-l2`, same result). `e4_oracle.py`, `run_e4.py` and
  `test_e4_replay.py` were not edited: `--golden` and `E4_SOURCE_ROOT` already exist, so driver mode needs no shim
  for the golden path (`test_e4_replay.py`'s constant golden path is not used by 6.6).
- The shim was first registered at 21 lines; before its first run the two double blank-line separators became
  single ones (no statement changed) and it was re-registered at 19 lines (`task-6.6-apparatus-2`).
- Line budget: `e4_record2.py` 19 counted lines against the plan's 20; running total 667 (fact `task-6.5-phase1`)
  + 19 = **686 of 900**, tests excluded (as recorded; the live total is 691 because `e4_rates.py` grew to
  55 lines after the 6.4 review, see the 6.5 review).

### Not claimed

Zero divergence on 74 sequences is a reading on this corpus, not equivalence: the alphabet is the one of
scope-rationale section 4 minus `lifecycle_review`, the generated sequences are 3 to 8 operations over at most two
starting states, and the residue of 5.2 (unreached functions, the deselected node, the skipped operator-tier test, a
breaching window through the real retained-evidence join, threaded races) is untouched. The one duplicate sequence
contributes nothing new; the 47 others are new operation sequences from the same alphabet and domains, not new
input classes. Nothing about the regenerator's determinism or about other scopes.

### Commands as executed (PowerShell 7; full argv, cwd and env in the receipt; driver `<scratch>\e66_heavy.py` = guard, `scaled(600)` note, receipt run with the thread pins and `--timeout` scaled + 120)

```
PY=D:\Source\lesscode\.runs\e2-facdrone\venv\Scripts\python.exe   D=D:\Source\lesscode\docs\experiments\deconvolution   E4=$D\e4-lifecycle   R=$E4\results\receipt.json
PP_ORIG="<frozen>/src;$D/e1b-facdrone-evidence-publication;D:/Source/lesscode/.runs/e1b-facdrone/extras;$D/e2-facdrone-edge-census"   PP_CAND="<cand-turn3>/src;(same tail)"
$PY $D\receipt_atomic.py $R run e6-hash7-before --no-probe -- $PY <scratch>\hash7.py <frozen> <cand-turn3>
<fact task-6.6-apparatus>; $PY $D\receipt_atomic.py $R run e6-shim-compile --no-probe --cwd $E4 -- $PY -c "ast.parse(open('e4_record2.py').read())"; <fact task-6.6-apparatus-2>
$PY <scratch>\e66_heavy.py e6-record-a 600 $E4 {"PYTHONPATH":PP_ORIG} -- $PY e4_record2.py record --out D:/Source/lesscode/.runs/e4-lifecycle/held-out-a.json   # then e6-record-b
$PY $D\receipt_atomic.py $R run e6-validate --no-probe -- $PY <scratch>\e66_validate.py            # hashes, counts, sidecar line 2, results\held-out-e4.json.sha256, fact task-6.6-corpus
$PY <scratch>\e66_heavy.py e6-replay-frozen 600 $E4 {"PYTHONPATH":PP_ORIG} -- $PY e4_oracle.py replay --golden D:/Source/lesscode/.runs/e4-lifecycle/held-out-a.json
$PY <scratch>\e66_heavy.py e6-replay-cand 600 $E4 {"E4_SOURCE_ROOT":<cand-turn3>,"PYTHONPATH":PP_CAND} -- $PY e4_oracle.py replay --golden D:/Source/lesscode/.runs/e4-lifecycle/held-out-a.json
$PY $D\receipt_atomic.py $R run e6-overlap --no-probe -- $PY <scratch>\e66_overlap.py; $PY $D\receipt_atomic.py $R run e6-host-facts --no-probe -- $PY <scratch>\hostfacts.py
$PY <scratch>\e66_heavy.py e6-replay-cand-l2 600 $E4 {"E4_SOURCE_ROOT":<cand-turn3>,"PYTHONPATH":PP_CAND} -- $PY e4_record2.py replay --golden <held-out-a> --l2 $E4\results\sql-divergences-e6-cand.json
$PY $D\receipt_atomic.py $R run e6-hash7-after --no-probe -- $PY <scratch>\hash7.py <frozen> <cand-turn3>; <fact task-6.6-replay>; <fact task-6.6-summary>
```
