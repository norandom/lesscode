# Implementation Plan: constraint-guided-compression (Slice A production + G1-R bounded spike)

Scope note. Production tasks cover **Slice A only** (the demonstrated apparatus:
surface derivation, scope measures, test-infrastructure detection, and the
single-turn regenerator base with the two-tier reward controller). Slice B
production claims (multi-rollout `Refine`/`BestOfN` adoption, the optional
`dspy` adapter) remain withheld pending the `G1-R` spike per `Req 7.10` and
`prototype-gate.md`; task 6 is that bounded spike, nothing more. Every
language-specific analysis step stays behind the per-language extractor seam
(design.md "Deferred Target Languages", requirements Amendment 2, 2026-11-07):
TypeScript and further target languages are TODO items — adding one is a new
sibling extractor module plus tests, never a rewrite of controllers, reward
ladder, receipts, or regenerator. Tasks marked "owner go" spend model budget
and start only on the owner's explicit answer. Frozen apparatus
(`recording.py`, `native/src/*.rs`, frozen experiment drivers) is never edited.

- [ ] 1. Foundation: register the four evaluator modules in the package
  - Export the surface, measures, testinfra, and regenerator modules from the
    evaluator package (exports only, no behavior change; `cli.py` untouched
    because its subcommands and exits are an evidence contract).
  - Keep the regenerator CLI reachable as a module command with the preserved
    exit codes 0/2/3/4/5/6 and keep top-level imports stdlib-only; lazy
    external tools (`ruff` via `uv`, endpoint SDK, `dspy`) stay inside their
    owning call sites.
  - Done when the module command prints usage with exit 0, all four modules
    import without side effects or a specimen environment, and the existing
    package test collection is unchanged.
  - _Requirements: 2.1_

- [ ] 2. Surface derivation and candidate verification (Contract B)
  - [ ] 2.1 (P) Contract-B surface derivation with versioned extractor
    - Derive the public surface of a scope from explicit scope files plus
      recorder files (Contract B: symbols the recorder oracle imports are
      part of the contract), reporting callable kinds, unresolved targets,
      anchors, and extractor version; no module-global scope state, no
      language knowledge outside this seam.
    - Done when the derived report for the retained E4 material includes the
      recorder-imported symbol and marks every unresolvable target as
      unresolved rather than absent.
    - _Boundary: SurfaceDeriver_
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 7.1_
  - [ ] 2.2 Candidate surface verification for Tier-1 rejection
    - Verify a candidate tree against the surface report: missing symbols,
      kind mismatches, and local rebinding of patched module globals
      (parameter shadowing, local assignment, default-arg capture), returning
      the structured pass/fail result the static gate consumes.
    - Done when each violation class is reported with the offending symbol
      and location, and a clean candidate passes.
    - _Boundary: SurfaceDeriver_
    - _Depends: 2.1_
    - _Requirements: 2.5, 5.7_
  - [ ] 2.3 Surface unit suite (offline)
    - Cover Contract A vs Contract B (recorder-imported symbol case), the
      three candidate-verification violation classes, and the rebind modes.
    - Done when the suite runs green offline on synthetic trees with zero
      specimen dependencies.
    - _Depends: 2.1, 2.2_
    - _Requirements: 7.1_

- [ ] 3. Scope measures: three readings, structure, and edit scope
  - [ ] 3.1 (P) As-written measures with pure/shell structure validation
    - Measure lines, statements, functions, qualified names, write sites,
      call edges, and module hops for a scope, and validate declared
      structure: write sites inside shell, no engine/connection/session
      parameters or SQL attribute calls in pure functions, undeclared
      functions rejected.
    - Done when measures for the retained E3/E4 scope files reproduce the
      frozen readings and a structure violation is reported with its site.
    - _Boundary: ScopeMeasurer_
    - _Requirements: 2.2, 2.3, 3.1, 6.2_
  - [ ] 3.2 Separated-form v1 and canonical reading
    - Implement the nine separated-form statement rules with the exact
      base-plus-extra identity (separated minus extra equals written), and
      the canonical reading through the pinned formatter run in isolation;
      when the formatter is unavailable the canonical reading is reported
      unavailable, never zero.
    - Done when all nine rules hold on the rule table cases and the
      canonical reading is None (not 0) with the formatter unavailable.
    - _Boundary: ScopeMeasurer_
    - _Depends: 3.1_
    - _Requirements: 6.2, 6.3_
  - [ ] 3.3 Composition entry point, edit scope, and measure gaps
    - Provide the one-call composition returning written, separated,
      canonical, and target gaps; enforce edit scope by hash identity
      outside the declared files (dependency changes surface as explicit
      decisions, never silent); gaps label candidates marginal or rejected
      against preregistered targets.
    - Done when one call reproduces the three readings plus gaps for the
      frozen scope, and an out-of-scope edit is refused with the offending
      path.
    - _Boundary: ScopeMeasurer_
    - _Depends: 3.1, 3.2_
    - _Requirements: 1.5, 2.6, 3.6, 6.1, 6.4_
  - [ ] 3.4 Measures unit suite (offline)
    - Cover the written-measure fields, structure validation cases, all nine
      separated rules with the identity, canonical unavailability, edit-scope
      refusal, and gap computation against targets.
    - Done when the suite runs green offline and exercises every rule with
      its own case.
    - _Depends: 3.1, 3.2, 3.3_
    - _Requirements: 7.1_

- [ ] 4. Test-infrastructure detection as a package module
  - [ ] 4.1 (P) Versioned detector relocation and rule stamping
    - Relocate the demonstrated detector into the package as a new versioned
      module (the frozen experiment original stays untouched), reporting
      runner, plugins, marker classes, fixtures, store facts, hazards,
      scope-referencing test files, and detection-rule versions; non-Python
      repositories return an unknown runner with zero seeds cleanly.
    - Done when the relocated module's report on the retained validation
      inputs equals the demonstrated output and rule versions are recorded.
    - _Boundary: TestInfraDetector_
    - _Requirements: 1.3, 3.5, 7.6_
  - [ ] 4.2 Detector unit suite (offline)
    - Cover config discovery across the accepted config files, fixture and
      plugin resolution, marker exclusion rules, store-fact and hazard
      detection, and the clean unknown-runner behavior on a non-Python
      layout.
    - Done when the suite runs green offline on synthetic repositories,
      including one non-Python layout case.
    - _Depends: 4.1_
    - _Requirements: 7.1_

- [ ] 5. Probabilistic regenerator: single-turn base with two-tier reward
  - [ ] 5.1 Turn contracts, endpoints, and provenance-preserving receipts
    - Preserve the demonstrated single-turn contracts (request/result, prompt
      build, candidate preparation and writing, hash set) plus the four
      recorded amendments (Contract-B surface, preregistered parameters,
      LF-byte hashes, case-distinct labels); support both endpoint styles
      with redacted keys everywhere.
    - Done when a recorded turn round-trips through the receipt with all
      five hash fields over raw stored bytes and no key material appears in
      any serialized output.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 2.2_
    - _Requirements: 1.1, 1.2, 3.2, 7.3, 8.5_
  - [ ] 5.2 Tier-1 static gate and adaptive schedule
    - Implement the static ladder (transport/stop, JSON, scope refusal,
      syntax, surface, structure, static targets) with the adaptive schedule
      (thinking-budget recovery, bounded temperature steps, missing-symbol
      injection into feedback) so static failures never launch a runtime
      oracle.
    - Done when every ladder rung reports its score and diagnostics in under
      two seconds on the frozen failure artifacts with zero runtime calls.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.1, 2.2, 3.3_
    - _Requirements: 4.2_
  - [ ] 5.3 Tier-2 runtime gate with pinned partial credit and feedback boundary
    - Wrap the recording core evaluator with the documented return contract
      (verdict, first divergence, fraction keys), apply the pinned
      partial-credit formula with fail-closed missing keys, keep
      infrastructure failures a separate failure kind never read as kill or
      equivalence, and enforce the feedback content boundary: divergence
      identities and measure gaps only, never held-out sequence names.
    - Done when the formula reproduces the documented scores on synthetic
      fractions, a missing fraction key scores as zero, and a payload
      containing a held-out name is rejected before dispatch.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.2_
    - _Requirements: 4.2, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.5_
  - [ ] 5.4 Refine/BestOfN controller: budgets, stops, and outcome separation
    - Implement the nesting (outer turns, BestOfN rollouts per turn, inner
      static retries), the budget and no-progress stops on outer turns, best
      trajectory retention on exhaustion, candidate-tree restoration before
      each write, append-only receipt notes with atomic protocol, resume
      identity re-verification, and the outcome object keeping validity,
      compression, and adoption separate with adoption always false.
    - Done when the simulated four-turn failure-and-recovery sequence reaches
      acceptance, exhaustion retains the best trajectory, a hash mismatch
      aborts with the integrity exit, and outcome fields never conflate.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.3_
    - _Requirements: 1.4, 1.5, 3.4, 4.1, 4.3, 4.4, 4.5, 4.6, 4.7, 6.6, 6.7, 7.4, 7.11_
  - [ ] 5.5 Regenerator unit suite (offline)
    - Cover key redaction and payload formatting, the six adverse controls
      (five frozen plus the feedback-leak amendment) returning sub-gate
      scores with zero evaluator invocations, the simulated multi-turn
      recovery, exhaustion stops, and holdout-disjoint rejection.
    - Done when the suite runs green offline with every control failing
      closed and no runtime oracle call recorded for Tier-1 controls.
    - _Depends: 5.4_
    - _Requirements: 5.8, 7.5_

- [ ] 6. G1-R bounded spike (Slice B gate; owner go for 6.2)
  - [ ] 6.1 Deterministic G1-R verification driver
    - Build the spike that discharges the deterministic checklist on frozen
      E3/E4 artifacts: six Tier-1 controls fail closed without runtime
      invocation, the recorded three-turn transition replay (thinking-budget
      failure, missing recorder symbol, acceptance) passes Tier 1 in under
      two seconds per check, no-progress exhaustion halts with the best
      trajectory, the replaceable-apparatus accounting stays within the
      twenty-percent ceiling against the expanded baseline stratum, and the
      admission binary verifies every emitted receipt and sidecar
      byte-exactly; retain negative results in the gate record.
    - Done when the driver runs the full deterministic checklist end-to-end
      and writes the gate evidence record with every item pass or fail.
    - _Boundary: ProbabilisticRegenerator, SurfaceDeriver, ScopeMeasurer_
    - _Depends: 2.3, 3.4, 4.2, 5.5_
    - _Requirements: 5.8, 7.2, 7.8, 7.9_
  - [ ] 6.2 Live weaker-model recovery run (owner go)
    - Execute at least one live non-deterministic or weaker-model run that
      recovers from a Tier-1 or Tier-2 failure to full acceptance within the
      frozen turn and retry budgets, appending receipts and recording cost.
    - Done when the run's receipts show a recorded failure followed by
      acceptance within budget and the gate record cites it.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 6.1_
    - _Requirements: 7.9_

- [ ] 7. Final verification and evidence records
  - [ ] 7.1 Full-suite verification, self-trial checks, and record updates
    - Run the four new offline suites together with the existing package
      suites and the native self-checks untouched code must still pass;
      confirm the evaluator-package evidence rules (stable records, exits,
      no duplicate representations) hold; update the spec records with the
      dated verification evidence and reciprocal research-log links.
    - Done when every suite is green in one fresh run, no frozen file or
      receipt changed (hash check), and the spec and research records carry
      the dated evidence with planned, executed, and measured states kept
      distinct.
    - _Depends: 6.1_
    - _Requirements: 7.7, 8.1, 8.2, 8.3, 8.4, 8.6_

## Deferred (documented, not scheduled)

- TypeScript extraction evaluation and further target languages: deferred by
  requirements Amendment 2 (2026-11-07); the language extractor seam (tasks
  2.1, 3.1, 4.1 keep language knowledge behind versioned extractors) makes
  each addition a new sibling module plus suite. G0-era TypeScript fixtures
  are preserved as that stage's first material, subject to revalidation.
  _Requirements: 7.1 (TypeScript reading), 7.10 (Slice B withholding)_
- Slice B production tasks (multi-rollout adoption, optional dspy adapter):
  withheld pending G1-R outcome per `Req 7.10` and `prototype-gate.md`.
