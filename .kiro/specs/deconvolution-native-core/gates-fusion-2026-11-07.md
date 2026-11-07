# 1:1 gate map: deconvolution-native-core

Written 2026-11-07 (owner request) from the arbitrated fusion plan. Every gate of the
deconvolution loop (spec `deconvolution-loop`, requirements 1-12) maps 1:1 to a kiro
phase gate, a native component/subcommand and a named smoketest. Smoketests are
fixture replays of committed E3/E4 artefacts (insta snapshots; cargo nextest);
no live store, no model endpoint, no heavy job. Exit codes: 0 accepted, 1 rejected /
not-demonstrated, 2 incomplete, 3 invalid input / baseline-ineligible / IO failure.
Preregistered decision table: stale result file -> 1 (round failed); solver gate
refusal from insufficient evidence -> 2; completed-but-inconclusive -> 1; never 0
outside acceptance; "inconclusive" never surfaces as equivalence (req 9).

## Runtime gates (loop artefact -> component -> smoketest)

| Loop gate | Kiro phase gate | Subcommand | Smoketest (fixture -> expected) |
|---|---|---|---|
| Surface derivation (req 1) | task review; final validation | `surface derive` | `surface_e4_matches_committed`: frozen E4 copy -> byte-identical to committed surface artefact; unresolved names -> exit 2 |
| Double-recording byte-identity (req 2, L1+L2) | prototype gate S1 + task review | `record` | `record_replay_committed_pair`: two committed E4 recordings byte-compare equal, exit 0, plus S1 verdict artifact exists; live double-recording is S1 only |
| Replay zero-divergence acceptance (req 3) | task review; final validation | `replay` + `accept` | `accept_e4_zero_divergence`: committed E4 trace vs regenerated tree -> zero L1 divergence, same seed baseline, identical harness outputs, exit 0 |
| Fresh-result-file rule (req 3) | task review | `accept` | `accept_stale_result_file_rejected`: stale result file -> exit 1, never 0 |
| First-divergence report (req 3) | task review | `accept` | `accept_reports_first_divergence`: planted divergence -> exit 1, first divergence only, snapshot matches committed report |
| Query differential acceptance (req 3.2) | task review; final validation | `differential --query` | `query_differential_100_perturbed_states`: committed artefacts over >=100 perturbed seeded states -> verdicts byte-identical |
| Three-way measures (req 4) | task review; final validation | `measure` | `measure_e4_three_readings`: as-written + canonical + separated-form byte-identical to committed E4 measures |
| Structural contract (req 5) | task review | `measure --contract` / `accept --contract` | `structural_contract_e4_pure_share`: pure share matches committed; below minimum -> sub-step not counted, exit 2 |
| Edit-scope hash check (req 6.4/6.5, 11.2) | per-task review; final validation | `scope verify` | `scope_hash_crlf_e4`: hashes over bytes as stored match committed; tampered outside file -> void, exit 3 |
| Patch sentinel (req 6.6) | task review | `accept --patch-check` | `patch_sentinel_kill`: patched name not preserved -> kill criterion fires, candidate voided, exit 1 |
| Turn rule (req 6.7) | task review; final validation | `report turn` | `turn_rule_failed_turn_e4`: committed E4 turn-2 failed-turn record -> next-turn inputs = unchanged inputs + first divergence + measure gaps only, byte-for-byte |
| Round cap / void rules (req 6.2-6.4) | task review | `report turn` | `round_void_on_outside_change`; `round_cap_enforced` -> exit 2 at the preregistered cap |
| Receipt fields (req 6.8, 11.2/11.3) | prototype gate S2; task review | `record` (emits) / `report` (verifies) | `receipt_replay_e4_byte_for_byte`: re-emit from committed inputs -> byte-identical; prompts/responses/model/seed hashed |
| Independent review protocol (req 7) | per-task review | `report review-pack` | `review_pack_no_rationale`: pack from E4 artefacts -> rationale markers absent; hashes/measures/hypotheses present |
| Sensitivity: sample + classify + restore (req 8.1-8.6) | task review; final validation | `mutate sample/apply/restore` + `score` | `sensitivity_replay_e3_kill_rate`: recompute 0.643, assert reported FAILED vs the 0.70 bar, exit 1; `mutant_restore_hash`; `both_tree_composition_stated` |
| Missing-test report (req 8.7) | task review | `score --missing-tests` | `missing_tests_e4_snapshot`: byte-identical to committed E4 report |
| Four solver verifier gates (req 9.2) | task review; final validation | `differential --solver` | four refusal tests: `solver_gate1_paths_explored` (<=1 path -> no verdict, exit 2); `solver_gate2_driver_agreement` (corpus disagreement -> reported, exit 1); `solver_gate3_planted_mutant_found` (renaming not exhausted -> no verdict, exit 2); `solver_gate4_concrete_replay`; plus `solver_inconclusive_not_equivalent` (completed inconclusive -> exit 1, never 0) |
| Failed control voids session (contract §7) | task review | `differential` | `failed_control_voids_session` |
| Failed control voids differential step only (req 9.5) | task review | `differential` | `failed_control_voids_differential_only`: replay E4's void -> differential voided, run not voided |
| Held-out corpus (req 2.3/2.7; E4 T7) | task review; final validation | `accept --held-out` | `held_out_74_zero_divergence`: 74 committed sequences, seed 20261107 -> zero divergence, exit 0 |
| Residue / protection statement (req 10) | final validation | `residue` | `residue_req10_2_form_e4`: regenerated statement byte-identical to committed; failed predictions next to held ones |
| Kill criteria (3 groups) + kill-rate convention (contract §7) | task review; final validation | `score` | `kill_criteria_groups_void_correctly`: one synthetic artefact per group -> voided at candidate/sub-step/run level; convention = killed/(killed+survived) |
| Host guard / adaptive deadlines (req 11.4/11.5) | prototype gate S6; task review | `host guard` / `host probe` | `host_guard_scales_deadline` (fixture load probes; no fixed wall-clock cap); `host_guard_one_heavy_job` (second heavy job refused in sandbox) |
| Test-infra detection (req 12.4) | task review | `scope inspect` | `testinfra_detected_e4`: detection report byte-identical to committed pre-run report |
| Budgets / line overrun (req 11.6) | final validation | `report budgets` | `budget_overrun_reported`: overruns reported next to held predictions; line-budget check on the native tree itself |
| Baseline deselections (req 12.2) | task review | `scope inspect` | `baseline_deselections_equal`: same deselections applied to original and candidate, never repaired |

## Kiro phase gates (each has a 1:1 executable check over a committed evidence registry)

| Phase | Gate check (fails closed) |
|---|---|
| 0 pre-kiro owner gate | owner go/no-go recorded (four open questions + recorder-imports ratification); not a kiro phase |
| 1 requirements | `requirements_traceability`: every loop req 1-12 maps to a component/spike/exclusion; no do-not-build item present |
| 2 prototype gate (spikes) | `spike_verdicts_complete`: every spike S1-S9 has a dated verdict artifact before design opens |
| 3 design | `design_evidence_coverage`: every reuse choice cites a spike verdict or committed evidence ID |
| 4 tasks | `task_smoketest_coverage`: every task names an existing smoketest |
| 5 implementation | per-task independent review (`kiro-review`): reviewer gets plan + diff + two trees only; the task's named smoketest passes on both trees; `scope verify` per task |
| 6 final validation | full fixture-replay suite green on native Windows; 5-target CI matrix green; byte-for-byte E3/E4 replay; release build; complexity caps pass |
