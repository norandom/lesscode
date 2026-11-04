# Lesscode

[![DOI](https://zenodo.org/badge/1440672292.svg)](https://zenodo.org/badge/latestdoi/1440672292)

The goal is substantial code simplification while preserving features. In grown
codebases, custom implementations, boilerplate and supporting layers can often
be replaced by capabilities already provided by the project's framework,
libraries or language. Lesscode aims to help identify those opportunities and
check the resulting changes.

The work is to move responsibility into suitable existing capabilities: use a
framework shorthand, reuse a library operation, or express the same behavior
with clearer idiomatic code. Success means less code and structure to maintain
for the same features and public contracts. Moving custom code into another
module, hiding it behind a wrapper, or dropping a feature does not establish
that result. A replacement's dependencies, configuration and behavioral
differences also count when judging whether it simplifies the project.

## What works today

The [local mmhmem/mvibe experiment](docs/experiments/local-adoption/README.md)
records separate production/test footprints, Cargo-installed Rust analysis,
reproduction data and a rejected standard-library replacement. These are field
baselines, with no new compression savings claimed.

The [subsequent real mvibe trial](docs/experiments/proxy-routing-trial/README.md)
removes **23 module lines and 16 statements**, with all **1,172 tests passing**
before and after. A [Facdrone CSV trial](docs/experiments/facdrone-csv-trial/README.md)
preserves 54 tested CSV artifacts but saves only three lines, so it is rejected
as marginal. These are isolated, test-validated candidates; neither used SMT
or changed the specimen checkout.

The local Rust library and `lesscode-core` CLI now compare supplied Python and
TypeScript capabilities. They count the complete replacement, including helpers
and changed callers, and admit frozen, source-bound behavioral evidence before
classifying compression. Native analysis uses pinned open-source dependencies;
no hosted account, commercial analyzer, MCP service or solver is required.

Start with the [native usage and scoped readiness report](docs/reports/native-core-readiness.md)
and [build instructions](native/README.md). You supply the original, candidate,
reviewed scope and externally executed checks. The core leaves specimens unchanged
and writes JSON plus Markdown to fresh output outside the source roots.

The exercised Windows workflow includes real ATS/NPU/TypeScript trials and a
reviewed Python self-application: 29 statements and 32 physical lines removed
across the complete two-module contracts capability. The exact approved policy
contracts file is adopted. These scoped results do not establish general discovery
efficacy, net repository shrinkage or reduced maintenance effort. Native checks
passed on Linux through Dagger and on native Windows/macOS runners; see the
[CI results and evidence](docs/ci-validation.md#rust-core-dagger-and-actions).

The existing `lesscode-baseline` Python evaluator remains runnable as a research
and reference implementation. Its commands and historical evidence below are
preserved. The native CLI is a separate command and schema, not a record migration.

## What you get

| Review question | Evidence |
| --- | --- |
| Did the declared behavior checks pass? | Baseline and candidate outcomes, expected check IDs and diagnostic logs |
| What structural changes occurred? | Native statement, decision and function counts, with unavailable or unresolved call layers disclosed |
| How much source changed? | Complete-capability physical and logical counts, with tests, support and dependency changes distinguished |
| Can someone inspect the result? | Versioned JSON, a readable summary, raw outputs and a hash-checked artifact index |
| Was the comparison held constant? | Source fingerprints, frozen profile, check inventory and recorded tool identities |

`accepted` means the candidate passed the frozen target and declared checks. It does not prove
equivalence or certify code quality. Missing required evidence prevents
acceptance; an observed behavior failure cannot be offset by fewer lines.

The Python reference evaluator additionally retains its historical Lizard/jscpd
and source-size fields. Those definitions are separate from native measurements;
bytes and compressed size are not current compression-benefit criteria.

## A measured example on Lesscode

The [reusable compression procedure](docs/compression-procedure.md) and repo-local
agent skill select complete responsibilities and candidate families before
choosing tools. They compare whole replacements and retain rejected attempts.

The [reader-family prototype](docs/experiments/reader-family/README.md) exercises
that loop on five real records: 1,126 comparisons and the same 103 contract tests
pass, but the full replacement saves only three lines. It remains an experiment.

The [outcome-hotspot trial](docs/experiments/outcome-hotspot/README.md) reduces a
72-line classifier to 60 lines, with CCN 54 to 45. It matches 20,258 differential
cases and 65 unchanged tests; Z3 verifies the extracted final-decision boundary.
The candidate remains experimental, with an SMT-augmented LLM pass planned next.

The [trial progress report](docs/reports/compression-trial-progress.md) separates
earlier results, the current candidate, proof boundaries and subsequent attempts.
The [continued trial](docs/experiments/outcome-llm/README.md) retains an unsafe
proposal and its verified revision: CCN reaches 43, with no additional line savings.

The [capability-guided pilot](docs/experiments/capability-guided/run-01/README.md)
ran one repository discovery pass and six local sessions with dependencies frozen.
Both arms found the same 12-line / 9-LLOC reuse candidate; this pilot did not show
additional verified removal from the shared inventory. All attempts are retained.

The [graph-guided trial](docs/experiments/report-graph/README.md) compares a local
rewrite with joint validation/parser changes. Both match 6,234 cases and pass 18
existing tests, but LLOC rises from 69 to 70 and 73 respectively. Neither is adopted.
The graph exposed data and error-order obligations, without a compression gain.

The [feature-path hypothesis experiment](docs/experiments/feature-path/README.md)
traces actual analyzer evidence through decision, reporting and recovery. Two
proposed representation collapses lose required behavior; both are rejected.
It establishes no additional compression and retains the counterexamples.

The [ATS bilingual reporting trial](docs/experiments/ats-report/README.md) selects
a genuinely repeated capability in a separate legacy codebase. Including all
translation data and wrappers, the candidate reduces 384 physical lines to 202
and 298 LLOC to 135, matching 8,820 frozen cases. Function count rises, but total
decision structure falls. This is a scoped experiment, with no upstream changes.

The [active CPU/NPU backend trial](docs/experiments/npu-backends/README.md) repeats
the mechanism: 171 to 101 owned physical lines, 68 to 47 LLOC, 3,168 matching
generated cases and 339 unchanged tests passing on both versions. The
[two-trial synthesis](docs/reports/capability-compression.md) records what worked,
all replacement costs and the remaining limits. No upstream adoption is claimed.

The [code-flow synthesis experiment](docs/reports/code-flow-synthesis.md)
compares two verified representations of synthetic integer functions. Direct
expressions reduced 66 physical lines to 22; a shared kernel and table used 35.
Sharing made an already-compact control larger. The report includes charts,
diffs, proof limits and the next step: Facdrone code measurements before any
Kiro specification changes.

The [follow-up measurements](docs/experiments/facdrone/selected-parts.md) select
Facdrone examples without changing them and identify Lesscode's record conversion
boundary as a substantial target. An initial standard-library substitution was
rejected by existing tests: less source is useful only when behavior survives.

The [Lesscode synthesis self-application](docs/reports/lesscode-self-application.md)
now inventories all 282 production function definitions and tests a real
classifier with CEGIS. Its table preserves the modeled behavior and passes the
same 44 tests, but grows from 12 to 17 lines. The report retains this rejection,
explicit exclusions and separate production/test statistics; substantial
real-project compression remains unproven.

The [Facdrone empirical roadmap](docs/experiments/facdrone/README.md) examines
a modularized quantitative-finance project and proposes responsibility-level
replacement trials, separate production/test accounting, and maintenance
exercises for humans and agents. It distinguishes inspected evidence from
unproven simplification hypotheses.

The [library replacement trial](docs/experiments/leaf-contracts/README.md) records
an empirical adoption decision: five library-backed record readers matched
5,404 differential cases, but saved only four physical lines including their
support code. That candidate was not adopted into the runtime. Production code,
test code and trial code are counted separately in its raw results.

We applied the workflow to this package and simplified the threshold evaluator:

![One function improved: cyclomatic complexity 13 to 9 and logical lines 27 to 21.](docs/images/self-evaluation-local.svg)

This is a local result. Across 13 product modules, LLOC changes from 4,132 to
4,126 and the highest CCN stays 56. Source shrinks by 98 bytes, while compressed
size grows by 5. Whole-package evaluation is incomplete because duplication
inventory could not be reconciled; a separate `decision.py` scope is accepted.
No maintenance-hours or runtime improvement is claimed.

The [worked example](docs/examples/self-evaluation.md) includes the code change,
remaining hotspots, fixed checks, raw evidence and reproduction commands.
The [measurement guide](docs/measurement-guide.md) explains convolution, naming,
idiom and maintenance effort beyond line counts.

## Try an unchanged pair first

Clone this repository, then follow [runtime setup](docs/runtime-setup.md) and
[analyzer setup](docs/analyzer-setup.md). Setup uses uv and provisions the pinned
tools explicitly. Evaluation never installs them.

From the repository root, compare the included licensed fixture with itself.
Use new output names if you repeat the example.

<details>
<summary>Windows PowerShell</summary>

```powershell
.\.venv\Scripts\lesscode-baseline.exe plan --profile benchmarks/profiles/python.json --baseline tests/fixtures/real/boltons_mathutils/source --candidate tests/fixtures/real/boltons_mathutils/source --tool-paths .tools/analyzers/entrypoints.json --out ../lesscode-first-plan.json
.\.venv\Scripts\lesscode-baseline.exe run --plan ../lesscode-first-plan.json --out ../lesscode-first-run
Get-Content ../lesscode-first-run/summary.md
```

</details>

<details>
<summary>Linux or macOS</summary>

```sh
.venv/bin/lesscode-baseline plan --profile benchmarks/profiles/python.json --baseline tests/fixtures/real/boltons_mathutils/source --candidate tests/fixtures/real/boltons_mathutils/source --tool-paths .tools/analyzers/entrypoints.json --out ../lesscode-first-plan.json
.venv/bin/lesscode-baseline run --plan ../lesscode-first-plan.json --out ../lesscode-first-run
cat ../lesscode-first-run/summary.md
```

</details>

With the pinned tools available, expect an accepted run, unchanged structural
measurements, 23 functions and maximum cyclomatic complexity 15. Open
`summary.md` for the decision and limitations; `report.json` and the indexed raw
artifacts contain the supporting records. Planning prints the commands; inspect
them before running checks.

The fixture profile contains checks specific to boltons. For your own code,
define the scope and independent checks before preparing the comparison. See
[profile configuration](docs/profiles.md). Start with one module and one change;
the [adoption guide](docs/adoption.md) gives a review process you can try without
making it a merge gate.

## Read the technical details

The [documentation index](docs/README.md) groups the guides by task.

The [dated research log](docs/research/README.md) tracks experiments, revised
hypotheses and the evidence behind the next tool/spec decision.

- [Native readiness and working example](docs/reports/native-core-readiness.md): current CLI, empirical outcomes and platform gaps.
- [All 32 native criteria](.kiro/specs/capability-compression-core/requirement-coverage.md): concrete artifact coverage.
- [Technical overview](docs/technical-overview.md): architecture, measurement definitions, decision precedence, storage and experiment design.
- [CLI reference](docs/cli.md): commands, optional evidence, installed-wheel usage and exit codes.
- [Profiles](docs/profiles.md): scope rules, check contracts, thresholds and evaluator ownership.
- [Validation evidence](docs/ci-validation.md): native Windows, Linux and macOS checks and retained artifacts.

The Python baseline completed 33 independently reviewed tasks and covers all 56
acceptance criteria. The [final validation report](.kiro/specs/efficiency-baseline/validation.md)
links the evidence. These fixtures validate the evaluator, not the effectiveness
of a simplification skill on arbitrary repositories.

## Where the project is going

The next proposed feature is configurable measured rules for coding agents:
limits, change budgets and regression ratchets with actionable violations.
The general configurable enforcement workflow is not implemented yet. This
repository already has scoped [AGENTS.md guidance](src/lesscode_baseline/AGENTS.md)
for role-revealing names, idiomatic code and justified abstractions. The current
CLI evaluates supplied changes under a fixed profile.

The discovery skill and native Python/TypeScript comparison workflow are now
implemented for the exercised scopes. General deterministic rewrites, Java
support and MCP integration are outside the active native delivery. The CLI runs independently
of a skill or MCP server. See the [roadmap](.kiro/steering/roadmap.md).

## License

Lesscode is licensed under the [MIT License](LICENSE). Third-party dependencies
and fixtures retain their own licenses and notices; see the
[native dependency inventory](native/LICENSES.md).

## Citation

Releases are archived on Zenodo; the DOI badge above resolves to the latest
archived release. Citation metadata is in [CITATION.cff](CITATION.cff).
