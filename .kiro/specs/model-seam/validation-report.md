# Validation report: the model seam

Written 2026-11-16, 17:25 to 18:10 Europe/Berlin (from the clock), in the main context, after tasks 1 to 8.3.
Method: `kiro-validate-impl`, run inline (no subagent). Research entry:
[R113](../../../docs/research/2026-11-16/02-model-seam-release-and-final-validation.md).

## Verdict

**GO.** Every requirement has evidence, the six first cases pass through the released form of both
binaries, the pipeline is green on all six jobs for `0c1686c`, and the release dry run produced both
archives for the five targets.

One finding (criterion 3.8) is a gap in the tests, not in the behaviour. Three points are recorded
under "Open and not shown" for the owner: the cause of the ARM Linux job's two overruns was not found,
the seam's suite runs on Linux only in the pipeline, and the publish stage was not exercised.

The gate branches are deleted, on the owner's word of 2026-11-16: `model-seam-gate-p4` (`37effda`) and
`model-seam-gate-p2` (`99c394c`), each one prototype commit that is not on main, and the dry-run branch.

## Checks across the whole feature

| Check | Result |
| --- | --- |
| Tasks | 1 to 8.3 ticked; 8.4 is this report |
| `native` suite (`cargo nextest run --locked`) | 400 of 400 |
| `model` suite | 80 of 80 |
| `tools/guide-optimizer` suite | 3 of 3 |
| clippy `-D warnings`, `cargo fmt --check`, budgets | clean for the three crates; budgets exit 0 for `native` and `model` |
| Spec audit, `tools\bin\kiro-cli.exe check --feature model-seam` | 73 criteria, 0 errors, 0 warnings, 37 info (files and components not cross-named in the design's plan; no topics declared) |
| Pipeline (`0c1686c`), run 43911624473 | success, six jobs; see "Pipeline" below for the two runs before it |
| Release dry run on branch `model-seam-release-dry-run` (`214e11c`), run 43904108284 | success: `lesscode-core-<target>` and `lesscode-model-<target>` for all five targets; the publish stage is skipped on a manual run |
| License inventory, `scripts/license-inventory.ps1 -Check` | `native` 83 packages and `model` 294 packages, nothing to change |

## Dependency direction and boundaries

- `native/Cargo.lock` names no client library; the pipeline checks it (`separation-lock`).
- `model/` depends on `native/` (the endpoint file's reading, the core's checks for `check` and
  `measure`). `native/` does not depend on `model/`; it starts the executable (`native/src/seam.rs`).
- `tools/guide-optimizer` depends on `model/` and has no client of its own (R111).
- The step flow's and the driver's suites run in the pipeline only where no `lesscode-model` is found
  (`separation-suites`).
- No model name and no address under `model/src` (`separation-model-source`).
- Not as the design's text says, and recorded as amendments: Amendments 2 to 10 of the design.

## Requirements and their evidence

Tests are named by file and test; all are in the suites counted above.

### Requirement 1: describing an endpoint

| # | Evidence |
| --- | --- |
| 1.1 | `call::a_local_address_is_called_like_any_other_and_the_reply_is_reported`; live R108 |
| 1.2 | `call::only_stated_settings_go_on_the_wire`; the receipts of this validation show the level sent (`released/receipt-released-1.json`) |
| 1.3 | live: cases 1 and 6 set the model for one run (R108, this validation) |
| 1.4 | `failures::a_missing_file_is_named`, `failures::a_missing_entry_is_named_with_its_file` |
| 1.5 | `failures::an_unknown_style_in_the_file_names_the_known_ones`, `call::an_unknown_style_is_refused_before_any_call` |
| 1.6 | `call::a_local_address_is_called_like_any_other_and_the_reply_is_reported`; live cases 5 and 6 |
| 1.7 | `failures::a_refusal_carries_the_endpoints_message_shortened_and_without_the_key`, `record::a_successful_and_a_failed_call_each_leave_one_line_without_the_key`; no kept file of the live runs holds a key |
| 1.8 | `call::the_source_names_no_model_and_holds_no_address`, in the pipeline as `separation-model-source`. Limit: the scan reads the files directly in `model/src` and finds addresses by `http://` and `https://` |

### Requirement 2: one call on every endpoint

| # | Evidence |
| --- | --- |
| 2.1 | `call_command::an_answered_call_is_one_reply_on_standard_output_and_exit_code_0` |
| 2.2 | `call::only_stated_settings_go_on_the_wire` |
| 2.3 | `call::text_in_parts_is_joined_and_the_reasoning_is_left_out` |
| 2.4 | `call::a_local_address_is_called_like_any_other_and_the_reply_is_reported` |
| 2.5 | `call::a_reply_cut_off_at_the_limit_is_reported_as_cut_off` |
| 2.6 | `call::a_reply_says_whether_it_has_the_structure_asked_for`. Design Amendment 2: the structure is checked on the reply, no response format is sent |
| 2.7 | R108 (all six through `lesscode-model call` and `check`). Released form, this validation: all six through `regenerate run` |
| 2.8 | as 1.8 |

### Requirement 3: waiting with an estimate

| # | Evidence |
| --- | --- |
| 3.1 | `waiting::a_wait_stated_in_a_header_is_made`; live R109 (ten calls refused, waited out, answered) |
| 3.2 | `failures::the_notices_name_the_host_and_the_times`; R109 |
| 3.3 | `waiting::a_wait_stated_in_a_header_is_made`, `waiting::a_wait_stated_in_the_message_is_made` |
| 3.4 | `waiting::with_no_wait_stated_the_first_step_is_made_and_then_doubled` |
| 3.5 | `slow_answer::with_no_earlier_answer_the_user_is_told_after_a_minute_and_then_every_half_minute`, `slow_answer::the_expected_time_is_twice_the_median_of_the_earlier_answers`; seen live in case 2 of this validation |
| 3.6 | `waiting::refused_for_good_the_limit_ends_the_call_with_its_report`, `wait::tests::the_limits_are_ten_minutes_of_waiting_and_sixty_for_an_answer` |
| 3.7 | `waiting::the_waits_are_in_the_reply_and_in_the_one_line_of_the_record` |
| 3.8 | **Finding.** By construction: the loop in `wait.rs` sends the next request only after the one before has ended. The design promised a test in which the stub never sees more than one open request; no test asserts that count |

### Requirement 4: saying why a call failed

| # | Evidence |
| --- | --- |
| 4.1 | `call_command::a_failed_call_states_its_reason_on_standard_error_and_ends_with_exit_code_3` |
| 4.2 | `failures::a_refusal_carries_the_endpoints_message_shortened_and_without_the_key` |
| 4.3 | `failures::an_endpoint_that_cannot_be_reached_says_so`, `failures::every_kind_of_failure_has_one_line_that_says_which_it_is`, `slow_answer::the_limit_ends_a_call_whose_answer_does_not_come` |
| 4.4 | the exit-code tests of `call_command`, `check_command` and `measure`; `regenerate run` ends with 3 (native `regenerator` tests with the scripted executable) |
| 4.5 | `repair::a_call_that_fails_ends_the_loop_as_a_failure_of_the_endpoint_not_as_a_rejection`; exit codes 3 and 4 of `check` |

### Requirement 5: checking a configured model

| # | Evidence |
| --- | --- |
| 5.1, 5.2 | `check_command::an_answer_that_passes_the_checks_says_the_endpoint_can_be_used`; live R108 and this validation (clean directory, release build) |
| 5.3 | `check_command::a_refusal_is_reported_in_the_endpoints_words_with_exit_code_3` |
| 5.4 | `check_command::answers_the_checks_reject_end_after_three_calls_with_exit_code_4_and_the_check_named` |
| 5.5 | `check_command::a_wait_before_the_answer_is_reported`; the report line of the live runs |
| 5.6 | `check_command::answers_the_checks_reject_end_after_three_calls_with_exit_code_4_and_the_check_named` |
| 5.7 | the directory count in `check_command`; clean-directory try of task 8.3: no file written |

### Requirement 6: repairing a rejected answer

| # | Evidence |
| --- | --- |
| 6.1 | `repair::a_rejected_reply_is_repaired_by_a_call_that_shows_request_reply_and_reasons`, `repair::the_second_repair_is_a_fresh_call_that_shows_only_the_reply_just_rejected` |
| 6.2 | `repair::a_reply_that_was_cut_off_or_did_not_arrive_is_asked_again_with_the_request_as_it_was` |
| 6.3 | `repair::the_caller_states_how_many_repairs_are_allowed` |
| 6.4 | `repair::a_reply_that_passes_at_once_is_accepted_after_one_call` |
| 6.5 | `repair::after_two_repairs_the_last_reply_comes_back_rejected_with_the_reasons_of_every_attempt` |
| 6.6 | `repair::every_attempt_is_recorded_with_its_verdict_its_stop_and_its_tokens` |
| 6.7 | `repair::an_accepted_outcome_exists_only_after_a_verdict_that_passed` |

### Requirement 7: the fence stays, and the core stays free of models

| # | Evidence |
| --- | --- |
| 7.1 | pipeline step `separation-lock`; `native/Cargo.lock` has no client library |
| 7.2 | pipeline step `separation-suites` (52 tests of the `step_*` and `driver_*` binaries where no `lesscode-model` is found); the message of the missing executable (`native/tests/seam.rs`, and seen with the release build in task 8.3) |
| 7.3 | `repair.rs`: the check runs on the reply's text; `regenerate run` parses and checks before it writes a candidate (unchanged order, native `regenerator` tests) |
| 7.4 | the `compile_fail` doc test of `repair.rs` (`model-doctest` in the pipeline) |
| 7.5 | nothing in admission, the control cases or the step flow changed in this feature; their suites pass unchanged (R112) |

### Requirement 8: the existing model-driven commands

| # | Evidence |
| --- | --- |
| 8.1 | R112; live through the release binaries: five cases of this validation, `"mode": "seam-1"` |
| 8.2 | R111 |
| 8.3, 8.4 | native `regenerator` tests of task 7.3 (planted violation failed); the five receipts under `docs/experiments/model-seam/released/` state host, model, output limit and the reasoning level, no temperature |
| 8.5 | native `regenerator` test of a failed call; seen in task 8.3 |
| 8.6 | the recorded three-turn replay and the stored receipt of gate P4 with a new turn added pass admission (R112) |

### Requirement 9: a record of every call

| # | Evidence |
| --- | --- |
| 9.1 | `record::a_successful_and_a_failed_call_each_leave_one_line_without_the_key`, `waiting::a_call_ended_by_the_limit_leaves_one_line_with_the_report` |
| 9.2 | `record::a_successful_and_a_failed_call_each_leave_one_line_without_the_key` (reasoning and cached input tokens) |
| 9.3 | the same test; `failures::a_refusal_carries_the_endpoints_message_shortened_and_without_the_key` |
| 9.4 | `record::without_a_path_nothing_is_written`, `call_command::the_record_and_the_limits_named_on_the_command_line_are_used` |

### Requirement 10: the indicator

| # | Evidence |
| --- | --- |
| 10.1 | `measure::every_trial_goes_through_the_check_with_the_repairs_allowed_and_is_counted` |
| 10.2, 10.3 | the same test; `measure::the_command_reports_the_built_in_set_and_ends_with_exit_code_0_when_good_enough`; live R110 |
| 10.4 | `measure::three_quarters_of_the_first_replies_are_good_enough_direct` |
| 10.5 | `measure::below_that_nine_tenths_within_the_repairs_are_good_enough_with_the_repair_loop` |
| 10.6 | `measure::a_model_that_is_not_good_enough_as_is_ends_with_exit_code_4_and_the_optimiser_is_named` |
| 10.7 | `measure::twenty_trials_are_run_unless_another_number_is_stated` |
| 10.8 | `measure::the_quick_mode_runs_five_trials_and_has_its_own_verdicts`, `measure::the_quick_mode_has_its_three_outcomes`; live in task 8.3 ("good enough so far") |
| 10.9 | `measure::the_report_names_task_set_check_model_and_trials_beside_the_verdict` |
| 10.10 | `measure::the_built_in_set_has_four_tasks_and_a_good_answer_to_each_passes_the_cores_checks` |
| 10.11 | `measure::a_task_set_supplied_by_the_caller_takes_the_place_of_the_built_in_one` |
| 10.12 | `measure::a_trial_whose_call_failed_is_not_run_and_does_not_count_against_the_model` |
| 10.13 | `measure::more_than_a_fifth_of_the_trials_not_run_gives_no_verdict_and_says_why`, `measure::once_no_verdict_is_possible_the_trials_left_are_not_started` |
| 10.14 | `measure::only_the_three_verdicts_that_say_good_enough_end_with_exit_code_0` |
| 10.15 | the directory check in `measure` |

## The first cases in the released form

`docs/experiments/model-seam/run-released.ps1`, with the two executables unpacked from the Windows
archives of the release dry run (`lesscode-core 1.2.0`, `lesscode-model 0.1.0`), side by side, no
`LESSCODE_MODEL_BIN`. One `regenerate run` per case, 2026-11-16, 17:18 to 17:20 and 17:41. Receipts and candidates:
`docs/experiments/model-seam/released/`.

| Case | Exit | Seconds | Tokens in, out | Candidate written | Sent |
| --- | --- | --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | 0 | 1 | 645, 226 | yes | host, model, limit 64,000, level "high" |
| 2 Mistral Large, Mistral's API | 0 | 73 | 650, 1,427 | yes | the same; one "still waiting" notice at 60 s |
| 3 `gpt-6.1-sol`, OpenAI's API | 0 | 16 | 641, 507 | yes | level "medium" |
| 4 `claude-opus-5-5`, Anthropic's API (17:41, one call, on the owner's word) | 0 | 6 | 887, 525 | yes | host, model, limit; no level, no temperature |
| 5 GLM 5.3, local endpoint | 0 | 1 | 645, 175 | yes | level "high" |
| 6 Mistral Large, local endpoint | 0 | 27 | 650, 1,022 | yes | level "high" |

No call waited for a rate limit and none failed. Only the Windows archives were run; the archives of
the other four targets were built and listed, not executed here.

## Pipeline

| Run | Commit | Result |
| --- | --- | --- |
| 43902113944 | `5aac5a1` (group 7) | cancelled: the ARM Linux job ran past its 30 minutes and left no log; the other five jobs passed |
| 43903666288 | `05190db` (task 8.1) | cancelled for the same reason; the other five jobs passed, the seam's five steps on four platforms |
| 43911624473 | `0c1686c` (tasks 8.2, 8.3 and the trimmed script) | **success**, all six jobs |

Run 43911624473, from its evidence: Linux through Dagger ran every step, among them `model-fmt`,
`model-clippy`, `model-test` (80 of 80), `model-doctest`, `model-budgets`, `separation-suites`,
`separation-lock` and `separation-model-source`; the core's suite passed 399 of 399 there. The other
four platforms ran the core's steps, `separation-suites`, `separation-lock` and the optimiser's steps.

| Job | 05190db | 0c1686c |
| --- | --- | --- |
| Linux through Dagger | 6 min 25 s | 5 min 5 s |
| Linux ARM | over 30 min, cancelled | 5 min 49 s |
| Windows | 19 min 5 s | 13 min 16 s |
| macOS Apple Silicon | 11 min 37 s | 10 min 9 s |
| macOS Intel | 17 min 33 s | 20 min 7 s |

Owner decision of 2026-11-16: the seam's own steps run in the Linux job only, the step and driver suites
run once, and the seam and the guide optimiser share one target directory. Task 8.1 says "on the
pipeline's platforms"; by this decision the seam's suite is not run on Windows, macOS and ARM Linux in
the pipeline. It passes on Windows locally (80 of 80).

Why the ARM Linux job ran past its limit on the two earlier commits was not found: neither cancelled
job left a log or evidence. The same job took about 6 minutes after task 6 and takes about 6 minutes
now. The macOS Intel job did not get faster; it still compiles the client library twice for the guide
optimiser.
## Open and not shown

- Why the ARM Linux job overran on `5aac5a1` and `05190db`. If it returns, a time limit per test in nextest would name a hanging test.
- The seam's suite on Windows, macOS and ARM Linux in the pipeline (owner decision).
- Criterion 3.8 has no test of its own.
- The install scripts' path for `lesscode-model` was syntax-checked and not run: no release carries the
  second archive yet.
- The publish stage (signing ten archives, attestation) runs only for a tag and was not exercised.
- Local Dagger runs were killed with exit 137 on this machine (WSL has 8 GB) in four of the five runs made in the main context; the
  hosted pipeline is the evidence for tasks 7.4 and 8.1.
- The owner has not commented on design Amendments 3, 5, 6, 7, 8 and 10.
