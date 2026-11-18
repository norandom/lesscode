# R150: Of six specs that claim completion, one has claims no code supports today, three name Python that is retired, and three tests report a pass while skipping

- Report date/time and timezone: 2026-11-18, 13:35, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 13:14 to 13:24 (from the clock and the test output)
- Status: completed
- Spec: [deconvolution-loop](../../../.kiro/specs/deconvolution-loop/requirements.md), [deconvolution-native-core](../../../.kiro/specs/deconvolution-native-core/requirements.md), [capability-compression-core](../../../.kiro/specs/capability-compression-core/requirements.md), [efficiency-baseline](../../../.kiro/specs/efficiency-baseline/requirements.md), [dogfood-self-application](../../../.kiro/specs/dogfood-self-application/requirements.md), [refinement-evidence-integrity](../../../.kiro/specs/refinement-evidence-integrity/requirements.md); task 1.4 of the [corrective tasks](../../../.kiro/corrective-tasks.md)
- Hypothesis ID and predecessor: completes the sweep that [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md) began; uses [R148](14-adapter-specs-audit.md) and [R149](15-e3-e4-figure-audit.md)
- Type: retrospective audit with reruns; no model call

## Decision

For each validation claim three questions were asked: is there a command, did it run on real inputs,
and can it run today.

| Spec | Claim | Command | Real inputs | Runs today |
| --- | --- | --- | --- | --- |
| `deconvolution-loop` | E4 accepted at turn 3 with the recorded figures | `run_e4.py`, `measures4.py` and the others of R149 | yes | **yes**, reproduces (R149) |
| `deconvolution-loop` | The recorded three turns replay to the recorded transitions | `regenerator` test `e4_three_turn_replay_reproduces_recorded_transitions` | yes, the frozen E4 turns and tree | **yes**, 18 s |
| `deconvolution-loop` | The Python regenerator program and its 59 tests | `pytest tests/test_regenerator.py` | yes | **no**, retired in `4f0aed4` |
| `deconvolution-native-core` | Size budgets hold | `lesscode-core budgets` | yes, `native/src` | **yes**: 72 files, 65 within budget, 7 with a kept ceiling, 0 exceeded |
| `deconvolution-native-core` | The historical bindings give 21 / 11 / 5 / 1 / 4 and exit 1 | `lesscode-core admission` | yes | **yes**, same counts, exit 1 |
| `deconvolution-native-core` | E3 and E4 recordings reproduce the frozen digests; frozen traces replay without a difference | `recording` tests, three of them | yes, when written | **no, and the tests say PASS.** They return early because `src/lesscode_baseline/recording.py` is gone |
| `deconvolution-native-core` | Nine adverse controls fail closed | `s10_controls2.py`, `tests/test_recording.py` | yes | **no**, retired in `4f0aed4`; the kept result files remain |
| `capability-compression-core` | Native suite on Windows, Linux and macOS (102, 103, 103 tests) | Actions run 41333477507 | yes | **not rerun**; the audit file is kept, the raw artifacts have expired; today's local suite passes |
| `capability-compression-core` | `prepare` and `compare` work | `cli` and `workflow` tests | yes | **yes**, 8 and 12 pass |
| `capability-compression-core` | Python suites, the adopted self-application (29 statements removed) | `unittest`, paired regressions | yes | **no**, retired in `4f0aed4` |
| `efficiency-baseline` | 363 tests, installed wheel, three platforms, 1,344 artifacts | `unittest discover`, Actions run 41198054658 | yes | **no**, the evaluator is Python and retired; two receipts are kept |
| `dogfood-self-application` | Requirements 1 to 4 and 6: clone, commands, paths, fixtures, skill | `cli_deconvolute`, `cli`, `admission` tests | yes | **yes**; the commands were rebuilt on the driver since, receipts are now `-v2` |
| `dogfood-self-application` | Requirement 5: isolated target directory for specimen runs | a line in `adapters/rust/mod.rs` | no run was shown | **not supported.** No command reaches that module (R148). 5.3 and 5.4 have no evidence |
| `dogfood-self-application` | Requirement 7.4: `refactor/*` and `deconvolute/*` pushes get extra checks | `cli_hook` tests | yes, when written | **no longer true.** The behaviour was removed; today's test is `refactor_branch_push_behaves_like_normal_branch_push` |
| `dogfood-self-application` | Requirement 7.1 to 7.3: the hook checks budgets | `cli_hook` tests | yes | **yes**, 6 pass |
| `refinement-evidence-integrity` | The nine Rust probes of R39 pass after the fix | `probe.rs` as its own test crate | yes, real child processes | **yes**, 9 of 9 |
| `refinement-evidence-integrity` | The five Python probes and the Python suite pass | `test_reference.py` | yes | **no**, retired; the run of 8 November at `f7c254f` is kept with exit 0 for both |

`dogfood-self-application` has claims that no code supports. Its `spec.json` gets the note and status
of the two specs of R134. The other five keep their phase and get an `audit` block that says what
runs today. Their claims had commands and real inputs when they were made.

## Hypothesis and falsification

R134 found two specs whose validation rested on code that did not do the thing. The sweep asks
whether the remaining completed specs are of the same kind. A claim with no command, or a command
that never saw real input, would say so. Requirement 5 of the dogfood spec is of that kind. The three
skipping tests are a second kind that R134 did not name: a real check that stopped checking when its
input was deleted and still reports success.

## Frozen inputs and method

- Repository at `e6c3f47`; `lesscode-core 2.0.0` rebuilt from this repository at 13:19.
- `cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --test-threads 8 --no-fail-fast`: 401 run, 401 passed.
- `cargo +1.98.1 test --locked --manifest-path native/Cargo.toml --doc`: exit 0, no doctests.
- `lesscode-core budgets --config native/complexity-budgets.json --root native/src`.
- `lesscode-core admission --root . --bindings docs/experiments/deconvolution/native-prototype/s2-bindings.json`.
- Six specimen-dependent tests run alone with `--no-capture`, to read what they print.
- R39's Rust probes: the test crate that `run.ps1` writes, with `R39_PYTHON` set to the specimen interpreter. `run.ps1` itself cannot run, because it hashes Python files that no longer exist.
- Python: `git ls-tree` at the parent of `4f0aed4` lists 113 files under `src` and `tests`; today there are none. The tags `v1.0.0` to `v1.0.2` contain them.

## Evidence and results

`D:\BuildArtifacts\lesscode\corrective-audit\task74\1.4\`: `nextest-full.log`, `doctest.log`,
`budgets.json`, `s2-admission.json`, `skip-check.log`, `r39\native.log`.

The skip check printed, for `e3_double_record_reproduces_frozen_lf_body_digest`,
`e4_double_record_reproduces_frozen_lf_body_digest` and `frozen_traces_replay_without_divergence`:
"skipping: pinned specimen venv, Python baseline package, or frozen proto2 recordings absent", each
followed by PASS. The interpreter, the shim and the recordings are present. The missing file is the
Python package. The three adapter validation reports of 9 November cite these tests as "golden digests
verified".

### A stale build, found and removed during this task

The first full run reported 402 tests, and two `regenerator` tests also skipped although their inputs
exist. The test binaries in the shared target directory had been compiled at 07:59 from the prototype
copy under `D:\BuildArtifacts\lesscode\corrective-audit\foreground-20261118\prototype\native`, and
Cargo took them as fresh. The command-line binary of 10:17 listed `measure-scope`, which only that
copy has. `cargo clean -p lesscode-core` removed 9,469 files (8.5 GiB), and everything was run again
from this repository. The numbers above are from the second run. The first run's output is kept under
`stale-build\`.

Effect on R148: its six analysis receipts and three density receipts were regenerated with the
rebuilt binary and are identical in bytes. Its 32 tests pass in the full run. Its findings stand.

## Limits and interpretation

- Whether each of the 56 criteria of `efficiency-baseline` and the 32 of `capability-compression-core`
  was met was not rechecked. The audit asked only whether the validation had commands and real input.
- The Python claims could be replayed from a checkout of `v1.0.2`. That was not done.
- `deconvolution-loop` and `deconvolution-native-core` have unchecked boxes in `tasks.md` (3 and 11)
  while their phase says complete. The boxes were not reconciled here.
- `verification.md` of `refinement-evidence-integrity` still says the fix is not claimed. It predates
  the implementation.
- Evidence written on 18 November before 13:19 with this repository's own test binaries may have used
  the prototype copy's build. The frozen selector binaries of tasks 70 to 73 were not examined.

## Tool/spec consequence and next gate

- New corrective task 3.4: the three `recording` tests must stop reporting a pass. Delete them with the
  Rust recorder that depends on the retired Python package, or mark them ignored with the reason.
- Task 3.3 has two more sentences to correct: the adapter reports' "golden digests verified".
- Task 5.1's rule should cover a skipped check: a test that cannot run is not a pass.
- Copies of the crate need their own target directory. The prototype copy did not have one.

## COBESY check

Each row answers the three questions. "No" under "Runs today" for retired Python is not a finding
against the claim. The one unsupported requirement and the three skipping tests are named separately.
