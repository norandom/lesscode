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

- [x] 1.7 Repair the shared optional-evidence provenance contract
  - Implement the designed optional-artifact envelope using existing metric and runtime records, with source/plan identities, timestamp and generated versus producer-supplied origin.
  - Preserve existing record shapes and reject malformed envelopes, duplicate evidence identities and unsupported versions through the shared strict readers.
  - Done when populated envelopes round-trip without losing producer/population/runtime details and invalid provenance is rejected; this repairs the foundation omission found before 3.4.
  - _Boundary: Shared contracts_
  - _Depends: 1.3, 1.4_
  - _Requirements: 3.6, 4.6, 4.7, 8.4_

- [x] 1.8 Repair durable plan resource and tool identities
  - Define the designed PreparedPlan wrapper, resolved resource identities and explicit available/unavailable tool bindings without changing existing Plan/Profile record shapes.
  - Validate paths, fingerprints, versions, duplicate identities, declared resource/tool coverage and availability invariants through shared strict readers.
  - Done when a serialized plan retains the context required for reloading and identifying resource changes, including partial identity and diagnostics for missing tools.
  - _Boundary: Shared contracts_
  - _Depends: 1.3_
  - _Requirements: 1.1, 5.1, 5.2, 5.3, 5.7, 8.7_

- [x] 2. Capture inputs and execute bounded checks
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

- [x] 2.5 Normalize native and independent check outcomes
  - Read fresh normalized or supported JUnit results, retain declared check identities, and bind evidence to the current source and plan.
  - Reject dangerous XML declarations, stale/missing reports and duplicate IDs; retain failed, skipped, missing and errored results without allowing successful output to override nonzero exit.
  - Done when native and independent fixture checks produce accurate evidence including deletion/skipping and mismatch cases.
  - _Boundary: Execution_
  - _Depends: 2.4_
  - _Requirements: 4.1, 4.4, 4.5, 6.5_

- [x] 3. Measure structural and optional evidence
- [x] 3.1 Produce deterministic size metrics and input diagnostics
  - Validate Python syntax and stage only measured product paths without inheriting project analyzer configuration.
  - Calculate byte, defined physical-line and deterministic compressed-size values with runtime/definition provenance.
  - Done when repeated fixtures produce identical metrics and malformed syntax or unavailable inputs yield explicit unavailable evidence.
  - _Boundary: Metrics_
  - _Depends: 1.2, 2.2, 2.4_
  - _Requirements: 2.3, 3.3, 3.4, 3.5, 3.7_

- [x] 3.2 Measure per-file and per-function complexity
  - Run the pinned complexity worker through bounded execution and normalize file/function records, including valid files with no functions.
  - Produce distribution/hotspot values and detect the pinned tool's complete source-suppression controls.
  - Done when real fixture locations/counts match known outcomes and suppressed or malformed measurements cannot become complete evidence.
  - _Boundary: Metrics_
  - _Depends: 3.1_
  - _Requirements: 3.1, 3.4, 3.5, 3.7_

- [x] 3.3 Measure duplication with complete source reconciliation
  - Invoke the pinned duplication CLI with external configuration, normalized paths, explicit limits and fresh JSON output.
  - Use verified aggregate pair-span statistics rather than the zero per-clone token field; reconcile scanned paths and zero-token inputs against staged source.
  - Done when real duplicate fixtures report the documented denominator, while silent omissions, suppression markers, absent reports and invalid limits produce unavailable evidence.
  - _Boundary: Metrics_
  - _Depends: 3.2_
  - _Requirements: 3.2, 3.4, 3.5, 3.7_

- [x] 3.4 (P) Normalize supplied optional measurements
  - Consume coverage, mutation, architecture, nesting and runtime evidence through the shared typed contracts without adding measurement engines.
  - Retain populations, numerators/denominators, explanation and provenance; distinguish newly generated evidence from producer-supplied artifacts.
  - Done when stale/foreign evidence is rejected and population changes remain visible for comparison; this task touches Execution tests/code, not pending Metrics files.
  - _Boundary: Execution_
  - _Depends: 1.3, 1.7, 2.5_
  - _Requirements: 3.6, 4.6, 4.7, 8.4_

- [x] 3.5 Align the duplication adapter with the public metric identifier
  - Return the mandatory public key duplication for available and unavailable duplication evidence, retaining the producer, aggregate pair-span definition and unchanged typed payload.
  - Done when actual analyzer output matches the Profile required metric and Decision lookup without a downstream alias or fabricated scalar projection.
  - _Boundary: Metrics_
  - _Depends: 3.3, 4.3_
  - _Requirements: 3.2, 3.4, 3.7, 5.4, 6.6_

- [x] 4. Freeze plans and enforce comparison rules
- [x] 4.1 Prepare immutable executable evaluation plans
  - Resolve supplied profile paths and executables, record source/evaluator/tool identities, and validate required checks/metrics, thresholds and positive limits.
  - Reject overlapping output/source/evaluator roots; prepare command previews without running project checks or installing dependencies.
  - Done when the pilot plan is reproducible, tool absence is recorded explicitly, and changed evaluator resources are identified by integrity comparison.
  - _Boundary: CLI and Plan_
  - _Depends: 1.3, 1.6, 1.8, 2.2, 2.5_
  - _Requirements: 1.1, 1.5, 5.1, 5.2, 5.3, 5.7, 8.7_

- [x] 4.2 (P) Compare metric definitions and calculate honest deltas
  - Compare scope, producer version, definitions and population evidence before calculating changes.
  - Apply declared absolute/percentage rules, preserve non-comparability and require recorded population explanations where specified.
  - Done when zero baseline, unavailable inputs and mismatched definitions produce the designed results; Decision files can be tested independently of plan preparation.
  - _Boundary: Decision_
  - _Depends: 1.3_
  - _Requirements: 2.4, 4.6, 4.7, 6.2, 6.3, 6.4, 6.6_

- [x] 4.3 Apply behavior-first eligibility and acceptance
  - Implement the corrected precedence and retain all reasons: any failed, skipped, missing, errored or unfinished baseline behavior check makes the trial ineligible, regardless of its required flag.
  - Reject observed candidate failures and breached rules, distinguish missing mandatory evidence, and keep optional metric absence nonblocking unless required.
  - Done when table-driven cases cover every decision state and a shorter failing candidate cannot outrank behavior evidence.
  - _Boundary: Decision_
  - _Depends: 4.2_
  - _Requirements: 4.2, 4.3, 4.5, 4.8, 5.3, 5.4, 5.5, 5.6, 5.7_

- [x] 4.4 Isolate fixed version-probe environments
  - Prevent inherited runtime startup configuration from executing undeclared code during plan preparation; retain only explicitly needed OS runtime values.
  - Done when a real Node preload regression no longer executes during the jscpd version probe, parent environment remains unchanged and pinned tool discovery still succeeds.
  - _Boundary: CLI and Plan_
  - _Depends: 4.1, 6.2_
  - _Requirements: 5.1, 5.2, 6.5, 8.7_

- [x] 5. Persist results and account for controlled experiments
- [x] 5.1 Persist recoverable run artifacts
  - Persist prepared/running state and raw artifact references, then finalize reports atomically with artifact identities.
  - Prevent overwrite/path escape, retain partial diagnostics after interruption and expose surviving artifacts on persistence failure.
  - Done when interrupted or failed writes never appear as a finished accepted run and reruns use distinct identities.
  - _Boundary: Reports_
  - _Depends: 1.3_
  - _Requirements: 6.1, 6.5, 6.7_

- [x] 5.2 Render truthful comparison summaries
  - Render the versioned machine report and concise human summary with decision reasons, limitations, separate metrics and protection level.
  - Show product/test changes, elapsed evaluation time, missing evidence and the distinction between passing checks and equivalence.
  - Done when all decision fixtures produce mutually consistent persisted and human-readable outcomes without a compensating quality score.
  - _Boundary: Reports_
  - _Depends: 5.1, 4.3_
  - _Requirements: 4.8, 5.6, 5.7, 6.1, 6.6, 6.7, 8.1_

- [x] 5.3 (P) Reconcile planned trials and controlled conditions
  - Import producer records and pair by corpus/task/repetition, validating the recorded starting state and intended intervention fields.
  - Reject duplicate identities, materialize missing planned trials, separate extra records and flag confounded pairings.
  - Done when synthetic multi-arm inputs retain every scheduled attempt and identify each unintended control difference; no Reports file changes are needed.
  - _Boundary: Experiments_
  - _Depends: 1.4, 1.6_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.8_

- [x] 5.4 Summarize outcomes, variation and observed cost
  - Calculate the designed eligibility, correctness and acceptance counts, including rejected/incomplete attempts and budget-exhausted records.
  - Separate all-attempt results from accepted-only structural statistics; report descriptive variation and sample counts with undefined small-sample values explicit.
  - Done when hand-calculated fixtures match denominators/statistics and missing tokens/cost remain unavailable without being inferred from size.
  - _Boundary: Experiments_
  - _Depends: 5.3, 4.3_
  - _Requirements: 7.4, 7.5, 7.6, 7.7, 8.1, 8.2, 8.3, 8.4_

- [x] 6. Integrate the command-line workflows
- [x] 6.1 Integrate the complete pair-evaluation lifecycle
  - Connect prepared plans, capture, frozen measurement, sequential baseline/candidate checks, before/after integrity verification, decisions and persistence.
  - Connect explicit optional-evidence inputs to the existing provenance reader without changing Profile or replacing core measurements; retain raw artifacts and typed populations/runtime identities.
  - Retain partial observations and independent check inventory; preserve original source and never retry or revert candidate code automatically.
  - Done when an unchanged pair completes and a tampered run persists invalid status with all available evidence.
  - _Boundary: Evaluation integration_
  - _Depends: 2.5, 3.3, 3.4, 4.1, 4.3, 5.2_
  - _Requirements: 1.1, 1.5, 3.6, 4.1, 4.2, 4.3, 4.6, 4.7, 5.2, 5.3, 6.5, 6.7, 8.4, 8.5, 8.8_

- [x] 6.2 Expose plan and run commands with stable exit behavior
  - Connect installed/module entry points to preparation and evaluation, printing resolved commands before execution and honoring the designed exit mapping.
  - Ship executable usage examples with the CLI reference as part of command integration, covering local installation and result interpretation.
  - Done when clean installed invocations return the expected outputs/codes for accepted, rejected, invalid, ineligible, incomplete and persistence-error fixtures.
  - _Boundary: CLI and Plan integration_
  - _Depends: 6.1_
  - _Requirements: 1.1, 1.3, 1.4, 6.1, 6.7, 8.7_

- [x] 6.3 Integrate experiment summaries into the CLI and report artifacts
  - Connect trial import and aggregation to the installed summary command and atomic machine/human artifacts.
  - Apply the designed aggregate exit policy while preserving separate per-state counts, confounding and observed telemetry.
  - Done when a recorded multi-arm pilot is summarized end to end without invoking an LLM or modifying any source input.
  - _Boundary: Experiments and Reports integration_
  - _Depends: 5.4, 6.2_
  - _Requirements: 6.1, 6.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 8.2, 8.3, 8.4_

- [ ] 7. Validate the integrated evaluator across supported environments
- [x] 7.1 Verify reproducibility and behavior-preservation outcomes
  - Run real installed analyzers and the licensed/synthetic corpus through the full CLI, checking known results and repeat structural equality.
  - Align the pilot corpus content_hash with its canonical scoped Snapshot identity, keeping raw upstream file hashes and license provenance separate and unchanged.
  - Verify added/removed/untracked inputs and unchanged original-source fingerprints across accepted and shorter-but-broken candidates.
  - Done when the reproducible pilot produces inspectable success/rejection evidence with no dependence on model access or source upload.
  - _Boundary: Evaluation integration_
  - _Depends: 6.3_
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 2.1, 2.2, 2.3, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.3, 4.4, 6.1, 6.5, 7.2, 7.8, 8.5, 8.8_

- [ ] 7.2 Verify incomplete evidence, tampering and recovery paths
  - Exercise missing analyzers, suppressed/omitted source, parse errors, deleted checks, optional baseline behavior failures and optional metric absence.
  - Inject evaluator changes, stale reports, interrupted writes and bounded child-spawning workloads, retaining observable diagnostics.
  - Repair discovered baseline Execution/Plan audit gaps: retain analyzer command/outcome metadata and effective configuration, and reject declared analyzer options that the fixed adapters cannot honor instead of silently ignoring them.
  - Done when each failure yields the specified invalid/ineligible/incomplete/rejected state, optional baseline checks cannot bypass eligibility, and descendants terminate.
  - _Boundary: Evaluation integration_
  - _Depends: 7.1_
  - _Requirements: 2.4, 2.6, 3.4, 3.7, 4.2, 4.5, 5.1, 5.3, 5.5, 5.7, 6.5, 6.7, 8.6_

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


- 2.5: JUnit metadata/outcome payloads cannot contain executable suite/case nodes; regression prevents manufactured passing cases. Native suite107run6POSIXskip; independent APPROVED.


- 3.1: stage_product provides verified product-only temporary snapshots; measure_sizes produces source_bytes, physical_lines and compressed_source_bytes with explicit runtime/definition provenance. Complete measure facade waits for3.2/3.3.


- 3.2: Pinned Lizard worker validates syntax and file/function inventories; detects suppression controls through pinnedreader normalization. Boltons23functions min1max15median1p954 independently verified. Qualified names retained; leaf names reconcile AST identities.


- 3.3: Pinned jscpd4.0.5 uses pair-span aggregate denominator; never perclone.tokens0. Missing fresh JSON (incl empty-only scopes) or meaningful short-file omission stays unavailable. Windows Node requires SystemRoot; no project env inherited. Native133tests6skip independently reviewed.


- 1.7 corrective foundation: OptionalEvidenceArtifact binds existing metrics/runtime payloads to source/plan/timestamp/origin without changing nested recordshapes. Strictsharedreader tested/reviewed;3.4 can now consume it. Queue30executabletasks after this omitted foundation contract was restored.


- 3.4: read_optional_evidence retains validated OptionalEvidenceArtifact/raw bytes and typed Failures. Generated artifacts require time/source/plan/scope/origin checks; imported provenance stays producerclaims. Populationcomparison belongs4.2.


- 1.8 corrective foundation: PreparedPlan/ResourceIdentity/ToolBinding preserve reloadable profile/resource/tool state while Plan/Profile serialization stays unchanged. Available tools require exact pins; unavailable partial identities keep diagnostics.166tests6skip independentlyverified. Queue31executables.


- 4.1: prepare returns PreparedPlan; verify_integrity reconstructs resolved commands from frozen profile/resources/bindings without probes, detecting argv/env/cwd/limits/executable edits. Fingerprints include resolved paths. Task6.2 must package/resolve lock resources for installed-wheel operation; config currently uses repository_ROOT for locks/entrypoints defaults.


- 4.2: compare_metric(s) returns MetricComparison; use status for compatibility, not delta.status. Structured metrics have no inferredscalarprojection. evaluate_threshold returns passed/failed/unavailable; percent rules unitpercent, others exactmetricunit. Populationchanges need candidateexplanation; rawrecords preserved.


- 4.3: compare returns authoritative Comparison with precedence/allreasons. Allbaselinebehaviornonpasses incloptional are ineligible. NormalizedCheckEvidence is authoritative when executionsabsent; suppliedunsuccessful executions overridepassingrecords. Structuredrequiredmeasurements use MetricComparison.status.


- 5.1: RunStore lifecycle persists manifests/rawrefs and publishes hashedindex last; recover treats invalid/missingindex as incomplete. persist needs explicitartifact_root for referencedbytes. Planhash supplied byintegration (canonicalSHA256ofUTF8dumpsPreparedPlan). Summaryrendering stays5.2.235tests7skips independentlyverified.


- 5.2: render_summary produces escaped human evidence without scoreprojections; finalize hashes summary beforecompletionindex; recover requires indexedsummary. Agenttelemetry remains unavailable unless supplied; evaluationelapsed distinct.243tests7skips independentlyverified.


- 5.3: reconcile returns detached Reconciliation with scheduled records or missing outcomes, extras and corpus/task/repetition pairings. Comparable means declared metadata only. Allowed interventions are exact declared keys/per-arm values; revision-only source anchoring remains explicit limitation.260 tests,7 skips reviewed.


- 5.4: aggregate returns Aggregation(report,runs); explicit scalar deltas only, all-attempt and accepted-only descriptive populations separate. Required completeness differs from failure; unsuccessful executions cannot establish correctness. Verified artifact recovery required. Independent review283tests7skips; focused40PASS.

- 3.5 corrective Metrics alignment: public duplication key matches mandatory Profile/Decision lookup; exact pair-span definition and typed payload unchanged. Real analyzer integration regression passed; independent285tests7skips. Queue32executables.

- 6.1: evaluation.evaluate consumes PreparedPlan, hashes canonical serialization, guards public adapter/check workloads, runs checks in fresh copies and persists partial/final evidence. OptionalEvidenceInput binds imports without changing Profile; runtime envelopes retained separately. Real unchanged accepted/broken rejected with originals preserved; independent301tests7skips.

- 6.2: installed plan/run commands disclose workloads, preserve strict new outputs and map0/1/2/3/4. Package assets retain exact locks; --tool-paths supplies explicit metadata outside checkout. Real installed all decision states and persistence verified; independent308tests7skips. Rebuild wheel after CLI edits.

- 6.3: summarize imports explicit repeatable single-record --trials files and recovers completed run artifacts. Separate experiment storage publishes report/evaluations/summary before final hash index; corruption or missing files recover incomplete. Summary exit0 is report success (including baseline-ineligible-only), distinct from run3. Independent321tests7skips and installed summary smoke passed.

- 4.4 corrective Plan isolation: fixed probes receive empty POSIX environment or only OS-derived SystemRoot on Windows. Actual inherited Node preload regression fixed; parent environment preserved and pinned preparation succeeds. Independent323tests7skips; queue33executables.

- 7.1: executable corpus uses canonical Snapshot identity; upstream raw bytes/license retained. validate_pilot runs six installed CLI evaluations and predeclared actual3trial summary; known23functions/max15 and duplication236/118/.5 verified, originals/untracked inventory retained. Completed --resume is read-only with canonical plan binding; partial pilots require fresh output. Independent332tests7skips and real recovery/tamper probes passed.
