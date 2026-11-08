# Implementation Plan: Refinement Evidence Integrity

## Tasks

- [x] 1. Oracle process execution and verdict consistency in Rust
  - Check process return code from `recording::call` in `RuntimeOracle::evaluate`; return infrastructure failure on non-zero exit code or timeout regardless of written verdict file.
  - Validate verdict payload: ensure `ok == true` is rejected as contradictory if `passed < total` or `failures` is non-empty.
  - Verify against positive control, process failure, timeout, and contradictory counts probes.
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 5.4_

- [x] 2. Candidate retention and filesystem delivery in Rust and Python
  - Snapshot candidate files whenever a rollout improves `best`.
  - Restore the exact selected candidate files to `cand_root` before returning from `run_refine_loop` upon acceptance or exhaustion.
  - Verify that multi-rollout turns deliver the passing candidate rather than later failing candidate files on disk.
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 4.6, 5.2_

- [x] 3. Turn feedback propagation and obligation tracking in Rust and Python
  - Scope `turn_feedback` across turns in `run_refine_loop` and populate it from `best` at turn completion; enforce feedback boundary.
  - Ensure prompt on subsequent turns receives applicable runtime counterexamples.
  - Correct `discharged_obligations` to never discharge obligations exhibited by a failing candidate. Retain active unresolved obligations.
  - Verify feedback propagation and obligation retention across attempts.
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 5.3_

- [x] 4. Independent outcome decisions for validity and compression in Rust and Python
  - Separate search stopping from behavioral validity: `validity` requires full behavioral pass, tier 2 verification, and zero unresolved obligations.
  - Fix `compression_met`: require explicit non-empty static targets and zero gaps; report false (unmeasured) when targets are absent.
  - Verify low search threshold cannot manufacture validity, and absent measurements cannot manufacture compression.
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 5.5_

- [ ] 5. End-to-end verification and regression suite execution
  - Run full R39 adversarial probe battery in Rust (9 cases: 1 positive control + 8 expected-pass regressions).
  - Run full R39 probe battery in Python (5 cases: expected-pass regressions).
  - Verify existing native regenerator suite (45/45 pass) and Python regenerator suite (59/59 pass).
  - Verify canonical line count complies with the R38 7,004 line ceiling.
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.5_
