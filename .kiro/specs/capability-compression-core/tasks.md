# Implementation Plan

Each executable subtask is a 1–3 hour implementation/test unit. Component boundaries
refer to the design's file ownership. No tests below are optional.

Parallel branches 2.1, 3.1 and 4.1 may start after 1.2; each edits only its own
module and local tests. Later work within a module remains sequential. Other
work follows listed order and explicit dependencies. Shared model changes require
coordination rather than parallel edits. Empirical evidence gaps remain open;
completing an attempted trial does not itself establish maturity.

- [x] 1. Establish the native build and shared contracts
- [x] 1.1 Establish a reproducible native package
  - Create one library/CLI package with pinned toolchain and dependencies; verify grammar selection and license obligations.
  - Compile minimal Python/TS/TSX parser probes and establish test execution on Windows/Linux; record prerequisites, without adding runtimes to native analysis.
  - Done: pinned build and parser smoke tests pass; Python source and historical evidence remain unchanged.
  - _Boundary: CLI/library foundation_
  - _Requirements: 2.2, 2.3, 4.4_

- [x] 1.2 Define closed trial and result records
  - Encode versioned manifests, frozen trials, targets, ownership maps, checks, metric availability and results; reject unknown fields and duplicate IDs.
  - Establish module boundaries, checked integer counts and stable serialization; keep command execution and rule expressions outside the model.
  - Done: valid records round-trip and malformed, duplicate or unsupported target fields fail fixtures.
  - _Boundary: Model_
  - _Requirements: 1.1, 2.3, 2.6, 2.7, 3.1, 4.1_

- [ ] 2. Account for complete capability scope
- [x] 2.1 (P) Inventory and fingerprint frozen source
  - Inventory tracked/untracked maintained files with fixed exclusions, categories and normalized identities; reject path escape, symlinks/reparse points and case collisions.
  - Bind supplied capability inventory to the source snapshot; retain generated/tool-state exclusions explicitly.
  - Done: equivalent relocated roots have equal identities; changed content or classification is detected by fixtures.
  - _Boundary: Scope_
  - _Depends: 1.2_
  - _Requirements: 1.1, 2.1, 4.3, 4.4, 4.5_

- [ ] 2.2 Resolve reviewed ownership selections
  - Resolve whole-file and complete-node selectors, reject missing/ambiguous resolutions and union overlapping spans.
  - Retain entrypoints, ownership rationale and unresolved relationship review; prevent omitted declared source from becoming zero.
  - Done: fixtures distinguish reviewed scope from ambiguous, narrowed or omitted ownership.
  - _Boundary: Scope_
  - _Depends: 2.1_
  - _Requirements: 1.1, 1.3, 1.4_

- [ ] 2.3 Include replacement support beyond original files
  - Account symmetrically for changed/new/deleted production files, helpers, tables and callers; retain capability and wider-boundary totals separately.
  - Block unclassified changes and changed excluded maintained source; disclose dependency changes and unreviewed ownership.
  - Done: moved/escaped helpers and split-function cases cannot hide maintained implementation; overlapping spans are counted once.
  - _Boundary: Scope_
  - _Requirements: 1.2, 1.3, 2.1, 2.5, 4.5_

- [ ] 3. Measure supported source structure
- [ ] 3.1 (P) Establish Python metric definitions and measurements
  - Implement versioned physical lines, logical statements, aggregate decisions and function counts using explicit node tables.
  - Cover compound statements, comprehensions, short circuits, nested definitions, Unicode/CRLF and error/unavailable states with independent hand-counted fixtures.
  - Done: fixture counts and definition identities agree; no Radon/Lizard equivalence is implied.
  - _Boundary: Metrics_
  - _Depends: 1.2_
  - _Requirements: 2.2, 2.3, 2.4, 2.6_

- [ ] 3.2 Add independently defined TypeScript and TSX metrics
  - Extend explicit language tables for declarations, wrappers, arrows, branches and handlers without combining language scores.
  - Cover unsupported syntax, parse errors and non-UTF-8 input; retain physical counts when structural measurements are unavailable.
  - Done: hand-reviewed typed fixtures pass with independent definition identities and no double-counted declaration wrappers.
  - _Boundary: Metrics_
  - _Requirements: 2.2, 2.3, 2.4_

- [ ] 3.3 Expose internal call layers and unresolved edges
  - Resolve only unambiguous local calls and directly resolved imports inside declared scope; locate all remaining call relationships.
  - Detect recursion/cycles and report partial lower bounds separately from unavailable full depth.
  - Done: direct, imported, ambiguous and recursive fixtures produce correct resolved/unresolved evidence.
  - _Boundary: Metrics_
  - _Depends: 2.2, 3.2_
  - _Requirements: 1.4, 2.4_

- [ ] 4. Admit evidence and decide comparisons
- [ ] 4.1 (P) Validate supplied behavioral evidence
  - Bind both sides to source/trial/check identities and raw artifacts; validate differential mismatch and expected/observed case counts.
  - Distinguish failed, missing, changed and erroneous checks; expose tested domain and provenance limits.
  - Done: stale, altered and incomplete results cannot pass; collector exit zero cannot conceal mismatches.
  - _Boundary: Compare_
  - _Depends: 1.2_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [ ] 4.2 Apply frozen targets and decision precedence
  - Implement all design statuses, preserving every finding and required metric availability; never compensate behavior failures with size savings.
  - Require positive production statement reduction and declared bounds; no target remains descriptive, unavailable metrics remain unavailable.
  - Done: independent decision-table tests cover all statuses, conflicting findings, zero counts and formatting-only reductions.
  - _Boundary: Compare_
  - _Requirements: 2.3, 2.5, 2.6, 2.7, 2.8, 3.3, 3.4, 4.6_

- [ ] 4.3 Render deterministic comparison evidence
  - Produce JSON and readable Markdown with hypothesis, complete accounting, definitions, outcomes, dependency disclosure and limitations.
  - Sort stable payloads and separate execution metadata; retain failures and incomplete trials.
  - Done: repeated comparison records render identically apart from metadata and never claim universal equivalence or maintenance savings.
  - _Boundary: Compare_
  - _Requirements: 2.1, 3.6, 4.1, 4.2, 4.3, 4.6_

- [ ] 5. Integrate the working command workflow
- [ ] 5.1 Integrate freezing and paired comparison
  - Connect model, inventory, metrics and comparison through the two library operations; preserve the module dependency direction.
  - Recheck sources before returning, join ownership reviews with unresolved edges, and reject changed frozen definitions.
  - Done: prepared trials produce correctly categorized comparisons and detect mid-comparison source changes.
  - _Boundary: CLI/library integration with Scope, Metrics, Compare_
  - _Depends: 2.3, 3.3, 4.3_
  - _Requirements: 1.1, 1.3, 2.1, 3.1, 3.2, 4.3, 4.4_

- [ ] 5.2 Expose and test the native CLI
  - Provide prepare/compare arguments, actionable diagnostics and all specified exit codes; require fresh output outside source roots.
  - Test no-target and missing-evidence reporting, output refusal and unchanged source hashes.
  - Done: end-to-end CLI fixtures exercise every status without changing existing Python commands.
  - _Boundary: CLI/library integration_
  - _Requirements: 2.8, 4.2, 4.4, 4.6_

- [ ] 5.3 Integrate portable correctness checks
  - Add native Windows/Linux/macOS CI using pinned build and meaningful unit/CLI fixtures; check platform path cases.
  - Keep hardware, model downloads and performance benchmarks out; expose missing platform evidence rather than claiming success.
  - Done: the workflow is executable and local available-platform checks pass; actual platform runs are captured during final validation.
  - _Boundary: CLI/library validation infrastructure_
  - _Depends: 5.2_
  - _Requirements: 2.2, 4.3, 4.4_

- [ ] 6. Validate the external empirical workflow
- [ ] 6.1 Prepare reproducible external replay inputs
  - Create new ATS/NPU manifests and narrow research-only conversions without changing originals; verify existing uv environments and fixture prerequisites.
  - Preserve historical metric names, raw artifacts and missing provenance; do not invent digests or reinterpret collector success.
  - Done: replay inputs validate against frozen native records with explicit coverage and unavailable evidence.
  - _Boundary: External trial fixtures_
  - _Depends: 5.3_
  - _Requirements: 1.1, 2.3, 3.2, 4.1, 4.4_

- [ ] 6.2 Replay ATS and its behavioral negative control
  - Run unchanged frozen observations for supplied original/candidate/mutant in isolated copies, then compare using native definitions.
  - Verify full replacement accounting and rejection of the known mutant; report new versus historical counts separately.
  - Done: matching candidate evidence and detected adverse behavior are retained in a reproducible integrated trial.
  - _Boundary: External trial integration_
  - _Requirements: 1.2, 2.2, 3.3, 3.5, 4.1, 4.2_

- [ ] 6.3 Replay NPU and challenge scope/evidence manipulation
  - Exercise the full provider/helper replacement with existing software-only checks and negative control in isolated copies.
  - Add moved helpers, scope narrowing, classification changes, missing checks and stale evidence variants to the integrated workflow fixtures.
  - Done: the typed replacement reports its actual evidence and every manipulation is rejected or incomplete without hardware access.
  - _Boundary: External trial integration_
  - _Requirements: 1.2, 1.3, 2.1, 2.5, 3.2, 3.3, 3.5, 4.4_

- [ ] 6.4 Integrate and verify the skill command recipe
  - Use the recorded ATS replay to exercise the skill recipe end-to-end.
  - Add scope/inventory review and paired-check instructions plus tested native usage.
  - Done: the recipe reproduces the recorded outcome without changing checks or source.
  - _Boundary: Skill/CLI integration_
  - _Depends: 6.2_
  - _Requirements: 4.1, 4.2, 4.5_

- [ ] 6.5 Run a bounded real TypeScript capability trial
  - Select one existing small typed capability with runnable native project tests in the first 30 minutes; freeze one installed-capability inventory.
  - Evaluate one isolated candidate with a maximum two-hour edit/validation budget, no dependency additions and no upstream adoption.
  - If no feasible target is found or evidence cannot finish, retain an explicit incomplete trial and do not claim TS maturity; development continues.
  - Done: reproducible evidence records actual counts and check outcomes or precise outstanding evidence.
  - _Boundary: External trial integration_
  - _Depends: 6.1, 6.4_
  - _Requirements: 2.2, 3.5, 4.1, 4.2, 4.5_

- [ ] 7. Complete bounded self-application and final validation
- [ ] 7.1 Freeze a supported Lesscode self-trial
  - Select one complete supported capability and record rationale, baseline/evaluator revisions, target, scope, exclusions and independent expected outcomes.
  - Prepare isolated baseline/candidate copies and frozen evaluator/checks outside the editable workspace; verify the relevant existing uv test environment.
  - Done: the self-trial inputs are reproducible and do not require Rust parsing, feature cuts or rewriting uncommitted work.
  - _Boundary: Self-trial_
  - _Depends: 6.2, 6.3_
  - _Requirements: 5.1, 5.2, 5.3, 5.8_

- [ ] 7.2 Test self-compression without forcing adoption
  - Evaluate one bounded candidate with unchanged relevant checks and project regression suites on both sides; use independent expected-result/adverse fixtures for evaluator changes.
  - Count the whole replacement and report structural tradeoffs separately from tests and research; retain baseline unless benefit and behavior are demonstrated.
  - Done: complete evidence supports acceptance or justified rejection; incomplete evidence leaves maturity open without stopping development.
  - _Boundary: Self-trial_
  - _Requirements: 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

- [ ] 7.3 Validate and publish scoped readiness evidence
  - Run the integrated native/adverse/regression checks and capture available platform evidence; verify all criteria against concrete artifacts.
  - Record workflow maturity separately from compression outcome and unsupported coverage; include working usage and evidence links in the existing indexes.
  - Done: final validation records passed, rejected and outstanding evidence honestly; no forced reduction or retired layer is needed to finish implementation work.
  - _Boundary: CLI/library and Self-trial validation integration_
  - _Depends: 6.5, 7.2_
  - _Requirements: 4.2, 4.4, 5.1, 5.5, 5.6, 5.7, 5.8_


