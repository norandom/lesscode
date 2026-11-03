# E2 calibration

Population: 24 entry responsibilities; 7 tests, 0 failed or errored; production functions reached 6.8% (census partial: below 60%).

| Prediction | Holds | Evidence |
| --- | --- | --- |
| C1 | yes | A write sites 1 (0 greater), churn 19 (2 greater); top decile = fewer than 3 greater |
| C2 | no | C write sites 0 vs median 0.0, churn 0 vs median 0.0 |
| C3 | yes | B churn 721 vs median 0.0 |
| C4 | yes | 3 responsibilities outside A, B, C and R11 in the top decile on two or more of the first five |
| C5 | no | call_edges 0.9377, hops_max 0.6751, write_sites 0.1091, churn_max 0.6914, dos 0.8163, ordering 0.349 |

## Cases as declared file sets

| Case | call_edges | hops_max | write_sites | churn_max | dos | ordering | lines | statements | static-only |
|---|---|---|---|---|---|---|---|---|---|
| A | 47 | 2 | 1 | 19 | 0.8054 | 1 | 1255 | 632 | 90 |
| B | 49 | 2 | 0 | 721 | 0.7894 | 0 | 993 | 418 | 76 |
| C | 0 | 0 | 0 | 0 | 0.7255 | 0 | 854 | 283 | 60 |

Lines and statements above are those of executed callables; declared sizes (R15 basis): A {'files': 8, 'physical_lines': 2682, 'ast_statements': 1256}; B {'files': 6, 'physical_lines': 1938, 'ast_statements': 925}; C {'files': 5, 'physical_lines': 1865, 'ast_statements': 693}.

## Top twenty responsibilities

| # | Entry | top decile | call_edges | hops_max | write_sites | churn_max | dos | ordering | files | lines |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `packages/facdrone-reporting/facdrone_reporting/simulation_requests.py::record_simulation_request::13` | call_edges, hops_max, churn_max, dos | 62 | 5 | 0 | 721 | 0.7718 | 0 | 7 | 306 |
| 2 | `packages/facdrone-reporting/facdrone_reporting/commands/report_root.py::persist_simulation_reporting_request::171` | call_edges, hops_max, churn_max, dos | 61 | 4 | 0 | 721 | 0.7558 | 0 | 6 | 295 |
| 3 | `src/facdrone/shells/config/mandate_loader.py::select_mandate::210` | call_edges, write_sites, dos | 109 | 0 | 1 | 0 | 0.6203 | 0 | 7 | 625 |
| 4 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence_with_run_slices::308` | hops_max, churn_max | 39 | 2 | 0 | 19 | 0.5538 | 1 | 4 | 234 |
| 5 | `src/facdrone/shells/decision_evidence_publication.py::bind_evidence_with_run_slices::359` | hops_max | 40 | 2 | 0 | 15 | 0.5539 | 1 | 4 | 236 |
| 6 | `src/facdrone/shells/decision_evidence_publication.py::seal_evidence::236` | hops_max | 31 | 2 | 0 | 13 | 0.5097 | 0 | 4 | 186 |
| 7 | `src/facdrone/shells/decision_input_slice_store.py::delta_row::66` | hops_max | 6 | 2 | 0 | 2 | 0.4874 | 0 | 2 | 43 |
| 8 | `src/facdrone/shells/decision_input_slice_store.py::full_row::58` | hops_max | 6 | 2 | 0 | 0 | 0.1916 | 0 | 2 | 28 |
| 9 | `src/facdrone/shells/cppi/state_repository.py::DecisionEventRecord.__post_init__::117` | write_sites | 1 | 0 | 1 | 0 | 0.1655 | 0 | 2 | 11 |
| 10 | `src/facdrone/shells/decision_evidence_store.py::seal_decision_evidence::223` |  | 23 | 1 | 0 | 4 | 0.1926 | 0 | 3 | 132 |
| 11 | `src/facdrone/shells/decision_evidence_publication.py::slice_identity::147` |  | 5 | 1 | 0 | 0 | 0.0 | 0 | 1 | 25 |
| 12 | `packages/facdrone-reporting/facdrone_reporting/library/config.py::parse_reporting_config::716` |  | 21 | 0 | 0 | 0 | 0.0 | 0 | 1 | 185 |
| 13 | `src/facdrone/shells/decision_evidence_publication.py::deduplicated_inputs::151` |  | 1 | 1 | 0 | 0 | 0.0 | 0 | 1 | 26 |
| 14 | `src/facdrone/shells/decision_evidence_publication.py::payload_batches::405` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 12 |
| 15 | `src/facdrone/shells/decision_evidence_publication.py::stored_evidence_payload::430` |  | 2 | 0 | 0 | 0 | 0.0 | 0 | 1 | 26 |
| 16 | `packages/facdrone-reporting/facdrone_reporting/library/config.py::reporting_config_for_mandate::681` |  | 1 | 0 | 0 | 0 | 0.0625 | 0 | 2 | 31 |
| 17 | `src/facdrone/shells/wiring/composition.py::project_root::47` |  | 1 | 0 | 0 | 0 | 0.0 | 0 | 1 | 14 |
| 18 | `src/facdrone/shells/cppi/state_codec.py::_event_model::80` |  | 1 | 0 | 0 | 0 | 0.0 | 0 | 1 | 14 |
| 19 | `packages/facdrone-kernel/facdrone_kernel/domain/ledger.py::Reliability.__post_init__::39` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 6 |
| 20 | `packages/facdrone-kernel/facdrone_kernel/domain/ledger.py::StateReliability.__post_init__::69` |  | 0 | 0 | 0 | 0 | 0.0 | 0 | 1 | 6 |

File sets: `edges.json`, `responsibilities[].closure_files`.
Repeat-run differences: 0 entries.
