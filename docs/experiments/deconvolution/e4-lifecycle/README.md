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
