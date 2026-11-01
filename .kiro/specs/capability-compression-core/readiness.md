# Scoped readiness review

Task 7.3; requirements 4.2, 4.4, 5.1, 5.5, 5.6, 5.7, 5.8.

- STATUS: MANUAL_VERIFY_REQUIRED for feature-wide cross-platform readiness.
- Local task evidence: VERIFIED on Windows for the exercised workflow.
- Workflow complete: selected Python contracts self-trial, after external replay.
- Compression outcome: accepted scoped replacement; exact independently reviewed
  policy-contract file adopted. This is separate from workflow completion.
- Unsupported maturity coverage: native Rust implementation analysis, TypeScript
  self-compression, whole-project simplification and universal equivalence.

The [readiness report](../../../docs/reports/native-core-readiness.md) contains
working commands, exact empirical scopes, cost accounting and remaining platform
work. The [coverage map](requirement-coverage.md) accounts for all 32 criteria.
The [fresh receipt](../../../docs/evidence/native-core-readiness/receipt.json)
binds the usable binary, ATS readmission, unchanged specimen files and 96 matching
live Python source/test files. Full native nextest passes 102/102; build, doctest,
fmt and clippy pass. Full Python regressions are the guarded isolated paired
515-identity runs (504 passes, 11 platform skips each), not a claimed fresh live
full-suite run. The exact reviewed candidate bytes were adopted and live contract
checks passed 70 tests/104 subtests.

Design alignment: Boundary Commitments; Architecture Pattern & Boundary Map;
CLI/library, Scope, Metrics, Evidence and comparison contracts; Data Models;
System Flows; Testing Strategy. The implementation retains one Cargo library/CLI,
closed records, explicit ownership, versioned language measurements and supplied
external evidence. It does not add a solver, rewrite engine, MCP layer or Rust
parser. Integration fixtures test adverse scope/evidence/decision paths; real
ATS/NPU/TS trials and the self-trial exercise the complete workflow.

No local implementation blocker remains in this task's documentation boundary.
Final task acceptance belongs to independent review. Feature-wide GO remains
withheld: latest integrated Linux and macOS runs are missing. Earlier WSL parser
validation is historical; later WSL SIGBUS at 99% temporary-disk utilization is
not a passing current run. The three-platform workflow is prepared, not dispatched.
No remote action or disk cleanup was performed here. Missing platform evidence
does not retroactively invalidate the scoped Python trial or force code deletion.

## Verification Result
- STATUS: MANUAL_VERIFY_REQUIRED
- CLAIM_TYPE: FEATURE_GO
- CLAIM: Full cross-platform capability-compression-core readiness.
- EVIDENCE: Windows native 102/102; build/doctest/fmt/clippy/CLI readmission;
  paired guarded Python regressions; all 32 criteria mapped to concrete artifacts.
- GAPS: Current Linux/macOS integrated runs unavailable; no whole-project maturity.
- NOTES: Retain actual latest three-platform job logs before feature-wide GO.
