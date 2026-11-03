# E1b apparatus: trace oracle on Facdrone's evidence-publication scope

Preregistration and amendments: [plan.md](plan.md). Amendment 1 fixes the seed set and amendment 2
the harness details. Amendment 3 (owner) sets module-level test selection and 60 mutants.
Amendments 4 to 6 (execution session) record the baseline-failure handling, the collection-error
rerun and the 3-hour cap. The owner's per-line amendment, also headed "Amendment 4", sets per-line
test selection. Amendment 7 implements it.

**Status (2026-11-03): executed in full, native Windows.** All 60 sampled mutants completed in
both arms, so no count is partial. The research entry under `docs/research/` has not been
written yet.

## Results

### Predictions

| | Prediction | Measured | Verdict |
| --- | --- | --- | --- |
| Q1 | k_B ≥ k_A + 0.5 × (1 − k_A) | k_A = 42/59 = **0.712**, threshold 0.856; k_B = 44/59 = **0.746** (closes 2 of 17 survivors, 11.8% of the gap) | **fails** |
| Q2 | Arm B catches M1, M2, M3 | B catches all three; A misses M3 | **holds** |
| Q3 | B coverage ≥ 85% and ≥ A + 5 points | full seed set: B 95.36%, A 95.23% (coverage.py, statements + branches); branch-only B 88.62%, A 88.32% | **fails** (first part holds, gain +0.13 points) |
| Q4 | ≥ 3 behaviours only generated sequences produce, ≥ 1 on a failure path | 2: `commit:EventConflictError`, `sim_persist:ValueError`, both failure paths; `harness_suspects` empty | **fails** (2 < 3) |
| Q5 | two recordings agree on every digest without masking | both recordings byte-identical, sha256 `92a48f77…7671` (51 sequences, 290 operations); replay on the frozen copy reports no divergence | **holds** |

Per the plan's kill criteria, Q1 failing after the budget **rejects H8(a) for the high-scatter
class on this scope**, under the method as amended. The oracle mechanics work: digests are
deterministic, replay is clean, and the replay catches a stored-representation mutant (M3)
that the seed tests miss. But the replay adds little discrimination over a seed suite that
already executes 95% of the scope.

### Two-arm mutation table (cosmic-ray 8.7.0, same 60-mutant sample, seed 20261103)

| Arm | Killed | Survived | Incompetent | Timeout | Void | Kill rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A: seeds (R16 + selected pytest files) | 42 | 17 | 0 | 0 | 1 | 42/59 = 0.712 |
| B: A + trace replay | 44 | 15 | 0 | 0 | 1 | 44/59 = 0.746 |

- Population 805 mutants: 498 in `cppi/state_repository.py`, 307 in `decision_evidence_publication.py`.
  The sample holds 42 and 18 respectively.
- Killed only by B (2): both on line 299 of `decision_evidence_publication.py`, the
  `route.kind == "scored"` condition in the run-slice binder (`Eq_LtE` and `AddNot`). The
  replay diverges at `S01 op 2 /sql[…]/content_digest`. Killed only by A: none.
- **Void (1, both arms):** `ReplaceBinaryOperator_Mul_Pow` turns the keyword-only `*` of
  `seal_evidence` (line 238) into `**`, a syntax error. The fail-closed root probe cannot import
  the scope and prints `run_arm: INVALID`. Amendment 2.6 voids it, so it counts in neither
  arm. A syntax error would otherwise be incompetent or killed.
- **Uncovered (8 of 60; owner's per-line amendment):** no seed file's coverage context executes
  a changed line. They count as survived in the primary figures. Observed: 7 survived in both
  arms (the R16 check and the replay included) and 1 is the void above. Five of them change
  `|` in type annotations (lines 638 ×2, 255 of `state_repository`; 191, 263 of the publication
  module). Both modules use `from __future__ import annotations`, so these are very likely
  equivalent mutants. Two (`Eq_Is` and `Eq_LtE`, lines 240 and 241) sit in the legacy branch of
  `_placed_under`, which no seed executes. That is a real gap.
- Covered survivors in both arms: 8, all in `state_repository` (lines 150, 166, 169, 186, 200,
  418, 584, 642). Most are `==` to `>=`, `<=` or `is` swaps on query predicates and identity
  comparisons that the seed data does not separate.
- Timeouts: none. The per-mutant deadlines were never reached (`deadline exceeded` appears
  in neither dump).

### Hand-written semantic mutants (`run_arm.py` exit; nonzero = caught)

| Mutant | Selected seed files | Arm A | Arm B | Catching evidence |
| --- | ---: | --- | --- | --- |
| M1 ledger adopted before commit | 38 | caught (93 s) | caught (89 s) | `test_run_slice_writers_db.py::test_live_failure_after_batch_insert_rolls_back_then_can_retry` (both arms, `-x` stops there) |
| M2 legacy route skips reference validation | 31 | caught (83 s) | caught (90 s) | `test_run_slice_writers_db.py::test_live_legacy_dictionary_cannot_bypass_reference_presence` |
| M3 `compact_capture` zlib level 1 → 9 | 3 | **missed** (exit 0) | caught | replay: `S09-codecs-compact op 10 /result/consumed_capture` differs |

### Coverage of the 8 scope files (1,217 statements, 334 branches)

| Run | coverage.py % | branch-only % | Missing statements / branches |
| --- | ---: | ---: | --- |
| Arm A, whole 191-file seed set (primary) | 95.23 | 88.32 | 35 / 39 |
| Arm B, same + replay (primary) | 95.36 | 88.62 | 34 / 38 |
| Arm A, 151-file module union (amendment 3 set, superseded) | 92.97 | 83.83 | 55 / 54 |
| Arm B, same + replay | 93.23 | 84.43 | 53 / 52 |

The replay adds exactly one statement and one branch, both in `decision_evidence_publication.py`.
`decision_evidence_store.py` stays at 14 missing statements and `state_repository.py` at 10 in
both arms. Per file: `results/cov-a/scope-coverage.json`, `results/cov-b/scope-coverage.json`.

### Time

| Measure | Value |
| --- | ---: |
| Seed-confirmation coverage run (gate tier, serial), incl. research rerun | 2,601.5 s + 117.0 s |
| T_A, unmutated Arm A, whole seed set | 400.4 s |
| T_B, unmutated Arm B, whole seed set | 513.8 s |
| Per-mutant selected sets, unmutated (24 distinct + whole set) | A 6.3 to 400 s, B 37.8 to 514 s; sums 2,534 s and 2,851 s |
| Deadlines (max(120, 5 × t)) | A 120 to 2,002 s, B 189 to 2,569 s; cosmic-ray backstop 2,122.2 / 2,689.2 s |
| cosmic-ray baseline, Arm A | 486.5 s (passed) |
| cosmic-ray exec Arm A / Arm B (60 mutants each) | 2,790.9 s / 3,057.2 s (cap 10,800 s, not reached) |
| M1 to M3, six arm runs | 450 s |
| Wall clock, step 0 to last coverage run (receipt) | 5.51 h (11:40 to 17:11 UTC) |

### Harness lines (physical, `wc -l`)

| File | Lines |
| --- | ---: |
| `e1b_oracle.py` | 406 |
| `run_arm.py` | 221 |
| `scope.py` | 78 |
| `sample_mutants.py` | 74 |
| `receipt.py` | 20 |
| `test_e1b_replay.py` | 12 |
| **Harness total** | **811** (budget 820 after the owner's per-line amendment; 700 originally, 760 after amendment 3) |
| `results/analysis/cr_counts.py` (post-hoc counting of the dumps, outside the harness) | 65 |

The original budget of 700 was exceeded by 111 lines. The overrun went to selection
(module-level, then per-line), baseline-failure deselection, per-set deadlines and the
wall-clock cap.

## Files

| File | Role |
| --- | --- |
| `scope.py` | the 8 R15 case A files, 18 entry callables, hashes against the R15 manifest; `changed_lines` (amendment 7) |
| `e1b_oracle.py` | recorder/replayer: embedded PostgreSQL through `e2_winshim`, SQL listener, table dumps, 11 seed + 40 Hypothesis sequences, first-divergence report |
| `test_e1b_replay.py` | Arm B pytest target: `replay_engine(pg_engine)` |
| `run_arm.py` | arm runner and cosmic-ray test command: R16 check, per-line selection, baseline deselects, coverage with per-file contexts, seed confirmation, `time` (per-set deadlines), `cap` (wall-clock cap) |
| `sample_mutants.py` | shared 60-mutant sample with each mutant's changed lines; writes the deadline rule into the configs |
| `receipt.py` | E2's `receipt.py` pointed at this directory |
| `seeds.md` | coverage-confirmed seed set: R16 harness + 191 pytest files |
| `cosmic-ray-arm-a.toml`, `cosmic-ray-arm-b.toml` | the two mutated modules, local distributor, `--deadline auto`, backstop timeout |
| `mutants/m1.patch`, `m2.patch`, `m3.patch` | hand-written mutants |
| `results/receipt.json` | every command (argv, cwd, env, UTC start/end, seconds, exit), `facts` (versions, package list, hashes, line counts), `notes` (frozen hashes, k_A before Arm B, summary, the WSL incident) |
| `results/selection-map.json` | frozen line-keyed map (sha256 `3d7a5197…8e66`); `selection-map.module-level.json` is the amendment 3 map (`0ee9a89d…c150`) |
| `results/mutant-sample.json`, `results/deadlines.json` | sample with changed lines; measured per-set times and deadlines |
| `results/cr-arm-a.jsonl`, `cr-arm-b.jsonl`, `cr-counts.json` | cosmic-ray dumps and counts (per mutant, intersection, arm-only kills) |
| `results/golden-trace.json` (+ `-run2`, `.sha256`) | the golden trace (8.7 MB) and the determinism repeat |
| `results/r16-baseline.json` | R16 harness output on the frozen copy in this venv |
| `results/cov-all/`, `cov-a/`, `cov-b/`, `cov-*-union/` | coverage data: census, arms on the whole seed set, arms on the module union |
| `results/baseline-failures-by-cause.json`, `seeds-from-coverage.txt`, `seeds-static-candidates.txt` | census classification and seed lists |
| `results/logs/` | console output of every recorded step |

## Environment

- Native Windows 11, plain user account. No symlink privilege, no elevation, no Developer Mode.
- Frozen source `D:\Source\lesscode\.runs\e2-facdrone\source` (Facdrone `d49f48bd…`). After the
  run, all 2,657 files tracked at that commit match the frozen copy by git blob hash (CRLF from
  extraction normalised). `scope.py` reports `changed: []` for the frozen copy and for both
  cosmic-ray copies after `exec`. `D:\Source\facdrone` HEAD is unchanged, and `git status
  --short` there is empty. Mutants ran only in `.runs/e1b-facdrone/{arm-a,arm-b,m1,m2,m3}`.
- Interpreter: the E2 venv (CPython 3.13.16, coverage 7.13.0). Hypothesis 6.170.14 in
  `.runs/e1b-facdrone/extras`. cosmic-ray 8.7.0 through `uv tool run`. SQLAlchemy 2.0.50. Thread
  variables are set to 1 for every run, and pytest always runs serially, one heavy job at a time.

## Oracle in brief

The oracle runs operation sequences against Facdrone's own embedded PostgreSQL. Each
sequence starts by truncating the 10 tables. Every operation records:

- its return value or its exception (type, message, cause type);
- every SQL statement with its parameters (`before_cursor_execute`);
- an ordered dump of the 10 tables, with JSON columns as stored text;
- the files written under the sequence's artifact root.

The operations are:

- `boot`, `restart`, `handover` and `commit` on the live writer `SqlLiveStateRepository`.
  `commit` takes four modes: typed run-slice, legacy v1 dictionaries, dictionary-shaped
  batches carrying run-slice references, and absent capture. It takes three decision modes:
  next, duplicate id (which injects a rollback after the slices are published) and stale
  version. Namespaces are `auto`, explicit or blank. Input shapes are single, duplicate
  reference, forward reference, missing reference and twin slices.
- `read`, which calls `read_resolved_evidence` for every execution.
- `tamper` on `row_digest` or `content_digest`.
- `sim_persist` and `sim_supersede` (`persist_run` and `delete_run_evidence` with Facdrone's
  `full_result` fixtures).
- `seal`, which runs the legacy and run-slice sealers and binders, `payload_batches` and
  both stored-payload functions.
- `compact`, which runs `compact_capture` with the research `StudySlices` writer.
- `r16`, which runs the R16 harness in-process.

The seed sequences S01 to S11 transcribe `test_live_slice_namespace_db.py`,
`test_run_slice_writers_db.py`, `test_decision_slice_publisher_db.py` (the chain of 20 links
that ends in a keyframe) and the R16 harness. Replay re-executes the recorded operations
and prints the first divergent sequence, operation and JSON path.

## Commands as executed (Git Bash; full argv in the receipt)

```bash
ED=D:/Source/lesscode/docs/experiments/deconvolution/e1b-facdrone-evidence-publication
S=D:/Source/lesscode/.runs/e2-facdrone/source
V=D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe
W=D:/Source/lesscode/.runs/e1b-facdrone
PP="$ED;$W/extras;D:/Source/lesscode/docs/experiments/deconvolution/e2-facdrone-edge-census"
EV="--env PYTHONPATH=$PP --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env PYTHONWARNINGS=ignore"
R="uv.exe run --no-project --python 3.13 python receipt.py run"
CR="uv.exe tool run --python 3.13 --from cosmic-ray==8.7.0 cosmic-ray"
# 0. R16 baseline
$R r16-record $EV -- $V run_arm.py r16-record
# 1. seed confirmation (amendments 4, 5), then seed list + line-keyed selection map (amendment 7)
$R seeds-coverage --cwd $S $EV -- $V $ED/run_arm.py all --cov $ED/results/cov-all
$R seeds-coverage-research-rerun --cwd $S $EV --env E1B_GROUP=packages/facdrone-research -- $V $ED/run_arm.py all --cov $ED/results/cov-all
$R seeds-list-line-map $EV -- $V run_arm.py seeds results/cov-all
# 2. golden trace twice, compare (Q5), replay
$R record-1 $EV -- $V e1b_oracle.py record
$R record-2 $EV -- $V e1b_oracle.py record --out results/golden-trace-run2.json
$R replay-frozen $EV -- $V e1b_oracle.py replay
# 3./4. unmutated arms with coverage on the whole seed set (primary; union runs kept as *-union)
$R arm-a-cov-full --cwd $S $EV -- $V $ED/run_arm.py A --cov $ED/results/cov-a   # then coverage combine; run_arm.py report
$R arm-b-cov-full --cwd $S $EV -- $V $ED/run_arm.py B --cov $ED/results/cov-b
# 6. working copies (native shutil.copytree), init, shared sample with changed lines
$R copy-arm-a-native -- $V -c "import shutil; shutil.copytree(r'$S', r'$W/arm-a', ignore=shutil.ignore_patterns('__pycache__'))"   # same for arm-b
$R cr-init-a --cwd $W/arm-a -- $CR init $ED/cosmic-ray-arm-a.toml $W/arm-a.sqlite                    # same for b
$R sample -- uv.exe tool run --python 3.13 --from cosmic-ray==8.7.0 python sample_mutants.py sample $W/arm-a.sqlite $W/arm-b.sqlite 60 20261103
# 5. per-set deadlines (amendment 7), then the configs
$R time-a $EV -- $V run_arm.py time A ; $R time-b $EV -- $V run_arm.py time B
$R timeout-configs-2 -- $V sample_mutants.py timeout
# 7. Arm A, k_A into the receipt, then Arm B; 3 h cap each (amendment 6)
$R cr-baseline-a-3 --cwd $W/arm-a -- $CR --verbosity INFO baseline $ED/cosmic-ray-arm-a.toml
$R cr-exec-a --cwd $W/arm-a -- $V $ED/run_arm.py cap 10800 -- $CR --verbosity INFO exec $ED/cosmic-ray-arm-a.toml $W/arm-a.sqlite
$R cr-exec-b --cwd $W/arm-b -- $V $ED/run_arm.py cap 10800 -- $CR --verbosity INFO exec $ED/cosmic-ray-arm-b.toml $W/arm-b.sqlite
$CR dump $W/arm-a.sqlite > results/cr-arm-a.jsonl ; $CR dump $W/arm-b.sqlite > results/cr-arm-b.jsonl
$V results/analysis/cr_counts.py results/cr-arm-a.jsonl results/cr-arm-b.jsonl > results/cr-counts.json
# 8. M1 to M3, own copies, both arms
$R copy-m1 -- $V -c "...copytree to $W/m1..." ; $R apply-m1 --cwd $W/m1 -- git apply $ED/mutants/m1.patch
$R m1-arm-A --cwd $W/m1 $EV -- $V $ED/run_arm.py A ; $R m1-arm-B --cwd $W/m1 $EV -- $V $ED/run_arm.py B   # same for m2, m3
```

Kill rate = killed / (killed + survived). Incompetent results are excluded, `run_arm: INVALID`
outputs are void, deadline outputs count as killed but are listed separately, and uncovered
mutants count as survived (owner's per-line amendment).

## Deviations, amendments and incidents

1. **Amendment 4 (session):** the census runs every group even after failures. Failing node ids
   (236) are frozen in the selection map and deselected in both arms when their file is
   selected. 10 lie in seed files.
2. **Amendment 5:** serial pytest aborted `packages/facdrone-research` at collection (8 modules
   import `fcntl`). The census therefore passes `--continue-on-collection-errors`, and the group
   was rerun alone.
3. **Amendment 6:** amendment 2.8's 72 s check was superseded (T_A > 72 s). A 3 h cap per `exec`
   applied instead. Neither arm reached it (46.5 and 51.0 min).
4. **Owner's per-line amendment and Amendment 7:** per-mutant selection by changed line, 820-line
   budget, uncovered mutants counted as survived, deadlines max(120, 5 × measured set time).
   T_A and T_B were first measured on amendment 3's 151-file module union (409.5 s and 470.8 s;
   receipt `arm-a-base`, `arm-b-base`). That union is superseded, and its coverage is kept as a
   secondary row. Order changed: init and sample before timeouts, because the deadlines depend
   on the sample.
5. **Harness corrections during execution** (each in the receipt):
   - `cosmic-ray baseline` in 8.7.0 takes only the config (the first attempt, exit 2, had a
     session argument).
   - The first deadline rewrite also deleted the closing quote of `test-command`, and the
     config failed to load (exit 78). The regex was fixed, the quote restored, and the configs
     rewritten (`timeout-configs-2`) before any `exec`.
   - cosmic-ray stores operator arguments JSON-encoded twice. `sample_mutants.lines` decodes
     both layers.
   - Outcomes are upper-case in 8.7.0, and the counting script normalises them.
6. **Incident, native-only rule:** receipt labels `copy-arm-a` and `copy-arm-b` ran `bash -c cp …`
   through Python's `subprocess`. Windows `CreateProcess` searches `System32` before `PATH`, so
   `bash` resolved to `C:\Windows\System32\bash.exe`, the **WSL launcher**. Both calls therefore
   executed in WSL. Both failed at `cp` ("cannot stat" the D: path) and changed nothing. The
   copies were redone with native `shutil.copytree` (`copy-arm-*-native`). No other step
   invoked bash from Python. This is recorded as a note in the receipt.
7. The plan's predictions Q1 to Q5 were not changed.

## Limitations

1. **Unprotected behaviour on this host.** The census found 236 failing tests on the unmodified
   copy. By cause (junit text): 136 symlink privilege (`WinError 1314`, tests that symlink
   `conf/base` into a temp tree: `test_mandate_record` 47, `test_daemon_profile` 40,
   `test_cfd_two_step_record` 17, `test_reporting_production_composition` 8, and 13 more
   modules), 22 file lock (`WinError 32` on temp files: `test_exit_observation_cache`,
   `test_exit_observations`, `test_exit_evaluate`, `test_exit_walk_forward_boundaries` and
   others), 19 POSIX-only (`fcntl` imports, including the 8 research modules that fail at
   collection), and 67 other. The task brief cites 131 symlink and 22 file-lock failures;
   this census counts 136 symlink failures by its own classification. None was
   repaired. Whatever those tests guard (mandate and daemon-profile records built from
   `conf/base`, exit-observation caches, POSIX file locking in research stores) is
   **unprotected in both arms**. Six seed files lose tests to the deselect list, among them
   `test_study_slice_evidence.py::test_slice_store_refuses_symlink_escape` and four
   `test_exit_trail_golden.py` cases.
2. **Small sample, one scope.** 60 of 805 mutants, 18 of them in the publication module, where
   both B-only kills fall. The Q1 verdict has a wide interval: 44/59 against a threshold of
   ≈ 51/59.
3. **Uncovered and equivalent mutants.** 8 of 60 have no selecting seed file, and 5 of them are
   probably equivalent (annotations under `from __future__ import annotations`). They depress
   both kill rates equally. Without them, k_A = 42/52 = 0.808 and k_B = 44/52 = 0.846; the
   corresponding Q1 threshold would be 0.904, so Q1 fails either way.
4. **Selection is coverage-based, not exhaustive.** A seed file that would kill a mutant without
   executing its changed lines is not run. Examples are import-time effects and lines that
   coverage attributes to the empty collection context. This favours survival in both arms
   equally. The replay always runs in Arm B whatever the selection.
5. **Fail-closed probe and syntax errors.** The root probe cannot tell a syntax-error mutant
   from a wrong root, so that mutant is void instead of killed or incompetent.
6. **Masking and boundary** (as before): only `datetime` values are masked, so a consistent
   timestamp shift stays invisible. Effects outside SQL, the 10 tables, return values and
   artifact files are not intercepted. Safety rule 3 (fail closed on unintercepted effects)
   is therefore not enforced.
7. **Q4 counts signatures** (operation × outcome type). Novel values inside dumps do not
   count. The generated distribution is fixed by seed, not by design.
8. **Windows process control.** The R16 subprocess and the import probe have no deadline. No
   hang occurred.
9. The `facdrone_kernel` scope file always loads from the frozen copy (editable install). It is
   not mutated. Its coverage is attributed to the frozen path.
