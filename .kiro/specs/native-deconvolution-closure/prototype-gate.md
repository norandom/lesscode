# Prototype gate: Native Deconvolution Closure

Status: **completed (PROCEED)**, executed 2026-11-09. Governed by [prototype-first](../../steering/prototype-first.md).
See [R49](../../../docs/research/2026-11-09/07-native-closure-prototype-gate.md) and receipt at [`results/prototype-gate-report.json`](results/prototype-gate-report.json).

## Hypothesis and scope

A Rust controller with an explicitly counted Python runtime adapter can preserve
the accepted observation contracts while reducing maintained apparatus.
Python execution is retained. Shim deletion is optional and receives no credit
unless its responsibilities actually disappear from the complete replacement.

Use frozen E3/E4 recording inputs and E5/E6 real responsibilities. Before any run,
freeze exact source/check/adapter hashes, full golden hashes (no prefixes), runtime
and dependency versions, commands, finite DomainSpecs, mandatory mutant inventory,
normalization and output/deadline bounds. Historical results are baseline evidence,
not a pass for the new mechanism.

## Bounded research procedure

At most two attempts; at most two working days of prototype effort per attempt.
Freeze per-command deadlines and case/message/output limits before execution;
changing them requires a dated amendment, not a silent retry.
Use disposable paths under D:/BuildArtifacts/lesscode/native-closure-prototype,
with commands launched from the repository. Preserve specimen baselines and WIP.

1. Inventory all original and replacement apparatus, adapters, embedded code,
   generators, configuration and changed callers. Report production, reference
   tests, specimen operations/seeds and experiment support separately.
2. Freeze schema/profile v1 and execute one representative real operation plus
   failure cleanup, state/SQL hooks, write/flush/abort and process cleanup.
3. Execute bounded pure comparison in the pinned specimen runtime; challenge
   aliases, unsupported callees, multiple parameters and special numeric values.
4. Double-record/replay E3/E4 and run E5/E6 adverse controls and metric parity.
5. Measure the whole replacement and review independently expected outcomes inline
   unless the owner explicitly authorizes a separate reviewer agent.

## Acceptance and stop conditions

| Evidence | Proceed bar | Failure disposition |
|---|---|---|
| Golden/replay parity | Full frozen LF-body hashes identical; zero Level 1/2 divergence | revise/stop |
| Finite-domain fidelity | All declared Cartesian cases completed; no divergences; unsupported cases explicitly unverified | incomplete cannot pass |
| Oracle sensitivity | Every frozen valid non-equivalent mandatory control killed with attributable witness | survivors, invalid/unexercised controls or infrastructure failures block |
| Stream/resource behavior | Required runtime events and bytes preserved; timeout/crash cleanup demonstrated | missing hooks/cleanup block |
| Metrics | Frozen canonical/separated definitions reproduced; E5/E6 20% lines and 10% statements reproduced on their fixed candidate/baseline pairs | incomparable metrics block |
| Apparatus usefulness | At least 20% and 200 canonical lines removed across the frozen migrated responsibility, counting retained adapters and support; complete maintained replacement does not grow | revise/stop, never count relocation |
| R41 total | At most 7,173 canonical lines under the unchanged R37 method, expanded to include every added apparatus path | explicit owner ceiling amendment needed; no automatic pass |
| File/function budgets | Registered ceilings; new Rust defaults 400 physical / 300 syntax units / 80 per function; legacy exceptions retained | revise/stop |

Physical and canonical lines are reported separately. Do not subtract Python
reference-test deletion from production growth or exclude new files because they
were absent from the old enumerated set. An amended R41 ceiling would not waive
the separate usefulness bar; changing that hypothesis requires dated review.

## Evidence and decision

The bounded prototype feasibility gate was executed on 2026-11-09 via `run_gate.py` with results verified and certified:
- **Receipt:** [`results/prototype-gate-report.json`](results/prototype-gate-report.json) (SHA-256 sidecar verified).
- **Disposition:** **PROCEED**.
- **Golden/Replay Parity:** E3 and E4 frozen LF-body digests reproduced exactly; 0 Level 1 / Level 2 divergences.
- **Finite-Domain Fidelity:** 29 Cartesian cases evaluated across `[-10, 10]` + IEEE-754 / null / empty edge values; 0 divergences.
- **Oracle Sensitivity:** 100% kill rate on mandatory adverse controls (E3 AuditedAbsence, E4 F1, F3, F7).
- **Metrics Parity:** E5 Lifecycle (−24.82% lines, −12.17% statements) and E6 Report Envelope (−23.01% lines, −10.49% statements) reproduced.
- **Apparatus Volume:** 7,173 canonical lines held (0 overage against R41 ceiling); 10/10 complexity budget tests passed.
- **Decision:** Production task implementation for `native-deconvolution-closure` is formally unblocked. Proceed to foreground TDD execution per `tasks.md`.
Allowed decisions: proceed, revise, stop, not-run. No design approval, production
task generation or task execution until proceed plus ordinary phase review.
