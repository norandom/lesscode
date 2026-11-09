# Prototype gate: Java adapter

Status: **proceed**, 2026-11-09. Prototype feasibility gate executed and certified.
Cryptographic receipt: [results/prototype-gate-report.json](results/prototype-gate-report.json) (`207765c042ef02e263c6d19648c19532675dc9c52b95e6044a179d409e2b3410`).
See [R51](../../../docs/research/2026-11-09/09-java-adapter-prototype-gate.md), [design](design.md), and [tasks](tasks.md).

## Hypothesis and admission

A supported `java-junit-v1` profile preserves observed behavior while enabling
whole-responsibility compression on a real specimen, with counted runtime support.
Specimen admitted: `apache/commons-csv` at commit `2bd66bf181d44ffc92806f585d3955e915666f7f` (Apache License 2.0).
Target responsibility: `src/main/java/org/apache/commons/csv/CSVPrinter.java`.
Baseline tests: `org.apache.commons.csv.CSVPrinterTest` (168 tests, 0 failures, 0 errors, 7 skipped).
Whole-responsibility compression: 221 -> 136 canonical lines (−38.46%, 85 lines removed), 112 -> 81 statements (−27.68%, 31 statements removed).
Oracle sensitivity: 100% kill rate across all 4 mandatory adverse controls (`JAVA_ADV_1` through `JAVA_ADV_4`).

Before execution freeze: source/revision/license and dirty-tree hashes, complete
owned implementation/callers, baseline and candidate populations, evaluator/check
hashes, exact runtime/parser/build/formatter versions and locks, profile/schema,
entrypoints, frame/effect/domain manifests, normalizations, valid mandatory
controls and expected witnesses. Baseline tests must pass; missing source or
dependencies leaves the gate not-run. Do not select a specimen solely because
it is easy to parse. Select named code that could disappear and the replacement.

## Scope and execution bounds

At most two attempts, two working days per attempt. Per-run limits use shared
contract defaults: 10,000 cases; 60 seconds/case; 900 seconds/probe batch;
1 MiB/frame; 64 MiB output/batch; 1,800 seconds/build or regression stage.
A dated pre-run amendment is required to raise a limit. Use isolated artifacts
under D:/BuildArtifacts/lesscode/java-junit-v1; launch commands from the relevant repo.
Respect cache settings; no WSL development execution or drive-root jobs.

## Research checklist

1. Screen/admit the real responsibility and freeze all inputs/controls/metrics.
2. Demonstrate one real success and failure operation through the counted runtime
   adapter, including required state, output/flush, cleanup and source identities.
3. Challenge codec/finite-domain coverage and unsupported constructs. A sampled
   run cannot certify the required exhaustive DomainSpec.
4. Generate one complete replacement, run paired probes, mandatory controls and
   unchanged relevant regression suites; retain failures without narrowing tests.
5. Compare complete specimen and apparatus inventories; review every surviving
   control, metric gap and budget change before considering proceed.

## Targets, falsifiers and decision

| Dimension | Proceed condition | Otherwise |
|---|---|---|
| Behavior | Zero divergences across complete declared finite domain and sequences; matching reset/error/cleanup contracts | divergent or unverified; revise/stop |
| Oracle sensitivity | Every calibrated valid non-equivalent mandatory control killed with expected witness | survivors, invalid/unexercised controls or infrastructure errors block |
| Evidence | Correct identities, complete events/cases/tests and deterministic paired repeat | gaps block; no false completion |
| Compression | At least 20% canonical lines AND 10% statements, at least 50 canonical lines AND 10 statements removed from whole responsibility | useful reduction unproven; revise/stop |
| Accounting | Full helpers/callers/adapters/configuration counted; wider specimen does not export complexity | revise/stop |
| Apparatus | Existing plus added apparatus within unchanged R41 7,173 canonical-line ceiling and registered per-file/function caps | owner-reviewed amendment required before production, not automatic pass |
| Conformance | Shared adverse protocol/value fixtures plus unchanged existing language/CLI behavior | revise |
| Delivery | Pinned tests, runtime smoke and applicable existing native/Dagger platform checks before delivery claim | delivery remains unverified |

Measure specimen savings separately from apparatus growth. No invented savings
from moving support code, excluded strings or deleting checks. No implicit budget
allocation is granted. A ceiling amendment does not alter compression targets.
Report outcomes as proceed/revise/stop/not-run with evidence; only proceed and
ordinary design approval unlock production task generation.

## Evidence record

No raw result is available yet. Create dated research entries, reciprocal spec
links and index updates when execution begins. Preserve earlier attempts.
A synthetic control validates the checker, not the real-code compression claim.
