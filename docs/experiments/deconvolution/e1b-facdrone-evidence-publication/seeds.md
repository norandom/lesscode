# E1b seed set (Arm A), per plan amendment 1

- Written: 2026-11-03, Europe/Berlin, development phase. **Status: coverage-confirmed 2026-11-03 (see below).**
- Definition (amendment 1): `tests/unit/test_decision_slice_publisher_db.py`, every other test
  file that coverage shows executing lines of the eight scope files under the gate's marker
  expression `not port_baseline and not live_broker`, and the R16 codec harness.

## Members

1. **R16 codec harness** (non-pytest): `docs/experiments/constraint-modeler/evidence-publication/check.py`,
   unchanged, 128 observations. `run_arm.py` runs it first in both arms against the root under
   test and compares its stdout byte for byte with `results/r16-baseline.json`, which
   `run_arm.py r16-record` writes from the frozen copy with the E2 venv. R16's own
   `results/baseline.json` cannot serve as that baseline: in this venv 120 of 128 observations
   are identical and 8 differ only in pydantic's documentation URL (`errors.pydantic.dev/2.12`
   in R16's venv, `/2.13` here; smoke receipt label `smoke-r16-check-frozen`).
2. **Pytest seeds**: the block below. `run_arm.py` groups them by suite (root `tests/`, then each
   satellite package under its own `pyproject.toml`, as `scripts/gates.sh` does) and runs them
   serially with `-p e2_winshim` and the gate's marker expression.

## How the static list was derived (2026-11-03)

Running coverage on 115 files one by one does not fit the development-phase limit (single
files under two minutes, one at a time; the PostgreSQL start alone is about 10 s per process).
Candidates were therefore derived statically over the frozen copy:

- test files (`test_*.py` under `tests/` and `packages/*/tests/`) that import any scope module
  directly (`decision_evidence_publication`, `decision_evidence_store`, `decision_evidence_reader`,
  `decision_input_slice_store`, `decision_slice_publisher`, `cppi.state_repository`,
  `wiring.run_evidence_store`, `facdrone_kernel.domain.decision_evidence`): 109 files;
- plus test files importing the Facdrone test helpers that drive the scope
  (`facdrone_testing.core.{evidence, evidence_reader, run_slices, consumed_evidence,
  cppi_durability, decision_moments}`): 7 further files;
- minus `packages/facdrone-research/tests/test_lifecycle_full_loop_db.py`, which the gate tier
  ignores (E2 README).

Result: 115 files (98 root, 12 reporting, 5 research) of 889 test files. Many reporting
files are `*_e2e` operator-DB tests that the embedded server skips; they stay in the list
because coverage, not the name, decides membership.

**Known gap of the static list.** 41 production and test-support modules outside the scope
import scope modules (state codec, cppi cell and daemon, sim drivers, alpha runners, handover
roots). Tests that reach the scope through them without importing it are missing here. The
coverage step will therefore most likely enlarge the set, possibly substantially.

## Confirmation procedure (execution window, before Arm A)

```bash
python run_arm.py all --cov results/cov-all     # whole gate tier, serial, per-file contexts
python run_arm.py seeds results/cov-all > results/seeds-from-coverage.txt
```

`run_arm.py all` runs the gate tier with coverage restricted to the eight scope files and a
plugin (`-p run_arm`) that switches the coverage context to each test's file. `seeds` lists every
test file whose context executed at least one scope line; import-time execution during
collection (empty context) does not count. The block below is then replaced by that list,
with a dated note here and the old list kept in git history; `test_decision_slice_publisher_db.py`
must be in it (it is a named member).

## Coverage confirmation, 2026-11-03 (execution session)

Executed 2026-11-03 (receipt labels `seeds-coverage`, `seeds-coverage-research-rerun`, `seeds-list`;
plan amendments 4 and 5). Whole gate tier, serial, coverage restricted to the 8 scope files, one
context per test file. **191 test files** executed at least one scope line (174 root, 12 reporting,
5 research). 99 of the 115 static candidates are confirmed. 16 static candidates executed no scope
line, and 92 files outside the static list did execute scope lines. The static list is kept
verbatim in `results/seeds-static-candidates.txt` (this directory is not yet in git), and the raw
output in `results/logs/seeds-list.stdout.log`. `tests/unit/test_decision_slice_publisher_db.py`
is a member.

Amendment 3 selection. `results/selection-map.json` (sha256 in the receipt) lists, per mutated
module, the seed files whose context executed at least one line of that module:
147 for `decision_evidence_publication.py` and 60 for `cppi/state_repository.py` (151 in the
union). Its `deselect` list (236 node ids) holds every test that failed in this run. 10 of them
lie in selected files and are deselected in both arms.

## Seed files

```seeds
packages/facdrone-reporting/tests/test_p1_explanation_acceptance.py
packages/facdrone-reporting/tests/test_paper_comparison.py
packages/facdrone-reporting/tests/test_paper_diagnosis.py
packages/facdrone-reporting/tests/test_paper_diagnosis_evidence.py
packages/facdrone-reporting/tests/test_paper_diagnosis_reader_db.py
packages/facdrone-reporting/tests/test_paper_first_week.py
packages/facdrone-reporting/tests/test_paper_holdings_reuse.py
packages/facdrone-reporting/tests/test_paper_reconciliation.py
packages/facdrone-reporting/tests/test_paper_review.py
packages/facdrone-reporting/tests/test_paper_snapshot_db.py
packages/facdrone-reporting/tests/test_reporting_production_composition.py
packages/facdrone-reporting/tests/test_usd_run_persistence.py
packages/facdrone-research/tests/test_conditional_census.py
packages/facdrone-research/tests/test_study_arm_evidence.py
packages/facdrone-research/tests/test_study_attribution_routes.py
packages/facdrone-research/tests/test_study_evidence_compaction.py
packages/facdrone-research/tests/test_study_slice_evidence.py
tests/contracts/test_catchup_packaging.py
tests/contracts/test_client_acceptance.py
tests/contracts/test_client_wire_compat.py
tests/contracts/test_client_wire_consumption.py
tests/contracts/test_composed_policy_parity.py
tests/contracts/test_consumed_inputs.py
tests/contracts/test_consumed_publication.py
tests/contracts/test_evidence_compaction_equivalence.py
tests/contracts/test_evidence_single_pass.py
tests/contracts/test_execution_delivery_http.py
tests/contracts/test_execution_reports.py
tests/contracts/test_live_trail_parity.py
tests/contracts/test_p1_mandate_identity.py
tests/contracts/test_sim_consumed_publication.py
tests/contracts/test_sim_server_parity.py
tests/contracts/test_target_ancestry.py
tests/contracts/test_thin_protocol_kit.py
tests/contracts/test_thin_protocol_kit_account.py
tests/contracts/test_thin_protocol_kit_resting.py
tests/contracts/test_thin_transfer_window.py
tests/unit/test_acceptance_crisis_restart.py
tests/unit/test_acceptance_parity.py
tests/unit/test_acquisition_acceptance.py
tests/unit/test_alpha_policy.py
tests/unit/test_alpha_policy_admission.py
tests/unit/test_alpha_policy_fx.py
tests/unit/test_alpha_policy_sizing.py
tests/unit/test_alpha_policy_views.py
tests/unit/test_baseline_store.py
tests/unit/test_baseline_store_db.py
tests/unit/test_burn_in_cache.py
tests/unit/test_cfd_mechanics.py
tests/unit/test_cli_supersede.py
tests/unit/test_close_retry_storage.py
tests/unit/test_combine_properties.py
tests/unit/test_consumed_evidence_contract.py
tests/unit/test_consumed_scoring.py
tests/unit/test_cppi_app.py
tests/unit/test_cppi_commit_race.py
tests/unit/test_cppi_commit_race_db.py
tests/unit/test_cppi_composed_snapshot.py
tests/unit/test_cppi_decision_flow.py
tests/unit/test_cppi_decision_record_commit.py
tests/unit/test_cppi_durability.py
tests/unit/test_cppi_handover_flow.py
tests/unit/test_cppi_monitor_worker.py
tests/unit/test_cppi_root.py
tests/unit/test_cppi_snapshot_loader.py
tests/unit/test_cppi_trace_binding.py
tests/unit/test_cppi_writer_lock.py
tests/unit/test_crossed_stop_out.py
tests/unit/test_daemon_mode.py
tests/unit/test_daemon_mode_http.py
tests/unit/test_daemon_record_composition.py
tests/unit/test_day_close_reconciliation.py
tests/unit/test_decide_step_timing.py
tests/unit/test_decision_day_evidence.py
tests/unit/test_decision_day_evidence_db.py
tests/unit/test_decision_day_refusals.py
tests/unit/test_decision_day_served.py
tests/unit/test_decision_evidence_payload.py
tests/unit/test_decision_evidence_reader.py
tests/unit/test_decision_evidence_reader_db.py
tests/unit/test_decision_evidence_store.py
tests/unit/test_decision_input_slice_store.py
tests/unit/test_decision_slice_publisher_db.py
tests/unit/test_decision_stage_timing.py
tests/unit/test_decision_trace.py
tests/unit/test_deferred_intraday_routes.py
tests/unit/test_depot_capacity.py
tests/unit/test_depot_durability.py
tests/unit/test_depot_policy.py
tests/unit/test_depot_rebuy_crossed.py
tests/unit/test_depot_record.py
tests/unit/test_depot_serving.py
tests/unit/test_entry_gate_daily_stop.py
tests/unit/test_entry_gate_window.py
tests/unit/test_equal_weight_source.py
tests/unit/test_exit_trail_golden.py
tests/unit/test_factor_plugins_identity.py
tests/unit/test_fixed_weight_combine.py
tests/unit/test_flag_actuator.py
tests/unit/test_fresh_combine.py
tests/unit/test_futures_composition.py
tests/unit/test_futures_cppi_shortfall.py
tests/unit/test_futures_live_snapshot.py
tests/unit/test_futures_serving_projection.py
tests/unit/test_futures_wire_evidence_db.py
tests/unit/test_futures_wire_request.py
tests/unit/test_handover_binding.py
tests/unit/test_handover_repository.py
tests/unit/test_handover_stop_floors.py
tests/unit/test_io_execution_report_retry.py
tests/unit/test_io_persist_run_race_db.py
tests/unit/test_io_pin_cell_execute_store.py
tests/unit/test_io_pin_request_budget.py
tests/unit/test_io_pin_wait_commit.py
tests/unit/test_io_secondary_persist.py
tests/unit/test_joint_feasibility_decision.py
tests/unit/test_last_complete_bar_e2e.py
tests/unit/test_lifecycle_evidence.py
tests/unit/test_lifecycle_evidence_intervals.py
tests/unit/test_lifecycle_evidence_projection.py
tests/unit/test_lifecycle_monitor_runner_db.py
tests/unit/test_live_slice_namespace_db.py
tests/unit/test_live_state_repository.py
tests/unit/test_live_trail_serving.py
tests/unit/test_mandate_currency_matrix.py
tests/unit/test_mandate_event_evidence.py
tests/unit/test_mandate_generation.py
tests/unit/test_mandate_guard.py
tests/unit/test_mandate_guard_db.py
tests/unit/test_mandate_guard_retry.py
tests/unit/test_mandate_review_db.py
tests/unit/test_mandate_terminal_db.py
tests/unit/test_mechanics_binding.py
tests/unit/test_mechanism_telemetry.py
tests/unit/test_missed_day.py
tests/unit/test_missed_day_db.py
tests/unit/test_money_typing.py
tests/unit/test_operator_close_durability.py
tests/unit/test_operator_close_resolution.py
tests/unit/test_operator_close_serving.py
tests/unit/test_operator_resume_cadence.py
tests/unit/test_operator_resume_commit.py
tests/unit/test_operator_resume_durability.py
tests/unit/test_operator_resume_lift.py
tests/unit/test_operator_resume_serving.py
tests/unit/test_owed_venue_exit.py
tests/unit/test_panel_leg_floors.py
tests/unit/test_panel_leg_served_gate.py
tests/unit/test_policy_stage_emission.py
tests/unit/test_policy_trace.py
tests/unit/test_position_limit.py
tests/unit/test_price_limit_gate.py
tests/unit/test_protection_briefing.py
tests/unit/test_reported_position_protection.py
tests/unit/test_resting_entry_book.py
tests/unit/test_run_config_inspection.py
tests/unit/test_run_config_payload.py
tests/unit/test_run_evidence_store.py
tests/unit/test_run_slice_namespace.py
tests/unit/test_run_slice_policy.py
tests/unit/test_run_slice_sealing.py
tests/unit/test_run_slice_writers_db.py
tests/unit/test_serve_schedule.py
tests/unit/test_served_crossed_stops.py
tests/unit/test_session_bar_trail.py
tests/unit/test_session_flat.py
tests/unit/test_session_prices.py
tests/unit/test_session_schedule.py
tests/unit/test_session_stepper.py
tests/unit/test_session_stepper_edges.py
tests/unit/test_session_venue.py
tests/unit/test_sim_day_open.py
tests/unit/test_sim_driver.py
tests/unit/test_sim_evidence_moment.py
tests/unit/test_sim_futures_binding.py
tests/unit/test_sim_result.py
tests/unit/test_sim_trail_walk_forward.py
tests/unit/test_slice_memory.py
tests/unit/test_stage_projection.py
tests/unit/test_thin_attempts.py
tests/unit/test_thin_attempts_db.py
tests/unit/test_thin_completion.py
tests/unit/test_thin_price_shadow.py
tests/unit/test_thin_protection_journal.py
tests/unit/test_thin_protocol_journal.py
tests/unit/test_thin_shadow.py
tests/unit/test_thin_state_import_db.py
tests/unit/test_trail_floor_reset_db.py
tests/unit/test_trail_overnight_fold.py
tests/unit/test_transient_capture_codec.py
tests/unit/test_usd_run_persistence.py
```
