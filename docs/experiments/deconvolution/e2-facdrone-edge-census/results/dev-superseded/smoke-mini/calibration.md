# E2 calibration

Population: 22 entry responsibilities; 7 tests, 0 failed or errored; production functions reached 6.8% (census partial: below 60%).

| Prediction | Holds | Evidence |
| --- | --- | --- |
| C1 | yes | A write sites 1 (0 greater), churn 19 (2 greater); top decile = fewer than 3 greater |
| C2 | no | C write sites 0 vs median 0.0, churn 0 vs median 0.0 |
| C3 | yes | B churn 721 vs median 0.0 |
| C4 | yes | 4 responsibilities outside A, B, C and R11 in the top decile on two or more of the first five |
| C5 | no | call_edges 0.9589, hops_max 0.7085, write_sites 0.1247, churn_max 0.6864, dos 0.8742, ordering 0.3492 |

## Cases as declared file sets

| Case | call_edges | hops_max | write_sites | churn_max | dos | ordering | lines | statements | static-only |
|---|---|---|---|---|---|---|---|---|---|
| A | 47 | 2 | 1 | 19 | 0.8053 | 1 | 1255 | 632 | 90 |
| B | 49 | 2 | 0 | 721 | 0.7893 | 0 | 993 | 418 | 76 |
| C | 0 | 0 | 0 | 0 | 0.7255 | 0 | 854 | 283 | 60 |

## Top twenty responsibilities

| # | Entry | top decile | call_edges | hops_max | write_sites | churn_max | dos | ordering | files | lines |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `packages/facdrone-reporting/facdrone_reporting/simulation_requests.py::record_simulation_request::13` | call_edges, hops_max, churn_max, dos | 62 | 5 | 0 | 721 | 0.7717 | 0 | 7 | 306 |
| 2 | `packages/facdrone-reporting/facdrone_reporting/commands/report_root.py::persist_simulation_reporting_request::171` | call_edges, hops_max, churn_max, dos | 61 | 4 | 0 | 721 | 0.7557 | 0 | 6 | 295 |
| 3 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence_with_run_slices::308` | hops_max, churn_max | 39 | 2 | 0 | 19 | 0.5537 | 1 | 4 | 234 |
| 4 | `packages/facdrone-testing/facdrone_testing/core/run_slices.py::capture_details::13` | hops_max, dos | 8 | 2 | 0 | 0 | 0.7859 | 0 | 6 | 160 |
| 5 | `packages/facdrone-reporting/facdrone_reporting/testing/reporting_vocabulary.py::reporting_config::56` | call_edges, write_sites | 139 | 0 | 1 | 0 | 0.7413 | 0 | 10 | 841 |
| 6 | `src/facdrone/shells/decision_evidence_publication.py::bind_evidence_with_run_slices::359` | hops_max | 40 | 2 | 0 | 15 | 0.5539 | 1 | 4 | 236 |
| 7 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence::236` | hops_max | 31 | 2 | 0 | 13 | 0.5096 | 0 | 4 | 186 |
| 8 | `src/facdrone/shells/decision_input_slice_store.py::delta_row::66` | hops_max | 6 | 2 | 0 | 2 | 0.4874 | 0 | 2 | 43 |
| 9 | `src/facdrone/shells/decision_input_slice_store.py::full_row::58` | hops_max | 6 | 2 | 0 | 0 | 0.1916 | 0 | 2 | 28 |
| 10 | `src/facdrone/shells/cppi/state_repository.py::DecisionEventRecord.__post_init__::117` | write_sites | 1 | 0 | 1 | 0 | 0.1655 | 0 | 2 | 11 |
| 11 | `src/facdrone/shells/decision_evidence_store.py::seal_decision_evidence::223` |  | 23 | 1 | 0 | 4 | 0.1926 | 0 | 3 | 132 |
| 12 | `packages/facdrone-testing/facdrone_testing/core/evidence.py::evidence_batch::29` |  | 5 | 0 | 0 | 0 | 0.6691 | 0 | 4 | 117 |
| 13 | `src/facdrone/shells/decision_evidence_publication.py::slice_identity::147` |  | 5 | 1 | 0 | 0 | 0.0 | 0 | 1 | 25 |
| 14 | `src/facdrone/shells/decision_evidence_publication.py::payload_batches::405` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 12 |
| 15 | `src/facdrone/shells/decision_evidence_publication.py::stored_evidence_payload::430` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 26 |
| 16 | `src/facdrone/shells/cppi/state_codec.py::_event_model::80` |  | 1 | 0 | 0 | 0 | 0.0 | 0 | 1 | 14 |
| 17 | `packages/facdrone-testing/facdrone_testing/fixtures.py::_logging_state_restored::168` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 25 |
| 18 | `packages/facdrone-testing/facdrone_testing/fixtures.py::_migrated_schema::197` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 8 |
| 19 | `src/facdrone/shells/decision_evidence_publication.py::EvidenceBinding.__post_init__::68` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 2 |
| 20 | `packages/facdrone-kernel/facdrone_kernel/domain/decision_evidence.py::AuditedAbsence.__post_init__::21` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 2 |

File sets: `edges.json`, `responsibilities[].closure_files`.
No repeat run analysed.
