# Verification approach and current evidence

Status: proposed correction qualification, not a production task list or a fix
claim. Requirements approval and then design approval remain pending.

## Current results

At source revision `a755712`, the existing regenerator suites pass (Rust 45/45,
Python 59/59). New independent expected-behavior probes produce 8 Rust failures
plus one positive control pass, and 5 Python failures. All are retained in
[R39's evidence directory](../../../docs/experiments/deconvolution/refinement-correctness/results).
Several failures test the same defect in both languages; they are not thirteen
independent bugs. No full repository regression or cross-platform run was made.

## Requirements to adversarial evidence

| Requirement | Reproduced baseline failure | Required correction evidence |
|---|---|---|
| 1: candidate identity | `accepted_best_matches_delivered_candidate`, Python `test_best_candidate_is_retained` | Distinct candidate bytes; selected hash equals delivered hash; accepted and exhausted runs; ties; later failure; failed restoration |
| 2: oracle completeness | `failed_process_cannot_pass_from_success_file`, `timeout_cannot_pass_from_success_file`, `contradictory_counts_cannot_pass` | Real child processes; success/failure/timeout precedence; payload validation; absent/stale report; partial results retained; positive completed-success control |
| 3: feedback and obligations | `next_turn_receives_runtime_counterexample`, `still_failing_counterexample_remains_unresolved`, Python counterparts | Capture actual submitted prompts; causal repair response; blocked held-out leakage on each dispatch; candidate-specific recheck and obligation state |
| 4: outcome separation | `low_search_threshold_cannot_establish_validity`, `missing_measurement_cannot_establish_compression`, Python counterparts | Low threshold stops without validity; absent measurements remain unmeasured; measured pass/fail; actual adoption stays separate |
| 5: verifier adequacy | Existing suites pass alongside the new failing probes | Independent expected outcomes; transition sequence properties; named decision faults detected for the intended reason |
| 6: bounded recovery | Historical E3/E4 oracle-blind differences; R39 state loss | Versioned protocol; same checks on original/candidate; concrete real-code execution; bounded exhaustion retains fallback and obligations |

The additional cases in the last column are planned. Do not report them as run.
Each criterion in requirements.md must receive a concrete test/evidence mapping
in the later technical design; a pointer to this table alone is not coverage.

## Qualification gates after approval

1. **Correctness regression:** the unchanged R39 expected-behavior assertions
   must pass against the correction. Add the required edge cases above. Any
   interface change needs an explicit test migration preserving the assertions;
   deleting a test or lowering its expectation does not close a finding.
2. **Causal integration:** exercise actual controller, oracle process, file
   writes, selected-candidate recovery and receipt admission. A test model may
   be deterministic, but its repair must depend on receiving the diagnostic.
   Hash and replay the final selected artifact. Receipt integrity alone cannot
   establish truth of its verdict.
3. **Verifier sensitivity:** isolate and inject the six named decision faults
   in concept.md. Record source identity and the intended assertion that rejects
   each. Run a baseline in the same environment. Timeouts, compile failures and
   unrelated test failures are not successful behavioral detections.
4. **Regression and operability:** use cargo-nextest and the separate doctest
   command, pytest-xdist, relevant format/lint/budget checks, and the repository's
   existing Dagger native pipeline for required cross-platform validation. Keep
   known unrelated failures explicit. See [testing](../../../docs/testing.md)
   and [CI validation](../../../docs/ci-validation.md). Do not use WSL for
   development-task execution.
5. **Compression qualification:** before a new efficacy claim, freeze a real
   responsibility, removal hypothesis, observation matrix, useful-reduction
   target and attempt budget. Run the repaired loop on actual code under the
   revised protocol; preserve known negative cases and independent held-out
   evaluation. This is a later proposal, not Slice B adoption authorization.

Routine correction of the reproduced logic errors does not require inventing
a new compression mechanism. Any changed modeling assumptions still invoke
[prototype-first](../../steering/prototype-first.md). Repair evidence establishes
the corrected behavior only; it does not discharge the real-code usefulness gate.

## Runtime provenance limits

The diagnosis used the existing specimen environment: rustc 1.98.1, installed
nextest 0.9.150, CPython 3.13.16, pytest 9.0.3 and xdist 3.8.0. These differ from
the repository's documented nextest 0.9.143 installation and pytest 9.1.1 test
lock. The exploratory repros record the actual versions rather than claiming a
pinned release-validation run. No dependency was installed or upgraded. Future
qualification must reconcile and use its declared runner/dependency inventory.

## Completion claim

Diagnosis and corrective requirements: supported by fresh red/positive-control
evidence and inline review. Implementation correctness, fix success, full-suite
GO, production readiness and useful compression: not claimed.
