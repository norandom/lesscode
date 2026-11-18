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
