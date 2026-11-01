# Native capability comparison: scoped readiness

The local Rust library and `lesscode-core` CLI now prepare and compare supplied
Python and TypeScript capabilities. They measure complete replacements and admit
source-bound behavioral evidence. The core does not propose edits or execute
project tests. The existing discovery skill supplies the hypothesis and reviewed
scope; maintainers decide adoption. Analysis needs no hosted account, commercial
analyzer, Python/Node process, MCP service, solver or accelerator hardware.

Windows correctness and the exercised Python/TypeScript workflow are verified.
Final cross-platform readiness remains `MANUAL_VERIFY_REQUIRED`: latest Linux
and macOS matrix results are absent. This is scoped implementation evidence,
not a feature-wide GO or a general compression efficacy claim.

## Try the existing paired evidence

Build with the prerequisites in [native setup](../../native/README.md), then run
from the repository root. Use a fresh sibling output path on each invocation:

```powershell
cargo +1.98.1 build --locked --manifest-path native/Cargo.toml
$pair = 'docs/experiments/native-replay/ats-run/recorded'
native/target/debug/lesscode-core.exe compare --frozen "$pair/frozen.json" --baseline "$pair/baseline" --candidate "$pair/candidate" --evidence "$pair/candidate-evidence.json" --artifact-root $pair --out D:/Source/lesscode-first-native-result
Get-Content D:/Source/lesscode-first-native-result/report.md
```

This readmits retained ATS observations; it does not rerun them. The fresh
[readmission receipt](../evidence/native-core-readiness/receipt.json) records the
actual external output, accepted result and equal specimen hashes. Retained
[JSON](../evidence/native-core-readiness/ats-result.json) and
[Markdown](../evidence/native-core-readiness/ats-report.md) contain the result.
Use [the discovery recipe](../compression-procedure.md) to freeze a new manifest,
run unchanged checks externally on both copies and supply their artifacts.
`prepare --manifest MANIFEST --baseline ROOT --out FRESH.json` freezes the trial;
`compare` requires that freeze, both roots, evidence and explicit artifact roots.
Reports require new output outside the source roots. Exit codes are 0 for
prepared/accepted, 1 for rejected/not-demonstrated, 2 for incomplete and 3 for
invalid/baseline-ineligible/I/O failure. See [CLI help receipt](../evidence/native-core-readiness/help.log).

## What the trials establish

| Complete owned replacement | Native physical lines | Native statements | Outcome/adoption |
| --- | --- | --- | --- |
| [ATS Python reporting](../experiments/native-replay/ats-run/results.md) | 815 → 634 | 345 → 184 | Accepted; no upstream adoption |
| [NPU Python provider/helper](../experiments/native-replay/npu-run/results.md) | 799 → 729 | 264 → 246 | Accepted; no upstream adoption |
| [TypeScript degradation capability](../experiments/native-replay/typescript-run/results.md) | 400 → 381 | 93 → 82 | Accepted; no upstream adoption |
| [Lesscode Python contracts](../experiments/native-self-apply/trial/record.md) | 2342 → 2310 | 1599 → 1570 | Independently reviewed; exact policy-contract replacement adopted |

These are separate, manually selected scopes and targets. Native versioned
statement/decision definitions differ from historical Radon/Lizard measures.
Call layers remain unavailable where unresolved; reviewing ownership does not
resolve dynamic dispatch. ATS/NPU task 6.1 freezes with stricter call-layer targets
remain incomplete; later source-target trials are separately frozen results.
Known behavioral mutants, moved helpers, narrowed/reclassified scope, missing
checks and stale evidence are rejected or incomplete in the retained trials and
native fixtures. Failed preparation/candidate attempts remain in their records.
No pooled success rate, repository-wide shrinkage, maintenance savings, runtime
speedup or hardware result follows from these counts.

## Verification and maturity boundary

Fresh Windows checks on 2026-11-01 passed: locked nextest 0.9.143 ran 102 tests,
102 passed with no skips; doctests ran zero tests successfully; build, rustfmt
and clippy with warnings denied passed. The built executable reached usable
`--help` and accepted the real ATS pair. Raw [test log](../evidence/native-core-readiness/nextest.log),
[doctest log](../evidence/native-core-readiness/doctest.log),
[build log](../evidence/native-core-readiness/build.log),
[fmt log](../evidence/native-core-readiness/fmt.log) and
[clippy log](../evidence/native-core-readiness/clippy.log) retain command outcomes
with sibling exit-code files. [Parallel test instructions](../testing.md) retain
the separate doctest gate.

The self-trial completed after external fixtures. Its frozen isolated baseline
and candidate each collected the same 515 Python identities: 504 passed and 11
existing Windows/platform skips. Both passed 171 relevant checks and seven
independent methods with adverse controls. Explicit root/config/pythonpath and
source guards in every worker establish which implementation ran. The
[trial record](../experiments/native-self-apply/trial/record.md) links commands,
JUnit, failed earlier runs, source guards and provenance; collector exit zero
alone was never sufficient. Fresh live contract checks after adoption passed
70 tests and 104 subtests. The [byte comparison receipt](../evidence/native-core-readiness/receipt.json)
verifies all 96 live source/test files equal the admitted isolated candidate;
generated egg-info and bytecode are excluded explicitly. The full 515-test suite
was not repeated in the live checkout. Approved parallel-tooling/config changes
are separately enumerated in [integrity.json](../experiments/native-self-apply/trial/integrity.json).

Self-workflow execution is complete for the selected supported Python contracts
capability; demonstrated self-compression is 29 statements, 32 physical lines,
17 decisions and three functions removed. The target was at least 25 statements
with no line/decision/function increase. Only the independently approved file
with SHA-256 `65e939cc9e1d3acac3b0aed289d0283e9a160f28875a5f5a0d79cdb509a8dcfd`
was adopted. Rust self-analysis and TypeScript self-compression remain untested;
there is no whole-project maturity claim. A rejected beneficialness hypothesis
would retain baseline; no deletion quota is required. Missing evidence would keep
the corresponding maturity claim open.

Initial Windows and Debian WSL foundation builds passed earlier, but that does
not establish latest integrated Linux coverage. A later WSL build encountered
SIGBUS with its temporary disk at 99% usage; no reset or cleanup was performed
for this task. macOS was not executed. The prepared
[three-platform workflow](../../.github/workflows/native-core.yml) has not been
pushed or dispatched by this task; configuration is not passing CI evidence.
Run and retain current Linux/macOS matrix results before cross-platform GO.

## Implementation cost and trust limits

The [footprint receipt](../evidence/native-core-readiness/receipt.json) counts
6,149 physical Rust source lines including inline unit tests, separately from
973 integration-test lines. It cannot provide a production-only Rust total;
Rust parsing is outside the supported language scope. The new readiness verifier
is 69 research-support lines. The self-trial separately records 151 automated
check lines and 592 research-support lines in
[its footprint](../experiments/native-self-apply/trial/footprint.json), with
production/test/research categories in [whole accounting](../experiments/native-self-apply/trial/counts.json).
These are implementation costs, not specimen savings. Locks, dependency notices,
fixtures and reports add further maintained material. Counts do not establish
net repository reduction or effort saved.

Hashes establish consistency, not trusted execution or hostile-writer isolation.
Finite checks cover declared domains; external services, arbitrary runtime values
and universal equivalence are unproved. Source review, dependency knowledge and
adoption remain human/agent responsibilities. The native dependency direction
matches the [design](../../.kiro/specs/capability-compression-core/design.md):
model; scope/metrics/compare; library orchestration; thin CLI. The retained Python
CLI and unfinished rule-engine work are separate from native analysis.

The [32-criterion coverage map](../../.kiro/specs/capability-compression-core/requirement-coverage.md)
and [readiness gate](../../.kiro/specs/capability-compression-core/readiness.md)
record the precise validation boundary.
