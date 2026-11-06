# E4 plan: the deconvolution loop on the lifecycle monitor group (second scope)

- Written: 2026-11-06, Europe/Berlin, deconvolution-loop task 6.1, after task 5.2 (round 0 on an
  identical copy passed) and before any heavy step of E4: no regeneration turn, no acceptance run on a
  candidate, no mutant execution, no solver run, no second recording has happened. **Fixed**: the
  predictions, kill criteria, frozen inputs and budgets below do not change afterwards; open decisions
  taken later are dated amendments at the end (requirement 11.1).
- Hypothesis: H8(c) of [R19](../../../research/2026-11-03/01-dynamic-deconvolution-direction.md) on a
  second Python scope, with the loop as the spec `.kiro/specs/deconvolution-loop` describes it and the
  regenerator as the recorded single-turn program of task 4. What E4 tests is the prediction table of
  [the report's section 8](../../../reports/deconvolution-loop-applied.md) on one scope, and the two
  pieces that have no evidence yet: the regenerator program and the turn rule (requirements 6.7, 6.8).
- Type: preregistered. Predictions T1 to T7 and the structural contract S are fixed below with absolute
  bars; failed predictions are reported as failed (requirement 10.3).
- Review edits (edited 2026-11-06 after the independent review of 6.1, before the owner's go, so the
  plan was not yet frozen): the T1 verdict at turn 4, the sample-ceiling arithmetic in 6.4, the patching
  sentinel check in 6.2 and the kill-criteria headings, the 0.70 bar's citation, the CRLF rule for 6.4,
  and the registration rule for later apparatus files. Each place is marked "(edited 2026-11-06 after
  review, before go)". No number of any prediction changed.
- Runtime: native Windows 11, plain user account, no WSL, never a bare `bash` from scripts, no symlink
  privilege, one heavy job at a time, numeric thread counts pinned to one, every heavy step behind
  `guard.py` and every command through `receipt_atomic.py` into `results/receipt.json` (append-only).
  Interpreter `.runs/e2-facdrone/venv/Scripts/python.exe` (CPython 3.13.16) with
  `PYTHONDONTWRITEBYTECODE=1 PYTHONIOENCODING=utf-8`. Deadlines are adaptive (`adaptive.py`, reference
  `ref_bench_s` 0.212 of 2026-11-04 09:15); the factor that sets a deadline is taken from a receipted
  probe (lesson of 5.2, note `round0-ident-deadline-provenance`).

## Question

Can the regenerator program, given only the seven lifecycle files, their derived surface, the
definitions, the targets, the structural contract and the acceptance command, produce within at most four
turns a candidate that the E4 acceptance arm accepts (50 recorded sequences, 317 operations, 12 seed test
files with one deselection), with the size and edge reductions the report's section 8 predicts for scopes
of this class; is the acceptance corpus as sensitive on the candidate as on the original; does review,
random input or the differential find at least one difference the recording cannot see; and does a corpus
recorded after acceptance replay without divergence on the accepted candidate?

## Definitions

The definitions the regenerator receives are fixed in `results/request-e4-turn1.json` (`definitions`,
`contract`; hashed below) and are the E3 definitions with the seven files as the set: public API (every
name outside code imports, reads, patches or overrides, private names included), pure core (no engine,
connection or session; no SQL), effect shell, Level 1 observation (result or error with cause type,
ordered dumps of `signal_lifecycle_event`, `signal_monitor_reading`, `decision_evidence`, project-root
files, emitted records in emission order, CLI exit code and output), Level 2 observation (SQL text and
parameters), write site (the measurer's static regex, `.add` included), intra-set call edge and module hop
(the E3 measurer's static name-based rule). Kill rate: killed / (killed + survived), incompetent excluded,
survivors in the denominator (R22 convention, requirement 8.2); a reading that also excludes mutants
judged equivalent by inspection is post hoc and labelled (8.3).

## Frozen inputs

Specimen Facdrone `d49f48bd…`, frozen copy `.runs/e2-facdrone/source` (read only throughout; a tree
snapshot of its 4,851 files, sizes and mtimes hashes to `fa4cd12b…ea3a9` before and after this task's
only command that ran inside it, `cosmic-ray init`). Scope and alphabet as approved in
[scope-rationale.md](scope-rationale.md) (owner, 2026-11-06); apparatus and round 0 in [README.md](README.md).

**The seven scope files** (sha256 of the frozen copy, from receipt fact `task-5.2-round0`, re-verified by
`e4_sample.py` today in `results/e4-orig-mutant-sample.json`):

| File | sha256 | Lines |
| --- | --- | ---: |
| `src/facdrone/shells/lifecycle_monitor_runner.py` | `fdd35e1c42caabe83c5fcb8136924355a707f9a684fc828a84a454674657b2f5` | 397 |
| `src/facdrone/shells/lifecycle_store.py` | `35d9f27a7727c49cb49086254c2513b897a89621f3787ef69c6d65c09f38d974` | 289 |
| `src/facdrone/shells/cli/lifecycle_group.py` | `03bfef33c14209407d8fcbac9911ce0f3cb51601a68c64cc1e96a7d607d8b6cb` | 278 |
| `src/facdrone/shells/lifecycle_evidence.py` | `5ae9a370f74516249ca5663b6e9907069d97da8ba04ec0af23d28fda884fe354` | 215 |
| `src/facdrone/shells/lifecycle_projection.py` | `adb6075c061d48058a56b69655d0486bc6ea1f6a98e927774e0c3e596d94cea7` | 161 |
| `src/facdrone/shells/cppi/lifecycle_seams.py` | `10e5605f51589f89f5a9fbe83c7c3e5838e9a5baea696733244e93ff5ac0bf41` | 97 |
| `src/facdrone/shells/wiring/lifecycle_root.py` | `ffe5ac0b1ead257e781b65683d28160c2d3ad3232e47e008c1b36bfec6304f7f` | 58 |

**Surface**: `results/public-surface.json` `e0b4fdc4f32c3c6fc342afd10045cbaec6f48a8a419dbd4757766d508f7a354c`
(deriver `surface4.py`, 1,754 files scanned, 0 unresolved; the five `lifecycle_group` globals the tests
patch are explicit `patched` names). **Seeds**: `results/seeds.json`
`35626dd378c86fef7d024a597d8386611002f3a8fdda985d7abc56ad782e95bf`: 12 files
(`packages/facdrone-research/tests/test_lifecycle_readings_publish.py`, `tests/unit/test_cppi_monitor_worker.py`,
`test_lifecycle_evidence.py`, `test_lifecycle_evidence_intervals.py`, `test_lifecycle_evidence_projection.py`,
`test_lifecycle_monitor_binding_db.py`, `test_lifecycle_monitor_runner_db.py`, `test_lifecycle_projection.py`,
`test_lifecycle_run_evidence.py`, `test_lifecycle_store_db.py`, `test_lifecycle_window_commits.py`,
`test_monitor_windows.py`), three files dropped with reasons, one deselection
(`tests/unit/test_lifecycle_store_db.py::TestTheTableOnTheDatabase::test_the_event_id_is_the_stored_one_whatever_the_session_time_zone`),
marker expression `not port_baseline and not live_broker`; detection `results/testinfra.json`
`fcefcbf16e5c61513d18b9cefb33634e034b88a3ce3593fc1034bed654baf62c`.

**Acceptance corpus** (`.runs/e4-lifecycle/golden-a.json`, sidecar `results/golden-e4.json.sha256`
`d390c0ee…015c`): identity (LF body, what `record()` hashes) `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584`
(5,479,067 bytes); file as stored (CRLF) raw sha256 `62791b3269b843586300f394ce53c88193269ac0d509f0eb0443a5140c805bd3`
(5,643,360 bytes); 50 sequences (26 seed, 24 generated, Hypothesis seed 20261106), 317 operations,
24-operation alphabet (`admit, admit_durable, admitted, baseline_view, census_references, cli_enroll,
cli_readings, cli_retire, enroll, events, flag, ledger, live_evidence, open_flags, overdue_flags, readings,
retire, run, seams, seed_evidence, standing, standing_view, tick, transition`); two recordings byte-identical;
replay on the frozen original Level 1 none, Level 2 0; round 0 on the identical copy exit 0 (75.7 s).

**Apparatus** (sha256, physical lines; frozen E3/E1b/5.2 files are never edited, new behaviour goes into
new files):

| File | sha256 | Lines | Role |
| --- | --- | ---: | --- |
| `e4-lifecycle/e4_oracle.py` | `a6d0b899bdb4506c399e36725296ec261d505e006a76a679240e4606c9bdbd1d` | 534 | recorder and replayer (after the 5.2 review fix) |
| `e4-lifecycle/run_e4.py` | `3443be46423b52b47d89074c702df1a79a37a1dd74b9f66862cbf0b22fdd577b` | 77 | acceptance arm |
| `e4-lifecycle/surface4.py` | `26aa763aced681d61a436e4766abb0ec994d0092cb8edcb9854bc8367e4a81e1` | 60 | surface deriver; its `SET` is the seven files |
| `e4-lifecycle/test_e4_replay.py` | `7e3fabb383afa54d2a50c4d691bec7a84f6031c8b0b7c999956cad16d68001a1` | 32 | replay target inside the arm |
| `e4-lifecycle/measures4.py` | `29b0623d28f54fa6ac3a4c487814637fe17d8b6f9bd3e6ddd02110129c384be0` | 44 | this task: E3 measurers rebound to the seven files |
| `e4-lifecycle/e4_sample.py` | `107b4ff0ecbc7bbebbe30298b606e9da256b074a7d64556757708260b4fa41e7` | 53 | this task: the original's mutant sample |
| `e4-lifecycle/cosmic-ray-e4.toml` | `ab8c00a4004ce8209c720b64777336aada1afda2fa878631b79bb2e6a1fb46fe` | 22 | this task: mutation configuration, seven modules |
| `e4-lifecycle/results/request-e4-turn1.json` | `c2a98af36e24beec8a90bfad6b238ae235735f5eabda2f1fd42cebd828968396` | 38 | this task: the turn-1 regenerator request (feedback `null`) |
| `e4-lifecycle/reviewer-brief.md` | `2c8eeb263de9079df1292169ce00bfc37e74147b8322c3675fcd2b0c2e53db91` | 86 | this task: the 6.3 brief |
| `e4-lifecycle/mutants/M1.patch` … `M4.patch` | `89699566…035a3`, `01ee894d…6f181`, `7dceff53…d57eb`, `b0526f6d…63dea` | 11, 11, 20, 11 | this task: hand-written semantic mutants ([mutants/README.md](mutants/README.md) `457b5aa5…ecbd3`) |
| `regenerator/regen_turn.py` | `038cbad2363cf48382e11a55c779a5f1bc127fbe9781e0e055f764b27a9727ff` | 337 | the regenerator program (task 4, after review) |
| `e3-evidence-regeneration/measures.py` | `e9ff2d65f220d124d57bfb9f24eba3270bd9cb580529a9fc5307799d66d94538` | 54 | as-written measures and structure check |
| `e3-evidence-regeneration/measures2.py` | `4da2966cf5b8e85bd832005278d725e5dcd8694253b9fb35c7a15cd029156209` | 46 | canonical reading (ruff format 0.17.3, `--isolated --line-length 100 --target-version py313`) |
| `e3-evidence-regeneration/measures3.py` | `9dee44b5dd9f20389bd91ff5ca9ecf95960c6f148c3985454b51ced470fd24ce` | 69 | separated-form reading, rule version 1 |
| `e3-evidence-regeneration/qdiff.py` | `ca4ceeb422680587586fa5065c383e95f8f7605b7a7afe9642c951a7d32f9765` | 108 | query differential for Level 2 pairs |
| `e3-evidence-regeneration/t5_drivers.py` | `e5af562ad7036b02c69051f88ed574c36fe81a16a53e55edcdb0aaae5e25bfef` | 149 | E3's verifier harness, the pattern for 6.5 |
| `e3-evidence-regeneration/missing_tests.py` | `14127ae5347fe4f37ce5dbf42a22f688103913650fee00102f2e805254f44655` | 147 | residue report generator (task 2.1) |
| `e3b-original-sensitivity/t31_arm.py` | `a47d6c255cf6915e75d0e13876bb3831b7cffaf451032e68b1d43b029bc456d2` | 259 | sampling rule, hunks, restore-and-verify, classification (imported by `e4_sample.py`) |
| `e1b-facdrone-evidence-publication/sample_mutants.py` | `b1843245854238d01173b890c973bb8eb3226dc6cf1993a8252923addb31d75a` | 74 | `specs()` reader of a cosmic-ray session |
| `e1b-facdrone-evidence-publication/run_arm.py` | `22ff75bcce002c05db6d5dd1473445ec0d94bb3ad0aa5217dbee4b56660cdf41` | 221 | pytest group runner the arm uses |
| `e1b-facdrone-evidence-publication/e1b_oracle.py` | `c4c93118a98ae07b1174b42bc0b6a9b297048e6c993993c21797a746c988f074` | 406 | `Norm`, `canon`, `first_diff`, `embedded_engine` |
| `t51-integration/editscope.py`, `pureshare.py` | `4c1317c6…ccb5`, `6572808f…c705` | 65, 37 | edit-scope hash check; pure share by statements |
| `guard.py`, `receipt_atomic.py`, `adaptive.py` | `ae8de275…4d61`, `defdc5d9…8bbd`, `bdc93ddb…1b3a` | 56, 153, 126 | host guard, atomic receipt, adaptive deadlines |
| `adaptive-reference.json` | `9724f3eb6f18e69d430357cb8849b97d7416b88345f71335416375cb59e3601f` | 12 | idle reference 0.212 s |
| `testinfra/testinfra.py` | `1cbb55e2169c48ce0d9c137be781c4b41c98cca938552087d380bff9c0f9057c` | 626 | detection (not rerun in E4) |

Tools: cosmic-ray 8.7.0 through `uv tool run --python 3.13 --from cosmic-ray==8.7.0` (uv 0.13.5); ruff
0.17.3 through `uv tool run --offline`; CrossHair 0.0.112 and z3 from `.runs/e1c-facdrone/extras`;
Hypothesis and cosmic-ray extras from `.runs/e1b-facdrone/extras`; the anthropic SDK 2.6.0 from
`.runs/regen-extras`; the model endpoint from `~/.lesscode/model.env` (style, base URL, model name; the
key only through `Endpoint.api_key()`, never recorded). Task 4's smoke used `claude-opus-5-5`; the model
sent and returned is recorded per turn.

**Regenerator request** (turn 1, fixed): `results/request-e4-turn1.json`. Dry prompt built today without a
call (receipt `e4-dry-prompt`, texts under `.runs/e4-lifecycle/dry-prompt/`, not committed because
`user.txt` holds the specimen's source): system sha256 `5fab7bf26d199531772ac32cbb53eb592bb5ee156d6ac9664aac423ef596ae41`
(1,730 chars, byte-identical to E3's), user sha256 `0ffacb8b20d2a33d0151635273e49c2d80aa3d31133210abcae345e95a2228c9`
(90,590 chars, 2,268 lines: the seven files, the surface JSON, definitions, targets, contract, acceptance
command; no feedback section). Turn parameters: `--max-tokens 128000 --effort medium --thinking-display
summarized` (the parameters that produced a candidate in task 4; `high` effort spent the whole 64k budget
thinking). The fixed system text mentions "a codec harness"; the acceptance command in the request states
that this scope has none. Counted input is checked by the program before each call (cap 120,000 tokens; the
E3 prompt of 87,447 chars counted 36,115 tokens, so this one is expected near 38,000).

## Baseline measures of the original scope (computed today, `measures4.py`, receipt `measures4-baseline`; a second run `measures4-baseline-2` gave text-identical files, requirement 4.6)

| File | Lines as written | Lines canonical | Statements as written = canonical | Statements separated (rule 1) | Functions | Write sites |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `lifecycle_monitor_runner.py` | 397 | 397 | 189 | 208 | 14 | 3 |
| `lifecycle_store.py` | 289 | 289 | 142 | 163 | 17 | 8 |
| `lifecycle_group.py` | 278 | 278 | 158 | 173 | 9 | 0 |
| `lifecycle_evidence.py` | 215 | 215 | 114 | 127 | 11 | 1 |
| `lifecycle_projection.py` | 161 | 161 | 61 | 66 | 9 | 0 |
| `lifecycle_seams.py` | 97 | 97 | 56 | 65 | 5 | 0 |
| `lifecycle_root.py` | 58 | 58 | 39 | 42 | 4 | 0 |
| **total** | **1,495** | **1,495** | **759** | **844** | **69** | **12** |

Intra-set call edges 89 (distinct caller-function to callee-function pairs, static, by name), module hops
12. The original is unchanged by the formatter (as E3's was). Separated-form extras: from-import 75, tuple
assignment 7, chained comparison 3, others 0. Write sites by enclosing function: `MonitorRunner._recorded`
(1), `MonitorRunner._append` (2), `LifecycleStore.events`, `.admitted`, `.open_flags`, `.readings` (1 each),
`LifecycleStore._write` (4), `_consumed` (1); of these only `_append` (the reading insert) and `_write` (the
event insert) write, the rest are `SELECT` executes. Result files: `results/measures4-baseline.json`
`bec3e6bfcbfa2b5d4bd4c81742a91a6518e178c1e9c27387c5fb2ebb3a551018`, `-canonical.json`
`67aef1ab4bc67f05533961252f4915456712ac2cf4f41f9b990bb004af658edb`, `-separated.json`
`42124ddab2345c68ebdee8b3bcb69364f6e07ca186164f7873c1b2bcd613bc08`. The name-based edge rule counts
collisions (`standing`, `for_window`, `monitored`) as edges on both trees alike (rationale, section 5);
no correction is applied.

## Declared residue (unprotected by the acceptance; owner condition of 5.2)

`lifecycle_group.lifecycle_admit`, `lifecycle_group.lifecycle_review` (reads the wall clock; excluded from
the alphabet), `lifecycle_root.pod_lifecycle_standing`, `lifecycle_root.lifecycle_store`: reached by no seed
test on this host and by no oracle operation. `lifecycle_root.study_baseline` is reached only through
patched names in the arm (the oracle and the runner test patch `_cited_baseline`), so a mutant inside it
is expected to survive for the same reason. Also unprotected: the behaviour behind the one deselected node
(event id under a non-UTC session time zone), the skipped operator-tier test, a breaching window through the
real retained-evidence join, threaded races.

## Method

Every heavy step: `guard.py <label> --receipt results/receipt.json` first (max wait 1,800 s, `cpu_other`
0.35, RAM floor 8 GB, RAM minimum 6 GB), then `receipt_atomic.py results/receipt.json run <label> --cwd …
--env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1 --env PYTHONWARNINGS=ignore
--timeout <deadline + 120>`, with the deadline from `adaptive.scaled(base)` whose probe is written as the
note `<label>-deadline` (base, factor, scaled); a timeout at factor above 1.5 waits for idle and reruns once
as `<label>-rerun`. Light steps use `--no-probe`.

### 6.2 Regeneration and acceptance (at most 4 turns)

1. Turn k (k = 1..4): `regen_turn.py run --request results/request-e4-turn<k>.json --cand
   .runs/e4-lifecycle/cand-turn<k> --receipt results/receipt.json --label e4-turn<k> --max-tokens 128000
   --effort medium --thinking-display summarized`, cwd `regenerator/`. The program copies the frozen tree to
   a fresh root (one root per turn, never reused: the program refuses a root whose scope files already
   differ), verifies the seven files, records prompt, request-body, response and model hashes (fact
   `e4-turn<k>-inputs`, notes `e4-turn<k>-turn`, `-candidate` or `-failed`), and writes only paths inside
   the scope. The turn texts land under `regenerator/results/turns/e4-turn<k>/` (sidecars committed, the
   texts not). No wall-clock cap on the turn; a transport failure or a `stop_reason` other than `end_turn`
   is a failed turn that counts against the four (requirement 11.3: it stays in the receipt).
2. After each turn, in this order, each receipted: (a) edit-scope check `editscope.py <frozen> <cand>
   --scope <the seven> --dirs <whole tree> --mentions <result.json> <response.json>` → `results/editscope-e4-t<k>.json`;
   any added, removed or changed file outside the seven voids the run (kill criterion 1). (b) The
   structure declaration extracted from `result.json` → `results/structure-e4-t<k>.json`; `measures4.py
   --root <cand> --structure … --fmt-dir .runs/e4-lifecycle/fmt-t<k> --label e4-t<k>` (as written, canonical,
   separated, structure check) and `pureshare.py` → `results/measures4-e4-t<k>*.json`, `results/pureshare-e4-t<k>.json`.
   (c) Acceptance arm: `run_e4.py <cand> --round e4-t<k> --deadline <scaled 1,200 s>` → exit, the first
   Level 1 divergence (`<sequence> op <i> <path>`) or the first failing test node, and
   `results/sql-divergences-rounde4-t<k>.json` (fresh, 50 sequences, else the turn failed, requirement 3.3).
   (d) If `level2_ops` > 0: every distinct Level 2 pair goes to the query differential (`qdiff.py` as E3
   Amendment 1.3 and 1.4: boundary row, at least 100 loaded perturbed states, seed 20261106; if the E4
   divergence file's shape needs a reader shim, `qdiff4.py` is written and counted); a pair fails unless
   every state agrees. The receipt states whether a Level 2 divergence occurred and each pair's verdict.
   (e) Patching sentinel check (edited 2026-11-06 after review, before go; the testable form of kill
   criterion 2): `patchcheck.py` (light, no database, at most 60 lines, counted) runs on both trees,
   importing each module from the tree under test, and asserts two tiers. Tier 1, every name: for the
   nine `patched` surface names (`_cited_baseline`, `_lifecycle_store`, `_pod_of`, `configured_pods`,
   `read_parameters` on `lifecycle_group`; `_consumed`, `baseline_view`, `live_evidence` on
   `lifecycle_evidence`; `project_root` on `lifecycle_root`) plus `lifecycle_root.study_baseline`,
   `lifecycle_store.LifecycleStore` and `lifecycle_evidence.LiveEvidence.for_window`, the name is a key of
   `vars(module)` (a module-level binding, not reachable only through another module), and a static AST
   pass over the module shows no function body that re-imports or re-binds the same name locally or takes
   it as a parameter default (which would capture an import-time copy); for the instance attributes,
   `MonitorRunner(engine_stub, DECLARED, sources_stub)` exposes `_rules`, `_append`, `_evidence`,
   `_monitored`, `_engine` and `LifecycleStore(engine_stub)` exposes `_engine` and `standing`. Tier 2,
   sentinel lookups where a store-free trigger exists: the name is replaced on the module by a callable
   that raises `PatchSentinel`, the trigger is invoked, and `PatchSentinel` must propagate (proving the
   module's own code looked the name up at call time): `_consumed` through `read_evidence(None, lambda d:
   {}, (window,), 1)`; `_lifecycle_store`, `read_parameters`, `_cited_baseline`, `_pod_of`,
   `configured_pods`, `baseline_view`, `live_evidence` through `CliRunner().invoke(app, ["lifecycle",
   "readings", "carry@3", "--window", "2026-09-05"])`, checked in the function's lookup order with the
   earlier names stubbed by minimal fakes (a stub store whose `readings()` returns one row for that window
   at the declared rules revision); `LiveEvidence.for_window` through `live_evidence(None, …)` with
   `_consumed` stubbed to `[]`; `_monitored` and `_evidence` through `MonitorRunner.run(AS_OF, "d")`
   (both are called outside the per-signal `try`). Names without a store-free trigger (`LifecycleStore`,
   `study_baseline`, `project_root`, `_append`, `_rules`, `_engine`, `standing`) are covered by tier 1 and
   by the seed tests and oracle operations that patch them (`cli_readings`, `seams`, `rules: old`,
   `append: twice`). The check is run on the frozen original first (it must pass there, else the check is
   defective and is fixed before any candidate is judged); a failure on a candidate voids that candidate.
   Result file `results/patchcheck-e4-t<k>.json`.
3. A turn is **accepted** when (a) passes, (c) exits 0 with a fresh complete replay file and (d) passes
   for every pair (or there are none). Measures and the structure check are read on the accepted turn as
   T2 to T4 and S; they are not acceptance conditions. 6.2 ends at the first accepted turn or after four
   turns.
4. Turn rule (requirement 6.7), enforced by the program's request schema: after a failed turn, the next
   request is turn 1's request plus `feedback = {"kind": "failed", "first_divergence": <the first Level 1
   divergence, or the first failing test node, or the Level 2 pair that failed its differential, as one
   text>, "measure_gaps": <per measure: candidate value, target, gap>}` and nothing else from the turn
   (no change log, no review). A turn that failed for a transport or stop reason without a candidate gives
   the next turn no feedback section at all (there is no divergence to report; the request is turn 1's).
   The `accepted_scored` kind (surviving faults and realised counterexamples) cannot occur inside 6.2,
   because sensitivity scoring (6.4) follows acceptance; it is listed here as unexercised unless the
   owner orders a further turn after 6.4, which would be a dated amendment.
5. Round 0 is not rerun: the identical copy passed on 2026-11-06 (`round0-ident`). If the frozen copy or
   the identical copy ever diverges from the corpus in E4, that is an apparatus defect: stop (kill
   criterion 7).

### 6.3 Independent review

A fresh agent (a sonnet reviewer, as the task 4 and 5.x reviews) receives exactly what
[reviewer-brief.md](reviewer-brief.md) lists (plan, diff, both trees, measures, receipt) and not the
regenerator's change log or thinking; verifies hashes, surface signatures, patched names and measures
independently; states hypotheses H1..Hn (error and malformed-input paths first), probes each on both
trees and reports agreement; reports formatting effects; writes `review.md` with every probe and a
carried-forward list. Every hypothesis is carried by name into 6.4 and 6.5 (requirement 7.5).

### 6.4 Sensitivity on both trees

- Original: mutable copy `.runs/e4-lifecycle/orig-mut` (`copytree(symlinks=False)` of the frozen copy,
  seven files hash-verified). The sample is drawn: `results/e4-orig-mutant-sample.json`
  `cf9863639fc25f67b2278c75a321f16e93413184bc7d3b6cdb3ba88d598dd5a6`, 30 of the 1,468 specs that
  `cosmic-ray init cosmic-ray-e4.toml .runs/e4-lifecycle/e4-orig.sqlite` (session `4c58b923…274d80`, cwd
  the frozen copy, which it left unchanged) enumerated over the seven modules (population per module:
  `lifecycle_group` 343, `lifecycle_evidence` 287, `lifecycle_monitor_runner` 281, `lifecycle_store` 231,
  `lifecycle_projection` 150, `lifecycle_root` 102, `lifecycle_seams` 74); draw rule
  `sorted(random.Random(20261106).sample(sorted(keys), 30))` (E3's rule, `t31_arm.draw`), a second draw
  byte-identical. Sample per module: group 11, evidence 8, store 4, root 4, runner 1, projection 1, seams
  1. Operators: NumberReplacer 4, AddNot 3, ReplaceComparisonOperator 7 (Eq_LtE 2, Eq_GtE, Eq_Is, GtE_Eq,
  LtE_IsNot, NotEq_GtE), ReplaceBinaryOperator_BitOr 8 (FloorDiv 2, RShift 2, Add, BitAnd, LShift, Pow),
  ReplaceBinaryOperator_Div 5 (Add, FloorDiv, LShift, Mod, RShift), Pow_LShift 1, ReplaceTrueWithFalse 1,
  USub_Invert 1. Composition, read today from the hunks and recorded as information, not as a change to
  any prediction (edited 2026-11-06 after review, before go; receipt note `sample-composition-correction`):
  1 draw is a syntax error (#24, `{<<asdict(...)`, incompetent), so 29 are competent under the R22
  convention; of those, 8 change only type annotations (#11, 13, 14, 20, 21, 22, 23, 27; under
  `from __future__ import annotations`, equivalent), 4 sit in the declared residue (#5, 6, 8 in
  `lifecycle_review`, #28 in root `lifecycle_store`) and 2 in `study_baseline` (#26, 29), which the arm
  reaches only through patched names; these 14 are expected survivors, so at most 15 of 29 can be killed:
  the ceiling of the original's rate on this sample is **0.517**, and **0.483** (14 of 29) if #19
  (`stream_results=True` to `False` in `_consumed_query`, a fetch strategy with the same rows) is
  equivalent too. It is therefore stated in advance that "0.70 not met" on the original's sample is
  expected from the sample's composition and is not a result about the arm's sensitivity; the post hoc
  reading that excludes annotation-only and residue-sited mutants is reported beside it, labelled. T5
  stays on the candidate's own population and sample as the report states it; no target changes.
- Candidate: the same `init` on the accepted candidate root (`cosmic-ray-e4-cand.toml`, written in 6.4 with
  the candidate's module list, which must be the same seven paths), its own population, the same seed and
  draw rule (`e4_sample.py SESSION --root <cand> --out results/e4-cand-mutant-sample.json`), mutable copy
  `.runs/e4-lifecycle/cand-mut`. Comparability is stated as requirement 8.6 asks: same seed, rule, arm,
  classification and restore-and-verify, different populations over different code.
- Arm per mutant (both trees, serial): `e4_arm.py` (written in 6.4 from `t31_arm.py`'s pattern, roots and
  the seven files parameterised): apply the hunks, run `run_e4.py <mut> --round e4-m<k>` with deadline
  `scaled(max(120, 5 × unmutated arm seconds))` where the unmutated arm time is measured once per tree
  (`e4-base-orig`, `e4-base-cand`, base 1,200 s scaled); classify: exit 0 survived; exit 2 with
  `INVALID scope` incompetent (also a `SyntaxError` at import, reported as such); exit 1 from the arm
  killed, with the first failing node or divergence; timeout at factor ≤ 1.5 killed, at factor > 1.5 rerun
  once when idle; replay file not fresh is an apparatus fault (exit 3), not a classification. Restore by
  copying the reference file back and verify the seven hashes before and after every mutant (notes
  `-pre`, `-restore`), as 8.5 requires.
- Hand-written mutants M1 to M4 (`mutants/`, predictions in `mutants/README.md`) on the original as
  written, and ported to the candidate where the construct moved (the port is a patch file
  `mutants/M<k>-cand.patch` with the reason recorded; an unportable construct is reported as not run).
  Each runs the full arm with the base-1,200 s deadline and is reported caught or missed with the catching
  evidence (requirement 8.4). Apply and restore preserve the files' CRLF line endings byte for byte
  (`t31_arm.apply_hunks` splits on `\n` and keeps `\r`; `git apply` keeps the patch's CRLF content
  lines; the post-apply sha256 must equal the recorded mutated hash and the restore the frozen hash)
  (edited 2026-11-06 after review, before go).
- Reporting: killed, survived, incompetent and the rate for both trees; the 0.70 bar (the E3 T4 bar,
  R22/R25 convention; requirement 8.2 defines the rate, not the bar) unchanged and stated met or not met
  for each tree (edited 2026-11-06 after review, before go); every reviewer hypothesis naming a fault class
  addressed by name in the survivor classification (`results/e4-survivor-readings.json`).

### 6.5 Differential behind the four gates

At most **3 pairs** of pure functions (same signature on both trees), each tied to a named reviewer
hypothesis and chosen from the original's pure core: candidates are `_rules_revision`, `_flag_reason`,
`_reading_id`, `baseline_view`, `_daily`, `_lookback`, `_observed_day`, `LiveEvidence.for_window` (with a
scripted readings callable), `monitor_label`, `labeled_reason`, `_latest_basis`, `_baseline`, `_reading`,
`_latest_and_not_run`, `_flag`, `_state`, `signal_standing`, `_event_id`, `_window`, `_compare`,
`MonitorRunner._judge` with a stub flag writer. Per pair, before any solver run, the four verifier gates
of E3 Amendment 2.8 (symbolic-ness smoke with more than one path per side; agreement on 300 Hypothesis
inputs, seed 20261106, from the driver signatures; a planted mutant that yields a counterexample and an
equivalent renaming that exhausts; concrete replay of every counterexample), then CrossHair `diffbehavior`
at 300 s base scaled through E1c-2's launcher, verdict `counterexample | inconclusive | exhausted`, never
`inconclusive` read as equivalent. A corpus-check disagreement is reported as a difference found by the
corpus and the pair's solver verdict is marked not gated (9.3). A realisable counterexample is replayed as
a new sequence through `e4_oracle.py` where the alphabet can express it, otherwise reported as
inexpressible (9.6). Domains as E3 Amendment 2.7 (strings from `abc` up to 8, lists up to 3, enumerations
as indexes, datetimes as day offsets from `AS_OF`). A failing control voids 6.5 and leaves 6.2 to 6.4
standing (9.5). Drivers `e5_drivers.py` are counted research code.

### 6.6 Held-out corpus after acceptance

A new corpus version from the frozen original, recorded after acceptance with a seed that did not exist
before: Hypothesis seed **20261107**, 48 generated sequences (twice the acceptance corpus's), the same 26
seed sequences, the same alphabet, normalisations and `record()` (`e4_record2.py`, a shim of at most 20
lines that rebinds `e4_oracle.SEED` and `N_GENERATED` and calls `record`, counted), output
`.runs/e4-lifecycle/held-out-a.json` and `-b.json` with sidecars; the same validation as 5.2: two recordings
byte-identical (raw bytes now equal the hashed body since the 5.2 fix), replay against the frozen original
with zero divergence at both levels. Then `e4_oracle.py replay --golden <held-out>` in driver mode with
`E4_SOURCE_ROOT=<accepted candidate root>`: the first Level 1 divergence or none, and the Level 2 count,
receipted. T7 is read on this replay; the 48 generated sequences are the sequences the regenerator could
not have been shaped by (the seed sequences repeat the acceptance corpus's by construction and are
reported separately).

### 6.7 Residue report, research entry, index

`missing_tests.py` (task 2.1) over the 6.4 outcomes and the 6.5 replay records (with a reader shim for the
E4 file names if needed, counted) → `results/missing-tests-e4.{md,json}`; the residue declaration of
requirement 10.1 (unreached functions, surviving mutants not judged equivalent, unresolved differential
targets, inexpressible differences, input classes the corpus never carried); the research entry
`docs/research/2026-11-0X/NN-e4-lifecycle-loop.md` reporting T1 to T7 and S each as held or failed with the
numbers, the held-out row, the two kill rates and the protection statement in the form of 10.2 only; the
research index, the spec's research links and the handover updated in the same change.

## Predictions, fixed before execution (one scope; the report's section 8 bands read on it)

| Id | Prediction (report section 8) | Absolute bar on this scope | Held when | Failed when |
| --- | --- | --- | --- | --- |
| T1 | rounds to acceptance: 1 on two of three scopes, at most 3 on the third | acceptance within **≤ 3 turns** | a turn k ≤ 3 is accepted | four turns exhausted without acceptance, or every turn void (kill criteria); an acceptance exactly at turn 4 is "not held, not falsified", reported as such (edited 2026-11-06 after review, before go) |
| T2 | lines after canonical formatting of both trees: 10 to 20 % fewer | original canonical 1,495; band **1,196 to 1,345**; bar **≤ 1,345** | candidate canonical lines ≤ 1,345 (a figure below 1,196 is beyond the band and still held, reported as such) | candidate canonical lines ≥ 1,346 (under 10 % fewer) |
| T3 | functions: 20 to 30 % fewer | original 69; band **49 to 55**; falsifier under 15 % fewer, i.e. **≥ 59** | candidate functions ≤ 55 (fewer than 49 is beyond the band, held) | candidate functions ≥ 59; 56 to 58 is reported as "below the band, above the falsifier": not held, not falsified |
| T4 | intra-scope call edges: 25 to 35 % fewer | original 89; band **58 to 66**; falsifier under 20 % fewer, i.e. **≥ 72** | candidate edges ≤ 66 (fewer than 58 is beyond the band, held) | candidate edges ≥ 72; 67 to 71 is "below the band, above the falsifier": not held, not falsified |
| T5 | kill rate of the recording on the candidate: 0.60 to 0.75 | R22 convention on the candidate's 30-draw sample (seed 20261106); **band 0.60 to 0.75**; the original's rate on its own sample reported beside it; the 0.70 bar (the E3 T4 bar, R22/R25 convention; edited 2026-11-06 after review, before go) stays the protection bar and is stated met or not met for both trees | 0.60 ≤ rate ≤ 0.75 | rate < 0.55, or rate > 0.85 with no change in method; 0.55 to 0.59 and 0.76 to 0.85 are "outside the band, not falsified" |
| T6 | differences invisible to the recording: at least one per scope, on an error or malformed-input path | at least one disagreement on both trees found by the review's probes (6.3), by the 300 random inputs of a pair's corpus gate, or by a realised solver counterexample (6.5), on an error or malformed-input path, that the acceptance replay did not report | at least one such difference, concretely replayed | none found by all three together (6.5 void counts as "not found by the differential") |
| T7 | divergence on a corpus recorded after acceptance: zero | held-out corpus of 6.6 (seed 20261107, 26 + 48 sequences) replayed on the accepted candidate | zero Level 1 divergence on every sequence (Level 2 pairs, if any, passing the query differential) | any Level 1 divergence, or a Level 2 pair failing the differential |
| S | structural contract (requirements 5.1 to 5.5) | write sites ≤ **12** and every one inside a declared shell function; pure share by function count ≥ **0.50**; every function declared, every pure check passing; the patched names remain module globals looked up at call time: `_consumed`, `baseline_view`, `live_evidence` on `lifecycle_evidence`, `study_baseline`, `project_root` on `lifecycle_root`, `_lifecycle_store`, `read_parameters`, `_cited_baseline`, `_pod_of`, `configured_pods` on `lifecycle_group`, `LifecycleStore` on `lifecycle_store`, `LiveEvidence.for_window`; the instance attributes `_rules`, `_append`, `_evidence`, `_monitored`, `_engine`, `standing` keep their names and roles | all of it on the accepted candidate | any part; the pure share by statements inside defs is reported beside it without a bar |

T1 and T7 decide trust (the report: a divergence on later sequences means the alphabet is too weak and is
to be strengthened before anything is tuned). T2 to T4 are read on the accepted candidate only; if no turn
is accepted they are reported as not read. Statements (as written, canonical, separated) are reported on
both trees without a target, because section 8 predicts none.

## Kill criteria (the receipt keeps everything; the reason is reported; headings edited 2026-11-06 after review, before go)

### Criteria that void a candidate (the turn is void, counts against the four; the next request is turn 1's without a feedback section, since a void turn has no divergence to report)

1. A candidate that adds, removes or changes any file outside the seven scope files, any test, the
   recorded corpus or any apparatus file (hash check over the whole tree, 6.2 step 2a).
2. Patching, the lazy-import risk the owner carried from the rationale, in its two forms: (a) a seed test
   in the 12 files fails on the candidate for a patching reason (an `AttributeError` on a patch target, a
   `monkeypatch.setattr` whose target is missing): this is a failed turn with the node as the first
   divergence, not a void; (b) the silent form, a test passing on the candidate although a patched name
   moved so that the patch no longer reaches the code: defined and tested by the sentinel check of 6.2
   step 2e (`patchcheck.py`, both tiers, both trees); a candidate that fails it is void, whatever the arm
   said.

### Criteria that void a sub-step (the other steps stand)

3. The acceptance replay file not fresh or not covering every sequence on a candidate (requirement 3.3):
   the turn failed (feedback: "replay incomplete", no divergence).
4. A verifier control of 6.5 fails: 6.5 is void, reported as void; 6.2 to 6.4 and 6.6 stand.
5. An apparatus file named in this plan edited after the first heavy step (the first `e4-turn1` call)
   without a dated amendment here: the steps that used it are void until rerun under the amendment. New
   apparatus files that 6.4 to 6.6 author (`e4_arm.py`, `cosmic-ray-e4-cand.toml`, `e5_drivers.py`,
   `e4_record2.py`, `patchcheck.py`, the shims) are registered by a receipt fact with their hashes and line
   counts before their first run, not by amendment (edited 2026-11-06 after review, before go).
6. A held-out recording (6.6) that is not byte-identical on re-record after the declared normalisations:
   6.6 is void (apparatus defect), T7 is reported as not read; 6.2 to 6.5 stand.

### Criteria that stop the run

7. Level 1 divergence on the frozen copy or the identical copy at any point, or a replay file not fresh on
   them: stop, apparatus defect, nothing after it is read.
8. Four turns exhausted without acceptance, or every turn void: T1 failed; 6.2 ends; 6.3 to 6.6 have no
   candidate and are reported as not run; the residue report and the research entry are still written
   (a negative result is a result).

## Budgets

- Adaptive deadlines (requirement 11.4; the factor from a receipted probe, bounds [1.0, 4.0]): acceptance
  arm base 1,200 s scaled (round 0 took 75.7 s); mutant arm base max(120, 5 × the unmutated arm's seconds
  on that tree) scaled; recording base 600 s scaled; replay base 600 s scaled; solver 300 s base per pair
  scaled, 15 s per smoke condition (60 s where a path takes longer, as E3); query differential base 600 s
  scaled. A timeout at factor above 1.5 reruns once when idle and the first run is not counted. No fixed
  wall-clock cap on a regenerator turn; the turn's budget is 128,000 output tokens, 120,000 counted input
  tokens, at most **4 turns**, and the transport cut observed in task 4 (silent stream over about five
  minutes) is handled by keeping the thinking display on, not by a cap.
- Model budget: at most 4 turns at the task-4 parameters (one turn there used 36,557 input and 84,992
  output tokens in 668 s); the receipt holds usage per turn.
- Sensitivity: 30 mutants per tree plus 4 hand-written per tree (8 full acceptance runs); at about 76 s
  idle per arm, under two hours idle for both trees.
- Differential: at most 3 pairs.
- Research-code line budget (requirement 11.6): **900 physical lines** for every apparatus file authored
  in 6.1, 6.4, 6.5 and the 6.6 recorder shim, tests excluded, counted and reported, overruns reported and
  never compacted. Spent in 6.1: `measures4.py` 44 + `e4_sample.py` 53 = **97** of the 140 allowed for
  6.1. Justification from the record: E3's apparatus took 545 lines against a budget of 400 (surface,
  measures, run_e3, qdiff, t4_arm, t5_drivers, receipt3, step3: the overrun was 145, in the T4 arm and the
  T5 drivers); E3b's original-side arm `t31_arm.py` took 259 (a port of the T4 arm with roots
  parameterised); 5.2's apparatus took 671 against 600. E4 needs: the 6.4 arm for two trees and seven
  files, of t31_arm's size (about 260), its summary (t32_summary was 82), the candidate-side sampler reuse
  (0: `e4_sample.py` takes `--root`), the 6.5 drivers (t5_drivers was 149) and possibly a qdiff and a
  missing-tests reader shim (about 60 each), the 6.6 shim (20), the patching sentinel check `patchcheck.py`
  (at most 60; edited 2026-11-06 after review, before go): 97 + 260 + 82 + 149 + 120 + 20 + 60 = 788, with
  112 for what the record says always appears (E3 +36 %, 5.2 +12 %). An overrun is a reported fact.

## Not claimed

No adoption: an accepted candidate is a research result on one scope, not a patch for Facdrone. No
equivalence: the protection level is the acceptance corpus (50 sequences, 317 operations) plus the
measured sensitivity on both trees plus the differential outcome within its bounds, with the residue
declared above and in 6.7, and nothing stronger (requirement 10.2). No determinism of the regenerator (the
endpoint exposes no seed). No generalisation beyond this scope: section 8's medians over three scopes
need two more scopes; E4 reports one row of each. The regenerator's change log is the model's assertion
and is checked only through the arm, the review and 6.4 to 6.6. Nothing about Rust, TypeScript or any
other language.

## Owner go

Owner go for 6.2 to 6.7: **given 2026-11-06** (menu answer "Go for 6.2 to 6.7", after the cost and time were stated: up to 4 regenerator turns at about 37k input / 85k output tokens each on claude-opus-5-5, sensitivity on both trees about 2 hours of host time). The plan is frozen from this line on; later changes are dated amendments.

## Amendment 1 (2026-11-06, after the 6.3 review)

The apparatus table above lists `reviewer-brief.md` at `2c8eeb26…` (86 lines), which is the version before the 6.1 review edits; the brief handed to the 6.3 reviewer is `ce953945…` (89 lines), the hash recorded in receipt fact `task-6.1-apparatus-2`. No target, prediction or procedure changes. Found by the 6.3 reviewer.
