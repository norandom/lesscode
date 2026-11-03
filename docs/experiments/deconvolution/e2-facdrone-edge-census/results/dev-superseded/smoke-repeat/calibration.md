# E2 calibration

Population: 19 entry responsibilities; 6 tests, 0 failed or errored; production functions reached 1.6% (census partial: below 60%).

| Prediction | Holds | Evidence |
| --- | --- | --- |
| C1 | yes | A write sites 1 (0 greater), churn 19 (0 greater); top decile = fewer than 2 greater |
| C2 | no | C write sites 0 vs median 0, churn 0 vs median 0 |
| C3 | no | B churn 0 vs median 0 |
| C4 | no | 1 responsibilities outside A, B, C and R11 in the top decile on two or more of the first five |
| C5 | no | call_edges 0.9358, hops_max 0.7915, write_sites -0.1724, churn_max 0.7207, dos 0.8339, ordering 0.533 |

## Cases as declared file sets

| Case | call_edges | hops_max | write_sites | churn_max | dos | ordering | lines | statements | static-only |
|---|---|---|---|---|---|---|---|---|---|
| A | 47 | 2 | 1 | 19 | 0.8053 | 1 | 1255 | 632 | 90 |
| B | 0 | 0 | 0 | 0 | 0.0 | 0 | 164 | 55 | 114 |
| C | 0 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 | 65 |

## Top twenty responsibilities

| # | Entry | top decile | call_edges | hops_max | write_sites | churn_max | dos | ordering | files | lines |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `src/facdrone/shells/decision_evidence_publication.py::bind_evidence_with_run_slices::359` | call_edges, hops_max, churn_max | 40 | 2 | 0 | 15 | 0.5539 | 1 | 4 | 236 |
| 2 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence_with_run_slices::308` | call_edges, hops_max, churn_max | 39 | 2 | 0 | 19 | 0.5537 | 1 | 4 | 234 |
| 3 | `packages/facdrone-testing/facdrone_testing/core/run_slices.py::capture_details::13` | hops_max, dos | 8 | 2 | 0 | 0 | 0.7859 | 0 | 6 | 160 |
| 4 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence::236` | hops_max | 31 | 2 | 0 | 13 | 0.5096 | 0 | 4 | 186 |
| 5 | `src/facdrone/shells/decision_input_slice_store.py::delta_row::66` | hops_max | 6 | 2 | 0 | 2 | 0.4874 | 0 | 2 | 43 |
| 6 | `src/facdrone/shells/decision_input_slice_store.py::full_row::58` | hops_max | 6 | 2 | 0 | 0 | 0.1916 | 0 | 2 | 28 |
| 7 | `packages/facdrone-testing/facdrone_testing/core/evidence.py::evidence_batch::29` | dos | 5 | 0 | 0 | 0 | 0.6691 | 0 | 4 | 117 |
| 8 | `src/facdrone/shells/cppi/state_repository.py::DecisionEventRecord.__post_init__::117` | write_sites | 1 | 0 | 1 | 0 | 0.1655 | 0 | 2 | 11 |
| 9 | `src/facdrone/shells/decision_evidence_store.py::seal_decision_evidence::223` |  | 23 | 1 | 0 | 4 | 0.1926 | 0 | 3 | 132 |
| 10 | `src/facdrone/shells/decision_evidence_publication.py::slice_identity::147` |  | 5 | 1 | 0 | 0 | 0.0 | 0 | 1 | 25 |
| 11 | `src/facdrone/shells/decision_evidence_publication.py::payload_batches::405` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 12 |
| 12 | `src/facdrone/shells/decision_evidence_publication.py::stored_evidence_payload::430` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 26 |
| 13 | `src/facdrone/shells/cppi/state_codec.py::_event_model::80` |  | 1 | 0 | 0 | 0 | 0.0 | 0 | 1 | 14 |
| 14 | `packages/facdrone-testing/facdrone_testing/fixtures.py::_logging_state_restored::168` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 25 |
| 15 | `packages/facdrone-testing/facdrone_testing/fixtures.py::_migrated_schema::197` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 8 |
| 16 | `src/facdrone/shells/decision_evidence_publication.py::EvidenceBinding.__post_init__::68` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 2 |
| 17 | `packages/facdrone-kernel/facdrone_kernel/domain/decision_evidence.py::AuditedAbsence.__post_init__::21` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 2 |
| 18 | `packages/facdrone-kernel/facdrone_kernel/domain/ledger.py::Reliability.__post_init__::39` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 6 |
| 19 | `packages/facdrone-kernel/facdrone_kernel/domain/ledger.py::StateReliability.__post_init__::69` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 6 |

File sets: `edges.json`, `responsibilities[].closure_files`.
Repeat-run differences: 0 entries.
