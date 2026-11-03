# E2 calibration

Population: 2275 entry responsibilities; 8405 tests, 236 failed or errored; production functions reached 94.7%.

| Prediction | Holds | Evidence |
| --- | --- | --- |
| C1 | yes | A write sites 3 (3 greater), churn 269 (63 greater); top decile = fewer than 228 greater |
| C2 | no | C write sites 1 vs median 0, churn 0 vs median 0 |
| C3 | yes | B churn 1332416 vs median 0 |
| C4 | yes | 310 responsibilities outside A, B, C and R11 in the top decile on two or more of the first five |
| C5 | no | call_edges 0.8875, hops_max 0.4099, write_sites 0.4248, churn_max 0.4089, dos 0.708, ordering 0.3735 |

## Cases as declared file sets

| Case | call_edges | hops_max | write_sites | churn_max | dos | ordering | lines | statements | static-only |
|---|---|---|---|---|---|---|---|---|---|
| A | 194 | 4 | 3 | 269 | 0.8392 | 174 | 2676 | 1252 | 2 |
| B | 178 | 3 | 1 | 1332416 | 0.7852 | 0 | 1860 | 880 | 8 |
| C | 97 | 0 | 1 | 0 | 0.6872 | 0 | 1629 | 604 | 11 |

Lines and statements above are those of executed callables; declared sizes (R15 basis): A {'files': 8, 'physical_lines': 2682, 'ast_statements': 1256}; B {'files': 6, 'physical_lines': 1938, 'ast_statements': 925}; C {'files': 5, 'physical_lines': 1865, 'ast_statements': 693}.

## Top twenty responsibilities

| # | Entry | top decile | call_edges | hops_max | write_sites | churn_max | dos | ordering | files | lines |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `src/facdrone/shells/wiring/alpha_root.py::build_alpha_runner.<locals>._run::96` | call_edges, hops_max, write_sites, churn_max, dos | 1301 | 9 | 12 | 1107 | 0.9777 | 45 | 134 | 9611 |
| 2 | `src/facdrone/shells/cppi/app.py::_decision_routes.<locals>.post_decision::88` | call_edges, hops_max, write_sites, churn_max, dos | 2042 | 14 | 3 | 783 | 0.9871 | 555 | 204 | 13444 |
| 3 | `packages/facdrone-reporting/facdrone_reporting/shells/artifact_publisher.py::AtomicArtifactPublisher.publish::123` | call_edges, hops_max, write_sites, churn_max, dos | 784 | 6 | 3 | 1318091 | 0.8976 | 6 | 21 | 4123 |
| 4 | `src/facdrone/shells/serving/decision_request.py::decide_request::253` | call_edges, hops_max, write_sites, churn_max, dos | 1183 | 9 | 3 | 339 | 0.9723 | 4 | 113 | 7536 |
| 5 | `src/facdrone/shells/sim/driver.py::run_simulation::601` | call_edges, hops_max, write_sites, churn_max, dos | 1206 | 7 | 15 | 297 | 0.9666 | 4 | 108 | 7486 |
| 6 | `packages/facdrone-reporting/facdrone_reporting/pipelines/run_report.py::DefaultRunReportBuilder.build::72` | call_edges, hops_max, write_sites, churn_max, dos | 1269 | 5 | 1 | 35377 | 0.9626 | 5 | 50 | 9837 |
| 7 | `packages/facdrone-reporting/facdrone_reporting/shells/xlsx_renderer.py::XlsxArtifactRenderer.render::27` | call_edges, hops_max, write_sites, churn_max, dos | 720 | 5 | 3 | 1332417 | 0.8768 | 0 | 20 | 3701 |
| 8 | `src/facdrone/shells/cppi/app.py::_decision_routes.<locals>.post_report::101` | call_edges, hops_max, write_sites, churn_max, dos | 875 | 8 | 3 | 180 | 0.9771 | 160 | 94 | 5052 |
| 9 | `packages/facdrone-dash/facdrone_dash/app.py::_register_overview_callback.<locals>.render_overview::107` | call_edges, hops_max, write_sites, churn_max, dos | 1162 | 5 | 1 | 4281 | 0.943 | 10 | 44 | 6991 |
| 10 | `src/facdrone/shells/cli/pod_dispatch.py::_config_dispatcher.<locals>.dispatch::78` | call_edges, hops_max, write_sites, churn_max, dos | 450 | 6 | 2 | 472 | 0.914 | 3 | 70 | 3144 |
| 11 | `packages/facdrone-client/facdrone_client/ctrader/cycle_stage.py::run_client_cycle::54` | call_edges, hops_max, write_sites, churn_max, dos | 375 | 8 | 2 | 204 | 0.9457 | 12 | 40 | 2096 |
| 12 | `src/facdrone/shells/wiring/alpha_root.py::assemble_alpha_deps::103` | call_edges, hops_max, write_sites, churn_max, dos | 680 | 4 | 1 | 2104 | 0.957 | 4 | 78 | 3799 |
| 13 | `packages/facdrone-client/facdrone_client/ctrader/root.py::run_cycle::184` | call_edges, hops_max, write_sites, churn_max, dos | 443 | 9 | 2 | 106 | 0.9561 | 36 | 49 | 2360 |
| 14 | `src/facdrone/shells/serving/mandate_guard.py::report_outcome::148` | call_edges, hops_max, write_sites, churn_max, dos | 406 | 6 | 2 | 127 | 0.9668 | 0 | 53 | 2317 |
| 15 | `packages/facdrone-client/facdrone_client/ctrader/loop.py::serve::77` | call_edges, hops_max, write_sites, churn_max, dos | 292 | 8 | 2 | 202 | 0.95 | 14 | 43 | 1614 |
| 16 | `src/facdrone/shells/cli/config_group.py::config_overview::20` | call_edges, hops_max, write_sites, churn_max, dos | 508 | 5 | 1 | 1214 | 0.9376 | 6 | 58 | 3000 |
| 17 | `src/facdrone/shells/cppi/binding.py::bind_cppi::46` | call_edges, hops_max, write_sites, churn_max, dos | 355 | 8 | 1 | 2000 | 0.9072 | 14 | 52 | 1957 |
| 18 | `packages/facdrone-reporting/facdrone_reporting/commands/app.py::config_metrics::219` | call_edges, hops_max, write_sites, churn_max, dos | 412 | 5 | 1 | 967 | 0.9445 | 6 | 58 | 2433 |
| 19 | `src/facdrone/shells/decision_core.py::decide_day::1408` | call_edges, hops_max, write_sites, churn_max, dos | 924 | 8 | 1 | 173 | 0.962 | 3 | 101 | 6477 |
| 20 | `packages/facdrone-client/facdrone_client/cycle.py::ClientCycle.run::128` | call_edges, hops_max, write_sites, churn_max, dos | 487 | 7 | 2 | 79 | 0.9475 | 7 | 39 | 2591 |

File sets: `edges.json`, `responsibilities[].closure_files`.
Repeat-run differences: 321 entries.
