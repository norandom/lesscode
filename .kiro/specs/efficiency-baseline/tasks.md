# Implementation Plan

All executable subtasks are scoped to approximately 1–3 hours, including focused tests. Complete foundation tasks before core work. Unmarked tasks follow sequential order; marked tasks may overlap only the stated peers after their explicit prerequisites. Shared contracts are frozen at the end of foundation; changes to them require serial coordination. Tests belong to their component task unless an explicit integration task names the cross-component flow. No validation task is optional.

- [x] 1. Establish the runnable foundation and shared evidence contracts
- [x] 1.1 Create the local package and test entry points
  - Establish or explicitly locate and verify the designed CPython 3.12.10 and Node 24.22.1 runtimes, recording resolved executable paths; create the installable command entry point and standard-library test discovery while excluding generated environments and runs from source measurement.
  - Make help and version commands runnable without an LLM account or analyzer execution.
  - Done when clean local setup verifies both pinned runtimes, installs the package, starts the CLI and discovers a passing initial test suite.
  - _Boundary: CLI and Plan_
  - _Requirements: 1.1, 1.3, 1.4_

- [x] 1.2 Lock and verify the open-source analyzer toolchain
  - Using the verified runtimes from 1.1, install the designed Python and Node analyzer versions in isolated tool environments and record exact transitive locks, integrity hashes and applicable license notices.
  - Verify resolved interpreter/package entry points with small real analyzer probes, including forward-slash Windows inputs; do not add runtime auto-installation.
  - Done when clean dependency installation reproduces the pins and both analyzers emit the expected versioned probe evidence.
  - _Boundary: Metrics_
  - _Depends: 1.1_
  - _Requirements: 1.2, 1.4, 3.4, 3.7_

- [x] 1.3 Validate evaluation plans and evidence records
  - Implement the design's typed profile, plan, snapshot, metric, check, failure and run-result records and JSON boundary validation.
  - Reject unknown fields/versions, non-finite values, duplicate identities and escaping paths; preserve explicit unavailable states and required flags.
  - Done when valid evaluation records round-trip and malformed inputs fail with field-level diagnostics before command execution.
  - _Boundary: Shared contracts_
  - _Depends: 1.1_
  - _Requirements: 2.1, 2.2, 3.4, 3.7, 4.5, 5.1, 6.1, 6.5_

- [x] 1.4 Validate experiment and telemetry records
  - Extend the same shared contracts with corpus, arm, planned-trial, producer-record and telemetry definitions from the design.
  - Keep observed costs, runtime workloads, generation provenance and missing values explicit; freeze the shared contracts before parallel core work.
  - Done when representative multi-arm records round-trip and duplicate IDs, missing provenance fields and invalid telemetry are rejected.
  - _Boundary: Shared contracts_
  - _Requirements: 7.1, 7.2, 7.8, 8.2, 8.3, 8.4_

- [x] 1.5 Provide synthetic fixtures and independent checks
  - Create minimal unchanged, duplicated and shorter-but-broken Python pairs plus independent golden/invariant checks outside candidate source.
  - Include normalized and JUnit evidence examples, source suppression cases, missing outputs and evaluator-tampering inputs.
  - Done when expected outcomes are known independently of the future evaluator and fixture identities are recorded.
  - _Boundary: Validation fixtures_
  - _Depends: 1.3, 1.4_
  - _Requirements: 3.1, 3.2, 3.5, 4.3, 4.4, 4.5, 5.3_

- [x] 1.6 Provide a licensed pilot corpus and explicit evaluation profile
  - Select a small real Python fixture at an immutable revision, retain its permissive license and record source and content identity.
  - Supply runnable native/independent checks, required metrics, tool settings, declared limits and experiment controls using the frozen contracts.
  - Done when the corpus/profile validates and its real-project baseline checks pass with explicit dependency setup.
  - _Boundary: Validation fixtures_
  - _Depends: 1.2, 1.3, 1.4_
  - _Requirements: 1.2, 5.1, 7.1, 7.8_

- [ ] 2. Capture inputs and execute bounded checks
- [x] 2.1 Classify source scope and identify actual content
  - Implement portable scope matching and category precedence over tracked and untracked content, separating product, tests and support files.
  - Record canonical content identities and file changes; reject unreadable inputs, case collisions, symlinks and reparse points.
  - Done when fixture manifests show exact expected inclusions, exclusions and changed-file identities on repeat capture.
  - _Boundary: Snapshot_
  - _Depends: 1.3, 1.5_
  - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6_

- [x] 2.2 Create consistent disposable evaluation copies
  - Copy selected content while preserving required mode flags and detecting concurrent source changes.
  - Separate frozen measurement input from writable check copies; constrain cleanup to owned directories and support diagnostic retention.
  - Done when write-producing checks cannot alter original fixtures through ordinary relative writes and copy races fail visibly.
  - _Boundary: Snapshot_
  - _Requirements: 2.1, 2.6, 8.5, 8.8_

- [x] 2.3 (P) Run resolved commands with bounded logs and POSIX lifecycle
  - Execute resolved argument arrays with declared environments and working directories; stream local logs and record monotonic elapsed time.
  - Enforce timeout/log budgets and POSIX descendant termination without hidden retries or shell interpolation.
  - Done when host-independent execution contracts pass, POSIX lifecycle tests are implemented and independently reviewed, and host-inapplicable tests are explicitly reserved for Linux CI in 7.3. Do not require WSL on a Windows development host or claim skipped tests passed; this task owns Execution files only and may overlap Snapshot tasks.
  - _Boundary: Execution_
  - _Depends: 1.3, 1.5_
  - _Requirements: 4.1, 6.5, 6.7, 8.1, 8.6_

- [x] 2.4 Enforce Windows process-tree lifecycle
  - Add the designed suspended launch, kill-on-close job assignment and resume sequence through the private Windows helper.
  - Fail before execution if containment setup fails and retain the common execution evidence contract.
  - Done when Windows timeout and log-limit probes terminate the parent and child while preserving logs; serialize with the prior Execution task.
  - _Boundary: Execution_
  - _Depends: 2.3_
  - _Requirements: 1.4, 8.6_

- [ ] 2.5 Normalize native and independent check outcomes
  - Read fresh normalized or supported JUnit results, retain declared check identities, and bind evidence to the current source and plan.
  - Reject dangerous XML declarations, stale/missing reports and duplicate IDs; retain failed, skipped, missing and errored results without allowing successful output to override nonzero exit.
  - Done when native and independent fixture checks produce accurate evidence including deletion/skipping and mismatch cases.
  - _Boundary: Execution_
  - _Depends: 2.4_
  - _Requirements: 4.1, 4.4, 4.5, 6.5_

- [ ] 3. Measure structural and optional evidence
- [ ] 3.1 Produce deterministic size metrics and input diagnostics
  - Validate Python syntax and stage only measured product paths without inheriting project analyzer configuration.
  - Calculate byte, defined physical-line and deterministic compressed-size values with runtime/definition provenance.
  - Done when repeated fixtures produce identical metrics and malformed syntax or unavailable inputs yield explicit unavailable evidence.
  - _Boundary: Metrics_
  - _Depends: 1.2, 2.2, 2.4_
  - _Requirements: 2.3, 3.3, 3.4, 3.5, 3.7_

- [ ] 3.2 Measure per-file and per-function complexity
  - Run the pinned complexity worker through bounded execution and normalize file/function records, including valid files with no functions.
  - Produce distribution/hotspot values and detect the pinned tool's complete source-suppression controls.
  - Done when real fixture locations/counts match known outcomes and suppressed or malformed measurements cannot become complete evidence.
  - _Boundary: Metrics_
  - _Depends: 3.1_
  - _Requirements: 3.1, 3.4, 3.5, 3.7_

- [ ] 3.3 Measure duplication with complete source reconciliation
  - Invoke the pinned duplication CLI with external configuration, normalized paths, explicit limits and fresh JSON output.
  - Use verified aggregate pair-span statistics rather than the zero per-clone token field; reconcile scanned paths and zero-token inputs against staged source.
  - Done when real duplicate fixtures report the documented denominator, while silent omissions, suppression markers, absent reports and invalid limits produce unavailable evidence.
  - _Boundary: Metrics_
  - _Depends: 3.2_
  - _Requirements: 3.2, 3.4, 3.5, 3.7_

- [ ] 3.4 (P) Normalize supplied optional measurements
  - Consume coverage, mutation, architecture, nesting and runtime evidence through the shared typed contracts without adding measurement engines.
  - Retain populations, numerators/denominators, explanation and provenance; distinguish newly generated evidence from producer-supplied artifacts.
  - Done when stale/foreign evidence is rejected and population changes remain visible for comparison; this task touches Execution tests/code, not pending Metrics files.
  - _Boundary: Execution_
  - _Depends: 1.3, 2.5_
  - _Requirements: 3.6, 4.6, 4.7, 8.4_

- [ ] 4. Freeze plans and enforce comparison rules
- [ ] 4.1 Prepare immutable executable evaluation plans
  - Resolve supplied profile paths and executables, record source/evaluator/tool identities, and validate required checks/metrics, thresholds and positive limits.
  - Reject overlapping output/source/evaluator roots; prepare command previews without running project checks or installing dependencies.
  - Done when the pilot plan is reproducible, tool absence is recorded explicitly, and changed evaluator resources are identified by integrity comparison.
  - _Boundary: CLI and Plan_
  - _Depends: 1.3, 1.6, 2.2, 2.5_
  - _Requirements: 1.1, 1.5, 5.1, 5.2, 5.3, 5.7, 8.7_

- [ ] 4.2 (P) Compare metric definitions and calculate honest deltas
  - Compare scope, producer version, definitions and population evidence before calculating changes.
  - Apply declared absolute/percentage rules, preserve non-comparability and require recorded population explanations where specified.
  - Done when zero baseline, unavailable inputs and mismatched definitions produce the designed results; Decision files can be tested independently of plan preparation.
  - _Boundary: Decision_
  - _Depends: 1.3_
  - _Requirements: 2.4, 4.6, 4.7, 6.2, 6.3, 6.4, 6.6_

- [ ] 4.3 Apply behavior-first eligibility and acceptance
  - Implement the corrected precedence and retain all reasons: any failed, skipped, missing, errored or unfinished baseline behavior check makes the trial ineligible, regardless of its required flag.
  - Reject observed candidate failures and breached rules, distinguish missing mandatory evidence, and keep optional metric absence nonblocking unless required.
  - Done when table-driven cases cover every decision state and a shorter failing candidate cannot outrank behavior evidence.
  - _Boundary: Decision_
  - _Depends: 4.2_
  - _Requirements: 4.2, 4.3, 4.5, 4.8, 5.3, 5.4, 5.5, 5.6, 5.7_

- [ ] 5. Persist results and account for controlled experiments
- [ ] 5.1 Persist recoverable run artifacts
  - Persist prepared/running state and raw artifact references, then finalize reports atomically with artifact identities.
  - Prevent overwrite/path escape, retain partial diagnostics after interruption and expose surviving artifacts on persistence failure.
  - Done when interrupted or failed writes never appear as a finished accepted run and reruns use distinct identities.
  - _Boundary: Reports_
  - _Depends: 1.3_
  - _Requirements: 6.1, 6.5, 6.7_

- [ ] 5.2 Render truthful comparison summaries
  - Render the versioned machine report and concise human summary with decision reasons, limitations, separate metrics and protection level.
  - Show product/test changes, elapsed evaluation time, missing evidence and the distinction between passing checks and equivalence.
  - Done when all decision fixtures produce mutually consistent persisted and human-readable outcomes without a compensating quality score.
  - _Boundary: Reports_
  - _Depends: 5.1, 4.3_
  - _Requirements: 4.8, 5.6, 5.7, 6.1, 6.6, 6.7, 8.1_

- [ ] 5.3 (P) Reconcile planned trials and controlled conditions
  - Import producer records and pair by corpus/task/repetition, validating the recorded starting state and intended intervention fields.
  - Reject duplicate identities, materialize missing planned trials, separate extra records and flag confounded pairings.
  - Done when synthetic multi-arm inputs retain every scheduled attempt and identify each unintended control difference; no Reports file changes are needed.
  - _Boundary: Experiments_
  - _Depends: 1.4, 1.6_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.8_

- [ ] 5.4 Summarize outcomes, variation and observed cost
  - Calculate the designed eligibility, correctness and acceptance counts, including rejected/incomplete attempts and budget-exhausted records.
  - Separate all-attempt results from accepted-only structural statistics; report descriptive variation and sample counts with undefined small-sample values explicit.
  - Done when hand-calculated fixtures match denominators/statistics and missing tokens/cost remain unavailable without being inferred from size.
  - _Boundary: Experiments_
  - _Depends: 5.3, 4.3_
  - _Requirements: 7.4, 7.5, 7.6, 7.7, 8.1, 8.2, 8.3, 8.4_

- [ ] 6. Integrate the command-line workflows
- [ ] 6.1 Integrate the complete pair-evaluation lifecycle
  - Connect prepared plans, capture, frozen measurement, sequential baseline/candidate checks, before/after integrity verification, decisions and persistence.
  - Retain partial observations and independent check inventory; preserve original source and never retry or revert candidate code automatically.
  - Done when an unchanged pair completes and a tampered run persists invalid status with all available evidence.
  - _Boundary: Evaluation integration_
  - _Depends: 2.5, 3.3, 3.4, 4.1, 4.3, 5.2_
  - _Requirements: 1.1, 1.5, 4.1, 4.2, 4.3, 5.2, 5.3, 6.5, 6.7, 8.5, 8.8_

- [ ] 6.2 Expose plan and run commands with stable exit behavior
  - Connect installed/module entry points to preparation and evaluation, printing resolved commands before execution and honoring the designed exit mapping.
  - Ship executable usage examples with the CLI reference as part of command integration, covering local installation and result interpretation.
  - Done when clean installed invocations return the expected outputs/codes for accepted, rejected, invalid, ineligible, incomplete and persistence-error fixtures.
  - _Boundary: CLI and Plan integration_
  - _Depends: 6.1_
  - _Requirements: 1.1, 1.3, 1.4, 6.1, 6.7, 8.7_

- [ ] 6.3 Integrate experiment summaries into the CLI and report artifacts
  - Connect trial import and aggregation to the installed summary command and atomic machine/human artifacts.
  - Apply the designed aggregate exit policy while preserving separate per-state counts, confounding and observed telemetry.
  - Done when a recorded multi-arm pilot is summarized end to end without invoking an LLM or modifying any source input.
  - _Boundary: Experiments and Reports integration_
  - _Depends: 5.4, 6.2_
  - _Requirements: 6.1, 6.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 8.2, 8.3, 8.4_

- [ ] 7. Validate the integrated evaluator across supported environments
- [ ] 7.1 Verify reproducibility and behavior-preservation outcomes
  - Run real installed analyzers and the licensed/synthetic corpus through the full CLI, checking known results and repeat structural equality.
  - Verify added/removed/untracked inputs and unchanged original-source fingerprints across accepted and shorter-but-broken candidates.
  - Done when the reproducible pilot produces inspectable success/rejection evidence with no dependence on model access or source upload.
  - _Boundary: Evaluation integration_
  - _Depends: 6.3_
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 2.1, 2.2, 2.3, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.3, 4.4, 6.1, 6.5, 8.5, 8.8_

- [ ] 7.2 Verify incomplete evidence, tampering and recovery paths
  - Exercise missing analyzers, suppressed/omitted source, parse errors, deleted checks, optional baseline behavior failures and optional metric absence.
  - Inject evaluator changes, stale reports, interrupted writes and bounded child-spawning workloads, retaining observable diagnostics.
  - Done when each failure yields the specified invalid/ineligible/incomplete/rejected state, optional baseline checks cannot bypass eligibility, and descendants terminate.
  - _Boundary: Evaluation integration_
  - _Depends: 7.1_
  - _Requirements: 2.4, 2.6, 3.7, 4.2, 4.5, 5.3, 5.5, 5.7, 6.7, 8.6_

- [ ] 7.3 Run supported-platform integration in CI
  - Configure Windows, Linux and macOS jobs using locked runtimes/dependencies and the same fixtures; include package startup and genuine analyzer execution.
  - Verify path handling, original preservation, determinism and platform-specific process-tree timeout behavior; retain job evidence without weakening checks for unsupported environments.
  - Done when all available platform jobs have explicit pass/failure evidence; an unexecuted platform remains an outstanding validation gate.
  - _Boundary: Runtime validation integration_
  - _Depends: 7.2_
  - _Requirements: 1.2, 1.4, 3.5, 6.7, 8.6_

- [ ] 7.4 Complete the integrated readiness check
  - Run the full unit/integration suite and clean-install plan/run/summarize smoke paths against the final state.
  - Reconcile requirement coverage, artifact readability, boundary imports and all platform evidence; preserve unresolved failures rather than declare completion.
  - Done when the packaged evaluator starts, all mandatory checks pass and the real pilot reports can be independently inspected; no skill effectiveness claim is inferred.
  - _Boundary: Evaluation integration_
  - _Depends: 7.3_
  - _Requirements: 1.1, 1.4, 4.8, 5.4, 6.1, 6.7, 7.5_

## Task-Plan Review

- Task-plan review gate: PASS; all 56 acceptance criteria covered, with component contracts, runtime prerequisites, integration and validation represented.
- Task-graph sanity review: PASS, independent mode. The reviewer requested explicit Node runtime setup; the draft was repaired once and passed the permitted re-review.
- Baseline eligibility follows the user-confirmed rule: any failed or incomplete declared baseline behavior check makes the trial ineligible; optional metrics are nonblocking unless required.
- Counts: 7 major groups, 29 executable subtasks, 4 parallel markers. All implementation checkboxes remain open.

## Implementation Notes

- User correction after 2.3: WSL is operational according to the user. Prior tool-call timeouts did not justify a general host outage claim. Continue native Windows development; validate POSIX-specific behavior on Linux CI in 7.3, without making WSL a prerequisite. The former STOP_FOR_HUMAN is superseded by this user-approved validation placement; cross-platform release evidence remains mandatory.

- 2.3 debug round 1: No repository fix can establish missing POSIX execution while WSL launch fails before Python. Docker uses the same WSL backend. Pending process.py/test_process.py are preserved uncommitted; no completion or review approval claimed. Do not reset WSL globally without accounting for existing sessions.

- 1.1: Windows workspace resolves to D:\Source\lesscode; use the verified Python 3.12.10 executable or py -3.12, not the Microsoft Store python alias. Canonical tests: .venv-task11/Scripts/python.exe -m unittest discover -v. Build: py -3.12 -m pip wheel . --no-deps --wheel-dir dist. Smoke: installed lesscode-baseline --help and --version.


- 1.2: Run explicit tool setup with py -3.12 tools/setup_toolchain.py; probes use tools/probe_toolchain.py. Evaluation must never install. jscpd probe uses 20 tokens; production detection remains 50. Linux/macOS execution awaits platform validation.


- 1.3: Source tests require PYTHONPATH set to resolved src; installed-wheel smoke is separate. Shared contracts expose strict from_dict/loads/dumps; required metric keys are complexity, duplication and source_bytes.



- 1.5: Synthetic fixture oracle remains outside candidates; sample evidence bindings are illustrative. Actual execution must freshly bind plan/source identities. Fixtures are LF-pinned through .gitattributes.


- 1.6: Profile-relative trusted runner resources live under benchmarks/profiles/pilot, outside candidates. Plan preparation must resolve their argument before changing cwd. Real boltons module/license bytes are protected by nested Git attributes.


- 2.1: Snapshot | Failure is the concrete typed Result representation. capture_scope identifies source; copy capture follows in 2.2. Supplementary revision can be None; actual source bytes remain authoritative.



- 2.3 resumed: User authorized native-host development with POSIX lifecycle runtime checks deferred to native Linux CI7.3.80 tests run,6POSIX skips disclosed; independent review APPROVED. Persistence OSError propagation regression covered. Windows execution intentionally awaits2.4.


- 2.4: Native Windows JobObject lifecycle verified;94testsrun6POSIXskips. Shared test aliases must tolerate skip decorators returning original functions. Private _winapi use is tied to pinned CPython3.12.10.

