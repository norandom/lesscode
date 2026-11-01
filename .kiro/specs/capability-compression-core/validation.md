# Final integration validation

Decision: **MANUAL_VERIFY_REQUIRED**. Independent validator:
`/root/final_core_validation`. Validated implementation revision `63d4cfb` and
current matching runtime source. All 22 tasks are implemented and independently
reviewed. This does not close the feature-wide platform gate.

## Verified evidence

- Windows native nextest: 102 passed, zero skipped; separate doctest command
  passed with zero doctests. Locked build, formatting and Clippy passed.
- Actual executable help and fresh ATS readmission passed.
- Fresh Python focused checks: 103 tests and 156 subtests passed.
- Both full guarded isolated Python suites: 504 passed, 11 skipped, identical
  515 top-level identities plus 485 passing subtests. JUnit emits 1,000 cases
  including those subtests; there are no failures or errors. Raw logs, receipt
  hashes, eight worker collections and ten source-origin records were verified.
- All 96 live Python source/test hashes match the accepted candidate overlay.
  Adopted contracts SHA-256:
  `65e939cc9e1d3acac3b0aed289d0283e9a160f28875a5f5a0d79cdb509a8dcfd`.
  No fresh live full-suite rerun is claimed.
- All 32 acceptance criteria and five requirement sections are mapped. Integration,
  component boundaries, allowed dependencies and CLI liveness pass locally.
  No concrete product defect or blocked implementation task was identified.

The scoped Python self-application is reviewed and adopted: 29 logical statements,
32 physical lines, 17 decisions and three functions removed across the complete
two-module boundary. Retained failures and finite-domain limitations remain part
of the record. No whole-project maturity, maintenance-effort reduction or universal
equivalence claim follows.

## Missing mandatory verification

Current integrated Linux/macOS results are unavailable. Earlier WSL checks are
historical; later temporary-disk SIGBUS is not passing current evidence. The
three-platform workflow exists but was not remotely dispatched in this run.

1. Make the intended reviewed revision available on an authorized remote branch.
2. Run `.github/workflows/native-core.yml` against that exact revision.
3. Retain successful `native-core-ubuntu-24.04-*`, `native-core-windows-2025-*`
   and `native-core-macos-15-intel-*` artifacts, including source/runner identity,
   toolchain, formatting, Clippy, nextest, doctest, build and help results.
4. Re-run the final gate with those exact-revision results. Failed, cancelled or
   skipped jobs do not satisfy the platform requirement.

## Operational note

A redundant parallel shell launch encountered host PowerShell allocation/ICU
startup errors before executing formatting/Clippy. Both subsequently passed
individually. No unresolved product failure resulted; no host tuning or cleanup
was performed.

## Verification Result

- STATUS: MANUAL_VERIFY_REQUIRED
- CLAIM_TYPE: FEATURE_GO
- CLAIM: Full cross-platform capability-compression-core readiness.
- EVIDENCE: Windows full native checks and liveness, focused Python checks,
  source-bound paired regressions, complete traceability and boundary assessment.
- GAPS: Current Linux/macOS integrated runs.
- NOTES: Preserve scoped local conclusions and withhold feature completion/GO
  until the remaining platform evidence passes.
