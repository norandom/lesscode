# Integrated readiness evidence: efficiency-baseline

Assessment date: 2026-10-31. This is task 7.4's validation record for the Python evaluator, captured before its independent review. Subsequently, all 33 tasks were completed and independently approved, and the final feature gate returned GO. See the authoritative [final validation](validation.md); capture-time evidence below is retained.

## Revision and scope

The exact native runtime revision is `778e46258dbca42d5b07191a4f011b2cb858fb61`. The captured validation checkout was `f16f3820530e2b1ea2e2752ba9ecb0d03d13c5b6`; `git diff 778e46258dbca42d5b07191a4f011b2cb858fb61 f16f3820530e2b1ea2e2752ba9ecb0d03d13c5b6 --name-only` lists only `.kiro/specs/efficiency-baseline/tasks.md`. Thus packaged/runtime/test/tool/workflow inputs were identical to the native-tested revision. Task 7.4 and final status publication edit documentation only; they change no evaluator source, schema, fixture, profile, lock, gate or tool resource. Existing unrelated untracked authoring scaffolding is preserved.

Task brief: establish fresh full-suite and clean installed plan/run/summarize evidence; reconcile all 56 numbered acceptance criteria, actual report readability, import boundaries and native platform outcomes; retain failures and limits. This is a documentation/validation task. No artificial feature flag or implementation-mirroring test was added, and no fabricated RED test is claimed.

## Fresh final-state Windows suite

Command, run from the checkout with `PYTHONPATH` set to the resolved `src` directory:

```powershell
$env:PYTHONPATH = (Resolve-Path src).Path
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python -m unittest discover -q
```

Result: exit **0**, **363 tests in 1126.475 seconds**, `OK (skipped=11)`. The ten POSIX-only skips match native Windows CI. The additional local skip is `tests.test_report.RunPersistenceTests.test_symlink_escape_is_rejected`: a focused verbose invocation confirmed `WinError 1314`, required symlink privilege unavailable (one test, 0.016 seconds, exit 0 with one explicit skip). The same test executed successfully in native Windows CI; the local skipped probe does not substitute for that evidence.

Retained local evidence root: `C:/Users/mariu/AppData/Local/Temp/lesscode-readiness-74-13b04d57cb3f4143bcf7a5c8cceaa6be`. `tests.log` SHA-256: `fef19e2fc7f2838b070288433e4a486d9061db498b32632d1f299b929f44f5c5`; `tests-exit.txt` records 0; `local-skip.log` records the privilege skip. The log includes intentional `[failed] exit=7` followed by `[later] exit=0` from the CI helper's failure-retention unit test; the final unittest summary passes. No test failure was masked.

The durable [fresh validation record](../../../docs/evidence/readiness-2026-10-31.json) retains this command, decoded output and original output bytes in base64, with the hash above. Base64 preserves exact stdout bytes across Git text-newline conversion; decode `suite.raw_output_base64` to verify `suite.raw_output_sha256`.

This local run was slower than native CI. A read-only `py-spy dump` at about 17 minutes found active Python provenance verification in `test_evaluation.setUpClass → evaluate → verify_integrity → _resource → pathlib.resolve`, with no overdue external child workload. `suite-stack.txt` retains the snapshot. No timeout or performance SLA is inferred from these elapsed times, and no integrity check was bypassed.

## Fresh clean installed smoke

Result: exit **0**, all **11 command stages exit 0**. The owned `clean_smoke.py` driver and `smoke-final/commands.json` in the evidence root retain actual argv, cwd, time budgets, elapsed times and stdout/stderr paths. To reproduce, copy the driver into a fresh owned directory and invoke that copy through `uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe python <fresh-directory>/clean_smoke.py`, retaining its referenced native fixture inputs. The compact validation record retains stage results and wheel identity. The original failed attempt under `smoke/` is separate and preserved: its build exited 2 because the driver incorrectly used the external scratch cwd without project metadata. Correcting only the driver's build cwd to the checkout produced the final success. No product failure or runtime fix is hidden.

| Check | Actual result |
|---|---|
| `uv.exe build --wheel --python .venv-task11/Scripts/python.exe --out-dir <fresh scratch>/wheel` from checkout | Exit 0; 9.804 seconds |
| `uv.exe venv --managed-python --python 3.12.10 <scratch>/installed`, then `uv.exe pip install --python <installed-python> --no-deps <wheel>` | Both exit 0; fresh managed interpreter/environment |
| Installed console and module `--help` / `--version` | Four commands exit 0 outside checkout with `PYTHONPATH` absent |
| Actual installed `plan` using licensed boltons unchanged pair and explicit existing pinned tool metadata | Exit 0; 5.259 seconds; commands and durable PreparedPlan exposed |
| Actual installed `run` with that plan | Exit 0; 110.283 seconds; accepted under frozen profile |
| Actual installed `summarize` with a one-trial schedule and explicit producer record | Exit 0; 1.215 seconds; one accepted scheduled trial, correctness denominator 1; unused arm denominator 0 remains unavailable |
| Installed strict run/experiment recovery, module imports and summary readability | Exit 0; 1.339 seconds; complete/accepted run, complete experiment; ten orchestration modules imported from installed site-packages |

Wheel: `lesscode_baseline-0.1.0-py3-none-any.whl`; SHA-256 `88119823ddefc23639ce71cf685a9e66aaf50636a1e1b6f9cd89cccb14d3a4eb`. Package startup came from `<evidence-root>/installed/Lib/site-packages/lesscode_baseline`, not checkout imports. The existing explicitly provisioned analyzer toolchain was reused and its identities checked by plan preparation; new analyzer installation was already exercised by the exact native CI jobs. This smoke adds one genuine accepted evaluation and one complete report, not a fresh six-case or causal experiment. Its adapted manifest/trial metadata was written before planning/running the new pair.

`original-inputs.json` records exact fingerprints before evaluation; the driver verified identical original boltons and profile-directory bytes afterward. `runs/control-1/{report.json,summary.md,artifacts.json}` and `summary/{report.json,summary.md,evaluations.json,artifacts.json}` are real inspectable outputs. The fresh human summary states schema 1, acceptance under the recorded profile and unverified unchecked behavior. The fresh experiment summary exposes eligible/planned denominators, including the unused arm's zero denominator. Strict recovery verifies all indexed bytes; both human summaries decode as UTF-8 without replacement characters. Boundary AST inspection of every package module found only standard-library absolute imports and local package dependencies; the installed runtime import check covered CLI/config/contracts/decision/evaluation/experiments/metrics/process/report/snapshot.

## Verification result

- Status: **VERIFIED**, claim type **TEST_OR_BUILD**.
- Claim: the current runtime-equivalent checkout passes the fresh canonical suite and builds/starts/evaluates/summarizes/recovers from a fresh installed wheel with the limits above.
- Evidence: actual command exits and retained stdout, strict artifact recovery, source-preservation fingerprints, 56-criterion coverage audit, 80 exact notice hashes, import inspection and native evidence reconciliation.
- Gaps: local symlink-privilege skip is disclosed and covered by actual native Windows CI. Task 7.4 independent review and final feature-level validation are still pending; no **TASK complete** or **FEATURE_GO** claim is made here.

## Native platform evidence

Authority: [workflow run 41198054658](https://github.com/norandom/lesscode/actions/runs/41198054658), exact SHA above. Independent final task 7.3 review approved these actual results. The durable [machine-readable native audit](../../../docs/evidence/native-41198054658.json) retains measured results and archive identities.

| Native host | Canonical suite | Host-inapplicable skips | Suite seconds | Stages | Actual descendant timeout |
|---|---|---|---:|---|---|
| Ubuntu 24.04 | 363 tests, PASS | 14 Windows-only | 281.481 | 17/17 exit 0 | One test executed, PASS |
| Windows 2025 | 363 tests, PASS | 10 POSIX-only | 498.079 | 17/17 exit 0 | One test executed, PASS |
| macOS 15 Intel | 363 tests, PASS | 14 Windows-only | 578.398 | 17/17 exit 0 | One test executed, PASS |

Each job provisioned CPython 3.12.10, uv 0.13.5, Node 24.22.1, Lizard 1.17.31 and jscpd 4.0.5; genuine probes and wheel startup ran. Each clean installed pilot completed six actual evaluations (four accepted, two rejected), a three-trial experiment (two accepted, one rejected), and read-only completed recovery. Across the jobs, the audit inspected 18 runs, three experiments, 1,344 indexed artifacts, 180 typed analyzer execution records and 36 fixed jscpd configs. All 51 command stages exited 0; individual pilot rejection/summary subprocesses intentionally exited 1 and were checked as such by the pilot driver.

Known values: the licensed boltons fixture has 23 functions and max CCN 15; the duplicate fixture has 236 total tokens, 118 duplicated pair-span tokens and ratio 0.5. Within-host repeated control evidence matches after excluding raw artifact paths. Generated synthetic text uses native newlines: Windows candidate 147 bytes versus Unix 141 bytes. These have distinct recorded input identities; unequal generated inputs do not support a cross-host identical-metric claim.

| Archive | Artifact ID | SHA-256 |
|---|---:|---|
| [Ubuntu](https://github.com/norandom/lesscode/actions/runs/41198054658/artifacts/13042135141) | 13042135141 | `2b315d090b3ae3639512acb598b9888f49dfcdddeaaf89c54dcdd0afd25d033b` |
| [Windows](https://github.com/norandom/lesscode/actions/runs/41198054658/artifacts/13041692411) | 13041692411 | `dde639ccce6857be7f796896bb1b52ab8a5c87e0311f3ddd43ba54d15adde879` |
| [macOS](https://github.com/norandom/lesscode/actions/runs/41198054658/artifacts/13041423607) | 13041423607 | `3cd500904a44b6fbe8fd07cce5db7d4f7be00dfb344393120f2ff9390ea0e07e` |

Raw artifacts have 14-day GitHub retention and may expire. A compact audit summary records the observed evidence; it cannot replace raw bytes for a later re-audit. Download and verify archives while available, or rerun `.github/workflows/baseline-platforms.yml` at the desired exact revision. [CI documentation](../../../docs/ci-validation.md) describes provisioning, artifact contents and failure retention. Earlier failed runs remain historical diagnostic evidence and are not counted as final platform passes.

## Inspecting genuine reports

Inside each archive, inspect `validation/commands.json`, `validation/result.json`, `validation/runtime.json`, `validation/pilot/validation.json`, and `validation/pilot/commands.json`. Then inspect `validation/pilot/runs/control-1/{plan.json,report.json,summary.md,artifacts.json}`, the `candidate-1` rejection, and `validation/pilot/summary/{report.json,summary.md,evaluations.json,artifacts.json}`. Native `report.recover` and `recover_experiment` audits verified index size/hash bindings and strict typed reports, rather than merely trusting successful process exits.

The actual human run summary identifies schema version, decision, check diagnostics, executions, metric producer definitions, local protection, retained artifacts and the declared-check limitation. The actual experiment summary identifies corpus/license/controls, scheduled outcomes, eligible correctness denominators, acceptance denominators, exhausted budgets and separate structural populations. Producer text is escaped and can appear as encoded punctuation; it remains data. No summary claims universal equivalence.

## Coverage, integration and limitations

[Requirement coverage](requirement-coverage.md) maps all 56 original criteria with exact numbering. Every criterion has implementation and executable evidence for the declared Python scope; optional coverage/mutation/nesting/architecture/runtime and telemetry contracts are tested using typed supplied evidence. No external mutation/coverage engine, model generation, provider spend measurement or causal skill experiment is asserted by the fixture pilot.

Boundary inspection covers all package imports and the actual installed orchestration modules. `contracts.py` remains the typed serialization boundary. Configuration and snapshot modules do not import orchestration; metrics uses the bounded execution service; decisions and report rendering do not launch tools. CLI lazy imports reach concrete configuration/evaluation/aggregation functions. The Windows process helper is imported only by the native execution branch. Absolute imports are standard-library modules; pinned analyzers run as explicit subprocesses. There are no cloud, agent SDK, MCP or rewrite-engine imports. Wheel package data supplies exact locks; no checkout-dependent default lock fallback is required for installed execution.

Concrete design alignment: preparation and evaluation use `PreparedPlan`; canonical plan hashes use its authoritative UTF-8 serialization. Aggregation retains `ExperimentReport` plus typed verified run references in `Aggregation`, preserving public record shapes. Python invocation retains lexical absolute venv context while integrity fingerprints resolved binary/environment resources. These representation clarifications do not relax evaluator integrity, path containment or acceptance contracts.

The implemented CLI reference is `docs/cli.md` and the actual native workflow is `.github/workflows/baseline-platforms.yml`; the design's file table now records these concrete names in place of its earlier proposed names. Their installation/reporting and three-platform validation responsibilities are unchanged.

The evaluator executes declared trusted commands. Disposable copies and local fingerprints preserve normal source workflows and detect change; they do not contain hostile absolute-path writes, privileges or network activity. Adversarial protection needs externally controlled isolation. Declared checks are finite evidence, not semantic equivalence. Scalar deltas follow their recorded definition; structured metrics do not receive invented scalar scores. Missing evidence remains unavailable or incomplete. Summary command exit 0 records successful reporting, including baseline-ineligible-only outcomes; pair `run` exit 0 means accepted under that profile. These are distinct claims.

All 80 hash-named license notices retain original distribution bytes, including CRLF. Whitespace validation uses `git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check`; normalizing upstream bytes to satisfy a different whitespace interpretation would invalidate their identities.
