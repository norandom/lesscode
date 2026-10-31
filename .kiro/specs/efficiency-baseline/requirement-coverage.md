# Acceptance-criteria coverage: efficiency-baseline

Assessment date: 2026-10-31. All 56 original numbered acceptance criteria are mapped below. “Covered” means implementation and executable evidence exist for the declared Python evaluator contract. It does not mean complete behavioral equivalence or general corpus effectiveness. Task 7.4 independent review passed and the [final feature gate](validation.md) returned GO. See [readiness](readiness.md) for exact revisions, execution results, platform skips and artifact access.

Test paths below are repository-relative. The full suite executes these tests; native CI also executes genuine analyzers and installed CLI pilots. Optional imported evidence is checked as supplied evidence, not independently manufactured coverage, mutation or runtime measurements.

| Criterion | Observable contract and implementation | Executable evidence | Assessment / limit |
|---|---|---|---|
| 1.1 | `cli.py`, `config.prepare`, `evaluation.evaluate`: supplied baseline/candidate plan and run | `tests/test_cli.py`; `tests/test_evaluation.py`; clean installed plan/run | Covered; Python slice |
| 1.2 | Open-source pinned Lizard/jscpd, transitive locks and retained notices | `tests/test_toolchain.py`; native genuine analyzer probes | Covered; setup downloads are explicit |
| 1.3 | Standard-library CLI without model/service dependency | `pyproject.toml` has no runtime package dependencies; installed console/module startup and evaluation | Covered; no LLM used |
| 1.4 | Native Windows, Linux and macOS package/check/pilot workflow | Run 41198054658, all three jobs and 51 stages passed | Covered on recorded versions/runners |
| 1.5 | Local source copies, subprocesses and filesystem reports | `tests/test_evaluation.py`; `tests/test_snapshot.py`; native pilots | Covered; trusted commands may themselves use networks |
| 2.1 | `snapshot.capture_scope`: actual selected byte identity, revision supplementary | `test_identity_is_sorted_actual_content_not_git_revision`, `test_tracked_working_tree_and_untracked_bytes_are_captured` in `tests/test_snapshot.py` | Covered |
| 2.2 | Frozen scope, exclusions and file categories in Snapshot/Profile | `test_classification_precedence_and_explicit_exclusions` in `tests/test_snapshot.py`; native synthetic-inventory report | Covered |
| 2.3 | Product-only measurement staging and separate test-file delta | `tests/test_metrics.py`; `test_test_file_changes_are_reported_outside_product_metrics` in `tests/test_evaluation.py` | Covered |
| 2.4 | `decision.compare_metrics` identifies every compatibility difference | `test_every_metadata_difference_prevents_delta_and_identifies_values` in `tests/test_decision.py` | Covered; profile changes separately invalidate integrity |
| 2.5 | Added/removed selected files enter manifest and comparison | `test_added_removed_modified_and_test_changes` in `tests/test_snapshot.py`; native synthetic-inventory | Covered |
| 2.6 | Unreadable/missing/unsafe sources return Failure, never success | `test_unreadable_file_and_missing_root_return_failure` in `tests/test_snapshot.py`; `tests/test_config.py` | Covered |
| 3.1 | Lizard typed functions, locations, distribution and hotspots | `tests/test_metrics.py`; native boltons 23 functions, max CCN 15 | Covered; pinned Lizard definition |
| 3.2 | jscpd clone spans, pair tokens and explicit denominator | `test_real_pair_span_counts_locations_and_configuration_isolation` in `tests/test_metrics.py`; native 236 total / 118 duplicated / 0.5 | Covered; pair-span ratio, not unique-source coverage |
| 3.3 | Bytes, physical lines and deterministic gzip bytes | `test_exact_definitions_determinism_and_provenance`, `test_physical_lines_preserve_byte_newlines` in `tests/test_metrics.py` | Covered |
| 3.4 | MetricEvidence retains producer/version/definition/unit/scope | `tests/test_contracts.py`; `tests/test_metrics.py`; typed native reports | Covered |
| 3.5 | Identical inputs/config/tools yield equal structural evidence | `tests/test_metrics.py`; repeated native control-1/control-2 | Covered within matching identities; native generated newline bytes differ across hosts |
| 3.6 | Optional nesting/architecture typed findings remain separate | `tests/test_optional_contracts.py`; `tests/test_optional_evidence.py` | Covered as configured evidence consumption |
| 3.7 | Missing tools, unsupported syntax and failures stay unavailable | `tests/test_metrics.py`; `tests/test_adverse_evaluation.py` | Covered; never substituted with zero |
| 4.1 | Declared checks run on each fresh side; outcomes/logs retained | `tests/test_checks.py`; `tests/test_evaluation.py`; native pilots | Covered |
| 4.2 | Any failed/incomplete declared baseline check is ineligible | `test_all_declared_baseline_checks_even_optional_must_pass` in `tests/test_decision.py`; `tests/test_adverse_evaluation.py` | Covered; optional behavior differs from optional metrics |
| 4.3 | Observed candidate failure rejects despite smaller source | `test_shorter_candidate_failure_cannot_be_compensated` in `tests/test_decision.py`; two genuine native rejections per OS | Covered |
| 4.4 | Frozen independent oracle/API/invariant checks reject mismatch | `tests/test_checks.py`; `tests/test_fixtures.py`; `tests/test_pilot.py` | Covered for supplied declarations |
| 4.5 | Required skipped/missing/error evidence is incomplete | `test_candidate_required_skipped_missing_error_and_absent_incomplete` in `tests/test_decision.py`; `tests/test_checks.py` | Covered |
| 4.6 | Coverage/mutation scope, numerator and denominator retained | `tests/test_optional_contracts.py`; `tests/test_optional_evidence.py`; `tests/test_decision.py` | Covered as supplied evidence; no engine integration claimed |
| 4.7 | Population changes disclosed; candidate explanation required | `test_population_changes_disclose_ids_hash_denominator_and_explanation`, `test_old_or_empty_population_explanation_does_not_explain_candidate` in `tests/test_decision.py` | Covered; explanation is recorded, not proof |
| 4.8 | Human summaries state declared-check limits | `tests/test_report.py`; actual run `summary.md` and limitations | Covered; no equivalence claim |
| 5.1 | PreparedPlan freezes profile/rules/resources/tool identities before execution | `tests/test_plan_contracts.py`; `tests/test_config.py`; native frozen plans | Covered; immutable identity rather than attestation |
| 5.2 | Both sides use same frozen independent evaluator inventory | `tests/test_config.py`; `tests/test_checks.py`; `tests/test_evaluation.py` | Covered |
| 5.3 | Resource/config/test mutations invalidate trial | `tests/test_config.py`; `test_rehashed_frozen_rule_change_cannot_diverge_from_recorded_profile` and `test_real_evaluator_mutation_is_observable` in `tests/test_adverse_evaluation.py` | Covered; local change detection |
| 5.4 | Mandatory complete evidence and rules yield accepted under profile | `tests/test_decision.py`; real unchanged accepted native and clean-install runs | Covered; not universal correctness |
| 5.5 | Missing required metrics/checks cannot accept | `tests/test_decision.py`; `test_missing_analyzers_are_incomplete_when_behavior_checks_pass` in `tests/test_adverse_evaluation.py` | Covered |
| 5.6 | Decision precedence and separate evidence, no compensating score | `test_every_primary_state_and_priority`, `test_shorter_candidate_failure_cannot_be_compensated` in `tests/test_decision.py`; `tests/test_report.py` | Covered; structured payloads have no invented projection |
| 5.7 | Protection mode distinguishes local detection/external claims | `tests/test_contracts.py`; `test_integrity_unavailable_never_accepts_and_protection_is_disclosed` in `tests/test_decision.py` | Covered; execution is not hostile-code containment |
| 6.1 | Versioned JSON and escaped Markdown, hash-indexed recovery | `tests/test_report.py`; `tests/test_experiment_cli.py`; installed recovery | Covered; real summaries inspected |
| 6.2 | Comparable scalar delta = after minus before | `test_signed_deltas_and_zero_baseline` in `tests/test_decision.py`; `tests/test_report.py` | Covered; structured metrics remain structured |
| 6.3 | Nonzero-baseline percentage = 100 × delta / baseline | `test_percentage_uses_declared_formula_order` in `tests/test_decision.py` | Covered |
| 6.4 | Zero/unavailable percentage carries unavailable reason | `test_unavailable_and_missing_are_not_zero`, `test_signed_deltas_and_zero_baseline` in `tests/test_decision.py` | Covered |
| 6.5 | Durable raw outputs, typed analyzer executions/configs, source/runtime identities | `tests/test_report.py`; `tests/test_adverse_evaluation.py`; 1,344 native indexed artifacts and 180 typed analyzer audits | Covered; reproduction requires retained tools/inputs as recorded |
| 6.6 | Related analyzers retain separate producer identities | `test_related_analyzers_remain_separately_identified` in `tests/test_decision.py` | Covered |
| 6.7 | CLI exit states and incomplete recovery expose errors/partial results | `tests/test_cli.py`; `tests/test_experiment_cli.py`; `tests/test_report.py`; `tests/test_adverse_evaluation.py` | Covered; summarize success differs from pair acceptance |
| 7.1 | Manifest declares corpus/task/arms/repetitions/budgets/controls | `tests/test_experiment_contracts.py`; `tests/test_experiments.py`; native three-trial experiment | Covered; producer supplies experiments |
| 7.2 | Trial records retain source/model/settings/prompt/permissions/order/seed | `tests/test_experiment_contracts.py`; `tests/test_experiments.py` | Covered as supplied metadata |
| 7.3 | Reconciliation identifies undeclared condition differences | `test_every_unintended_control_difference_is_specific`, `test_confounded_pairs_retain_outcomes_but_no_paired_sample` in `tests/test_experiments.py` | Covered; matching metadata does not prove causal control |
| 7.4 | Failed/missing/exhausted attempts retained, extras separate | `test_hand_calculated_counts_include_rejected_incomplete_missing_invalid`, `test_exhaustion_retained_even_when_evaluation_complete` in `tests/test_experiments.py` | Covered |
| 7.5 | Eligible correctness denominator and decision counts separate | `tests/test_experiments.py`; `tests/test_experiment_cli.py`; native two-accepted/one-rejected summary | Covered; correctness and acceptance rates differ |
| 7.6 | All-attempt and accepted-only scalar distributions separate | `test_structural_selection_and_hand_calculated_paired_variation`, `test_all_attempt_structural_does_not_select_for_baseline_behavior_eligibility` in `tests/test_experiments.py` | Covered |
| 7.7 | Descriptive repetition statistics retain n and undefined small-sample stddev | `tests/test_experiment_contracts.py`; `test_structural_selection_and_hand_calculated_paired_variation` in `tests/test_experiments.py` | Covered; no inferential confidence interval in v1 |
| 7.8 | Corpus source/revision/content/license/provenance retained | `tests/test_pilot.py`; `tests/test_experiment_contracts.py`; `benchmarks/corpus.json` | Covered; no authorship inference |
| 8.1 | Evaluator elapsed and supplied agent elapsed remain distinct | `tests/test_report.py`; `test_telemetry_and_evaluator_time_remain_distinct_with_provenance` in `tests/test_experiments.py` | Covered |
| 8.2 | Observed token/cost retain source, units and timestamp | `tests/test_experiment_contracts.py`; `tests/test_experiments.py` | Covered as supplied observations |
| 8.3 | Absent tokens/cost explicitly unavailable | `test_explicit_unavailable_telemetry_and_zero_observations` in `tests/test_experiment_contracts.py`; native pilot records | Covered; pilot has no model usage |
| 8.4 | Runtime benchmarks retain workload/environment separately | `tests/test_optional_contracts.py`; `tests/test_optional_evidence.py`; `tests/test_experiments.py` | Covered as supplied evidence |
| 8.5 | Source copies preserve original and uncommitted bytes | `tests/test_snapshot.py`; `tests/test_evaluation.py`; native pilot frozen fingerprints | Covered for normal trusted workloads |
| 8.6 | Bounded commands terminate descendants and retain incomplete diagnostics | `tests/test_process.py`; `tests/test_adverse_evaluation.py`; actual native timeout test on all three OS | Covered; no skipped substitute for native timeout |
| 8.7 | CLI exposes resolved commands before running workloads | `test_commands_flushed_before_execution_and_surviving_storage_failure` in `tests/test_cli.py`; retained native plan/run stdout | Covered |
| 8.8 | Rejection diagnostics retained, originals never rewritten/reverted | `test_real_shorter_broken_pair_rejected_without_reverting_source` in `tests/test_evaluation.py`; native real/synthetic rejection artifacts | Covered |

No numbered acceptance criterion is omitted from this assessment. Imported optional evidence and producer telemetry are tested with typed fixtures; the pilot does not claim to have run external coverage/mutation engines, LLM generations or a runtime benchmark. The final feature gate assessed this mapping together with independent review findings and current test evidence.
