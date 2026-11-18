# Corrective 2.3 · E5 solver experiment

Frozen 18 November 2026 at 09:13 Europe/Berlin, before the experiment. The owner approved an external Z3 process and a counted CPython invocation adapter for this prototype. Rust owns the domain, translation, query, verdict and fresh replay. The [brief](brief.md) retains the discovery-first order for the feature.

## Requirement and provisional design

Can Rust use the actual baseline and candidate source of E5’s pure `signal_standing` to find an input on which a changed guard affects the returned report? Use the existing E5 rewrite, which folded `_latest_basis` into the report builder. Manually select that responsibility. Translate only its row-retention predicate from both source versions. Use two distinct monitors and ordered windows so keeping the older row has an observable effect. Compare the complete JSON report by executing the real function in each arm.

| Frozen input | SHA-256 |
| --- | --- |
| Baseline lifecycle_projection.py | adb6075c061d48058a56b69655d0486bc6ea1f6a98e927774e0c3e596d94cea7 |
| E5 lifecycle_projection.py | 4eac76fa9b31d9f3953e0a1de8a79b49139ba64451dd67dafae6fd23973615d4 |
| signal_lifecycle.py in both arms | c8d7311f4495cf22b49f48f5b3d2078bb69d253a49f25d6f755b4eb41181f0c9 |
| Specimen CPython launcher | 602f83da0874772f7a9480e15029b8ff57e752f9e07f398a743df042a6b3ca09 |
| Z3 executable | 2bffeee10ba02a510d07be9c3fdf9c22f10ccc9724dc84201c5a40196f1a1e8f |

Use two nonempty reading dictionaries, older then newest, each with disposition `within`. For each row independently choose one of three nonempty digest strings and one of three nonempty revision strings. Enumerate all 81 assignments. Fix the unrecorded standing, UTC moment, overdue interval, monitor identities, and absent baseline and flag. Compare the older window's membership with the modeled predicate for all three arms. Digest and revision are separate string families.

In a copy of the candidate change only the revision read in the exclusion guard from the current row to the newest row. Require Z3 to find a difference before using any observed function outputs. Request the model only after SAT and use it unchanged to invoke the baseline, clean candidate and planted copy in fresh specimen processes.

## Decision criteria

Require UNSAT for the clean guard comparison, SAT for the planted comparison, zero clean whole-report differences, agreement of every modeled membership with its observed report, and a fresh replay where the baseline and clean candidate agree and the planted report includes the stale window. Reject missing inputs or outputs, unsupported guard syntax, solver errors, unknown, timeout, or an unreproduced witness. Limit each solver call to ten seconds and each specimen process to sixty seconds. Preserve generated queries, raw solver output, inputs, outputs, source hashes and tool versions.

This experiment measures verification feasibility. It proposes no additional source removal (target 0 lines and 0% incremental reduction). Count the adapter and research apparatus separately. A successful result qualifies this bounded check for further research. Feature-wide discovery, reduction targets, independent evaluation and production approval remain separate gates. Record the outcome in the [research index](../../../docs/research/README.md).

## Outcome · 18 November

Proceed with bounded-check research on the demonstrated filter. Both clean functions returned identical reports on every assignment. Z3 identified a witness for the planted revision error, confirmed in fresh executions of all three arms. The state-consolidation feature gate awaits the discovery and compression experiments. [R139, evidence and replay](../../../docs/research/2026-11-18/05-e5-rust-z3-check.md)

## Discovery on the prepared job machine · preregistration

Recorded 18 November at 09:35 Europe/Berlin. Use the source and tests pinned by [R130](../../../docs/research/2026-11-17/17-hidden-state-machine-fitness.md), manifest SHA-256 `52c9fe089d40ae6a6b60f9f53e8a90541a2f4ea1b64baf22d1fffd1cdb05604f`. Read only the supplied package's Python modules during each extraction. Keep the sibling implementation, tests and oracle outside that input. The investigator has seen the fixture. The tool receives no expected names or transition map.

Require a source-backed entity and field, initial value, three states, both event guards and destinations, and accepted and refused outcomes. Resolve the two mixins across imports. Distinguish independently written rules from a single table with distributed handlers. Verify a renamed version to challenge dependence on fixture names. Refuse unsupported handler effects or unresolved ownership. A count of assignments cannot pass.

On the two separately extracted machines, compare initial state, returned acceptance and state after each event, for the two events and all histories of length zero through four (31). Use the pinned external Z3 from R139, at most ten seconds per query, five length queries per pair and one shortest-witness query when necessary. Replace the table’s finish precondition with the idle state in a copy. Require clean agreement, a solver witness for the changed table and replay of that history through the frozen specimen test in both clean and changed trees. Run the original oracle suite with pytest-xdist. Store source identities, extracted reports, raw queries, solver responses and test results. Any failed criterion limits the conclusion.

This calibration uses a synthetic machine with a known answer. It has no rewrite or reduction target (zero additional lines removed, zero percent). A pass qualifies the recognizer for a separate real-code discovery experiment. A successful recovery on this fixture alone cannot unlock production or consolidation.

The calibration passed on 18 November. The Rust report recovered both guarded moves with source lines and correctly identified the separately supplied table. The 31 clean histories agreed. The shortest planted witness, `finish` on a fresh job, reproduced through the frozen specimen test. Proceed to a separate discovery experiment on the real stale-worker case. [R140 and replay](../../../docs/research/2026-11-18/06-rust-job-machine-discovery.md)

## Real Rust driver · preregistration, 18 November at 09:55 Europe/Berlin

Can a Rust-driven source review recover a real implicit protocol and predict a distinguishing history? Give the configured model only the seven complete pre-fix files used in R131, from `60f79f34880065134c198039923f0fdfd21ea3fd`. They are `run_command.rs`, `runner.rs`, `workspace.rs`, `store.rs`, `config.rs`, `process.rs` under the driver, and `native/tests/driver_run.rs`. Separately supply the four R130 explicit-table modules with the same neutral instructions. The evaluator selected the subsystem knowing the historical case. Exclude history, the fix, later tests, research notes and expected answers from the discovery inputs. Freeze each first response before opening the reference fix or later regression.

Use the released Rust `lesscode-model call`. Qualify the configured endpoint on an unrelated long catalogue first, with three deterministic retrievals and a normal finish. Target at least 18,313 reported input tokens and sufficient room for the real packet. Use the repository’s configured endpoint first and the configured workstation endpoints in default, Mistral, OpenAI order if qualification fails. Stop on the first qualified endpoint. Allow one real and one control answer, up to 4,000 output tokens each, 300 seconds per answer and 30 seconds of refusal wait. Do not repair or switch endpoints in response to answer quality.

Require an entity and cross-file guards and effects with checkable excerpts, a prediction that an edited test changes stored-outcome eligibility while an existing worker may execute old test bytes, and an observable difference for a reused versus fresh worker. The control should locate the explicit table without labelling distributed callers as scattered rules. Check any consequential source assertion, freeze the scores, then test the predicted histories against the historical driver and its fix. If transport fails or the prediction misses, record any behavioral replay as independent evaluator evidence. A passing discovery check qualifies this case only. No rewrite or compression target (0 lines and 0% incremental reduction) is included.

At 10:00 the first trial closed. The real arm used 4,000 of 4,000 output tokens for reasoning and returned zero text. The control identified the central table but omitted required history evidence IDs. The preregistered pair did not pass. Before opening the fix, a separate diagnostic tests whether output capacity alone explains the missing real report. Send the same frozen real request once with only its output limit raised to 16,000, using the same endpoint, reasoning level, timeout, citation and semantic criteria. Send no feedback, prior answer or additional source. Retain the first result and score this diagnostic separately. Stop after this call regardless of outcome.

The diagnostic also returned zero visible characters. The independent historical CLI replay verified the changed-test identity, stale worker bytes and a wrong surviving mutant before the fix. The fixed and fresh workers both killed it. This result calls for a revision to the source discovery method. General discovery and consolidation remain open. [R141](../../../docs/research/2026-11-18/07-real-rust-worker-discovery.md)

## Native worker trace · preregistration, 18 November 2026

On `60f79f3` and `aa9d7d7`, can Rust connect the project's test bytes and resulting identity, worker creation or reopening, successful file copies, and the directory supplied to the test process? This is a known-case experiment. Use syntax-tree probes on disposable historical sources. Retain original source hashes, exact byte ranges, lines and quotations. Keep the actual mutation driver and historical test command in the loop. One worker, the single tracked test file and R141's four histories bound the trial.

Predict the child-visible test hash by carrying the last observed source bytes through successful copies to the path addressed by the test command. Use only driver events for this prediction. Then compare with observations from the child directory, historical outcomes and an independently executed uninstrumented replay. A missing dependency or an unreadable trace must yield an error. On an entirely reused result, report that no test command ran if the trace shows none.

Require stale bytes only for the edited test in the pre-fix reopened worker, current bytes for the corresponding fixed and fresh workers, and agreement for every observed test command. As an adverse control disable the refresh with a short-circuiting false guard in the fixed driver, keeping the call and helper present. Require the old failure to reappear. Remove the necessary copy evidence from a trace and require the predictor to refuse it. This checks empirical path reconstruction on the exercised histories. Attribution to unexecuted paths, independent discovery, consolidation and production remain open. No rewrite or incremental reduction target (zero lines and zero percent).

On 18 November the known-case trace passed. The 69 independent child observations matched the reconstructed file paths and hashes. All seven launches in the changed, reopened worker saw stale bytes before the fix. The fixed worker refreshed all seven, and disabling refresh restored all seven stale reads. All five mutation results were reused in each unchanged run, which still executed two baseline checks. Both damaged traces were rejected. Proceed to a bounded prediction test with the changed histories withheld. The feature-level discovery and consolidation gate remains open. [R142 and replay](../../../docs/research/2026-11-18/08-native-worker-byte-trace.md)
